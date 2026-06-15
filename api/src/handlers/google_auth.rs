use serde::Deserialize;
use worker::*;

/// Subset of Google's tokeninfo response we care about.
#[derive(Deserialize)]
pub struct GoogleIdentity {
    pub aud: String,
    pub iss: String,
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

/// Verify a Google ID token via Google's tokeninfo endpoint. This avoids
/// shipping a JWT/crypto stack to wasm; Google validates the signature and
/// expiry, we validate audience and issuer. On failure returns a ready-to-send
/// 401 `Response`.
pub async fn verify_google(
    id_token: &str,
    client_id: &str,
) -> std::result::Result<GoogleIdentity, Response> {
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        urlencoding::encode(id_token)
    );
    let unauthorized = || Response::error("invalid token", 401).unwrap();
    let resp = reqwest::get(&url).await.map_err(|_| unauthorized())?;
    if !resp.status().is_success() {
        return Err(unauthorized());
    }
    let info: GoogleIdentity = resp.json().await.map_err(|_| unauthorized())?;
    if info.aud != client_id {
        return Err(unauthorized());
    }
    if info.iss != "https://accounts.google.com" && info.iss != "accounts.google.com" {
        return Err(unauthorized());
    }
    Ok(info)
}
