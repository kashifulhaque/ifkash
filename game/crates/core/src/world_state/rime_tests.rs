//! Tests for Rimeshelf, the Whitewarden Catacombs, and the Bound Ring (WP-066): deep snow, the
//! Warden Braziers (and the revive rule they stop), the seal sigil, each area's contents against
//! docs/game/02-world.md, the walks between Lanterns (under 45 seconds at run speed), and scripted
//! bots that walk the region path and both dungeons. The golden replays `m6_rimeshelf`,
//! `m6_whitewarden`, and `m6_bound_ring` are those walks.
//!
//! The bot is the Waking Hollow bot (`hollow_tests::Bot`); its legs come from the baked navigation
//! grid, as in `cave_tests`. Open ground is nowhere near flat in the bot's eyes: the path legs
//! follow the packed spine, so the walk is the one a player would run.
//!
//! Record the goldens: `cargo test -p lf-core record_the_rime_goldens -- --ignored`, then, from
//! `game/`, `cargo run -p lf-tools -- replay record-script tests/replays/m6_rimeshelf.script --out
//! tests/replays/m6_rimeshelf.lfr` (and the same for `m6_whitewarden` and `m6_bound_ring`).

use lf_content::Content;

use super::hollow_tests::{Bot, Leg};
use super::mechanics::{brazier, slumber};
use crate::ai::special::{self, revive::DeathBlow};
use crate::arena::Id;
use crate::events::{Event, LifeEvent};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::nav::Search;
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

const RIME: &str = "rimeshelf";
const WARD: &str = "whitewarden";
const RING: &str = "bound_ring";
const RIME_SEED: u64 = 66;
const WARD_SEED: u64 = 67;
const RING_SEED: u64 = 68;
/// The longest run, in ticks (a stuck run stops here).
const MAX_TICKS: u64 = 40_000;
const BUDGET: u32 = 4_000_000;
/// Acceptance: no walk between Lanterns takes longer than this at run speed, in seconds.
const MAX_WALK_S: f32 = 45.0;

fn world(scenario: &str, seed: u64) -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(seed, &content, scenario).unwrap_or_else(|e| panic!("{scenario}: {e:?}"))
}

fn rime() -> World {
    world(RIME, RIME_SEED)
}

fn ward() -> World {
    world(WARD, WARD_SEED)
}

fn ring() -> World {
    world(RING, RING_SEED)
}

fn player_pos(w: &World) -> [f32; 3] {
    w.actors.get(w.player).unwrap().pos
}

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.vel = [0.0; 3];
}

fn enemy(w: &World, name: &str) -> Id {
    let i = w
        .layout
        .enemies
        .iter()
        .position(|e| e.name == name)
        .unwrap_or_else(|| panic!("no enemy {name}"));
    w.state.ai_enemies[i]
}

fn step(w: &mut World, move_x: i8, move_z: i8, buttons: u16) -> Vec<Event> {
    let input = TickInput {
        frame: ActionFrame {
            move_x,
            move_z,
            buttons,
        },
        commands: Vec::new(),
    };
    tick(w, &input);
    w.events.drain()
}

fn idle(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut out = Vec::new();
    for _ in 0..ticks {
        out.extend(step(w, 0, 0, 0));
    }
    out
}

/// Presses `INTERACT` at `pos` for one tick, then lets go.
fn interact_at(w: &mut World, pos: [f32; 3]) {
    put_player(w, pos);
    step(w, 0, 0, buttons::INTERACT);
    step(w, 0, 0, 0);
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

fn seconds(w: &World, path: &[[f32; 3]]) -> f32 {
    length(path) / w.locomotion.run_speed
}

fn lantern(w: &World, id: &str) -> [f32; 3] {
    w.layout
        .lantern(id)
        .unwrap_or_else(|| panic!("no {id}"))
        .1
        .pos
}

fn taken(w: &World, id: &str) -> bool {
    w.state.flags.is_set(&flags::pickup_taken(id))
}

/// The item IDs an area's `entities.ron` places, in pickups and chests.
fn placed_items(area: &str) -> Vec<String> {
    let content = Content::load_embedded().unwrap();
    content
        .level(area)
        .unwrap()
        .entities
        .entities
        .iter()
        .filter_map(|e| match e {
            lf_content::level::Entity::Pickup(p) => Some(p.item.clone()),
            lf_content::level::Entity::Chest(c) => c.loot.item.clone(),
            _ => None,
        })
        .collect()
}

fn in_snow(w: &World, x: f32, z: f32) -> bool {
    w.layout.snow.iter().any(|s| {
        (x - s.center[0]).abs() <= s.half_extents[0] && (z - s.center[1]).abs() <= s.half_extents[1]
    })
}

/// Whether any point along the straight segments through `stops` is in deep snow.
fn touches_snow(w: &World, stops: &[[f32; 3]]) -> bool {
    stops.windows(2).any(|s| {
        (0..=200).any(|k| {
            let t = k as f32 / 200.0;
            in_snow(
                w,
                s[0][0] + (s[1][0] - s[0][0]) * t,
                s[0][2] + (s[1][2] - s[0][2]) * t,
            )
        })
    })
}

/// The packed route between Lanterns: along the spine (x = 0) to the fork at z = -30, and out
/// along the spur to the Lantern. Each stop is a point on the packed ground.
const FORK: [f32; 3] = [0.0, 0.0, -30.0];

fn packed(w: &World, from: &str, to: &str) -> Vec<[f32; 3]> {
    let a = lantern(w, from);
    let b = lantern(w, to);
    // The spine climbs the Steps' ramp from z = 56 (0.3 m a meter).
    let spine = |z: f32| [0.0, (z - 56.0).max(0.0) * 0.3, z];
    match (from, to) {
        ("lantern_l18", "lantern_l19" | "lantern_l21") => vec![a, FORK, b],
        ("lantern_l19" | "lantern_l21", "lantern_l18") => vec![a, FORK, b],
        ("lantern_l19", "lantern_l21") | ("lantern_l21", "lantern_l19") => vec![a, FORK, b],
        ("lantern_l18", "lantern_l20") => vec![a, spine(70.0), b],
        ("lantern_l19" | "lantern_l21", "lantern_l20") => vec![a, FORK, spine(70.0), b],
        _ => panic!("no packed route {from} -> {to}"),
    }
}

/// The nav path through the stops, joined, and its walking time.
fn via(w: &World, stops: &[[f32; 3]]) -> (Vec<[f32; 3]>, f32) {
    let mut all: Vec<[f32; 3]> = Vec::new();
    for pair in stops.windows(2) {
        let p = walk_path(w, pair[0], pair[1]);
        all.extend(if all.is_empty() { &p[..] } else { &p[1..] });
    }
    let s = seconds(w, &all);
    (all, s)
}

// ---------------------------------------------------------------------------------------
// Rimeshelf: the Lanterns, the walks between them, deep snow

/// The four Lanterns exist, with the IDs the map and the Lantern menu already name.
#[test]
fn rimeshelf_has_lanterns_l18_to_l21() {
    let w = rime();
    for id in ["lantern_l18", "lantern_l19", "lantern_l20", "lantern_l21"] {
        assert!(w.layout.lantern(id).is_some(), "{id}");
    }
    assert_eq!(w.layout.lanterns.len(), 4, "no Lantern but L18-L21");
}

/// Acceptance: every walk between Lanterns (along the packed ground, the way a player runs) takes
/// under 45 seconds at run speed, as does the walk from the arrival gate to L18 and from L20 to
/// B10's fog gate (within 20 seconds). The packed legs touch no deep snow.
#[test]
fn every_walk_between_lanterns_is_under_45_seconds() {
    let w = rime();
    let spawn = w.layout.spawn;
    let (_, s) = via(&w, &[spawn, lantern(&w, "lantern_l18")]);
    assert!(s < 20.0, "arrival to L18: {s} s");
    for (from, to) in [
        ("lantern_l18", "lantern_l19"),
        ("lantern_l18", "lantern_l21"),
        ("lantern_l19", "lantern_l21"),
        ("lantern_l18", "lantern_l20"),
        ("lantern_l19", "lantern_l20"),
        ("lantern_l21", "lantern_l20"),
    ] {
        let stops = packed(&w, from, to);
        assert!(
            !touches_snow(&w, &stops),
            "{from} -> {to} runs through snow"
        );
        let (_, s) = via(&w, &stops);
        assert!(s < MAX_WALK_S, "{from} -> {to}: {s} s on the packed route");
        // The shortest nav path can only be shorter.
        let (_, direct) = via(&w, &[lantern(&w, from), lantern(&w, to)]);
        assert!(direct <= s + 0.1, "{from} -> {to}: direct {direct} s");
    }
    // L20 is within 20 seconds of B10's fog gate (the plane z = 96).
    let (_, s) = via(&w, &[lantern(&w, "lantern_l20"), [0.0, 6.0, 96.0]]);
    assert!(s < 20.0, "L20 to B10's gate: {s} s");
}

/// Deep snow slows the player by a quarter (data/locomotion.ron) and a sprint through it costs
/// half again as much stamina; the packed spine does neither.
#[test]
fn deep_snow_slows_the_player_by_a_quarter() {
    // Run +Z along the spine, then +X through the southeast drift, for the same time.
    let run = |from: [f32; 3], mx: i8, mz: i8| {
        let mut w = rime();
        // Nothing to fight: take the wolves and Husks out.
        for i in 0..w.layout.enemies.len() {
            let id = w.state.ai_enemies[i];
            w.actors.get_mut(id).unwrap().pos = [0.0, -500.0, 0.0];
        }
        put_player(&mut w, from);
        idle(&mut w, 10);
        let a = player_pos(&w);
        for _ in 0..90 {
            step(&mut w, mx, mz, 0);
        }
        let b = player_pos(&w);
        sqrt((b[0] - a[0]).powi(2) + (b[2] - a[2]).powi(2))
    };
    let path = run([0.0, 0.0, -100.0], 0, 127);
    let snow = run([40.0, 0.0, -80.0], 127, 0);
    let ratio = snow / path;
    assert!(
        (0.70..=0.80).contains(&ratio),
        "snow {snow} m against path {path} m: ratio {ratio}"
    );
    let w = rime();
    assert!(in_snow(&w, 40.0, -80.0) && !in_snow(&w, 0.0, -100.0));
    assert_eq!(w.locomotion.mechanics.snow_slow_percent, 25);
    assert_eq!(w.locomotion.mechanics.snow_sprint_stamina_percent, 150);
}

/// A sprint costs more stamina in the snow: 1.5 times the packed ground's.
#[test]
fn a_sprint_through_deep_snow_costs_half_again_as_much_stamina() {
    let spent = |from: [f32; 3], mx: i8, mz: i8| {
        let mut w = rime();
        for i in 0..w.layout.enemies.len() {
            let id = w.state.ai_enemies[i];
            w.actors.get_mut(id).unwrap().pos = [0.0, -500.0, 0.0];
        }
        put_player(&mut w, from);
        idle(&mut w, 10);
        let before = w.actors.get(w.player).unwrap().stamina;
        // Sprint: hold the dodge button after the stick is pushed (the sprint starts after 8
        // ticks held), so the first ticks are the same on both grounds.
        for _ in 0..40 {
            step(&mut w, mx, mz, buttons::DODGE);
        }
        before - w.actors.get(w.player).unwrap().stamina
    };
    let path = spent([0.0, 0.0, -100.0], 0, 127);
    let snow = spent([40.0, 0.0, -80.0], 127, 0);
    assert!(path > 0.0, "the sprint costs something on the path: {path}");
    assert!(snow > path * 1.2, "snow {snow} against path {path}");
}

// ---------------------------------------------------------------------------------------
// Rimeshelf: contents

/// Rimeshelf has what the spec puts in it: the foundry's Smiths and the Frost Husks, the wolves,
/// a Bell Warden, B10's arena (Echo Stone, fog-gate frame; no boss yet), the four connections.
#[test]
fn rimeshelf_has_what_the_spec_puts_in_it() {
    let w = rime();
    let content = Content::load_embedded().unwrap();
    let area = &content.level(RIME).unwrap().area;
    let kinds = |n: &str| w.layout.enemies.iter().filter(|e| e.enemy == n).count();
    assert!(kinds("foundry_smith") >= 2, "the foundry's Smiths");
    assert!(kinds("husk_frost") >= 4);
    assert!(kinds("rime_wolf") >= 6, "wolves hunt in packs");
    assert!(kinds("bell_warden") >= 1);
    assert!(w.layout.bosses.is_empty(), "B10 has no definition yet");
    assert_eq!(w.layout.snow.len(), 5);
    // B10's Echo Stone is inside the gate (the plane z = 96), on the terrace.
    let stones = &content.level(RIME).unwrap().entities;
    let stone = stones
        .entities
        .iter()
        .find_map(|e| match e {
            lf_content::level::Entity::EchoStone(s) => Some(s.clone()),
            _ => None,
        })
        .expect("an Echo Stone");
    assert!(stone.pos[2] > 96.0 && (stone.pos[1] - 6.0).abs() < 0.01);
    // Four connections: the castle twice (the back gate and the chimney), the catacombs, the Ring.
    let to = |n: &str| area.connections.iter().filter(|c| c.to == n).count();
    assert_eq!(to("castle"), 2);
    assert_eq!(to("whitewarden"), 1);
    assert_eq!(to("bound_ring"), 1);
    // Materials and key items the spec places in Rimeshelf (docs/game/04, "Materials and key
    // items"): a Wick Bead and the Mail satchel in the Pass, a Forgeglass Chunk, the foundry's
    // Flask Dew; weapons, a charm, and armor from the tables.
    let item_ids = placed_items(RIME);
    for want in [
        "itm_wick_bead",
        "itm_mail_satchel",
        "itm_forgeglass_chunk",
        "itm_flask_dew",
        "wpn_iron_knuckles",
        "wpn_rimeforged_claymore",
        "chm_rimeward",
        "arm_rimeforged_head",
        "arm_rimeforged_body",
        "itm_art_scroll_rime_edge",
    ] {
        assert!(item_ids.iter().any(|i| i == want), "no {want} in Rimeshelf");
    }
    // The Courier's quest names the satchel's pickup; Chronicle Page 4 stays unplaced (the
    // portfolio has four work entries today).
    assert!(w.layout.pickups.iter().any(|p| p.id == "satchel_rimeshelf"));
    assert!(w.layout.pickups.iter().all(|p| p.id != "page_chronicle_4"));
    // The two ways to the castle exist: the back gate is the castle's, `gate_castle_back`.
    let castle = world("castle", 1);
    assert!(
        castle
            .layout
            .doors
            .iter()
            .any(|d| d.id == "gate_castle_back")
    );
}

/// Every pickup and chest in the three areas puts its item in the player's pack (the registry
/// knows every ID, including the armor and the stubs).
#[test]
fn every_item_the_three_areas_place_can_be_taken() {
    for (area, seed) in [(RIME, RIME_SEED), (WARD, WARD_SEED), (RING, RING_SEED)] {
        let content = Content::load_embedded().unwrap();
        let entities = content.level(area).unwrap().entities.entities.clone();
        world(area, seed);
        for e in &entities {
            let item = match e {
                lf_content::level::Entity::Pickup(p) => Some(p.item.clone()),
                lf_content::level::Entity::Chest(c) => c.loot.item.clone(),
                _ => None,
            };
            if let Some(item) = item {
                assert!(
                    content.registry.contains(&item),
                    "{area}: item {item} isn't in the registry"
                );
            }
        }
    }
}

/// The foundry: the east door opens on the casting floor, the doorway in the divider leads to the
/// annex, and a second doorway to the office. All reachable on foot from L20's spine.
#[test]
fn the_foundry_is_walkable_from_its_yard_to_the_office() {
    let w = rime();
    let yard = [-8.0, 0.0, 7.0];
    for to in [
        [-22.0, 0.0, 7.0],  // the casting floor, just inside the door
        [-58.0, 1.2, 6.0],  // the anvil with the claymore
        [-30.0, 0.0, 36.0], // the chimney room
        [-20.0, 0.0, 36.0], // the Rimeforged Mail chest
        [-50.0, 0.0, 40.0], // the office
        [-46.0, 0.0, 48.0], // the Forgeglass Chunk
    ] {
        let p = walk_path(&w, yard, [to[0], 0.0, to[2]]);
        assert!(length(&p) < 140.0, "{to:?}: {} m", length(&p));
    }
    // The yard is on the spine's side: from L18 it's an easy walk.
    let (_, s) = via(&w, &[lantern(&w, "lantern_l18"), yard]);
    assert!(s < MAX_WALK_S, "L18 to the foundry's door: {s} s");
}

/// The chimney drops the player into the castle: they step off the pit's lip, fall the 6 m (under
/// the 8 m the fall damage starts at), and land inside the area's connection to the castle.
#[test]
fn the_chimney_drops_into_the_castle_connection() {
    let mut w = rime();
    let content = Content::load_embedded().unwrap();
    let area = &content.level(RIME).unwrap().area;
    let hole = area
        .connections
        .iter()
        .find(|c| c.to == "castle" && c.pos[1] < -1.0)
        .expect("the chimney's connection")
        .pos;
    // Stand on the south lip, facing north, and walk in.
    put_player(&mut w, [-22.0, 0.0, 38.4]);
    w.actors.get_mut(w.player).unwrap().yaw = 0.0;
    let hp = w.actors.get(w.player).unwrap().hp;
    let mut nearest = f32::MAX;
    for _ in 0..180 {
        step(&mut w, 0, 127, 0);
        let p = player_pos(&w);
        let d =
            sqrt((p[0] - hole[0]).powi(2) + (p[1] - hole[1]).powi(2) + (p[2] - hole[2]).powi(2));
        nearest = nearest.min(d);
    }
    assert!(
        nearest <= 2.5,
        "the player came within {nearest} m of the connection"
    );
    assert_eq!(
        w.actors.get(w.player).unwrap().hp,
        hp,
        "a 6 m fall costs nothing"
    );
}

// ---------------------------------------------------------------------------------------
// Warden Braziers and the revive

fn skeleton_down_by(w: &mut World, name: &str, blow: DeathBlow) -> Id {
    // Away from the fight: nobody sees the player.
    put_player(w, [0.0, 0.0, -92.0]);
    let id = enemy(w, name);
    let a = w.actors.get_mut(id).unwrap();
    a.hp = 0;
    a.dead = true;
    special::note_death_blow(w, id, blow);
    id
}

fn revives(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Event::Revive(_)))
        .count()
}

/// The teaching room: a skeleton killed beside an unlit brazier stands up in 180 ticks; one killed
/// beside a lit brazier stays down.
#[test]
fn a_lit_brazier_keeps_a_skeleton_down_and_an_unlit_one_does_not() {
    let mut w = ward();
    skeleton_down_by(&mut w, "skel_ash", DeathBlow::default());
    assert_eq!(revives(&idle(&mut w, 200)), 1, "unlit: it gets back up");

    let mut w = ward();
    w.state.flags.raise(&brazier::lit_flag("brz_ash"));
    let sk = skeleton_down_by(&mut w, "skel_ash", DeathBlow::default());
    assert_eq!(
        revives(&idle(&mut w, 400)),
        0,
        "lit and near: it stays down"
    );
    assert!(w.actors.get(sk).unwrap().dead);
}

/// A brazier lit while the body lies there stops it rising; a far one doesn't count; one that is
/// up stays up (the flame doesn't kill).
#[test]
fn lighting_a_brazier_stops_the_bodies_beside_it_but_not_those_far_away() {
    let mut w = ward();
    let sk = skeleton_down_by(&mut w, "skel_ash", DeathBlow::default());
    idle(&mut w, 100);
    // Light the brazier (5 m from the skeleton's spawn) with INTERACT, from beside it.
    interact_at(&mut w, [4.0, 0.0, -63.2]);
    put_player(&mut w, [0.0, 0.0, -92.0]);
    assert!(brazier::is_lit(&w, "brz_ash"));
    assert_eq!(revives(&idle(&mut w, 300)), 0, "lit in time: it stays down");
    assert!(w.actors.get(sk).unwrap().dead);

    // The Walk's far brazier is 30 m from the Ash skeleton: lighting it doesn't save that one.
    let mut w = ward();
    let sk = skeleton_down_by(&mut w, "skel_ash", DeathBlow::default());
    w.state.flags.raise(&brazier::lit_flag("brz_walk_b"));
    assert_eq!(
        revives(&idle(&mut w, 200)),
        1,
        "a far brazier doesn't count"
    );
    assert!(!w.actors.get(sk).unwrap().dead);

    // A skeleton that is up stays up beside a lit brazier.
    let mut w = ward();
    w.state.flags.raise(&brazier::lit_flag("brz_ash"));
    let sk = enemy(&w, "skel_ash");
    idle(&mut w, 30);
    assert!(!w.actors.get(sk).unwrap().dead);
}

/// Radiance or a critical keeps a skeleton down with or without a brazier (the spec's other two
/// ways), and the prompt offers the brazier until it is lit.
#[test]
fn radiance_and_criticals_still_work_and_the_brazier_offers_a_prompt() {
    for blow in [
        DeathBlow {
            radiance: true,
            critical: false,
        },
        DeathBlow {
            radiance: false,
            critical: true,
        },
    ] {
        let mut w = ward();
        let sk = skeleton_down_by(&mut w, "skel_walk_a1", blow);
        assert_eq!(revives(&idle(&mut w, 400)), 0, "{blow:?}");
        assert!(w.actors.get(sk).unwrap().dead);
    }
    let mut w = ward();
    put_player(&mut w, [4.0, 0.0, -63.2]);
    idle(&mut w, 2);
    assert_eq!(super::mechanics::prompt(&w), brazier::PROMPT_LIGHT);
    let ev = {
        step(&mut w, 0, 0, buttons::INTERACT);
        w.events.drain()
    };
    drop(ev);
    idle(&mut w, 2);
    assert!(brazier::is_lit(&w, "brz_ash"));
    assert_eq!(super::mechanics::prompt(&w), 0, "lit: nothing to offer");
}

/// The Vigil: six skeletons sleep until the chest is opened; four braziers stand in the corners
/// and every lit one quiets those beside it.
#[test]
fn the_vigil_wakes_when_the_chest_is_opened() {
    let mut w = ward();
    let names = [
        "skel_vigil_a",
        "skel_vigil_b",
        "skel_vigil_c",
        "skel_vigil_d",
        "skel_vigil_e",
        "skel_vigil_f",
    ];
    for n in names {
        assert!(slumber::asleep(&w, enemy(&w, n)), "{n} sleeps");
    }
    interact_at(&mut w, [0.0, 0.0, 6.8]);
    assert!(w.state.flags.is_set(&flags::chest_opened("chest_vigil")));
    for n in names {
        assert!(!slumber::asleep(&w, enemy(&w, n)), "{n} should be awake");
    }
    // Each corner brazier is within 6 m of the two skeletons beside it.
    for (b, s) in [
        ("brz_vigil_a", "skel_vigil_a"),
        ("brz_vigil_b", "skel_vigil_b"),
        ("brz_vigil_c", "skel_vigil_c"),
        ("brz_vigil_d", "skel_vigil_d"),
    ] {
        let bp = w.layout.braziers.iter().find(|x| x.id == b).unwrap().pos;
        let sp = w.layout.enemies.iter().find(|x| x.name == s).unwrap().pos;
        let d = sqrt((bp[0] - sp[0]).powi(2) + (bp[2] - sp[2]).powi(2));
        assert!(d <= 6.0, "{b} to {s}: {d} m");
    }
}

/// What the spec puts in D6: the Twin Wardens (two Bell Wardens), skeletons, braziers in the
/// teaching room, test room, and twist room, the Whitewarden Seal, Wide Mend, a Rewrite Quill, and
/// the Lost Folio (the Archivist's quest pickup), a one-way shortcut, and no Lantern inside.
#[test]
fn whitewarden_has_what_the_spec_puts_in_d6() {
    let w = ward();
    let content = Content::load_embedded().unwrap();
    let area = &content.level(WARD).unwrap().area;
    assert!(area.connections.iter().any(|c| c.to == "rimeshelf"));
    assert!(
        w.layout.lanterns.is_empty(),
        "no Lantern inside a mini-dungeon"
    );
    let kinds = |n: &str| w.layout.enemies.iter().filter(|e| e.enemy == n).count();
    assert_eq!(kinds("bell_warden"), 2, "the Twin Wardens");
    assert!(kinds("whitewarden_skeleton") >= 12);
    assert!(w.layout.braziers.len() >= 8);
    for id in [
        "seal_whitewarden",
        "folio_whitewarden",
        "quill_ossuary",
        "spl_wide_mend_ossuary",
    ] {
        assert!(w.layout.pickups.iter().any(|p| p.id == id), "{id}");
    }
    let seal = w
        .layout
        .pickups
        .iter()
        .find(|p| p.id == "seal_whitewarden")
        .unwrap();
    assert_eq!(
        Some(seal.item),
        content.registry.num("wpn_whitewarden_seal")
    );
    // The Twins are deep in the Vault: 24 m or more from its door, beyond a Bell Warden's sight.
    for n in ["warden_twin_a", "warden_twin_b"] {
        let p = w.layout.enemies.iter().find(|e| e.name == n).unwrap().pos;
        assert!(p[2] - 28.0 >= 24.0, "{n} at {p:?}");
    }
}

// ---------------------------------------------------------------------------------------
// The seal sigil

/// The seal door stands open until the sigil is touched; an unarmed sigil is cold: touching it
/// raises the flag and nothing shuts.
#[test]
fn a_cold_sigil_can_be_touched_and_shuts_nothing() {
    let mut w = ring();
    idle(&mut w, 2);
    assert!(w.state.flags.is_set(&flags::door_open("door_ring_seal")));
    put_player(&mut w, [0.0, 0.0, -16.0]);
    idle(&mut w, 2);
    assert_eq!(super::mechanics::prompt(&w), brazier::PROMPT_TOUCH);
    step(&mut w, 0, 0, buttons::INTERACT);
    idle(&mut w, 3);
    assert!(w.state.flags.is_set(&brazier::touched_flag("sigil_ring")));
    assert!(!w.state.flags.is_set(&brazier::sealed_flag("sigil_ring")));
    assert!(w.state.flags.is_set(&flags::door_open("door_ring_seal")));
    assert!(!w.layout.sigils[0].armed, "B11 doesn't exist yet");
}

/// An armed sigil seals: touching it shuts the door behind the player (they can't walk out), the
/// prompt goes, and the world's release opens it again.
#[test]
fn an_armed_sigil_seals_the_door_until_it_is_released() {
    let mut w = ring();
    w.layout.sigils[0].armed = true;
    idle(&mut w, 2);
    // Walk in through the door: the way is open.
    put_player(&mut w, [0.0, 0.0, -40.0]);
    for _ in 0..240 {
        step(&mut w, 0, 127, 0);
    }
    assert!(player_pos(&w)[2] > -16.0, "walked in: {:?}", player_pos(&w));
    // Touch the sigil.
    put_player(&mut w, [0.0, 0.0, -16.0]);
    idle(&mut w, 2);
    let ev = step(&mut w, 0, 0, buttons::INTERACT);
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::Mechanic(m) if m.kind == brazier::SIGIL_TOUCHED)),
        "touched"
    );
    let ev2 = idle(&mut w, 3);
    assert!(
        ev.iter()
            .chain(ev2.iter())
            .any(|e| matches!(e, Event::Mechanic(m) if m.kind == brazier::SIGIL_SEALED)),
        "sealed"
    );
    assert!(w.state.flags.is_set(&brazier::sealed_flag("sigil_ring")));
    assert!(!w.state.flags.is_set(&flags::door_open("door_ring_seal")));
    assert_eq!(super::mechanics::prompt(&w), 0, "nothing more to touch");
    // They can't leave: walking south, they stop at the door.
    put_player(&mut w, [0.0, 0.0, -18.0]);
    for _ in 0..240 {
        step(&mut w, 0, -127, 0);
    }
    assert!(player_pos(&w)[2] > -23.0, "sealed in: {:?}", player_pos(&w));
    // The prisoner falls: the world releases the seal and the door sinks again.
    w.state.flags.raise(&brazier::released_flag("sigil_ring"));
    idle(&mut w, 3);
    assert!(w.state.flags.is_set(&flags::door_open("door_ring_seal")));
    for _ in 0..240 {
        step(&mut w, 0, -127, 0);
    }
    assert!(
        player_pos(&w)[2] < -26.0,
        "walked out: {:?}",
        player_pos(&w)
    );
    // The renderer's words: the brazier and sigil words end the state (no braziers here: one sigil).
    let all = super::mechanics::render_state(&w);
    assert_eq!(*all.last().unwrap(), 1.0, "touched, no longer sealed");
}

/// What the spec puts in D7: the sealed arena (a ring of standing stones, the sigil, the plinth),
/// the rewards behind it, and no Lantern inside (L21 stands outside, in Rimeshelf).
#[test]
fn the_bound_ring_has_what_the_spec_puts_in_d7() {
    let w = ring();
    let content = Content::load_embedded().unwrap();
    let level = content.level(RING).unwrap();
    assert!(level.area.connections.iter().any(|c| c.to == "rimeshelf"));
    assert!(w.layout.lanterns.is_empty());
    assert!(w.layout.bosses.is_empty(), "B11 has no definition yet");
    let stones = level
        .layout
        .landmarks
        .iter()
        .filter(|l| l.name.starts_with("standing_stone"))
        .count();
    assert_eq!(stones, 12, "a ring of twelve standing stones");
    for id in [
        "wpn_exile_stylus_plinth",
        "spl_unbinding_lance_alcove",
        "chm_last_stand_alcove",
        "arm_exile_head_alcove",
    ] {
        assert!(w.layout.pickups.iter().any(|p| p.id == id), "{id}");
    }
    assert_eq!(
        w.layout.chests.len(),
        2,
        "the Exile's Robe and the Starglass"
    );
    assert_eq!(w.layout.sigils.len(), 1);
}

// ---------------------------------------------------------------------------------------
// The scripted walks

fn goto(w: &World, legs: &mut Vec<Leg>, at: [f32; 3], to: [f32; 3]) {
    for p in &walk_path(w, at, to)[1..] {
        legs.push(Leg::Goto(p[0], p[2]));
    }
}

/// A short press of `INTERACT`, then a pause long enough for a press edge.
fn press(legs: &mut Vec<Leg>) {
    legs.push(Leg::Interact);
    legs.push(Leg::Wait(4));
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
    let p = player_pos(w);
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, Event::Life(LifeEvent::Died { .. }))),
        "the bot died at tick {} near {:?}, on leg {} of {}",
        w.tick,
        p,
        r.bot.leg,
        r.bot.legs.len()
    );
    assert!(
        r.bot.done(),
        "stopped at leg {} of {} (tick {}, at {:?})",
        r.bot.leg,
        r.bot.legs.len(),
        w.tick,
        p
    );
}

/// Rimeshelf's region path: the arrival gate, L18 (rest), the fork, the east spur to L19 (rest),
/// back and west to L21 (rest), then the spine north past the foundry's yard and up Bellwright's
/// Steps to L20 (rest) and B10's fog-gate frame, with a few things taken on the way. The Pass's
/// wolves and Husks near the spine are fought; the foundry's Smiths are not on it.
fn rime_legs(w: &World) -> Vec<Leg> {
    let mut legs = Vec::new();
    let at = std::cell::Cell::new(w.layout.spawn);
    let to = |legs: &mut Vec<Leg>, p: [f32; 3]| {
        goto(w, legs, at.get(), p);
        at.set(p);
    };
    // L18, with the Wick Bead on the path.
    to(&mut legs, [3.0, 0.0, -88.0]);
    press(&mut legs);
    to(&mut legs, lantern(w, "lantern_l18"));
    legs.push(Leg::Rest);
    // The Pass to the fork, the east spur, L19.
    to(&mut legs, FORK);
    to(&mut legs, [60.0, 0.0, -30.0]);
    to(&mut legs, lantern(w, "lantern_l19"));
    legs.push(Leg::Rest);
    // The west spur to L21.
    to(&mut legs, FORK);
    to(&mut legs, [-70.0, 0.0, -30.0]);
    to(&mut legs, lantern(w, "lantern_l21"));
    legs.push(Leg::Rest);
    // North past the foundry's yard, up the Steps, to L20 and the gate's frame.
    to(&mut legs, FORK);
    to(&mut legs, [0.0, 0.0, 50.0]);
    to(&mut legs, [0.0, 6.0, 78.0]);
    to(&mut legs, lantern(w, "lantern_l20"));
    legs.push(Leg::Rest);
    to(&mut legs, [0.0, 6.0, 97.0]);
    legs
}

/// The Catacombs' route: the Hall of Ash (light the brazier, kill the skeleton), the Ossuary Walk
/// (both braziers lit, four skeletons), the Quill Ossuary (the three pickups), the sleeping
/// Vigil, and to the Vault's door.
fn ward_legs(w: &World) -> Vec<Leg> {
    let mut legs = Vec::new();
    let at = std::cell::Cell::new(w.layout.spawn);
    let to = |legs: &mut Vec<Leg>, p: [f32; 3]| {
        goto(w, legs, at.get(), p);
        at.set(p);
    };
    // The Hall of Ash: the brazier first, then the skeleton comes to the bot, who fights it on the
    // lit ground.
    to(&mut legs, [4.0, 0.0, -63.2]);
    press(&mut legs);
    legs.push(Leg::Wait(60));
    // The Ossuary Walk: light the first brazier by the way in, fight the pair, light the second.
    to(&mut legs, [-5.0, 0.0, -36.0]);
    press(&mut legs);
    to(&mut legs, [5.0, 0.0, -23.0]);
    press(&mut legs);
    // The Quill Ossuary (the branch): its brazier first (a skeleton stands beside it), then the
    // three pickups.
    to(&mut legs, [-29.8, 0.0, -16.9]);
    press(&mut legs);
    to(&mut legs, [-36.0, 0.0, -25.0]);
    press(&mut legs);
    to(&mut legs, [-40.0, 0.0, -28.6]);
    press(&mut legs);
    to(&mut legs, [-30.0, 0.0, -28.6]);
    press(&mut legs);
    // The Vigil: straight through, up the middle. The chest in the middle wakes six skeletons, so
    // the bot leaves it shut (the test `the_vigil_wakes_when_the_chest_is_opened` opens it).
    to(&mut legs, [0.0, 0.0, -4.0]);
    to(&mut legs, [0.0, 0.0, 18.0]);
    // The Vault's door.
    to(&mut legs, [0.0, 0.0, 32.0]);
    legs
}

/// The Ring's route: the road, the alcove's charm, in through the doorway to the sigil, touch it,
/// then on past the plinth to the reward alcove and the stylus.
fn ring_legs() -> Vec<Leg> {
    // The Ring has no enemies, so no navigation grid: straight legs between known-clear points.
    let mut legs = vec![Leg::Goto(0.0, -46.0), Leg::Goto(-12.0, -46.0)];
    press(&mut legs);
    legs.extend([Leg::Goto(0.0, -46.0), Leg::Goto(0.0, -17.0)]);
    press(&mut legs);
    // Round the prisoner's plinth, up the passage to the reward alcove.
    legs.extend([
        Leg::Goto(5.0, -8.0),
        Leg::Goto(5.0, 22.0),
        Leg::Goto(2.0, 32.0),
        Leg::Goto(0.0, 41.0),
        Leg::Goto(0.0, 42.8),
    ]);
    press(&mut legs);
    legs
}

fn rime_walk() -> Run {
    let w = rime();
    let legs = rime_legs(&w);
    walk(w, legs)
}

fn ward_walk() -> Run {
    let w = ward();
    let legs = ward_legs(&w);
    walk(w, legs)
}

fn ring_walk() -> Run {
    walk(ring(), ring_legs())
}

/// The bot walks the region path alive: it rests at all four Lanterns and reaches B10's gate.
#[test]
fn the_rimeshelf_region_path_is_walkable() {
    let r = rime_walk();
    assert_walked(&r);
    let w = &r.world;
    for id in ["lantern_l18", "lantern_l19", "lantern_l20", "lantern_l21"] {
        assert!(
            w.state.flags.is_set(&flags::lantern_lit(id)),
            "{id} was lit"
        );
    }
    assert!(taken(w, "bead_pass"));
    let p = player_pos(w);
    assert!(
        (p[2] - 97.0).abs() < 1.0 && (p[1] - 6.0).abs() < 0.5,
        "{p:?}"
    );
}

/// The bot walks the catacombs to the Vault's door alive: every brazier it meant to light is lit,
/// the branch's three pickups are taken, and the Vigil slept through.
#[test]
fn the_whitewarden_critical_path_is_walkable() {
    let r = ward_walk();
    assert_walked(&r);
    let w = &r.world;
    for b in ["brz_ash", "brz_walk_a", "brz_walk_b", "brz_quill"] {
        assert!(brazier::is_lit(w, b), "{b}");
    }
    for p in [
        "quill_ossuary",
        "folio_whitewarden",
        "spl_wide_mend_ossuary",
    ] {
        assert!(taken(w, p), "{p}");
    }
    assert!(
        !w.state.flags.is_set(&flags::chest_opened("chest_vigil")),
        "the Vigil sleeps"
    );
    assert!(player_pos(w)[2] > 30.0);
}

/// The bot walks the Ring: in, touches the cold sigil, takes the charm, the stylus.
#[test]
fn the_bound_ring_critical_path_is_walkable() {
    let r = ring_walk();
    assert_walked(&r);
    let w = &r.world;
    assert!(w.state.flags.is_set(&brazier::touched_flag("sigil_ring")));
    for p in ["chm_last_stand_alcove", "wpn_exile_stylus_plinth"] {
        assert!(taken(w, p), "{p}");
    }
}

fn script_text(r: &Run, scenario: &str, seed: u64, what: &str) -> String {
    let mut text = format!(
        "# WP-066 golden: {what}.\n\
         # Generated by the ignored test `record_the_rime_goldens` in\n\
         # game/crates/core/src/world_state/rime_tests.rs.\n\
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

/// Writes `game/tests/replays/m6_rimeshelf.script`, `m6_whitewarden.script`, and
/// `m6_bound_ring.script`. See the module comment.
#[test]
#[ignore = "writes game/tests/replays/m6_*.script"]
fn record_the_rime_goldens() {
    let r = rime_walk();
    assert_walked(&r);
    write_script(
        "m6_rimeshelf",
        &script_text(
            &r,
            RIME,
            RIME_SEED,
            "Rimeshelf's region path: the arrival gate, L18, L19, L21, the spine past the foundry, Bellwright's Steps, L20, and B10's fog-gate frame",
        ),
    );
    let r = ward_walk();
    assert_walked(&r);
    write_script(
        "m6_whitewarden",
        &script_text(
            &r,
            WARD,
            WARD_SEED,
            "the Whitewarden Catacombs' critical path: the braziers, the skeletons, the Quill Ossuary, the Vigil, and the Twins' Vault's door",
        ),
    );
    let r = ring_walk();
    assert_walked(&r);
    write_script(
        "m6_bound_ring",
        &script_text(
            &r,
            RING,
            RING_SEED,
            "the Bound Ring: the road, the charm, the sigil, the plinth, and the reward alcove",
        ),
    );
}
