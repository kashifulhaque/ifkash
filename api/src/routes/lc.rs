use worker::*;
use crate::handlers;

pub async fn profile(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::lc::handle_profile(req, ctx).await
}

pub async fn submissions(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::lc::handle_submissions(req, ctx).await
}
