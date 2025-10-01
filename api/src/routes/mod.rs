pub mod hello;
pub mod ping;
pub mod hn;

use worker::*;

pub fn register_routes(router: Router<'_, ()>) -> Router<'_, ()> {
    router
        .get_async("/api/hello", hello::route)
        .get_async("/api/ping", ping::route)
        .get_async("/api/hn", hn::route)
}
