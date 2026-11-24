//! Research radar: new papers on post-training, RL, inference, and agents.
//!
//! A daily cron (`api/wrangler.toml`) runs one Exa search per topic for papers published
//! in the last week and stores them in `radar_items`. `GET /api/radar` only reads that
//! table, so visitors never trigger a paid call. If the cron hasn't run for longer than
//! `STALE_SECS` (a fresh database or a local Worker), the first read refreshes inline.

use super::exa::{self, n, now_secs};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;
use worker::*;

const STALE_SECS: i64 = 30 * 3600;
const LOOKBACK_DAYS: i64 = 7;
const RETAIN_DAYS: i64 = 60;
const PER_TOPIC: u32 = 8;
const MAX_ITEMS: i64 = 120;

/// (id, label, search query)
const TOPICS: [(&str, &str, &str); 4] = [
    (
        "post-training",
        "Post-training",
        "new methods for reinforcement learning and preference optimization in post-training of large language models",
    ),
    (
        "rl",
        "Reinforcement learning",
        "new reinforcement learning algorithms, reward modeling, and credit assignment research",
    ),
    (
        "inference",
        "Inference",
        "faster and cheaper LLM inference: serving systems, quantization, speculative decoding, KV cache",
    ),
    (
        "agents",
        "Agents",
        "LLM agents trained with reinforcement learning, tool use, and agent evaluation benchmarks",
    ),
];

const SUMMARY_PROMPT: &str = "In at most 25 words, say what is new in this paper and why it matters.";

/// One URL per arXiv paper: `/html/` and `/pdf/` become `/abs/`, and the version suffix goes.
fn canonical_url(url: &str) -> String {
    let Some(rest) = url
        .strip_prefix("https://arxiv.org/")
        .or_else(|| url.strip_prefix("http://arxiv.org/"))
    else {
        return url.to_string();
    };
    let id = rest.splitn(2, '/').nth(1).unwrap_or(rest).trim_end_matches(".pdf");
    let id = match id.rfind('v') {
        Some(i) if i > 0 && id[i + 1..].chars().all(|c| c.is_ascii_digit()) && i + 1 < id.len() => &id[..i],
        _ => id,
    };
    format!("https://arxiv.org/abs/{id}")
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RadarItem {
    pub url: String,
    pub topic: String,
    pub title: String,
    pub authors: String,
    pub published: String,
    pub summary: String,
    pub found: i64,
}

#[derive(Serialize, ToSchema)]
pub struct RadarTopic {
    pub id: String,
    pub label: String,
}

#[derive(Serialize, ToSchema)]
pub struct RadarResponse {
    /// Unix seconds of the last refresh, or 0 before the first one.
    pub updated: i64,
    pub topics: Vec<RadarTopic>,
    /// Newest first.
    pub items: Vec<RadarItem>,
}

async fn last_refresh(d1: &D1Database) -> Result<i64> {
    #[derive(Deserialize)]
    struct Row {
        value: i64,
    }
    let row: Option<Row> = d1
        .prepare("SELECT value FROM exa_meta WHERE key = 'radar_refresh'")
        .first(None)
        .await?;
    Ok(row.map(|r| r.value).unwrap_or(0))
}

fn iso_days_ago(days: i64) -> String {
    chrono::DateTime::from_timestamp(now_secs() - days * 86_400, 0)
        .map(|d| d.format("%Y-%m-%dT00:00:00.000Z").to_string())
        .unwrap_or_default()
}

/// Search every topic and store new papers. Claims the refresh clock first, so
/// overlapping callers don't all pay for the same searches.
pub async fn refresh(env: &Env) -> Result<usize> {
    let d1 = env.d1("IFKASH_D1")?;
    let now = now_secs();
    d1.prepare(
        "INSERT INTO exa_meta (key, value) VALUES ('radar_refresh', ?1) \
         ON CONFLICT(key) DO UPDATE SET value = ?1",
    )
    .bind(&[n(now)])?
    .run()
    .await?;

    let since = iso_days_ago(LOOKBACK_DAYS);
    let searches = TOPICS.iter().map(|(id, _, query)| {
        let body = json!({
            "query": query,
            "type": "auto",
            "category": "research paper",
            // Journals and other publishers drifted off topic in testing; arXiv didn't.
            "includeDomains": ["arxiv.org"],
            "numResults": PER_TOPIC,
            "startPublishedDate": since,
            "contents": { "summary": { "query": SUMMARY_PROMPT } },
        });
        async move { (*id, exa::call(env, "/search", &body).await) }
    });

    let mut stmts = Vec::new();
    for (topic, res) in join_all(searches).await {
        let json = match res {
            Ok(j) => j,
            Err(e) => {
                // Log and keep the other topics.
                let _ = e.response();
                continue;
            }
        };
        for r in json.get("results").and_then(Value::as_array).into_iter().flatten() {
            let (Some(url), Some(title)) = (
                r.get("url").and_then(Value::as_str),
                r.get("title").and_then(Value::as_str),
            ) else {
                continue;
            };
            let s = |k: &str| r.get(k).and_then(Value::as_str).unwrap_or("").trim().to_string();
            let published: String = s("publishedDate").chars().take(10).collect();
            stmts.push(
                d1.prepare(
                    "INSERT INTO radar_items (url, topic, title, authors, published, summary, found) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) ON CONFLICT(url) DO NOTHING",
                )
                .bind(&[
                    canonical_url(url).into(),
                    topic.into(),
                    title.trim().into(),
                    s("author").into(),
                    published.into(),
                    s("summary").into(),
                    n(now),
                ])?,
            );
        }
    }
    let stored = stmts.len();
    stmts.push(
        d1.prepare("DELETE FROM radar_items WHERE found < ?1")
            .bind(&[n(now - RETAIN_DAYS * 86_400)])?,
    );
    stmts.push(d1.prepare("DELETE FROM exa_rate WHERE window < ?1").bind(&[n(now / 3600 - 2)])?);
    d1.batch(stmts).await?;
    Ok(stored)
}

/// GET /api/radar — the latest papers the radar found.
#[utoipa::path(get, path = "/api/radar", tag = "Exa",
    responses((status = 200, description = "Recent papers by topic", body = RadarResponse)))]
pub async fn list(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.env.d1("IFKASH_D1")?;
    if now_secs() - last_refresh(&d1).await? > STALE_SECS {
        if let Err(e) = refresh(&ctx.env).await {
            console_error!("radar refresh failed: {e}");
        }
    }
    let items: Vec<RadarItem> = d1
        .prepare(
            "SELECT url, topic, title, authors, published, summary, found FROM radar_items \
             ORDER BY published DESC, found DESC LIMIT ?1",
        )
        .bind(&[n(MAX_ITEMS)])?
        .all()
        .await?
        .results()?;
    let topics = TOPICS
        .iter()
        .map(|(id, label, _)| RadarTopic { id: id.to_string(), label: label.to_string() })
        .collect();
    let mut resp = Response::from_json(&RadarResponse {
        updated: last_refresh(&d1).await?,
        topics,
        items,
    })?;
    resp.headers_mut().set("Cache-Control", "public, max-age=900")?;
    Ok(resp)
}
