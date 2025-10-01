mod handlers;
mod routes;

use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    let router = routes::register_routes(router);

    router.run(req, env).await
}
