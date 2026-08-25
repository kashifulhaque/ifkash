use crate::handlers;
use worker::*;

pub async fn route(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::edge_signal::handle(req, ctx).await
}
