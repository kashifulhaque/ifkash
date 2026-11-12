//! `lf-tools replay`: run, rehash, and record replay files.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Subcommand;
use lf_content::Content;
use lf_core::input::{ActionFrame, Command, buttons};
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
    /// Record a replay from a scripted input file (see `parse_script` for the format).
    RecordScript {
        script: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

pub fn replays_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/replays")
}

fn content(data: &Path) -> Result<Content, String> {
    // Same source as `lf-tools validate`.
    Content::load_dir(data).map_err(|e| e.to_string())
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
            let content = content(data)?;
            match run_replay(&replay, &content) {
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
            let content = content(data)?;
            for f in files {
                let old = load(&f)?;
                let new = Replay::record(
                    old.header.seed,
                    &old.header.scenario,
                    &content,
                    &expand(&old.frames),
                    old.commands.clone(),
                )
                .map_err(|e| format!("{}: {e}", f.display()))?;
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
            let replay = Replay::record(seed, "m0_flat", &content(data)?, &frames, commands)
                .map_err(|e| e.to_string())?;
            write_replay(&out, &replay)
        }
        Action::RecordScript { script, out } => {
            let text =
                fs::read_to_string(&script).map_err(|e| format!("{}: {e}", script.display()))?;
            let s = parse_script(&text).map_err(|e| format!("{}: {e}", script.display()))?;
            let replay =
                Replay::record(s.seed, &s.scenario, &content(data)?, &s.frames, s.commands)
                    .map_err(|e| e.to_string())?;
            write_replay(&out, &replay)
        }
    }
}

fn write_replay(out: &Path, replay: &Replay) -> Result<ExitCode, String> {
    let bytes = replay.encode();
    fs::write(out, &bytes).map_err(|e| format!("{}: {e}", out.display()))?;
    println!(
        "{}: {} ticks, {} bytes, final hash {:016x}",
        out.display(),
        replay.total_ticks(),
        bytes.len(),
        replay.hashes.last().map_or(0, |h| h.1)
    );
    Ok(ExitCode::SUCCESS)
}

/// A parsed input script.
pub struct Script {
    pub seed: u64,
    pub scenario: String,
    pub frames: Vec<ActionFrame>,
    /// `(tick, command)` pairs from `cmd` lines.
    pub commands: Vec<(u64, Command)>,
}

/// Parses an input script. Blank lines and text after `#` are ignored. The header lines
/// `seed <u64>` and `scenario <name>` are required; every other line is
/// `<ticks> <move_x> <move_z> [BUTTON+BUTTON...]`, holding that frame for `ticks` ticks.
/// Button names are the `ActionFrame` button names, for example `DODGE+JUMP`.
/// `cmd Rest`, `cmd SplitFlasks <vigil>`, `cmd LevelUp <attribute>`, `cmd Equip <slot> <item>`
/// (the item is a registry number), and `cmd Unequip <slot>` queue a command on the tick of
/// the next frame line (or on the tick after the last frame, if none follows).
pub fn parse_script(text: &str) -> Result<Script, String> {
    let mut seed = None;
    let mut scenario = None;
    let mut frames = Vec::new();
    let mut commands = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let err = |m: &str| format!("line {}: {m}: {raw:?}", n + 1);
        let words: Vec<&str> = line.split_whitespace().collect();
        match words[0] {
            "seed" if words.len() == 2 => {
                seed = Some(words[1].parse::<u64>().map_err(|_| err("bad seed"))?);
            }
            "scenario" if words.len() == 2 => scenario = Some(words[1].to_owned()),
            "cmd" if words.len() >= 2 => {
                let command = match (words[1], words.get(2)) {
                    ("Rest", None) => Command::Rest,
                    ("SplitFlasks", Some(v)) => Command::SplitFlasks {
                        vigil: v.parse().map_err(|_| err("bad vigil count"))?,
                    },
                    ("LevelUp", Some(v)) => Command::LevelUp {
                        attribute: v.parse().map_err(|_| err("bad attribute index"))?,
                    },
                    ("Equip", Some(v)) => Command::Equip {
                        slot: v.parse().map_err(|_| err("bad slot number"))?,
                        item: words
                            .get(3)
                            .and_then(|i| i.parse().ok())
                            .ok_or_else(|| err("Equip needs a slot and a registry number"))?,
                    },
                    ("Unequip", Some(v)) => Command::Unequip {
                        slot: v.parse().map_err(|_| err("bad slot number"))?,
                    },
                    _ => {
                        return Err(err(
                            "unknown command (Rest, SplitFlasks <vigil>, LevelUp <attribute>, \
                             Equip <slot> <item>, Unequip <slot>)",
                        ));
                    }
                };
                commands.push((frames.len() as u64, command));
            }
            _ if words.len() == 3 || words.len() == 4 => {
                let ticks: u32 = words[0].parse().map_err(|_| err("bad tick count"))?;
                let move_x: i8 = words[1].parse().map_err(|_| err("bad move_x"))?;
                let move_z: i8 = words[2].parse().map_err(|_| err("bad move_z"))?;
                let mut mask = 0u16;
                if let Some(names) = words.get(3) {
                    for name in names.split('+') {
                        mask |= button(name).ok_or_else(|| err("unknown button"))?;
                    }
                }
                let frame = ActionFrame {
                    move_x,
                    move_z,
                    buttons: mask,
                };
                frames.extend(std::iter::repeat_n(frame, ticks as usize));
            }
            _ => return Err(err("expected `<ticks> <move_x> <move_z> [BUTTONS]`")),
        }
    }
    Ok(Script {
        seed: seed.ok_or("missing `seed` line")?,
        scenario: scenario.ok_or("missing `scenario` line")?,
        frames,
        commands,
    })
}

fn button(name: &str) -> Option<u16> {
    Some(match name {
        "LIGHT" => buttons::LIGHT,
        "HEAVY" => buttons::HEAVY,
        "GUARD" => buttons::GUARD,
        "DODGE" => buttons::DODGE,
        "ART" => buttons::ART,
        "JUMP" => buttons::JUMP,
        "USE_ITEM" => buttons::USE_ITEM,
        "INTERACT" => buttons::INTERACT,
        "LOCK" => buttons::LOCK,
        "SWITCH_L" => buttons::SWITCH_L,
        "SWITCH_R" => buttons::SWITCH_R,
        "TWO_HAND" => buttons::TWO_HAND,
        _ => return None,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_lines_queue_commands_on_the_next_frame() {
        let s = parse_script(
            "seed 1\nscenario test_field\n3 0 0\ncmd Rest # at tick 3\ncmd SplitFlasks 2\n2 0 0\n",
        )
        .unwrap();
        assert_eq!(s.frames.len(), 5);
        assert_eq!(
            s.commands,
            [(3, Command::Rest), (3, Command::SplitFlasks { vigil: 2 })]
        );
        assert!(parse_script("seed 1\nscenario x\ncmd Dance\n").is_err());
    }
}
