pub mod openapi;
pub mod swagger_ui;
pub mod resume;
pub mod resume_api;
pub mod home_weather;
pub mod whoami;

use worker::*;

pub fn register_routes(router: Router<'_, ()>) -> Router<'_, ()> {
  router
    .get_async("/api/openapi", openapi::route)
    .get_async("/api/docs", swagger_ui::route)
    .get_async("/api/resume", resume::route)
    .get_async("/api/resume/latest", resume_api::get_latest)
    .get_async("/api/resume/history", resume_api::get_history)
    .get_async("/api/resume/record/:id", resume_api::get_by_id)
    .post_async("/api/resume/upload", resume_api::upload)
    .get_async("/api/home_weather", home_weather::route)
    .get_async("/api/whoami", whoami::route)
}
