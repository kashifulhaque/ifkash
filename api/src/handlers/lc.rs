use worker::*;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const LC_URL: &str = "https://leetcode.com/graphql";

#[derive(Deserialize, ToSchema)]
pub struct SubmissionsReqBody {
    pub username: String,
    pub count: Option<i32>,
}

#[derive(Serialize, Default, ToSchema)]
pub struct Slot {
    pub complete: i32,
    pub total: i32,
}

#[derive(Serialize, Default, ToSchema)]
pub struct LeetCodeProfile {
    pub easy: Slot,
    pub medium: Slot,
    pub hard: Slot,
    pub total: Slot,
}

/* ---- /api/lc/profile ---- */
#[utoipa::path(
    get,
    path = "/api/lc/profile",
    tag = "LeetCode",
    params(
        ("username" = Option<String>, Query, description = "LeetCode username (default: ifkash)")
    ),
    responses(
        (status = 200, description = "LeetCode profile statistics", body = LeetCodeProfile)
    )
)]
pub async fn handle_profile(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    // Take username from query, fallback to 'ifkash'
    let url = req.url()?;
    let username = url
        .query_pairs()
        .find(|(k, _)| k == "username")
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| "ifkash".to_string());

    // GraphQL body (as JSON string)
    let gql = json!({
        "query":
            "query userProblemsSolved($username: String!) { \
                allQuestionsCount { difficulty count } \
                matchedUser (username: $username) { \
                    submitStatsGlobal { acSubmissionNum { difficulty count } } \
                } \
                }",
        "variables": { "username": username },
        "operationName": "userProblemsSolved"
    })
    .to_string();

    // Build outbound POST
    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_body(Some(gql.into()));
    let mut outbound = Request::new_with_init(LC_URL, &init)?;
    outbound
        .headers_mut()?
        .set("Content-Type", "application/json")?;

    // Send + read body text
    let mut lc_res = Fetch::Request(outbound).send().await?;
    let body_text = lc_res.text().await?;

    // Parse what we need
    #[derive(Deserialize)]
    struct QuestionCount {
        difficulty: String,
        count: i32,
    }
    #[derive(Deserialize)]
    struct SubmissionNum {
        difficulty: String,
        count: i32,
    }
    #[derive(Deserialize)]
    struct SubmitStatsGlobal {
        #[serde(rename = "acSubmissionNum")]
        ac_submission_num: Vec<SubmissionNum>,
    }
    #[derive(Deserialize)]
    struct MatchedUser {
        #[serde(rename = "submitStatsGlobal")]
        submit_stats_global: SubmitStatsGlobal,
    }
    #[derive(Deserialize)]
    struct Data {
        #[serde(rename = "allQuestionsCount")]
        all_questions_count: Vec<QuestionCount>,
        #[serde(rename = "matchedUser")]
        matched_user: MatchedUser,
    }
    #[derive(Deserialize)]
    struct GqlResp {
        data: Data,
    }

    let parsed: GqlResp = serde_json::from_str(&body_text)
        .map_err(|_| Error::RustError("Error parsing JSON from LeetCode".into()))?;

    let mut out = LeetCodeProfile::default();

    for q in parsed.data.all_questions_count {
        match q.difficulty.as_str() {
            "All" => out.total.total = q.count,
            "Easy" => out.easy.total = q.count,
            "Medium" => out.medium.total = q.count,
            "Hard" => out.hard.total = q.count,
            _ => {}
        }
    }

    for s in parsed
        .data
        .matched_user
        .submit_stats_global
        .ac_submission_num
    {
        match s.difficulty.as_str() {
            "All" => out.total.complete = s.count,
            "Easy" => out.easy.complete = s.count,
            "Medium" => out.medium.complete = s.count,
            "Hard" => out.hard.complete = s.count,
            _ => {}
        }
    }

    Response::from_json(&out)
}

/* ---- /api/lc/submissions ---- */
#[utoipa::path(
    post,
    path = "/api/lc/submissions",
    tag = "LeetCode",
    request_body = SubmissionsReqBody,
    responses(
        (status = 200, description = "Recent submissions", body = Value),
        (status = 400, description = "Invalid request"),
        (status = 405, description = "Method not allowed")
    )
)]
pub async fn handle_submissions(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    if req.method() != Method::Post {
        return Response::error("Only POST method is allowed", 405);
    }

    let SubmissionsReqBody { username, count } = match req.json().await {
        Ok(v) => v,
        Err(_) => return Response::error("Invalid request body", 400),
    };

    if username.trim().is_empty() {
        return Response::error("Invalid username", 400);
    }

    let mut limit = count.unwrap_or(0);
    if limit < 0 {
        limit = 0;
    }
    if limit > 20 {
        limit = 20;
    }

    let gql = json!({
        "query":
            "query recentAcSubmissions($username: String!, $limit: Int!) { \
               recentAcSubmissionList(username: $username, limit: $limit) { \
                 id title titleSlug timestamp \
               } \
             }",
        "variables": { "username": username, "limit": limit },
        "operationName": "recentAcSubmissions"
    })
    .to_string();

    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_body(Some(gql.into()));
    let mut outbound = Request::new_with_init(LC_URL, &init)?;
    outbound
        .headers_mut()?
        .set("Content-Type", "application/json")?;

    let mut lc_res = Fetch::Request(outbound).send().await?;
    let text = lc_res.text().await?;

    // passthrough JSON from LeetCode
    let val: Value = serde_json::from_str(&text)
        .unwrap_or_else(|_| json!({"error": "Failed to parse LeetCode response"}));
    Response::from_json(&val)
}
