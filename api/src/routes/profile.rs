use crate::handlers;
use worker::*;

pub async fn get_profile(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::profile::get_profile(req, ctx).await
}

pub async fn upsert_profile(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::profile::upsert_profile(req, ctx).await
}
