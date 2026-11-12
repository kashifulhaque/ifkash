//! Runs every golden replay in `game/tests/replays/` against the current content hash.

use std::path::PathBuf;

use lf_content::Content;
use lf_core::replay::{Replay, ReplayError, expand, run_replay};

fn replays_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/replays")
}

fn content_hash() -> u64 {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    Content::load_dir(&data)
        .expect("content loads")
        .content_hash()
}

fn golden_files() -> Vec<PathBuf> {
    let mut v: Vec<_> = std::fs::read_dir(replays_dir())
        .expect("replays dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "lfr"))
        .collect();
    v.sort();
    v
}

#[test]
fn every_golden_replay_passes() {
    let files = golden_files();
    assert!(!files.is_empty(), "no .lfr files in game/tests/replays/");
    let hash = content_hash();
    for f in files {
        let replay = Replay::decode(&std::fs::read(&f).unwrap()).unwrap();
        match run_replay(&replay, hash) {
            Ok(_) => {}
            Err(ReplayError::HashMismatch {
                tick,
                expected,
                actual,
            }) => panic!(
                "{}: first mismatching tick {tick}: expected {expected:016x}, got {actual:016x}",
                f.display()
            ),
            Err(e) => panic!("{}: {e}", f.display()),
        }
    }
}

#[test]
fn embedded_and_directory_content_hashes_agree() {
    let embedded = Content::load_embedded()
        .expect("embedded content")
        .content_hash();
    assert_eq!(embedded, content_hash());
}

#[test]
fn changing_one_input_fails_at_the_next_checkpoint() {
    let path = replays_dir().join("m0_random_3000.lfr");
    let good = Replay::decode(&std::fs::read(&path).unwrap()).unwrap();
    let hash = content_hash();
    let mut frames = expand(&good.frames);
    for changed in [0usize, 29, 30, 1234, frames.len() - 1] {
        let original = frames[changed];
        frames[changed].move_x = if original.move_x > 0 { -100 } else { 100 };
        let mut bad = good.clone();
        bad.frames = lf_core::replay::compress(&frames);
        frames[changed] = original;
        // The change applies on tick `changed`; the first checkpoint that sees it is the
        // first multiple of 30 (or the last tick) greater than `changed`.
        let first = good
            .hashes
            .iter()
            .map(|&(t, _)| t)
            .find(|&t| t > changed as u64)
            .unwrap();
        match run_replay(&bad, hash) {
            Err(ReplayError::HashMismatch { tick, .. }) => {
                assert_eq!(tick, first, "changed frame {changed}")
            }
            other => panic!("expected HashMismatch for frame {changed}, got {other:?}"),
        }
    }
}
