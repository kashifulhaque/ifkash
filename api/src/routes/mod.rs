pub mod hello;
pub mod ping;
pub mod hn;
pub mod lc;
pub mod openapi;
pub mod swagger_ui;
pub mod resume;

use worker::*;

pub fn register_routes(router: Router<'_, ()>) -> Router<'_, ()> {
  router
    .get_async("/api/hello", hello::route)
    .get_async("/api/ping", ping::route)
    .get_async("/api/openapi", openapi::route)
    .get_async("/api/docs", swagger_ui::route)
    .get_async("/api/hn", hn::route)
    .get_async("/api/lc/profile", lc::profile)
    .get_async("/api/resume", resume::route)
    .post_async("/api/lc/submissions", lc::submissions)
}
