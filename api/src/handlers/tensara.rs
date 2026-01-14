use worker::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const TENSARA_API_BASE: &str = "https://tensara.org/api/trpc/users.getByUsername";

#[derive(Deserialize, Serialize, ToSchema)]
pub struct TensaraStats {
    pub submissions: i32,
    #[serde(rename = "solvedProblems")]
    pub solved_problems: i32,
    pub ranking: i32,
    pub rating: i32,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct RecentSubmission {
    pub id: String,
    #[serde(rename = "problemId")]
    pub problem_id: String,
    #[serde(rename = "problemName")]
    pub problem_name: String,
    pub date: String,
    pub status: String,
    pub runtime: String,
    pub gflops: String,
    #[serde(rename = "gpuType")]
    pub gpu_type: String,
    pub language: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct LanguagePercentage {
    pub language: String,
    pub percentage: i32,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ActivityData {
    pub date: String,
    pub count: i32,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CommunityStats {
    #[serde(rename = "totalPosts")]
    pub total_posts: i32,
    #[serde(rename = "totalLikes")]
    pub total_likes: i32,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct TensaraUser {
    pub id: String,
    pub username: String,
    pub name: String,
    pub image: String,
    #[serde(rename = "joinedAt")]
    pub joined_at: String,
    pub stats: TensaraStats,
    #[serde(rename = "recentSubmissions")]
    pub recent_submissions: Vec<RecentSubmission>,
    #[serde(rename = "communityStats")]
    pub community_stats: CommunityStats,
    #[serde(rename = "activityData")]
    pub activity_data: Vec<ActivityData>,
    #[serde(rename = "languagePercentage")]
    pub language_percentage: Vec<LanguagePercentage>,
}

#[derive(Deserialize)]
struct TensaraApiResponse {
    result: TensaraApiData,
}

#[derive(Deserialize)]
struct TensaraApiData {
    data: TensaraDataJson,
}

#[derive(Deserialize)]
struct TensaraDataJson {
    json: TensaraUser,
}

/* ---- /api/tensara/profile ---- */
#[utoipa::path(
    get,
    path = "/api/tensara/profile",
    tag = "Tensara",
    params(
        ("username" = Option<String>, Query, description = "Tensara username (default: kashifulhaque)")
    ),
    responses(
        (status = 200, description = "Tensara user profile and statistics", body = TensaraUser)
    )
)]
pub async fn handle_profile(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    // Take username from query, fallback to 'kashifulhaque'
    let url = req.url()?;
    let username = url
        .query_pairs()
        .find(|(k, _)| k == "username")
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| "kashifulhaque".to_string());

    // Build the API URL with the username
    // The input parameter needs to be URL encoded JSON: {"json":{"username":"kashifulhaque"}}
    let input = format!(r#"{{"json":{{"username":"{}"}}}}"#, username);
    let encoded_input = urlencoding::encode(&input);
    let api_url = format!("{}?input={}", TENSARA_API_BASE, encoded_input);

    // Build outbound GET request
    let mut init = RequestInit::new();
    init.with_method(Method::Get);
    let outbound = Request::new_with_init(&api_url, &init)?;

    // Send request
    let mut tensara_res = Fetch::Request(outbound).send().await?;
    let body_text = tensara_res.text().await?;

    // Parse the response
    let parsed: TensaraApiResponse = serde_json::from_str(&body_text)
        .map_err(|_| Error::RustError("Error parsing JSON from Tensara API".into()))?;

    // Return the user data
    Response::from_json(&parsed.result.data.json)
}
