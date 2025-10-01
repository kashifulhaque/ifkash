use worker::*;
use serde::{Deserialize, Serialize};
use futures::{stream, StreamExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Story {
    pub by: Option<String>,
    pub id: i32,
    pub score: Option<i32>,
    pub time: i64,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub story_type: Option<String>,
    pub url: Option<String>,
}

async fn fetch_top_stories() -> Result<Vec<i32>> {
    let mut response = Fetch::Url(
        "https://hacker-news.firebaseio.com/v0/topstories.json"
            .parse()
            .map_err(|e| Error::RustError(format!("Invalid URL: {}", e)))?,
    )
    .send()
    .await?;

    let story_ids: Vec<i32> = response.json().await?;
    Ok(story_ids)
}

async fn fetch_story(id: i32) -> Result<Story> {
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);

    let mut response = Fetch::Url(
        url.parse()
            .map_err(|e| Error::RustError(format!("Invalid URL: {}", e)))?,
    )
    .send()
    .await?;

    let story: Story = response.json().await?;
    Ok(story)
}

pub async fn handle(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    // Fetch top story IDs
    let story_ids = match fetch_top_stories().await {
        Ok(ids) => ids,
        Err(e) => {
            console_error!("Error fetching top stories: {:?}", e);
            return Response::error("Error fetching top stories", 500);
        }
    };

    // Calculate timestamp for one week ago
    let one_week_ago = (Date::now().as_millis() / 1000) as i64 - (7 * 24 * 60 * 60);

    // Build a stream of fetch futures with a concurrency limit
    const CONCURRENCY: usize = 24;

    let fetches = story_ids.into_iter().map(|id| async move {
        // Return (id, Result<Story>) so we can log which one failed
        let res = fetch_story(id).await;
        (id, res)
    });

    let mut stream = stream::iter(fetches).buffer_unordered(CONCURRENCY);

    let mut stories: Vec<Story> = Vec::with_capacity(20);
    while let Some((id, res)) = stream.next().await {
        match res {
            Ok(story) => {
                if story.time >= one_week_ago {
                    stories.push(story);
                }
            }
            Err(e) => {
                console_error!("Error fetching story {}: {:?}", id, e);
            }
        }

        if stories.len() >= 20 {
            break;
        }
    }

    stories.sort_by(|a, b| b.score.unwrap_or(0).cmp(&a.score.unwrap_or(0)));
    if stories.len() > 20 {
        stories.truncate(20);
    }

    Response::from_json(&stories)
}
