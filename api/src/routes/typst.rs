use worker::*;
use crate::handlers;

pub async fn compile(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::typst::compile(req, ctx).await
}
