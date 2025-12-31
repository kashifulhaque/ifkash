use worker::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use super::resume_api::verify_auth_header;

#[derive(Deserialize, ToSchema)]
pub struct AiRewriteRequest {
    pub code: String,
    pub job_description: String,
    pub model: String,
}

#[derive(Serialize, ToSchema)]
pub struct AiRewriteResponse {
    pub success: bool,
    pub code: String,
    pub message: Option<String>,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
    error: Option<OpenRouterError>,
}

#[derive(Deserialize)]
struct OpenRouterError {
    message: String,
}

#[derive(Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
}

#[derive(Deserialize)]
struct OpenRouterMessage {
    content: String,
}

// POST /api/ai/rewrite - Rewrite Typst code based on JD
#[utoipa::path(
    post,
    path = "/api/ai/rewrite",
    tag = "AI",
    security(
        ("bearer_auth" = [])
    ),
    request_body = AiRewriteRequest,
    responses(
        (status = 200, description = "Rewritten code", body = AiRewriteResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    )
)]
pub async fn rewrite(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Verify authentication
    let headers = req.headers();
    verify_auth_header(&headers)?;

    let body: AiRewriteRequest = req.json().await?;
    
    // Get OpenRouter API Key
    let api_key = match ctx.secret("OPENROUTER_API_KEY") {
        Ok(s) => s.to_string(),
        Err(_) => return Response::error("Missing OPENROUTER_API_KEY", 500),
    };

    let client = reqwest::Client::new();
    
    let prompt = format!(
        "You are an expert Resume Optimizer utilizing Typst. 
        Your task is to tailor the user's resume for a specific Job Description (JD).
        
        RULES:
        1. Keep the exact same Typst structure, imports, and function calls.
        2. ONLY modify the content strings (bullet points, summary, skills) to match the keywords in the JD.
        3. Do NOT remove any sections unless they are completely irrelevant.
        4. Do NOT add markdown formatting (like ```typst). Return ONLY the raw Typst code.
        5. Maintain a professional tone.

         JOB DESCRIPTION:
        {}

        CURRENT TYPST CODE:
        {}
        ",
        body.job_description,
        body.code
    );

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("HTTP-Referer", "https://ifkash.dev") // OpenRouter requirement
        .header("X-Title", "Resume Editor")
        .json(&json!({
            "model": body.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
        .send()
        .await
        .map_err(|e| Error::RustError(format!("Failed to call OpenRouter: {}", e)))?;

    if !response.status().is_success() {
         let text = response.text().await.unwrap_or_default();
         return Response::error(format!("OpenRouter API failed: {}", text), 500);
    }

    let or_response: OpenRouterResponse = response
        .json()
        .await
        .map_err(|e| Error::RustError(format!("Failed to parse OpenRouter response: {}", e)))?;

    if let Some(err) = or_response.error {
        return Response::error(format!("OpenRouter error: {}", err.message), 500);
    }

    let new_code = or_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| Error::RustError("No content returned from AI".into()))?;

    // Cleanup: Remove markdown code fences if the AI included them despite instructions
    let clean_code = new_code
        .replace("```typst", "")
        .replace("```", "")
        .trim()
        .to_string();

    Response::from_json(&AiRewriteResponse {
        success: true,
        code: clean_code,
        message: None
    })
}
