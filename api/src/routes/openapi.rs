use worker::*;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    OpenApi,
};
use crate::handlers;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::resume::handle,
        handlers::resume_api::get_latest,
        handlers::resume_api::get_history,
        handlers::resume_api::get_by_id,
        handlers::resume_api::upload,
        handlers::home_weather::handle,
        handlers::edge_signal::handle,
        handlers::latticefall::list_messages,
        handlers::latticefall::post_message,
        handlers::latticefall::rate_message,
        handlers::latticefall::list_afterimages,
        handlers::latticefall::post_afterimage,
        handlers::latticefall::list_wanderers,
        handlers::latticefall::post_wanderer,
        handlers::latticefall::put_save,
        handlers::latticefall::get_save,
        handlers::latticefall::delete_save,
        handlers::latticefall::get_marks,
        handlers::latticefall::put_marks,
        handlers::latticefall::speedrun_leaderboard,
        handlers::latticefall::post_speedrun,
        handlers::latticefall::post_telemetry,
    ),
    components(
        schemas(
            handlers::resume::ResumeMetadataResponse,
            handlers::resume::ResumeUrlResponse,
            handlers::resume_api::LatestResumeResponse,
            handlers::resume_api::ResumeHistoryItem,
            handlers::resume_api::UploadResponse,
            handlers::home_weather::HomeWeatherResponse,
            handlers::edge_signal::EdgeSignalResponse,
            handlers::home_weather::LocalityWeatherData,
        )
    ),
    tags(
        (name = "Resume", description = "Resume endpoints"),
        (name = "Resume API", description = "Resume API endpoints"),
        (name = "Weather", description = "Local weather endpoints"),
        (name = "Edge", description = "Cloudflare edge telemetry endpoints"),
        (name = "Cinderheart", description = "Cinderheart online features, cloud saves, and telemetry"),
    ),
    modifiers(&SecurityAddon),
    info(
        title = "ifkash API",
        version = "1.0.0",
        description = "Personal website APIs powered by Cloudflare Workers",
        contact(
            name = "Kashif",
            email = "me@ifkash.dev"
        )
    ),
    servers(
        (url = "https://ifkash.dev", description = "Production server"),
        (url = "http://localhost:8787", description = "Local development server")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we have components
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

pub async fn route(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let openapi_json = ApiDoc::openapi()
        .to_pretty_json()
        .map_err(|e| Error::RustError(e.to_string()))?;

    let mut resp = Response::ok(openapi_json)?;
    resp.headers_mut().set("Content-Type", "application/json")?;
    resp.headers_mut().set("Cache-Control", "no-cache")?;
    Ok(resp)
}
