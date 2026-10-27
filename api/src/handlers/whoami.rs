use serde::Serialize;
use worker::*;

use super::access;

#[derive(Serialize)]
struct WhoamiResponse {
    email: String,
}

/// Returns the authenticated user's email. Protected the same way as the
/// resume write endpoints: Cloudflare Access at the edge plus a verified
/// Access JWT pinned to `OWNER_EMAIL`. The frontend admin dashboard uses this
/// both to gate the page and to show who is signed in.
pub async fn whoami(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    match access::authorized_email(&req, &ctx).await {
        Some(email) => Response::from_json(&WhoamiResponse { email }),
        None => access::unauthorized(),
    }
}
