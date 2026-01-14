use worker::*;
use crate::handlers;

pub async fn profile(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::tensara::handle_profile(req, ctx).await
}
