//! Tests for Mirrormere (WP-060): the walks between Lanterns are short and stay on dry ground,
//! the Cathedral's tower shows from L06, the lake is a wall until the Belfry's sluice opens and
//! then the ford wades, and a scripted bot walks the main route from the Verge road to every
//! Lantern. The golden replay `m6_mirrormere` is that walk.
//!
//! The bot is the Waking Hollow bot (`hollow_tests::Bot`); its legs come from the baked
//! navigation grid, as in `verge_tests`.
//!
//! Record the golden: `cargo test -p lf-core record_the_mirrormere_golden -- --ignored`, then
//! `cargo run -p lf-tools -- replay record-script tests/replays/m6_mirrormere.script --out
//! tests/replays/m6_mirrormere.lfr` (from `game/`).

use lf_content::Content;

use super::hollow_tests::{Bot, Leg};
use super::mechanics::water_level;
use crate::events::{Event, LifeEvent};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::nav::Search;
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

const SCENARIO: &str = "mirrormere";
const SEED: u64 = 60;
/// The longest run, in ticks (the walk takes about 4,500; this only stops a stuck run).
const MAX_TICKS: u64 = 30_000;
/// The Lantern chain, in walking order.
const LANTERNS: [&str; 4] = ["lantern_l06", "lantern_l07", "lantern_l08", "lantern_l09"];
/// Acceptance: no walk between Lanterns takes longer than this at run speed, in seconds.
const MAX_WALK_S: f32 = 45.0;
/// A path request's node budget.
const BUDGET: u32 = 2_000_000;
/// Water deeper than this is a wall (`locomotion.ron`, `mechanics.water_deep_over_m`).
const DEEP_M: f32 = 1.4;
/// How far inside a pool's rectangle a path point has to be to count as in the water. The nav
/// grid's smoothed paths cut a corner of the lip by under a meter (at the causeway's head);
/// the wall only slides the player along it.
const LIP_SLACK_M: f32 = 1.0;

fn world() -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(SEED, &content, SCENARIO).expect("mirrormere exists")
}

fn lantern_pos(w: &World, id: &str) -> [f32; 3] {
    w.layout
        .lanterns
        .iter()
        .find(|l| l.id == id)
        .unwrap_or_else(|| panic!("no Lantern {id}"))
        .pos
}

/// The smoothed nav path between two points, as waypoints.
fn walk_path(w: &World, from: [f32; 3], to: [f32; 3]) -> Vec<[f32; 3]> {
    w.nav
        .as_ref()
        .expect("Mirrormere has enemies, so it has a nav grid")
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

/// Whether a point is inside a pool that is deep now (an invisible wall for the player).
fn in_deep_water(w: &World, x: f32, z: f32) -> bool {
    w.layout.pools.iter().enumerate().any(|(i, p)| {
        (x - p.center[0]).abs() < p.half_extents[0] - LIP_SLACK_M
            && (z - p.center[1]).abs() < p.half_extents[1] - LIP_SLACK_M
            && water_level::depth(w, i) > DEEP_M
    })
}

/// Whether the path, sampled every half meter, enters deep water.
fn path_enters_deep_water(w: &World, path: &[[f32; 3]]) -> bool {
    path.windows(2).any(|s| {
        let (dx, dz) = (s[1][0] - s[0][0], s[1][2] - s[0][2]);
        let d = sqrt(dx * dx + dz * dz);
        let n = (d / 0.5).ceil().max(1.0) as u32;
        (0..=n).any(|k| {
            let t = k as f32 / n as f32;
            in_deep_water(w, s[0][0] + dx * t, s[0][2] + dz * t)
        })
    })
}

/// Every walk along the Lantern chain, and from the Verge road to L06, takes under 45 seconds
/// at run speed with no content in the way, and none of them wades into deep water (the nav
/// grid knows the ford is walkable ground; the player can't use it until the sluice opens, so
/// the main route must not need it).
#[test]
fn every_walk_between_lanterns_is_under_45_seconds() {
    let w = world();
    let mut stops: Vec<(String, [f32; 3])> = vec![("the Verge road".into(), w.layout.spawn)];
    for id in LANTERNS {
        stops.push((id.into(), lantern_pos(&w, id)));
    }
    for pair in stops.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let path = walk_path(&w, a.1, b.1);
        let s = seconds(&w, &path);
        println!("{} -> {}: {s:.1} s", a.0, b.0);
        assert!(s < MAX_WALK_S, "{} -> {} takes {s:.1} s", a.0, b.0);
        assert!(s > 8.0, "{} -> {} is only {s:.1} s: too close", a.0, b.0);
        assert!(
            !path_enters_deep_water(&w, &path),
            "{} -> {} wades into deep water: {path:?}",
            a.0,
            b.0
        );
    }
    // L09 is within 20 seconds of B06's fog gate (docs/game/02-world.md, Placement rules).
    let gate = [85.0, 0.0, 19.0];
    let l09 = lantern_pos(&w, "lantern_l09");
    let s = seconds(&w, &walk_path(&w, l09, gate));
    println!("lantern_l09 -> the nave's fog gate: {s:.1} s");
    assert!(s < 20.0, "L09 to the fog gate takes {s:.1} s");
    // The side branches are reachable from the Lantern they leave: the chief's house, the
    // islet's wading pool, the marsh's far reeds, and the Cathedral's forecourt.
    let (l07, l08) = (
        lantern_pos(&w, "lantern_l07"),
        lantern_pos(&w, "lantern_l08"),
    );
    for (what, from, to) in [
        ("the chief's house", l07, [-92.0, 0.0, -10.0]),
        ("the islet shore", l07, [-134.0, 0.0, -27.0]),
        ("the marsh chest", l07, [-118.0, 0.0, 84.0]),
        ("the north marsh", l08, [-88.0, 0.0, 100.0]),
        ("the forecourt", l09, [85.0, 0.0, 24.0]),
    ] {
        let s = seconds(&w, &walk_path(&w, from, to));
        println!("{what}: {s:.1} s");
        assert!(s < MAX_WALK_S, "{what} takes {s:.1} s");
    }
}

/// The Cathedral's bell tower is in a straight line of sight from L06: no collider stands
/// between the Lantern and the silhouette. The lake is a basin, so nothing hides it.
#[test]
fn the_cathedral_tower_is_visible_from_l06() {
    let w = world();
    let l06 = lantern_pos(&w, "lantern_l06");
    let eye = [l06[0], l06[1] + 1.6, l06[2]];
    for (what, target) in [
        ("spire tip", [104.0, 55.0, 24.0]),
        ("belfry", [102.0, 40.0, 22.0]),
        ("tower shoulder", [100.5, 30.0, 20.5]),
    ] {
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let dist = sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
        let dir = [d[0] / dist, d[1] / dist, d[2] / dist];
        // A hit on the tower's own face just before the target point is fine.
        let hit = w.physics.raycast(eye, dir, dist + 20.0);
        assert!(
            hit.is_none_or(|h| h > dist - 6.0),
            "something blocks the view of the tower's {what} at {hit:?} m of {dist:.0}"
        );
    }
}

/// Drives the player toward (x, z) for `ticks` ticks with no buttons.
fn push_toward(w: &mut World, x: f32, z: f32, ticks: u32) {
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

fn player_pos(w: &World) -> [f32; 3] {
    w.actors.get(w.player).unwrap().pos
}

/// The lake's depths follow the Belfry's sluice flag: the ford is a wall at first and wades
/// after, and the open lake stays a wall.
#[test]
fn the_sluice_lowers_the_lake_by_one_and_a_half_meters() {
    let mut w = world();
    let ford = w
        .layout
        .pools
        .iter()
        .position(|p| p.name == "ford")
        .unwrap();
    let mere = w
        .layout
        .pools
        .iter()
        .position(|p| p.name == "mere_west_south")
        .unwrap();
    let (ford0, mere0) = (water_level::depth(&w, ford), water_level::depth(&w, mere));
    assert!(!water_level::sluice_open(&w, "mirrormere"));
    water_level::open_sluice(&mut w, "mirrormere");
    assert_eq!(
        w.state.flags.get("sluice.mirrormere.open"),
        1,
        "the flag the Belfry sets"
    );
    let (ford1, mere1) = (water_level::depth(&w, ford), water_level::depth(&w, mere));
    // Every lake pool and the ford lose the same 1.5 m of surface (depth is above the floor).
    for (i, p) in w.layout.pools.iter().enumerate() {
        if p.lowered_by.is_some() {
            let after = water_level::depth(&w, i);
            assert!(
                (p.depth_m - after - 1.5).abs() < 1e-4,
                "{} drops {} m",
                p.name,
                p.depth_m - after
            );
        }
    }
    assert!(
        ford0 > DEEP_M && ford1 < 0.6,
        "the ford: {ford0} -> {ford1}"
    );
    assert!(
        mere0 > DEEP_M && mere1 > DEEP_M,
        "the lake: {mere0} -> {mere1}"
    );
    // The renderer gets the new depths in its state words.
    let words = crate::world_state::mechanics::render_state(&w);
    assert!(words.contains(&ford1) && words.contains(&mere1));
}

/// The invisible wall holds the player at the lip of the ford until the sluice opens; then
/// they wade across to the Cathedral's side door. They can't wade the open lake at all.
#[test]
fn the_ford_is_a_wall_until_the_sluice_opens() {
    let mut w = world();
    // The village shore, at the ford's west end, facing east; every enemy is out of the way.
    w.actors.get_mut(w.player).unwrap().pos = [-66.0, 0.0, 23.0];
    push_toward(&mut w, 66.0, 23.0, 400);
    let p = player_pos(&w);
    assert!(p[0] < -59.5, "the deep ford let the player in: {p:?}");
    water_level::open_sluice(&mut w, "mirrormere");
    push_toward(&mut w, 66.0, 23.0, 1_500);
    let p = player_pos(&w);
    assert!(p[0] > 62.0, "the player never crossed the ford: {p:?}");
    assert!(p[1] > -0.5, "the player should be on the island: {p:?}");
    // The lowered lake is still a wall: walk from the island straight north into the water
    // beside the causeway.
    w.actors.get_mut(w.player).unwrap().pos = [72.0, 0.0, 28.0];
    push_toward(&mut w, 72.0, 70.0, 300);
    let p = player_pos(&w);
    assert!(p[2] < 31.5, "the lowered lake let the player in: {p:?}");
}

/// The shortcut the sluice opens: with the lake lowered, L07 to the Cathedral's side door is a
/// short wade, shorter than the walk around by L08 and the causeway.
#[test]
fn the_lowered_ford_is_a_shortcut_from_the_village_to_the_side_door() {
    let w = world();
    let l07 = lantern_pos(&w, "lantern_l07");
    let door = [66.0, 0.0, 23.0];
    let direct = walk_path(&w, l07, door);
    let s = seconds(&w, &direct);
    println!("L07 -> the side door by the ford: {s:.1} s");
    assert!(s < MAX_WALK_S, "the ford walk takes {s:.1} s");
    // The path is a straight line along the ford, so look for a waypoint on each side of its
    // middle rather than a waypoint in it.
    assert!(
        direct
            .iter()
            .any(|p| p[0] < -55.0 && (p[2] - 23.0).abs() < 4.0)
            && direct
                .iter()
                .any(|p| p[0] > 55.0 && (p[2] - 23.0).abs() < 4.0),
        "the nav path doesn't use the ford: {direct:?}"
    );
    // Without the ford, the way round is the main route: L07 -> L08 -> L09 -> the forecourt.
    let l08 = lantern_pos(&w, "lantern_l08");
    let l09 = lantern_pos(&w, "lantern_l09");
    let round = seconds(&w, &walk_path(&w, l07, l08))
        + seconds(&w, &walk_path(&w, l08, l09))
        + seconds(&w, &walk_path(&w, l09, [85.0, 0.0, 24.0]));
    println!("L07 -> the forecourt the long way: {round:.1} s");
    assert!(
        s < round * 0.75,
        "the ford saves too little: {s:.1} s vs {round:.1} s"
    );
}

/// Walks the nav path from `at` to `to`, leaving off the first point (where it stands).
fn goto(w: &World, legs: &mut Vec<Leg>, at: [f32; 3], to: [f32; 3]) {
    for p in &walk_path(w, at, to)[1..] {
        legs.push(Leg::Goto(p[0], p[2]));
    }
}

/// One leg-by-leg walk: the Verge road, each Lantern (rest at it), in order. After L07 the
/// route goes into the chief's house for Chronicle Page 2 and the Wick Bead, then on to L08.
fn legs(w: &World) -> Vec<Leg> {
    let mut legs = Vec::new();
    let mut at = w.layout.spawn;
    for id in LANTERNS {
        let l = lantern_pos(w, id);
        // Stop one meter short of the Lantern, on the side the path arrives from.
        let path = walk_path(w, at, l);
        for p in &path[1..path.len() - 1] {
            legs.push(Leg::Goto(p[0], p[2]));
        }
        let (last, prev) = (path[path.len() - 1], path[path.len() - 2]);
        let (dx, dz) = (prev[0] - last[0], prev[2] - last[2]);
        let d = sqrt(dx * dx + dz * dz).max(0.001);
        legs.push(Leg::Goto(last[0] + dx / d, last[2] + dz / d));
        legs.push(Leg::Rest);
        legs.push(Leg::Wait(10));
        at = l;
        if id == "lantern_l07" {
            // The chief's house: in at the east door, the Page on the table, the bead by it.
            let page = [-97.0, 0.0, -10.0];
            goto(w, &mut legs, at, page);
            legs.push(Leg::Interact);
            legs.push(Leg::Wait(5));
            let bead = [-94.6, 0.0, -11.6];
            goto(w, &mut legs, page, bead);
            legs.push(Leg::Interact);
            legs.push(Leg::Wait(5));
            at = bead;
        }
    }
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
fn the_main_route_reaches_every_lantern() {
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
    for id in LANTERNS {
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
        "stopped at leg {} of {}",
        r.bot.leg,
        r.bot.legs.len()
    );
    // The side prizes on the route: Chronicle Page 2 and the Wick Bead in the chief's house.
    assert!(
        w.state
            .flags
            .is_set(&flags::pickup_taken("page_chronicle_2"))
    );
    assert!(w.state.flags.is_set(&flags::pickup_taken("bead_chief")));
    assert!(w.state.flags.is_set(&flags::codex("work.fiery")));
    assert_eq!(
        p.inventory.flasks.max_charges, 5,
        "the Wick Bead adds a charge"
    );
    // The walk stays under five minutes of game time.
    assert!(w.tick < 18_000, "took {} ticks", w.tick);
}

/// Every enemy the spec puts in Mirrormere is placed, the relic, the Page, the key items, and
/// B06's Echo Stone are in the level, and B06 itself is an M7 work package.
#[test]
fn mirrormere_has_its_enemies_relics_and_echo_stone() {
    let content = Content::load_embedded().unwrap();
    let level = content.scenario(SCENARIO).unwrap();
    for kind in [
        "drowned",
        "reed_stalker",
        "marsh_crawler",
        "acolyte_bell",
        "choir_knight",
    ] {
        assert!(
            level.enemies.iter().any(|e| e.enemy == kind),
            "no {kind} in Mirrormere"
        );
    }
    assert_eq!(
        level
            .enemies
            .iter()
            .filter(|e| e.enemy == "choir_knight")
            .count(),
        1,
        "exactly one Choir Knight"
    );
    assert!(level.enemies.len() < 40, "{} enemies", level.enemies.len());
    let codex: Vec<&str> = level
        .pickups
        .iter()
        .filter_map(|p| p.codex.as_deref())
        .collect();
    for entry in ["projects.nopokedb", "work.fiery"] {
        assert!(codex.contains(&entry), "no pickup unlocks {entry}");
    }
    let items: Vec<&str> = level.pickups.iter().map(|p| p.item.as_str()).collect();
    for item in ["itm_wick_bead", "itm_flask_dew", "itm_mail_satchel"] {
        assert!(items.contains(&item), "no {item} in Mirrormere");
    }
    // The Echo Stone is in the level format, inside the nave, and the world resolves it.
    assert_eq!(level.echo_stones.len(), 1);
    let stone = &level.echo_stones[0];
    assert_eq!(stone.id, "stone_b06");
    assert!(
        stone.pos[0] > 68.0 && stone.pos[0] < 104.0 && stone.pos[2] > -17.0 && stone.pos[2] < 17.0,
        "the stone is outside the nave: {:?}",
        stone.pos
    );
    let w = world();
    assert_eq!(w.echo_table.stones.len(), 1);
    assert!(level.bosses.is_empty(), "B06 is an M7 work package");
}

/// Blight water builds Blight in the Reed Marsh, and the village flood only slows.
#[test]
fn the_marsh_builds_blight_and_the_village_does_not() {
    let blight_after = |pos: [f32; 3]| {
        let mut w = world();
        w.actors.get_mut(w.player).unwrap().pos = pos;
        for _ in 0..120 {
            let input = TickInput {
                frame: ActionFrame::default(),
                commands: Vec::new(),
            };
            tick(&mut w, &input);
            w.events.drain();
        }
        let p = w.actors.get(w.player).unwrap();
        let kind = lf_content::moves::StatusKind::Blight;
        p.status.meter_milli[crate::combat::status::index(kind)]
            + u32::from(p.status.active_ticks(kind))
    };
    assert!(
        blight_after([-100.0, 0.0, 66.0]) > 0,
        "the marsh builds no Blight"
    );
    assert_eq!(
        blight_after([-104.0, 0.0, -20.0]),
        0,
        "the village builds Blight"
    );
}

/// Writes `game/tests/replays/m6_mirrormere.script`. See the module comment.
#[test]
#[ignore = "writes game/tests/replays/m6_mirrormere.script"]
fn record_the_mirrormere_golden() {
    let r = walk();
    assert!(r.bot.done());
    let mut text = String::from(
        "# WP-060 golden: the main route of Mirrormere, from the Verge road through L06, L07, L08,\n\
         # L09, resting at each Lantern.\n\
         # Generated by the ignored test `record_the_mirrormere_golden` in\n\
         # game/crates/core/src/world_state/mirrormere_tests.rs.\n\
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
        "/../../tests/replays/m6_mirrormere.script"
    );
    std::fs::write(path, text).unwrap();
}
