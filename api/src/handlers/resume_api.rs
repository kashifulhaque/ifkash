use worker::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use wasm_bindgen::JsValue;
use base64::{engine::general_purpose::STANDARD, Engine as _};

use super::access;

const PUBLIC_PDF_URL: &str = "https://ifkash.dev/api/resume";
const PDF_FILENAME: &str = "Kashiful_Haque.pdf";
const TYPST_FILENAME: &str = "Kashiful_Haque.typ";
const STATIC_TYPST_URL: &str = "https://ifkash.dev/assets/Kashiful_Haque.typ";

/// A full row from the `resume_versions` D1 table.
#[derive(Debug, Deserialize)]
struct ResumeRow {
    id: i64,
    version: Option<String>,
    notes: Option<String>,
    typst_source: String,
    created: String,
    updated: String,
}

/// Metadata-only row (no source/PDF) used for the history listing.
#[derive(Debug, Deserialize)]
struct ResumeMetaRow {
    id: i64,
    version: Option<String>,
    notes: Option<String>,
    created: String,
    updated: String,
}

#[derive(Debug, Deserialize)]
struct InsertedRow {
    id: i64,
}

#[derive(Serialize, ToSchema)]
pub struct LatestResumeResponse {
    pub id: i64,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub created: String,
    pub updated: String,
    /// Raw Typst source for the editor to load.
    pub typst_source: String,
    pub pdf_url: String,
    pub pdf_filename: String,
    pub typst_filename: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResumeHistoryItem {
    pub id: i64,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub created: String,
    pub updated: String,
    pub pdf_url: String,
}

#[derive(Serialize, ToSchema)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub id: i64,
    pub version: Option<String>,
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn js_opt(value: &Option<String>) -> JsValue {
    match value {
        Some(s) => JsValue::from_str(s),
        None => JsValue::NULL,
    }
}

// GET /api/resume/latest - latest resume incl. Typst source (Access-protected)
#[utoipa::path(
    get,
    path = "/api/resume/latest",
    tag = "Resume API",
    responses(
        (status = 200, description = "Latest resume details", body = LatestResumeResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_latest(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !access::is_authorized(&req, &ctx) {
        return access::unauthorized();
    }

    let d1 = ctx.d1("IFKASH_D1")?;
    let row: Option<ResumeRow> = d1
        .prepare(
            "SELECT id, version, notes, typst_source, created, updated \
             FROM resume_versions ORDER BY datetime(created) DESC, id DESC LIMIT 1",
        )
        .first(None)
        .await?;

    match row {
        Some(r) => Response::from_json(&LatestResumeResponse {
            id: r.id,
            version: r.version,
            notes: r.notes,
            created: r.created,
            updated: r.updated,
            typst_source: r.typst_source,
            pdf_url: PUBLIC_PDF_URL.to_string(),
            pdf_filename: PDF_FILENAME.to_string(),
            typst_filename: TYPST_FILENAME.to_string(),
        }),
        // Empty table (fresh migration): seed the editor from the committed
        // static source so the first save creates the first D1 version.
        None => {
            let source = reqwest::Client::new()
                .get(STATIC_TYPST_URL)
                .send()
                .await
                .map_err(|e| Error::RustError(format!("Failed to load static Typst: {}", e)))?
                .text()
                .await
                .map_err(|e| Error::RustError(format!("Failed to read static Typst: {}", e)))?;

            Response::from_json(&LatestResumeResponse {
                id: 0,
                version: Some("0".to_string()),
                notes: None,
                created: now_iso(),
                updated: now_iso(),
                typst_source: source,
                pdf_url: PUBLIC_PDF_URL.to_string(),
                pdf_filename: PDF_FILENAME.to_string(),
                typst_filename: TYPST_FILENAME.to_string(),
            })
        }
    }
}

// POST /api/resume/upload - save a new version (Access-protected)
#[utoipa::path(
    post,
    path = "/api/resume/upload",
    tag = "Resume API",
    request_body(content = UploadResponse, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Upload successful", body = UploadResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    )
)]
pub async fn upload(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !access::is_authorized(&req, &ctx) {
        return access::unauthorized();
    }

    let form = req.form_data().await?;

    // PDF (rendered in the browser) arrives as a file part.
    let pdf_bytes = match form.get("pdf_file") {
        Some(FormEntry::File(f)) => f.bytes().await?,
        _ => return Response::error("Missing pdf_file", 400),
    };

    // Typst source arrives either as a text field or a file part.
    let typst_source = match form.get("typst_source") {
        Some(FormEntry::Field(s)) => s,
        _ => match form.get("typst_file") {
            Some(FormEntry::File(f)) => String::from_utf8(f.bytes().await?.to_vec())
                .map_err(|e| Error::RustError(format!("Invalid Typst UTF-8: {}", e)))?,
            _ => return Response::error("Missing typst_source", 400),
        },
    };

    let version = match form.get("version") {
        Some(FormEntry::Field(s)) if !s.is_empty() => Some(s),
        _ => None,
    };
    let notes = match form.get("notes") {
        Some(FormEntry::Field(s)) if !s.is_empty() => Some(s),
        _ => None,
    };

    let pdf_base64 = STANDARD.encode(pdf_bytes.as_slice());
    let now = now_iso();

    let d1 = ctx.d1("IFKASH_D1")?;
    let inserted: Option<InsertedRow> = d1
        .prepare(
            "INSERT INTO resume_versions (version, notes, typst_source, pdf_base64, created, updated) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING id",
        )
        .bind(&[
            js_opt(&version),
            js_opt(&notes),
            JsValue::from_str(&typst_source),
            JsValue::from_str(&pdf_base64),
            JsValue::from_str(&now),
            JsValue::from_str(&now),
        ])?
        .first(None)
        .await?;

    let id = inserted.map(|r| r.id).unwrap_or_default();

    Response::from_json(&UploadResponse {
        success: true,
        message: "Resume saved".to_string(),
        id,
        version,
    })
}

// GET /api/resume/history - list versions (Access-protected)
#[utoipa::path(
    get,
    path = "/api/resume/history",
    tag = "Resume API",
    responses(
        (status = 200, description = "Resume history", body = Vec<ResumeHistoryItem>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_history(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !access::is_authorized(&req, &ctx) {
        return access::unauthorized();
    }

    let d1 = ctx.d1("IFKASH_D1")?;
    let result = d1
        .prepare(
            "SELECT id, version, notes, created, updated \
             FROM resume_versions ORDER BY datetime(created) DESC, id DESC LIMIT 50",
        )
        .all()
        .await?;

    let rows: Vec<ResumeMetaRow> = result.results()?;
    let history: Vec<ResumeHistoryItem> = rows
        .into_iter()
        .map(|r| ResumeHistoryItem {
            id: r.id,
            version: r.version,
            notes: r.notes,
            created: r.created,
            updated: r.updated,
            pdf_url: format!("{}?id={}", PUBLIC_PDF_URL, r.id),
        })
        .collect();

    Response::from_json(&history)
}

// GET /api/resume/record/:id - load a specific version (Access-protected)
#[utoipa::path(
    get,
    path = "/api/resume/record/{id}",
    tag = "Resume API",
    params(
        ("id" = i64, Path, description = "Resume version ID")
    ),
    responses(
        (status = 200, description = "Resume details", body = LatestResumeResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_by_id(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !access::is_authorized(&req, &ctx) {
        return access::unauthorized();
    }

    let id = match ctx.param("id").and_then(|s| s.parse::<f64>().ok()) {
        Some(id) => id,
        None => return Response::error("Invalid ID", 400),
    };

    let d1 = ctx.d1("IFKASH_D1")?;
    let row: Option<ResumeRow> = d1
        .prepare(
            "SELECT id, version, notes, typst_source, created, updated \
             FROM resume_versions WHERE id = ?1",
        )
        .bind(&[JsValue::from_f64(id)])?
        .first(None)
        .await?;

    match row {
        Some(r) => Response::from_json(&LatestResumeResponse {
            id: r.id,
            version: r.version,
            notes: r.notes,
            created: r.created,
            updated: r.updated,
            typst_source: r.typst_source,
            pdf_url: format!("{}?id={}", PUBLIC_PDF_URL, r.id),
            pdf_filename: PDF_FILENAME.to_string(),
            typst_filename: TYPST_FILENAME.to_string(),
        }),
        None => Response::error("Resume not found", 404),
    }
}
