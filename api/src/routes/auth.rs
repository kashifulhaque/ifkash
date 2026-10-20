use crate::handlers;
use worker::*;

pub async fn create_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::google_auth::create_session(req, ctx).await
}

pub async fn delete_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::google_auth::delete_session(req, ctx).await
}
