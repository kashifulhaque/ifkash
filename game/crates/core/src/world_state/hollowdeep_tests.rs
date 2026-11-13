//! Tests for Hollowdeep and the Cinderdeep Mine (WP-062): the darkness rule and its glowing
//! zones, the mine carts (ride one down, push one into a sturdy wall), the Root Stair's trapdoor,
//! the walks between Lanterns, and two scripted bots that walk each area's critical path. The
//! golden replays `m6_cinderdeep_mine` and `m6_hollowdeep` are those walks.
//!
//! The bot is the Waking Hollow bot (`hollow_tests::Bot`); its legs come from the baked navigation
//! grid, as in `mirrormere_tests`. A ride is an `Interact`, then a `Wait` as long as the trip.
//!
//! Record the goldens: `cargo test -p lf-core record_the_ -- --ignored`, then, from `game/`,
//! `cargo run -p lf-tools -- replay record-script tests/replays/m6_cinderdeep_mine.script --out
//! tests/replays/m6_cinderdeep_mine.lfr` (and the same for `m6_hollowdeep`).

use lf_content::Content;

use super::hollow_tests::{Bot, Leg};
use super::mechanics::{darkness, mine_cart, trapdoor};
use crate::events::{Event, LifeEvent};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::nav::Search;
use crate::tick::tick;
use crate::world::{DT, World};
use crate::world_state::flags;

const MINE: &str = "cinderdeep_mine";
const DEEP: &str = "hollowdeep";
const MINE_SEED: u64 = 62;
const DEEP_SEED: u64 = 63;
/// The longest run, in ticks (a stuck run stops here).
const MAX_TICKS: u64 = 30_000;
/// Acceptance: no walk between Lanterns takes longer than this at run speed, in seconds.
const MAX_WALK_S: f32 = 45.0;
/// A boss's fog gate is within this many seconds of a Lantern (docs/game/02-world.md).
const GATE_WALK_S: f32 = 20.0;
const BUDGET: u32 = 4_000_000;
const DEEP_LANTERNS: [&str; 3] = ["lantern_l11", "lantern_l12", "lantern_l13"];

fn world(scenario: &str, seed: u64) -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(seed, &content, scenario).unwrap_or_else(|e| panic!("{scenario}: {e:?}"))
}

fn mine() -> World {
    world(MINE, MINE_SEED)
}

fn deep() -> World {
    world(DEEP, DEEP_SEED)
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
        .expect("the area has enemies, so it has a nav grid")
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

/// Seconds to walk `path` at run speed.
fn seconds(w: &World, path: &[[f32; 3]]) -> f32 {
    length(path) / w.locomotion.run_speed
}

fn player_pos(w: &World) -> [f32; 3] {
    w.actors.get(w.player).unwrap().pos
}

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.vel = [0.0; 3];
}

/// One tick with the given stick and buttons.
fn step(w: &mut World, move_x: i8, move_z: i8, buttons: u16) {
    let input = TickInput {
        frame: ActionFrame {
            move_x,
            move_z,
            buttons,
        },
        commands: Vec::new(),
    };
    tick(w, &input);
    w.events.drain();
}

/// Presses `INTERACT` for one tick, then lets go for one.
fn press_interact(w: &mut World) {
    step(w, 0, 0, buttons::INTERACT);
    step(w, 0, 0, 0);
}

/// Drives the player toward (x, z) for `ticks` ticks with no buttons.
fn push_toward(w: &mut World, x: f32, z: f32, ticks: u32) {
    for _ in 0..ticks {
        let p = player_pos(w);
        let (dx, dz) = (x - p[0], z - p[2]);
        let d = sqrt(dx * dx + dz * dz).max(0.001);
        step(w, (dx / d * 127.0) as i8, (dz / d * 127.0) as i8, 0);
    }
}

// ---------------------------------------------------------------------------------------
// Darkness

/// Outside the glowing zones, the player sees 6 m; a light (the relic's flag, or a Lantern Flare)
/// makes it 14 m; in a glowing zone or beside a Lantern there is no limit. The renderer's word is
/// that number.
#[test]
fn darkness_limits_sight_outside_the_glowing_zones() {
    let mut w = deep();
    // The spawn is inside the arrival cave's glow.
    assert!(darkness::lit_at(&w, w.layout.spawn[0], w.layout.spawn[2]));
    assert_eq!(darkness::visibility_m(&w), None);
    // The dark nook west of the Fungal Shore: no glow, no Lantern.
    let nook = [-110.0, 0.0, -64.0];
    assert!(!darkness::lit_at(&w, nook[0], nook[2]));
    put_player(&mut w, nook);
    assert_eq!(darkness::visibility_m(&w), Some(6.0));
    let words = super::mechanics::render_state(&w);
    assert_eq!(
        *words.last().unwrap(),
        6.0,
        "the renderer's visibility word"
    );
    // The smoltorch's flag is a light.
    w.state.flags.raise("codex.projects.smoltorch");
    assert_eq!(darkness::visibility_m(&w), Some(14.0));
    w.state.flags.clear("codex.projects.smoltorch");
    assert_eq!(darkness::visibility_m(&w), Some(6.0));
    // A Lantern Flare burns for 600 ticks, then goes out; a second flare keeps the later end.
    darkness::flare(&mut w, 600);
    assert_eq!(darkness::visibility_m(&w), Some(14.0));
    for _ in 0..300 {
        step(&mut w, 0, 0, 0);
    }
    assert_eq!(darkness::visibility_m(&w), Some(14.0), "300 ticks in");
    for _ in 0..301 {
        step(&mut w, 0, 0, 0);
    }
    assert_eq!(darkness::visibility_m(&w), Some(6.0), "burnt out");
    // A Lantern lights the ground around it: 9 m from L11, 12 m away is dark.
    let l11 = lantern_pos(&w, "lantern_l11");
    assert!(darkness::lit_at(&w, l11[0] + 8.0, l11[2]));
    assert!(!darkness::lit_at(&w, l11[0] + 20.0, l11[2] + 40.0));
    // An area with no darkness rule is lit everywhere, and sends no visibility word.
    let m = mine();
    assert_eq!(darkness::visibility_m(&m), None);
    assert!(darkness::lit_at(&m, 0.0, 0.0));
    let plain = super::mechanics::render_state(&m);
    let counted = 1
        + m.layout.ladders.len()
        + m.mech.lifts.len()
        + m.layout.doors.len()
        + m.layout.levers.len()
        + m.layout.hollow_walls.len()
        + m.layout.pickups.len()
        + m.layout.chests.len()
        + m.layout.messages.len()
        + m.layout.pools.len()
        + m.layout.carts.len()
        + m.layout.trapdoors.len();
    assert_eq!(plain.len(), counted, "no darkness word in the mine");
}

/// The bake's rule and the core's rule are one function: every stop on Hollowdeep's critical
/// path is lit, and so is the walk to B09's gate and the way up the Root Stair.
#[test]
fn the_critical_path_stays_inside_the_glowing_zones() {
    let w = deep();
    let l = |id: &str| lantern_pos(&w, id);
    let mut stops = vec![w.layout.spawn];
    stops.extend(DEEP_LANTERNS.iter().map(|id| l(id)));
    stops.push([104.0, 0.0, 92.0]); // B09's fog gate
    for pair in stops.windows(2) {
        for p in walk_path(&w, pair[0], pair[1]).windows(2) {
            let (dx, dz) = (p[1][0] - p[0][0], p[1][2] - p[0][2]);
            let n = (sqrt(dx * dx + dz * dz) / 0.5).ceil().max(1.0) as u32;
            for k in 0..=n {
                let t = k as f32 / n as f32;
                let (x, z) = (p[0][0] + dx * t, p[0][2] + dz * t);
                assert!(
                    darkness::lit_at(&w, x, z),
                    "{:?} -> {:?} is dark at ({x:.1}, {z:.1})",
                    pair[0],
                    pair[1]
                );
            }
        }
    }
    // The Root Stair's way up is lit too (L13 -> the ladder's foot), though it is a loop.
    for p in walk_path(&w, l("lantern_l13"), [146.0, 20.0, -34.0]) {
        assert!(
            darkness::lit_at(&w, p[0], p[2]),
            "the Root Stair is dark at {p:?}"
        );
    }
    // And the dark places exist: the nook, the ford, the Glasswing Mouth's east end.
    for (what, x, z) in [
        ("the nook", -110.0, -64.0),
        ("the ford", 25.0, 44.0),
        ("the lakeside", -40.0, -80.0),
    ] {
        assert!(!darkness::lit_at(&w, x, z), "{what} should be dark");
    }
}

// ---------------------------------------------------------------------------------------
// Mine carts

fn cart_index(w: &World, id: &str) -> usize {
    w.layout.carts.iter().position(|c| c.id == id).unwrap()
}

/// Rides the gallery cart down: the player is carried at the cart's speed and held in place (no
/// stick, no buttons), steps off at the bottom, and rides it back up from the same spot.
#[test]
fn a_ridden_cart_carries_the_player_down_the_rail_and_back() {
    let mut w = mine();
    let i = cart_index(&w, "cart_gallery");
    let cart = w.layout.carts[i].clone();
    let len = mine_cart::rail_len(&cart);
    let trip = (len / cart.speed_mps / DT).ceil() as u32;
    // Beside the cart at the top (the gallery's slope under the rail is y = -22 (z + 52) / 112).
    put_player(&mut w, [1.0, -0.6, -49.5]);
    step(&mut w, 0, 0, 0);
    assert_eq!(super::mechanics::prompt(&w), mine_cart::PROMPT_RIDE);
    press_interact(&mut w);
    assert!(mine_cart::riding(&w), "INTERACT beside the cart boards it");
    // Held in place: the stick and every button do nothing, and the player is on the rail.
    for n in 0..trip / 2 {
        step(&mut w, 127, -127, buttons::LIGHT | buttons::JUMP);
        let run = w.mech.carts[i];
        let (at, _) = mine_cart::point_at(&cart, run.s);
        let p = player_pos(&w);
        assert!(
            (p[0] - at[0]).abs() < 0.05
                && (p[2] - at[2]).abs() < 0.05
                && (p[1] - at[1]).abs() < 0.05,
            "tick {n}: the player is at {p:?}, the cart at {at:?}"
        );
    }
    let mut ticks = trip / 2;
    while mine_cart::riding(&w) && ticks < trip * 2 {
        step(&mut w, 0, 0, 0);
        ticks += 1;
    }
    assert!(
        (i64::from(ticks) - i64::from(trip)).abs() <= 3,
        "the ride took {ticks} ticks, expected about {trip}"
    );
    let end = cart.rail[cart.rail.len() - 1];
    let p = player_pos(&w);
    assert!(
        (p[0] - end[0]).abs() < 0.2 && (p[2] - end[2]).abs() < 0.2,
        "{p:?} vs {end:?}"
    );
    // They can walk again, and the cart waits at the bottom for the ride back.
    for _ in 0..5 {
        step(&mut w, 0, 127, 0);
    }
    assert!(
        player_pos(&w)[2] > end[2] + 0.2,
        "the player can move after the ride"
    );
    put_player(&mut w, [4.5, end[1], end[2] - 1.0]);
    step(&mut w, 0, 0, 0);
    assert_eq!(super::mechanics::prompt(&w), mine_cart::PROMPT_RIDE);
    press_interact(&mut w);
    assert!(mine_cart::riding(&w));
    let mut back = 0;
    while mine_cart::riding(&w) && back < trip * 2 {
        step(&mut w, 0, 0, 0);
        back += 1;
    }
    let start = cart.rail[0];
    let p = player_pos(&w);
    assert!(
        (p[2] - start[2]).abs() < 0.3,
        "the ride back ends at {p:?}, not {start:?}"
    );
    // A cart that is rolling can't be boarded mid-trip, and nothing here is in the state hash
    // once both carts are home again.
    assert_eq!(w.mech.carts[i].s, 0.0);
}

/// The push cart rolls into the sturdy wall and breaks it. A sword swing at the same wall
/// doesn't; the same swing does reveal an ordinary hollow wall.
#[test]
fn a_pushed_cart_breaks_the_sturdy_wall_and_a_sword_does_not() {
    let mut w = mine();
    let wall = w
        .layout
        .hollow_walls
        .iter()
        .position(|h| h.id == "wall_smoltorch")
        .unwrap();
    assert!(w.layout.hollow_walls[wall].sturdy);
    let revealed = flags::wall_revealed("wall_smoltorch");
    // The wall seals the corridor: walking west stops at it.
    put_player(&mut w, [-29.0, -22.0, 78.0]);
    push_toward(&mut w, -46.0, 78.0, 240);
    assert!(
        player_pos(&w)[0] > -33.2,
        "the wall let the player through: {:?}",
        player_pos(&w)
    );
    // A swing against it (facing west, close) reveals nothing.
    put_player(&mut w, [-31.4, -22.0, 78.0]);
    w.actors.get_mut(w.player).unwrap().yaw = -core::f32::consts::FRAC_PI_2;
    for _ in 0..60 {
        step(&mut w, 0, 0, buttons::LIGHT);
        step(&mut w, 0, 0, 0);
    }
    assert!(
        !w.state.flags.is_set(&revealed),
        "a sword broke the sturdy wall"
    );
    // The same swing reveals the wall once it isn't sturdy, so the swing did reach it.
    w.layout.hollow_walls[wall].sturdy = false;
    put_player(&mut w, [-31.4, -22.0, 78.0]);
    w.actors.get_mut(w.player).unwrap().yaw = -core::f32::consts::FRAC_PI_2;
    for _ in 0..60 {
        step(&mut w, 0, 0, buttons::LIGHT);
        step(&mut w, 0, 0, 0);
    }
    assert!(
        w.state.flags.is_set(&revealed),
        "the swing never reached the wall"
    );
    w.layout.hollow_walls[wall].sturdy = true;
    w.state.flags.clear(&revealed);
    super::mechanics::sync_colliders(&mut w);
    // Now shove the cart (after the swings have finished).
    let i = cart_index(&w, "cart_smoltorch");
    put_player(&mut w, [-8.6, -22.0, 77.4]);
    for _ in 0..90 {
        step(&mut w, 0, 0, 0);
    }
    assert_eq!(super::mechanics::prompt(&w), mine_cart::PROMPT_PUSH);
    press_interact(&mut w);
    assert!(
        !mine_cart::riding(&w),
        "a pushed cart doesn't carry the player"
    );
    let len = mine_cart::rail_len(&w.layout.carts[i]);
    let trip = (len / w.layout.carts[i].speed_mps / DT).ceil() as u32;
    for _ in 0..trip + 5 {
        step(&mut w, 0, 0, 0);
    }
    assert!(
        w.state.flags.is_set(&revealed),
        "the cart never broke the wall"
    );
    assert!(
        w.state
            .flags
            .is_set(&mine_cart::spent_flag("cart_smoltorch"))
    );
    assert_eq!(w.mech.carts[i].s, len);
    // It stays at the end: no prompt, no second shove.
    put_player(&mut w, [-30.0, -22.0, 78.0]);
    step(&mut w, 0, 0, 0);
    assert_ne!(super::mechanics::prompt(&w), mine_cart::PROMPT_PUSH);
    // The way into the chamber is open.
    push_toward(&mut w, -46.0, 78.0, 300);
    assert!(
        player_pos(&w)[0] < -40.0,
        "the broken wall still blocks: {:?}",
        player_pos(&w)
    );
}

/// Dying on a cart lets go of it, and the cart finishes its trip empty.
#[test]
fn dying_on_a_cart_lets_go() {
    let mut w = mine();
    put_player(&mut w, [1.0, -0.6, -49.5]);
    step(&mut w, 0, 0, 0);
    press_interact(&mut w);
    assert!(mine_cart::riding(&w));
    {
        let p = w.actors.get_mut(w.player).unwrap();
        p.hp = 0;
        p.dead = true;
    }
    step(&mut w, 0, 0, 0);
    assert!(!mine_cart::riding(&w), "a dead player isn't carried");
}

// ---------------------------------------------------------------------------------------
// The Root Stair's trapdoor

/// The hatch opens from below only: it drops the ladder, the ladder climbs to the ledge, and from
/// the ledge there is nothing to push.
#[test]
fn the_root_stair_trapdoor_opens_from_below_and_drops_the_ladder() {
    let mut w = deep();
    let ladder = 0;
    assert_eq!(w.layout.ladders[ladder].id, "root_ladder");
    let foot = [146.0, 20.0, -34.0];
    put_player(&mut w, foot);
    step(&mut w, 0, 0, 0);
    assert_eq!(super::mechanics::prompt(&w), trapdoor::PROMPT_OPEN);
    assert!(!trapdoor::is_open(&w, 0));
    let hatch_before = w
        .physics
        .mover_center(w.layout.lifts.len() + w.layout.doors.len() + w.layout.hollow_walls.len())
        .unwrap();
    press_interact(&mut w);
    assert!(trapdoor::is_open(&w, 0));
    assert!(
        w.state.flags.is_set(&flags::ladder_down("root_ladder")),
        "the ladder dropped"
    );
    let hatch_after = w
        .physics
        .mover_center(w.layout.lifts.len() + w.layout.doors.len() + w.layout.hollow_walls.len())
        .unwrap();
    assert!(
        hatch_after[0] < hatch_before[0] - 1.9,
        "the hatch slid aside: {hatch_before:?} -> {hatch_after:?}"
    );
    // Now the ladder is there to climb.
    step(&mut w, 0, 0, 0);
    assert_eq!(super::mechanics::prompt(&w), 6, "climb the ladder");
    press_interact(&mut w);
    assert!(super::mechanics::climbing(&w));
    for _ in 0..200 {
        step(&mut w, 127, 0, 0);
        if !super::mechanics::climbing(&w) {
            break;
        }
    }
    let p = player_pos(&w);
    assert!(
        (p[1] - 24.0).abs() < 0.1,
        "the ladder tops out at the ledge: {p:?}"
    );
    // From the ledge there is no hatch to push: it opens from below only (the ladder can be
    // climbed back down, which is the prompt a player there gets).
    step(&mut w, 0, 0, 0);
    assert_ne!(super::mechanics::prompt(&w), trapdoor::PROMPT_OPEN);
    // And the ledge walks on to the Spire's connection.
    push_toward(&mut w, 148.6, -34.0, 60);
    let p = player_pos(&w);
    // The Spire connection (area.ron) is a 1.2 m sphere around (148.6, 24, -34).
    assert!(
        (p[0] - 148.6).abs() < 1.2 && (p[1] - 24.0).abs() < 0.5,
        "the player didn't reach the connection: {p:?}"
    );
    // The shut hatch can't be bypassed: a fresh world's ladder is raised until the hatch opens.
    let fresh = deep();
    assert!(!fresh.state.flags.is_set(&flags::ladder_down("root_ladder")));
}

// ---------------------------------------------------------------------------------------
// Walks

/// The mine: the walk from the Verge mouth to L10 and from L10 to B08's fog gate and to the exit
/// are under 45 seconds, and B08's gate is within 20 seconds of L10.
#[test]
fn every_walk_in_the_mine_is_under_45_seconds() {
    let w = mine();
    let l10 = lantern_pos(&w, "lantern_l10");
    let gate = [44.0, -22.0, 82.0];
    for (what, from, to, max, min) in [
        ("the mouth -> L10", w.layout.spawn, l10, MAX_WALK_S, 8.0),
        ("L10 -> B08's fog gate", l10, gate, GATE_WALK_S, 4.0),
        ("L10 -> the exit", l10, [0.0, -22.0, 98.0], MAX_WALK_S, 4.0),
        (
            "L10 -> the push cart",
            l10,
            [-8.6, -22.0, 77.4],
            MAX_WALK_S,
            1.0,
        ),
        (
            "L10 -> the smoltorch",
            l10,
            [-46.0, -22.0, 78.0],
            MAX_WALK_S,
            4.0,
        ),
    ] {
        let s = seconds(&w, &walk_path(&w, from, to));
        println!("{what}: {s:.1} s");
        assert!(s < max, "{what} takes {s:.1} s");
        assert!(s > min, "{what} takes only {s:.1} s");
    }
    // The ride is quicker than the walk it replaces.
    let cart = &w.layout.carts[cart_index(&w, "cart_gallery")];
    let ride = mine_cart::rail_len(cart) / cart.speed_mps;
    let walk = seconds(&w, &walk_path(&w, [1.0, -0.6, -49.5], [3.0, -21.2, 56.0]));
    println!("the gallery: ride {ride:.1} s, walk {walk:.1} s");
    assert!(ride < walk * 0.6);
}

/// Hollowdeep: every walk along the Lantern chain is under 45 seconds, B09's gate is within 20
/// seconds of L13, and the side branches (the nook, the island's ford, the Root Stair) are under
/// 45 seconds from the Lantern they leave.
#[test]
fn every_walk_between_hollowdeep_lanterns_is_under_45_seconds() {
    let w = deep();
    let mut stops: Vec<(String, [f32; 3])> = vec![("the arrival cave".into(), w.layout.spawn)];
    for id in DEEP_LANTERNS {
        stops.push((id.into(), lantern_pos(&w, id)));
    }
    for pair in stops.windows(2) {
        let s = seconds(&w, &walk_path(&w, pair[0].1, pair[1].1));
        println!("{} -> {}: {s:.1} s", pair[0].0, pair[1].0);
        assert!(
            s < MAX_WALK_S,
            "{} -> {} takes {s:.1} s",
            pair[0].0,
            pair[1].0
        );
        assert!(
            s > 8.0,
            "{} -> {} takes only {s:.1} s",
            pair[0].0,
            pair[1].0
        );
    }
    let (l11, l12, l13) = (
        lantern_pos(&w, "lantern_l11"),
        lantern_pos(&w, "lantern_l12"),
        lantern_pos(&w, "lantern_l13"),
    );
    let gate = [104.0, 0.0, 92.0];
    let s = seconds(&w, &walk_path(&w, l13, gate));
    println!("lantern_l13 -> B09's fog gate: {s:.1} s");
    assert!(s < GATE_WALK_S, "L13 to the fog gate takes {s:.1} s");
    for (what, from, to) in [
        ("the nook's sword", l11, [-116.0, 0.0, -64.0]),
        ("the Belfry lift", l11, [-90.0, 0.0, -50.0]),
        ("the Mossgut lift", l12, [-98.0, 0.0, 18.0]),
        ("the ford's landing", l12, [25.0, 0.0, 42.0]),
        ("the island's relic by the ford", l12, [34.0, 0.0, -50.0]),
        ("Glasswing Grotto's arch", l12, [-16.0, 0.0, 96.0]),
        ("the Root Stair's foot", l13, [74.0, 0.0, -34.0]),
        ("the Root Stair's top", l13, [146.0, 20.0, -34.0]),
        ("the endark alcove", l13, [106.0, 8.57, -20.0]),
    ] {
        let s = seconds(&w, &walk_path(&w, from, to));
        println!("{what}: {s:.1} s");
        assert!(s < MAX_WALK_S, "{what} takes {s:.1} s");
    }
}

/// The Lamp Tree shows over the lake from L11, the landmark of the entry route.
#[test]
fn the_lamp_tree_is_visible_from_l11() {
    let w = deep();
    let l11 = lantern_pos(&w, "lantern_l11");
    let eye = [l11[0], l11[1] + 1.6, l11[2]];
    for (what, target) in [
        ("the cap's west face", [13.0, 31.5, -50.0]),
        ("the trunk's upper west face", [23.0, 20.0, -50.0]),
        ("the root's west face", [20.0, 1.5, -50.0]),
    ] {
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let dist = sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
        let dir = [d[0] / dist, d[1] / dist, d[2] / dist];
        let hit = w.physics.raycast(eye, dir, dist + 20.0);
        assert!(
            hit.is_none_or(|h| h > dist - 6.0),
            "something blocks the view of the Lamp Tree's {what} at {hit:?} m of {dist:.0}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// The scripted walks

/// Walks the nav path from `at` to `to`, leaving off the first point (where it stands).
fn goto(w: &World, legs: &mut Vec<Leg>, at: [f32; 3], to: [f32; 3]) {
    for p in &walk_path(w, at, to)[1..] {
        legs.push(Leg::Goto(p[0], p[2]));
    }
}

/// Goes to a Lantern: stops a meter short on the side the path arrives from, rests.
fn rest_at(w: &World, legs: &mut Vec<Leg>, at: [f32; 3], lantern: [f32; 3]) {
    let path = walk_path(w, at, lantern);
    for p in &path[1..path.len() - 1] {
        legs.push(Leg::Goto(p[0], p[2]));
    }
    let (last, prev) = (path[path.len() - 1], path[path.len() - 2]);
    let (dx, dz) = (prev[0] - last[0], prev[2] - last[2]);
    let d = sqrt(dx * dx + dz * dz).max(0.001);
    legs.push(Leg::Goto(last[0] + dx / d, last[2] + dz / d));
    legs.push(Leg::Rest);
    legs.push(Leg::Wait(10));
}

/// The ticks a cart takes to finish its trip, with a margin.
fn trip_ticks(w: &World, id: &str) -> u16 {
    let c = &w.layout.carts[cart_index(w, id)];
    ((mine_cart::rail_len(c) / c.speed_mps / DT).ceil() as u16) + 20
}

/// The mine's route: the mouth, the ride, L10 (rest), the push cart, the smoltorch chamber's
/// three pickups, and the way on to Hollowdeep.
fn mine_legs(w: &World) -> Vec<Leg> {
    let mut legs = Vec::new();
    let stand_top = [1.0, -0.6, -49.5];
    goto(w, &mut legs, w.layout.spawn, stand_top);
    legs.push(Leg::Interact);
    legs.push(Leg::Wait(trip_ticks(w, "cart_gallery")));
    let bottom = [3.0, -21.2, 56.0];
    let l10 = lantern_pos(w, "lantern_l10");
    rest_at(w, &mut legs, bottom, l10);
    // The push cart, from L10.
    let stand_cart = [-8.6, -22.0, 77.4];
    goto(w, &mut legs, l10, stand_cart);
    legs.push(Leg::Interact);
    legs.push(Leg::Wait(trip_ticks(w, "cart_smoltorch")));
    // The chamber: the relic, the pick, the locket.
    let relic = [-45.2, -22.0, 78.0];
    goto(w, &mut legs, stand_cart, relic);
    legs.push(Leg::Interact);
    legs.push(Leg::Wait(5));
    let pick = [-42.8, -22.0, 74.6];
    goto(w, &mut legs, relic, pick);
    legs.push(Leg::Interact);
    legs.push(Leg::Wait(5));
    let locket = [-42.8, -22.0, 81.4];
    goto(w, &mut legs, pick, locket);
    legs.push(Leg::Interact);
    legs.push(Leg::Wait(5));
    // On to the exit tunnel.
    goto(w, &mut legs, locket, [0.0, -22.0, 96.0]);
    legs
}

/// Hollowdeep's route: the arrival cave, L11 (rest), the Flask Dew, L12 (rest), L13 (rest), and
/// B09's fog gate.
fn deep_legs(w: &World) -> Vec<Leg> {
    let mut legs = Vec::new();
    let mut at = w.layout.spawn;
    for id in DEEP_LANTERNS {
        let l = lantern_pos(w, id);
        rest_at(w, &mut legs, at, l);
        at = l;
        if id == "lantern_l11" {
            // The Flask Dew by the Lantern.
            let dew = [-57.2, 0.0, -53.2];
            goto(w, &mut legs, at, dew);
            legs.push(Leg::Interact);
            legs.push(Leg::Wait(5));
            at = dew;
        }
    }
    goto(w, &mut legs, at, [104.0, 0.0, 90.0]);
    legs
}

struct Run {
    frames: Vec<ActionFrame>,
    commands: Vec<(u64, Command)>,
    world: World,
    events: Vec<Event>,
    bot: Bot,
}

fn walk(mut w: World, legs: Vec<Leg>) -> Run {
    let mut bot = Bot::with_legs(legs);
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

fn assert_walked(r: &Run) {
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
    assert!(
        r.bot.done(),
        "stopped at leg {} of {} (tick {}, at {:?})",
        r.bot.leg,
        r.bot.legs.len(),
        w.tick,
        p.pos
    );
}

fn mine_walk() -> Run {
    let w = mine();
    let legs = mine_legs(&w);
    walk(w, legs)
}

fn deep_walk() -> Run {
    let w = deep();
    let legs = deep_legs(&w);
    walk(w, legs)
}

/// The bot rides the cart down, rests at L10, pushes the cart into the wall, takes the smoltorch,
/// and walks out the exit tunnel. Taking the relic makes the dark in Hollowdeep a 14 m one.
#[test]
fn the_mine_route_rides_pushes_and_reaches_the_exit() {
    let r = mine_walk();
    assert_walked(&r);
    let w = &r.world;
    assert!(w.state.flags.is_set(&flags::lantern_lit("lantern_l10")));
    assert!(
        w.state
            .flags
            .is_set(&flags::wall_revealed("wall_smoltorch"))
    );
    assert!(
        w.state
            .flags
            .is_set(&flags::pickup_taken("relic_smoltorch"))
    );
    assert!(w.state.flags.is_set(&flags::codex("projects.smoltorch")));
    assert!(w.state.flags.is_set(&flags::pickup_taken("pick_chamber")));
    assert!(w.state.flags.is_set(&flags::pickup_taken("locket_chamber")));
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, Event::Mechanic(m) if m.kind == mine_cart::CART_STARTED)),
        "the ride started"
    );
    let p = w.actors.get(w.player).unwrap();
    assert!((p.pos[2] - 96.0).abs() < 1.5, "ended at {:?}", p.pos);
    assert!(w.tick < 12_000, "took {} ticks", w.tick);
}

/// The bot walks Hollowdeep's Lantern chain to B09's gate, resting at each Lantern, in the glow.
#[test]
fn the_hollowdeep_route_reaches_every_lantern_and_the_gate() {
    let r = deep_walk();
    assert_walked(&r);
    let w = &r.world;
    for id in DEEP_LANTERNS {
        assert!(
            w.state.flags.is_set(&flags::lantern_lit(id)),
            "{id} not lit"
        );
    }
    assert!(w.state.flags.is_set(&flags::pickup_taken("dew_shore")));
    let p = w.actors.get(w.player).unwrap();
    assert!(
        (p.pos[2] - 90.0).abs() < 1.5 && (p.pos[0] - 104.0).abs() < 2.0,
        "ended at {:?}",
        p.pos
    );
    assert!(w.tick < 18_000, "took {} ticks", w.tick);
}

/// Every enemy the spec puts in Hollowdeep and the mine is placed; the relics, materials, and
/// the Echo Stones are in the levels; the bosses are M7 work packages.
#[test]
fn the_levels_have_their_enemies_relics_materials_and_echo_stones() {
    let content = Content::load_embedded().unwrap();
    let deep = content.scenario(DEEP).unwrap();
    for kind in [
        "cave_moth",
        "lamp_mite",
        "husk_miner",
        "shard_golem",
        "glasswing_stalker",
    ] {
        assert!(
            deep.enemies.iter().any(|e| e.enemy == kind),
            "no {kind} in Hollowdeep"
        );
    }
    assert!(deep.enemies.len() < 40, "{} enemies", deep.enemies.len());
    // One ambush per dungeon: the two Glasswing Stalkers are the region's only ambushers.
    assert_eq!(
        deep.enemies
            .iter()
            .filter(|e| e.enemy == "glasswing_stalker")
            .count(),
        2
    );
    let mine = content.scenario(MINE).unwrap();
    assert!(mine.enemies.iter().any(|e| e.enemy == "husk_miner"));
    let items = |sc: &lf_content::Scenario| -> Vec<String> {
        sc.pickups
            .iter()
            .map(|p| p.item.clone())
            .chain(sc.chests.iter().filter_map(|c| c.loot.item.clone()))
            .collect()
    };
    for item in [
        "itm_flask_dew",
        "relic_ferray",
        "relic_endark",
        "itm_forgeglass_chunk",
        "itm_forgeglass_shard",
        "wpn_lantern_sword",
    ] {
        assert!(
            items(deep).iter().any(|i| i == item),
            "no {item} in Hollowdeep"
        );
    }
    for item in ["relic_smoltorch", "itm_forgeglass_shard", "wpn_miners_pick"] {
        assert!(
            items(mine).iter().any(|i| i == item),
            "no {item} in the mine"
        );
    }
    let codex = |sc: &lf_content::Scenario| -> Vec<String> {
        sc.pickups.iter().filter_map(|p| p.codex.clone()).collect()
    };
    for entry in ["projects.ferray", "projects.endark"] {
        assert!(
            codex(deep).contains(&entry.to_owned()),
            "no pickup unlocks {entry}"
        );
    }
    assert!(codex(mine).contains(&"projects.smoltorch".to_owned()));
    // The darkness rule reads the flag the smoltorch pickup raises.
    assert_eq!(
        deep.darkness.as_ref().unwrap().torch_flag,
        "codex.projects.smoltorch"
    );
    // The Echo Stones sit inside the boss arenas, and the world resolves them.
    for (sc, scenario, id, x, z) in [
        (deep, DEEP, "stone_b09", (78.0, 146.0), (94.0, 146.0)),
        (mine, MINE, "stone_b08", (44.0, 78.0), (54.0, 92.0)),
    ] {
        assert_eq!(sc.echo_stones.len(), 1);
        let s = &sc.echo_stones[0];
        assert_eq!(s.id, id);
        assert!(
            s.pos[0] > x.0 && s.pos[0] < x.1 && s.pos[2] > z.0 && s.pos[2] < z.1,
            "{id} is outside its arena: {:?}",
            s.pos
        );
        let w = world(scenario, 1);
        assert_eq!(w.echo_table.stones.len(), 1);
        assert!(
            sc.bosses.is_empty() || id == "stone_b08",
            "only B08 is placed so far; B09 is an M7 work package"
        );
    }
    // The Lanterns, in the order of the spec's checkpoint table.
    assert_eq!(
        mine.lanterns
            .iter()
            .map(|l| l.id.as_str())
            .collect::<Vec<_>>(),
        ["lantern_l10"]
    );
    assert_eq!(
        deep.lanterns
            .iter()
            .map(|l| l.id.as_str())
            .collect::<Vec<_>>(),
        DEEP_LANTERNS
    );
}

fn script_text(r: &Run, scenario: &str, seed: u64, what: &str) -> String {
    let mut text = format!(
        "# WP-062 golden: {what}.\n\
         # Generated by the ignored tests `record_the_*_golden` in\n\
         # game/crates/core/src/world_state/hollowdeep_tests.rs.\n\
         # Each line: <ticks> <move_x> <move_z> [BUTTONS].\n"
    );
    text += &format!("seed {seed}\nscenario {scenario}\n");
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
    text
}

fn write_script(name: &str, text: &str) {
    let path = format!(
        "{}/../../tests/replays/{name}.script",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::write(path, text).unwrap();
}

/// Writes `game/tests/replays/m6_cinderdeep_mine.script`. See the module comment.
#[test]
#[ignore = "writes game/tests/replays/m6_cinderdeep_mine.script"]
fn record_the_cinderdeep_mine_golden() {
    let r = mine_walk();
    assert_walked(&r);
    let text = script_text(
        &r,
        MINE,
        MINE_SEED,
        "the Cinderdeep Mine's route: the mouth, the ride down, L10, the push cart, the smoltorch chamber, the exit",
    );
    write_script("m6_cinderdeep_mine", &text);
}

/// Writes `game/tests/replays/m6_hollowdeep.script`. See the module comment.
#[test]
#[ignore = "writes game/tests/replays/m6_hollowdeep.script"]
fn record_the_hollowdeep_golden() {
    let r = deep_walk();
    assert_walked(&r);
    let text = script_text(
        &r,
        DEEP,
        DEEP_SEED,
        "Hollowdeep's critical path: the arrival cave, L11, L12, L13, B09's fog gate",
    );
    write_script("m6_hollowdeep", &text);
}
