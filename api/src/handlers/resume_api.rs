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
#[utoipa::path(
    post,
    path = "/api/resume/upload",
    tag = "Resume API",
    security(
        ("bearer_auth" = [])
    ),
    request_body(content = UploadResponse, content_type = "multipart/form-data"), 
    responses(
        (status = 200, description = "Upload successful", body = UploadResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    )
)]
pub async fn upload(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Verify authentication
    let headers = req.headers();
    verify_auth_header(&headers)?;

    // Parse multipart form data
    let form = req.form_data().await?;

    // Get PDF file
    let pdf_entry = form.get("pdf_file").ok_or(Error::RustError("Missing pdf_file".into()))?;
    let (pdf_name, pdf_bytes) = match pdf_entry {
        FormEntry::File(f) => (f.name(), f.bytes().await?),
        _ => return Err(Error::RustError("pdf_file must be a file".into())),
    };

    // Get Typst file
    let typst_entry = form.get("typst_file").ok_or(Error::RustError("Missing typst_file".into()))?;
    let (typst_name, typst_bytes) = match typst_entry {
        FormEntry::File(f) => (f.name(), f.bytes().await?),
        _ => return Err(Error::RustError("typst_file must be a file".into())),
    };
    
    // Get version if provided
    let version = form.get("version").map(|e| match e {
        FormEntry::Field(s) => s,
        _ => "".to_string(),
    });

    // Get PocketBase configuration
    let pocketbase_url = ctx.var("POCKETBASE_URL")?.to_string();
    let pb_email = ctx.secret("POCKETBASE_EMAIL")?.to_string();
    let pb_password = ctx.secret("POCKETBASE_PASSWORD")?.to_string();

    let client = reqwest::Client::new();

    // Authenticate with PocketBase
    let token = authenticate_pocketbase(&client, &pocketbase_url, &pb_email, &pb_password).await?;

    // Create Multipart Form for PocketBase Request
    let mut multipart = reqwest::multipart::Form::new()
        .part("pdf_file", reqwest::multipart::Part::bytes(pdf_bytes).file_name(pdf_name))
        .part("typst_file", reqwest::multipart::Part::bytes(typst_bytes).file_name(typst_name));

    if let Some(v) = version {
        if !v.is_empty() {
             multipart = multipart.text("version", v);
        }
    }

    // Send to PocketBase to create new record
    let create_url = format!("{}/api/collections/resumes/records", pocketbase_url);
    
    let pb_response = client
        .post(&create_url)
        .header("Authorization", format!("Bearer {}", token))
        .multipart(multipart)
        .send()
        .await
        .map_err(|e| Error::RustError(format!("Failed to upload to PocketBase: {}", e)))?;
        
    if !pb_response.status().is_success() {
         let text = pb_response.text().await.unwrap_or_default();
         return Err(Error::RustError(format!("PocketBase upload failed: {}", text)));
    }

    // Return success response
    Response::from_json(&UploadResponse {
        success: true,
        message: "Resume uploaded successfully".to_string(),
        pocketbase_url: format!("{}/_/", pocketbase_url),
    })
}

// GET /api/resume/history - Returns list of resume versions (authenticated)
#[utoipa::path(
    get,
    path = "/api/resume/history",
    tag = "Resume API",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Resume history", body = Vec<LatestResumeResponse>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_history(req: Request, ctx: RouteContext<()>) -> Result<Response> {
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

    // Fetch resume records (limit 50)
    let list_url = format!(
        "{}/api/collections/resumes/records?sort=-created&perPage=50",
        pocketbase_url
    );

    let pb_response = client
        .get(&list_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| Error::RustError(format!("Failed to fetch from PocketBase: {}", e)))?;

    if !pb_response.status().is_success() {
        return Response::error("Failed to fetch resume history", 500);
    }

    let list_response: PocketBaseListResponse = pb_response
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Failed to parse PocketBase response: {}", e)))?;

    let history: Vec<LatestResumeResponse> = list_response
        .items
        .into_iter()
        .map(|resume| {
            let pdf_url = format!(
                "{}/api/files/{}/{}/{}",
                pocketbase_url, resume.collection_id, resume.id, resume.pdf_file
            );
            
            let typst_url = format!(
                "{}/api/files/{}/{}/{}",
                pocketbase_url, resume.collection_id, resume.id, resume.typst_file
            );

            LatestResumeResponse {
                id: resume.id,
                version: resume.version,
                notes: resume.notes,
                created: resume.created,
                updated: resume.updated,
                pdf_url,
                typst_url,
                pdf_filename: "Kashiful_Haque.pdf".to_string(),
                typst_filename: resume.typst_file,
            }
        })
        .collect();

    Response::from_json(&history)
}

// GET /api/resume/record/:id - Returns specific resume record (authenticated)
#[utoipa::path(
    get,
    path = "/api/resume/record/{id}",
    tag = "Resume API",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Resume record ID")
    ),
    responses(
        (status = 200, description = "Resume details", body = LatestResumeResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_by_id(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Verify authentication
    let headers = req.headers();
    verify_auth_header(&headers)?;

    let id = ctx.param("id").unwrap_or(&"".to_string()).to_string();
    if id.is_empty() {
        return Response::error("Missing ID", 400);
    }

    // Get PocketBase configuration
    let pocketbase_url = ctx.var("POCKETBASE_URL")?.to_string();
    let pb_email = ctx.secret("POCKETBASE_EMAIL")?.to_string();
    let pb_password = ctx.secret("POCKETBASE_PASSWORD")?.to_string();

    let client = reqwest::Client::new();

    // Authenticate with PocketBase
    let token = authenticate_pocketbase(&client, &pocketbase_url, &pb_email, &pb_password).await?;

    // Fetch specific record
    let record_url = format!(
        "{}/api/collections/resumes/records/{}",
        pocketbase_url, id
    );

    let pb_response = client
        .get(&record_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| Error::RustError(format!("Failed to fetch from PocketBase: {}", e)))?;

    if pb_response.status() == reqwest::StatusCode::NOT_FOUND {
        return Response::error("Resume not found", 404);
    }
    
    if !pb_response.status().is_success() {
        return Response::error("Failed to fetch resume from PocketBase", 500);
    }

    let resume: ResumeRecord = pb_response
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Failed to parse PocketBase response: {}", e)))?;

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
        id: resume.id,
        version: resume.version,
        notes: resume.notes,
        created: resume.created,
        updated: resume.updated,
        pdf_url,
        typst_url,
        pdf_filename: "Kashiful_Haque.pdf".to_string(),
        typst_filename: resume.typst_file,
    })
}
