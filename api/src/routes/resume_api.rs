use worker::*;
use crate::handlers;

pub async fn get_latest(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::resume_api::get_latest(req, ctx).await
}

pub async fn upload(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::resume_api::upload(req, ctx).await
}
