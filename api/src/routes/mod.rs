pub mod hello;
pub mod ping;
pub mod hn;
pub mod lc;
pub mod openapi;

use worker::*;

pub fn register_routes(router: Router<'_, ()>) -> Router<'_, ()> {
    router
        .get_async("/api/hello", hello::route)
        .get_async("/api/ping", ping::route)
        .get_async("/api/openapi", openapi::route)
        .get_async("/api/hn", hn::route)
        .get_async("/api/lc/profile", lc::profile)
        .post_async("/api/lc/submissions", lc::submissions)
}
