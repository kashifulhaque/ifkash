use super::access::is_local_dev;
use serde::{Deserialize, Serialize};
use worker::*;

/// How long a minted session lasts. Long enough that signing in before a
/// workout still covers a session that runs past the hour, and a phone that
/// sleeps through the Google refresh timer stays signed in.
const SESSION_TTL_SECS: i64 = 48 * 60 * 60;
/// Renew a session that is inside this window, so opening the app keeps the
/// clock moving without a write on every request.
const RENEW_WITHIN_SECS: i64 = 36 * 60 * 60;

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

fn now_secs() -> i64 {
    (worker::Date::now().as_millis() / 1000) as i64
}

fn bearer(req: &Request) -> Result<String> {
    let auth = req.headers().get("Authorization")?.unwrap_or_default();
    Ok(auth
        .strip_prefix("Bearer ")
        .unwrap_or(auth.as_str())
        .trim()
        .to_string())
}

fn new_token() -> Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|e| Error::RustError(e.to_string()))?;
    let mut token = String::from("sess_");
    for b in bytes {
        token.push_str(&format!("{b:02x}"));
    }
    Ok(token)
}

#[derive(Deserialize)]
struct SessionRow {
    google_sub: String,
    email: String,
    name: String,
    picture: String,
    expires_at: i64,
}

/// Resolve a bearer token to a Google identity. Session tokens (minted by
/// `create_session`) are checked locally. Anything else is a Google ID token
/// and is verified with Google.
pub async fn identity_from_bearer(
    token: &str,
    client_id: &str,
    d1: &D1Database,
) -> std::result::Result<GoogleIdentity, Response> {
    if token.starts_with("sess_") {
        return match session_identity(d1, token).await {
            Ok(Some(id)) => Ok(id),
            Ok(None) => Err(Response::error("invalid token", 401).unwrap()),
            Err(_) => Err(Response::error("session lookup failed", 500).unwrap()),
        };
    }
    verify_google(token, client_id).await
}

async fn session_identity(d1: &D1Database, token: &str) -> Result<Option<GoogleIdentity>> {
    let row: Option<SessionRow> = d1
        .prepare(
            "SELECT google_sub, email, name, picture, expires_at \
             FROM auth_sessions WHERE token = ?1",
        )
        .bind(&[token.into()])?
        .first(None)
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let now = now_secs();
    if row.expires_at <= now {
        let _ = d1
            .prepare("DELETE FROM auth_sessions WHERE token = ?1")
            .bind(&[token.into()])?
            .run()
            .await;
        return Ok(None);
    }
    if row.expires_at - now < RENEW_WITHIN_SECS {
        let next = now + SESSION_TTL_SECS;
        let _ = d1
            .prepare("UPDATE auth_sessions SET expires_at = ?1 WHERE token = ?2")
            .bind(&[(next as f64).into(), token.into()])?
            .run()
            .await;
    }
    Ok(Some(GoogleIdentity {
        aud: String::new(),
        iss: "session".to_string(),
        sub: row.google_sub,
        email: Some(row.email),
        name: Some(row.name),
        picture: Some(row.picture),
    }))
}

#[derive(Serialize)]
struct SessionResponse {
    token: String,
    expires_at: i64,
    sub: String,
}

/// POST /api/auth/session — exchange a live Google ID token for a session.
pub async fn create_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if is_local_dev(&ctx) {
        let exp = now_secs() + SESSION_TTL_SECS;
        return Response::from_json(&SessionResponse {
            token: "local-dev".to_string(),
            expires_at: exp,
            sub: "local-dev-user".to_string(),
        });
    }

    let token = bearer(&req)?;
    if token.is_empty() {
        return Response::error("unauthorized", 401);
    }
    let client_id = ctx.env.var("GOOGLE_CLIENT_ID")?.to_string();
    let info = match verify_google(&token, &client_id).await {
        Ok(i) => i,
        Err(resp) => return Ok(resp),
    };

    let d1 = ctx.d1("IFKASH_D1")?;
    let now = now_secs();
    let _ = d1
        .prepare("DELETE FROM auth_sessions WHERE expires_at <= ?1")
        .bind(&[(now as f64).into()])?
        .run()
        .await;

    let session = new_token()?;
    let exp = now + SESSION_TTL_SECS;
    let email = info.email.clone().unwrap_or_default();
    let name = info.name.clone().unwrap_or_default();
    let picture = info.picture.clone().unwrap_or_default();
    d1.prepare(
        "INSERT INTO auth_sessions (token, google_sub, email, name, picture, expires_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&[
        session.clone().into(),
        info.sub.clone().into(),
        email.into(),
        name.into(),
        picture.into(),
        (exp as f64).into(),
    ])?
    .run()
    .await?;

    Response::from_json(&SessionResponse {
        token: session,
        expires_at: exp,
        sub: info.sub,
    })
}

/// DELETE /api/auth/session — revoke the session presented as the bearer.
pub async fn delete_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let token = bearer(&req)?;
    if token.starts_with("sess_") {
        let d1 = ctx.d1("IFKASH_D1")?;
        d1.prepare("DELETE FROM auth_sessions WHERE token = ?1")
            .bind(&[token.into()])?
            .run()
            .await?;
    }
    Response::from_json(&serde_json::json!({ "ok": true }))
}
