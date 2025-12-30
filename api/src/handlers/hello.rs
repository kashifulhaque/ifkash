use worker::*;

#[utoipa::path(
    get,
    path = "/api/hello",
    tag = "Utils",
    responses(
        (status = 200, description = "Hello World", body = String)
    )
)]
pub async fn handle(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("Hello from Cloudflare Workers! ✨")
}
