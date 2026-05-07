use worker::*;
use crate::handlers;

pub async fn route(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::home_weather::handle(req, ctx).await
}
