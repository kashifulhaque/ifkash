//! `lf-tools`: command-line tools for Latticefall content and replays.

mod assets;
mod bench;
mod gen_ts;
mod replay;
mod validate;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// `game/data/`, resolved at compile time so the tools work from any directory.
fn default_data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// `src/lib/latticefall/gen/` at the repository root.
fn default_gen_out() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../src/lib/latticefall/gen")
}

#[derive(Parser)]
#[command(name = "lf-tools", about = "Latticefall command-line tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate the content data in `game/data/`.
    Validate {
        /// The data directory. Defaults to `game/data/`.
        #[arg(long)]
        data: Option<PathBuf>,
    },
    /// Run the simulation benchmarks.
    Bench {
        /// Number of ticks to simulate.
        #[arg(long, default_value_t = 100_000)]
        ticks: u64,
    },
    /// Record, replay, or rehash golden replays.
    Replay {
        #[command(subcommand)]
        action: replay::Action,
    },
    /// Optimize the source art in `game/assets-src/` into `static/latticefall/`.
    Assets {
        /// The source directory. Defaults to `game/assets-src/`.
        #[arg(long)]
        src: Option<PathBuf>,
        /// The output directory. Defaults to `static/latticefall/`.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Generate TypeScript types and a UI digest from the content data.
    GenTs {
        /// The data directory. Defaults to `game/data/`.
        #[arg(long)]
        data: Option<PathBuf>,
        /// The output directory. Defaults to `src/lib/latticefall/gen/`.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Validate { data } => validate::run(&data.unwrap_or_else(default_data)),
        Command::GenTs { data, out } => gen_ts::run(
            &data.unwrap_or_else(default_data),
            &out.unwrap_or_else(default_gen_out),
        ),
        Command::Assets { src, out } => assets::run(
            &src.unwrap_or_else(assets::default_src),
            &out.unwrap_or_else(assets::default_out),
        ),
        Command::Bench { ticks } => bench::run(ticks),
        Command::Replay { action } => replay::run(action, &default_data()),
    }
}
