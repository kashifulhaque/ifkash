use worker::*;
use serde_json::json;

pub async fn handle(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::from_json(&json!({ "pong": true }))
}
