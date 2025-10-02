use worker::*;
use crate::kv::KvStore;
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};

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

const KV_BINDING: &str = "IFKASH_HN";
const ITEM_TTL_SECS: u64 = 4 * 60 * 60; // 4h
const TOP_TTL_SECS: u64 = 120;          // 2m for topstories list
const CONCURRENCY: usize = 12;
const MAX_IDS_TO_SCAN: usize = 100;

async fn fetch_top_stories_live() -> Result<Vec<i32>> {
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

async fn fetch_story_live(id: i32) -> Result<Story> {
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

async fn kv_get_json<T: for<'de> Deserialize<'de>>(kv: &KvStore, key: &str) -> Result<Option<T>> {
    match kv.get(key).text().await? {
        Some(s) => Ok(serde_json::from_str::<T>(&s).ok()),
        None => Ok(None),
    }
}

async fn kv_put_json<T: Serialize>(kv: &KvStore, key: &str, value: &T, ttl: u64) {
    if let Ok(s) = serde_json::to_string(value) {
        // Best-effort write; ignore errors
        let _ = kv.put(key, s).expect("REASON")
            .expiration_ttl(ttl)
            .execute()
            .await;
    }
}

// Cached “topstories”
async fn get_topstories(kv: &KvStore) -> Result<Vec<i32>> {
    if let Some(ids) = kv_get_json::<Vec<i32>>(kv, "top:latest").await? {
        return Ok(ids);
    }
    let ids = fetch_top_stories_live().await?;
    kv_put_json(kv, "top:latest", &ids, TOP_TTL_SECS).await;
    Ok(ids)
}

// Cached story fetch
async fn get_story(kv: &KvStore, id: i32) -> Result<Story> {
    let key = format!("item:{id}");
    if let Some(story) = kv_get_json::<Story>(kv, &key).await? {
        return Ok(story);
    }
    let story = fetch_story_live(id).await?;
    kv_put_json(kv, &key, &story, ITEM_TTL_SECS).await;
    Ok(story)
}

pub async fn handle(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.kv(KV_BINDING).map_err(|e| {
        console_error!("KV binding missing: {:?}", e);
        Error::RustError("KV unavailable".into())
    })?;

    // Get top IDs (from KV cache w/ short TTL)
    let story_ids = match get_topstories(&kv).await {
        Ok(ids) => ids,
        Err(e) => {
            console_error!("Error fetching top stories: {:?}", e);
            return Response::error("Error fetching top stories", 500);
        }
    };

    let one_week_ago = (Date::now().as_millis() / 1000) as i64 - (7 * 24 * 60 * 60);

    // Only scan first N IDs to avoid excessive work on cold runs
    let ids = story_ids.into_iter().take(MAX_IDS_TO_SCAN);

    let fetches = ids.map(|id| {
        let kv = kv.clone();
        async move {
            // We return Option<Story> so we can filter early
            match get_story(&kv, id).await {
                Ok(story) if story.time >= one_week_ago => Some(story),
                _ => None,
            }
        }
    });

    let mut stream = stream::iter(fetches).buffer_unordered(CONCURRENCY);

    let mut stories: Vec<Story> = Vec::with_capacity(20);
    while let Some(opt) = stream.next().await {
        if let Some(story) = opt {
            stories.push(story);
            if stories.len() >= 20 {
                break;
            }
        }
    }

    // If we still didn’t reach 20 (rare), you can either return what you have
    // or do a second pass with the next chunk of IDs.

    stories.sort_by(|a, b| b.score.unwrap_or(0).cmp(&a.score.unwrap_or(0)));
    if stories.len() > 20 {
        stories.truncate(20);
    }

    Response::from_json(&stories)
}
