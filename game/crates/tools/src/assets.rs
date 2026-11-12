//! `lf-tools assets`: optimizes the source art in `game/assets-src/` into `static/latticefall/`.
//!
//! The job list is `game/assets-src/assets.json`. Each entry is either a glTF binary that
//! goes through `gltf-transform optimize` (meshopt-compressed geometry and animation, KTX2
//! textures when the file has any) or a plain file that is copied. A `manifest.json` listing
//! every output, grouped by its first path segment (the region), is written next to them.
//!
//! The tool runs `gltf-transform` from the repository's `node_modules` (a Bun devDependency)
//! and falls back to `bunx`. KTX2 conversion needs KTX-Software's `toktx` on the `PATH`
//! (docs/game/decisions/0005-asset-pipeline-tools.md); files with no textures don't.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde_json::{Value, json};

/// Version used when falling back to `bunx`; matches the root `package.json`.
const GLTF_TRANSFORM_VERSION: &str = "4.5.1";

/// `game/assets-src/`.
pub fn default_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets-src")
}

/// `static/latticefall/` at the repository root.
pub fn default_out() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../static/latticefall")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn run(src: &Path, out: &Path) -> ExitCode {
    match build(src, out) {
        Ok(count) => {
            println!("assets: {count} files written to {}", out.display());
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("assets: {msg}");
            ExitCode::FAILURE
        }
    }
}

struct Job {
    src: PathBuf,
    out: String,
    copy: bool,
}

fn read_jobs(src: &Path) -> Result<Vec<Job>, String> {
    let path = src.join("assets.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let json: Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let entries = json
        .get("assets")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{}: missing an \"assets\" array", path.display()))?;
    entries
        .iter()
        .map(|e| {
            let field = |k: &str| {
                e.get(k)
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("{}: entry without \"{k}\": {e}", path.display()))
            };
            let out = field("out")?;
            if out.starts_with('/') || out.split('/').any(|p| p == "..") || !out.contains('/') {
                return Err(format!(
                    "{}: bad output path \"{out}\" (use region/name)",
                    path.display()
                ));
            }
            Ok(Job {
                src: src.join(field("src")?),
                out: out.to_string(),
                copy: e.get("copy").and_then(Value::as_bool).unwrap_or(false),
            })
        })
        .collect()
}

/// True when the GLB's JSON chunk lists at least one image.
fn glb_has_textures(path: &Path) -> Result<bool, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let bad = || format!("{}: not a GLB file", path.display());
    if bytes.len() < 20 || &bytes[0..4] != b"glTF" {
        return Err(bad());
    }
    let len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
    let json_bytes = bytes.get(20..20 + len).ok_or_else(bad)?;
    let json: Value =
        serde_json::from_slice(json_bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(json
        .get("images")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty()))
}

fn gltf_transform() -> Command {
    let local = repo_root().join("node_modules/.bin/gltf-transform");
    if local.exists() {
        Command::new(local)
    } else {
        let mut c = Command::new("bunx");
        c.arg(format!("@gltf-transform/cli@{GLTF_TRANSFORM_VERSION}"));
        c
    }
}

fn optimize(job: &Job, dest: &Path) -> Result<(), String> {
    let textured = glb_has_textures(&job.src)?;
    if textured && Command::new("toktx").arg("--version").output().is_err() {
        return Err(format!(
            "{} has textures but `toktx` (KTX-Software) is not installed; see docs/game/decisions/0005-asset-pipeline-tools.md",
            job.src.display()
        ));
    }
    // Skinned characters: keep the scene graph, don't join meshes, and keep the flat material
    // colors (armor tints later edit them), so palette textures and simplification are off.
    let status = gltf_transform()
        .arg("optimize")
        .arg(&job.src)
        .arg(dest)
        .args(["--compress", "meshopt"])
        .args([
            "--flatten",
            "false",
            "--join",
            "false",
            "--instance",
            "false",
        ])
        .args(["--palette", "false", "--simplify", "false"])
        .args([
            "--texture-compress",
            if textured { "ktx2" } else { "false" },
        ])
        .status()
        .map_err(|e| format!("could not run gltf-transform (run `bun install`): {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "gltf-transform failed on {} ({status})",
            job.src.display()
        ))
    }
}

fn build(src: &Path, out: &Path) -> Result<usize, String> {
    let jobs = read_jobs(src)?;
    let mut regions: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for job in &jobs {
        let dest = out.join(&job.out);
        if let Some(dir) = dest.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        println!("assets: {} -> {}", job.src.display(), job.out);
        if job.copy {
            std::fs::copy(&job.src, &dest).map_err(|e| format!("{}: {e}", job.src.display()))?;
        } else {
            optimize(job, &dest)?;
        }
        let bytes = std::fs::metadata(&dest)
            .map_err(|e| format!("{}: {e}", dest.display()))?
            .len();
        let (region, name) = job.out.split_once('/').unwrap_or(("", &job.out));
        regions
            .entry(region.to_string())
            .or_default()
            .push(json!({ "path": name, "bytes": bytes }));
    }
    let manifest = json!({ "regions": regions });
    let text = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(out.join("manifest.json"), text + "\n")
        .map_err(|e| format!("manifest.json: {e}"))?;
    Ok(jobs.len() + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_escaping_output_paths() {
        let dir = std::env::temp_dir().join(format!("lf-assets-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("assets.json"),
            r#"{"assets":[{"src":"a.glb","out":"../x.glb"}]}"#,
        )
        .unwrap();
        assert!(read_jobs(&dir).is_err());
        std::fs::write(
            dir.join("assets.json"),
            r#"{"assets":[{"src":"a.glb","out":"actors/x.glb"}]}"#,
        )
        .unwrap();
        let jobs = read_jobs(&dir).unwrap();
        assert_eq!(jobs[0].out, "actors/x.glb");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn shipped_job_list_parses() {
        let jobs = read_jobs(&default_src()).unwrap();
        assert!(
            jobs.iter().all(|j| j.src.exists()),
            "every source file exists"
        );
    }
}
