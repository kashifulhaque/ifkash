//! `lf-tools replay`: run, rehash, and record replay files.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Subcommand;
use lf_content::Content;
use lf_core::input::{ActionFrame, Command};
use lf_core::replay::{Replay, ReplayError, expand, run_replay};
use lf_core::rng::Pcg32;

#[derive(Subcommand)]
pub enum Action {
    /// Run a replay and check every recorded hash.
    Run { file: PathBuf },
    /// Re-record expected hashes and the content hash in place. With no FILE, rehashes
    /// every `.lfr` in `game/tests/replays/`.
    Rehash { file: Option<PathBuf> },
    /// Record a replay with random inputs (the placeholder golden replay).
    RecordRandom {
        #[arg(long)]
        seed: u64,
        #[arg(long)]
        ticks: u64,
        #[arg(long)]
        out: PathBuf,
    },
}

pub fn replays_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/replays")
}

fn content_hash(data: &Path) -> Result<u64, String> {
    // Same source as `lf-tools validate`.
    Content::load_dir(data)
        .map(|c| c.content_hash())
        .map_err(|e| e.to_string())
}

fn load(file: &Path) -> Result<Replay, String> {
    let bytes = fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
    Replay::decode(&bytes).map_err(|e| format!("{}: {e}", file.display()))
}

pub fn run(action: Action, data: &Path) -> ExitCode {
    match go(action, data) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn go(action: Action, data: &Path) -> Result<ExitCode, String> {
    match action {
        Action::Run { file } => {
            let replay = load(&file)?;
            let hash = content_hash(data)?;
            match run_replay(&replay, hash) {
                Ok(report) => {
                    println!(
                        "{}: ticks {}, final hash {:016x}, OK",
                        file.display(),
                        report.ticks,
                        report.final_hash
                    );
                    Ok(ExitCode::SUCCESS)
                }
                Err(e @ ReplayError::HashMismatch { .. }) => {
                    println!("{}: MISMATCH: {e}", file.display());
                    Ok(ExitCode::FAILURE)
                }
                Err(e) => Err(format!("{}: {e}", file.display())),
            }
        }
        Action::Rehash { file } => {
            let files = match file {
                Some(f) => vec![f],
                None => {
                    let mut v: Vec<PathBuf> = fs::read_dir(replays_dir())
                        .map_err(|e| e.to_string())?
                        .filter_map(|e| e.ok().map(|e| e.path()))
                        .filter(|p| p.extension().is_some_and(|x| x == "lfr"))
                        .collect();
                    v.sort();
                    v
                }
            };
            let hash = content_hash(data)?;
            for f in files {
                let old = load(&f)?;
                let new = Replay::record(
                    old.header.seed,
                    &old.header.scenario,
                    hash,
                    &expand(&old.frames),
                    old.commands.clone(),
                );
                let fin = |r: &Replay| r.hashes.last().map_or(0, |h| h.1);
                fs::write(&f, new.encode()).map_err(|e| format!("{}: {e}", f.display()))?;
                println!(
                    "{}: final hash {:016x} -> {:016x}",
                    f.display(),
                    fin(&old),
                    fin(&new)
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        Action::RecordRandom { seed, ticks, out } => {
            let (frames, commands) = random_inputs(seed, ticks);
            let replay = Replay::record(seed, "m0_flat", content_hash(data)?, &frames, commands);
            let bytes = replay.encode();
            fs::write(&out, &bytes).map_err(|e| format!("{}: {e}", out.display()))?;
            println!(
                "{}: {} ticks, {} bytes, final hash {:016x}",
                out.display(),
                ticks,
                bytes.len(),
                replay.hashes.last().map_or(0, |h| h.1)
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// Random stick and button input held for 1 to 12 ticks at a time, plus a few commands.
/// Deterministic in `seed`.
pub fn random_inputs(seed: u64, ticks: u64) -> (Vec<ActionFrame>, Vec<(u64, Command)>) {
    let mut rng = Pcg32::new(seed, 0x1f2);
    let mut frames = Vec::with_capacity(ticks as usize);
    while (frames.len() as u64) < ticks {
        let hold = 1 + rng.next_u32() % 12;
        let still = rng.next_u32().is_multiple_of(5);
        let stick = |r: &mut Pcg32| (r.next_u32() % 255) as i32 - 127;
        let frame = ActionFrame {
            move_x: if still { 0 } else { stick(&mut rng) as i8 },
            move_z: if still { 0 } else { stick(&mut rng) as i8 },
            buttons: if rng.next_u32().is_multiple_of(3) {
                (rng.next_u32() & 0xffff) as u16
            } else {
                0
            },
        };
        for _ in 0..hold {
            if (frames.len() as u64) < ticks {
                frames.push(frame);
            }
        }
    }
    let commands = [
        (100, Command::Rest),
        (900, Command::LevelUp { attribute: 2 }),
        (1500, Command::Equip { slot: 0, item: 7 }),
        (2400, Command::Gesture(3)),
    ]
    .into_iter()
    .filter(|&(t, _)| t < ticks)
    .collect();
    (frames, commands)
}
