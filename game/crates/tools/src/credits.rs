//! `lf-tools credits`: collects the title, author, source, and license of every asset pack in
//! `game/assets-src/` into one JSON file, for the in-game and on-site credits.
//!
//! A pack is a folder with a `SOURCE.md`. Its first `# ` heading is the title, and bullets of
//! the form `- **Author:** ...`, `- **License:** ...`, and `- **Source:** ...` give the rest.
//! A pack's license file is any file whose name starts with `LICENSE`. A pack with no license
//! file gets a warning, not an error, so one gap doesn't hide the rest of the credits.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{Value, json};

/// Where the credits JSON goes by default: the audio module, which the game imports.
pub fn default_out() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../src/lib/latticefall/audio/credits.json")
}

pub fn run(src: &Path, out: &Path) -> ExitCode {
    match build(src, out) {
        Ok(count) => {
            println!("credits: {count} packs written to {}", out.display());
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("credits: {msg}");
            ExitCode::FAILURE
        }
    }
}

/// One asset pack's credit line. Missing fields are `None`, and they become `null` in the JSON.
#[derive(Debug, Default, PartialEq)]
struct Pack {
    folder: String,
    title: String,
    author: Option<String>,
    license: Option<String>,
    source: Option<String>,
    license_file: Option<String>,
}

fn build(src: &Path, out: &Path) -> Result<usize, String> {
    let packs = collect(src)?;
    for pack in &packs {
        if pack.license_file.is_none() {
            eprintln!("credits: warning: {} has no license file", pack.folder);
        }
    }
    let value = json!({ "packs": packs.iter().map(pack_json).collect::<Vec<Value>>() });
    let text = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(out, text + "\n").map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(packs.len())
}

fn pack_json(pack: &Pack) -> Value {
    json!({
        "folder": pack.folder,
        "title": pack.title,
        "author": pack.author,
        "license": pack.license,
        "source": pack.source,
        "licenseFile": pack.license_file,
    })
}

/// Reads every pack folder under `src`, in name order.
fn collect(src: &Path) -> Result<Vec<Pack>, String> {
    let entries = std::fs::read_dir(src).map_err(|e| format!("{}: {e}", src.display()))?;
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir() && !is_hidden(p))
        .collect();
    dirs.sort();
    let mut packs = Vec::new();
    for dir in dirs {
        let folder = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let source = dir.join("SOURCE.md");
        if !source.is_file() {
            eprintln!("credits: warning: {folder} has no SOURCE.md, skipped");
            continue;
        }
        let text =
            std::fs::read_to_string(&source).map_err(|e| format!("{}: {e}", source.display()))?;
        let mut pack = parse_source(&text).map_err(|e| format!("{folder}/SOURCE.md: {e}"))?;
        pack.folder = folder;
        pack.license_file = license_file(&dir)?;
        packs.push(pack);
    }
    Ok(packs)
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with('.'))
}

/// The first file in `dir` whose name starts with `LICENSE`, if any.
fn license_file(dir: &Path) -> Result<Option<String>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.to_ascii_uppercase().starts_with("LICENSE"))
        .collect();
    names.sort();
    Ok(names.into_iter().next())
}

/// Reads a `SOURCE.md`: the title from the first `# ` heading, and the labeled bullets.
fn parse_source(text: &str) -> Result<Pack, String> {
    let title = text
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(|t| t.trim().to_string())
        .ok_or("no `# ` title")?;
    Ok(Pack {
        title,
        author: field(text, "Author"),
        license: field(text, "License"),
        source: field(text, "Source"),
        ..Pack::default()
    })
}

/// The value of a `- **Label:** value` bullet, or `None` when the label is absent or empty.
fn field(text: &str, label: &str) -> Option<String> {
    let prefix = format!("- **{label}:**");
    text.lines()
        .find_map(|line| line.trim_start().strip_prefix(prefix.as_str()))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_title_and_labeled_bullets() {
        let text = "# Sample Pack\n\n- **Author:** Jane Doe\n- **License:** CC0 1.0\n- **Source:** https://example.test\n";
        let pack = parse_source(text).unwrap();
        assert_eq!(pack.title, "Sample Pack");
        assert_eq!(pack.author.as_deref(), Some("Jane Doe"));
        assert_eq!(pack.license.as_deref(), Some("CC0 1.0"));
        assert_eq!(pack.source.as_deref(), Some("https://example.test"));
    }

    #[test]
    fn a_missing_or_empty_field_is_none() {
        let pack = parse_source("# Bare\n- **Author:**\n").unwrap();
        assert_eq!(pack.author, None);
        assert_eq!(pack.license, None);
    }

    #[test]
    fn a_source_file_needs_a_title() {
        assert!(parse_source("- **License:** CC0\n").is_err());
    }

    #[test]
    fn every_shipped_pack_has_a_title_and_the_audio_pack_is_listed() {
        let packs = collect(&crate::assets::default_src()).unwrap();
        assert!(packs.iter().all(|p| !p.title.is_empty()));
        let audio = packs
            .iter()
            .find(|p| p.folder == "audio")
            .expect("the audio pack has SOURCE.md");
        assert!(audio.license.is_some());
    }
}
