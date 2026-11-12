//! The replay format and runner (docs/game/01-architecture.md, "State hash and golden
//! replays"). A replay is a seed, a run-length-encoded `ActionFrame` stream, commands keyed
//! by tick, and expected state hashes every `CHECKPOINT_INTERVAL` ticks. The file is
//! `postcard` bytes (`.lfr`), so the native tools, WASM/Node, and Python all load the same
//! file. This module does no I/O and does not know about `lf-content`: callers pass the
//! content hash.

use serde::{Deserialize, Serialize};

use crate::input::{ActionFrame, Command, TickInput};
use crate::tick::tick;
use crate::world::World;

/// Bumped on any incompatible change to the layout below.
pub const FORMAT_VERSION: u16 = 1;
/// Expected hashes are recorded every this many ticks, and after the final tick.
pub const CHECKPOINT_INTERVAL: u64 = 30;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayHeader {
    pub format_version: u16,
    pub content_hash: u64,
    pub seed: u64,
    pub scenario: String,
    pub start_tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replay {
    pub header: ReplayHeader,
    /// Run-length-encoded frames: each frame and how many consecutive ticks it is held.
    pub frames: Vec<(ActionFrame, u32)>,
    /// Commands keyed by the tick they are applied on (the value of `World::tick` before
    /// that tick runs), in application order.
    pub commands: Vec<(u64, Command)>,
    /// `(tick, hash)`: `World::tick` and `World::state_hash` after that tick has run.
    pub hashes: Vec<(u64, u64)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayError {
    Decode(String),
    UnsupportedVersion(u16),
    /// Only replays that start from a fresh world (`start_tick == 0`) are supported so far.
    UnsupportedStart(u64),
    ContentHashMismatch {
        expected: u64,
        actual: u64,
    },
    /// The first checkpoint whose hash differs from the recorded one.
    HashMismatch {
        tick: u64,
        expected: u64,
        actual: u64,
    },
    /// A recorded checkpoint lies beyond the last tick of the replay.
    CheckpointOutOfRange(u64),
}

impl core::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Decode(e) => write!(f, "cannot decode replay: {e}"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported replay format version {v}"),
            Self::UnsupportedStart(t) => write!(f, "unsupported start tick {t}"),
            Self::ContentHashMismatch { expected, actual } => write!(
                f,
                "content hash mismatch: replay {expected:016x}, current {actual:016x}"
            ),
            Self::HashMismatch {
                tick,
                expected,
                actual,
            } => write!(
                f,
                "state hash mismatch at tick {tick}: expected {expected:016x}, got {actual:016x}"
            ),
            Self::CheckpointOutOfRange(t) => write!(f, "checkpoint at tick {t} is past the end"),
        }
    }
}

impl std::error::Error for ReplayError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayReport {
    pub ticks: u64,
    pub final_hash: u64,
}

/// Run-length-encodes a frame stream. Runs split only at changes (and at `u32::MAX`).
pub fn compress(frames: &[ActionFrame]) -> Vec<(ActionFrame, u32)> {
    let mut out: Vec<(ActionFrame, u32)> = Vec::new();
    for &f in frames {
        match out.last_mut() {
            Some((last, run)) if *last == f && *run < u32::MAX => *run += 1,
            _ => out.push((f, 1)),
        }
    }
    out
}

/// Expands run-length-encoded frames. Zero-length runs expand to nothing.
pub fn expand(runs: &[(ActionFrame, u32)]) -> Vec<ActionFrame> {
    let mut out = Vec::new();
    for &(f, n) in runs {
        out.extend(core::iter::repeat_n(f, n as usize));
    }
    out
}

impl Replay {
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("replay serialization cannot fail")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ReplayError> {
        let r: Self =
            postcard::from_bytes(bytes).map_err(|e| ReplayError::Decode(e.to_string()))?;
        if r.header.format_version != FORMAT_VERSION {
            return Err(ReplayError::UnsupportedVersion(r.header.format_version));
        }
        Ok(r)
    }

    /// Total ticks in the replay.
    pub fn total_ticks(&self) -> u64 {
        self.frames.iter().map(|&(_, n)| u64::from(n)).sum()
    }

    /// Runs `frames` and `commands` from a fresh world and records the expected hashes.
    pub fn record(
        seed: u64,
        scenario: &str,
        content_hash: u64,
        frames: &[ActionFrame],
        commands: Vec<(u64, Command)>,
    ) -> Replay {
        let mut commands = commands;
        commands.sort_by_key(|&(t, _)| t); // stable: keeps order within a tick
        let mut r = Replay {
            header: ReplayHeader {
                format_version: FORMAT_VERSION,
                content_hash,
                seed,
                scenario: scenario.to_string(),
                start_tick: 0,
            },
            frames: compress(frames),
            commands,
            hashes: Vec::new(),
        };
        r.hashes = simulate(&r, |_, _| Ok(()))
            .expect("recording cannot mismatch")
            .1;
        r
    }
}

/// Steps a fresh world through the replay, calling `check(tick, hash)` at every checkpoint
/// tick. Returns the report and the hash at every checkpoint.
fn simulate(
    r: &Replay,
    mut check: impl FnMut(u64, u64) -> Result<(), ReplayError>,
) -> Result<(ReplayReport, Vec<(u64, u64)>), ReplayError> {
    if r.header.start_tick != 0 {
        return Err(ReplayError::UnsupportedStart(r.header.start_tick));
    }
    let total = r.total_ticks();
    if let Some(&(t, _)) = r.hashes.iter().find(|&&(t, _)| t > total || t == 0) {
        return Err(ReplayError::CheckpointOutOfRange(t));
    }
    let mut world = World::new(r.header.seed);
    let mut cmd_i = 0;
    let mut seen = Vec::new();
    let mut final_hash = world.state_hash();
    let mut ran = 0u64;
    for &(frame, run) in &r.frames {
        for _ in 0..run {
            let mut input = TickInput {
                frame,
                commands: Vec::new(),
            };
            while cmd_i < r.commands.len() && r.commands[cmd_i].0 <= world.tick {
                if r.commands[cmd_i].0 == world.tick {
                    input.commands.push(r.commands[cmd_i].1.clone());
                }
                cmd_i += 1;
            }
            tick(&mut world, &input);
            ran += 1;
            if ran.is_multiple_of(CHECKPOINT_INTERVAL) || ran == total {
                final_hash = world.state_hash();
                check(ran, final_hash)?;
                seen.push((ran, final_hash));
            }
        }
    }
    Ok((
        ReplayReport {
            ticks: ran,
            final_hash,
        },
        seen,
    ))
}

/// Replays `r` against `content_hash`, failing on the first mismatching checkpoint.
pub fn run_replay(r: &Replay, content_hash: u64) -> Result<ReplayReport, ReplayError> {
    if r.header.content_hash != content_hash {
        return Err(ReplayError::ContentHashMismatch {
            expected: r.header.content_hash,
            actual: content_hash,
        });
    }
    let (report, _) = simulate(r, |t, actual| {
        match r.hashes.iter().find(|&&(ct, _)| ct == t) {
            Some(&(_, expected)) if expected != actual => Err(ReplayError::HashMismatch {
                tick: t,
                expected,
                actual,
            }),
            _ => Ok(()),
        }
    })?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(x: i8, z: i8) -> ActionFrame {
        ActionFrame {
            move_x: x,
            move_z: z,
            buttons: 0,
        }
    }

    #[test]
    fn compress_expand() {
        let frames = vec![f(1, 0), f(1, 0), f(0, 0), f(1, 0)];
        let runs = compress(&frames);
        assert_eq!(runs, vec![(f(1, 0), 2), (f(0, 0), 1), (f(1, 0), 1)]);
        assert_eq!(expand(&runs), frames);
        assert!(compress(&[]).is_empty());
    }

    #[test]
    fn record_then_run_is_ok() {
        let frames: Vec<_> = (0..95).map(|i| f((i % 7) as i8 * 10, 50)).collect();
        let r = Replay::record(5, "m0_flat", 9, &frames, vec![(3, Command::Rest)]);
        assert_eq!(r.hashes.last().unwrap().0, 95);
        assert_eq!(r.hashes.len(), 4);
        let rep = run_replay(&Replay::decode(&r.encode()).unwrap(), 9).unwrap();
        assert_eq!(rep.ticks, 95);
        assert_eq!(rep.final_hash, r.hashes.last().unwrap().1);
        assert!(matches!(
            run_replay(&r, 10),
            Err(ReplayError::ContentHashMismatch { .. })
        ));
    }
}
