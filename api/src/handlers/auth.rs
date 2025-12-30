use worker::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
struct PocketBaseAuthResponse {
    token: String,
    record: PocketBaseAdminRecord,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PocketBaseAdminRecord {
    pub id: String,
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub success: bool,
    pub token: String,
    pub user: PocketBaseAdminRecord,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyResponse {
    pub valid: bool,
}

async fn authenticate_with_pocketbase(
    client: &reqwest::Client,
    pocketbase_url: &str,
    email: &str,
    password: &str,
) -> Result<PocketBaseAuthResponse> {
    let auth_url = format!("{}/api/collections/_superusers/auth-with-password", pocketbase_url);
    
    console_log!("Authenticating with PocketBase: {}", auth_url);
    console_log!("Email: {}", email);
    
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

    let status = auth_response.status();
    console_log!("PocketBase response status: {}", status);

    if !status.is_success() {
        let error_body = auth_response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        
        console_log!("PocketBase error body: {}", error_body);
        
        return Err(Error::RustError(format!(
            "PocketBase authentication failed ({}): {}",
            status, error_body
        )));
    }

    let auth_data: PocketBaseAuthResponse = auth_response
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Failed to parse auth response: {}", e)))?;

    Ok(auth_data)
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn login(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    console_log!("Login endpoint called");
    
    // Parse login request
    let login_req: LoginRequest = req.json().await?;
    console_log!("Received login request for: {}", login_req.email);

    // Get PocketBase configuration
    let pocketbase_url = ctx.var("POCKETBASE_URL")?.to_string();
    
    let client = reqwest::Client::new();

    // Authenticate with PocketBase
    let auth_data = authenticate_with_pocketbase(
        &client,
        &pocketbase_url,
        &login_req.email,
        &login_req.password,
    ).await?;

    console_log!("Authentication successful");

    // For now, we'll return the PocketBase token directly
    // In a production system, you might want to issue your own JWT
    Response::from_json(&LoginResponse {
        success: true,
        token: auth_data.token,
        user: auth_data.record,
    })
}

#[utoipa::path(
    get,
    path = "/api/auth/verify",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Token is valid", body = VerifyResponse),
        (status = 401, description = "Invalid token")
    )
)]
pub async fn verify(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    // Extract Authorization header
    let headers = req.headers();
    let auth_header = match headers.get("Authorization")? {
        Some(h) => h,
        None => return Response::error("Missing Authorization header", 401),
    };

    // Check if it starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return Response::error("Invalid Authorization header format", 401);
    }

    let token = &auth_header[7..]; // Skip "Bearer "

    // For now, just return success if token exists
    // In production, you'd verify the JWT signature
    if token.is_empty() {
        return Response::error("Invalid token", 401);
    }

    Response::from_json(&VerifyResponse { valid: true })
}
