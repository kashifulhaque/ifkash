use crate::handlers;
use worker::*;

pub async fn list_sessions(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::workout::list_sessions(req, ctx).await
}

pub async fn create_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::workout::create_session(req, ctx).await
}

pub async fn upsert_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::workout::upsert_session(req, ctx).await
}

pub async fn get_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::workout::get_session(req, ctx).await
}

pub async fn delete_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::workout::delete_session(req, ctx).await
}

pub async fn list_bodyweight(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::workout::list_bodyweight(req, ctx).await
}

pub async fn add_bodyweight(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::workout::add_bodyweight(req, ctx).await
}

pub async fn delete_bodyweight(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::workout::delete_bodyweight(req, ctx).await
}
