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

// KV Cache helpers
const KV_BINDING: &str = "IFKASH_HN";
const ITEM_TTL_SECS: u64 = 4 * 60 * 60;   // 4 hours

async fn kv_get_story(kv: &KvStore, id: i32) -> Result<Option<Story>> {
    let key = format!("item:{}", id);
    match kv.get(&key).text().await? {
        Some(s) => {
            match serde_json::from_str::<Story>(&s) {
                Ok(story) => Ok(Some(story)),
                Err(e) => {
                    console_warn!("KV parse failed for {}: {}", key, e);
                    Ok(None)
                }
            }
        }
        None => Ok(None),
    }
}

// A wrapper that first tries KV, otherwise fetches and stores.
async fn get_story_with_cache(kv: &KvStore, id: i32) -> Result<Story> {
    if let Some(story) = kv_get_story(kv, id).await? {
        return Ok(story);
    }

    let story = fetch_story(id).await?;
    // fire-and-forget write (don’t block the hot path if you don’t want to)
    let kv = kv.clone();
    let story_clone = story.clone();
    // spawn_local is not available; do the write inline and await:
    let _ = kv
        .put(&format!("item:{}", story_clone.id), serde_json::to_string(&story_clone).unwrap_or_default())
        .and_then(|p| Ok(p.expiration_ttl(ITEM_TTL_SECS)))
        .and_then(|p| Ok(p))
        .and_then(|p| futures::executor::block_on(p.execute()));
    Ok(story)
}

pub async fn handle(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Grab KV
    let kv = match ctx.kv(KV_BINDING) {
        Ok(ns) => ns,
        Err(e) => {
            console_error!("KV binding missing or invalid: {:?}", e);
            return Response::error("KV unavailable", 500);
        }
    };

    // Fetch top story IDs
    let story_ids = match fetch_top_stories().await {
        Ok(ids) => ids,
        Err(e) => {
            console_error!("Error fetching top stories: {:?}", e);
            return Response::error("Error fetching top stories", 500);
        }
    };

    // One-week cutoff
    let one_week_ago = (Date::now().as_millis() / 1000) as i64 - (7 * 24 * 60 * 60);

    // Concurrency cap so we don't hammer either HN or KV runtime
    const CONCURRENCY: usize = 24;

    let fetches = story_ids.into_iter().map(|id| {
        let kv = kv.clone();
        async move {
            let res = get_story_with_cache(&kv, id).await;
            (id, res)
        }
    });

    let mut s = stream::iter(fetches).buffer_unordered(CONCURRENCY);

    let mut stories: Vec<Story> = Vec::with_capacity(20);
    while let Some((_id, res)) = s.next().await {
        match res {
            Ok(story) => {
                if story.time >= one_week_ago {
                    stories.push(story);
                }
            }
            Err(e) => {
                // KV miss + network fail will land here
                console_warn!("Story fetch failed: {:?}", e);
            }
        }
        if stories.len() >= 20 {
            break; // early exit; remaining in-flight futures will be dropped
        }
    }

    // Sort by score desc and cap to 20
    stories.sort_by(|a, b| b.score.unwrap_or(0).cmp(&a.score.unwrap_or(0)));
    if stories.len() > 20 {
        stories.truncate(20);
    }

    Response::from_json(&stories)
}
