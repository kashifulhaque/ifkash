use worker::*;

pub async fn handle(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("Hello from Cloudflare Workers! ✨")
}
