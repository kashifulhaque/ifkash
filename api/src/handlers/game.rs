use super::google_auth::verify_google;
use serde::{Deserialize, Serialize};
use worker::*;

const MAX_SCORE: i64 = 10_000_000;
const LEADERBOARD_SIZE: u32 = 20;

#[derive(Deserialize)]
struct SubmitRequest {
    id_token: String,
    score: i64,
}

#[derive(Deserialize)]
struct SubmitDailyRequest {
    id_token: String,
    score: i64,
    day: String,
}

/// Civil date (Y, M, D) from a day count since the Unix epoch (UTC).
/// Howard Hinnant's civil_from_days algorithm.
fn ymd_from_epoch_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn day_string_for_epoch_days(days: i64) -> String {
    let (y, m, d) = ymd_from_epoch_days(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Is `day` well-formed (YYYY-MM-DD) and within ±1 day of the server's UTC
/// date? The ±1 window absorbs the player's local-vs-UTC timezone offset.
fn day_is_acceptable(day: &str) -> bool {
    let bytes = day.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes.iter().enumerate().all(|(i, &c)| {
        if i == 4 || i == 7 {
            c == b'-'
        } else {
            c.is_ascii_digit()
        }
    }) {
        return false;
    }
    let now_days = (worker::Date::now().as_millis() / 86_400_000) as i64;
    (now_days - 1..=now_days + 1).any(|d| day_string_for_epoch_days(d) == day)
}

#[derive(Serialize)]
struct SubmitResponse {
    best: i64,
    rank: i64,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct LeaderboardEntry {
    name: String,
    picture: Option<String>,
    score: i64,
}

#[derive(Serialize)]
struct LeaderboardResponse {
    entries: Vec<LeaderboardEntry>,
}

/// POST /api/game/score — verify the Google token, upsert the player and
/// their best score, return their best + leaderboard rank.
pub async fn submit_score(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: SubmitRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => return Response::error("bad request", 400),
    };
    if body.score < 0 || body.score > MAX_SCORE {
        return Response::error("bad score", 400);
    }

    let client_id = ctx.env.var("GOOGLE_CLIENT_ID")?.to_string();
    let info = match verify_google(&body.id_token, &client_id).await {
        Ok(i) => i,
        Err(resp) => return Ok(resp),
    };

    let name = info.name.unwrap_or_else(|| "anonymous".to_string());
    let email = info.email.unwrap_or_default();
    let picture = info.picture.unwrap_or_default();

    let d1 = ctx.d1("IFKASH_D1")?;
    d1.prepare(
        "INSERT INTO game_users (google_sub, email, name, picture) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(google_sub) DO UPDATE SET email = ?2, name = ?3, picture = ?4",
    )
    .bind(&[
        info.sub.clone().into(),
        email.into(),
        name.clone().into(),
        picture.into(),
    ])?
    .run()
    .await?;

    d1.prepare(
        "INSERT INTO game_scores (user_id, best_score, games_played, updated) \
         SELECT id, ?2, 1, datetime('now') FROM game_users WHERE google_sub = ?1 \
         ON CONFLICT(user_id) DO UPDATE SET \
           best_score = MAX(best_score, ?2), \
           games_played = games_played + 1, \
           updated = datetime('now')",
    )
    // Bind the score as f64 — D1 rejects bigint JsValues, and the INTEGER
    // column stores it losslessly (scores are well within f64's safe range).
    .bind(&[info.sub.clone().into(), (body.score as f64).into()])?
    .run()
    .await?;

    #[derive(Deserialize)]
    struct RankRow {
        best: i64,
        rank: i64,
    }
    let row: Option<RankRow> = d1
        .prepare(
            "SELECT s.best_score AS best, \
               1 + (SELECT COUNT(*) FROM game_scores x WHERE x.best_score > s.best_score) AS rank \
             FROM game_scores s JOIN game_users u ON u.id = s.user_id \
             WHERE u.google_sub = ?1",
        )
        .bind(&[info.sub.into()])?
        .first(None)
        .await?;

    let row = row.ok_or_else(|| Error::RustError("score row missing after upsert".into()))?;
    Response::from_json(&SubmitResponse {
        best: row.best,
        rank: row.rank,
        name,
    })
}

/// GET /api/game/leaderboard — top players by best score.
pub async fn leaderboard(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let d1 = ctx.d1("IFKASH_D1")?;
    let rows = d1
        .prepare(
            "SELECT u.name AS name, u.picture AS picture, s.best_score AS score \
             FROM game_scores s JOIN game_users u ON u.id = s.user_id \
             ORDER BY s.best_score DESC, s.updated ASC LIMIT ?1",
        )
        .bind(&[LEADERBOARD_SIZE.into()])?
        .all()
        .await?;
    let entries: Vec<LeaderboardEntry> = rows.results()?;
    Response::from_json(&LeaderboardResponse { entries })
}

/// POST /api/game/daily/score — verify the token, upsert the player's best
/// score for the given day, return their best + rank on that day's board.
pub async fn submit_daily_score(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: SubmitDailyRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => return Response::error("bad request", 400),
    };
    if body.score < 0 || body.score > MAX_SCORE {
        return Response::error("bad score", 400);
    }
    if !day_is_acceptable(&body.day) {
        return Response::error("bad day", 400);
    }

    let client_id = ctx.env.var("GOOGLE_CLIENT_ID")?.to_string();
    let info = match verify_google(&body.id_token, &client_id).await {
        Ok(i) => i,
        Err(resp) => return Ok(resp),
    };

    let name = info.name.clone().unwrap_or_else(|| "anonymous".to_string());
    let email = info.email.clone().unwrap_or_default();
    let picture = info.picture.clone().unwrap_or_default();

    let d1 = ctx.d1("IFKASH_D1")?;
    // Ensure the player row exists (mirrors submit_score).
    d1.prepare(
        "INSERT INTO game_users (google_sub, email, name, picture) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(google_sub) DO UPDATE SET email = ?2, name = ?3, picture = ?4",
    )
    .bind(&[
        info.sub.clone().into(),
        email.into(),
        name.clone().into(),
        picture.into(),
    ])?
    .run()
    .await?;

    d1.prepare(
        "INSERT INTO daily_scores (user_id, day, score, updated) \
         SELECT id, ?2, ?3, datetime('now') FROM game_users WHERE google_sub = ?1 \
         ON CONFLICT(user_id, day) DO UPDATE SET \
           score = MAX(score, ?3), \
           updated = datetime('now')",
    )
    .bind(&[
        info.sub.clone().into(),
        body.day.clone().into(),
        (body.score as f64).into(),
    ])?
    .run()
    .await?;

    #[derive(Deserialize)]
    struct RankRow {
        best: i64,
        rank: i64,
    }
    let row: Option<RankRow> = d1
        .prepare(
            "SELECT s.score AS best, \
               1 + (SELECT COUNT(*) FROM daily_scores x WHERE x.day = s.day AND x.score > s.score) AS rank \
             FROM daily_scores s JOIN game_users u ON u.id = s.user_id \
             WHERE u.google_sub = ?1 AND s.day = ?2",
        )
        .bind(&[info.sub.into(), body.day.into()])?
        .first(None)
        .await?;

    let row = row.ok_or_else(|| Error::RustError("daily score row missing after upsert".into()))?;
    Response::from_json(&SubmitResponse {
        best: row.best,
        rank: row.rank,
        name,
    })
}

/// GET /api/game/daily/leaderboard?day=YYYY-MM-DD — top players for that day.
pub async fn daily_leaderboard(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;
    let day = url
        .query_pairs()
        .find(|(k, _)| k == "day")
        .map(|(_, v)| v.into_owned())
        .unwrap_or_default();
    if !day_is_acceptable(&day) {
        return Response::error("bad day", 400);
    }

    let d1 = ctx.d1("IFKASH_D1")?;
    let rows = d1
        .prepare(
            "SELECT u.name AS name, u.picture AS picture, s.score AS score \
             FROM daily_scores s JOIN game_users u ON u.id = s.user_id \
             WHERE s.day = ?1 \
             ORDER BY s.score DESC, s.updated ASC LIMIT ?2",
        )
        .bind(&[day.into(), LEADERBOARD_SIZE.into()])?
        .all()
        .await?;
    let entries: Vec<LeaderboardEntry> = rows.results()?;
    Response::from_json(&LeaderboardResponse { entries })
}
