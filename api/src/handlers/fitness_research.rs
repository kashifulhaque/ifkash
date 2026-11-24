//! Workout research notes (`POST /api/fitness/research`).
//!
//! The insights page reads the user's training log, picks up to three signals (a
//! stalled lift, a muscle under its weekly set target, the rate of weight change, a
//! long break, or low cardio), and sends them here. Each signal maps to a fixed
//! question, so this endpoint is not a general search proxy. Exa answers it from a
//! short list of evidence-based sources. Answers are cached for 30 days, because the
//! same signal from any user asks the same question.

use super::exa;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;
use worker::*;

const TTL_SECS: i64 = 30 * 24 * 3600;
const PER_HOUR: i64 = 30;
const MAX_SIGNALS: usize = 3;

const EVIDENCE_DOMAINS: [&str; 8] = [
    "pubmed.ncbi.nlm.nih.gov",
    "pmc.ncbi.nlm.nih.gov",
    "strongerbyscience.com",
    "examine.com",
    "jissn.biomedcentral.com",
    "sportsmedicine-open.springeropen.com",
    "frontiersin.org",
    "bjsm.bmj.com",
];

const MUSCLES: [&str; 10] = [
    "chest", "back", "shoulders", "biceps", "triceps", "quads", "hamstrings", "glutes", "calves", "core",
];

const INSTRUCTIONS: &str = "At most 60 words, in 2 or 3 sentences: practical, evidence-based \
advice for a recreational lifter. Start with the single most useful change to make. One \
paragraph, no lists. Use only these sources.";

#[derive(Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Signal {
    /// A lift whose best set hasn't improved for several sessions.
    Stall { exercise: String },
    /// A muscle getting fewer weekly sets than the plan asks for.
    LowVolume { muscle: String },
    /// The 4-week rate band from the bodyweight trend.
    WeightRate { band: String },
    /// Days since the last session.
    Break { days: u32 },
    /// Weekly cardio well under the plan.
    LowCardio,
}

#[derive(Deserialize, ToSchema)]
pub struct ResearchRequest {
    pub signals: Vec<Signal>,
}

#[derive(Serialize, ToSchema)]
pub struct ResearchNote {
    /// The question asked, in plain words.
    pub question: String,
    pub answer: String,
    pub sources: Vec<super::ask::Source>,
}

fn clean_exercise(name: &str) -> Option<String> {
    let name = name.trim();
    let ok = (2..=40).contains(&name.chars().count())
        && name.chars().all(|c| c.is_alphanumeric() || " -'()".contains(c));
    ok.then(|| name.to_lowercase())
}

/// The fixed question for a signal, or None when the signal is malformed or needs none.
fn question(signal: &Signal) -> Option<String> {
    Some(match signal {
        Signal::Stall { exercise } => {
            format!("How to break a strength plateau on the {}", clean_exercise(exercise)?)
        }
        Signal::LowVolume { muscle } => {
            let m = MUSCLES.iter().find(|m| **m == muscle.as_str())?;
            format!("How many weekly sets do the {m} need to grow, and how to add them")
        }
        Signal::WeightRate { band } => match band.as_str() {
            "aggressive" | "very fast" => {
                "How to keep muscle and strength while losing weight quickly".to_string()
            }
            "slow" | "holding" => "How to restart fat loss after a weight-loss plateau".to_string(),
            "gaining" => "How fast to gain weight on a lean bulk to limit fat gain".to_string(),
            _ => return None,
        },
        Signal::Break { days } if *days >= 7 => {
            "How to return to lifting after a break of one to three weeks".to_string()
        }
        Signal::Break { .. } => return None,
        Signal::LowCardio => {
            "How much cardio to do alongside strength training without hurting gains".to_string()
        }
    })
}

/// POST /api/fitness/research — research-backed notes for up to three training signals.
#[utoipa::path(post, path = "/api/fitness/research", tag = "Exa", request_body = ResearchRequest,
    responses((status = 200, description = "One note per usable signal", body = [ResearchNote]),
        (status = 400, description = "Bad signals"), (status = 429, description = "Rate limited")))]
pub async fn research(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: ResearchRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => return exa::json_error("send {\"signals\": [...]}", 400),
    };
    let mut questions: Vec<String> = body.signals.iter().filter_map(question).collect();
    questions.dedup();
    questions.truncate(MAX_SIGNALS);
    if questions.is_empty() {
        return Response::from_json(&Vec::<Value>::new());
    }

    let ip = exa::client_ip(&req);
    // Cached notes first; the rest are asked in parallel, each counted against the limit.
    let mut slots: Vec<Option<Value>> = Vec::new();
    let mut misses = Vec::new();
    for q in questions {
        let key = format!("fit2:{}", exa::normalize(&q));
        match exa::cache_get(&ctx.env, &key).await {
            Some(hit) => slots.push(Some(hit)),
            None if exa::within_rate(&ctx.env, &format!("fit:{ip}"), PER_HOUR).await? => {
                misses.push((slots.len(), key, q));
                slots.push(None);
            }
            None => {}
        }
    }
    let env = &ctx.env;
    let asked = futures::future::join_all(misses.into_iter().map(|(slot, key, q)| async move {
        let res = exa::call(
            env,
            "/search",
            &json!({
                "query": q,
                "type": "auto",
                "numResults": 5,
                "includeDomains": EVIDENCE_DOMAINS,
                "contents": { "highlights": { "maxCharacters": 300 } },
                "outputSchema": { "type": "text", "description": INSTRUCTIONS },
            }),
        )
        .await;
        let json = match res {
            Ok(j) => j,
            // A failed question is left out; the others still return.
            Err(e) => {
                let _ = e.response();
                return (slot, None);
            }
        };
        let answer = exa::answer_text(&json);
        if answer.is_empty() {
            return (slot, None);
        }
        let note = json!({ "question": q, "answer": answer, "sources": exa::answer_sources(&json) });
        exa::cache_put(env, &key, &note, TTL_SECS).await;
        (slot, Some(note))
    }))
    .await;
    for (slot, note) in asked {
        slots[slot] = note;
    }
    let notes: Vec<Value> = slots.into_iter().flatten().collect();
    Response::from_json(&notes)
}
