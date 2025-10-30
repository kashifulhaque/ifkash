use worker::*;

pub async fn route(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let openapi_json = include_str!("../../../static/openapi.json");

    let mut resp = Response::ok(openapi_json)?;
    resp.headers_mut().set("Content-Type", "application/json")?;
    resp.headers_mut().set("Cache-Control", "public, max-age=3600")?;
    Ok(resp)
}
