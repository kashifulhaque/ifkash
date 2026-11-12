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
            if let Event::Hit(h) = event {
                hits.push(h);
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

/// Runs a golden replay and returns the world at the end, with every event it emitted
/// and the player and the one other actor (the dummy) it ran against.
fn run_duel(name: &str, scenario: &str) -> (lf_core::World, Vec<lf_core::events::Event>) {
    use lf_core::input::TickInput;
    use lf_core::{World, tick};

    let content = content();
    let replay =
        Replay::decode(&std::fs::read(replays_dir().join(format!("{name}.lfr"))).unwrap()).unwrap();
    assert_eq!(replay.header.scenario, scenario);
    let mut w = World::new(replay.header.seed, &content, scenario).unwrap();
    let mut events = Vec::new();
    for frame in expand(&replay.frames) {
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        events.extend(w.events.drain());
    }
    (w, events)
}

/// The id of the scenario's only dummy.
fn only_dummy(w: &lf_core::World) -> lf_core::arena::Id {
    w.actors
        .iter()
        .find(|(id, _)| *id != w.player)
        .map(|(id, _)| id)
        .expect("a dummy")
}

/// The WP-024 parry golden: the duelist swings, the player parries it (no damage, the
/// duelist is parried), then a riposte lands for three times the attack rating.
#[test]
fn m2_parry_riposte_parries_then_ripostes() {
    use lf_core::events::{CritKind, Event};

    let (w, events) = run_duel("m2_parry_riposte", "duel_parry");
    let duelist = only_dummy(&w);
    let dagger_moves = |id: &str| w.moves.num_of(id).unwrap();
    // Order of events: the parry, then the critical, then the hit it deals.
    let kinds: Vec<&str> = events
        .iter()
        .map(|e| match e {
            Event::Parry(_) => "parry",
            Event::Critical(_) => "critical",
            Event::Hit(_) => "hit",
            Event::Stagger(_) => "stagger",
            Event::StanceBreak(_) => "stance_break",
            Event::Life(_) => "life",
            Event::Projectile(_) => "projectile",
            Event::StatusProc(_) => "status_proc",
            Event::Awareness(_) => "awareness",
        })
        .collect();
    assert_eq!(kinds, ["parry", "critical", "hit"], "{events:?}");
    let Event::Parry(p) = &events[0] else {
        unreachable!()
    };
    assert_eq!((p.attacker, p.target), (duelist, w.player));
    assert_eq!(p.move_num, dagger_moves("dummy_slash"));
    let Event::Critical(c) = &events[1] else {
        unreachable!()
    };
    assert_eq!(
        (c.attacker, c.target, c.kind),
        (w.player, duelist, CritKind::Riposte)
    );
    let Event::Hit(h) = &events[2] else {
        unreachable!()
    };
    assert_eq!((h.attacker, h.target), (w.player, duelist));
    assert_eq!(h.move_num, dagger_moves("crit_riposte"));
    // The parry cost the player nothing, and the riposte dwarfs a light's ~29 damage.
    let player = w.actors.get(w.player).unwrap();
    assert_eq!(player.hp, player.max_hp);
    assert!(h.damage > 150, "{}", h.damage);
}

/// The WP-024 stance golden: two charged heavies break the elite's stance, and nothing
/// else (no stagger) happens to it first.
#[test]
fn m2_stance_break_breaks_with_charged_heavies() {
    use lf_core::events::Event;

    let (w, events) = run_duel("m2_stance_break", "duel_stance");
    let elite = only_dummy(&w);
    let charged = w.moves.num_of("ss_heavy_charged").unwrap();
    let hits: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Hit(h) => Some(h),
            _ => None,
        })
        .collect();
    assert_eq!(hits.len(), 2, "{events:?}");
    assert!(
        hits.iter()
            .all(|h| h.target == elite && h.move_num == charged)
    );
    let breaks: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::StanceBreak(b) => Some(b),
            _ => None,
        })
        .collect();
    assert_eq!(breaks.len(), 1, "{events:?}");
    assert_eq!(breaks[0].target, elite);
    assert!(events.iter().all(|e| !matches!(e, Event::Stagger(_))));
    // The break came with the second hit, and the elite is still kneeling at the end.
    let at = |e: &Event| events.iter().position(|x| x == e).unwrap();
    assert!(at(&Event::StanceBreak(breaks[0].clone())) > at(&Event::Hit(hits[0].clone())));
    assert!(w.actors.get(elite).unwrap().poise.kneel > 0);
}

/// The WP-024 backstab golden: the sentry faces away, so LIGHT is a backstab, not a
/// light attack.
#[test]
fn m2_backstab_stabs_the_unaware() {
    use lf_core::events::{CritKind, Event};

    let (w, events) = run_duel("m2_backstab", "duel_backstab");
    let sentry = only_dummy(&w);
    let kinds: Vec<_> = events
        .iter()
        .map(|e| match e {
            Event::Critical(c) => {
                assert_eq!(
                    (c.attacker, c.target, c.kind),
                    (w.player, sentry, CritKind::Backstab)
                );
                "critical"
            }
            Event::Hit(h) => {
                assert_eq!(h.move_num, w.moves.num_of("crit_backstab").unwrap());
                assert!(h.damage > 150, "{}", h.damage);
                "hit"
            }
            other => panic!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(kinds, ["critical", "hit"]);
    assert!(
        !w.actors.get(sentry).unwrap().poise.unaware,
        "the stab alerts it"
    );
}

/// The WP-027 golden: an arrow that falls into the floor, an arrow at the locked dummy,
/// and a seeker bolt launched sideways that curves onto it.
#[test]
fn m2_projectiles_arrow_and_seeker_do_what_the_script_says() {
    use lf_core::events::{Event, ProjectileEventKind as K};
    use lf_core::input::TickInput;
    use lf_core::{World, tick};

    let content = content();
    let replay =
        Replay::decode(&std::fs::read(replays_dir().join("m2_projectiles.lfr")).unwrap()).unwrap();
    assert_eq!(replay.header.scenario, "m2_range");
    let mut w = World::new(replay.header.seed, &content, "m2_range").unwrap();
    let mark = w
        .actors
        .iter()
        .find(|(_, a)| a.pos == [3.0, 0.0, 10.0])
        .map(|(id, _)| id)
        .expect("dummy_mark");
    let aside = w
        .actors
        .iter()
        .find(|(_, a)| a.pos == [-5.0, 0.0, 8.0])
        .map(|(id, _)| id)
        .expect("dummy_aside");
    let (mut hits, mut life) = (Vec::new(), Vec::new());
    let mut first_bolt_vel = None;
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
                Event::Projectile(p) => life.push(p),
                _ => {}
            }
        }
        if first_bolt_vel.is_none() {
            first_bolt_vel = w
                .projectiles
                .iter()
                .find(|(_, p)| p.def == 5001)
                .map(|(_, p)| p.vel);
        }
    }
    let arrow = w.projectile_defs.num_of("proj_arrow").unwrap();
    let bolt = w.projectile_defs.num_of("proj_seeker_bolt").unwrap();
    let kinds: Vec<(K, u32)> = life.iter().map(|e| (e.kind, e.def)).collect();
    assert_eq!(
        kinds,
        [
            (K::Spawned, arrow),
            (K::Impact, arrow),
            (K::Spawned, arrow),
            (K::Struck, arrow),
            (K::Spawned, bolt),
            (K::Struck, bolt),
        ],
        "{life:?}"
    );
    // Arrow 1 fell into the floor about 20 m out, short of the 29.5 m wall.
    let floor = &life[1];
    assert!(floor.ended && floor.pos[1] < 0.1, "{floor:?}");
    assert!((19.0..20.5).contains(&floor.pos[2]), "{floor:?}");
    // Arrows and bolt struck the locked dummy and nothing else.
    assert!(life[3].ended && life[5].ended);
    assert_eq!((life[3].target, life[5].target), (Some(mark), Some(mark)));
    let shot = w.moves.num_of("stub_bow_shot").unwrap();
    let seeker = w.moves.num_of("stub_bow_seeker").unwrap();
    assert_eq!(hits.len(), 2, "{hits:?}");
    assert_eq!(
        (hits[0].move_num, hits[1].move_num),
        (shot, seeker),
        "{hits:?}"
    );
    for h in &hits {
        assert_eq!((h.attacker, h.target, h.hitbox), (w.player, mark, 255));
        assert!(h.damage > 0 && !h.killed);
    }
    let taken: u32 = hits.iter().map(|h| h.damage).sum();
    assert_eq!(w.actors.get(mark).unwrap().hp, 300 - taken);
    assert_eq!(w.actors.get(aside).unwrap().hp, 300);
    // The bolt left sideways (+X, 70 degrees off the facing) and still hit: it curved.
    let v = first_bolt_vel.expect("the bolt flew");
    assert!(v[0] > 10.0 && v[2] < 4.0, "{v:?}");
    assert!(w.projectiles.iter().next().is_none());
}

/// The WP-030 AI golden: three grunts notice the player (Idle to Aware to Combat), land
/// hits on it, then leash (Return) and end up Idle at home after the player sprints away.
#[test]
fn m3_ai_notice_grunts_notice_attack_and_leash() {
    use lf_core::events::Event;

    let (w, events) = run_duel("m3_ai_notice", "ai_yard");
    let trio: Vec<_> = w.state.ai_enemies[..3].to_vec();
    for &g in &trio {
        let path: Vec<u8> = events
            .iter()
            .filter_map(|e| match e {
                Event::Awareness(a) if a.actor == g => Some(a.to),
                _ => None,
            })
            .collect();
        // Aware, Combat, Return, Idle (codes 2, 3, 4, 0).
        assert_eq!(path, [2, 3, 4, 0], "{g:?}");
        let a = w.actors.get(g).unwrap();
        assert_eq!(a.hp, a.max_hp);
    }
    let hits = events
        .iter()
        .filter(
            |e| matches!(e, Event::Hit(h) if h.target == w.player && trio.contains(&h.attacker)),
        )
        .count();
    assert!(hits > 0, "{events:?}");
}
