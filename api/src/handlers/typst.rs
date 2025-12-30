use worker::*;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompileRequest {
    pub code: String,
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

#[utoipa::path(
    post,
    path = "/api/typst/compile",
    tag = "Typst",
    security(
        ("bearer_auth" = [])
    ),
    request_body = CompileRequest,
    responses(
        (status = 200, description = "Compiled PDF", body = String, content_type = "application/pdf"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Compilation failure")
    )
)]
pub async fn compile(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    console_log!("Typst compile endpoint called");

    // Verify authentication
    let headers = req.headers();
    let token = verify_auth_header(&headers)?;

    // Parse request
    let compile_req: CompileRequest = req.json().await?;
    console_log!("Compiling Typst code ({} bytes)", compile_req.code.len());

    // Get Typst service URL
    let typst_service_url = match ctx.var("TYPST_SERVICE_URL") {
        Ok(url) => url.to_string(),
        Err(_) => "https://typst.ifkash.dev".to_string(),
    };

    let compile_url = format!("{}/compile", typst_service_url);

    // Forward request to Typst service
    let client = reqwest::Client::new();
    let response = client
        .post(&compile_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "code": compile_req.code
        }))
        .send()
        .await
        .map_err(|e| Error::RustError(format!("Failed to call Typst service: {}", e)))?;

    let status = response.status();
    console_log!("Typst service response: {}", status);

    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        
        console_log!("Typst compilation failed: {}", error_body);
        
        return Response::error(
            format!("Compilation failed: {}", error_body),
            status.as_u16()
        );
    }

    // Get PDF bytes
    let pdf_bytes = response
        .bytes()
        .await
        .map_err(|e| Error::RustError(format!("Failed to read PDF: {}", e)))?;

    console_log!("Compilation successful ({} bytes)", pdf_bytes.len());

    // Return PDF
    let headers = Headers::new();
    headers.set("Content-Type", "application/pdf")?;
    headers.set("Content-Length", &pdf_bytes.len().to_string())?;
    headers.set("Access-Control-Allow-Origin", "*")?;

    Ok(Response::from_bytes(pdf_bytes.to_vec())?.with_headers(headers))
}
