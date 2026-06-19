use crate::handlers;
use worker::*;

pub async fn analyze(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::meals::analyze(req, ctx).await
}

pub async fn list(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::meals::list(req, ctx).await
}

pub async fn update(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::meals::update(req, ctx).await
}

pub async fn delete(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::meals::delete(req, ctx).await
}
