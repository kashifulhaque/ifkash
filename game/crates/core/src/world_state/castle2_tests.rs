//! Tests for Castle Graymantle, part 2 (WP-065): the walks between L15, L16, L17, and B07's fog gate
//! are under 45 seconds; the keep's two routes, the library's ladder and hollow wall, the bell
//! tower, and the corridor to the antechamber can be walked; the second shortcut (the rampart lift)
//! opens from the court and then runs both ways; the back gate is a flag-keyed door that only the
//! world's flag opens; and a scripted bot walks from the arrival road through L14, L15, and L16 to
//! L17 and B07's fog gate. The golden replay `m6_castle_part2` is that walk.
//!
//! Part 1's helpers live in `castle_tests`.
//!
//! Record the golden: `cargo test -p lf-core record_the_castle_part2_golden -- --ignored`, then
//! `cargo run -p lf-tools -- replay record-script tests/replays/m6_castle_part2.script --out
//! tests/replays/m6_castle_part2.lfr` (from `game/`).

use lf_content::Content;

use super::castle_tests::{
    MAX_WALK_S, STAIR_FOOT, goto, lantern_pos, legs, player_pos, press, push_toward, put_player,
    quiet_world, seconds, walk_path, world,
};
use super::hollow_tests::{Bot, Leg};
use crate::events::{Event, LifeEvent};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

/// B07's fog gate: the plane z = 340 across the gap in the antechamber's north wall.
const B07_GATE: [f32; 3] = [0.0, 12.0, 340.0];
/// The way into the keep, at the court's north edge.
const KEEP_DOOR: [f32; 3] = [0.0, 12.0, 260.0];
/// The top of the lift, on the bastion slab; the bottom, in the cage on the Lower Ward's floor.
const LIFT_TOP: [f32; 3] = [18.0, 12.0, 208.0];
const LIFT_BOTTOM: [f32; 3] = [18.0, 0.0, 198.0];
/// The belfry's floor, where the Art Scroll lies.
const BELFRY: [f32; 3] = [-14.0, 38.0, 291.0];
/// The longest run, in ticks.
const MAX_TICKS: u64 = 14_000;

fn secs(w: &World, a: [f32; 3], b: [f32; 3]) -> f32 {
    seconds(w, &walk_path(w, a, b))
}

/// Every walk between Lanterns is under 45 seconds at run speed; L17 is within 20 seconds of B07's
/// fog gate; the shortcut's two walking legs are each under 45 seconds, and together too.
#[test]
fn every_walk_between_lanterns_is_under_45_seconds() {
    let w = world();
    let (l15, l16, l17) = (
        lantern_pos(&w, "lantern_l15"),
        lantern_pos(&w, "lantern_l16"),
        lantern_pos(&w, "lantern_l17"),
    );
    for (what, a, b) in [
        ("L15 -> L16", l15, l16),
        ("L16 -> L15", l16, l15),
        ("L16 -> L17", l16, l17),
        ("L17 -> L16", l17, l16),
    ] {
        let s = secs(&w, a, b);
        println!("{what}: {s:.1} s");
        assert!(s < MAX_WALK_S, "{what} takes {s:.1} s");
    }
    let s = secs(&w, l17, B07_GATE);
    println!("L17 -> B07's fog gate: {s:.1} s");
    assert!(s < 20.0, "L17 to the fog gate takes {s:.1} s");
    // The second shortcut: L16 to the lift's top, the ride, and the lift's bottom to L15.
    let up = secs(&w, l16, LIFT_TOP);
    let down = secs(&w, LIFT_BOTTOM, l15);
    let ride = 12.0 / w.locomotion.mechanics.lift_speed_mps;
    println!("L16 -> lift {up:.1} s, ride {ride:.1} s, lift -> L15 {down:.1} s");
    assert!(up < MAX_WALK_S && down < MAX_WALK_S);
    assert!(
        up + down < MAX_WALK_S,
        "the shortcut walks {:.1} s",
        up + down
    );
    // The rampart route: the Lower Ward's wall-walk (its north end) to L16.
    let s = secs(&w, [-50.0, 7.0, 208.0], l16);
    println!("the wall-walk's end -> L16: {s:.1} s");
    assert!(s < MAX_WALK_S);
    // The side branches are reachable from L16 within 45 seconds.
    for (what, to) in [
        ("the library by the dining stair", [-24.0, 18.0, 283.5]),
        ("the library's west door", [-31.0, 18.0, 286.0]),
        ("the Gradient Rod", [0.0, 18.0, 278.2]),
        ("the corridor's foot", [21.0, 12.0, 320.0]),
        ("the belfry", BELFRY),
        ("the Boo alcove", [30.5, 18.0, 272.5]),
        ("the east perch", [41.0, 16.0, 249.5]),
        ("the west perch", [-41.0, 16.0, 247.0]),
        ("the captain's greatsword", [36.0, 12.0, 234.0]),
    ] {
        let s = secs(&w, l16, to);
        println!("L16 -> {what}: {s:.1} s");
        assert!(s < MAX_WALK_S, "{what} takes {s:.1} s");
    }
    let s = secs(&w, l17, [0.0, 12.3, 378.2]);
    println!("L17 -> the Castellan's desk: {s:.1} s");
    assert!(s < MAX_WALK_S);
}

/// Walks the nav path to `to`, driving the real player along it.
fn walk_to(w: &mut World, to: [f32; 3]) {
    let from = player_pos(w);
    for p in &walk_path(w, from, to)[1..] {
        for _ in 0..12 {
            push_toward(w, p[0], p[2], 40);
            let q = player_pos(w);
            if (q[0] - p[0]).abs() < 0.5 && (q[2] - p[2]).abs() < 0.5 {
                break;
            }
        }
    }
}

/// The keep's two ways up: the dining stair inside, and the west rampart stair along the wall; the
/// bell tower's stair to the belfry; the library's north door to the corridor down to the
/// antechamber. Each is walked by the real player along the nav path.
#[test]
fn the_keep_the_tower_and_the_corridor_can_be_walked() {
    let mut w = quiet_world();
    put_player(&mut w, KEEP_DOOR);
    for (what, to, y) in [
        ("the dining stair's top", [-24.0, 18.0, 283.5], 18.0),
        ("the corridor's foot", [21.0, 12.0, 320.0], 12.0),
        ("L17", lantern_pos(&w, "lantern_l17"), 12.0),
    ] {
        walk_to(&mut w, to);
        let p = player_pos(&w);
        assert!(
            (p[1] - y).abs() < 0.5 && (p[0] - to[0]).abs() < 1.5 && (p[2] - to[2]).abs() < 1.5,
            "never reached {what}: at {p:?}"
        );
    }
    // The west rampart stair, from the court to the library's west door.
    put_player(&mut w, [-31.0, 12.0, 258.0]);
    walk_to(&mut w, [-31.0, 18.0, 286.0]);
    let p = player_pos(&w);
    assert!(
        (p[1] - 18.0).abs() < 0.5 && p[2] > 283.0,
        "the rampart stair did not reach the landing: {p:?}"
    );
    push_toward(&mut w, -20.0, 283.8, 120);
    let p = player_pos(&w);
    assert!(
        p[0] > -24.0 && (p[1] - 18.0).abs() < 0.5,
        "not in the library: {p:?}"
    );
    // The bell tower: in by its door, round the well twice, out on the belfry.
    put_player(&mut w, [-18.8, 18.0, 284.0]);
    walk_to(&mut w, BELFRY);
    let p = player_pos(&w);
    assert!(
        (p[1] - 38.0).abs() < 0.5
            && (p[0] - BELFRY[0]).abs() < 1.5
            && (p[2] - BELFRY[2]).abs() < 1.5,
        "never reached the belfry: {p:?}"
    );
    // The belfry's parapet keeps the player on it.
    push_toward(&mut w, -14.0, 320.0, 200);
    let p = player_pos(&w);
    assert!(p[2] < 300.0 && p[1] > 37.0, "off the tower: {p:?}");
}

/// The ladder at the library's south wall climbs to the upper shelf.
#[test]
fn the_ladder_climbs_to_the_upper_shelf() {
    let mut w = quiet_world();
    put_player(&mut w, [-10.0, 18.0, 269.5]);
    press(&mut w, buttons::INTERACT);
    assert!(w.mech.climb.is_some(), "the ladder did not grab");
    for _ in 0..600 {
        if w.mech.climb.is_none() {
            break;
        }
        // Up is toward the ladder's facing (-Z).
        let input = TickInput {
            frame: ActionFrame {
                move_x: 0,
                move_z: -127,
                buttons: 0,
            },
            commands: Vec::new(),
        };
        tick(&mut w, &input);
        w.events.drain();
    }
    assert!(w.mech.climb.is_none(), "never reached the top");
    let p = player_pos(&w);
    assert!((p[1] - 22.0).abs() < 0.3, "not on the gallery: {p:?}");
    push_toward(&mut w, -4.0, 266.5, 120);
    let p = player_pos(&w);
    assert!(
        p[1] > 21.7 && (p[2] - 266.5).abs() < 1.0,
        "off the gallery: {p:?}"
    );
}

/// A hit on the library's east wall (behind the last bookcase) reveals the Boo alcove.
#[test]
fn the_hollow_wall_opens_the_boo_alcove() {
    let mut w = quiet_world();
    let revealed = flags::wall_revealed("wall_library_boo");
    put_player(&mut w, [24.5, 18.0, 273.0]);
    // The wall holds until it is struck.
    push_toward(&mut w, 31.0, 273.0, 60);
    assert!(player_pos(&w)[0] < 27.0, "walked through the wall");
    assert!(!w.state.flags.is_set(&revealed));
    for _ in 0..6 {
        press(&mut w, buttons::LIGHT);
        wait(&mut w, 30);
        if w.state.flags.is_set(&revealed) {
            break;
        }
    }
    assert!(
        w.state.flags.is_set(&revealed),
        "a swing did not reveal the wall"
    );
    push_toward(&mut w, 31.0, 272.5, 160);
    let p = player_pos(&w);
    assert!(
        p[0] > 29.0 && (p[1] - 18.0).abs() < 0.5,
        "not in the alcove: {p:?}"
    );
}

/// Waits `ticks` with no input.
fn wait(w: &mut World, ticks: u32) {
    for _ in 0..ticks {
        tick(w, &TickInput::default());
        w.events.drain();
    }
}

/// The second shortcut: the cage's door opens only from inside, the lift is called from the
/// court, and once the door is open the lift runs both ways.
#[test]
fn the_rampart_lift_opens_from_the_court_and_runs_both_ways() {
    let mut w = quiet_world();
    // From the Lower Ward the cage door is shut and stays shut.
    put_player(&mut w, [18.0, 0.0, 198.0]);
    press(&mut w, buttons::INTERACT);
    assert!(!w.state.flags.is_set(&flags::door_open("door_lift_cage")));
    push_toward(&mut w, 18.0, 206.0, 150);
    assert!(
        player_pos(&w)[2] < 200.6,
        "walked through the shut cage door"
    );
    // From the court: call the lift with the lever at its top stop.
    put_player(&mut w, [18.0, 12.0, 207.4]);
    press(&mut w, buttons::INTERACT);
    assert!(w.state.flags.is_set(&flags::lever_pulled("lever_lift_top")));
    wait(&mut w, 330);
    assert!(
        (w.mech.lifts[0].rise - 12.0).abs() < 0.05,
        "the lift is at {}",
        w.mech.lifts[0].rise
    );
    // Step on, and ride down with the same lever.
    push_toward(&mut w, 18.0, 204.8, 60);
    let p = player_pos(&w);
    assert!(
        (p[1] - 12.0).abs() < 0.4 && p[2] < 205.3,
        "not on the platform: {p:?}"
    );
    press(&mut w, buttons::INTERACT);
    wait(&mut w, 330);
    let p = player_pos(&w);
    assert!(p[1] < 0.5, "the lift did not bring the player down: {p:?}");
    // The cage's door opens from inside.
    push_toward(&mut w, 18.0, 202.0, 60);
    press(&mut w, buttons::INTERACT);
    assert!(
        w.state.flags.is_set(&flags::door_open("door_lift_cage")),
        "the door did not open from inside"
    );
    push_toward(&mut w, 18.0, 190.0, 200);
    assert!(player_pos(&w)[2] < 199.0, "stuck in the cage");
    // Now the Lower Ward can use it: in through the open door, and up with the bottom lever.
    push_toward(&mut w, 18.0, 203.5, 200);
    press(&mut w, buttons::INTERACT);
    assert!(
        w.state
            .flags
            .is_set(&flags::lever_pulled("lever_lift_bottom"))
    );
    wait(&mut w, 330);
    let p = player_pos(&w);
    assert!(p[1] > 11.5, "the lift did not carry the player up: {p:?}");
}

/// The back gate is shut until the world raises its flag, and only that opens it.
#[test]
fn the_back_gate_opens_only_on_its_flag() {
    let mut w = quiet_world();
    let flag = flags::door_open("gate_castle_back");
    // The dais's east side, past the desk, beside the gate.
    put_player(&mut w, [2.0, 12.3, 383.0]);
    press(&mut w, buttons::INTERACT);
    assert!(!w.state.flags.is_set(&flag), "INTERACT opened the gate");
    push_toward(&mut w, 0.0, 392.0, 200);
    assert!(player_pos(&w)[2] < 384.8, "walked through the shut gate");
    // B07 raises the flag when Voss falls (`ai::bosses::b07`).
    w.state.flags.set(&flag, 1);
    wait(&mut w, 5);
    push_toward(&mut w, 0.0, 392.0, 240);
    let p = player_pos(&w);
    assert!(p[2] > 388.0, "the open gate leads nowhere: {p:?}");
    // The passage ends at the north curtain.
    push_toward(&mut w, 0.0, 420.0, 200);
    assert!(player_pos(&w)[2] < 398.0);
}

/// Part 2's enemies, items, relic, Chronicle Page, Echo Stone, and mechanics are placed.
#[test]
fn the_castle_has_its_second_half() {
    let content = Content::load_embedded().unwrap();
    let level = content.scenario("castle").unwrap();
    for id in ["lantern_l14", "lantern_l15", "lantern_l16", "lantern_l17"] {
        assert!(level.lanterns.iter().any(|l| l.id == id), "no {id}");
    }
    // Ramparts: archers, one captain; the dining hall's four sleepers are the only ambush.
    let count = |kind: &str| level.enemies.iter().filter(|e| e.enemy == kind).count();
    assert_eq!(count("gray_captain"), 1);
    assert!(count("gray_archer") >= 3);
    let sleepers: Vec<_> = level
        .enemies
        .iter()
        .filter(|e| e.wakes_on.is_some())
        .collect();
    assert_eq!(sleepers.len(), 4, "one ambush: the dining hall's");
    for s in &sleepers {
        assert!(s.name.starts_with("servant_dining"));
        assert_eq!(s.wakes_on.as_deref(), Some("pickup.chunk_dining.taken"));
    }
    assert!(
        level.pickups.iter().any(|p| p.id == "chunk_dining"),
        "the feast wakes them"
    );
    // Items: the spec's library, ramparts, keep, tower, and desk.
    let items: Vec<&str> = level.pickups.iter().map(|p| p.item.as_str()).collect();
    for item in [
        "itm_flask_dew",
        "chm_parriers_eye",
        "wpn_gray_greatsword",
        "itm_forgeglass_chunk",
        "wpn_gradient_rod",
        "spl_homing_motes",
        "spl_aether_wave",
        "itm_rewrite_quill",
        "itm_lost_folio",
        "relic_boo",
        "itm_art_scroll_windward_parry",
        "chronicle_page_3",
        "spl_gradient_descent",
    ] {
        assert!(items.contains(&item), "no {item} in the castle");
    }
    let page = level
        .pickups
        .iter()
        .find(|p| p.item == "chronicle_page_3")
        .unwrap();
    assert_eq!(page.codex.as_deref(), Some("work.american-express"));
    assert!(
        page.pos[2] > 372.0 && page.pos[2] < 384.0,
        "the page is off the dais"
    );
    let boo = level
        .pickups
        .iter()
        .find(|p| p.item == "relic_boo")
        .unwrap();
    assert_eq!(boo.codex.as_deref(), Some("projects.boo"));
    // B07's Echo Stone is inside the hall, past the fog gate.
    let stone = level
        .echo_stones
        .iter()
        .find(|s| s.id == "stone_b07")
        .unwrap();
    assert!(
        stone.pos[0].abs() < 22.0 && stone.pos[2] > B07_GATE[2] && stone.pos[2] < 384.0,
        "the stone is outside the hall: {:?}",
        stone.pos
    );
    assert_eq!(world().echo_table.stones.len(), 2);
    // Castellan Voss (WP-075) waits mid-hall, behind the fog gate; B03 (WP-071) holds the bridge.
    let voss = level
        .bosses
        .iter()
        .find(|b| b.boss == "b07_castellan_voss")
        .expect("B07 is placed");
    assert_eq!(voss.gate.pos, B07_GATE);
    assert!(
        voss.pos[0].abs() < 22.0 && voss.pos[2] > B07_GATE[2] && voss.pos[2] < 384.0,
        "Voss is outside the hall: {:?}",
        voss.pos
    );
    assert!(
        level
            .bosses
            .iter()
            .all(|b| b.boss == "b03_bridgewarden_corvane" || b.boss == "b07_castellan_voss"),
        "an unexpected boss in the castle"
    );
    // Mechanics.
    let gate = level
        .doors
        .iter()
        .find(|d| d.id == "gate_castle_back")
        .unwrap();
    assert!(matches!(gate.kind, lf_content::scenario::DoorKind::Locked));
    assert!(level.doors.iter().any(|d| d.id == "door_lift_cage"));
    assert_eq!(level.levers.len(), 3);
    assert_eq!(level.ladders.len(), 2);
    assert_eq!(level.lifts.len(), 1);
    assert_eq!(level.hollow_walls.len(), 1);
}

/// The route leg by leg, from the arrival road: part 1's legs to the foot of the keep stair, then
/// L16 (rest), the keep's door, the dining stair, the library, the corridor, L17 (rest), and B07's
/// fog gate (one step past its plane: Voss's gate seals behind the player, which starts his intro).
fn route(w: &World) -> Vec<Leg> {
    let mut l = legs(w);
    let l16 = lantern_pos(w, "lantern_l16");
    let l17 = lantern_pos(w, "lantern_l17");
    let near16 = [l16[0] + 1.0, l16[1], l16[2]];
    goto(w, &mut l, STAIR_FOOT, near16);
    l.push(Leg::Rest);
    l.push(Leg::Wait(10));
    let mut at = near16;
    for stop in [
        KEEP_DOOR,
        [-24.0, 12.0, 267.0],
        [-24.0, 18.0, 283.5],
        [21.0, 18.0, 287.5],
        [21.0, 18.0, 296.0],
    ] {
        goto(w, &mut l, at, stop);
        at = stop;
    }
    let near17 = [l17[0] + 1.0, l17[1], l17[2]];
    goto(w, &mut l, at, near17);
    l.push(Leg::Rest);
    l.push(Leg::Wait(10));
    goto(w, &mut l, near17, [0.0, 12.0, 338.5]);
    l.push(Leg::Goto(0.0, 342.0));
    l
}

struct Run {
    frames: Vec<ActionFrame>,
    commands: Vec<(u64, Command)>,
    world: World,
    events: Vec<Event>,
    bot: Bot,
}

fn walk() -> Run {
    let mut w = world();
    let mut bot = Bot::with_legs(route(&w));
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
fn the_route_reaches_l16_l17_and_the_fog_gate() {
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
    for id in ["lantern_l14", "lantern_l15", "lantern_l16", "lantern_l17"] {
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
    // The walk ends one step past B07's fog gate, in the hall (the gate has sealed: his intro).
    assert!(
        p.pos[0].abs() < 2.0
            && p.pos[2] > B07_GATE[2]
            && p.pos[2] < 344.0
            && (p.pos[1] - 12.0).abs() < 0.5,
        "ended at {:?}",
        p.pos
    );
    // The dining hall's sleepers were never woken.
    assert!(!w.state.flags.is_set(&flags::pickup_taken("chunk_dining")));
    assert!(w.tick < 18_000, "took {} ticks", w.tick);
}

/// Writes `game/tests/replays/m6_castle_part2.script`. See the module comment.
#[test]
#[ignore = "writes game/tests/replays/m6_castle_part2.script"]
fn record_the_castle_part2_golden() {
    let r = walk();
    assert!(r.bot.done());
    let mut text = String::from(
        "# WP-065 golden: from the arrival road through L14, the cliff path, L15, the Lower Ward, and\n\
         # the keep stair, to L16 on the windward court, through the keep (the dining stair, the\n\
         # library, the corridor) to L17, and one step past B07's fog gate.\n\
         # Generated by the ignored test `record_the_castle_part2_golden` in\n\
         # game/crates/core/src/world_state/castle2_tests.rs.\n\
         # Each line: <ticks> <move_x> <move_z> [BUTTONS].\n",
    );
    text += "seed 64\nscenario castle\n";
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
        "/../../tests/replays/m6_castle_part2.script"
    );
    std::fs::write(path, text).unwrap();
}
