use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use js_sys::{Array, Function, Object, Promise, Reflect, Uint8Array};
use serde::Deserialize;
use serde_json::Value;
use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker::*;

/// How long to reuse the team's signing keys before fetching them again.
const CERTS_TTL_SECS: i64 = 60 * 60;
/// Minimum gap between fetches triggered by an unknown `kid`, so forged
/// tokens can't make every request refetch the keys.
const CERTS_MIN_REFETCH_SECS: i64 = 5 * 60;
/// Tolerated clock difference when checking `exp` and `nbf`.
const CLOCK_SKEW_SECS: i64 = 30;

/// Returns true if the request is allowed to reach a protected endpoint.
pub async fn is_authorized(req: &Request, ctx: &RouteContext<()>) -> bool {
    authorized_email(req, ctx).await.is_some()
}

/// Returns the owner's email if the request carries a valid Cloudflare Access
/// token for them, or `None` otherwise.
///
/// Cloudflare Access signs every request it lets through with a JWT in the
/// `Cf-Access-Jwt-Assertion` header. The Worker verifies that token itself
/// instead of trusting the plain `Cf-Access-Authenticated-User-Email` header,
/// so a path the Access application doesn't cover still fails closed. The
/// token must be signed by the `ACCESS_TEAM_DOMAIN` team, issued for one of
/// the `ACCESS_AUD` applications, unexpired, and for `OWNER_EMAIL`. If any of
/// those vars is unset, every request is rejected.
///
/// Under `wrangler dev` there is no Access in front of the Worker, so local
/// dev is detected via a `LOCAL_DEV` flag set in `api/.dev.vars` (gitignored,
/// never present in production).
pub async fn authorized_email(req: &Request, ctx: &RouteContext<()>) -> Option<String> {
    if is_local_dev(ctx) {
        return Some("local-dev@localhost".to_string());
    }

    let owner = config(ctx, "OWNER_EMAIL")?;
    let team = config(ctx, "ACCESS_TEAM_DOMAIN")?;
    let auds = config(ctx, "ACCESS_AUD")?;
    let token = req.headers().get("Cf-Access-Jwt-Assertion").ok()??;

    let issuer = format!(
        "https://{}",
        team.trim_start_matches("https://").trim_end_matches('/')
    );
    let claims = verify_access_jwt(&token, &issuer, &auds).await?;
    let email = claims.get("email")?.as_str()?;
    email
        .eq_ignore_ascii_case(&owner)
        .then(|| email.to_string())
}

/// Dev-only bypass driven by the `LOCAL_DEV` flag in `api/.dev.vars`. It is
/// never set in production, so Cloudflare Access stays enforced when deployed.
/// Shared by the Cloudflare-Access (editor) check and the per-user Google ID
/// token check used by the fitness handlers.
pub fn is_local_dev(ctx: &RouteContext<()>) -> bool {
    let truthy = |v: String| matches!(v.as_str(), "1" | "true" | "TRUE");
    if let Ok(v) = ctx.secret("LOCAL_DEV") {
        if truthy(v.to_string()) {
            return true;
        }
    }
    if let Ok(v) = ctx.var("LOCAL_DEV") {
        if truthy(v.to_string()) {
            return true;
        }
    }
    false
}

/// Standard 401 response for unauthenticated requests to protected endpoints.
pub fn unauthorized() -> Result<Response> {
    Response::error("Unauthorized", 401)
}

/// A non-empty `[vars]` value, or `None` when it is missing or blank.
fn config(ctx: &RouteContext<()>, name: &str) -> Option<String> {
    ctx.var(name)
        .ok()
        .map(|v| v.to_string())
        .filter(|v| !v.trim().is_empty())
}

fn now_secs() -> i64 {
    (worker::Date::now().as_millis() / 1000) as i64
}

/// Verifies an Access JWT's RS256 signature and claims, returning the claims.
/// `auds` is a comma-separated list of accepted application AUD tags.
async fn verify_access_jwt(token: &str, issuer: &str, auds: &str) -> Option<Value> {
    let mut parts = token.split('.');
    let (header, payload, signature) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }

    let jose: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(header).ok()?).ok()?;
    if jose.get("alg")?.as_str()? != "RS256" {
        return None;
    }
    let kid = jose.get("kid")?.as_str()?;
    let jwk = signing_key(issuer, kid).await?;
    let signature = URL_SAFE_NO_PAD.decode(signature).ok()?;
    let signed = format!("{header}.{payload}");
    if !rs256_verify(&jwk, &signature, signed.as_bytes())
        .await
        .unwrap_or(false)
    {
        return None;
    }

    let claims: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).ok()?).ok()?;
    let now = now_secs();
    if claims.get("iss")?.as_str()? != issuer {
        return None;
    }
    if claims.get("exp")?.as_i64()? + CLOCK_SKEW_SECS <= now {
        return None;
    }
    if let Some(nbf) = claims.get("nbf").and_then(Value::as_i64) {
        if nbf - CLOCK_SKEW_SECS > now {
            return None;
        }
    }

    let allowed: Vec<&str> = auds
        .split(',')
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .collect();
    let aud_ok = match claims.get("aud")? {
        Value::String(aud) => allowed.contains(&aud.as_str()),
        Value::Array(list) => list
            .iter()
            .filter_map(Value::as_str)
            .any(|aud| allowed.contains(&aud)),
        _ => false,
    };
    aud_ok.then_some(claims)
}

#[derive(Clone)]
struct CertCache {
    issuer: String,
    fetched_at: i64,
    keys: Vec<Value>,
}

#[derive(Deserialize)]
struct CertsResponse {
    keys: Vec<Value>,
}

thread_local! {
    // Per-isolate cache of the team's public signing keys (JWKs).
    static CERTS: RefCell<Option<CertCache>> = RefCell::new(None);
}

fn find_key(keys: &[Value], kid: &str) -> Option<Value> {
    keys.iter()
        .find(|k| k.get("kid").and_then(Value::as_str) == Some(kid))
        .cloned()
}

/// Looks up the JWK for `kid` in the team's key set, fetching the set from
/// `<issuer>/cdn-cgi/access/certs` when the cache is stale or lacks the key.
async fn signing_key(issuer: &str, kid: &str) -> Option<Value> {
    let now = now_secs();
    let cached = CERTS
        .with(|c| c.borrow().clone())
        .filter(|c| c.issuer == issuer);
    if let Some(cache) = &cached {
        let key = find_key(&cache.keys, kid);
        let age = now - cache.fetched_at;
        if key.is_some() && age < CERTS_TTL_SECS {
            return key;
        }
        if key.is_none() && age < CERTS_MIN_REFETCH_SECS {
            return None;
        }
    }

    let url = format!("{issuer}/cdn-cgi/access/certs");
    let keys = match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<CertsResponse>().await.ok().map(|c| c.keys)
        }
        _ => None,
    };
    match keys {
        Some(keys) => {
            let key = find_key(&keys, kid);
            CERTS.with(|c| {
                *c.borrow_mut() = Some(CertCache {
                    issuer: issuer.to_string(),
                    fetched_at: now,
                    keys,
                })
            });
            key
        }
        // Keep using the stale keys if the refresh fails.
        None => cached.and_then(|c| find_key(&c.keys, kid)),
    }
}

/// Checks an RSASSA-PKCS1-v1_5 / SHA-256 signature with the Workers runtime's
/// Web Crypto API, which avoids shipping an RSA implementation in the WASM.
async fn rs256_verify(
    jwk: &Value,
    signature: &[u8],
    data: &[u8],
) -> std::result::Result<bool, JsValue> {
    let crypto = Reflect::get(&js_sys::global(), &"crypto".into())?;
    let subtle = Reflect::get(&crypto, &"subtle".into())?;

    let algorithm = Object::new();
    Reflect::set(&algorithm, &"name".into(), &"RSASSA-PKCS1-v1_5".into())?;
    Reflect::set(&algorithm, &"hash".into(), &"SHA-256".into())?;
    let jwk = js_sys::JSON::parse(&jwk.to_string())?;
    let usages = Array::of1(&"verify".into());

    let key = call(
        &subtle,
        "importKey",
        &[&"jwk".into(), &jwk, &algorithm, &JsValue::FALSE, &usages],
    )
    .await?;
    let valid = call(
        &subtle,
        "verify",
        &[
            &algorithm,
            &key,
            &Uint8Array::from(signature),
            &Uint8Array::from(data),
        ],
    )
    .await?;
    Ok(valid.as_bool() == Some(true))
}

/// Calls a promise-returning method on a JS object and awaits the result.
async fn call(
    target: &JsValue,
    method: &str,
    args: &[&JsValue],
) -> std::result::Result<JsValue, JsValue> {
    let func: Function = Reflect::get(target, &method.into())?.dyn_into()?;
    let args: Array = args.iter().collect();
    let promise: Promise = func.apply(target, &args)?.dyn_into()?;
    JsFuture::from(promise).await
}
