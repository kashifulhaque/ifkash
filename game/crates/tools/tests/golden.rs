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
            Event::Revive(_) => "revive",
            Event::Boss(_) => "boss",
            Event::Pickup(_) => "pickup",
            Event::Mechanic(_) => "mechanic",
            Event::Loot(_) => "loot",
            Event::Message(_) => "message",
            Event::Echo(_) => "echo",
            Event::Dialogue(_) => "dialogue",
            Event::Quest(_) => "quest",
            Event::Appearance(_) => "appearance",
            Event::Briar(_) => "briar",
            Event::Loom(_) => "loom",
            Event::Mark(_) => "mark",
            Event::Gesture(_) => "gesture",
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

// --- WP-032: enemy batch 1 ---

/// The `Hit` events of a replay run, in order.
fn hits(events: &[lf_core::events::Event]) -> Vec<&lf_core::events::Hit> {
    events
        .iter()
        .filter_map(|e| match e {
            lf_core::events::Event::Hit(h) => Some(h),
            _ => None,
        })
        .collect()
}

/// The WP-032 Husk golden: the husk walks up and lands both swings of its combo (62 and 79
/// damage from its attack rating of 90), then dies to the player's light.
#[test]
fn m3_husk_lands_its_two_hit_combo_and_dies() {
    let (w, events) = run_duel("m3_husk", "fight_husk");
    let husk = w.state.ai_enemies[0];
    let hs = hits(&events);
    let on_player: Vec<_> = hs
        .iter()
        .filter(|h| h.attacker == husk && h.target == w.player)
        .collect();
    let swipe = |id: &str| w.moves.num_of(id).unwrap();
    assert_eq!(on_player[0].move_num, swipe("husk_swipe_1"));
    assert_eq!(on_player[0].damage, 62);
    assert_eq!(on_player[1].move_num, swipe("husk_swipe_2"));
    assert_eq!(on_player[1].damage, 79);
    assert!(hs.iter().any(|h| h.target == husk && h.killed), "{hs:?}");
    assert!(w.actors.get(husk).unwrap().dead);
}

/// The Servant, Miner, and Frost Husk each land a swing; the Frost Husk's carries Rime.
#[test]
fn m3_husk_variants_each_swing() {
    let (w, events) = run_duel("m3_husk_variants", "fight_husk_variants");
    let hs = hits(&events);
    let by = |name: &str| {
        let i = w
            .layout
            .enemies
            .iter()
            .position(|s| s.name == name)
            .unwrap();
        w.state.ai_enemies[i]
    };
    let moves_by = |name: &str| -> Vec<u32> {
        hs.iter()
            .filter(|h| h.attacker == by(name) && h.target == w.player)
            .map(|h| h.move_num)
            .collect()
    };
    let m = |id: &str| w.moves.num_of(id).unwrap();
    assert!(moves_by("servant").contains(&m("husk_swipe_1")));
    assert!(moves_by("miner").contains(&m("husk_miner_pick")));
    assert!(moves_by("frost").contains(&m("husk_frost_swipe_1")));
    // The Frost Husk's swing built Rime on the player.
    let player = w.actors.get(w.player).unwrap();
    let rime = lf_core::combat::status::index(lf_content::moves::StatusKind::Rime);
    assert!(player.status.meter_milli[rime] > 0 || player.status.active[rime] > 0);
}

/// The pack attacks with lunges and snaps, never more than two at once, and dies to lights.
#[test]
fn m3_thorn_hounds_bite_and_die() {
    let (w, events) = run_duel("m3_thorn_hounds", "fight_thorn_hounds");
    let pack: Vec<_> = w.state.ai_enemies.clone();
    let hs = hits(&events);
    let on_player: Vec<_> = hs.iter().filter(|h| h.target == w.player).collect();
    assert!(on_player.len() >= 2, "{hs:?}");
    let attackers: std::collections::BTreeSet<_> = on_player.iter().map(|h| h.attacker).collect();
    assert!(attackers.len() >= 2, "more than one hound bit the player");
    assert!(on_player.iter().all(|h| pack.contains(&h.attacker)));
    let killed = hs
        .iter()
        .filter(|h| h.killed && pack.contains(&h.target))
        .count();
    assert!(killed >= 2, "{killed}");
}

/// The Gray Knight's guard takes most of the player's lights, then it attacks.
#[test]
fn m3_gray_knight_guards_then_cuts() {
    use lf_core::events::HitResult;
    let (w, events) = run_duel("m3_gray_knight", "fight_gray_knight");
    let knight = w.state.ai_enemies[0];
    let hs = hits(&events);
    let guarded: Vec<_> = hs
        .iter()
        .filter(|h| h.target == knight && h.result == HitResult::Guarded)
        .collect();
    assert!(!guarded.is_empty(), "{hs:?}");
    // A light does about 100 unguarded against the knight's defense; the guard takes 80%.
    assert!(guarded.iter().all(|h| h.damage < 30), "{guarded:?}");
    let cuts: Vec<_> = hs
        .iter()
        .filter(|h| h.attacker == knight && h.target == w.player)
        .collect();
    assert!(
        cuts.iter()
            .any(|h| h.move_num == w.moves.num_of("gk_slash_1").unwrap())
    );
    assert!(
        cuts.iter()
            .any(|h| h.move_num == w.moves.num_of("gk_slash_2").unwrap())
    );
}

/// The archer looses arrows about every 90 ticks, and each lands (81 damage).
#[test]
fn m3_gray_archer_shoots_on_a_cadence() {
    use lf_core::events::{Event, ProjectileEventKind};
    let (w, events) = run_duel("m3_gray_archer", "fight_gray_archer");
    let archer = w.state.ai_enemies[0];
    let spawns = events
        .iter()
        .filter(|e| matches!(e, Event::Projectile(p) if p.shooter == archer && p.kind == ProjectileEventKind::Spawned))
        .count();
    assert!(spawns >= 2, "{spawns}");
    let hs = hits(&events);
    let arrows: Vec<_> = hs.iter().filter(|h| h.attacker == archer).collect();
    assert!(arrows.len() >= 2);
    assert!(
        arrows
            .iter()
            .all(|h| h.damage == 81 && h.move_num == w.moves.num_of("ga_shoot").unwrap())
    );
}

/// The skeleton dies, lies still for 180 ticks, and revives at half HP.
#[test]
fn m3_whitewarden_revive_gets_back_up() {
    use lf_core::events::Event;
    let (w, events) = run_duel("m3_whitewarden_revive", "fight_skeleton");
    let sk = w.state.ai_enemies[0];
    let kill_tick = events
        .iter()
        .position(|e| matches!(e, Event::Hit(h) if h.target == sk && h.killed));
    let revive = events
        .iter()
        .position(|e| matches!(e, Event::Revive(r) if r.actor == sk));
    assert!(kill_tick.is_some() && revive.is_some(), "{events:?}");
    let Some(Event::Revive(r)) = revive.map(|i| &events[i]) else {
        unreachable!()
    };
    assert_eq!(r.hp, 90);
    assert!(!w.actors.get(sk).unwrap().dead);
}

// --- WP-037: enemy batch 3 and the elites ---

/// Runs a golden's frames tick by tick, calling `each` on the world after every tick, and
/// returns the world and all events.
fn run_each(
    name: &str,
    scenario: &str,
    mut each: impl FnMut(&lf_core::World),
) -> (lf_core::World, Vec<lf_core::events::Event>) {
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
        each(&w);
    }
    (w, events)
}

/// The moves the enemy `attacker` landed on the player, in order, by ID.
fn move_ids_landed(
    w: &lf_core::World,
    events: &[lf_core::events::Event],
    attacker: lf_core::arena::Id,
) -> Vec<String> {
    hits(events)
        .iter()
        .filter(|h| h.attacker == attacker && h.target == w.player)
        .map(|h| w.moves.get(h.move_num).unwrap().data.id.clone())
        .collect()
}

/// The thief lobs pitch bombs that land (62 damage each), and runs once the player rushes it.
#[test]
fn m3_lantern_thief_throws_bombs_and_flees() {
    use lf_core::events::{Event, ProjectileEventKind};
    let (w, events) = run_duel("m3_lantern_thief", "fight_lantern_thief");
    let thief = w.state.ai_enemies[0];
    let bombs = events
        .iter()
        .filter(|e| matches!(e, Event::Projectile(p) if p.shooter == thief && p.kind == ProjectileEventKind::Spawned))
        .count();
    assert!(bombs >= 3, "{bombs}");
    let landed: Vec<_> = hits(&events)
        .into_iter()
        .filter(|h| h.attacker == thief && h.target == w.player)
        .collect();
    assert!(landed.len() >= 3, "{landed:?}");
    assert!(
        landed
            .iter()
            .all(|h| h.move_num == w.moves.num_of("lt_bomb").unwrap() && h.damage == 62)
    );
    // It started 11 m out; the player rushed it and a light hit it, and it ran off.
    let t = w.actors.get(thief).unwrap();
    assert!(t.hp < t.max_hp, "the player's light landed");
    assert!(t.pos[2] > 12.0, "it fled: {:?}", t.pos);
    assert!(w.actors.get(w.player).unwrap().pos[2] > 5.0);
}

/// The pack circles one way and lunges one at a time: at no tick are two wolves in an
/// attack move, at no tick do two strafing wolves go opposite ways, and more than one wolf
/// takes a turn.
#[test]
fn m3_rime_wolves_circle_and_lunge_one_at_a_time() {
    use lf_content::enemy::Behavior;
    use lf_core::ai::Doing;
    let mut max_attacking = 0;
    let mut mixed_circling = false;
    let mut strafing_together = false;
    let mut max_rime = 0;
    let rime = lf_core::combat::status::index(lf_content::moves::StatusKind::Rime);
    let (w, events) = run_each("m3_rime_wolves", "fight_rime_wolves", |w| {
        let p = w.actors.get(w.player).unwrap();
        max_rime = max_rime.max(p.status.meter_milli[rime]);
        let brains: Vec<_> = w
            .state
            .ai_enemies
            .iter()
            .filter_map(|&id| {
                w.actors
                    .get(id)
                    .filter(|a| !a.dead)
                    .and_then(|a| a.ai.as_ref())
            })
            .collect();
        let attacking = brains
            .iter()
            .filter(|b| matches!(b.doing, Doing::Attack { .. }))
            .count();
        max_attacking = max_attacking.max(attacking);
        let strafing: Vec<bool> = brains
            .iter()
            .filter(|b| {
                matches!(b.doing, Doing::Behavior(Behavior::Strafe { .. })) && b.doing_ticks > 0
            })
            .map(|b| b.strafe_left)
            .collect();
        if strafing.len() >= 2 {
            strafing_together = true;
            mixed_circling |= strafing.iter().any(|&l| l != strafing[0]);
        }
    });
    assert_eq!(max_attacking, 1, "one wolf attacks at a time");
    assert!(strafing_together, "the wolves circled together");
    assert!(!mixed_circling, "they circled the same way");
    let wolves = w.state.ai_enemies.clone();
    let hs = hits(&events);
    let biters: std::collections::BTreeSet<_> = hs
        .iter()
        .filter(|h| h.target == w.player && wolves.contains(&h.attacker))
        .map(|h| h.attacker)
        .collect();
    assert!(biters.len() >= 2, "{hs:?}");
    // The bites build Rime.
    assert!(max_rime > 0, "{max_rime}");
}

/// The mimic is a chest for the first 150 ticks (no notice, no attack) while the player waits
/// 6 m away, wakes when the player walks within 2 m, and then fights.
#[test]
fn m3_mimic_stays_a_chest_until_approached() {
    use lf_core::ai::Awareness;
    let mut woke_at = None;
    let mut tick_n = 0u32;
    let mut dormant_ticks = 0u32;
    let (w, events) = run_each("m3_mimic", "fight_mimic", |w| {
        tick_n += 1;
        let m = w.state.ai_enemies[0];
        let b = w.actors.get(m).unwrap().ai.as_ref().unwrap();
        if b.awareness == Awareness::Idle {
            dormant_ticks += 1;
        } else if woke_at.is_none() {
            woke_at = Some(tick_n);
        }
    });
    let mimic = w.state.ai_enemies[0];
    let woke = woke_at.expect("it woke");
    assert!(woke > 150, "still a chest while the player waited: {woke}");
    assert_eq!(dormant_ticks, woke - 1);
    let landed = move_ids_landed(&w, &events, mimic);
    assert!(landed.contains(&"mc_chomp".to_owned()), "{landed:?}");
    assert!(w.actors.get(mimic).unwrap().hp < 900, "the player hit it");
}

/// The Bramble Brute lands its whole combo: both swings and the slam, in order.
#[test]
fn m3_bramble_brute_swings_swings_slams() {
    let (w, events) = run_duel("m3_bramble_brute", "fight_bramble_brute");
    let landed = move_ids_landed(&w, &events, w.state.ai_enemies[0]);
    assert_eq!(
        &landed[..3],
        ["bb_swing_1", "bb_swing_2", "bb_slam"],
        "{landed:?}"
    );
    assert!(!w.actors.get(w.player).unwrap().dead);
}

/// The Choir Knight's bell ring heavily staggers the player (a stun) for little damage, and
/// its halberd chain lands too.
#[test]
fn m3_choir_knight_rings_its_bell() {
    use lf_core::events::Event;
    let (w, events) = run_duel("m3_choir_knight", "fight_choir_knight");
    let knight = w.state.ai_enemies[0];
    let bell = w.moves.num_of("ck_bell").unwrap();
    let hs = hits(&events);
    let i = events
        .iter()
        .position(|e| matches!(e, Event::Hit(h) if h.attacker == knight && h.move_num == bell))
        .expect("the bell lands");
    assert!(
        matches!(&events[i + 1], Event::Stagger(s) if s.target == w.player && s.heavy),
        "{:?}",
        &events[i..]
    );
    let landed = move_ids_landed(&w, &events, knight);
    assert!(landed.contains(&"ck_halberd_1".to_owned()), "{landed:?}");
    // The bell lifts for 36 frames before the ring (the telegraph).
    let d = &w.moves.get(bell).unwrap().data;
    assert!(d.hitboxes[0].frames.0 >= 36);
    assert!(hs.iter().any(|h| h.move_num == bell && h.damage < 60));
}

/// The Captain's swings fall late (34 frames or more after they start) and every one can be
/// parried; the golden lands a delayed slash and overhead.
#[test]
fn m3_gray_captain_duels_with_delayed_swings() {
    use lf_content::moves::HitTag;
    let (w, events) = run_duel("m3_gray_captain", "fight_gray_captain");
    let captain = w.state.ai_enemies[0];
    let landed = move_ids_landed(&w, &events, captain);
    assert!(landed.len() >= 2, "{landed:?}");
    assert!(landed.iter().all(|m| m.starts_with("gc_")));
    for id in ["gc_slash", "gc_overhead"] {
        let d = &w.moves.get(w.moves.num_of(id).unwrap()).unwrap().data;
        assert!(d.hitboxes[0].frames.0 >= 34, "{id}");
        assert!(
            d.hitboxes
                .iter()
                .all(|h| !h.tags.contains(&HitTag::Unparryable))
        );
    }
    assert!(!w.actors.get(w.player).unwrap().dead);
}

/// The Smith's hammer slams the player (217 damage and a stagger), and leaves a Fire
/// hitbox up in front of it for about 3 seconds as the burning metal.
#[test]
fn m3_foundry_smith_slams_and_leaves_burning_metal() {
    use lf_content::moves::Element;
    let (w, events) = run_duel("m3_foundry_smith", "fight_foundry_smith");
    let smith = w.state.ai_enemies[0];
    let hammer = w.moves.num_of("fs_hammer").unwrap();
    let slams: Vec<_> = hits(&events)
        .into_iter()
        .filter(|h| h.attacker == smith && h.move_num == hammer && h.target == w.player)
        .collect();
    assert!(!slams.is_empty());
    assert!(slams.iter().all(|h| h.hitbox == 0 && h.damage == 217));
    let d = &w.moves.get(hammer).unwrap().data;
    let fire = &d.hitboxes[1];
    assert_eq!(fire.element, Some(Element::Fire));
    assert!(fire.frames.1 - fire.frames.0 >= 85, "{:?}", fire.frames);
    assert!(!w.actors.get(w.player).unwrap().dead);
}

/// The Bell Warden's chain is strike, strike, then the stun wave, over and over.
#[test]
fn m3_bell_warden_strikes_twice_then_sends_a_wave() {
    use lf_core::events::Event;
    let (w, events) = run_duel("m3_bell_warden", "fight_bell_warden");
    let warden = w.state.ai_enemies[0];
    let landed = move_ids_landed(&w, &events, warden);
    assert_eq!(
        &landed[..3],
        ["bw_strike_1", "bw_strike_2", "bw_wave"],
        "{landed:?}"
    );
    let wave = w.moves.num_of("bw_wave").unwrap();
    let i = events
        .iter()
        .position(|e| matches!(e, Event::Hit(h) if h.attacker == warden && h.move_num == wave))
        .unwrap();
    assert!(matches!(&events[i + 1], Event::Stagger(s) if s.target == w.player && s.heavy));
    assert!(!w.actors.get(w.player).unwrap().dead);
}
