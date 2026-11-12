//! Tests for Castle Graymantle, part 1 (WP-064): the walks between L14 and L15 are short, the
//! cliff path skips B03's bridge on its own floor, the gatehouse's lever opens the main gate and
//! the postern lets a winner of the bridge in, the bridge and the cliff path keep the player on
//! their floors, the Lower Ward holds the armory, kennel, chapel, wall-walk, and keep stair, and a
//! scripted bot walks the main route from the arrival road to the foot of the keep stair. The
//! golden replay `m6_castle_part1` is that walk.
//!
//! The bot is the Waking Hollow bot (`hollow_tests::Bot`); its legs come from the baked
//! navigation grid, as in `mirrormere_tests`. The grid ignores doors (they are movers added
//! after the bake), so legs that cross the gate are explicit.
//!
//! Record the golden: `cargo test -p lf-core record_the_castle_golden -- --ignored`, then
//! `cargo run -p lf-tools -- replay record-script tests/replays/m6_castle_part1.script --out
//! tests/replays/m6_castle_part1.lfr` (from `game/`).

use lf_content::Content;

use super::hollow_tests::{Bot, Leg};
use crate::events::{Event, LifeEvent};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::nav::Search;
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

pub(super) const SCENARIO: &str = "castle";
pub(super) const SEED: u64 = 64;
/// The longest run, in ticks (the walk takes about 5,000; this only stops a stuck run).
const MAX_TICKS: u64 = 12_000;
/// Acceptance: no walk between Lanterns takes longer than this at run speed, in seconds.
pub(super) const MAX_WALK_S: f32 = 45.0;
/// A path request's node budget.
pub(super) const BUDGET: u32 = 2_000_000;
/// The shelf of the cliff path under the bridge, at its south end.
const SHELF_SOUTH: [f32; 3] = [0.0, -8.0, 6.0];
/// The foot of the keep stair, where the golden ends: 13 m from the Gray Knights.
pub(super) const STAIR_FOOT: [f32; 3] = [0.0, 0.0, 184.0];
/// B03's fog gate (the placeholder in `entities.ron`).
const B03_GATE: [f32; 3] = [0.0, 0.0, 3.5];

pub(super) fn world() -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(SEED, &content, SCENARIO).expect("castle exists")
}

pub(super) fn lantern_pos(w: &World, id: &str) -> [f32; 3] {
    w.layout
        .lanterns
        .iter()
        .find(|l| l.id == id)
        .unwrap_or_else(|| panic!("no Lantern {id}"))
        .pos
}

/// The smoothed nav path between two points, as waypoints.
pub(super) fn walk_path(w: &World, from: [f32; 3], to: [f32; 3]) -> Vec<[f32; 3]> {
    w.nav
        .as_ref()
        .expect("the castle has enemies, so it has a nav grid")
        .find_path(&mut Search::new(), from, to, BUDGET)
        .unwrap_or_else(|e| panic!("no path from {from:?} to {to:?}: {e:?}"))
        .waypoints
}

pub(super) fn length(path: &[[f32; 3]]) -> f32 {
    path.windows(2)
        .map(|s| {
            let (dx, dz) = (s[1][0] - s[0][0], s[1][2] - s[0][2]);
            sqrt(dx * dx + dz * dz)
        })
        .sum()
}

/// Seconds to walk `path` at run speed.
pub(super) fn seconds(w: &World, path: &[[f32; 3]]) -> f32 {
    length(path) / w.locomotion.run_speed
}

/// The nav path through each stop in turn, joined, and its walking time.
pub(super) fn via(w: &World, stops: &[[f32; 3]]) -> (Vec<[f32; 3]>, f32) {
    let mut all: Vec<[f32; 3]> = Vec::new();
    for pair in stops.windows(2) {
        let p = walk_path(w, pair[0], pair[1]);
        all.extend(if all.is_empty() { &p[..] } else { &p[1..] });
    }
    let s = seconds(w, &all);
    (all, s)
}

/// Where the cliff path meets the cistern: through the arch, level with L15.
fn cistern_door() -> [f32; 3] {
    [0.0, -8.0, 64.0]
}

/// The main route's L14 to L15 leg: down the trench stair, along the shelf under the bridge,
/// through the arch.
fn cliff_stops(w: &World) -> Vec<[f32; 3]> {
    vec![
        lantern_pos(w, "lantern_l14"),
        SHELF_SOUTH,
        cistern_door(),
        lantern_pos(w, "lantern_l15"),
    ]
}

/// Every walk between Lanterns, and from the arrival road to L14, takes under 45 seconds at run
/// speed, by the cliff path and (once the main gate is open) by the bridge; L14 is within 20
/// seconds of B03's fog gate.
#[test]
fn every_walk_between_lanterns_is_under_45_seconds() {
    let w = world();
    let (l14, l15) = (
        lantern_pos(&w, "lantern_l14"),
        lantern_pos(&w, "lantern_l15"),
    );
    // The arrival road to L14.
    let s = seconds(&w, &walk_path(&w, w.layout.spawn, l14));
    println!("the arrival road -> L14: {s:.1} s");
    assert!(s < MAX_WALK_S, "the road to L14 takes {s:.1} s");
    // L14 to L15 by the cliff path.
    let (cliff, s_cliff) = via(&w, &cliff_stops(&w));
    println!("L14 -> L15 by the cliff path: {s_cliff:.1} s");
    assert!(s_cliff < MAX_WALK_S, "the cliff path takes {s_cliff:.1} s");
    assert!(
        s_cliff > 8.0,
        "the cliff path is only {s_cliff:.1} s: too close"
    );
    // L15 to the keep stair.
    let s = seconds(&w, &walk_path(&w, l15, STAIR_FOOT));
    println!("L15 -> the keep stair: {s:.1} s");
    assert!(s < MAX_WALK_S, "L15 to the stair takes {s:.1} s");
    // L15 to L14 over the bridge once the lever has opened the gate: up the cistern stair to the
    // forecourt, back into the tunnel, across the deck, to the plaza.
    let (back, s_back) = via(
        &w,
        &[
            l15,
            [9.0, 0.0, 93.0],
            [1.0, 0.0, 72.0],
            [1.0, 0.0, 58.0],
            [0.0, 0.0, 2.0],
            l14,
        ],
    );
    println!("L15 -> L14 by the main gate and the bridge: {s_back:.1} s");
    assert!(s_back < MAX_WALK_S, "the shortcut takes {s_back:.1} s");
    // It stays on the deck, never on the shelf, once it is over the cistern's stair.
    assert!(
        back.iter()
            .filter(|p| (p[2] - 30.0).abs() < 20.0)
            .all(|p| p[1] > -1.0),
        "the shortcut left the deck: {back:?}"
    );
    // L14 is within 20 seconds of B03's fog gate.
    let s = seconds(&w, &walk_path(&w, l14, B03_GATE));
    println!("L14 -> B03's fog gate: {s:.1} s");
    assert!(s < 20.0, "L14 to the fog gate takes {s:.1} s");
    // The cliff path really is the other floor: every waypoint under the deck is on the shelf.
    assert!(
        cliff
            .iter()
            .filter(|p| p[2] > 6.0 && p[2] < 58.0)
            .all(|p| p[1] < -7.0),
        "the cliff path climbs onto the bridge: {cliff:?}"
    );
    // The side branches are reachable from the Lantern they leave: the armory racks, the kennel,
    // the chapel altar, the guard post, the guard room, and the Wick Bead alcove.
    for (what, to) in [
        ("the armory's greatshield", [43.5, 0.3, 135.0]),
        ("the kennel", [-41.5, 0.0, 139.5]),
        ("the chapel altar", [40.5, 0.0, 174.0]),
        ("the guard post", [-42.0, 0.0, 196.0]),
        ("the guard room", [27.5, 0.0, 70.5]),
        ("the Wick Bead alcove", [-27.0, 0.0, 106.0]),
        ("the ladder's foot", [-46.5, 0.0, 130.0]),
    ] {
        let s = seconds(&w, &walk_path(&w, l15, to));
        println!("L15 -> {what}: {s:.1} s");
        assert!(s < MAX_WALK_S, "{what} takes {s:.1} s");
    }
}

/// Drives the player toward (x, z) for `ticks` ticks with no buttons.
pub(super) fn push_toward(w: &mut World, x: f32, z: f32, ticks: u32) {
    for _ in 0..ticks {
        let p = w.actors.get(w.player).unwrap().pos;
        let (dx, dz) = (x - p[0], z - p[2]);
        let d = sqrt(dx * dx + dz * dz).max(0.001);
        let input = TickInput {
            frame: ActionFrame {
                move_x: (dx / d * 127.0) as i8,
                move_z: (dz / d * 127.0) as i8,
                buttons: 0,
            },
            commands: Vec::new(),
        };
        tick(w, &input);
        w.events.drain();
    }
}

/// One tick with `buttons` held and no movement.
pub(super) fn press(w: &mut World, mask: u16) {
    let input = TickInput {
        frame: ActionFrame {
            move_x: 0,
            move_z: 0,
            buttons: mask,
        },
        commands: Vec::new(),
    };
    tick(w, &input);
    w.events.drain();
    tick(w, &TickInput::default());
    w.events.drain();
}

pub(super) fn player_pos(w: &World) -> [f32; 3] {
    w.actors.get(w.player).unwrap().pos
}

pub(super) fn put_player(w: &mut World, pos: [f32; 3]) {
    let id = w.player;
    let p = w.actors.get_mut(id).unwrap();
    p.pos = pos;
    p.vel = [0.0; 3];
    p.motor.grounded = true;
    // Let the world settle (the player lands on whatever floor is under them).
    push_toward(w, pos[0], pos[2], 6);
}

/// A world with every enemy removed, so a test of the level's mechanics isn't a fight.
pub(super) fn quiet_world() -> World {
    let mut w = world();
    let ids: Vec<_> = w.state.ai_enemies.clone();
    for id in ids {
        if let Some(a) = w.actors.get_mut(id) {
            a.pos = [0.0, -45.0, -80.0];
            a.hp = 0;
            a.dead = true;
        }
    }
    w
}

/// The gatehouse's lever opens the main gate, and only the lever does: the gate is a wall from
/// the bridge and from inside until it is pulled, and the one-way postern lets a player in from
/// the bridge but not out. With the gate open, the bridge is the way back to L14.
#[test]
fn the_gatehouse_lever_opens_the_main_gate_and_the_postern_is_one_way() {
    let mut w = quiet_world();
    assert!(!w.state.flags.is_set(&flags::door_open("gate_main")));
    // From the bridge: the main gate is shut, and INTERACT does not open a locked door.
    put_player(&mut w, [1.0, 0.0, 58.0]);
    press(&mut w, buttons::INTERACT);
    assert!(
        !w.state.flags.is_set(&flags::door_open("gate_main")),
        "a locked gate opened with INTERACT"
    );
    push_toward(&mut w, 1.0, 70.0, 120);
    assert!(
        player_pos(&w)[2] < 62.0,
        "walked through the shut main gate: {:?}",
        player_pos(&w)
    );
    // The postern opens from the bridge side.
    put_player(&mut w, [-3.0, 0.0, 60.0]);
    push_toward(&mut w, -3.0, 70.0, 60);
    assert!(player_pos(&w)[2] < 62.0, "the postern was open");
    press(&mut w, buttons::INTERACT);
    assert!(
        w.state.flags.is_set(&flags::door_open("door_postern")),
        "the postern did not open from the bridge"
    );
    push_toward(&mut w, -3.0, 70.0, 120);
    assert!(
        player_pos(&w)[2] > 66.0,
        "stuck at the open postern: {:?}",
        player_pos(&w)
    );
    // Inside, the lever lifts the bar of the main gate.
    put_player(&mut w, [-3.0, 0.0, 76.5]);
    press(&mut w, buttons::INTERACT);
    assert!(
        w.state
            .flags
            .is_set(&flags::lever_pulled("lever_gate_main"))
    );
    assert!(w.state.flags.is_set(&flags::door_open("gate_main")));
    // The bridge is then open from the Barbican to the plaza: through the gate, along the deck.
    push_toward(&mut w, 1.0, 66.0, 120);
    push_toward(&mut w, 1.0, 40.0, 400);
    let p = player_pos(&w);
    assert!(p[2] < 45.0 && p[1] > -1.0, "never crossed the gate: {p:?}");
    push_toward(&mut w, 0.0, 8.0, 600);
    push_toward(&mut w, -12.0, -26.0, 500);
    let p = player_pos(&w);
    assert!(
        p[2] < -20.0 && p[1] > -1.0,
        "the open gate leads nowhere: {p:?}"
    );
}

/// The bridge's parapets and the shelf's lips keep the player on their floors, and the trench's
/// walls keep them out of the drop.
#[test]
fn the_parapets_lips_and_trench_walls_hold() {
    let mut w = quiet_world();
    // The deck: pushing east or west from mid-span stops at the parapet.
    put_player(&mut w, [0.0, 0.0, 30.0]);
    push_toward(&mut w, 20.0, 30.0, 200);
    let p = player_pos(&w);
    assert!(p[0] < 6.0 && p[1] > -1.0, "over the east parapet: {p:?}");
    put_player(&mut w, [0.0, 0.0, 30.0]);
    push_toward(&mut w, -20.0, 30.0, 200);
    let p = player_pos(&w);
    assert!(p[0] > -6.0 && p[1] > -1.0, "over the west parapet: {p:?}");
    // The shelf, a floor below the deck.
    put_player(&mut w, [0.0, -8.0, 30.0]);
    let p = player_pos(&w);
    assert!((p[1] + 8.0).abs() < 0.3, "not on the shelf: {p:?}");
    push_toward(&mut w, 10.0, 30.0, 200);
    let p = player_pos(&w);
    assert!(p[0] < 2.6 && p[1] > -9.0, "off the shelf's east lip: {p:?}");
    put_player(&mut w, [0.0, -8.0, 30.0]);
    push_toward(&mut w, -10.0, 30.0, 200);
    let p = player_pos(&w);
    assert!(
        p[0] > -2.6 && p[1] > -9.0,
        "off the shelf's west lip: {p:?}"
    );
    // The trench stair: its walls keep the player in the stair (or out of it).
    put_player(&mut w, [0.0, -4.0, -8.0]);
    push_toward(&mut w, 8.0, -8.0, 200);
    let p = player_pos(&w);
    assert!(
        p[0] < 3.0 && p[1] < -2.0,
        "out of the trench's east wall: {p:?}"
    );
    // The plaza's cliff edge: walking north from the plaza east of the pylons stops at the lip.
    put_player(&mut w, [30.0, 0.0, -10.0]);
    push_toward(&mut w, 30.0, 20.0, 200);
    let p = player_pos(&w);
    assert!(p[2] < 1.0 && p[1] > -1.0, "over the cliff edge: {p:?}");
}

/// The cistern stair climbs from the cistern to the forecourt, and the well's parapets keep the
/// player from walking off into the hole.
#[test]
fn the_cistern_stair_climbs_to_the_forecourt() {
    let mut w = quiet_world();
    put_player(&mut w, [9.0, -8.0, 72.0]);
    push_toward(&mut w, 9.0, 94.0, 400);
    let p = player_pos(&w);
    assert!(p[2] > 90.0 && p[1] > -0.3, "stuck on the stair: {p:?}");
    // On the courtyard's slab, west of the well, the parapet stops a push toward the hole.
    put_player(&mut w, [0.0, 0.0, 80.0]);
    push_toward(&mut w, 9.0, 80.0, 200);
    let p = player_pos(&w);
    assert!(p[0] < 5.0 && p[1] > -0.3, "fell into the well: {p:?}");
}

/// The wall-walk's ladder climbs the west wall.
#[test]
fn the_ladder_climbs_to_the_wall_walk() {
    let mut w = quiet_world();
    put_player(&mut w, [-47.4, 0.0, 130.0]);
    press(&mut w, buttons::INTERACT);
    assert!(w.mech.climb.is_some(), "the ladder did not grab");
    for _ in 0..600 {
        if w.mech.climb.is_none() {
            break;
        }
        let input = TickInput {
            frame: ActionFrame {
                // The stick is projected on the ladder's facing (-X): up is -X.
                move_x: -127,
                move_z: 0,
                buttons: 0,
            },
            commands: Vec::new(),
        };
        tick(&mut w, &input);
        w.events.drain();
    }
    assert!(w.mech.climb.is_none(), "never reached the top");
    let p = player_pos(&w);
    assert!((p[1] - 7.0).abs() < 0.3, "not on the wall-walk: {p:?}");
    // Along the wall-walk, north to the drop (the parapet is open at z 157..173), then down onto
    // the item ledge.
    push_toward(&mut w, -49.5, 140.0, 200);
    push_toward(&mut w, -49.5, 166.0, 200);
    let p = player_pos(&w);
    assert!(p[1] > 6.7, "off the wall-walk: {p:?}");
    push_toward(&mut w, -45.5, 166.0, 120);
    let p = player_pos(&w);
    assert!(p[1] > 2.0 && p[1] < 3.0, "did not land on the ledge: {p:?}");
}

/// Every enemy the spec puts in the castle's first half is placed, the items part 1 owns are in
/// the level, B03's Echo Stone is on the bridge, and B03 is an M7 work package.
#[test]
fn the_castle_has_its_enemies_items_and_echo_stone() {
    let content = Content::load_embedded().unwrap();
    let level = content.scenario(SCENARIO).unwrap();
    let count = |kind: &str| level.enemies.iter().filter(|e| e.enemy == kind).count();
    for kind in [
        "husk_servant",
        "gray_knight",
        "thorn_hound",
        "acolyte",
        "gray_archer",
    ] {
        assert!(count(kind) > 0, "no {kind} in the castle");
    }
    assert_eq!(
        count("mimic_chest"),
        1,
        "exactly one Mimic Chest in the game"
    );
    assert_eq!(count("thorn_hound"), 3, "a pack of three in the kennel");
    assert_eq!(count("acolyte"), 2, "the chapel's two Acolytes");
    assert_eq!(
        level
            .enemies
            .iter()
            .filter(|e| e.name.starts_with("knight_stair"))
            .count(),
        2,
        "two Gray Knights guard the keep stair"
    );
    assert!(level.enemies.len() < 60, "{} enemies", level.enemies.len());
    // The Mimic stands in the armory (x 24..46, z 120..150); the other enemies stay out of it.
    let in_armory = |p: &[f32; 3]| p[0] > 24.0 && p[0] < 46.0 && p[2] > 120.0 && p[2] < 150.0;
    let mimic = level
        .enemies
        .iter()
        .find(|e| e.enemy == "mimic_chest")
        .unwrap();
    assert!(in_armory(&mimic.pos), "the Mimic is outside the armory");
    assert_eq!(
        level.enemies.iter().filter(|e| in_armory(&e.pos)).count(),
        1
    );
    // Items: the Wick Bead (the Barbican), the Barbican's shield and charm, the armory's racks,
    // the Lower Ward's weapons, and the chapel's spell.
    let items: Vec<&str> = level.pickups.iter().map(|p| p.item.as_str()).collect();
    for item in [
        "itm_wick_bead",
        "wpn_castle_kite",
        "chm_bulwark",
        "wpn_whisperfang",
        "wpn_gray_halberd",
        "wpn_morning_star",
        "wpn_graymantle_greatshield",
        "arm_graymantle_body",
        "arm_graymantle_head",
        "chm_ox",
        "spl_steadfast",
    ] {
        assert!(items.contains(&item), "no {item} in the castle");
    }
    assert!(level.chests.len() >= 3, "chests: {}", level.chests.len());
    // The Echo Stone is on the bridge, inside the fog gate, and the world resolves it.
    let stone = level
        .echo_stones
        .iter()
        .find(|s| s.id == "stone_b03")
        .expect("B03's stone");
    assert!(
        stone.pos[0].abs() < 6.0 && stone.pos[2] > B03_GATE[2] && stone.pos[2] < 60.0,
        "the stone is off the bridge: {:?}",
        stone.pos
    );
    assert!(!world().echo_table.stones.is_empty());
    assert!(level.bosses.is_empty(), "B03 is an M7 work package");
    // Two Lanterns, the gate and postern, one lever, and the ladder.
    // (Part 2 adds Lanterns, doors, levers, and ladders of its own; see castle2_tests.)
    for id in ["lantern_l14", "lantern_l15"] {
        assert!(level.lanterns.iter().any(|l| l.id == id));
    }
    for id in ["gate_main", "door_postern"] {
        assert!(level.doors.iter().any(|d| d.id == id));
    }
    assert!(level.levers.iter().any(|l| l.id == "lever_gate_main"));
    assert!(level.ladders.iter().any(|l| l.id == "ladder_wall_west"));
}

/// Drives the player to a Lantern's side and rests: used by the walk below.
pub(super) fn goto(w: &World, legs: &mut Vec<Leg>, at: [f32; 3], to: [f32; 3]) {
    for p in &walk_path(w, at, to)[1..] {
        legs.push(Leg::Goto(p[0], p[2]));
    }
}

/// The route leg by leg: the arrival road, L14 (rest), the cliff path, L15 (rest), the cistern's
/// stair, the gatehouse lever, the inner arch, and the Lower Ward to the foot of the keep stair.
pub(super) fn legs(w: &World) -> Vec<Leg> {
    let mut legs = Vec::new();
    let l14 = lantern_pos(w, "lantern_l14");
    let l15 = lantern_pos(w, "lantern_l15");
    // The road to L14: stop a meter short of the Lantern, on the side the path arrives from.
    let road = walk_path(w, w.layout.spawn, l14);
    for p in &road[1..road.len() - 1] {
        legs.push(Leg::Goto(p[0], p[2]));
    }
    let (last, prev) = (road[road.len() - 1], road[road.len() - 2]);
    let (dx, dz) = (prev[0] - last[0], prev[2] - last[2]);
    let d = sqrt(dx * dx + dz * dz).max(0.001);
    legs.push(Leg::Goto(last[0] + dx / d, last[2] + dz / d));
    legs.push(Leg::Rest);
    legs.push(Leg::Wait(10));
    // The cliff path to L15.
    let mut at = l14;
    for stop in [SHELF_SOUTH, cistern_door()] {
        goto(w, &mut legs, at, stop);
        at = stop;
    }
    let path = walk_path(w, at, l15);
    for p in &path[1..path.len() - 1] {
        legs.push(Leg::Goto(p[0], p[2]));
    }
    let (last, prev) = (path[path.len() - 1], path[path.len() - 2]);
    let (dx, dz) = (prev[0] - last[0], prev[2] - last[2]);
    let d = sqrt(dx * dx + dz * dz).max(0.001);
    legs.push(Leg::Goto(last[0] + dx / d, last[2] + dz / d));
    legs.push(Leg::Rest);
    legs.push(Leg::Wait(10));
    // Up the cistern's stair to the forecourt, then back into the tunnel for the lever.
    let top = [9.0, 0.0, 94.0];
    goto(w, &mut legs, l15, top);
    // The nav cell the leg ends in is up to half a meter off: stop well inside the 1.6 m reach.
    let lever = [-3.0, 0.0, 76.0];
    goto(w, &mut legs, top, lever);
    legs.push(Leg::Interact);
    legs.push(Leg::Wait(5));
    // Through the inner arch and across the Lower Ward to the keep stair's foot.
    let arch = [0.0, 0.0, 110.0];
    goto(w, &mut legs, lever, arch);
    goto(w, &mut legs, arch, STAIR_FOOT);
    legs
}

/// What a run produced.
struct Run {
    frames: Vec<ActionFrame>,
    commands: Vec<(u64, Command)>,
    world: World,
    events: Vec<Event>,
    bot: Bot,
}

fn walk() -> Run {
    let mut w = world();
    let mut bot = Bot::with_legs(legs(&w));
    bot.approach = true;
    let (mut frames, mut commands, mut events) = (Vec::new(), Vec::new(), Vec::new());
    while w.tick < MAX_TICKS && !bot.done() {
        let input: TickInput = bot.act(&w);
        frames.push(input.frame);
        for c in &input.commands {
            commands.push((w.tick, c.clone()));
        }
        tick(&mut w, &input);
        events.extend(w.events.drain());
        if events
            .iter()
            .any(|e| matches!(e, Event::Life(LifeEvent::Died { .. })))
        {
            break;
        }
    }
    Run {
        frames,
        commands,
        world: w,
        events,
        bot,
    }
}

#[test]
fn the_main_route_reaches_both_lanterns_and_the_keep_stair() {
    let r = walk();
    let w = &r.world;
    let p = w.actors.get(w.player).unwrap();
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, Event::Life(LifeEvent::Died { .. }))),
        "the bot died at tick {} near {:?}, on leg {} of {}",
        w.tick,
        p.pos,
        r.bot.leg,
        r.bot.legs.len()
    );
    for id in ["lantern_l14", "lantern_l15"] {
        assert!(
            w.state.flags.is_set(&flags::lantern_lit(id)),
            "{id} was not lit (leg {} of {}, tick {}, at {:?})",
            r.bot.leg,
            r.bot.legs.len(),
            w.tick,
            p.pos
        );
    }
    assert!(
        r.bot.done(),
        "stopped at leg {} of {} at {:?}",
        r.bot.leg,
        r.bot.legs.len(),
        p.pos
    );
    // The lever opened the main gate on the way.
    assert!(
        w.state
            .flags
            .is_set(&flags::lever_pulled("lever_gate_main"))
    );
    assert!(w.state.flags.is_set(&flags::door_open("gate_main")));
    // The walk ends at the foot of the keep stair, in the Lower Ward.
    assert!(
        (p.pos[0] - STAIR_FOOT[0]).abs() < 2.0
            && (p.pos[2] - STAIR_FOOT[2]).abs() < 2.0
            && p.pos[1].abs() < 0.5,
        "ended at {:?}",
        p.pos
    );
    // The walk stays under five minutes of game time.
    assert!(w.tick < 18_000, "took {} ticks", w.tick);
}

/// Writes `game/tests/replays/m6_castle_part1.script`. See the module comment.
#[test]
#[ignore = "writes game/tests/replays/m6_castle_part1.script"]
fn record_the_castle_golden() {
    let r = walk();
    assert!(r.bot.done());
    let mut text = String::from(
        "# WP-064 golden: from the arrival road through L14, the cliff path under the bridge, L15,\n\
         # the cistern's stair and the gatehouse lever, to the foot of the Lower Ward's keep stair.\n\
         # Generated by the ignored test `record_the_castle_golden` in\n\
         # game/crates/core/src/world_state/castle_tests.rs.\n\
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
        "/../../tests/replays/m6_castle_part1.script"
    );
    std::fs::write(path, text).unwrap();
}
