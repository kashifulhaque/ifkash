use worker::*;
use serde_json::json;

#[utoipa::path(
    get,
    path = "/api/ping",
    tag = "Utils",
    responses(
        (status = 200, description = "Pong", body = inline(serde_json::Value))
    )
)]
pub async fn handle(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::from_json(&json!({ "pong": true }))
}
