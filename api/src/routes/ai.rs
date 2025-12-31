use worker::*;
use crate::handlers;

pub async fn rewrite(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::ai::rewrite(req, ctx).await
}
