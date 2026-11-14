//! Tests for the Lattice Spire (WP-067): the Spire Gate opens for any two Great Sigils and keeps
//! them, the outer spiral's three gaps are jumps of 2.5 m with a fallback ladder each, the walks
//! between Lanterns are under 45 seconds, and a scripted bot walks from L22 to B12's fog gate. The
//! golden replay `m6_spire` is that walk.
//!
//! The nav grid has no jump or ladder links, so the walk is a chain of nav paths and three scripted
//! hops (the gaps).
//!
//! Record the golden: `cargo test -p lf-core record_the_spire_golden -- --ignored`, then
//! `cargo run -p lf-tools -- replay record-script tests/replays/m6_spire.script --out
//! tests/replays/m6_spire.lfr` (from `game/`).

use lf_content::Content;

use crate::ai::Awareness;
use crate::events::{Event, MechanicEvent};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::math::dmath::{cos, sin, sqrt};
use crate::nav::Search;
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

const SCENARIO: &str = "spire";
const SEED: u64 = 67;
/// The longest run, in ticks (the walk takes about 4,000 with its fights; this only stops a
/// stuck run).
const MAX_TICKS: u64 = 20_000;
/// Acceptance: no walk between Lanterns takes longer than this at run speed, in seconds.
const MAX_WALK_S: f32 = 45.0;
/// A path request's node budget.
const BUDGET: u32 = 2_000_000;
const SIGILS: [&str; 4] = [
    "itm_great_sigil_choir",
    "itm_great_sigil_castellan",
    "itm_great_sigil_matriarch",
    "itm_great_sigil_bellwright",
];

fn world() -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(SEED, &content, SCENARIO).expect("spire exists")
}

fn lantern_pos(w: &World, id: &str) -> [f32; 3] {
    w.layout
        .lanterns
        .iter()
        .find(|l| l.id == id)
        .unwrap_or_else(|| panic!("no Lantern {id}"))
        .pos
}

fn walk_path(w: &World, from: [f32; 3], to: [f32; 3]) -> Vec<[f32; 3]> {
    w.nav
        .as_ref()
        .expect("the Spire has enemies, so it has a nav grid")
        .find_path(&mut Search::new(), from, to, BUDGET)
        .unwrap_or_else(|e| panic!("no path from {from:?} to {to:?}: {e:?}"))
        .waypoints
}

fn length(path: &[[f32; 3]]) -> f32 {
    path.windows(2)
        .map(|s| {
            let (dx, dz) = (s[1][0] - s[0][0], s[1][2] - s[0][2]);
            sqrt(dx * dx + dz * dz)
        })
        .sum()
}

fn flat(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
    sqrt(dx * dx + dz * dz)
}

/// The three gaps of the outer spiral: where to take off (0.2 m short of the near edge) and where
/// to land (0.8 m past the far edge), on the ring's centerline, with the landing's floor height.
const HOPS: [([f32; 3], [f32; 3]); 3] = [
    ([-12.0, 2.17, 0.55], [-12.0, 2.27, 4.05]),
    ([-3.03, 3.6, 12.0], [0.47, 3.7, 12.0]),
    ([12.0, 6.36, -7.72], [12.0, 6.46, -11.22]),
];
/// The gap's width, as the fallback ladders and the layout notes give it, in meters.
const GAP_M: f32 = 2.5;

/// The Gate's front: where the player stands to open it, and the forecourt past it.
const GATE_FRONT: [f32; 3] = [2.0, 0.0, -16.2];
const FORECOURT: [f32; 3] = [2.0, 0.0, -12.0];
/// The top of the last ring piece, in front of F1's south door.
const RING_END: [f32; 3] = [8.0, 6.9, -11.8];
/// The fog gate's plane (z = -5.25, x 5..8) and a point on each side of it.
const FOG_GATE: [f32; 3] = [6.5, 22.0, -5.25];
const FOG_SOUTH: [f32; 3] = [6.5, 22.0, -7.0];
const FOG_NORTH: [f32; 3] = [6.5, 22.0, -3.6];

/// The walk from L22 to L23: the gate, the ring's three hops, and the three floors.
/// Each leg's walking time is `length / run_speed`; a hop is its straight-line distance.
fn l22_to_l23_seconds(w: &World) -> f32 {
    let l22 = lantern_pos(w, "lantern_l22");
    let l23 = lantern_pos(w, "lantern_l23");
    let mut at = l22;
    let mut meters = 0.0;
    for p in [GATE_FRONT, FORECOURT] {
        meters += length(&walk_path(w, at, p));
        at = p;
    }
    for (from, to) in HOPS {
        meters += length(&walk_path(w, at, from));
        meters += flat(from, to);
        at = to;
    }
    meters += length(&walk_path(w, at, RING_END));
    meters += length(&walk_path(w, RING_END, l23));
    meters / w.locomotion.run_speed
}

/// Every walk between Lanterns is under 45 seconds at run speed: L22 to L23 through the gate, the
/// outer spiral, and the three gauntlet floors; and L23 to B12's fog gate is under 20.
#[test]
fn every_walk_between_lanterns_is_under_45_seconds() {
    let w = world();
    let s = l22_to_l23_seconds(&w);
    println!("lantern_l22 -> lantern_l23: {s:.1} s");
    assert!(s < MAX_WALK_S, "L22 to L23 takes {s:.1} s");
    assert!(s > 15.0, "L22 to L23 is only {s:.1} s: too close");
    let l23 = lantern_pos(&w, "lantern_l23");
    let gate = length(&walk_path(&w, l23, FOG_SOUTH)) + flat(FOG_SOUTH, FOG_GATE);
    let g = gate / w.locomotion.run_speed;
    println!("lantern_l23 -> the fog gate: {g:.1} s");
    assert!(g < 20.0, "L23 to the fog gate takes {g:.1} s");
    // The spawn is by L22 (the golden begins there); the gate is a few seconds on.
    let l22 = lantern_pos(&w, "lantern_l22");
    assert!(flat(w.layout.spawn, l22) < 6.0, "the spawn is not at L22");
}

/// Presses INTERACT for one tick and lets the world settle for a few more.
fn press_interact(w: &mut World) -> Vec<Event> {
    let mut events = Vec::new();
    for i in 0..10 {
        let input = TickInput {
            frame: ActionFrame {
                buttons: if i == 0 { buttons::INTERACT } else { 0 },
                ..ActionFrame::default()
            },
            commands: Vec::new(),
        };
        tick(w, &input);
        events.extend(w.events.drain());
    }
    events
}

fn gate_open(w: &World) -> bool {
    w.state.flags.is_set(&flags::door_open("spire_gate"))
}

fn held(w: &World, id: &str) -> u16 {
    let num = w.items.num_of(id).expect("a Sigil is an item");
    w.actors.get(w.player).unwrap().inventory.count(num)
}

/// A fresh world at the gate's front with exactly the Sigils in `have` (by index), and no others.
fn at_the_gate(have: &[usize]) -> World {
    let mut w = world();
    let player = w.player;
    for (i, id) in SIGILS.iter().enumerate() {
        let num = w.items.num_of(id).unwrap();
        let p = w.actors.get_mut(player).unwrap();
        let n = p.inventory.count(num);
        if n > 0 {
            p.inventory.remove(num, n);
        }
        if have.contains(&i) {
            p.inventory.pick_up(num, 1, &w.items);
        }
    }
    w.actors.get_mut(player).unwrap().pos = GATE_FRONT;
    for _ in 0..20 {
        tick(&mut w, &TickInput::default());
        w.events.drain();
    }
    w
}

/// The Spire Gate opens for any two different Great Sigils, not for none or one, and it takes
/// nothing. The scenario starts with two (a temporary grant, see `levels/spire/area.ron`).
#[test]
#[allow(clippy::needless_range_loop)]
fn the_spire_gate_opens_for_any_two_great_sigils() {
    let w = world();
    assert_eq!(
        SIGILS.iter().filter(|s| held(&w, s) > 0).count(),
        2,
        "the scenario grants two Sigils"
    );
    assert!(!gate_open(&w));
    for have in [vec![], vec![0], vec![3]] {
        let mut w = at_the_gate(&have);
        let events = press_interact(&mut w);
        assert!(!gate_open(&w), "the gate opened for {have:?}");
        assert!(
            events.iter().any(|e| matches!(
                e,
                Event::Mechanic(MechanicEvent {
                    kind: MechanicEvent::DOOR_NEEDS_KEY,
                    ..
                })
            )),
            "no 'needs a key' event for {have:?}"
        );
    }
    for a in 0..4 {
        for b in (a + 1)..4 {
            let mut w = at_the_gate(&[a, b]);
            press_interact(&mut w);
            assert!(gate_open(&w), "the gate stayed shut for Sigils {a} and {b}");
            assert_eq!(held(&w, SIGILS[a]), 1, "the gate took Sigil {a}");
            assert_eq!(held(&w, SIGILS[b]), 1, "the gate took Sigil {b}");
        }
    }
    // Three or four open it too.
    let mut w = at_the_gate(&[0, 1, 2, 3]);
    press_interact(&mut w);
    assert!(gate_open(&w));
}

/// The open gate lets the player through to the forecourt, and the shut one is a wall.
#[test]
fn the_gate_is_a_wall_until_it_opens() {
    let drive = |w: &mut World, ticks: u32| {
        for _ in 0..ticks {
            let input = TickInput {
                frame: ActionFrame {
                    move_x: 0,
                    move_z: 127,
                    buttons: 0,
                },
                commands: Vec::new(),
            };
            tick(w, &input);
            w.events.drain();
        }
        w.actors.get(w.player).unwrap().pos
    };
    let mut shut = at_the_gate(&[0, 1]);
    let p = drive(&mut shut, 120);
    assert!(p[2] < -14.0, "the shut gate let the player through: {p:?}");
    let mut open = at_the_gate(&[0, 1]);
    press_interact(&mut open);
    assert!(gate_open(&open));
    let p = drive(&mut open, 120);
    assert!(
        p[2] > -13.0,
        "the open gate did not let the player in: {p:?}"
    );
}

/// Each floor's hall of enemies is shut off from the stair strip by a one-way door: it opens from
/// the strip side with INTERACT, and not from inside the hall.
#[test]
fn the_halls_open_from_the_stair_strips_only() {
    // (door id, a point on the strip side, a point on the hall side, floor y)
    for (id, strip, hall, y) in [
        ("hall_door_f1", [6.5, 7.0], [3.0, 7.0], 7.0),
        ("hall_door_f2", [-6.5, 6.5], [-7.5, 3.5], 12.0),
        ("hall_door_f3", [-5.0, 0.5], [-1.5, 0.5], 17.0),
    ] {
        let from = |at: [f32; 2]| {
            let mut w = world();
            let player = w.player;
            w.actors.get_mut(player).unwrap().pos = [at[0], y, at[1]];
            for _ in 0..20 {
                tick(&mut w, &TickInput::default());
                w.events.drain();
            }
            let events = press_interact(&mut w);
            (w.state.flags.is_set(&flags::door_open(id)), events)
        };
        assert!(from(strip).0, "{id} stayed shut from the strip");
        let (open, events) = from(hall);
        assert!(!open, "{id} opened from inside the hall");
        assert!(
            events.iter().any(|e| matches!(
                e,
                Event::Mechanic(MechanicEvent {
                    kind: MechanicEvent::DOOR_ONE_WAY,
                    ..
                })
            )),
            "{id}: no 'opens from the other side' event"
        );
    }
}

/// The spiral has three 2.5 m gaps, each with a ladder on its far face that climbs out of the pit,
/// and every ladder's foot is below the ring at its gap.
#[test]
fn each_gap_has_a_fallback_ladder() {
    let w = world();
    assert_eq!(w.layout.ladders.len(), HOPS.len(), "one ladder per gap");
    for (from, to) in HOPS {
        let gap = flat(from, to) - 1.0;
        assert!(
            (gap - GAP_M - 0.0).abs() < 0.35,
            "the gap between {from:?} and {to:?} is {gap:.2} m wide, not {GAP_M}"
        );
        assert!(gap <= GAP_M + 0.35, "a gap over 2.5 m on the critical path");
        // A ladder stands in this gap: its foot is within the pit, short of the landing.
        let ladder = w
            .layout
            .ladders
            .iter()
            .find(|l| flat([l.pos[0], 0.0, l.pos[2]], to) < 1.6)
            .unwrap_or_else(|| panic!("no ladder at the gap landing at {to:?}"));
        assert!(
            ladder.pos[1] < to[1] - 2.0,
            "the ladder's foot ({:?}) is not in a pit under {to:?}",
            ladder.pos
        );
        let top = ladder.pos[1] + ladder.height_m;
        assert!(
            (top - to[1]).abs() < 0.25,
            "the ladder tops out at {top:.2}, the ring is at {:.2}",
            to[1]
        );
    }
}

/// Climbs out of a gap: the player falls (or walks) into the pit, grabs the ladder at its foot,
/// and ends on the ring beyond the gap.
#[test]
fn a_fallback_ladder_climbs_out_of_the_pit() {
    let mut w = world();
    for (_, to) in HOPS {
        let l = w
            .layout
            .ladders
            .iter()
            .find(|l| flat([l.pos[0], 0.0, l.pos[2]], to) < 1.6)
            .unwrap()
            .clone();
        let player = w.player;
        {
            let p = w.actors.get_mut(player).unwrap();
            p.pos = [l.pos[0], l.pos[1], l.pos[2]];
            p.vel = [0.0; 3];
        }
        for _ in 0..30 {
            tick(&mut w, &TickInput::default());
            w.events.drain();
        }
        let events = press_interact(&mut w);
        assert!(
            events.iter().any(|e| matches!(
                e,
                Event::Mechanic(MechanicEvent {
                    kind: MechanicEvent::LADDER_GRABBED,
                    ..
                })
            )),
            "no ladder grab at {:?}",
            l.pos
        );
        let (fx, fz) = (sin(l.yaw), cos(l.yaw));
        for _ in 0..1_200 {
            let input = TickInput {
                frame: ActionFrame {
                    move_x: (fx * 127.0) as i8,
                    move_z: (fz * 127.0) as i8,
                    buttons: 0,
                },
                commands: Vec::new(),
            };
            tick(&mut w, &input);
            w.events.drain();
            if w.mech.climb.is_none() {
                break;
            }
        }
        for _ in 0..30 {
            tick(&mut w, &TickInput::default());
            w.events.drain();
        }
        let p = w.actors.get(w.player).unwrap().pos;
        assert!(
            (p[1] - to[1]).abs() < 0.4,
            "the ladder at {:?} left the player at {p:?}, not on the ring ({to:?})",
            l.pos
        );
    }
}

/// One thing the scripted player does.
#[derive(Clone, Copy, Debug)]
enum Act {
    /// Walk to (x, z) at a run.
    Goto(f32, f32),
    /// Walk to (x, z) at a walk (a quiet step: noise 2 m instead of 6).
    Sneak(f32, f32),
    /// Press INTERACT.
    Interact,
    /// Rest at the Lantern in reach.
    Rest,
    /// Stand still for this many ticks.
    Wait(u16),
    /// Run and jump toward (x, z); done when the player is down within a meter of it.
    Hop(f32, f32),
}

/// Walks the nav path from `at` to `to`, leaving off the first point.
fn goto(w: &World, acts: &mut Vec<Act>, at: [f32; 3], to: [f32; 3]) {
    for p in &walk_path(w, at, to)[1..] {
        acts.push(Act::Goto(p[0], p[2]));
    }
}

/// `goto`, at a walk.
fn sneak(w: &World, acts: &mut Vec<Act>, at: [f32; 3], to: [f32; 3]) {
    for p in &walk_path(w, at, to)[1..] {
        acts.push(Act::Sneak(p[0], p[2]));
    }
}

/// The whole route from the spawn: rest at L22, open the gate, the three hops, the floors, rest at
/// L23, and walk up to the fog gate.
fn route(w: &World) -> Vec<Act> {
    let l22 = lantern_pos(w, "lantern_l22");
    let l23 = lantern_pos(w, "lantern_l23");
    let mut acts = Vec::new();
    // L22: stop a meter short of it and rest.
    let near = [l22[0], 0.0, l22[2] - 1.0];
    goto(w, &mut acts, w.layout.spawn, near);
    acts.push(Act::Rest);
    acts.push(Act::Wait(10));
    // The Gate.
    goto(w, &mut acts, near, GATE_FRONT);
    acts.push(Act::Interact);
    acts.push(Act::Wait(40));
    let mut at = GATE_FRONT;
    goto(w, &mut acts, at, FORECOURT);
    at = FORECOURT;
    for (from, to) in HOPS {
        goto(w, &mut acts, at, from);
        acts.push(Act::Hop(to[0], to[2]));
        at = to;
    }
    goto(w, &mut acts, at, RING_END);
    // F1's door, the three floors and their stairs, and L23: one nav path, at a walk so that the
    // halls' enemies (6 m of hearing at a run, 2 m at a walk) stay quiet.
    let beside = [l23[0] + 1.0, l23[1], l23[2]];
    sneak(w, &mut acts, RING_END, beside);
    acts.push(Act::Rest);
    acts.push(Act::Wait(10));
    sneak(w, &mut acts, beside, FOG_SOUTH);
    // The fog gate's plane, then one step past it: B12's gate seals behind the player, and the
    // walk ends there (WP-080).
    acts.push(Act::Sneak(FOG_NORTH[0], FOG_NORTH[2]));
    acts
}

/// What a run produced.
struct Run {
    frames: Vec<ActionFrame>,
    commands: Vec<(u64, Command)>,
    world: World,
    /// The act the run stopped in, and how many there are.
    stopped_in: usize,
    acts: usize,
    /// The worst awareness any enemy reached.
    worst: Awareness,
}

fn walk() -> Run {
    let mut w = world();
    let acts = route(&w);
    let (mut frames, mut commands) = (Vec::new(), Vec::new());
    let mut worst = Awareness::Idle;
    let mut stopped_in = acts.len();
    let mut leg = 0;
    let mut wait = 0u16;
    let mut jump_t = 0u32;
    let mut last_buttons = 0u16;
    while w.tick < MAX_TICKS && leg < acts.len() {
        let p = w.actors.get(w.player).unwrap();
        let (pos, grounded) = (p.pos, p.motor.grounded);
        let mut frame = ActionFrame::default();
        let mut cmds = Vec::new();
        match acts[leg] {
            Act::Goto(x, z) | Act::Sneak(x, z) => {
                let (dx, dz) = (x - pos[0], z - pos[2]);
                let d = sqrt(dx * dx + dz * dz);
                if d < 0.3 {
                    leg += 1;
                    continue;
                }
                let quiet = matches!(acts[leg], Act::Sneak(..));
                let mag = if quiet {
                    39.0
                } else if d < 1.0 {
                    50.0
                } else {
                    127.0
                };
                frame.move_x = (dx / d * mag) as i8;
                frame.move_z = (dz / d * mag) as i8;
            }
            Act::Interact => {
                frame.buttons = buttons::INTERACT;
                leg += 1;
            }
            Act::Rest => {
                cmds.push(Command::Rest);
                leg += 1;
            }
            Act::Wait(n) => {
                if wait == 0 {
                    wait = n;
                }
                wait -= 1;
                if wait == 0 {
                    leg += 1;
                }
            }
            Act::Hop(x, z) => {
                let (dx, dz) = (x - pos[0], z - pos[2]);
                let d = sqrt(dx * dx + dz * dz).max(0.001);
                if jump_t > 6 && grounded && d < 1.0 {
                    jump_t = 0;
                    leg += 1;
                    continue;
                }
                if jump_t > 150 {
                    break;
                }
                frame.move_x = (dx / d * 127.0) as i8;
                frame.move_z = (dz / d * 127.0) as i8;
                if jump_t == 0 {
                    frame.buttons = buttons::JUMP;
                }
                jump_t += 1;
            }
        }
        // At a boss's fog gate, Interact passes through it, as a player would (issue #82).
        if crate::ai::boss::walking_into_fog(&w, frame.move_x, frame.move_z) {
            frame.buttons |= buttons::INTERACT;
        }
        // Every button is a press edge.
        frame.buttons &= !last_buttons;
        last_buttons = frame.buttons;
        let input = TickInput {
            frame,
            commands: cmds,
        };
        frames.push(input.frame);
        for c in &input.commands {
            commands.push((w.tick, c.clone()));
        }
        tick(&mut w, &input);
        w.events.drain();
        // Through B12's fog gate: it sealed, and the walk is over (WP-080).
        if w.bosses
            .first()
            .is_some_and(|b| b.stage != crate::ai::boss::Stage::Dormant)
        {
            leg = acts.len();
        }
        for id in &w.state.ai_enemies {
            if let Some(b) = w.actors.get(*id).and_then(|a| a.ai.as_ref())
                && b.awareness.code() > worst.code()
            {
                worst = b.awareness;
            }
        }
        if w.actors.get(w.player).is_some_and(|p| p.dead) {
            break;
        }
        stopped_in = leg;
    }
    let stopped_in = if leg >= acts.len() {
        acts.len()
    } else {
        stopped_in
    };
    Run {
        frames,
        commands,
        world: w,
        stopped_in,
        acts: acts.len(),
        worst,
    }
}

/// The scripted player walks from L22 through the Gate, up the spiral, through the three floors to
/// L23, and on to B12's fog gate, without a single enemy noticing it (every enemy is in a hall that
/// the stair strips are walled off from, and hearing is checked with the real perception code).
#[test]
fn the_route_reaches_the_fog_gate() {
    let r = walk();
    let w = &r.world;
    let p = w.actors.get(w.player).unwrap();
    assert!(
        !p.dead,
        "the player died at tick {} near {:?}, in act {} of {}",
        w.tick, p.pos, r.stopped_in, r.acts
    );
    for id in ["lantern_l22", "lantern_l23"] {
        assert!(
            w.state.flags.is_set(&flags::lantern_lit(id)),
            "{id} was not lit (act {} of {}, tick {}, at {:?})",
            r.stopped_in,
            r.acts,
            w.tick,
            p.pos
        );
    }
    assert!(gate_open(w), "the Spire Gate never opened");
    assert!(
        (p.pos[1] - 22.0).abs() < 0.5 && p.pos[2] > -5.25 && p.pos[2] < -3.0,
        "the player ended at {:?}, not past B12's fog gate (act {} of {})",
        p.pos,
        r.stopped_in,
        r.acts
    );
    // B12's gate sealed behind the player (WP-080).
    assert!(
        matches!(w.bosses[0].stage, crate::ai::boss::Stage::Intro { .. }),
        "{:?}",
        w.bosses[0].stage
    );
    assert!(
        r.worst.code() < Awareness::Aware.code(),
        "an enemy noticed the player ({:?})",
        r.worst
    );
    // Under five minutes of game time.
    assert!(w.tick < 18_000, "took {} ticks", w.tick);
}

/// Every elite and enemy the spec puts in the Spire is placed, the materials and the last
/// Chronicle Page are in the level, and B12's Echo Stone is inside the arena.
#[test]
fn the_spire_has_its_floors_items_and_echo_stone() {
    let content = Content::load_embedded().unwrap();
    let level = content.scenario(SCENARIO).unwrap();
    // One floor per pair of regions, by height: the elites of each.
    let on_floor = |lo: f32, hi: f32| -> Vec<&str> {
        level
            .enemies
            .iter()
            .filter(|e| e.pos[1] > lo && e.pos[1] < hi)
            .map(|e| e.enemy.as_str())
            .collect()
    };
    let (f1, f2, f3) = (
        on_floor(6.0, 8.0),
        on_floor(11.0, 13.0),
        on_floor(16.0, 18.0),
    );
    for (floor, kinds) in [
        (&f1, ["bramble_brute", "choir_knight"].as_slice()),
        (
            &f2,
            ["gray_captain", "gray_knight", "gray_archer"].as_slice(),
        ),
        (
            &f3,
            ["foundry_smith", "bell_warden", "rime_wolf"].as_slice(),
        ),
    ] {
        for k in kinds {
            assert!(floor.contains(k), "no {k} on its floor: {floor:?}");
        }
    }
    assert!(level.enemies.len() < 40, "{} enemies", level.enemies.len());
    let items: Vec<&str> = level.pickups.iter().map(|p| p.item.as_str()).collect();
    for item in ["itm_flask_dew", "itm_starglass", "chronicle_page_5"] {
        assert!(items.contains(&item), "no {item} in the Spire");
    }
    let loot: Vec<&str> = level
        .chests
        .iter()
        .filter_map(|c| c.loot.item.as_deref())
        .collect();
    for item in ["itm_forgeglass_chunk", "wpn_spire_longbow"] {
        assert!(loot.contains(&item), "no chest holds {item}");
    }
    assert!(
        level
            .pickups
            .iter()
            .any(|p| p.codex.as_deref() == Some("work.wand-ai")),
        "the Page does not unlock the current role"
    );
    // B12: an Echo Stone inside the arena, and no boss yet.
    assert_eq!(level.echo_stones.len(), 1);
    let stone = &level.echo_stones[0];
    assert_eq!(stone.id, "stone_b12");
    assert!(
        (stone.pos[1] - 22.0).abs() < 0.01 && stone.pos[2] > -5.0 && stone.pos[0].abs() < 18.0,
        "the stone is outside the arena: {:?}",
        stone.pos
    );
    // B12 (WP-080) stands on the roof, its gate across the corridor's opening.
    assert_eq!(level.bosses.len(), 1);
    assert_eq!(level.bosses[0].boss, "b12_the_unspooled");
    assert_eq!(level.bosses[0].gate.pos, FOG_GATE);
    let w = world();
    assert_eq!(w.echo_table.stones.len(), 1);
    // Two Lanterns, L22 outside the gate and L23 on the third floor.
    let (l22, l23) = (
        lantern_pos(&w, "lantern_l22"),
        lantern_pos(&w, "lantern_l23"),
    );
    assert!(l22[2] < -14.0, "L22 is not outside the gate");
    assert!((l23[1] - 17.0).abs() < 0.01, "L23 is not on floor 3");
}

/// Writes `game/tests/replays/m6_spire.script`. See the module comment.
#[test]
#[ignore = "writes game/tests/replays/m6_spire.script"]
fn record_the_spire_golden() {
    let r = walk();
    assert!(
        !r.world.actors.get(r.world.player).unwrap().dead,
        "the player died"
    );
    assert_eq!(r.stopped_in, r.acts, "the route did not finish");
    let mut text = String::from(
        "# WP-067 golden: from L22 Spire Foot through the Spire Gate (two Great Sigils), the\n\
         # outer spiral's three jumps, the three gauntlet floors, and L23, to B12's fog gate.\n\
         # Generated by the ignored test `record_the_spire_golden` in\n\
         # game/crates/core/src/world_state/spire_tests.rs.\n\
         # Each line: <ticks> <move_x> <move_z> [BUTTONS].\n",
    );
    text += &format!("seed {SEED}\nscenario {SCENARIO}\n");
    let names = [
        (buttons::LIGHT, "LIGHT"),
        (buttons::DODGE, "DODGE"),
        (buttons::JUMP, "JUMP"),
        (buttons::USE_ITEM, "USE_ITEM"),
        (buttons::INTERACT, "INTERACT"),
        (buttons::LOCK, "LOCK"),
    ];
    let mut at = 0usize;
    while at < r.frames.len() {
        for (tick, c) in &r.commands {
            if *tick == at as u64 {
                assert_eq!(*c, Command::Rest);
                text += "cmd Rest\n";
            }
        }
        let f = r.frames[at];
        let mut n = 1;
        while at + n < r.frames.len()
            && r.frames[at + n] == f
            && !r.commands.iter().any(|(t, _)| *t == (at + n) as u64)
        {
            n += 1;
        }
        text += &format!("{n} {} {}", f.move_x, f.move_z);
        let held: Vec<&str> = names
            .iter()
            .filter(|(m, _)| f.buttons & m != 0)
            .map(|(_, s)| *s)
            .collect();
        if !held.is_empty() {
            text += &format!(" {}", held.join("+"));
        }
        text.push('\n');
        at += n;
    }
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/replays/m6_spire.script"
    );
    std::fs::write(path, text).unwrap();
}
