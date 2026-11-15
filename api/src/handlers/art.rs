//! Serves Cinderheart's private game art from the `ART_BUCKET` R2 bucket.
//!
//! Some game art, such as Mixamo characters, may ship inside the game but may not sit in
//! the public repository. The asset pipeline bakes it locally into
//! `static/latticefall/private/` (gitignored), and the owner uploads it to R2 under
//! `private/`. In production the game loads it from here; in local development it reads
//! the baked files from the dev server instead. See
//! `docs/game/tutorial-production-plan.md`, "Private art".

use worker::*;

/// Browsers and the edge may reuse a file for an hour. A rebake keeps the same name, so
/// keep this short enough that a new bake reaches players the same day.
const CACHE_CONTROL: &str = "public, max-age=3600";

/// The content type for an allowed file name, or `None` for a name this route doesn't serve.
fn content_type(file: &str) -> Option<&'static str> {
    let (stem, ext) = file.rsplit_once('.')?;
    let stem_ok = !stem.is_empty()
        && stem
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if !stem_ok {
        return None;
    }
    match ext {
        "glb" => Some("model/gltf-binary"),
        "json" => Some("application/json"),
        _ => None,
    }
}

/// `GET /api/art/private/:file`: one baked file, such as `paladin.glb` or `paladin.json`.
pub async fn private_file(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let file = ctx.param("file").map(String::as_str).unwrap_or_default();
    let Some(mime) = content_type(file) else {
        return Response::error("Not found", 404);
    };
    let Some(object) = ctx
        .bucket("ART_BUCKET")?
        .get(format!("private/{file}"))
        .execute()
        .await?
    else {
        return Response::error("Not found", 404);
    };
    let etag = object.http_etag();
    let bytes = object
        .body()
        .ok_or_else(|| Error::RustError("Empty R2 object body".to_string()))?
        .bytes()
        .await?;
    let headers = Headers::new();
    headers.set("Content-Type", mime)?;
    headers.set("Cache-Control", CACHE_CONTROL)?;
    headers.set("ETag", &etag)?;
    Ok(Response::from_bytes(bytes)?.with_headers(headers))
}

#[cfg(test)]
mod tests {
    use super::content_type;

    #[test]
    fn serves_only_plain_glb_and_json_names() {
        assert_eq!(content_type("paladin.glb"), Some("model/gltf-binary"));
        assert_eq!(content_type("paladin_2.json"), Some("application/json"));
        assert_eq!(content_type("../paladin.glb"), None);
        assert_eq!(content_type("Paladin.glb"), None);
        assert_eq!(content_type("paladin.fbx"), None);
        assert_eq!(content_type(".glb"), None);
        assert_eq!(content_type("paladin"), None);
    }
}
