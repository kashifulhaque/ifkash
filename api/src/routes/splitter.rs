use crate::handlers;
use worker::*;

pub async fn list_groups(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::splitter::list_groups(req, ctx).await
}

pub async fn create_group(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::splitter::create_group(req, ctx).await
}

pub async fn get_group(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::splitter::get_group(req, ctx).await
}

pub async fn delete_group(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::splitter::delete_group(req, ctx).await
}

pub async fn add_member(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::splitter::add_member(req, ctx).await
}

pub async fn delete_member(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::splitter::delete_member(req, ctx).await
}

pub async fn add_expense(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::splitter::add_expense(req, ctx).await
}

pub async fn delete_expense(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    handlers::splitter::delete_expense(req, ctx).await
}
