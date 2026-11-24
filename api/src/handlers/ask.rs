//! Ask my portfolio (`POST /api/ask`) and site search (`GET /api/search`).
//!
//! Both are public and limited to Kashif's own domains (`exa::OWN_DOMAINS`). Answers
//! are cached for a week and search results for a day, so repeated questions are
//! free. Live calls are rate limited per client address.

use super::exa::{self, ExaFail};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;
use worker::*;

const ASK_TTL_SECS: i64 = 7 * 24 * 3600;
const SEARCH_TTL_SECS: i64 = 24 * 3600;
const ASK_PER_HOUR: i64 = 20;
const SEARCH_PER_HOUR: i64 = 60;
const MAX_QUESTION: usize = 300;

#[derive(Deserialize, ToSchema)]
pub struct AskRequest {
    /// A question about Kashif or his work, 3 to 300 characters.
    pub question: String,
}

#[derive(Serialize, ToSchema)]
pub struct Source {
    pub url: String,
    pub title: String,
}

#[derive(Serialize, ToSchema)]
pub struct AskResponse {
    /// A short answer with numbered citation markers such as `[1]`.
    pub answer: String,
    /// The pages the answer cites, in marker order.
    pub sources: Vec<Source>,
    pub cached: bool,
}

#[derive(Serialize, ToSchema)]
pub struct SearchHit {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub published: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct SearchResponse {
    pub results: Vec<SearchHit>,
    pub cached: bool,
}

const ASK_INSTRUCTIONS: &str = "Answer in at most 70 words (2 to 4 sentences, one paragraph) about Kashiful Haque (Kashif), \
the ML engineer who owns ifkash.dev. Write in the third person and use only these sources. \
If the sources don't answer the question, say that the site doesn't cover it.";

fn with_cached(mut v: Value, cached: bool) -> Value {
    v["cached"] = Value::Bool(cached);
    v
}

/// POST /api/ask — answer a question about Kashif from his own pages.
#[utoipa::path(post, path = "/api/ask", tag = "Exa", request_body = AskRequest,
    responses((status = 200, description = "Answer with sources", body = AskResponse),
        (status = 400, description = "Bad question"), (status = 429, description = "Rate limited"),
        (status = 503, description = "Daily search budget used up")))]
pub async fn ask(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: AskRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => return exa::json_error("send {\"question\": \"...\"}", 400),
    };
    let question = body.question.trim();
    if question.chars().count() < 3 || question.chars().count() > MAX_QUESTION {
        return exa::json_error("ask a question between 3 and 300 characters", 400);
    }
    let key = format!("ask2:{}", exa::normalize(question));
    if let Some(hit) = exa::cache_get(&ctx.env, &key).await {
        return Response::from_json(&with_cached(hit, true));
    }
    let ip = exa::client_ip(&req);
    if !exa::within_rate(&ctx.env, &format!("ask:{ip}"), ASK_PER_HOUR).await? {
        return exa::json_error("too many questions; try again in an hour", 429);
    }

    let res = exa::call(
        &ctx.env,
        "/search",
        &json!({
            "query": question,
            "type": "auto",
            "numResults": 6,
            "includeDomains": exa::OWN_DOMAINS,
            "contents": { "highlights": { "maxCharacters": 400 } },
            "outputSchema": { "type": "text", "description": ASK_INSTRUCTIONS },
        }),
    )
    .await;
    let json = match res {
        Ok(j) => j,
        Err(e) => return e.response(),
    };
    let answer = exa::answer_text(&json);
    if answer.is_empty() {
        return ExaFail::Internal("empty answer".into()).response();
    }
    let out = json!({ "answer": answer, "sources": exa::answer_sources(&json) });
    exa::cache_put(&ctx.env, &key, &out, ASK_TTL_SECS).await;
    Response::from_json(&with_cached(out, false))
}

/// GET /api/search?q= — search Kashif's site, blog, and GitHub.
#[utoipa::path(get, path = "/api/search", tag = "Exa",
    params(("q" = String, Query, description = "Search text, 2 to 300 characters")),
    responses((status = 200, description = "Matching pages", body = SearchResponse),
        (status = 400, description = "Bad query"), (status = 429, description = "Rate limited"),
        (status = 503, description = "Daily search budget used up")))]
pub async fn search(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;
    let q = url
        .query_pairs()
        .find(|(k, _)| k == "q")
        .map(|(_, v)| v.trim().to_string())
        .unwrap_or_default();
    if q.chars().count() < 2 || q.chars().count() > MAX_QUESTION {
        return exa::json_error("search for 2 to 300 characters", 400);
    }
    let key = format!("search:{}", exa::normalize(&q));
    if let Some(hit) = exa::cache_get(&ctx.env, &key).await {
        return Response::from_json(&with_cached(hit, true));
    }
    let ip = exa::client_ip(&req);
    if !exa::within_rate(&ctx.env, &format!("search:{ip}"), SEARCH_PER_HOUR).await? {
        return exa::json_error("too many searches; try again in an hour", 429);
    }

    let res = exa::call(
        &ctx.env,
        "/search",
        &json!({
            "query": q,
            "type": "auto",
            "numResults": 8,
            "includeDomains": exa::OWN_DOMAINS,
            "contents": { "highlights": { "maxCharacters": 220 } },
        }),
    )
    .await;
    let json = match res {
        Ok(j) => j,
        Err(e) => return e.response(),
    };
    let results: Vec<Value> = json
        .get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| {
            let url = r.get("url")?.as_str()?;
            let title = r.get("title").and_then(Value::as_str).unwrap_or(url);
            let snippet = r
                .pointer("/highlights/0")
                .and_then(Value::as_str)
                .unwrap_or("")
                .split_whitespace()
                // Highlights are Markdown; drop heading and list markers.
                .filter(|w| !w.chars().all(|c| matches!(c, '#' | '*' | '-' | '>' | '|')))
                .collect::<Vec<_>>()
                .join(" ")
                .trim_start_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            Some(json!({
                "url": url,
                "title": title.trim(),
                "snippet": snippet,
                "published": r.get("publishedDate").and_then(Value::as_str),
            }))
        })
        .collect();
    let out = json!({ "results": results });
    exa::cache_put(&ctx.env, &key, &out, SEARCH_TTL_SECS).await;
    Response::from_json(&with_cached(out, false))
}
