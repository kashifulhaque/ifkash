use worker::*;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    OpenApi,
};
use crate::handlers;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::hello::handle,
        handlers::ping::handle,
        handlers::hn::handle,
        handlers::lc::handle_profile,
        handlers::lc::handle_submissions,
        handlers::tensara::handle_profile,
        handlers::auth::login,
        handlers::auth::verify,
        handlers::resume::handle,
        handlers::resume_api::get_latest,
        handlers::resume_api::get_history,
        handlers::resume_api::get_by_id,
        handlers::resume_api::upload,
        handlers::typst::compile,
        handlers::ai::rewrite,
    ),
    components(
        schemas(
            handlers::hn::Story,
            handlers::lc::SubmissionsReqBody,
            handlers::lc::Slot,
            handlers::lc::LeetCodeProfile,
            handlers::tensara::TensaraStats,
            handlers::tensara::RecentSubmission,
            handlers::tensara::LanguagePercentage,
            handlers::tensara::ActivityData,
            handlers::tensara::CommunityStats,
            handlers::tensara::TensaraUser,
            handlers::auth::LoginRequest,
            handlers::auth::LoginResponse,
            handlers::auth::PocketBaseAdminRecord,
            handlers::auth::VerifyResponse,
            handlers::resume::ResumeMetadataResponse,
            handlers::resume::ResumeUrlResponse,
            handlers::resume::ResumeRecord,
            handlers::resume_api::LatestResumeResponse,
            handlers::resume_api::UploadResponse,
            handlers::typst::CompileRequest,
            handlers::ai::AiRewriteRequest,
            handlers::ai::AiRewriteResponse,
        )
    ),
    tags(
        (name = "Utils", description = "Utility endpoints"),
        (name = "Hacker News", description = "Hacker News endpoints"),
        (name = "LeetCode", description = "LeetCode endpoints"),
        (name = "Tensara", description = "Tensara GPU computing endpoints"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Resume", description = "Resume endpoints"),
        (name = "Resume API", description = "Resume API endpoints"),
        (name = "Typst", description = "Typst compilation service"),
        (name = "AI", description = "AI Integration endpoints"),
    ),
    modifiers(&SecurityAddon),
    info(
        title = "ifkash API",
        version = "1.0.0",
        description = "Personal website APIs powered by Cloudflare Workers",
        contact(
            name = "Kashif",
            email = "haque.kashiful7@gmail.com"
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
