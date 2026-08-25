use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use worker::*;

#[derive(Debug, Serialize, ToSchema)]
pub struct EdgeSignalResponse {
    pub colo: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub transport: String,
    pub tls: String,
    pub signals: Option<i64>,
    pub counted: bool,
}

#[derive(Deserialize)]
struct SignalCount {
    signals: i64,
}

async fn record_signal(ctx: &RouteContext<()>, colo: &str) -> Result<i64> {
    let d1 = ctx.d1("IFKASH_D1")?;
    d1.prepare(
        "INSERT INTO edge_signals (colo, signals, updated) \
         VALUES (?1, 1, datetime('now')) \
         ON CONFLICT(colo) DO UPDATE SET \
           signals = edge_signals.signals + 1, updated = datetime('now')",
    )
    .bind(&[colo.into()])?
    .run()
    .await?;

    let total: Option<SignalCount> = d1
        .prepare("SELECT COALESCE(SUM(signals), 0) AS signals FROM edge_signals")
        .first(None)
        .await?;

    Ok(total.map(|row| row.signals).unwrap_or_default())
}

#[utoipa::path(
    get,
    path = "/api/edge-signal",
    tag = "Edge",
    responses(
        (status = 200, description = "Cloudflare edge metadata and aggregate signal count", body = EdgeSignalResponse)
    )
)]
pub async fn handle(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let (colo, city, country, transport, tls) = match req.cf() {
        Some(cf) => (
            cf.colo(),
            cf.city(),
            cf.country(),
            cf.http_protocol(),
            cf.tls_version(),
        ),
        None => (
            "DEV".to_string(),
            Some("Local development".to_string()),
            None,
            "HTTP".to_string(),
            "LOCAL".to_string(),
        ),
    };

    let (signals, counted) = match record_signal(&ctx, &colo).await {
        Ok(total) => (Some(total), true),
        Err(error) => {
            console_error!("Edge signal counter unavailable: {:?}", error);
            (None, false)
        }
    };

    let mut response = Response::from_json(&EdgeSignalResponse {
        colo,
        city,
        country,
        transport,
        tls,
        signals,
        counted,
    })?;
    response.headers_mut().set("Cache-Control", "no-store")?;
    Ok(response)
}
