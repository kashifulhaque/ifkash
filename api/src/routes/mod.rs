pub mod openapi;
pub mod swagger_ui;
pub mod resume;
pub mod resume_api;
pub mod home_weather;
pub mod edge_signal;
pub mod whoami;
pub mod game;
pub mod splitter;
pub mod workout;
pub mod profile;
pub mod auth;

use crate::handlers;
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
    .get_async("/api/edge-signal", edge_signal::route)
    .get_async("/api/whoami", whoami::route)
    .post_async("/api/game/score", game::submit_score)
    .get_async("/api/game/leaderboard", game::leaderboard)
    .post_async("/api/game/daily/score", game::submit_daily_score)
    .get_async("/api/game/daily/leaderboard", game::daily_leaderboard)
    // Exchange a Google ID token for a longer session. The session is what the
    // fitness and splitter routes accept when the ID token has expired.
    .post_async("/api/auth/session", auth::create_session)
    .delete_async("/api/auth/session", auth::delete_session)
    // Expense splitter — all endpoints require a Google ID token in the
    // `Authorization: Bearer <token>` header; data is scoped to that user.
    .get_async("/api/splitter/groups", splitter::list_groups)
    .post_async("/api/splitter/groups", splitter::create_group)
    .get_async("/api/splitter/groups/:id", splitter::get_group)
    .delete_async("/api/splitter/groups/:id", splitter::delete_group)
    .post_async("/api/splitter/groups/:id/members", splitter::add_member)
    .delete_async("/api/splitter/members/:id", splitter::delete_member)
    .post_async("/api/splitter/groups/:id/expenses", splitter::add_expense)
    .delete_async("/api/splitter/expenses/:id", splitter::delete_expense)
    // Workout tracker — bearer is a Google ID token or a session minted from
    // one. Data is scoped to that user.
    .get_async("/api/workout/sessions", workout::list_sessions)
    .post_async("/api/workout/sessions", workout::create_session)
    .put_async("/api/workout/sessions", workout::upsert_session)
    .get_async("/api/workout/sessions/:id", workout::get_session)
    .delete_async("/api/workout/sessions/:id", workout::delete_session)
    .get_async("/api/workout/log", workout::list_log)
    .get_async("/api/workout/bodyweight", workout::list_bodyweight)
    .post_async("/api/workout/bodyweight", workout::add_bodyweight)
    .delete_async("/api/workout/bodyweight/:id", workout::delete_bodyweight)
    // Body profile — drives the BMI/BMR/TDEE metrics and daily nutrition targets.
    // Same bearer as the workout routes; data is scoped to that user.
    .get_async("/api/profile", profile::get_profile)
    .put_async("/api/profile", profile::upsert_profile)
    // Cinderheart (latticefall) online features. Reads are anonymous; writes need a
    // bearer (Google ID token or minted session) and are rate limited per user.
    // Telemetry is opt-in and anonymous.
    .get_async("/api/latticefall/messages", handlers::latticefall::list_messages)
    .post_async("/api/latticefall/messages", handlers::latticefall::post_message)
    .post_async("/api/latticefall/messages/:id/rate", handlers::latticefall::rate_message)
    .get_async("/api/latticefall/afterimages", handlers::latticefall::list_afterimages)
    .post_async("/api/latticefall/afterimages", handlers::latticefall::post_afterimage)
    .get_async("/api/latticefall/wanderers", handlers::latticefall::list_wanderers)
    .post_async("/api/latticefall/wanderers", handlers::latticefall::post_wanderer)
    .get_async("/api/latticefall/save", handlers::latticefall::get_save)
    .put_async("/api/latticefall/save", handlers::latticefall::put_save)
    .delete_async("/api/latticefall/save", handlers::latticefall::delete_save)
    .get_async("/api/latticefall/marks", handlers::latticefall::get_marks)
    .put_async("/api/latticefall/marks", handlers::latticefall::put_marks)
    .get_async("/api/latticefall/speedrun/leaderboard", handlers::latticefall::speedrun_leaderboard)
    .post_async("/api/latticefall/speedrun", handlers::latticefall::post_speedrun)
    .post_async("/api/latticefall/telemetry", handlers::latticefall::post_telemetry)
}
