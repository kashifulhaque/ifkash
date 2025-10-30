use worker::*;

pub async fn route(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let openapi_json = include_str!("../../../static/openapi.json");

    Response::ok(openapi_json)?
        .with_headers([
            ("Content-Type", "application/json"),
            ("Cache-Control", "public, max-age=3600"),
        ])
}