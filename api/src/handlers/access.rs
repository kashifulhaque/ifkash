use worker::*;

/// Returns true if the request is allowed to reach a protected endpoint.
///
/// Authentication is handled at the edge by Cloudflare Access, which injects
/// the `Cf-Access-Authenticated-User-Email` header on requests that have
/// passed an Access policy. We additionally pin the email to `OWNER_EMAIL`
/// (when configured) as defense-in-depth.
///
/// Local development (`wrangler dev` on localhost) bypasses the check so the
/// editor still works without Access in front of it.
pub fn is_authorized(req: &Request, ctx: &RouteContext<()>) -> bool {
    if let Ok(url) = req.url() {
        if matches!(url.host_str(), Some("localhost") | Some("127.0.0.1")) {
            return true;
        }
    }

    let email = match req.headers().get("Cf-Access-Authenticated-User-Email") {
        Ok(Some(email)) if !email.is_empty() => email,
        _ => return false,
    };

    if let Ok(owner) = ctx.var("OWNER_EMAIL") {
        let owner = owner.to_string();
        if !owner.is_empty() && owner != email {
            return false;
        }
    }

    true
}

/// Standard 401 response for unauthenticated requests to protected endpoints.
pub fn unauthorized() -> Result<Response> {
    Response::error("Unauthorized", 401)
}
