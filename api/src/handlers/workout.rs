use super::google_auth::verify_google;
use serde::{Deserialize, Serialize};
use worker::*;

// ---- helpers ---------------------------------------------------------------

/// D1 rejects bigint JsValues, so bind every integer column as f64 (all our ids
/// and gram amounts are well within f64's safe-integer range).
fn n(v: i64) -> wasm_bindgen::JsValue {
    (v as f64).into()
}

/// Resolve the signed-in user from the `Authorization: Bearer <id_token>` header,
/// upserting them into `game_users`. The outer `Result` carries infra errors (use
/// `?`); the inner `Result` carries an auth failure `Response` to return as-is.
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

fn parse_id(ctx: &RouteContext<()>, name: &str) -> Option<i64> {
    ctx.param(name).and_then(|s| s.parse::<i64>().ok())
}

/// Confirm `session_id` exists and is owned by `owner_id`.
async fn owns_session(d1: &D1Database, owner_id: i64, session_id: i64) -> Result<bool> {
    #[derive(Deserialize)]
    struct IdRow {
        id: i64,
    }
    let row: Option<IdRow> = d1
        .prepare("SELECT id FROM workout_sessions WHERE id = ?1 AND owner_id = ?2")
        .bind(&[n(session_id), n(owner_id)])?
        .first(None)
        .await?;
    Ok(row.is_some())
}

// ---- request / response shapes ---------------------------------------------

#[derive(Serialize, Deserialize)]
struct SessionRow {
    id: i64,
    day_label: String,
    date: String,
    notes: String,
    created: String,
}

#[derive(Serialize, Deserialize)]
struct SetRow {
    id: i64,
    exercise: String,
    set_index: i64,
    reps: i64,
    weight_g: i64,
}

#[derive(Deserialize)]
struct SetInput {
    reps: i64,
    weight_g: i64,
}

#[derive(Deserialize)]
struct ExerciseInput {
    exercise: String,
    sets: Vec<SetInput>,
}

#[derive(Deserialize)]
struct CreateSessionRequest {
    #[serde(default)]
    day_label: String,
    date: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    exercises: Vec<ExerciseInput>,
}

#[derive(Serialize)]
struct SessionDetail {
    session: SessionRow,
    sets: Vec<SetRow>,
}

#[derive(Serialize, Deserialize)]
struct BodyweightRow {
    id: i64,
    date: String,
    weight_g: i64,
}

#[derive(Deserialize)]
struct AddBodyweightRequest {
    date: String,
    weight_g: i64,
}

#[derive(Serialize)]
struct IdResponse {
    id: i64,
}

// ---- session handlers ------------------------------------------------------

/// GET /api/workout/sessions — list the user's sessions (newest first).
pub async fn list_sessions(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let d1 = ctx.d1("IFKASH_D1")?;
    let rows: Vec<SessionRow> = d1
        .prepare(
            "SELECT id, day_label, date, notes, created FROM workout_sessions \
             WHERE owner_id = ?1 ORDER BY date DESC, id DESC",
        )
        .bind(&[n(owner_id)])?
        .all()
        .await?
        .results()?;
    Response::from_json(&rows)
}

/// POST /api/workout/sessions — create a session with all its sets.
pub async fn create_session(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let body: CreateSessionRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => return Response::error("bad request", 400),
    };
    let date = body.date.trim();
    if date.is_empty() {
        return Response::error("date required", 400);
    }

    let d1 = ctx.d1("IFKASH_D1")?;
    #[derive(Deserialize)]
    struct InsertedRow {
        id: i64,
    }
    let inserted: Option<InsertedRow> = d1
        .prepare(
            "INSERT INTO workout_sessions (owner_id, day_label, date, notes) \
             VALUES (?1, ?2, ?3, ?4) RETURNING id",
        )
        .bind(&[
            n(owner_id),
            body.day_label.trim().into(),
            date.into(),
            body.notes.trim().into(),
        ])?
        .first(None)
        .await?;
    let session_id = inserted
        .ok_or_else(|| Error::RustError("session insert returned no id".into()))?
        .id;

    for exercise in &body.exercises {
        let name = exercise.exercise.trim();
        if name.is_empty() {
            continue;
        }
        for (i, set) in exercise.sets.iter().enumerate() {
            d1.prepare(
                "INSERT INTO workout_sets (session_id, exercise, set_index, reps, weight_g) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .bind(&[
                n(session_id),
                name.into(),
                n((i + 1) as i64),
                n(set.reps.max(0)),
                n(set.weight_g.max(0)),
            ])?
            .run()
            .await?;
        }
    }

    Response::from_json(&IdResponse { id: session_id })
}

/// GET /api/workout/sessions/:id — full session detail (session + sets).
pub async fn get_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let session_id = match parse_id(&ctx, "id") {
        Some(id) => id,
        None => return Response::error("invalid id", 400),
    };
    let d1 = ctx.d1("IFKASH_D1")?;

    let session: Option<SessionRow> = d1
        .prepare(
            "SELECT id, day_label, date, notes, created FROM workout_sessions \
             WHERE id = ?1 AND owner_id = ?2",
        )
        .bind(&[n(session_id), n(owner_id)])?
        .first(None)
        .await?;
    let session = match session {
        Some(s) => s,
        None => return Response::error("not found", 404),
    };

    let sets: Vec<SetRow> = d1
        .prepare(
            "SELECT id, exercise, set_index, reps, weight_g FROM workout_sets \
             WHERE session_id = ?1 ORDER BY id ASC",
        )
        .bind(&[n(session_id)])?
        .all()
        .await?
        .results()?;

    Response::from_json(&SessionDetail { session, sets })
}

/// DELETE /api/workout/sessions/:id — delete a session and its sets.
pub async fn delete_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let session_id = match parse_id(&ctx, "id") {
        Some(id) => id,
        None => return Response::error("invalid id", 400),
    };
    let d1 = ctx.d1("IFKASH_D1")?;
    if !owns_session(&d1, owner_id, session_id).await? {
        return Response::error("not found", 404);
    }
    d1.prepare("DELETE FROM workout_sets WHERE session_id = ?1")
        .bind(&[n(session_id)])?
        .run()
        .await?;
    d1.prepare("DELETE FROM workout_sessions WHERE id = ?1")
        .bind(&[n(session_id)])?
        .run()
        .await?;
    Response::from_json(&IdResponse { id: session_id })
}

// ---- bodyweight handlers ---------------------------------------------------

/// GET /api/workout/bodyweight — list the user's bodyweight entries (oldest first).
pub async fn list_bodyweight(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let d1 = ctx.d1("IFKASH_D1")?;
    let rows: Vec<BodyweightRow> = d1
        .prepare(
            "SELECT id, date, weight_g FROM bodyweight_logs \
             WHERE owner_id = ?1 ORDER BY date ASC, id ASC",
        )
        .bind(&[n(owner_id)])?
        .all()
        .await?
        .results()?;
    Response::from_json(&rows)
}

/// POST /api/workout/bodyweight — add a bodyweight entry.
pub async fn add_bodyweight(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let body: AddBodyweightRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => return Response::error("bad request", 400),
    };
    let date = body.date.trim();
    if date.is_empty() {
        return Response::error("date required", 400);
    }
    if body.weight_g <= 0 {
        return Response::error("weight must be positive", 400);
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    #[derive(Deserialize)]
    struct InsertedRow {
        id: i64,
    }
    let inserted: Option<InsertedRow> = d1
        .prepare(
            "INSERT INTO bodyweight_logs (owner_id, date, weight_g) \
             VALUES (?1, ?2, ?3) RETURNING id",
        )
        .bind(&[n(owner_id), date.into(), n(body.weight_g)])?
        .first(None)
        .await?;
    let id = inserted
        .ok_or_else(|| Error::RustError("bodyweight insert returned no id".into()))?
        .id;
    Response::from_json(&IdResponse { id })
}

/// DELETE /api/workout/bodyweight/:id — delete a bodyweight entry.
pub async fn delete_bodyweight(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let entry_id = match parse_id(&ctx, "id") {
        Some(id) => id,
        None => return Response::error("invalid id", 400),
    };
    let d1 = ctx.d1("IFKASH_D1")?;
    #[derive(Deserialize)]
    struct IdRow {
        id: i64,
    }
    let owned: Option<IdRow> = d1
        .prepare("SELECT id FROM bodyweight_logs WHERE id = ?1 AND owner_id = ?2")
        .bind(&[n(entry_id), n(owner_id)])?
        .first(None)
        .await?;
    if owned.is_none() {
        return Response::error("not found", 404);
    }
    d1.prepare("DELETE FROM bodyweight_logs WHERE id = ?1")
        .bind(&[n(entry_id)])?
        .run()
        .await?;
    Response::from_json(&IdResponse { id: entry_id })
}
