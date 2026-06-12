use worker::*;
use crate::handlers;

pub async fn submit_score(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::game::submit_score(req, ctx).await
}

pub async fn leaderboard(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::game::leaderboard(req, ctx).await
}
