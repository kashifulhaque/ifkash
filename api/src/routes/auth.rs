use worker::*;
use crate::handlers;

pub async fn login(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::auth::login(req, ctx).await
}

pub async fn verify(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::auth::verify(req, ctx).await
}
