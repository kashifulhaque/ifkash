//! Shared Exa client for the radar, ask, site search, and workout research notes.
//!
//! Every call to `api.exa.ai` goes through [`call`], which enforces a daily spending
//! cap and records what Exa reports it charged. Public endpoints also call
//! [`within_rate`] before a live call and use [`cache_get`] and [`cache_put`] so a
//! repeated question costs nothing. The key is the `EXA_API_KEY` secret.

use serde::Deserialize;
use serde_json::Value;
use wasm_bindgen::JsValue;
use worker::*;

const EXA_BASE: &str = "https://api.exa.ai";
const D1: &str = "IFKASH_D1";

/// Daily spending cap in dollars when `EXA_DAILY_CAP_USD` is unset.
const DEFAULT_DAILY_CAP_USD: f64 = 2.0;

/// The domains that "ask" and site search are limited to. `ifkash.dev` also matches
/// its subdomains, such as the blog.
pub const OWN_DOMAINS: [&str; 3] = ["ifkash.dev", "blog.ifkash.dev", "github.com/kashifulhaque"];

/// Why a call didn't reach Exa or didn't succeed.
pub enum ExaFail {
    NoKey,
    CapReached,
    Upstream(u16, String),
    Internal(String),
}

impl ExaFail {
    /// The response a public endpoint returns for this failure.
    pub fn response(&self) -> Result<Response> {
        match self {
            ExaFail::NoKey => {
                console_error!("EXA_API_KEY secret missing");
                json_error("search isn't configured", 500)
            }
            ExaFail::CapReached => json_error("search is resting until tomorrow (daily limit reached)", 503),
            ExaFail::Upstream(status, body) => {
                console_error!("Exa returned {status}: {body}");
                json_error("the search provider returned an error", 502)
            }
            ExaFail::Internal(e) => {
                console_error!("Exa call failed: {e}");
                json_error("search failed", 502)
            }
        }
    }
}

impl From<Error> for ExaFail {
    fn from(e: Error) -> Self {
        ExaFail::Internal(e.to_string())
    }
}

pub fn json_error(msg: &str, status: u16) -> Result<Response> {
    Ok(Response::from_json(&serde_json::json!({ "error": msg }))?.with_status(status))
}

pub fn n(v: i64) -> JsValue {
    (v as f64).into()
}

pub fn now_secs() -> i64 {
    (Date::now().as_millis() / 1000) as i64
}

fn today_utc() -> String {
    chrono::DateTime::from_timestamp(now_secs(), 0)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn api_key(env: &Env) -> Option<String> {
    env.secret("EXA_API_KEY")
        .map(|s| s.to_string())
        .or_else(|_| env.var("EXA_API_KEY").map(|v| v.to_string()))
        .ok()
        .filter(|k| !k.trim().is_empty())
}

fn daily_cap_micros(env: &Env) -> i64 {
    let usd = env
        .var("EXA_DAILY_CAP_USD")
        .ok()
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .filter(|v| *v >= 0.0)
        .unwrap_or(DEFAULT_DAILY_CAP_USD);
    (usd * 1_000_000.0) as i64
}

/// Millionths of a dollar spent today, as Exa reported it.
async fn spent_today(d1: &D1Database) -> Result<i64> {
    #[derive(Deserialize)]
    struct Row {
        micros: i64,
    }
    let row: Option<Row> = d1
        .prepare("SELECT micros FROM exa_spend WHERE day = ?1")
        .bind(&[today_utc().into()])?
        .first(None)
        .await?;
    Ok(row.map(|r| r.micros).unwrap_or(0))
}

async fn record_spend(d1: &D1Database, micros: i64) -> Result<()> {
    d1.prepare(
        "INSERT INTO exa_spend (day, micros, calls) VALUES (?1, ?2, 1) \
         ON CONFLICT(day) DO UPDATE SET micros = micros + ?2, calls = calls + 1",
    )
    .bind(&[today_utc().into(), n(micros)])?
    .run()
    .await?;
    Ok(())
}

/// POST `body` to an Exa endpoint such as `/search`. Refuses once today's spend
/// reaches the cap, and records the cost Exa reports in `costDollars.total`.
pub async fn call(env: &Env, path: &str, body: &Value) -> std::result::Result<Value, ExaFail> {
    let key = api_key(env).ok_or(ExaFail::NoKey)?;
    let d1 = env.d1(D1)?;
    if spent_today(&d1).await? >= daily_cap_micros(env) {
        return Err(ExaFail::CapReached);
    }

    let headers = Headers::new();
    headers.set("x-api-key", &key)?;
    headers.set("content-type", "application/json")?;
    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(JsValue::from_str(&body.to_string())));
    let req = Request::new_with_init(&format!("{EXA_BASE}{path}"), &init)?;
    let mut resp = Fetch::Request(req).send().await?;
    let status = resp.status_code();
    let text = resp.text().await?;
    if status >= 400 {
        return Err(ExaFail::Upstream(status, text.chars().take(300).collect()));
    }
    let json: Value = serde_json::from_str(&text).map_err(|e| ExaFail::Internal(e.to_string()))?;

    // Fall back to a typical search price when Exa omits the cost.
    let cost = json
        .pointer("/costDollars/total")
        .and_then(Value::as_f64)
        .unwrap_or(0.007);
    if let Err(e) = record_spend(&d1, (cost * 1_000_000.0).round() as i64).await {
        console_error!("could not record Exa spend: {e}");
    }
    if let Some(w) = json.get("warnings") {
        console_warn!("Exa warnings on {path}: {w}");
    }
    Ok(json)
}

/// The client address, used only as a short-lived rate-limit key.
pub fn client_ip(req: &Request) -> String {
    req.headers()
        .get("cf-connecting-ip")
        .ok()
        .flatten()
        .unwrap_or_else(|| "local".to_string())
}

/// Count one live call against `key` in the current hour. True while within `limit`.
pub async fn within_rate(env: &Env, key: &str, limit: i64) -> Result<bool> {
    #[derive(Deserialize)]
    struct Row {
        count: i64,
    }
    let window = now_secs() / 3600;
    let row: Option<Row> = env
        .d1(D1)?
        .prepare(
            "INSERT INTO exa_rate (key, window, count) VALUES (?1, ?2, 1) \
             ON CONFLICT(key) DO UPDATE SET \
               count = CASE WHEN window = ?2 THEN count + 1 ELSE 1 END, window = ?2 \
             RETURNING count",
        )
        .bind(&[key.into(), n(window)])?
        .first(None)
        .await?;
    Ok(row.map(|r| r.count <= limit).unwrap_or(true))
}

/// Lowercase, trim, and collapse whitespace so near-identical queries share a cache row.
pub fn normalize(q: &str) -> String {
    q.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
        .chars()
        .take(300)
        .collect()
}

pub async fn cache_get(env: &Env, key: &str) -> Option<Value> {
    #[derive(Deserialize)]
    struct Row {
        body: String,
    }
    let d1 = env.d1(D1).ok()?;
    let row: Option<Row> = d1
        .prepare("SELECT body FROM exa_cache WHERE key = ?1 AND expires > ?2")
        .bind(&[key.into(), n(now_secs())])
        .ok()?
        .first(None)
        .await
        .ok()?;
    row.and_then(|r| serde_json::from_str(&r.body).ok())
}

/// Store `value` for `ttl_secs`, and clear expired rows. A failure never fails the request.
pub async fn cache_put(env: &Env, key: &str, value: &Value, ttl_secs: i64) {
    let Ok(d1) = env.d1(D1) else { return };
    let now = now_secs();
    let put = d1
        .prepare(
            "INSERT INTO exa_cache (key, body, expires) VALUES (?1, ?2, ?3) \
             ON CONFLICT(key) DO UPDATE SET body = ?2, expires = ?3",
        )
        .bind(&[key.into(), value.to_string().into(), n(now + ttl_secs)]);
    let sweep = d1.prepare("DELETE FROM exa_cache WHERE expires < ?1").bind(&[n(now)]);
    if let (Ok(put), Ok(sweep)) = (put, sweep) {
        if let Err(e) = d1.batch(vec![put, sweep]).await {
            console_error!("could not cache Exa result: {e}");
        }
    }
}

/// Sources an answer cites: the grounding citations, or the results when there are none.
pub fn answer_sources(json: &Value) -> Vec<Value> {
    let pick = |v: &Value| {
        let url = v.get("url").and_then(Value::as_str)?;
        Some((url.to_string(), v.get("title").and_then(Value::as_str).unwrap_or("").to_string()))
    };
    let mut found: Vec<(String, String)> = json
        .pointer("/output/grounding")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|g| g.get("citations").and_then(Value::as_array).into_iter().flatten())
        .filter_map(pick)
        .collect();
    if found.is_empty() {
        found = json
            .get("results")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .take(4)
            .filter_map(pick)
            .collect();
    }
    let mut seen = std::collections::HashSet::new();
    found
        .into_iter()
        .filter(|(url, _)| seen.insert(url.clone()))
        .map(|(url, title)| serde_json::json!({ "url": url, "title": title }))
        .collect()
}

/// The text Exa synthesized for a search that set `outputSchema: {type: "text"}`.
pub fn answer_text(json: &Value) -> String {
    json.pointer("/output/content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}
