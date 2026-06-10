use worker::*;
use crate::handlers;

pub async fn route(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::whoami::whoami(req, ctx).await
}
