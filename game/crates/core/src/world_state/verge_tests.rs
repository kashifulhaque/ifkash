//! Tests for the Verge (WP-044): the walks between Lanterns are short, the Spire shows from
//! L02, and a scripted bot walks the main route from the root-door to every Lantern. The
//! golden replay `m4_verge` is that walk.
//!
//! The bot is the Waking Hollow bot (`hollow_tests::Bot`): it fights the nearest awake enemy
//! with a roll-through-the-hit rule and otherwise follows its legs. The legs come from the
//! baked navigation grid, so the route is the one an enemy would walk.
//!
//! Record the golden: `cargo test -p lf-core record_the_verge_golden -- --ignored`, then
//! `cargo run -p lf-tools -- replay record-script tests/replays/m4_verge.script --out
//! tests/replays/m4_verge.lfr` (from `game/`).

use lf_content::Content;

use super::hollow_tests::{Bot, Leg};
use crate::events::{Event, LifeEvent};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::nav::Search;
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

const SCENARIO: &str = "verge";
const SEED: u64 = 44;
/// The longest run, in ticks (the walk takes about 4,500; this only stops a stuck run).
const MAX_TICKS: u64 = 30_000;
/// The Lantern chain, in walking order.
const LANTERNS: [&str; 4] = ["lantern_l02", "lantern_l03", "lantern_l04", "lantern_l05"];
/// Acceptance: no walk between Lanterns takes longer than this at run speed, in seconds.
const MAX_WALK_S: f32 = 45.0;
/// A path request's node budget: the grid has 636,000 cells.
const BUDGET: u32 = 2_000_000;

fn world() -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(SEED, &content, SCENARIO).expect("verge exists")
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
        .expect("the Verge has enemies, so it has a nav grid")
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

/// Every walk between two Lanterns, and from the root-door to L02, takes under 45 seconds
/// at run speed, with no content (enemies, doors) in the way. Counts the climb to the
/// aqueduct top and the walk to the Briar Thicket's gate as well.
#[test]
fn every_walk_between_lanterns_is_under_45_seconds() {
    let w = world();
    let spawn = w.layout.spawn;
    let l02 = lantern_pos(&w, "lantern_l02");
    let s = seconds(&w, &walk_path(&w, spawn, l02));
    println!("the root-door -> lantern_l02: {s:.1} s");
    assert!(s < MAX_WALK_S, "the root-door to L02 takes {s:.1} s");
    let mut stops: Vec<(String, [f32; 3])> = Vec::new();
    for id in LANTERNS {
        stops.push((id.into(), lantern_pos(&w, id)));
    }
    let mut longest = 0.0f32;
    for (i, (a, pa)) in stops.iter().enumerate() {
        for (b, pb) in &stops[i + 1..] {
            let s = seconds(&w, &walk_path(&w, *pa, *pb));
            println!("{a} -> {b}: {s:.1} s");
            longest = longest.max(s);
            assert!(s < MAX_WALK_S, "{a} -> {b} takes {s:.1} s");
        }
    }
    // The side branches are reachable from the Lantern they leave: the Briar Thicket's gate
    // (west side of the arena), the Windmill's foot, the Spire's door.
    let l03 = lantern_pos(&w, "lantern_l03");
    let l05 = lantern_pos(&w, "lantern_l05");
    for (what, from, to) in [
        ("Thicket gate", l03, [108.0, 0.0, -80.0]),
        ("Windmill foot", l05, [-62.0, 0.0, 30.0]),
        ("Spire door", l05, [-40.0, 0.0, 164.0]),
    ] {
        let s = seconds(&w, &walk_path(&w, from, to));
        println!("{what}: {s:.1} s");
        assert!(s < MAX_WALK_S, "{what} takes {s:.1} s");
    }
    // The aqueduct loop: L03 up the ramp to the chapel roof platform.
    let up = walk_path(&w, l03, [90.0, 6.0, 48.0]);
    println!("aqueduct top: {:.1} s", seconds(&w, &up));
    assert!(seconds(&w, &up) < MAX_WALK_S);
    assert!(
        longest > 10.0,
        "the regions' Lanterns should not be on top of each other"
    );
}

/// The Spire's tip and its base's top are in a straight line of sight from L02: no collider
/// stands between the Lantern and the silhouette.
#[test]
fn the_spire_is_visible_from_l02() {
    let w = world();
    let l02 = lantern_pos(&w, "lantern_l02");
    let eye = [l02[0], l02[1] + 1.6, l02[2]];
    for (what, target) in [
        ("tip", [-40.0, 98.0, 182.0]),
        ("upper shaft", [-40.0, 60.0, 176.0]),
        ("shoulder", [-37.0, 20.0, 170.0]),
    ] {
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let dist = sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
        let dir = [d[0] / dist, d[1] / dist, d[2] / dist];
        // A hit on the Spire's own face just before the target point is fine.
        let hit = w.physics.raycast(eye, dir, dist + 20.0);
        assert!(
            hit.is_none_or(|h| h > dist - 14.0),
            "something blocks the view of the Spire's {what} at {:?} m of {dist:.0}",
            hit
        );
    }
    // And the Spire is tall enough to read: its tip is far above the eye.
    let top = w
        .physics
        .raycast([-40.0, 110.0, 150.0], [0.0, -1.0, 0.0], 200.0);
    assert!(top.is_some());
}

/// Walks the nav path from `at` to `to`, leaving off the last point.
fn goto(w: &World, legs: &mut Vec<Leg>, at: [f32; 3], to: [f32; 3]) {
    let path = walk_path(w, at, to);
    for p in &path[1..] {
        legs.push(Leg::Goto(p[0], p[2]));
    }
}

/// One leg-by-leg walk: root-door, each Lantern (rest at it), in order. After L04 the route
/// takes the Chronicle Page from the altar and goes into the crypt, strikes the hollow
/// wall, and takes banana.cpp, then goes on to L05.
fn legs(w: &World) -> Vec<Leg> {
    let mut legs = Vec::new();
    let mut at = w.layout.spawn;
    let rest = |legs: &mut Vec<Leg>, w: &World, at: &mut [f32; 3], id: &str| {
        let l = lantern_pos(w, id);
        // Stop one meter short of the Lantern, on the side the path arrives from.
        let path = walk_path(w, *at, l);
        for p in &path[1..path.len() - 1] {
            legs.push(Leg::Goto(p[0], p[2]));
        }
        let (last, prev) = (path[path.len() - 1], path[path.len() - 2]);
        let (dx, dz) = (prev[0] - last[0], prev[2] - last[2]);
        let d = sqrt(dx * dx + dz * dz).max(0.001);
        legs.push(Leg::Goto(last[0] + dx / d, last[2] + dz / d));
        legs.push(Leg::Rest);
        legs.push(Leg::Wait(10));
        *at = l;
    };
    for id in ["lantern_l02", "lantern_l03", "lantern_l04"] {
        rest(&mut legs, w, &mut at, id);
    }
    // The Chronicle Page on the altar (90, 53).
    let page = [90.0, 0.0, 52.2];
    goto(w, &mut legs, at, page);
    legs.push(Leg::Interact);
    legs.push(Leg::Wait(5));
    // The crypt: stand 1.4 m from the hollow wall (x = 122), swing, wait for the dust, then
    // take the relic from the closet.
    let strike = [120.6, 0.0, 46.0];
    goto(w, &mut legs, page, strike);
    legs.push(Leg::Swing);
    legs.push(Leg::Wait(40));
    legs.push(Leg::Goto(126.4, 46.0));
    legs.push(Leg::Interact);
    legs.push(Leg::Wait(5));
    at = [126.4, 0.0, 46.0];
    rest(&mut legs, w, &mut at, "lantern_l05");
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
    // The side prizes on the route: the Page, the hollow wall, and banana.cpp.
    assert!(
        w.state
            .flags
            .is_set(&flags::pickup_taken("page_chronicle_1"))
    );
    assert!(w.state.flags.is_set(&flags::wall_revealed("wall_crypt")));
    assert!(
        w.state
            .flags
            .is_set(&flags::pickup_taken("relic_banana_cpp"))
    );
    assert!(w.state.flags.is_set(&flags::codex("work.corteva")));
    assert!(w.state.flags.is_set(&flags::codex("projects.banana-cpp")));
    // The walk stays under five minutes of game time.
    assert!(w.tick < 18_000, "took {} ticks", w.tick);
}

/// Every enemy the spec puts in the Verge is placed, and the elite is at the windmill.
#[test]
fn the_verge_has_its_enemies_and_relics() {
    let content = Content::load_embedded().unwrap();
    let level = content.scenario(SCENARIO).unwrap();
    for kind in [
        "husk",
        "thorn_hound",
        "lantern_thief",
        "acolyte",
        "bramble_brute",
    ] {
        assert!(
            level.enemies.iter().any(|e| e.enemy == kind),
            "no {kind} in the Verge"
        );
    }
    assert_eq!(
        level
            .enemies
            .iter()
            .filter(|e| e.enemy == "bramble_brute")
            .count(),
        1,
        "exactly one Bramble Brute"
    );
    assert!(level.enemies.len() < 40, "{} enemies", level.enemies.len());
    let codex: Vec<&str> = level
        .pickups
        .iter()
        .filter_map(|p| p.codex.as_deref())
        .collect();
    for entry in [
        "projects.banana-cpp",
        "projects.vicharak-micro-llama",
        "work.corteva",
    ] {
        assert!(codex.contains(&entry), "no pickup unlocks {entry}");
    }
    assert!(level.bosses.is_empty(), "B02 is an M7 work package");
}

/// Writes `game/tests/replays/m4_verge.script`. See the module comment.
#[test]
#[ignore = "writes game/tests/replays/m4_verge.script"]
fn record_the_verge_golden() {
    let r = walk();
    assert!(r.bot.done());
    let mut text = String::from(
        "# WP-044 golden: the main route of the Verge, from the root-door through L02, L03, L04,\n\
         # L05, resting at each Lantern, plus the Chronicle Page and banana.cpp.\n\
         # Generated by the ignored test `record_the_verge_golden` in\n\
         # game/crates/core/src/world_state/verge_tests.rs.\n\
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
        "/../../tests/replays/m4_verge.script"
    );
    std::fs::write(path, text).unwrap();
}
