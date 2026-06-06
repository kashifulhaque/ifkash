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
        handlers::ai::rewrite,
        handlers::home_weather::handle,
    ),
    components(
        schemas(
            handlers::resume::ResumeMetadataResponse,
            handlers::resume::ResumeUrlResponse,
            handlers::resume_api::LatestResumeResponse,
            handlers::resume_api::ResumeHistoryItem,
            handlers::resume_api::UploadResponse,
            handlers::ai::AiRewriteRequest,
            handlers::ai::AiRewriteResponse,
            handlers::home_weather::HomeWeatherResponse,
            handlers::home_weather::LocalityWeatherData,
        )
    ),
    tags(
        (name = "Resume", description = "Resume endpoints"),
        (name = "Resume API", description = "Resume API endpoints"),
        (name = "AI", description = "AI Integration endpoints"),
        (name = "Weather", description = "Local weather endpoints"),
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
