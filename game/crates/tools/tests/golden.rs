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
    // The stick only steers a player that is on the ground, out of landing recovery,
    // alive, and not locked in a move; in the air it keeps its takeoff velocity. Record
    // which ticks it steers.
    let steerable: Vec<bool> = {
        use lf_core::input::TickInput;
        use lf_core::{World, tick};
        let mut w = World::new(good.header.seed, &content, &good.header.scenario).unwrap();
        frames
            .iter()
            .map(|&frame| {
                let p = w.actors.get(w.player).unwrap();
                let m = &p.motor;
                let free = m.grounded && m.landing == 0 && !p.moves.locks_movement();
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
    // Each target, moved to the nearest tick where the stick steers inside the same
    // checkpoint window (forward first, then back), so the expected checkpoint stays the
    // same. Attacks and dodges lock the stick for whole moves.
    for target in [0usize, 29, 30, 1234, last] {
        let window = target / 30 * 30..(target / 30 * 30 + 30).min(frames.len());
        let changed = (target..window.end)
            .find(|&i| steerable[i])
            .or_else(|| (window.start..target).rev().find(|&i| steerable[i]))
            .or_else(|| (target..frames.len()).find(|&i| steerable[i]))
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

/// The WP-021 golden: the three-hit light chain against the training dummy lands exactly
/// three hits, one per light, all on `dummy_ahead`.
#[test]
fn m2_three_hit_chain_lands_exactly_three_hits() {
    use lf_core::events::Event;
    use lf_core::input::TickInput;
    use lf_core::{World, tick};

    let content = content();
    let replay =
        Replay::decode(&std::fs::read(replays_dir().join("m2_three_hit_chain.lfr")).unwrap())
            .unwrap();
    assert_eq!(replay.header.scenario, "test_field");
    let mut w = World::new(replay.header.seed, &content, "test_field").unwrap();
    let dummy = w
        .actors
        .iter()
        .find(|(_, a)| a.pos == [0.0, 0.0, 6.0])
        .map(|(id, _)| id)
        .expect("dummy_ahead");
    let mut hits = Vec::new();
    for frame in expand(&replay.frames) {
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        for event in w.events.drain() {
            match event {
                Event::Hit(h) => hits.push(h),
                Event::Life(_) => {}
            }
        }
    }
    assert_eq!(hits.len(), 3, "{hits:?}");
    let chain: Vec<_> = ["ss_light_1", "ss_light_2", "ss_light_3"]
        .iter()
        .map(|id| w.moves.num_of(id).unwrap())
        .collect();
    for (h, &num) in hits.iter().zip(&chain) {
        assert_eq!((h.attacker, h.target, h.move_num), (w.player, dummy, num));
    }
    // WP-023: the padded dummy takes damage on every light and survives all three.
    assert!(hits.iter().all(|h| h.damage > 0 && !h.killed), "{hits:?}");
    let taken: u32 = hits.iter().map(|h| h.damage).sum();
    let d = w.actors.get(dummy).unwrap();
    assert_eq!((d.hp, d.dead), (100 - taken, false));
}

/// Replays a golden on `test_field` and returns the life-cycle events with their ticks,
/// and the final world.
fn life_events(name: &str) -> (Vec<(usize, lf_core::events::LifeEvent)>, lf_core::World) {
    use lf_core::events::Event;
    use lf_core::input::TickInput;
    use lf_core::{World, tick};

    let content = content();
    let replay = Replay::decode(&std::fs::read(replays_dir().join(name)).unwrap()).unwrap();
    assert_eq!(replay.header.scenario, "test_field");
    let mut w = World::new(replay.header.seed, &content, "test_field").unwrap();
    let mut out = Vec::new();
    for (t, frame) in expand(&replay.frames).into_iter().enumerate() {
        let commands = replay
            .commands
            .iter()
            .filter(|(ct, _)| *ct == t as u64)
            .map(|(_, c)| c.clone())
            .collect();
        tick(&mut w, &TickInput { frame, commands });
        for event in w.events.drain() {
            if let Event::Life(e) = event {
                out.push((t, e));
            }
        }
    }
    (out, w)
}

/// The WP-029 golden: rest at the Lantern, die by falling off the world, respawn at the
/// Lantern, and walk back to recover the Remnant with all 500 Glimmer.
#[test]
fn m2_death_recover_gets_the_remnant_back() {
    use lf_core::events::LifeEvent::*;

    let (events, w) = life_events("m2_death_recover.lfr");
    let kinds: Vec<_> = events.iter().map(|(_, e)| e).collect();
    assert!(
        matches!(
            kinds[..],
            [
                Rested { lantern: 0, .. },
                Died { .. },
                RemnantDropped { glimmer: 500, .. },
                Respawned { .. },
                RemnantRecovered { glimmer: 500, .. }
            ]
        ),
        "{events:?}"
    );
    let Respawned { pos } = events[3].1 else {
        unreachable!()
    };
    assert_eq!(pos, [0.0, 0.0, -3.0], "respawned at lantern_field");
    let (RemnantDropped { pos: at, .. }, RemnantRecovered { pos: back, .. }) =
        (&events[2].1, &events[4].1)
    else {
        unreachable!()
    };
    assert_eq!(at, back);
    let p = w.actors.get(w.player).unwrap();
    assert!(!p.dead && p.inventory.glimmer == 500);
    assert!(w.state.death.remnant.is_none() && w.state.death.deaths == 1);
    assert_eq!(w.state.respawn_lantern.as_deref(), Some("lantern_field"));
}

/// The WP-029 golden: die, respawn, die again without recovering, and the Remnant is lost.
#[test]
fn m2_death_lose_remnant_loses_the_glimmer() {
    use lf_core::events::LifeEvent::*;

    let (events, w) = life_events("m2_death_lose_remnant.lfr");
    let kinds: Vec<_> = events.iter().map(|(_, e)| e).collect();
    assert!(
        matches!(
            kinds[..],
            [
                Rested { .. },
                Died { .. },
                RemnantDropped { glimmer: 500, .. },
                Respawned { .. },
                Died { .. },
                RemnantLost { glimmer: 500, .. },
                Respawned { .. }
            ]
        ),
        "{events:?}"
    );
    let p = w.actors.get(w.player).unwrap();
    assert!(!p.dead);
    assert_eq!(p.inventory.glimmer, 0, "the Glimmer is gone for good");
    assert!(w.state.death.remnant.is_none() && w.state.death.deaths == 2);
}
