//! `lf-tools bake`: bake derived data from the level colliders.

mod nav;

use std::path::Path;
use std::process::ExitCode;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Action {
    /// Bake the navigation grid of a scenario (docs/game/01-architecture.md, "Navigation").
    Nav {
        /// The scenario name, for example `test_field`.
        scenario: String,
        /// Where to write the grid. Defaults to `game/data/nav/<scenario>.nav`.
        #[arg(long)]
        out: Option<std::path::PathBuf>,
    },
}

pub fn run(action: Action, data: &Path) -> ExitCode {
    let result = match action {
        Action::Nav { scenario, out } => nav::run(data, &scenario, out.as_deref()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
