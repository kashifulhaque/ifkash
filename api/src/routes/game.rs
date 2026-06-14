use worker::*;
use crate::handlers;

pub async fn submit_score(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::game::submit_score(req, ctx).await
}

pub async fn leaderboard(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::game::leaderboard(req, ctx).await
}

pub async fn submit_daily_score(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::game::submit_daily_score(req, ctx).await
}

pub async fn daily_leaderboard(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::game::daily_leaderboard(req, ctx).await
}
