use worker::*;
use serde_json::json;

pub async fn latest(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Fast path: return cached blob from KV (IFKASH_HN)
    let kv = ctx.env.kv("IFKASH_HN")?;
    match kv.get("rentals:latest").text().await {
        Ok(Some(body)) => {
            // parse stored string into JSON and return it
            let v: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| Error::RustError(format!("invalid cached json: {}", e)))?;
            Ok(Response::from_json(&v)?)
        }
        _ => {
            // No cache yet
            Ok(Response::from_json(&json!({ "posts": [], "cached": false }))?)
        }
    }
}
