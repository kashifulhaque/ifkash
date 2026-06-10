use serde::Serialize;
use worker::*;

use super::access;

#[derive(Serialize)]
struct WhoamiResponse {
    email: String,
}

/// Returns the authenticated user's email. Protected the same way as the
/// resume write endpoints: Cloudflare Access at the edge plus the
/// `OWNER_EMAIL` pin. The frontend admin dashboard uses this both to gate
/// the page and to show who is signed in.
pub async fn whoami(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !access::is_authorized(&req, &ctx) {
        return access::unauthorized();
    }

    let email = req
        .headers()
        .get("Cf-Access-Authenticated-User-Email")?
        .filter(|e| !e.is_empty())
        // No Access header only happens via the local-dev bypass.
        .unwrap_or_else(|| "local-dev@localhost".to_string());

    Response::from_json(&WhoamiResponse { email })
}
