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
        let status = auth_response.status();
        let error_text = auth_response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(Error::RustError(format!(
            "PocketBase authentication failed ({}): {}",
            status, error_text
        )));
    }

    let auth_data: PocketBaseAuthResponse = auth_response
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Failed to parse auth response: {}", e)))?;

    Ok(auth_data.token)
}

pub async fn handle(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Get PocketBase configuration from environment
    let pocketbase_url = ctx.var("POCKETBASE_URL")?.to_string();
    let pb_email = ctx.secret("POCKETBASE_EMAIL")?.to_string();
    let pb_password = ctx.secret("POCKETBASE_PASSWORD")?.to_string();

    let client = reqwest::Client::new();

    // Authenticate with PocketBase to get JWT token
    let token = authenticate_pocketbase(&client, &pocketbase_url, &pb_email, &pb_password).await?;

    // Parse query parameters
    let url = req.url()?;
    let format = url
        .query_pairs()
        .find(|(key, _)| key == "format")
        .map(|(_, value)| value.to_string());

    // Fetch the latest resume record from PocketBase
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
        let status = pb_response.status();
        let error_text = pb_response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Response::error(
            format!("PocketBase API error ({}): {}", status, error_text),
            500,
        );
    }

    let list_response: PocketBaseListResponse = pb_response
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Failed to parse PocketBase response: {}", e)))?;

    let resume = list_response
        .items
        .first()
        .ok_or_else(|| Error::RustError("No resume found in PocketBase".to_string()))?;

    // Construct the file URL
    let file_url = format!(
        "{}/api/files/{}/{}/{}",
        pocketbase_url, resume.collection_id, resume.id, resume.pdf_file
    );

    // Handle different response formats
    match format.as_deref() {
        Some("json") => {
            // Return metadata with download URL
            Response::from_json(&json!({
                "id": resume.id,
                "version": resume.version,
                "notes": resume.notes,
                "created": resume.created,
                "updated": resume.updated,
                "pdf_url": file_url,
                "pdf_filename": "Kashiful_Haque.pdf",
            }))
        }
        Some("url") => {
            // Return just the URL
            Response::from_json(&json!({
                "url": file_url
            }))
        }
        _ => {
            // Default: Download the PDF and return it with the correct filename
            let pdf_response = client
                .get(&file_url)
                .send()
                .await
                .map_err(|e| Error::RustError(format!("Failed to download PDF: {}", e)))?;

            if !pdf_response.status().is_success() {
                return Response::error("Failed to download PDF from PocketBase", 500);
            }

            let pdf_bytes = pdf_response
                .bytes()
                .await
                .map_err(|e| Error::RustError(format!("Failed to read PDF bytes: {}", e)))?;

            let headers = Headers::new();
            headers.set("Content-Type", "application/pdf")?;
            headers.set(
                "Content-Disposition",
                "attachment; filename=\"Kashiful_Haque.pdf\"",
            )?;

            Ok(Response::from_bytes(pdf_bytes.to_vec())?
                .with_headers(headers))
        }
    }
}
