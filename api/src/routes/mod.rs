pub mod openapi;
pub mod swagger_ui;
pub mod resume;
pub mod resume_api;
pub mod home_weather;
pub mod whoami;
pub mod game;

use worker::*;

pub fn register_routes(router: Router<'_, ()>) -> Router<'_, ()> {
  router
    .get_async("/api/openapi", openapi::route)
    .get_async("/api/docs", swagger_ui::route)
    // Public, unauthenticated resume PDF for the homepage button. Lives on its
    // own `/api/cv` prefix (NOT under `/api/resume`) so the Cloudflare Access
    // application can protect the whole `/api/resume*` admin surface without
    // catching the public PDF, which shares no path prefix with it.
    .get_async("/api/cv", resume::route)
    // `/api/resume*` below is the admin/editor surface — keep it behind
    // Cloudflare Access at the edge.
    .get_async("/api/resume", resume::route)
    .get_async("/api/resume/latest", resume_api::get_latest)
    .get_async("/api/resume/history", resume_api::get_history)
    .get_async("/api/resume/record/:id", resume_api::get_by_id)
    .post_async("/api/resume/upload", resume_api::upload)
    .get_async("/api/home_weather", home_weather::route)
    .get_async("/api/whoami", whoami::route)
    .post_async("/api/game/score", game::submit_score)
    .get_async("/api/game/leaderboard", game::leaderboard)
}
