pub mod hello;
pub mod ping;
pub mod hn;
pub mod lc;
pub mod openapi;
pub mod swagger_ui;
pub mod resume;
pub mod auth;
pub mod resume_api;
pub mod typst;
pub mod ai;

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
    .get_async("/api/resume/latest", resume_api::get_latest)
    .get_async("/api/resume/history", resume_api::get_history)
    .get_async("/api/resume/record/:id", resume_api::get_by_id)
    .post_async("/api/resume/upload", resume_api::upload)
    .post_async("/api/typst/compile", typst::compile)
    .post_async("/api/ai/rewrite", ai::rewrite)
    .post_async("/api/auth/login", auth::login)
    .get_async("/api/auth/verify", auth::verify)
    .post_async("/api/lc/submissions", lc::submissions)
}
