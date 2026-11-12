//! `lf-tools bake nav`: bakes a scenario's navigation grid to a file.

use std::fs;
use std::path::Path;

use lf_content::{Content, NavConfig};
use lf_core::nav::bake::bake;
use lf_core::physics::PhysicsState;

pub fn run(data: &Path, scenario: &str, out: Option<&Path>) -> Result<(), String> {
    let content = Content::load_dir(data).map_err(|e| e.to_string())?;
    let level = content
        .scenario(scenario)
        .ok_or_else(|| format!("unknown scenario {scenario:?} (no scenarios/{scenario}.ron)"))?;
    let config = NavConfig::load_dir(data)?;
    let physics = PhysicsState::new(scenario, level);
    let grid = bake(&physics, level, &config);
    // Bake a second time and compare: a mismatch means the bake isn't deterministic.
    let again = bake(&physics, level, &config);
    if grid.hash() != again.hash() {
        return Err("baking twice gave different grids".into());
    }
    let path = match out {
        Some(p) => p.to_path_buf(),
        None => data.join("nav").join(format!("{scenario}.nav")),
    };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let bytes = grid.to_bytes();
    fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    println!(
        "baked {scenario}: {}x{} columns, {} cells, {} bytes, hash {:016x} -> {}",
        grid.width,
        grid.depth,
        grid.cell_count(),
        bytes.len(),
        grid.hash(),
        path.display()
    );
    Ok(())
}
