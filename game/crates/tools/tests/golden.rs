//! Runs every golden replay in `game/tests/replays/` against the current content hash.

use std::path::PathBuf;

use lf_content::Content;
use lf_core::replay::{Replay, ReplayError, expand, run_replay};

fn replays_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/replays")
}

fn content() -> Content {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    Content::load_dir(&data).expect("content loads")
}

fn content_hash() -> u64 {
    content().content_hash()
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
    let content = content();
    for f in files {
        let replay = Replay::decode(&std::fs::read(&f).unwrap()).unwrap();
        match run_replay(&replay, &content) {
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
    let content = content();
    let mut frames = expand(&good.frames);
    // The stick only steers a player that is on the ground, out of landing recovery, and
    // alive; in the air it keeps its takeoff velocity. Record which ticks it steers.
    let steerable: Vec<bool> = {
        use lf_core::input::TickInput;
        use lf_core::{World, tick};
        let mut w = World::new(good.header.seed, &content, &good.header.scenario).unwrap();
        frames
            .iter()
            .map(|&frame| {
                let m = &w.actors.get(w.player).unwrap().motor;
                let free = m.grounded && m.landing == 0;
                tick(
                    &mut w,
                    &TickInput {
                        frame,
                        commands: Vec::new(),
                    },
                );
                free
            })
            .collect()
    };
    let last = frames.len() - 1;
    // Each target, moved forward to the first tick where the stick steers (no later than
    // its checkpoint, so the expected checkpoint stays the same).
    for target in [0usize, 29, 30, 1234, last] {
        let changed = (target..frames.len())
            .find(|&i| steerable[i])
            .expect("a steerable tick");
        assert!(
            changed / 30 == target / 30 || target == last,
            "target {target}"
        );
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
        match run_replay(&bad, &content) {
            Err(ReplayError::HashMismatch { tick, .. }) => {
                assert_eq!(tick, first, "changed frame {changed}")
            }
            other => panic!("expected HashMismatch for frame {changed}, got {other:?}"),
        }
    }
}

/// The WP-010 golden route really walks, sprints, jumps the ledge, and falls the drop,
/// and the 10 m fall costs 8% of max HP (docs/game/02-world.md, Fall damage).
#[test]
fn m1_movement_route_does_what_it_says() {
    use lf_core::input::TickInput;
    use lf_core::{World, tick};

    let content = content();
    let replay =
        Replay::decode(&std::fs::read(replays_dir().join("m1_movement.lfr")).unwrap()).unwrap();
    assert_eq!(replay.header.scenario, "test_field");
    let mut w = World::new(replay.header.seed, &content, "test_field").unwrap();
    let (mut sprinted, mut jumped, mut on_ledge, mut fell) = (false, false, false, false);
    for frame in expand(&replay.frames) {
        let before = w.actors.get(w.player).unwrap().clone();
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        let p = w.actors.get(w.player).unwrap();
        sprinted |= p.motor.sprinting;
        jumped |= before.motor.grounded && !p.motor.grounded && p.vel[1] > 0.0;
        on_ledge |= p.motor.grounded && (p.pos[1] - 10.0).abs() < 0.05 && p.pos[2] > 30.0;
        if !before.motor.grounded && p.motor.grounded && p.motor.last_fall_m > 9.0 {
            fell = true;
            assert!(
                (p.motor.last_fall_m - 10.0).abs() < 0.05,
                "{}",
                p.motor.last_fall_m
            );
            let expected = (p.max_hp as f32 * 0.08 + 0.5) as u32;
            assert_eq!(before.hp - p.hp, expected, "fall damage");
        }
    }
    let p = w.actors.get(w.player).unwrap();
    assert!(
        sprinted && jumped && on_ledge && fell,
        "{sprinted} {jumped} {on_ledge} {fell}"
    );
    assert!(
        p.motor.grounded && p.pos[1].abs() < 0.05 && !p.dead,
        "{:?}",
        p.pos
    );
}
