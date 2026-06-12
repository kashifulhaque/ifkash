use serde::{Deserialize, Serialize};
use worker::*;

const MAX_SCORE: i64 = 10_000_000;
const LEADERBOARD_SIZE: u32 = 20;

#[derive(Deserialize)]
struct SubmitRequest {
    id_token: String,
    score: i64,
}

/// Subset of Google's tokeninfo response we care about.
#[derive(Deserialize)]
struct TokenInfo {
    aud: String,
    iss: String,
    sub: String,
    email: Option<String>,
    name: Option<String>,
    picture: Option<String>,
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

/// Verify a Google ID token via Google's tokeninfo endpoint. This avoids
/// shipping a JWT/crypto stack to wasm; Google validates the signature and
/// expiry, we validate audience and issuer.
async fn verify_google(id_token: &str, client_id: &str) -> std::result::Result<TokenInfo, Response> {
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        urlencoding::encode(id_token)
    );
    let unauthorized = || Response::error("invalid token", 401).unwrap();
    let resp = reqwest::get(&url).await.map_err(|_| unauthorized())?;
    if !resp.status().is_success() {
        return Err(unauthorized());
    }
    let info: TokenInfo = resp.json().await.map_err(|_| unauthorized())?;
    if info.aud != client_id {
        return Err(unauthorized());
    }
    if info.iss != "https://accounts.google.com" && info.iss != "accounts.google.com" {
        return Err(unauthorized());
    }
    Ok(info)
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
    .bind(&[info.sub.clone().into(), body.score.into()])?
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
