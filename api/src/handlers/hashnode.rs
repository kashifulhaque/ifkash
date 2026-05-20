use worker::*;
use worker::kv::KvStore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

const KV_BINDING: &str = "IFKASH_HN";
const CACHE_TTL_SECS: u64 = 300; // 5 min — blog list updates rarely
const HASHNODE_GQL_URL: &str = "https://gql.hashnode.com/";

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct HashnodeProxyRequest {
    /// GraphQL query string
    pub query: String,
    /// Optional GraphQL variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Value>,
    /// Optional GraphQL operation name
    #[serde(rename = "operationName", skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
}

// FNV-1a 64-bit — cheap, no deps; collisions don't matter for cache keys.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

async fn kv_get_string(kv: &KvStore, key: &str) -> Option<String> {
    match kv.get(key).text().await {
        Ok(v) => v,
        Err(_) => None,
    }
}

async fn kv_put_string(kv: &KvStore, key: &str, value: String, ttl: u64) {
    if let Ok(builder) = kv.put(key, value) {
        let _ = builder.expiration_ttl(ttl).execute().await;
    }
}

async fn fetch_hashnode_live(body: &str) -> Result<String> {
    let response = reqwest::Client::new()
        .post(HASHNODE_GQL_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| Error::RustError(format!("Failed to call Hashnode: {}", e)))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| Error::RustError(format!("Failed to read Hashnode response: {}", e)))?;

    if !status.is_success() {
        return Err(Error::RustError(format!(
            "Hashnode upstream returned {}: {}",
            status, text
        )));
    }

    Ok(text)
}

#[utoipa::path(
    post,
    path = "/api/hashnode",
    tag = "Hashnode",
    request_body = HashnodeProxyRequest,
    responses(
        (status = 200, description = "Forwarded GraphQL response from Hashnode"),
        (status = 400, description = "Invalid GraphQL request body"),
        (status = 502, description = "Hashnode upstream error"),
        (status = 500, description = "Server error")
    )
)]
pub async fn proxy(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let parsed: HashnodeProxyRequest = match req.json().await {
        Ok(p) => p,
        Err(e) => {
            console_error!("Invalid Hashnode proxy body: {:?}", e);
            return Response::error("Invalid GraphQL request body", 400);
        }
    };

    let normalized = serde_json::to_string(&parsed)
        .map_err(|e| Error::RustError(format!("Failed to serialize body: {}", e)))?;

    let kv = ctx.kv(KV_BINDING).map_err(|e| {
        console_error!("KV binding missing: {:?}", e);
        Error::RustError("KV unavailable".into())
    })?;

    let cache_key = format!("hashnode:{:016x}", fnv1a_64(normalized.as_bytes()));

    if let Some(cached) = kv_get_string(&kv, &cache_key).await {
        let mut resp = Response::ok(cached)?;
        resp.headers_mut().set("Content-Type", "application/json")?;
        resp.headers_mut().set("X-Cache", "HIT")?;
        return Ok(resp);
    }

    let payload = match fetch_hashnode_live(&normalized).await {
        Ok(p) => p,
        Err(e) => {
            console_error!("Hashnode fetch failed: {:?}", e);
            return Response::error("Failed to fetch from Hashnode", 502);
        }
    };

    kv_put_string(&kv, &cache_key, payload.clone(), CACHE_TTL_SECS).await;

    let mut resp = Response::ok(payload)?;
    resp.headers_mut().set("Content-Type", "application/json")?;
    resp.headers_mut().set("X-Cache", "MISS")?;
    Ok(resp)
}
