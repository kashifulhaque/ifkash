//! `lf-tools bake`: bake an area's derived data (docs/game/01-architecture.md, "Levels").
//!
//! `lf-tools bake AREA` compiles `game/data/levels/AREA/` with the kit catalogs and
//! writes three files:
//!
//! - The collision set, `colliders.json`: the heightfield and boxes the core builds its
//!   static level from.
//! - The navigation grid, `nav.bin`: postcard bytes of `lf_core::nav::NavGrid`.
//! - The render manifest, `AREA.json`: terrain, kit instances, scatter, atmosphere, and
//!   entity markers for the renderer (see [`manifest`]).
//!
//! The core compiles levels itself at load time with the same code, so the collision set
//! and grid are for tools and inspection; they go to `game/target/levels/AREA/` and
//! aren't committed. The manifest goes to `static/latticefall/levels/AREA.json` and is
//! committed, because the browser fetches it. The bake runs twice and fails if the two
//! runs differ, then prints warnings for problems a level author must fix (see
//! [`warnings`]).

mod area;
mod manifest;
mod nav;
mod warnings;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Subcommand};

/// Arguments of `lf-tools bake`.
#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct BakeArgs {
    /// The area to bake: a directory in `game/data/levels/`, for example `test_field`.
    pub area: Option<String>,
    /// Where to write `colliders.json` and `nav.bin`. Defaults to
    /// `game/target/levels/AREA/`.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Where to write the render manifest. Defaults to
    /// `static/latticefall/levels/AREA.json`.
    #[arg(long)]
    pub manifest: Option<PathBuf>,
    /// Exit with an error if the bake reports any warning.
    #[arg(long)]
    pub deny_warnings: bool,
    #[command(subcommand)]
    pub action: Option<Action>,
}

#[derive(Subcommand)]
pub enum Action {
    /// Bake only the navigation grid of a scenario or level
    /// (docs/game/01-architecture.md, "Navigation").
    Nav {
        /// The scenario name, for example `fight_husk` or `test_field`.
        scenario: String,
        /// Where to write the grid. Defaults to `game/data/nav/<scenario>.nav`.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

/// `static/latticefall/levels/` at the repository root.
pub fn default_manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../static/latticefall/levels")
}

/// `game/target/levels/`.
fn default_out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/levels")
}

pub fn run(args: BakeArgs, data: &Path) -> ExitCode {
    let result = match (args.action, args.area) {
        (Some(Action::Nav { scenario, out }), _) => nav::run(data, &scenario, out.as_deref()),
        (None, Some(area)) => {
            let out = args.out.unwrap_or_else(|| default_out_dir().join(&area));
            let manifest = args
                .manifest
                .unwrap_or_else(|| default_manifest_dir().join(format!("{area}.json")));
            area::run(data, &area, &out, &manifest, args.deny_warnings)
        }
        (None, None) => Err("name an area to bake, for example `bake test_field`".into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
