use worker::*;
use serde_json::json;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/api/hello", handle_hello)
        .get_async("/api/ping", handle_ping)
        .run(req, env)
        .await
}

async fn handle_hello(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("Hello from Cloudflare Workers! ✨")
}

async fn handle_ping(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::from_json(&json!({ "pong": true }))
}
