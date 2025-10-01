use worker::*;
use crate::handlers;

pub async fn route(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::ping::handle(req, ctx).await
}
