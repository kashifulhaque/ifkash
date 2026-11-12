//! `lf-tools`: command-line tools for Latticefall content and replays.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lf-tools", about = "Latticefall command-line tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate the content data in `game/data/`.
    Validate,
    /// Run the simulation benchmarks.
    Bench,
    /// Record, replay, or rehash golden replays.
    Replay,
    /// Generate TypeScript types from the Rust data model.
    GenTs,
}

fn main() {
    let cli = Cli::parse();
    let name = match cli.command {
        Command::Validate => "validate",
        Command::Bench => "bench",
        Command::Replay => "replay",
        Command::GenTs => "gen-ts",
    };
    println!("lf-tools {name}: not yet implemented");
}
