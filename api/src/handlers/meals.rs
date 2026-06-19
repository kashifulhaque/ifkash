use super::google_auth::verify_google;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::*;

// ---- helpers ---------------------------------------------------------------

/// D1 rejects bigint JsValues, so bind every integer column as f64 (all our ids
/// are well within f64's safe-integer range).
fn n(v: i64) -> wasm_bindgen::JsValue {
    (v as f64).into()
}

/// Resolve the signed-in owner from the `Authorization: Bearer <id_token>`
/// header, upserting them into `game_users`. The outer `Result` carries infra
/// errors (use `?`); the inner `Result` carries an auth failure `Response` the
/// caller should return as-is. Mirrors the splitter handler.
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

/// Confirm `meal_id` exists and belongs to `owner_id`.
async fn owns_meal(d1: &D1Database, owner_id: i64, meal_id: i64) -> Result<bool> {
    #[derive(Deserialize)]
    struct IdRow {
        id: i64,
    }
    let row: Option<IdRow> = d1
        .prepare("SELECT id FROM meals WHERE id = ?1 AND owner_id = ?2")
        .bind(&[n(meal_id), n(owner_id)])?
        .first(None)
        .await?;
    Ok(row.is_some())
}

// ---- request / response shapes ---------------------------------------------

#[derive(Serialize)]
struct Meal {
    id: i64,
    eaten_on: String,
    eaten_at: String,
    description: Option<String>,
    calories: Option<f64>,
    protein_g: Option<f64>,
    carbs_g: Option<f64>,
    fat_g: Option<f64>,
    items: Value,
    photo_r2_key: Option<String>,
}

#[derive(Deserialize)]
struct MealRow {
    id: i64,
    eaten_on: String,
    eaten_at: String,
    description: Option<String>,
    calories: Option<f64>,
    protein_g: Option<f64>,
    carbs_g: Option<f64>,
    fat_g: Option<f64>,
    items_json: Option<String>,
    photo_r2_key: Option<String>,
}

impl From<MealRow> for Meal {
    fn from(r: MealRow) -> Self {
        let items = r
            .items_json
            .as_deref()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .unwrap_or(Value::Array(vec![]));
        Meal {
            id: r.id,
            eaten_on: r.eaten_on,
            eaten_at: r.eaten_at,
            description: r.description,
            calories: r.calories,
            protein_g: r.protein_g,
            carbs_g: r.carbs_g,
            fat_g: r.fat_g,
            items,
            photo_r2_key: r.photo_r2_key,
        }
    }
}

#[derive(Serialize)]
struct Totals {
    calories: f64,
    protein_g: f64,
    carbs_g: f64,
    fat_g: f64,
}

#[derive(Serialize)]
struct DayResponse {
    meals: Vec<Meal>,
    totals: Totals,
}

#[derive(Deserialize)]
struct UpdatePayload {
    description: Option<String>,
    calories: Option<f64>,
    protein_g: Option<f64>,
    carbs_g: Option<f64>,
    fat_g: Option<f64>,
}

// ---- model call ------------------------------------------------------------

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MODEL: &str = "qwen/qwen3.7-plus";

const SYSTEM_PROMPT: &str = "You are a nutrition estimator. Look at the food photo and \
return ONLY a JSON object (no markdown, no prose) with this exact shape: \
{\"description\": string, \"items\": [{\"name\": string, \"grams\": number, \"calories\": number, \
\"protein_g\": number, \"carbs_g\": number, \"fat_g\": number}], \
\"total\": {\"calories\": number, \"protein_g\": number, \"carbs_g\": number, \"fat_g\": number}}. \
Estimate realistic portion sizes in grams. All macro values are in grams; calories in kcal. \
If the user supplies a hint, use it to refine portions.";

/// Strip a ```json fence if the model wrapped the object, then parse.
fn parse_model_json(content: &str) -> Option<Value> {
    let trimmed = content.trim();
    let inner = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.trim_start())
        .and_then(|s| s.strip_suffix("```"))
        .map(|s| s.trim())
        .unwrap_or(trimmed);
    serde_json::from_str::<Value>(inner).ok()
}

fn num(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(|x| x.as_f64())
}

// ---- POST /api/meals/analyze -----------------------------------------------

pub async fn analyze(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);

    let form = req.form_data().await?;

    let photo_bytes = match form.get("photo") {
        Some(FormEntry::File(f)) => f.bytes().await?,
        _ => return Response::error("Missing photo", 400),
    };
    let hint = match form.get("hint") {
        Some(FormEntry::Field(s)) if !s.trim().is_empty() => s,
        _ => String::new(),
    };
    let eaten_on = match form.get("eaten_on") {
        Some(FormEntry::Field(s)) if !s.trim().is_empty() => s,
        _ => return Response::error("Missing eaten_on", 400),
    };

    let api_key = match ctx.secret("OPENROUTER_API_KEY") {
        Ok(s) => s.to_string(),
        Err(_) => return Response::error("Missing OPENROUTER_API_KEY", 500),
    };

    // Build a data URL for the vision model.
    let b64 = STANDARD.encode(&photo_bytes);
    let data_url = format!("data:image/jpeg;base64,{}", b64);

    let user_text = if hint.is_empty() {
        "Analyze this meal photo.".to_string()
    } else {
        format!("Analyze this meal photo. Hint: {}", hint)
    };

    let body = serde_json::json!({
        "model": MODEL,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": SYSTEM_PROMPT },
            { "role": "user", "content": [
                { "type": "text", "text": user_text },
                { "type": "image_url", "image_url": { "url": data_url } }
            ]}
        ]
    });

    let resp = reqwest::Client::new()
        .post(OPENROUTER_URL)
        .bearer_auth(&api_key)
        .header("HTTP-Referer", "https://ifkash.dev")
        .header("X-Title", "ifkash meal tracker")
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::RustError(format!("OpenRouter request failed: {}", e)))?;

    if !resp.status().is_success() {
        let code = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        console_error!("OpenRouter {}: {}", code, text);
        return Response::error("Model request failed", 502);
    }

    let completion: Value = resp
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Bad model response: {}", e)))?;

    let content = completion
        .pointer("/choices/0/message/content")
        .and_then(|c| c.as_str())
        .unwrap_or("");

    let parsed = match parse_model_json(content) {
        Some(p) => p,
        None => {
            console_error!("Could not parse model JSON: {}", content);
            return Response::error("Could not read nutrition breakdown", 502);
        }
    };

    let description = parsed
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string();
    let items = parsed.get("items").cloned().unwrap_or(Value::Array(vec![]));
    let total = parsed.get("total").cloned().unwrap_or(Value::Null);

    let calories = num(&total, "calories");
    let protein_g = num(&total, "protein_g");
    let carbs_g = num(&total, "carbs_g");
    let fat_g = num(&total, "fat_g");
    let items_json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());

    // Store the photo in R2 (reuse the existing bucket binding) under a meals/ prefix.
    let photo_key = format!(
        "meals/{}/{}.jpg",
        owner_id,
        chrono::Utc::now().timestamp_millis()
    );
    let mut stored_key: Option<String> = None;
    match ctx.bucket("RESUME_BUCKET") {
        Ok(bucket) => {
            if bucket.put(&photo_key, photo_bytes).execute().await.is_ok() {
                stored_key = Some(photo_key.clone());
            }
        }
        Err(e) => console_error!("R2 bucket unavailable: {:?}", e),
    }

    let d1 = ctx.d1("IFKASH_D1")?;

    #[derive(Deserialize)]
    struct InsertedRow {
        id: i64,
    }
    let inserted: Option<InsertedRow> = d1
        .prepare(
            "INSERT INTO meals \
             (owner_id, eaten_on, description, calories, protein_g, carbs_g, fat_g, items_json, photo_r2_key) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) RETURNING id",
        )
        .bind(&[
            n(owner_id),
            eaten_on.clone().into(),
            description.clone().into(),
            calories.map(|v| v.into()).unwrap_or(wasm_bindgen::JsValue::NULL),
            protein_g.map(|v| v.into()).unwrap_or(wasm_bindgen::JsValue::NULL),
            carbs_g.map(|v| v.into()).unwrap_or(wasm_bindgen::JsValue::NULL),
            fat_g.map(|v| v.into()).unwrap_or(wasm_bindgen::JsValue::NULL),
            items_json.into(),
            stored_key
                .clone()
                .map(|k| k.into())
                .unwrap_or(wasm_bindgen::JsValue::NULL),
        ])?
        .first(None)
        .await?;

    let id = inserted.map(|r| r.id).unwrap_or_default();

    Response::from_json(&Meal {
        id,
        eaten_on,
        eaten_at: chrono::Utc::now().to_rfc3339(),
        description: Some(description),
        calories,
        protein_g,
        carbs_g,
        fat_g,
        items,
        photo_r2_key: stored_key,
    })
}

// ---- GET /api/meals?date=YYYY-MM-DD ----------------------------------------

pub async fn list(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);

    let url = req.url()?;
    let date = url
        .query_pairs()
        .find(|(k, _)| k == "date")
        .map(|(_, v)| v.into_owned());
    let date = match date {
        Some(d) if !d.is_empty() => d,
        _ => return Response::error("Missing date", 400),
    };

    let d1 = ctx.d1("IFKASH_D1")?;
    let result = d1
        .prepare(
            "SELECT id, eaten_on, eaten_at, description, calories, protein_g, carbs_g, fat_g, \
             items_json, photo_r2_key FROM meals \
             WHERE owner_id = ?1 AND eaten_on = ?2 ORDER BY datetime(eaten_at) ASC, id ASC",
        )
        .bind(&[n(owner_id), date.into()])?
        .all()
        .await?;

    let rows: Vec<MealRow> = result.results()?;
    let meals: Vec<Meal> = rows.into_iter().map(Meal::from).collect();

    let totals = meals.iter().fold(
        Totals {
            calories: 0.0,
            protein_g: 0.0,
            carbs_g: 0.0,
            fat_g: 0.0,
        },
        |mut acc, m| {
            acc.calories += m.calories.unwrap_or(0.0);
            acc.protein_g += m.protein_g.unwrap_or(0.0);
            acc.carbs_g += m.carbs_g.unwrap_or(0.0);
            acc.fat_g += m.fat_g.unwrap_or(0.0);
            acc
        },
    );

    Response::from_json(&DayResponse { meals, totals })
}

// ---- PATCH /api/meals/:id --------------------------------------------------

pub async fn update(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
        Some(id) => id,
        None => return Response::error("Invalid id", 400),
    };

    let d1 = ctx.d1("IFKASH_D1")?;
    if !owns_meal(&d1, owner_id, id).await? {
        return Response::error("not found", 404);
    }

    let payload: UpdatePayload = req.json().await?;

    d1.prepare(
        "UPDATE meals SET description = ?1, calories = ?2, protein_g = ?3, carbs_g = ?4, fat_g = ?5 \
         WHERE id = ?6 AND owner_id = ?7",
    )
    .bind(&[
        payload
            .description
            .map(|s| s.into())
            .unwrap_or(wasm_bindgen::JsValue::NULL),
        payload.calories.map(|v| v.into()).unwrap_or(wasm_bindgen::JsValue::NULL),
        payload.protein_g.map(|v| v.into()).unwrap_or(wasm_bindgen::JsValue::NULL),
        payload.carbs_g.map(|v| v.into()).unwrap_or(wasm_bindgen::JsValue::NULL),
        payload.fat_g.map(|v| v.into()).unwrap_or(wasm_bindgen::JsValue::NULL),
        n(id),
        n(owner_id),
    ])?
    .run()
    .await?;

    Response::from_json(&serde_json::json!({ "id": id }))
}

// ---- DELETE /api/meals/:id -------------------------------------------------

pub async fn delete(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
        Some(id) => id,
        None => return Response::error("Invalid id", 400),
    };

    let d1 = ctx.d1("IFKASH_D1")?;

    // Fetch the R2 key (if any) before deleting, so we can best-effort clean up.
    #[derive(Deserialize)]
    struct KeyRow {
        photo_r2_key: Option<String>,
    }
    let row: Option<KeyRow> = d1
        .prepare("SELECT photo_r2_key FROM meals WHERE id = ?1 AND owner_id = ?2")
        .bind(&[n(id), n(owner_id)])?
        .first(None)
        .await?;

    let key = match row {
        Some(r) => r.photo_r2_key,
        None => return Response::error("not found", 404),
    };

    d1.prepare("DELETE FROM meals WHERE id = ?1 AND owner_id = ?2")
        .bind(&[n(id), n(owner_id)])?
        .run()
        .await?;

    if let Some(k) = key {
        if let Ok(bucket) = ctx.bucket("RESUME_BUCKET") {
            let _ = bucket.delete(&k).await;
        }
    }

    Response::from_json(&serde_json::json!({ "id": id }))
}
