use worker::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
struct PocketBaseAuthResponse {
    token: String,
}

#[derive(Debug, Deserialize)]
struct PocketBaseListResponse {
    items: Vec<ResumeRecord>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResumeRecord {
    id: String,
    #[serde(rename = "collectionId")]
    collection_id: String,
    pdf_file: String,
    typst_file: String,
    version: Option<String>,
    notes: Option<String>,
    created: String,
    updated: String,
}

async fn authenticate_pocketbase(
    client: &reqwest::Client,
    pocketbase_url: &str,
    email: &str,
    password: &str,
) -> Result<String> {
    let auth_url = format!("{}/api/collections/_superusers/auth-with-password", pocketbase_url);
    
    let auth_response = client
        .post(&auth_url)
        .header("Content-Type", "application/json")
        .json(&json!({
            "identity": email,
            "password": password
        }))
        .send()
        .await
        .map_err(|e| Error::RustError(format!("Failed to authenticate with PocketBase: {}", e)))?;

    if !auth_response.status().is_success() {
        return Err(Error::RustError("PocketBase authentication failed".into()));
    }

    let auth_data: PocketBaseAuthResponse = auth_response
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Failed to parse auth response: {}", e)))?;

    Ok(auth_data.token)
}

fn verify_auth_header(headers: &Headers) -> Result<String> {
    let auth_header = match headers.get("Authorization")? {
        Some(h) => h,
        None => return Err(Error::RustError("Missing Authorization header".into())),
    };

    if !auth_header.starts_with("Bearer ") {
        return Err(Error::RustError("Invalid Authorization header format".into()));
    }

    Ok(auth_header[7..].to_string())
}

// GET /api/resume/latest - Returns latest resume with Typst file URL (authenticated)
pub async fn get_latest(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Verify authentication
    let headers = req.headers();
    verify_auth_header(&headers)?;

    // Get PocketBase configuration
    let pocketbase_url = ctx.var("POCKETBASE_URL")?.to_string();
    let pb_email = ctx.secret("POCKETBASE_EMAIL")?.to_string();
    let pb_password = ctx.secret("POCKETBASE_PASSWORD")?.to_string();

    let client = reqwest::Client::new();

    // Authenticate with PocketBase
    let token = authenticate_pocketbase(&client, &pocketbase_url, &pb_email, &pb_password).await?;

    // Fetch the latest resume record
    let list_url = format!(
        "{}/api/collections/resumes/records?sort=-created&perPage=1",
        pocketbase_url
    );

    let pb_response = client
        .get(&list_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| Error::RustError(format!("Failed to fetch from PocketBase: {}", e)))?;

    if !pb_response.status().is_success() {
        return Response::error("Failed to fetch resume from PocketBase", 500);
    }

    let list_response: PocketBaseListResponse = pb_response
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Failed to parse PocketBase response: {}", e)))?;

    let resume = list_response
        .items
        .first()
        .ok_or_else(|| Error::RustError("No resume found in PocketBase".to_string()))?;

    // Construct file URLs
    let pdf_url = format!(
        "{}/api/files/{}/{}/{}",
        pocketbase_url, resume.collection_id, resume.id, resume.pdf_file
    );
    
    let typst_url = format!(
        "{}/api/files/{}/{}/{}",
        pocketbase_url, resume.collection_id, resume.id, resume.typst_file
    );

    Response::from_json(&json!({
        "id": resume.id,
        "version": resume.version,
        "notes": resume.notes,
        "created": resume.created,
        "updated": resume.updated,
        "pdf_url": pdf_url,
        "typst_url": typst_url,
        "pdf_filename": "Kashiful_Haque.pdf",
        "typst_filename": resume.typst_file,
    }))
}

// POST /api/resume/upload - Upload new resume version (authenticated)
// For MVP, this returns a placeholder - users should upload via PocketBase UI
pub async fn upload(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Verify authentication
    let headers = req.headers();
    verify_auth_header(&headers)?;

    // For MVP, return a message that upload should be done via PocketBase UI
    Response::from_json(&json!({
        "success": false,
        "message": "Upload functionality coming soon. Please upload via PocketBase admin panel for now.",
        "pocketbase_url": ctx.var("POCKETBASE_URL")?.to_string() + "/_/"
    }))
}
