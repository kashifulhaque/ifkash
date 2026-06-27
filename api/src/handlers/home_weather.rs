use worker::*;
use worker::kv::KvStore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

const KV_BINDING: &str = "IFKASH_HN";
const CACHE_TTL_SECS: u64 = 300; // 5 min
const DEFAULT_LOCALITY: &str = "ZWL008797";
const WEATHER_UNION_URL: &str =
    "https://www.weatherunion.com/gw/weather/external/v0/get_locality_weather_data";

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LocalityWeatherData {
    pub temperature: Option<f64>,
    pub humidity: Option<f64>,
    pub wind_speed: Option<f64>,
    pub wind_direction: Option<f64>,
    pub rain_intensity: Option<f64>,
    pub rain_accumulation: Option<f64>,
    pub aqi_pm_10: Option<f64>,
    pub aqi_pm_2_point_5: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct HomeWeatherResponse {
    pub status: String,
    pub message: String,
    pub device_type: Option<i64>,
    pub locality_weather_data: LocalityWeatherData,
}

async fn kv_get_json<T: for<'de> Deserialize<'de>>(kv: &KvStore, key: &str) -> Option<T> {
    match kv.get(key).text().await {
        Ok(Some(s)) => serde_json::from_str::<T>(&s).ok(),
        _ => None,
    }
}

async fn kv_put_json<T: Serialize>(kv: &KvStore, key: &str, value: &T, ttl: u64) {
    if let Ok(s) = serde_json::to_string(value) {
        if let Ok(builder) = kv.put(key, s) {
            let _ = builder.expiration_ttl(ttl).execute().await;
        }
    }
}

async fn fetch_weather_live(locality_id: &str, api_key: &str) -> Result<Value> {
    let url = format!("{}?locality_id={}", WEATHER_UNION_URL, locality_id);

    let mut init = RequestInit::new();
    init.with_method(Method::Get);

    let mut outbound = Request::new_with_init(&url, &init)?;
    outbound
        .headers_mut()?
        .set("X-Zomato-Api-Key", api_key)?;

    let mut response = Fetch::Request(outbound).send().await?;

    if response.status_code() >= 400 {
        return Err(Error::RustError(format!(
            "WeatherUnion upstream returned {}",
            response.status_code()
        )));
    }

    let json: Value = response.json().await?;
    Ok(json)
}

#[utoipa::path(
    get,
    path = "/api/home_weather",
    tag = "Weather",
    params(
        ("locality_id" = Option<String>, Query, description = "WeatherUnion locality id (defaults to ZWL006658)")
    ),
    responses(
        (status = 200, description = "Home locality weather snapshot", body = HomeWeatherResponse),
        (status = 500, description = "Upstream or configuration error")
    )
)]
pub async fn handle(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;
    let locality_id = url
        .query_pairs()
        .find(|(k, _)| k == "locality_id")
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| DEFAULT_LOCALITY.to_string());

    let kv = ctx.kv(KV_BINDING).map_err(|e| {
        console_error!("KV binding missing: {:?}", e);
        Error::RustError("KV unavailable".into())
    })?;

    let cache_key = format!("weather:{}", locality_id);

    if let Some(cached) = kv_get_json::<Value>(&kv, &cache_key).await {
        return Response::from_json(&cached);
    }

    let api_key = match ctx.secret("WEATHER_UNION_API_KEY") {
        Ok(s) => s.to_string(),
        Err(_) => {
            console_error!("WEATHER_UNION_API_KEY secret missing");
            return Response::error("Missing WEATHER_UNION_API_KEY", 500);
        }
    };

    let payload = match fetch_weather_live(&locality_id, &api_key).await {
        Ok(p) => p,
        Err(e) => {
            console_error!("WeatherUnion fetch failed: {:?}", e);
            return Response::error("Failed to fetch weather", 502);
        }
    };

    kv_put_json(&kv, &cache_key, &payload, CACHE_TTL_SECS).await;

    Response::from_json(&payload)
}
