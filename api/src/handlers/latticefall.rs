//! Cinderheart (latticefall) online features: Messages, Afterimages, Wanderers, cloud
//! saves, Mark sync, the speedrun leaderboard, and opt-in telemetry.
//!
//! Reads are anonymous and edge-cacheable. Writes need a bearer token (a Google ID token
//! or a session minted from one) and are rate limited per user. Telemetry is anonymous
//! and rate limited per install and per client address. Records older than 90 days are
//! deleted lazily by whichever write arrives first after the hourly cleanup is due, so no
//! Cron Trigger is needed. Spec: docs/game/06-progression-and-systems.md.

use super::access::is_local_dev;
use super::google_auth::{identity_from_bearer, GoogleIdentity};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::*;

const RETENTION_SECS: i64 = 90 * 24 * 60 * 60;
const CLEANUP_EVERY_SECS: i64 = 60 * 60;
/// Messages rated at or below this are hidden from reads.
const HIDE_RATING: i64 = -3;
const MAX_MESSAGES_SHOWN: u32 = 40;
/// The spec shows at most 20 Afterimages per area.
const MAX_AFTERIMAGES_SHOWN: u32 = 20;
const MAX_WANDERERS_SHOWN: u32 = 8;
const AFTERIMAGE_TICKS: usize = 150;
const WANDERER_POINTS: usize = 120;
/// Fixed lists of the message system (the client owns the strings, the server the bounds).
const MAX_TEMPLATE_ID: i64 = 63;
const MAX_WORD_ID: i64 = 511;
const MAX_WORDS: usize = 2;
const MAX_SAVE_B64: usize = 600_000;
const MAX_MARKS_PER_PUT: usize = 128;
const MAX_TELEMETRY_BODY: usize = 48_000;
const MAX_TELEMETRY_EVENTS: usize = 300;
const MIN_SPEEDRUN_MS: i64 = 60_000;
const MAX_SPEEDRUN_MS: i64 = 86_400_000;
const LEADERBOARD_SIZE: u32 = 20;

/// Hourly write limits per user (telemetry: per install and per client address).
const LIMIT_MESSAGES: i64 = 12;
const LIMIT_VOTES: i64 = 120;
const LIMIT_AFTERIMAGES: i64 = 30;
const LIMIT_WANDERERS: i64 = 12;
const LIMIT_SAVES: i64 = 60;
const LIMIT_MARKS: i64 = 60;
const LIMIT_SPEEDRUN: i64 = 20;
const LIMIT_TELEMETRY_ANON: i64 = 12;
const LIMIT_TELEMETRY_IP: i64 = 120;

const TELEMETRY_KINDS: &[&str] = &[
    "session_start",
    "session_end",
    "area_entered",
    "lantern_lit",
    "death",
    "boss_attempt",
    "difficulty",
    "link_click",
];

fn n(v: i64) -> wasm_bindgen::JsValue {
    (v as f64).into()
}

fn now_secs() -> i64 {
    (Date::now().as_millis() / 1000) as i64
}

fn err(msg: &str, status: u16) -> Result<Response> {
    Response::error(msg, status)
}

fn cached(mut resp: Response, secs: u32) -> Result<Response> {
    resp.headers_mut()
        .set("Cache-Control", &format!("public, max-age={secs}"))?;
    Ok(resp)
}

/// Area IDs, anonymous IDs and Mark IDs: short identifiers.
fn valid_id(s: &str, min: usize, max: usize) -> bool {
    s.len() >= min
        && s.len() <= max
        && s
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_' || c == b'.')
}

fn valid_coord(v: f64) -> bool {
    v.is_finite() && v.abs() <= 100_000.0
}

fn valid_pos(x: f64, y: f64, z: f64) -> bool {
    valid_coord(x) && valid_coord(y) && valid_coord(z)
}

/// Resolve the signed-in user and upsert them into `game_users`. Returns the user's row
/// id, or a ready 401. A missing token is always a 401, including under `LOCAL_DEV`,
/// where any non-empty token maps to a fixed local identity.
async fn resolve_owner(
    req: &Request,
    ctx: &RouteContext<()>,
) -> Result<std::result::Result<i64, Response>> {
    let auth = req.headers().get("Authorization")?.unwrap_or_default();
    let token = auth.strip_prefix("Bearer ").unwrap_or(auth.as_str()).trim();
    if token.is_empty() {
        return Ok(Err(Response::error("unauthorized", 401)?));
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    let info = if is_local_dev(ctx) {
        GoogleIdentity {
            aud: String::new(),
            iss: String::new(),
            sub: "local-dev-user".to_string(),
            email: Some("dev@localhost".to_string()),
            name: Some("Local Dev".to_string()),
            picture: None,
        }
    } else {
        let client_id = ctx.env.var("GOOGLE_CLIENT_ID")?.to_string();
        match identity_from_bearer(token, &client_id, &d1).await {
            Ok(i) => i,
            Err(resp) => return Ok(Err(resp)),
        }
    };
    d1.prepare(
        "INSERT INTO game_users (google_sub, email, name, picture) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(google_sub) DO UPDATE SET email = ?2, name = ?3, picture = ?4",
    )
    .bind(&[
        info.sub.clone().into(),
        info.email.unwrap_or_default().into(),
        info.name.unwrap_or_else(|| "anonymous".to_string()).into(),
        info.picture.unwrap_or_default().into(),
    ])?
    .run()
    .await?;

    #[derive(Deserialize)]
    struct IdRow {
        id: i64,
    }
    let row: Option<IdRow> = d1
        .prepare("SELECT id FROM game_users WHERE google_sub = ?1")
        .bind(&[info.sub.into()])?
        .first(None)
        .await?;
    match row {
        Some(r) => Ok(Ok(r.id)),
        None => Ok(Err(Response::error("could not resolve user", 500)?)),
    }
}

/// Count one write against `key` in the current hour. Returns true when the call is
/// within `limit`.
async fn within_rate(d1: &D1Database, key: &str, limit: i64) -> Result<bool> {
    #[derive(Deserialize)]
    struct CountRow {
        count: i64,
    }
    let window = now_secs() / 3600;
    let row: Option<CountRow> = d1
        .prepare(
            "INSERT INTO lf_rate (key, window, count) VALUES (?1, ?2, 1) \
             ON CONFLICT(key) DO UPDATE SET \
               count = CASE WHEN window = ?2 THEN count + 1 ELSE 1 END, window = ?2 \
             RETURNING count",
        )
        .bind(&[key.into(), n(window)])?
        .first(None)
        .await?;
    Ok(row.map(|r| r.count <= limit).unwrap_or(true))
}

/// Delete records older than 90 days, at most once an hour. Called after writes; a
/// failure never fails the write.
async fn maybe_cleanup(d1: &D1Database) {
    let _ = run_cleanup(d1).await;
}

async fn run_cleanup(d1: &D1Database) -> Result<()> {
    #[derive(Deserialize)]
    struct Meta {
        value: i64,
    }
    let now = now_secs();
    let last: Option<Meta> = d1
        .prepare("SELECT value FROM lf_meta WHERE key = 'last_cleanup'")
        .first(None)
        .await?;
    if last
        .map(|m| now - m.value < CLEANUP_EVERY_SECS)
        .unwrap_or(false)
    {
        return Ok(());
    }
    // Claim the clock first so concurrent writes don't all run the cleanup.
    d1.prepare(
        "INSERT INTO lf_meta (key, value) VALUES ('last_cleanup', ?1) \
         ON CONFLICT(key) DO UPDATE SET value = ?1",
    )
    .bind(&[n(now)])?
    .run()
    .await?;
    let cutoff = now - RETENTION_SECS;
    let stale =
        |sql: &str, v: i64| -> Result<D1PreparedStatement> { d1.prepare(sql).bind(&[n(v)]) };
    d1.batch(vec![
        stale(
            "DELETE FROM lf_message_votes WHERE message_id IN \
             (SELECT id FROM lf_messages WHERE created < ?1)",
            cutoff,
        )?,
        stale("DELETE FROM lf_messages WHERE created < ?1", cutoff)?,
        stale("DELETE FROM lf_afterimages WHERE created < ?1", cutoff)?,
        stale("DELETE FROM lf_wanderers WHERE created < ?1", cutoff)?,
        stale("DELETE FROM lf_telemetry WHERE created < ?1", cutoff)?,
        stale("DELETE FROM lf_rate WHERE window < ?1", now / 3600 - 2)?,
    ])
    .await?;
    Ok(())
}

fn area_param(req: &Request) -> Option<String> {
    let url = req.url().ok()?;
    let area = url
        .query_pairs()
        .find(|(k, _)| k == "area")
        .map(|(_, v)| v.to_string())?;
    valid_id(&area, 1, 48).then_some(area)
}

/// A JSON body of bounded size, or a ready 400/413.
async fn read_body<T: for<'de> Deserialize<'de>>(
    req: &mut Request,
    max: usize,
) -> std::result::Result<T, Response> {
    let text = req
        .text()
        .await
        .map_err(|_| Response::error("bad request", 400).unwrap())?;
    if text.len() > max {
        return Err(Response::error("payload too large", 413).unwrap());
    }
    serde_json::from_str(&text).map_err(|_| Response::error("bad request", 400).unwrap())
}

/// Validate a list of `[x, y, z, moveId]` entries.
fn valid_ticks(ticks: &[Vec<f64>], max: usize) -> bool {
    !ticks.is_empty()
        && ticks.len() <= max
        && ticks.iter().all(|t| {
            t.len() == 4
                && valid_pos(t[0], t[1], t[2])
                && t[3].is_finite()
                && t[3] >= 0.0
                && t[3] <= 65_535.0
        })
}

#[derive(Deserialize)]
struct IdRow {
    id: i64,
}

// ---------------------------------------------------------------- Messages

#[derive(Deserialize)]
struct MessageRow {
    id: i64,
    area: String,
    x: f64,
    y: f64,
    z: f64,
    template_id: i64,
    word_ids: String,
    rating: i64,
}

/// GET /api/latticefall/messages?area=ID — visible messages for an area. Anonymous.
#[utoipa::path(get, path = "/api/latticefall/messages", tag = "Cinderheart",
    params(("area" = String, Query, description = "Area ID")),
    responses((status = 200, description = "Messages for the area")))]
pub async fn list_messages(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(area) = area_param(&req) else {
        return err("bad area", 400);
    };
    let d1 = ctx.d1("IFKASH_D1")?;
    let rows: Vec<MessageRow> = d1
        .prepare(
            "SELECT id, area, x, y, z, template_id, word_ids, rating FROM lf_messages \
             WHERE area = ?1 AND rating > ?2 ORDER BY rating DESC, id DESC LIMIT ?3",
        )
        .bind(&[area.into(), n(HIDE_RATING), n(MAX_MESSAGES_SHOWN as i64)])?
        .all()
        .await?
        .results()?;
    let out: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id, "area": r.area, "x": r.x, "y": r.y, "z": r.z,
                "template_id": r.template_id,
                "word_ids": serde_json::from_str::<Vec<i64>>(&r.word_ids).unwrap_or_default(),
                "rating": r.rating,
            })
        })
        .collect();
    cached(
        Response::from_json(&serde_json::json!({ "messages": out }))?,
        60,
    )
}

#[derive(Deserialize)]
struct PostMessage {
    anon_id: String,
    area: String,
    x: f64,
    y: f64,
    z: f64,
    template_id: i64,
    word_ids: Vec<i64>,
}

/// POST /api/latticefall/messages — leave a message built from a template and words.
#[utoipa::path(post, path = "/api/latticefall/messages", tag = "Cinderheart",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Message id"), (status = 401, description = "Not signed in"),
        (status = 429, description = "Rate limited")))]
pub async fn post_message(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner = match resolve_owner(&req, &ctx).await? {
        Ok(o) => o,
        Err(r) => return Ok(r),
    };
    let b: PostMessage = match read_body(&mut req, 2_000).await {
        Ok(b) => b,
        Err(r) => return Ok(r),
    };
    if !valid_id(&b.area, 1, 48)
        || !valid_id(&b.anon_id, 8, 64)
        || !valid_pos(b.x, b.y, b.z)
        || !(0..=MAX_TEMPLATE_ID).contains(&b.template_id)
        || b.word_ids.len() > MAX_WORDS
        || b.word_ids.iter().any(|w| !(0..=MAX_WORD_ID).contains(w))
    {
        return err("bad message", 400);
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    if !within_rate(&d1, &format!("msg:{owner}"), LIMIT_MESSAGES).await? {
        return err("rate limited", 429);
    }
    let words = serde_json::to_string(&b.word_ids).unwrap_or_else(|_| "[]".into());
    let row: Option<IdRow> = d1
        .prepare(
            "INSERT INTO lf_messages (area, x, y, z, template_id, word_ids, anon_id, user_id, created) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) RETURNING id",
        )
        .bind(&[
            b.area.into(),
            b.x.into(),
            b.y.into(),
            b.z.into(),
            n(b.template_id),
            words.into(),
            b.anon_id.into(),
            n(owner),
            n(now_secs()),
        ])?
        .first(None)
        .await?;
    maybe_cleanup(&d1).await;
    Response::from_json(&serde_json::json!({ "id": row.map(|r| r.id) }))
}

#[derive(Deserialize)]
struct RateBody {
    value: i64,
}

/// POST /api/latticefall/messages/:id/rate — rate a message +1 or -1 (one vote per user,
/// changeable). Messages whose rating falls to -3 stop showing.
#[utoipa::path(post, path = "/api/latticefall/messages/{id}/rate", tag = "Cinderheart",
    security(("bearer_auth" = [])),
    params(("id" = i64, Path, description = "Message id")),
    responses((status = 200, description = "New rating"), (status = 401, description = "Not signed in"),
        (status = 404, description = "No such message")))]
pub async fn rate_message(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner = match resolve_owner(&req, &ctx).await? {
        Ok(o) => o,
        Err(r) => return Ok(r),
    };
    let Some(id) = ctx.param("id").and_then(|s| s.parse::<i64>().ok()) else {
        return err("bad id", 400);
    };
    let b: RateBody = match read_body(&mut req, 200).await {
        Ok(b) => b,
        Err(r) => return Ok(r),
    };
    if b.value != 1 && b.value != -1 {
        return err("bad rating", 400);
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    if !within_rate(&d1, &format!("vote:{owner}"), LIMIT_VOTES).await? {
        return err("rate limited", 429);
    }
    let exists: Option<IdRow> = d1
        .prepare("SELECT id FROM lf_messages WHERE id = ?1")
        .bind(&[n(id)])?
        .first(None)
        .await?;
    if exists.is_none() {
        return err("not found", 404);
    }
    // The vote and the recount run in one batch so two raters can't lose an update.
    d1.batch(vec![
        d1.prepare(
            "INSERT INTO lf_message_votes (message_id, user_id, value) VALUES (?1, ?2, ?3) \
             ON CONFLICT(message_id, user_id) DO UPDATE SET value = ?3",
        )
        .bind(&[n(id), n(owner), n(b.value)])?,
        d1.prepare(
            "UPDATE lf_messages SET rating = \
             (SELECT COALESCE(SUM(value), 0) FROM lf_message_votes WHERE message_id = ?1) \
             WHERE id = ?1",
        )
        .bind(&[n(id)])?,
    ])
    .await?;
    #[derive(Deserialize)]
    struct Rating {
        rating: i64,
    }
    let r: Option<Rating> = d1
        .prepare("SELECT rating FROM lf_messages WHERE id = ?1")
        .bind(&[n(id)])?
        .first(None)
        .await?;
    Response::from_json(&serde_json::json!({ "rating": r.map(|r| r.rating).unwrap_or(0) }))
}

// -------------------------------------------------------------- Afterimages

#[derive(Deserialize)]
struct AfterimageRow {
    id: i64,
    area: String,
    x: f64,
    y: f64,
    z: f64,
    cause: String,
    snippet: String,
}

/// GET /api/latticefall/afterimages?area=ID — a random sample of at most 20. Anonymous.
#[utoipa::path(get, path = "/api/latticefall/afterimages", tag = "Cinderheart",
    params(("area" = String, Query, description = "Area ID")),
    responses((status = 200, description = "Sampled Afterimages for the area")))]
pub async fn list_afterimages(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(area) = area_param(&req) else {
        return err("bad area", 400);
    };
    let d1 = ctx.d1("IFKASH_D1")?;
    let rows: Vec<AfterimageRow> = d1
        .prepare(
            "SELECT id, area, x, y, z, cause, snippet FROM lf_afterimages \
             WHERE area = ?1 ORDER BY RANDOM() LIMIT ?2",
        )
        .bind(&[area.into(), n(MAX_AFTERIMAGES_SHOWN as i64)])?
        .all()
        .await?
        .results()?;
    let out: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id, "area": r.area, "x": r.x, "y": r.y, "z": r.z, "cause": r.cause,
                "snippet": serde_json::from_str::<Value>(&r.snippet).unwrap_or(Value::Null),
            })
        })
        .collect();
    cached(
        Response::from_json(&serde_json::json!({ "afterimages": out }))?,
        30,
    )
}

#[derive(Deserialize)]
struct PostAfterimage {
    anon_id: String,
    area: String,
    x: f64,
    y: f64,
    z: f64,
    cause: String,
    /// `[x, y, z, moveId]` per tick, at most 150 ticks.
    snippet: Vec<Vec<f64>>,
}

/// POST /api/latticefall/afterimages — record where and how a player died.
#[utoipa::path(post, path = "/api/latticefall/afterimages", tag = "Cinderheart",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Afterimage id"), (status = 401, description = "Not signed in"),
        (status = 429, description = "Rate limited")))]
pub async fn post_afterimage(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner = match resolve_owner(&req, &ctx).await? {
        Ok(o) => o,
        Err(r) => return Ok(r),
    };
    let b: PostAfterimage = match read_body(&mut req, 16_000).await {
        Ok(b) => b,
        Err(r) => return Ok(r),
    };
    if !valid_id(&b.area, 1, 48)
        || !valid_id(&b.anon_id, 8, 64)
        || !valid_id(&b.cause, 1, 32)
        || !valid_pos(b.x, b.y, b.z)
        || !valid_ticks(&b.snippet, AFTERIMAGE_TICKS)
    {
        return err("bad afterimage", 400);
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    if !within_rate(&d1, &format!("after:{owner}"), LIMIT_AFTERIMAGES).await? {
        return err("rate limited", 429);
    }
    let snippet = serde_json::to_string(&b.snippet).unwrap_or_else(|_| "[]".into());
    let row: Option<IdRow> = d1
        .prepare(
            "INSERT INTO lf_afterimages (area, x, y, z, cause, snippet, anon_id, user_id, created) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) RETURNING id",
        )
        .bind(&[
            b.area.into(),
            b.x.into(),
            b.y.into(),
            b.z.into(),
            b.cause.into(),
            snippet.into(),
            b.anon_id.into(),
            n(owner),
            n(now_secs()),
        ])?
        .first(None)
        .await?;
    maybe_cleanup(&d1).await;
    Response::from_json(&serde_json::json!({ "id": row.map(|r| r.id) }))
}

// ---------------------------------------------------------------- Wanderers

/// GET /api/latticefall/wanderers?area=ID — a random sample of position trails. The
/// response carries no identity. Anonymous.
#[utoipa::path(get, path = "/api/latticefall/wanderers", tag = "Cinderheart",
    params(("area" = String, Query, description = "Area ID")),
    responses((status = 200, description = "Sampled trails for the area")))]
pub async fn list_wanderers(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let Some(area) = area_param(&req) else {
        return err("bad area", 400);
    };
    let d1 = ctx.d1("IFKASH_D1")?;
    #[derive(Deserialize)]
    struct Row {
        trail: String,
    }
    let rows: Vec<Row> = d1
        .prepare("SELECT trail FROM lf_wanderers WHERE area = ?1 ORDER BY RANDOM() LIMIT ?2")
        .bind(&[area.into(), n(MAX_WANDERERS_SHOWN as i64)])?
        .all()
        .await?
        .results()?;
    let out: Vec<Value> = rows
        .into_iter()
        .map(|r| serde_json::from_str::<Value>(&r.trail).unwrap_or(Value::Null))
        .collect();
    cached(
        Response::from_json(&serde_json::json!({ "wanderers": out }))?,
        30,
    )
}

#[derive(Deserialize)]
struct PostWanderer {
    anon_id: String,
    area: String,
    /// `[x, y, z, moveId]` per sample, at most 120 samples.
    trail: Vec<Vec<f64>>,
}

/// POST /api/latticefall/wanderers — upload a short trail of the player walking.
#[utoipa::path(post, path = "/api/latticefall/wanderers", tag = "Cinderheart",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Wanderer id"), (status = 401, description = "Not signed in"),
        (status = 429, description = "Rate limited")))]
pub async fn post_wanderer(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner = match resolve_owner(&req, &ctx).await? {
        Ok(o) => o,
        Err(r) => return Ok(r),
    };
    let b: PostWanderer = match read_body(&mut req, 12_000).await {
        Ok(b) => b,
        Err(r) => return Ok(r),
    };
    if !valid_id(&b.area, 1, 48)
        || !valid_id(&b.anon_id, 8, 64)
        || !valid_ticks(&b.trail, WANDERER_POINTS)
    {
        return err("bad wanderer", 400);
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    if !within_rate(&d1, &format!("wander:{owner}"), LIMIT_WANDERERS).await? {
        return err("rate limited", 429);
    }
    let trail = serde_json::to_string(&b.trail).unwrap_or_else(|_| "[]".into());
    let row: Option<IdRow> = d1
        .prepare(
            "INSERT INTO lf_wanderers (area, trail, anon_id, user_id, created) \
             VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id",
        )
        .bind(&[
            b.area.into(),
            trail.into(),
            b.anon_id.into(),
            n(owner),
            n(now_secs()),
        ])?
        .first(None)
        .await?;
    maybe_cleanup(&d1).await;
    Response::from_json(&serde_json::json!({ "id": row.map(|r| r.id) }))
}

// --------------------------------------------------------------- Cloud save

#[derive(Deserialize)]
struct PutSave {
    /// Base64 of the `LfWorld.save()` blob.
    data: String,
    scenario: String,
    /// The world's seed as a decimal string (it is a u64).
    seed: String,
    hash: String,
    /// Milliseconds since the epoch, from the client.
    saved_at: i64,
}

#[derive(Serialize, Deserialize)]
struct SaveRow {
    data: String,
    scenario: String,
    seed: String,
    hash: String,
    saved_at: i64,
}

/// PUT /api/latticefall/save — store the player's cloud save (latest wins).
#[utoipa::path(put, path = "/api/latticefall/save", tag = "Cinderheart",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Saved"), (status = 401, description = "Not signed in"),
        (status = 413, description = "Blob too large")))]
pub async fn put_save(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner = match resolve_owner(&req, &ctx).await? {
        Ok(o) => o,
        Err(r) => return Ok(r),
    };
    let b: PutSave = match read_body(&mut req, MAX_SAVE_B64 + 2_000).await {
        Ok(b) => b,
        Err(r) => return Ok(r),
    };
    let blob_ok = b.data.len() <= MAX_SAVE_B64
        && base64::engine::general_purpose::STANDARD
            .decode(b.data.as_bytes())
            .map(|v| !v.is_empty())
            .unwrap_or(false);
    if !blob_ok
        || !valid_id(&b.scenario, 1, 64)
        || b.seed.is_empty()
        || b.seed.len() > 20
        || !b.seed.bytes().all(|c| c.is_ascii_digit())
        || !valid_id(&b.hash, 1, 64)
        || b.saved_at < 0
    {
        return err("bad save", 400);
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    if !within_rate(&d1, &format!("save:{owner}"), LIMIT_SAVES).await? {
        return err("rate limited", 429);
    }
    d1.prepare(
        "INSERT INTO lf_saves (user_id, data, scenario, seed, hash, saved_at, updated) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
         ON CONFLICT(user_id) DO UPDATE SET data = ?2, scenario = ?3, seed = ?4, hash = ?5, \
           saved_at = ?6, updated = ?7",
    )
    .bind(&[
        n(owner),
        b.data.into(),
        b.scenario.into(),
        b.seed.into(),
        b.hash.into(),
        n(b.saved_at),
        n(now_secs()),
    ])?
    .run()
    .await?;
    maybe_cleanup(&d1).await;
    Response::from_json(&serde_json::json!({ "ok": true }))
}

/// GET /api/latticefall/save — the player's cloud save, or 404 when there is none.
#[utoipa::path(get, path = "/api/latticefall/save", tag = "Cinderheart",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "The cloud save"), (status = 401, description = "Not signed in"),
        (status = 404, description = "No cloud save")))]
pub async fn get_save(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner = match resolve_owner(&req, &ctx).await? {
        Ok(o) => o,
        Err(r) => return Ok(r),
    };
    let d1 = ctx.d1("IFKASH_D1")?;
    let row: Option<SaveRow> = d1
        .prepare("SELECT data, scenario, seed, hash, saved_at FROM lf_saves WHERE user_id = ?1")
        .bind(&[n(owner)])?
        .first(None)
        .await?;
    match row {
        Some(r) => Response::from_json(&r),
        None => err("no save", 404),
    }
}

/// DELETE /api/latticefall/save — remove the player's cloud save.
#[utoipa::path(delete, path = "/api/latticefall/save", tag = "Cinderheart",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Deleted"), (status = 401, description = "Not signed in")))]
pub async fn delete_save(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner = match resolve_owner(&req, &ctx).await? {
        Ok(o) => o,
        Err(r) => return Ok(r),
    };
    let d1 = ctx.d1("IFKASH_D1")?;
    d1.prepare("DELETE FROM lf_saves WHERE user_id = ?1")
        .bind(&[n(owner)])?
        .run()
        .await?;
    Response::from_json(&serde_json::json!({ "ok": true }))
}

// --------------------------------------------------------------- Mark sync

#[derive(Deserialize)]
struct MarkIn {
    id: String,
    /// Milliseconds since the epoch.
    at: i64,
}

#[derive(Deserialize)]
struct PutMarks {
    marks: Vec<MarkIn>,
}

#[derive(Serialize, Deserialize)]
struct MarkRow {
    id: String,
    at: i64,
}

async fn marks_for(d1: &D1Database, owner: i64) -> Result<Vec<MarkRow>> {
    d1.prepare(
        "SELECT mark_id AS id, earned_at AS at FROM lf_marks WHERE user_id = ?1 ORDER BY earned_at",
    )
    .bind(&[n(owner)])?
    .all()
    .await?
    .results()
}

/// GET /api/latticefall/marks — the signed-in player's Marks.
#[utoipa::path(get, path = "/api/latticefall/marks", tag = "Cinderheart",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Earned Marks"), (status = 401, description = "Not signed in")))]
pub async fn get_marks(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner = match resolve_owner(&req, &ctx).await? {
        Ok(o) => o,
        Err(r) => return Ok(r),
    };
    let d1 = ctx.d1("IFKASH_D1")?;
    Response::from_json(&serde_json::json!({ "marks": marks_for(&d1, owner).await? }))
}

/// PUT /api/latticefall/marks — merge the client's Marks into the server's (a union; the
/// earliest time wins) and return the merged set.
#[utoipa::path(put, path = "/api/latticefall/marks", tag = "Cinderheart",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Merged Marks"), (status = 401, description = "Not signed in"),
        (status = 429, description = "Rate limited")))]
pub async fn put_marks(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner = match resolve_owner(&req, &ctx).await? {
        Ok(o) => o,
        Err(r) => return Ok(r),
    };
    let b: PutMarks = match read_body(&mut req, 12_000).await {
        Ok(b) => b,
        Err(r) => return Ok(r),
    };
    if b.marks.len() > MAX_MARKS_PER_PUT
        || b.marks.iter().any(|m| !valid_id(&m.id, 1, 40) || m.at < 0)
    {
        return err("bad marks", 400);
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    if !within_rate(&d1, &format!("marks:{owner}"), LIMIT_MARKS).await? {
        return err("rate limited", 429);
    }
    if !b.marks.is_empty() {
        let mut stmts = Vec::with_capacity(b.marks.len());
        for m in &b.marks {
            stmts.push(
                d1.prepare(
                    "INSERT INTO lf_marks (user_id, mark_id, earned_at) VALUES (?1, ?2, ?3) \
                     ON CONFLICT(user_id, mark_id) DO UPDATE SET \
                       earned_at = MIN(earned_at, excluded.earned_at)",
                )
                .bind(&[n(owner), m.id.clone().into(), n(m.at)])?,
            );
        }
        d1.batch(stmts).await?;
    }
    Response::from_json(&serde_json::json!({ "marks": marks_for(&d1, owner).await? }))
}

// ------------------------------------------------------ Speedrun leaderboard

#[derive(Serialize, Deserialize)]
struct SpeedrunEntry {
    name: String,
    picture: Option<String>,
    best_ms: i64,
}

/// GET /api/latticefall/speedrun/leaderboard — the fastest B12 clears. Anonymous.
#[utoipa::path(get, path = "/api/latticefall/speedrun/leaderboard", tag = "Cinderheart",
    responses((status = 200, description = "Top speedruns by in-game time")))]
pub async fn speedrun_leaderboard(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("IFKASH_D1")?;
    let entries: Vec<SpeedrunEntry> = d1
        .prepare(
            "SELECT u.name AS name, u.picture AS picture, s.best_ms AS best_ms \
             FROM lf_speedruns s JOIN game_users u ON u.id = s.user_id \
             ORDER BY s.best_ms ASC LIMIT ?1",
        )
        .bind(&[n(LEADERBOARD_SIZE as i64)])?
        .all()
        .await?
        .results()?;
    cached(
        Response::from_json(&serde_json::json!({ "entries": entries }))?,
        60,
    )
}

#[derive(Deserialize)]
struct PostSpeedrun {
    /// In-game milliseconds from new game to the B12 defeat.
    ms: i64,
    content_hash: String,
}

/// POST /api/latticefall/speedrun — submit a B12 clear time. The server keeps the best.
#[utoipa::path(post, path = "/api/latticefall/speedrun", tag = "Cinderheart",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Best time and rank"), (status = 401, description = "Not signed in"),
        (status = 429, description = "Rate limited")))]
pub async fn post_speedrun(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner = match resolve_owner(&req, &ctx).await? {
        Ok(o) => o,
        Err(r) => return Ok(r),
    };
    let b: PostSpeedrun = match read_body(&mut req, 500).await {
        Ok(b) => b,
        Err(r) => return Ok(r),
    };
    if !(MIN_SPEEDRUN_MS..=MAX_SPEEDRUN_MS).contains(&b.ms) || !valid_id(&b.content_hash, 1, 64) {
        return err("bad time", 400);
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    if !within_rate(&d1, &format!("speed:{owner}"), LIMIT_SPEEDRUN).await? {
        return err("rate limited", 429);
    }
    d1.prepare(
        "INSERT INTO lf_speedruns (user_id, best_ms, content_hash, updated) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(user_id) DO UPDATE SET \
           content_hash = CASE WHEN ?2 < best_ms THEN ?3 ELSE content_hash END, \
           updated = CASE WHEN ?2 < best_ms THEN ?4 ELSE updated END, \
           best_ms = MIN(best_ms, ?2)",
    )
    .bind(&[n(owner), n(b.ms), b.content_hash.into(), n(now_secs())])?
    .run()
    .await?;
    #[derive(Deserialize)]
    struct RankRow {
        best_ms: i64,
        rank: i64,
    }
    let row: Option<RankRow> = d1
        .prepare(
            "SELECT s.best_ms AS best_ms, \
               1 + (SELECT COUNT(*) FROM lf_speedruns x WHERE x.best_ms < s.best_ms) AS rank \
             FROM lf_speedruns s WHERE s.user_id = ?1",
        )
        .bind(&[n(owner)])?
        .first(None)
        .await?;
    match row {
        Some(r) => {
            Response::from_json(&serde_json::json!({ "best_ms": r.best_ms, "rank": r.rank }))
        }
        None => err("could not read rank", 500),
    }
}

// ---------------------------------------------------------------- Telemetry

#[derive(Deserialize)]
struct PostTelemetry {
    anon_id: String,
    session: String,
    /// Must be true. The client sends a batch only when the player opted in.
    consent: bool,
    events: Vec<Value>,
}

/// POST /api/latticefall/telemetry — one batch of opt-in, anonymous events. No sign-in.
/// Each event is a flat object with a `k` kind from a fixed list and a few scalar fields.
#[utoipa::path(post, path = "/api/latticefall/telemetry", tag = "Cinderheart",
    responses((status = 200, description = "Batch stored"), (status = 400, description = "Bad batch"),
        (status = 429, description = "Rate limited")))]
pub async fn post_telemetry(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let ip = req
        .headers()
        .get("cf-connecting-ip")?
        .unwrap_or_else(|| "local".to_string());
    let b: PostTelemetry = match read_body(&mut req, MAX_TELEMETRY_BODY).await {
        Ok(b) => b,
        Err(r) => return Ok(r),
    };
    if !b.consent
        || !valid_id(&b.anon_id, 8, 64)
        || !valid_id(&b.session, 8, 64)
        || b.events.is_empty()
        || b.events.len() > MAX_TELEMETRY_EVENTS
        || !b.events.iter().all(valid_event)
    {
        return err("bad batch", 400);
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    // The address is used only as a short-lived rate-limit key (dropped by the hourly
    // cleanup) and is never stored with the events.
    let ip_ok = within_rate(&d1, &format!("tel-ip:{ip}"), LIMIT_TELEMETRY_IP).await?;
    let anon_ok = within_rate(&d1, &format!("tel:{}", b.anon_id), LIMIT_TELEMETRY_ANON).await?;
    if !ip_ok || !anon_ok {
        return err("rate limited", 429);
    }
    let events = serde_json::to_string(&b.events).unwrap_or_else(|_| "[]".into());
    d1.prepare(
        "INSERT INTO lf_telemetry (anon_id, session, n_events, events, created) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&[
        b.anon_id.into(),
        b.session.into(),
        n(b.events.len() as i64),
        events.into(),
        n(now_secs()),
    ])?
    .run()
    .await?;
    maybe_cleanup(&d1).await;
    Response::from_json(&serde_json::json!({ "ok": true, "stored": b.events.len() }))
}

/// An event is a flat object: a known `k`, and only short strings, numbers, and booleans.
fn valid_event(e: &Value) -> bool {
    let Some(obj) = e.as_object() else {
        return false;
    };
    let kind_ok = obj
        .get("k")
        .and_then(Value::as_str)
        .map(|k| TELEMETRY_KINDS.contains(&k))
        .unwrap_or(false);
    kind_ok
        && obj.len() <= 12
        && obj.iter().all(|(key, v)| {
            key.len() <= 24
                && match v {
                    Value::String(s) => s.len() <= 120,
                    Value::Number(_) | Value::Bool(_) => true,
                    _ => false,
                }
        })
}
