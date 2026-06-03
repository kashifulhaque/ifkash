use worker::*;

/// Returns true if the request is allowed to reach a protected endpoint.
///
/// Authentication is handled at the edge by Cloudflare Access, which injects
/// the `Cf-Access-Authenticated-User-Email` header on requests that have
/// passed an Access policy. We additionally pin the email to `OWNER_EMAIL`
/// (when configured) as defense-in-depth.
///
/// Under `wrangler dev` there is no Access in front of the Worker, and the
/// simulated request host is the configured route host (e.g. `ifkash.dev`),
/// not `localhost`. So local dev is detected via a `LOCAL_DEV` flag set in
/// `api/.dev.vars` (gitignored, never present in production).
pub fn is_authorized(req: &Request, ctx: &RouteContext<()>) -> bool {
    if is_local_dev(ctx) {
        return true;
    }

    // Fallback for dev setups that do surface a localhost request host.
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

/// Dev-only bypass driven by the `LOCAL_DEV` flag in `api/.dev.vars`. It is
/// never set in production, so Cloudflare Access stays enforced when deployed.
fn is_local_dev(ctx: &RouteContext<()>) -> bool {
    let truthy = |v: String| matches!(v.as_str(), "1" | "true" | "TRUE");
    if let Ok(v) = ctx.secret("LOCAL_DEV") {
        if truthy(v.to_string()) {
            return true;
        }
    }
    if let Ok(v) = ctx.var("LOCAL_DEV") {
        if truthy(v.to_string()) {
            return true;
        }
    }
    false
}

/// Standard 401 response for unauthenticated requests to protected endpoints.
pub fn unauthorized() -> Result<Response> {
    Response::error("Unauthorized", 401)
}
