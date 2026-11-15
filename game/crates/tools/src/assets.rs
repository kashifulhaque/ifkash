//! `lf-tools assets`: optimizes the source art in `game/assets-src/` into `static/latticefall/`.
//!
//! The job list is `game/assets-src/assets.json`, plus `assets.private.json` beside it when
//! that file exists. The private list is gitignored: it holds art whose license forbids a
//! public repository (Mixamo characters), and its outputs stay out of the manifest and out
//! of Git (`static/latticefall/private/`); see docs/game/tutorial-production-plan.md,
//! "Private art". Each entry is either a glTF binary that goes through `gltf-transform
//! optimize` (meshopt-compressed geometry and animation, KTX2 textures when the file has
//! any) or a plain file that is copied. A `manifest.json` listing
//! every output, grouped by its first path segment (the region), is written next to them.
//!
//! The tool runs `gltf-transform` from the repository's `node_modules` (a Bun devDependency)
//! and falls back to `bunx`. Files with no textures need nothing else.
//!
//! A file with textures also needs KTX-Software's `ktx` tool, which
//! `game/scripts/install-ktx.sh` installs without sudo
//! (docs/game/decisions/0010-ktx2-textures.md). `gltf-transform optimize` then writes each
//! texture as KTX2 with mipmaps: UASTC for normal, occlusion, and metallic-roughness maps
//! (linear), and ETC1S for every other map (base color and emissive, sRGB). No texture is
//! written larger than the maximum texture size. When any output has KTX2 textures, the
//! tool also copies the Basis transcoder from the installed `three` package into
//! `basis/`, so the transcoder matches the renderer's Three.js version.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde_json::{Value, json};

/// Version used when falling back to `bunx`; matches the root `package.json`.
const GLTF_TRANSFORM_VERSION: &str = "4.5.1";

/// The KTX-Software release that `game/scripts/install-ktx.sh` installs. Keep them in sync.
const KTX_VERSION: &str = "4.4.2";
/// The oldest `ktx` that gltf-transform 4.5 accepts.
const KTX_VERSION_MIN: [u32; 3] = [4, 4, 0];
const KTX_HELP: &str = "run `bash game/scripts/install-ktx.sh` (or `mise run game:ktx`); see docs/game/decisions/0010-ktx2-textures.md";

/// The longest edge, in pixels, of a written texture unless the job or the command line
/// sets another. Larger sources are scaled down; smaller ones keep their size.
pub const DEFAULT_MAX_TEXTURE_SIZE: u32 = 2048;

/// The Basis transcoder files that the renderer's `KTX2Loader` fetches from `basis/`.
const TRANSCODER_FILES: [&str; 2] = ["basis_transcoder.js", "basis_transcoder.wasm"];

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

pub fn run(src: &Path, out: &Path, max_texture_size: u32) -> ExitCode {
    match build(src, out, max_texture_size) {
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
    /// The job's own `maxTextureSize`, which overrides the command line's.
    max_texture_size: Option<u32>,
    /// From `assets.private.json`: built, but left out of the manifest.
    private: bool,
}

fn check_texture_size(size: u32) -> Result<u32, String> {
    if (4..=8192).contains(&size) && size.is_power_of_two() {
        Ok(size)
    } else {
        Err(format!(
            "bad maximum texture size {size} (use a power of two from 4 to 8192)"
        ))
    }
}

/// The public job list, then the private one when it exists.
fn read_jobs(src: &Path) -> Result<Vec<Job>, String> {
    let mut jobs = read_job_file(src, "assets.json", false)?;
    if src.join(PRIVATE_JOBS).exists() {
        jobs.extend(read_job_file(src, PRIVATE_JOBS, true)?);
    }
    Ok(jobs)
}

/// The gitignored job list for art that must stay out of the public repository.
const PRIVATE_JOBS: &str = "assets.private.json";

fn read_job_file(src: &Path, name: &str, private: bool) -> Result<Vec<Job>, String> {
    let path = src.join(name);
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
            if private != out.starts_with("private/") {
                return Err(format!(
                    "{}: \"{out}\": private art goes under private/, and only private art does",
                    path.display()
                ));
            }
            Ok(Job {
                src: src.join(field("src")?),
                out: out.to_string(),
                copy: e.get("copy").and_then(Value::as_bool).unwrap_or(false),
                max_texture_size: match e.get("maxTextureSize") {
                    None => None,
                    Some(v) => Some(
                        v.as_u64()
                            .and_then(|n| u32::try_from(n).ok())
                            .ok_or_else(|| format!("maxTextureSize {v}: not a size"))
                            .and_then(check_texture_size)
                            .map_err(|m| format!("{}: \"{out}\": {m}", path.display()))?,
                    ),
                },
                private,
            })
        })
        .collect()
}

/// The MIME type of every image in the GLB's JSON chunk. Empty when it has no textures.
fn glb_image_types(path: &Path) -> Result<Vec<String>, String> {
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
        .map(|images| {
            images
                .iter()
                .map(|i| {
                    i.get("mimeType")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string()
                })
                .collect()
        })
        .unwrap_or_default())
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

/// Reads the version from `ktx --version` output, which looks like `ktx version: v4.4.2`.
fn parse_ktx_version(text: &str) -> Option<[u32; 3]> {
    let v = text
        .split_whitespace()
        .filter_map(|w| w.strip_prefix('v'))
        .find(|w| w.starts_with(|c: char| c.is_ascii_digit()))?;
    // A build past a release reads `v4.4.2~3`; a release candidate reads `v5.0.0-rc1`.
    let v = v.split(['~', '-', '+']).next()?;
    let mut parts = v.split('.').map(|p| p.parse::<u32>().ok());
    Some([parts.next()??, parts.next()??, parts.next()??])
}

/// Where `game/scripts/install-ktx.sh` puts KTX-Software: `LF_KTX_DIR`, or
/// `latticefall/ktx-VERSION` under `XDG_DATA_HOME` (`~/.local/share` by default).
fn ktx_install_dir(
    lf_ktx_dir: Option<String>,
    xdg_data_home: Option<String>,
    home: Option<String>,
) -> Option<PathBuf> {
    let set = |v: Option<String>| v.filter(|s| !s.is_empty());
    if let Some(dir) = set(lf_ktx_dir) {
        return Some(PathBuf::from(dir));
    }
    let data = set(xdg_data_home)
        .map(PathBuf::from)
        .or_else(|| set(home).map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(data.join(format!("latticefall/ktx-{KTX_VERSION}")))
}

/// Finds a usable `ktx`: the pinned install first, then the `PATH`. Returns the directory to
/// put first on the `PATH` for gltf-transform (none when `ktx` is already there).
fn find_ktx() -> Result<Option<PathBuf>, String> {
    let env = |k: &str| std::env::var(k).ok();
    let pinned = ktx_install_dir(env("LF_KTX_DIR"), env("XDG_DATA_HOME"), env("HOME"))
        .map(|d| d.join("bin"))
        .filter(|bin| bin.join("ktx").exists());
    let program = pinned
        .as_ref()
        .map_or_else(|| PathBuf::from("ktx"), |bin| bin.join("ktx"));
    let output = Command::new(&program)
        .arg("--version")
        .output()
        .map_err(|_| {
            format!("the `ktx` tool (KTX-Software {KTX_VERSION}) is not installed; {KTX_HELP}")
        })?;
    let text = String::from_utf8_lossy(&output.stdout);
    match parse_ktx_version(&text) {
        Some(v) if v >= KTX_VERSION_MIN => Ok(pinned),
        Some([a, b, c]) => Err(format!(
            "{} is KTX-Software {a}.{b}.{c}, but gltf-transform needs 4.4.0 or later; {KTX_HELP}",
            program.display()
        )),
        None => Err(format!(
            "could not read a version from `{} --version`; {KTX_HELP}",
            program.display()
        )),
    }
}

/// Optimizes one GLB. Returns true when the output has KTX2 textures.
fn optimize(job: &Job, dest: &Path, max_texture_size: u32) -> Result<bool, String> {
    let textured = !glb_image_types(&job.src)?.is_empty();
    let mut cmd = gltf_transform();
    if textured {
        let ktx_bin =
            find_ktx().map_err(|m| format!("{} has textures, but {m}", job.src.display()))?;
        if let Some(bin) = ktx_bin {
            let mut paths = vec![bin];
            paths.extend(std::env::split_paths(
                &std::env::var_os("PATH").unwrap_or_default(),
            ));
            cmd.env(
                "PATH",
                std::env::join_paths(paths).map_err(|e| e.to_string())?,
            );
        }
    }
    // Skinned characters: keep the scene graph, don't join meshes, and keep the flat material
    // colors (armor tints later edit them), so palette textures and simplification are off.
    let status = cmd
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
        .args(["--texture-size", &max_texture_size.to_string()])
        .status()
        .map_err(|e| format!("could not run gltf-transform (run `bun install`): {e}"))?;
    if !status.success() {
        return Err(format!(
            "gltf-transform failed on {} ({status})",
            job.src.display()
        ));
    }
    // gltf-transform logs a failed `ktx` run and keeps the source image, so check the output.
    if let Some(other) = glb_image_types(dest)?.iter().find(|t| *t != "image/ktx2") {
        return Err(format!(
            "{}: a texture was left as \"{other}\" instead of KTX2; read the gltf-transform output for the `ktx` error",
            dest.display()
        ));
    }
    Ok(textured)
}

/// Copies the Basis transcoder from the installed `three` package into `out/basis/`.
fn copy_transcoder(out: &Path) -> Result<(), String> {
    let from = repo_root().join("node_modules/three/examples/jsm/libs/basis");
    let to = out.join("basis");
    std::fs::create_dir_all(&to).map_err(|e| format!("{}: {e}", to.display()))?;
    for name in TRANSCODER_FILES {
        let src = from.join(name);
        std::fs::copy(&src, to.join(name))
            .map_err(|e| format!("{}: {e} (run `bun install`)", src.display()))?;
    }
    println!("assets: Basis transcoder -> basis/");
    Ok(())
}

fn build(src: &Path, out: &Path, max_texture_size: u32) -> Result<usize, String> {
    let max_texture_size = check_texture_size(max_texture_size)?;
    let jobs = read_jobs(src)?;
    let mut any_ktx2 = false;
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
            let size = job.max_texture_size.unwrap_or(max_texture_size);
            any_ktx2 |= optimize(job, &dest, size)?;
        }
        if job.private {
            continue;
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
    let mut written = jobs.len() + 1;
    if any_ktx2 {
        copy_transcoder(out)?;
        written += TRANSCODER_FILES.len();
    }
    Ok(written)
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
    fn reads_ktx_versions() {
        assert_eq!(parse_ktx_version("ktx version: v4.4.2\n"), Some([4, 4, 2]));
        assert_eq!(parse_ktx_version("ktx version: v4.3.0~12"), Some([4, 3, 0]));
        assert_eq!(
            parse_ktx_version("ktx version: v5.0.0-rc2"),
            Some([5, 0, 0])
        );
        assert_eq!(parse_ktx_version("command not found"), None);
        assert!([4, 3, 2] < KTX_VERSION_MIN && [4, 10, 0] >= KTX_VERSION_MIN);
    }

    #[test]
    fn resolves_the_ktx_install_dir() {
        let s = |v: &str| Some(v.to_string());
        assert_eq!(
            ktx_install_dir(s("/opt/ktx"), s("/data"), s("/home/a")),
            Some(PathBuf::from("/opt/ktx"))
        );
        assert_eq!(
            ktx_install_dir(None, s("/data"), s("/home/a")),
            Some(PathBuf::from(format!(
                "/data/latticefall/ktx-{KTX_VERSION}"
            )))
        );
        assert_eq!(
            ktx_install_dir(s(""), None, s("/home/a")),
            Some(PathBuf::from(format!(
                "/home/a/.local/share/latticefall/ktx-{KTX_VERSION}"
            )))
        );
        assert_eq!(ktx_install_dir(None, None, None), None);
    }

    #[test]
    fn checks_texture_sizes() {
        assert_eq!(check_texture_size(1024), Ok(1024));
        assert!(check_texture_size(1000).is_err());
        assert!(check_texture_size(2).is_err());
        assert!(check_texture_size(16384).is_err());
        let dir = std::env::temp_dir().join(format!("lf-assets-size-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let write = |size: &str| {
            std::fs::write(
                dir.join("assets.json"),
                format!(
                    r#"{{"assets":[{{"src":"a.glb","out":"dev/x.glb","maxTextureSize":{size}}}]}}"#
                ),
            )
            .unwrap();
        };
        write("512");
        assert_eq!(read_jobs(&dir).unwrap()[0].max_texture_size, Some(512));
        write("500");
        assert!(read_jobs(&dir).is_err());
        write("\"big\"");
        assert!(read_jobs(&dir).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn private_jobs_stay_under_private() {
        let dir = std::env::temp_dir().join(format!("lf-assets-private-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("assets.json"),
            r#"{"assets":[{"src":"a.glb","out":"actors/a.glb"}]}"#,
        )
        .unwrap();
        std::fs::write(
            dir.join(PRIVATE_JOBS),
            r#"{"assets":[{"src":"b.glb","out":"private/b.glb"}]}"#,
        )
        .unwrap();
        let jobs = read_jobs(&dir).unwrap();
        assert_eq!(
            jobs.iter().map(|j| j.private).collect::<Vec<_>>(),
            [false, true]
        );
        std::fs::write(
            dir.join(PRIVATE_JOBS),
            r#"{"assets":[{"src":"b.glb","out":"actors/b.glb"}]}"#,
        )
        .unwrap();
        assert!(read_jobs(&dir).is_err(), "private art outside private/");
        std::fs::write(
            dir.join("assets.json"),
            r#"{"assets":[{"src":"a.glb","out":"private/a.glb"}]}"#,
        )
        .unwrap();
        std::fs::remove_file(dir.join(PRIVATE_JOBS)).unwrap();
        assert!(read_jobs(&dir).is_err(), "public art under private/");
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
