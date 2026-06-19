use super::google_auth::verify_google;
use serde::{Deserialize, Serialize};
use worker::*;

// ---- helpers ---------------------------------------------------------------
// Mirrors the auth/bind helpers in `workout.rs` (each handler module keeps its
// own copy so they stay self-contained).

/// D1 rejects bigint JsValues, so bind every integer column as f64.
fn n(v: i64) -> wasm_bindgen::JsValue {
    (v as f64).into()
}

/// Resolve the signed-in user from the `Authorization: Bearer <id_token>` header,
/// upserting them into `game_users`.
async fn resolve_owner(
    req: &Request,
    ctx: &RouteContext<()>,
) -> Result<std::result::Result<i64, Response>> {
    let auth = req.headers().get("Authorization")?.unwrap_or_default();
    let token = auth.strip_prefix("Bearer ").unwrap_or(auth.as_str()).trim();
    if token.is_empty() {
        return Ok(Err(Response::error("unauthorized", 401)?));
    }
    let client_id = ctx.env.var("GOOGLE_CLIENT_ID")?.to_string();
    let info = match verify_google(token, &client_id).await {
        Ok(i) => i,
        Err(resp) => return Ok(Err(resp)),
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
        name.into(),
        picture.into(),
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

macro_rules! owner {
    ($req:expr, $ctx:expr) => {
        match resolve_owner(&$req, &$ctx).await? {
            Ok(id) => id,
            Err(resp) => return Ok(resp),
        }
    };
}

// ---- shapes ----------------------------------------------------------------

/// The profile defaults applied when a user has no saved row yet. Matches the
/// migration's column defaults (179 cm, male, age 28, moderate activity + cut).
fn default_profile() -> ProfileRow {
    ProfileRow {
        height_cm: 179,
        sex: "male".to_string(),
        age_years: 28,
        activity: 1.55,
        goal: "cut_moderate".to_string(),
    }
}

#[derive(Serialize, Deserialize)]
struct ProfileRow {
    height_cm: i64,
    sex: String,
    age_years: i64,
    activity: f64,
    goal: String,
}

/// GET response: the profile plus the most recent logged bodyweight (grams) so a
/// client can compute targets without a separate, token-specific bodyweight call.
#[derive(Serialize)]
struct ProfileResponse {
    height_cm: i64,
    sex: String,
    age_years: i64,
    activity: f64,
    goal: String,
    latest_weight_g: Option<i64>,
}

// ---- handlers --------------------------------------------------------------

/// GET /api/profile — the user's body profile (or defaults) + latest bodyweight.
pub async fn get_profile(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let d1 = ctx.d1("IFKASH_D1")?;

    let row: Option<ProfileRow> = d1
        .prepare(
            "SELECT height_cm, sex, age_years, activity, goal \
             FROM user_profile WHERE owner_id = ?1",
        )
        .bind(&[n(owner_id)])?
        .first(None)
        .await?;
    let p = row.unwrap_or_else(default_profile);

    #[derive(Deserialize)]
    struct WeightRow {
        weight_g: i64,
    }
    let latest: Option<WeightRow> = d1
        .prepare(
            "SELECT weight_g FROM bodyweight_logs \
             WHERE owner_id = ?1 ORDER BY date DESC, id DESC LIMIT 1",
        )
        .bind(&[n(owner_id)])?
        .first(None)
        .await?;

    Response::from_json(&ProfileResponse {
        height_cm: p.height_cm,
        sex: p.sex,
        age_years: p.age_years,
        activity: p.activity,
        goal: p.goal,
        latest_weight_g: latest.map(|w| w.weight_g),
    })
}

/// PUT /api/profile — upsert the user's body profile.
pub async fn upsert_profile(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let body: ProfileRow = match req.json().await {
        Ok(b) => b,
        Err(_) => return Response::error("bad request", 400),
    };

    // Light validation / clamping so bad input can't poison the metrics math.
    let height = body.height_cm.clamp(50, 300);
    let age = body.age_years.clamp(1, 120);
    let activity = if body.activity.is_finite() {
        body.activity.clamp(1.0, 2.5)
    } else {
        1.55
    };
    let sex = if body.sex == "female" { "female" } else { "male" };
    let goal = match body.goal.as_str() {
        "cut_aggressive" | "maintain" => body.goal.as_str(),
        _ => "cut_moderate",
    };

    let d1 = ctx.d1("IFKASH_D1")?;
    d1.prepare(
        "INSERT INTO user_profile (owner_id, height_cm, sex, age_years, activity, goal, updated) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now')) \
         ON CONFLICT(owner_id) DO UPDATE SET \
           height_cm = excluded.height_cm, sex = excluded.sex, age_years = excluded.age_years, \
           activity = excluded.activity, goal = excluded.goal, updated = datetime('now')",
    )
    .bind(&[
        n(owner_id),
        n(height),
        sex.into(),
        n(age),
        activity.into(),
        goal.into(),
    ])?
    .run()
    .await?;

    Response::from_json(&ProfileResponse {
        height_cm: height,
        sex: sex.to_string(),
        age_years: age,
        activity,
        goal: goal.to_string(),
        latest_weight_g: None,
    })
}
