use worker::*;
use crate::handlers;

pub async fn route(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::resume::handle(req, ctx).await
}
