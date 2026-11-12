//! `lf-tools bake AREA`: compile a level and write its collision set, navigation grid,
//! and render manifest.

use std::fs;
use std::path::Path;

use lf_content::{Content, Kits, Level, NavConfig, Scenario};
use lf_core::nav::NavGrid;
use lf_core::nav::bake::bake;
use lf_core::physics::PhysicsState;

use super::manifest;
use super::warnings::{self, Warning};

/// Everything one bake produces.
pub struct Baked {
    pub scenario: Scenario,
    pub grid: NavGrid,
    /// The render manifest as JSON text.
    pub manifest: String,
    /// The collision set as JSON text.
    pub colliders: String,
    pub warnings: Vec<Warning>,
}

/// Bakes a level. `name` labels the physics world.
pub fn bake_level(
    name: &str,
    level: &Level,
    kits: &Kits<'_>,
    nav: &NavConfig,
) -> Result<Baked, String> {
    let scenario = level.compile(kits)?;
    let physics = PhysicsState::new(name, &scenario);
    let grid = bake(&physics, &scenario, nav);
    let manifest = manifest::build(level, kits, &scenario)?;
    // The core's static level.
    let colliders = serde_json::json!({
        "heightfield": scenario.heightfield,
        "boxes": scenario.boxes,
    })
    .to_string();
    let warnings = warnings::check(level, &scenario, &grid);
    Ok(Baked {
        scenario,
        grid,
        manifest,
        colliders,
        warnings,
    })
}

pub fn run(
    data: &Path,
    area: &str,
    out: &Path,
    manifest_path: &Path,
    deny_warnings: bool,
) -> Result<(), String> {
    let content = Content::load_dir(data).map_err(|e| e.to_string())?;
    let level = content
        .level(area)
        .ok_or_else(|| format!("unknown area {area:?} (no levels/{area}/area.ron)"))?;
    let kits = Kits::new(content.kits.iter().map(|(f, c)| (f.as_str(), c)))?;
    let nav = NavConfig::load_dir(data)?;
    let baked = bake_level(area, level, &kits, &nav)?;
    // Bake a second time and compare: a mismatch means the bake isn't deterministic.
    let again = bake_level(area, level, &kits, &nav)?;
    if baked.grid.hash() != again.grid.hash()
        || baked.manifest != again.manifest
        || baked.colliders != again.colliders
    {
        return Err("baking twice gave different output".into());
    }
    if content.scenario(area) != Some(&baked.scenario) {
        return Err("the bake's level differs from the one the content loader compiled".into());
    }
    let write = |path: &Path, bytes: &[u8]| -> Result<(), String> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
    };
    let nav_path = out.join("nav.bin");
    let colliders_path = out.join("colliders.json");
    write(&nav_path, &baked.grid.to_bytes())?;
    write(&colliders_path, baked.colliders.as_bytes())?;
    write(manifest_path, baked.manifest.as_bytes())?;
    let sc = &baked.scenario;
    println!(
        "baked {area}: {} boxes{}, nav {}x{} columns, {} cells, hash {:016x}",
        sc.boxes.len(),
        sc.heightfield
            .as_ref()
            .map(|h| format!(", {}x{} terrain", h.cols, h.rows))
            .unwrap_or_default(),
        baked.grid.width,
        baked.grid.depth,
        baked.grid.cell_count(),
        baked.grid.hash(),
    );
    println!("  colliders -> {}", colliders_path.display());
    println!("  nav grid  -> {}", nav_path.display());
    println!("  manifest  -> {}", manifest_path.display());
    for w in &baked.warnings {
        println!("warning ({}): {}", w.kind.slug(), w.message);
    }
    println!("{} warning(s)", baked.warnings.len());
    if deny_warnings && !baked.warnings.is_empty() {
        return Err(format!(
            "{} warning(s) with --deny-warnings",
            baked.warnings.len()
        ));
    }
    Ok(())
}
