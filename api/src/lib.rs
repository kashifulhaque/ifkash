mod handlers;
mod routes;

use worker::*;

fn apply_cors(mut resp: Response) -> Result<Response> {
    let h = resp.headers_mut();
    h.set("access-control-allow-origin", "*")?;
    h.set("access-control-allow-methods", "GET,POST,PUT,PATCH,DELETE,OPTIONS")?;
    h.set("access-control-allow-headers", "*")?;
    h.set("access-control-max-age", "86400")?;
    Ok(resp)
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    if req.method() == Method::Options {
        return apply_cors(Response::empty()?);
    }

    let router = routes::register_routes(Router::new());
    let resp = router.run(req, env).await?; // existing behavior

    apply_cors(resp)
}

/// Daily cron (`[triggers]` in wrangler.toml): refresh the research radar.
#[event(scheduled)]
async fn scheduled(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    match handlers::radar::refresh(&env).await {
        Ok(n) => console_log!("radar refresh stored up to {n} papers"),
        Err(e) => console_error!("radar refresh failed: {e}"),
    }
}
