use worker::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Deserialize)]
struct PocketBaseAuthResponse {
    token: String,
}

#[derive(Debug, Deserialize)]
struct PocketBaseListResponse {
    items: Vec<ResumeRecord>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ResumeRecord {
    pub id: String,
    #[serde(rename = "collectionId")]
    pub collection_id: String,
    pub pdf_file: String,
    pub typst_file: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Serialize, ToSchema)]
pub struct LatestResumeResponse {
    pub id: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub created: String,
    pub updated: String,
    pub pdf_url: String,
    pub typst_url: String,
    pub pdf_filename: String,
    pub typst_filename: String,
}

#[derive(Serialize, ToSchema)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub pocketbase_url: String,
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
#[utoipa::path(
    get,
    path = "/api/resume/latest",
    tag = "Resume API",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Latest resume details", body = LatestResumeResponse),
        (status = 401, description = "Unauthorized")
    )
)]
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

    Response::from_json(&LatestResumeResponse {
        id: resume.id.clone(),
        version: resume.version.clone(),
        notes: resume.notes.clone(),
        created: resume.created.clone(),
        updated: resume.updated.clone(),
        pdf_url,
        typst_url,
        pdf_filename: "Kashiful_Haque.pdf".to_string(),
        typst_filename: resume.typst_file.clone(),
    })
}

// POST /api/resume/upload - Upload new resume version (authenticated)
// For MVP, this returns a placeholder - users should upload via PocketBase UI
#[utoipa::path(
    post,
    path = "/api/resume/upload",
    tag = "Resume API",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Upload instructions", body = UploadResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn upload(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Verify authentication
    let headers = req.headers();
    verify_auth_header(&headers)?;

    // For MVP, return a message that upload should be done via PocketBase UI
    Response::from_json(&UploadResponse {
        success: false,
        message: "Upload functionality coming soon. Please upload via PocketBase admin panel for now.".to_string(),
        pocketbase_url: ctx.var("POCKETBASE_URL")?.to_string() + "/_/"
    })
}
