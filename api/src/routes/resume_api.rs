use worker::*;
use crate::handlers;

pub async fn get_latest(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::resume_api::get_latest(req, ctx).await
}

pub async fn get_history(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::resume_api::get_history(req, ctx).await
}

pub async fn get_by_id(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::resume_api::get_by_id(req, ctx).await
}

pub async fn upload(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::resume_api::upload(req, ctx).await
}
