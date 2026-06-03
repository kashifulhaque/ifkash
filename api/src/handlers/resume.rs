use worker::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use wasm_bindgen::JsValue;

// Statically hosted resume (served by Cloudflare Pages, independent of D1)
// used as a graceful fallback when the database is unavailable or empty.
const STATIC_RESUME_URL: &str = "https://ifkash.dev/assets/Kashiful_Haque.pdf";
const PUBLIC_RESUME_URL: &str = "https://ifkash.dev/api/resume";
const RESUME_FILENAME: &str = "Kashiful_Haque.pdf";

#[derive(Debug, Deserialize)]
struct PdfRow {
    id: i64,
    version: Option<String>,
    notes: Option<String>,
    created: String,
    updated: String,
    pdf_key: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResumeMetadataResponse {
    pub id: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub created: String,
    pub updated: String,
    pub pdf_url: String,
    pub pdf_filename: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResumeUrlResponse {
    pub url: String,
}

#[utoipa::path(
    get,
    path = "/api/resume",
    tag = "Resume",
    params(
        ("format" = Option<String>, Query, description = "Response format: 'json', 'url', 'view' (inline), 'download' (attachment), or omit for inline view"),
        ("id" = Option<i64>, Query, description = "Specific version ID (defaults to latest)")
    ),
    responses(
        (status = 200, description = "Resume metadata", body = ResumeMetadataResponse, content_type = "application/json"),
        (status = 200, description = "Download URL", body = ResumeUrlResponse, content_type = "application/json"),
        (status = 200, description = "PDF File", body = String, content_type = "application/pdf")
    )
)]
pub async fn handle(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;
    let format = url
        .query_pairs()
        .find(|(key, _)| key == "format")
        .map(|(_, value)| value.to_string());
    let id = url
        .query_pairs()
        .find(|(key, _)| key == "id")
        .and_then(|(_, value)| value.parse::<f64>().ok());

    match serve_from_d1(&ctx, format.as_deref(), id).await {
        Ok(response) => Ok(response),
        Err(e) => {
            // D1 unavailable or empty: degrade gracefully to the static PDF.
            console_log!("Resume: D1 unavailable, serving static fallback: {}", e);
            serve_static_fallback(format.as_deref()).await
        }
    }
}

async fn serve_from_d1(
    ctx: &RouteContext<()>,
    format: Option<&str>,
    id: Option<f64>,
) -> Result<Response> {
    let d1 = ctx.d1("IFKASH_D1")?;

    let row: Option<PdfRow> = match id {
        Some(id) => {
            d1.prepare(
                "SELECT id, version, notes, created, updated, pdf_key \
                 FROM resume_versions WHERE id = ?1",
            )
            .bind(&[JsValue::from_f64(id)])?
            .first(None)
            .await?
        }
        None => {
            d1.prepare(
                "SELECT id, version, notes, created, updated, pdf_key \
                 FROM resume_versions ORDER BY datetime(created) DESC, id DESC LIMIT 1",
            )
            .first(None)
            .await?
        }
    };

    let row = row.ok_or_else(|| Error::RustError("No resume found in D1".to_string()))?;

    match format {
        Some("json") => Response::from_json(&ResumeMetadataResponse {
            id: row.id.to_string(),
            version: row.version,
            notes: row.notes,
            created: row.created,
            updated: row.updated,
            pdf_url: PUBLIC_RESUME_URL.to_string(),
            pdf_filename: RESUME_FILENAME.to_string(),
        }),
        Some("url") => Response::from_json(&ResumeUrlResponse {
            url: PUBLIC_RESUME_URL.to_string(),
        }),
        other => {
            let disposition_type = if matches!(other, Some("attachment") | Some("download")) {
                "attachment"
            } else {
                "inline"
            };

            // Fetch the rendered PDF bytes from R2.
            let object = ctx
                .bucket("RESUME_BUCKET")?
                .get(&row.pdf_key)
                .execute()
                .await?
                .ok_or_else(|| Error::RustError("Resume PDF not found in R2".to_string()))?;
            let pdf_bytes = object
                .body()
                .ok_or_else(|| Error::RustError("Empty R2 object body".to_string()))?
                .bytes()
                .await?;

            pdf_response_from_bytes(pdf_bytes, disposition_type)
        }
    }
}

async fn serve_static_fallback(format: Option<&str>) -> Result<Response> {
    match format {
        Some("json") => Response::from_json(&ResumeMetadataResponse {
            id: String::new(),
            version: Some("static-fallback".to_string()),
            notes: Some("Served from static asset; database is currently unavailable.".to_string()),
            created: String::new(),
            updated: String::new(),
            pdf_url: STATIC_RESUME_URL.to_string(),
            pdf_filename: RESUME_FILENAME.to_string(),
        }),
        Some("url") => Response::from_json(&ResumeUrlResponse {
            url: STATIC_RESUME_URL.to_string(),
        }),
        other => {
            let disposition_type = if matches!(other, Some("attachment") | Some("download")) {
                "attachment"
            } else {
                "inline"
            };

            let pdf_response = reqwest::Client::new()
                .get(STATIC_RESUME_URL)
                .send()
                .await
                .map_err(|e| Error::RustError(format!("Failed to fetch static resume: {}", e)))?;

            if !pdf_response.status().is_success() {
                return Response::error("Resume temporarily unavailable", 502);
            }

            let pdf_bytes = pdf_response
                .bytes()
                .await
                .map_err(|e| Error::RustError(format!("Failed to read static resume bytes: {}", e)))?;

            pdf_response_from_bytes(pdf_bytes.to_vec(), disposition_type)
        }
    }
}

fn pdf_response_from_bytes(bytes: Vec<u8>, disposition_type: &str) -> Result<Response> {
    let headers = Headers::new();
    headers.set("Content-Type", "application/pdf")?;
    headers.set(
        "Content-Disposition",
        &format!("{}; filename=\"{}\"", disposition_type, RESUME_FILENAME),
    )?;
    headers.set("Cache-Control", "public, max-age=300")?;

    Ok(Response::from_bytes(bytes)?.with_headers(headers))
}
