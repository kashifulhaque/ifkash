//! Tests for Mossgut Cave (D1) and Glasswing Grotto (D5), WP-063: the low-ceiling rule (a swing
//! that rises over the roof strikes the rock), the light beams (mirrors, sockets, doors), each
//! cave's contents, and a scripted bot that walks each critical path. The golden replays
//! `m6_mossgut_cave` and `m6_glasswing_grotto` are those walks.
//!
//! The bot is the Waking Hollow bot (`hollow_tests::Bot`); its legs come from the baked
//! navigation grid, as in `hollowdeep_tests`. A mirror is turned with an `Interact` and a short
//! `Wait` (every button is a press edge), and the lift is a call, a wait as long as the trip, a
//! step onto the platform, and a send.
//!
//! Record the goldens: `cargo test -p lf-core record_the_caves -- --ignored`, then, from
//! `game/`, `cargo run -p lf-tools -- replay record-script tests/replays/m6_mossgut_cave.script
//! --out tests/replays/m6_mossgut_cave.lfr` (and the same for `m6_glasswing_grotto`).

use lf_content::Content;

use super::hollow_tests::{Bot, Leg};
use super::mechanics::{light_beam, low_ceiling};
use crate::events::{Event, LifeEvent};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::nav::Search;
use crate::tick::tick;
use crate::world::{DT, World};
use crate::world_state::flags;

const MOSS: &str = "mossgut_cave";
const GLASS: &str = "glasswing_grotto";
const MOSS_SEED: u64 = 63;
const GLASS_SEED: u64 = 64;
/// The longest run, in ticks (a stuck run stops here).
const MAX_TICKS: u64 = 30_000;
const BUDGET: u32 = 4_000_000;

fn world(scenario: &str, seed: u64) -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(seed, &content, scenario).unwrap_or_else(|e| panic!("{scenario}: {e:?}"))
}

fn moss() -> World {
    world(MOSS, MOSS_SEED)
}

fn glass() -> World {
    world(GLASS, GLASS_SEED)
}

fn player_pos(w: &World) -> [f32; 3] {
    w.actors.get(w.player).unwrap().pos
}

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.vel = [0.0; 3];
}

fn enemy_index(w: &World, name: &str) -> usize {
    w.layout
        .enemies
        .iter()
        .position(|e| e.name == name)
        .unwrap_or_else(|| panic!("no enemy {name}"))
}

fn enemy(w: &World, name: &str) -> crate::arena::Id {
    w.state.ai_enemies[enemy_index(w, name)]
}

/// One tick with the given stick and buttons; the events are kept.
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

fn idle(w: &mut World, ticks: u32) {
    for _ in 0..ticks {
        step(w, 0, 0, 0);
    }
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

// ---------------------------------------------------------------------------------------
// Low ceilings

/// Swings `buttons` at the sleeping Crawler `crawler_nest_a` from a spot in the Warren (under
/// a roof) and returns the Crawler's HP lost and the events the swing raised.
fn swing_at_the_nest(mut w: World, press: u16) -> (u32, Vec<Event>) {
    let id = enemy(&w, "crawler_nest_a");
    let before = w.actors.get(id).unwrap().hp;
    // 1.6 m south of the Crawler, facing north (+Z): the stick pushes that way for one tick.
    put_player(&mut w, [-13.0, 0.0, 0.4]);
    w.actors.get_mut(w.player).unwrap().yaw = 0.0;
    let mut events = step(&mut w, 0, 0, press);
    for _ in 0..90 {
        events.extend(step(&mut w, 0, 0, 0));
    }
    let after = w.actors.get(id).unwrap().hp;
    (before - after, events)
}

fn struck_rock(events: &[Event]) -> bool {
    events
        .iter()
        .any(|e| matches!(e, Event::Mechanic(m) if m.kind == low_ceiling::CEILING_STRUCK))
}

/// Under the 2.0 m roof a light swing lands and a heavy one strikes the rock: the Wayfarer's
/// heavy reaches 2.2 m. Take the roof away and the same heavy lands.
#[test]
fn a_low_ceiling_stops_an_overhead_swing_and_lets_a_light_one_land() {
    let w = moss();
    assert!(
        low_ceiling::zone_at(&w, [-13.0, 0.0, 0.4]).is_some(),
        "the Warren is under a low ceiling"
    );
    let (light, ev) = swing_at_the_nest(moss(), buttons::LIGHT);
    assert!(light > 0, "a light swing lands under the roof");
    assert!(!struck_rock(&ev), "and doesn't strike the rock");
    let (heavy, ev) = swing_at_the_nest(moss(), buttons::HEAVY);
    assert_eq!(heavy, 0, "a heavy swing strikes the rock");
    assert!(
        struck_rock(&ev),
        "the host is told the swing struck the roof"
    );
    // The same heavy, in a cave with no low ceilings, lands.
    let mut open = moss();
    open.layout.low_ceilings.clear();
    let (heavy, ev) = swing_at_the_nest(open, buttons::HEAVY);
    assert!(heavy > 0, "a heavy swing lands where the roof is high");
    assert!(!struck_rock(&ev));
}

/// The rule is the hitbox's height over the attacker's feet, so it is a property of the move:
/// of every move the Wayfarer's weapons have, the thrusts and level cuts stay under 2.0 m and
/// the overheads go over it.
#[test]
fn the_roof_stops_what_rises_over_it_and_nothing_else() {
    let content = Content::load_embedded().unwrap();
    let top = |id: &str| -> f32 {
        use lf_content::moves::Shape;
        let m = content
            .move_by_id(id)
            .unwrap_or_else(|| panic!("no move {id}"));
        m.hitboxes
            .iter()
            .map(|h| match h.shape {
                Shape::Capsule { a, b, radius } => a[1].max(b[1]) + radius,
                Shape::Sphere { center, radius } => center[1] + radius,
            })
            .fold(0.0, f32::max)
    };
    for id in [
        "ss_light_1",
        "ss_light_2",
        "ss_light_3",
        "ss_running",
        "dag_light_1",
        "dag_heavy",
        "sp_light_1",
        "sp_heavy",
        "ts_light_1",
        "ts_heavy",
        "cs_light_1",
        "husk_swipe_1",
        "husk_swipe_2",
        "gw_lunge",
        "mc_bite",
    ] {
        assert!(top(id) <= 2.0, "{id} reaches {} m, under the roof", top(id));
    }
    for id in [
        "ss_heavy",
        "ss_plunge",
        "ss_jump_heavy",
        "ax_heavy",
        "gs_heavy",
        "hm_heavy",
    ] {
        assert!(top(id) > 2.0, "{id} reaches {} m, over the roof", top(id));
    }
}

/// Every roof slab has a low-ceiling zone under it, at the slab's height, so the rule and the
/// rock agree; and the open rooms (the mouth hall, the alcove, the pit) have none.
#[test]
fn the_roof_slabs_and_the_low_ceiling_zones_agree() {
    let content = Content::load_embedded().unwrap();
    let level = content.level(MOSS).unwrap();
    let sc = content.scenario(MOSS).unwrap();
    let roofs: Vec<_> = level
        .layout
        .landmarks
        .iter()
        .filter(|l| l.name.starts_with("roof_"))
        .collect();
    assert!(roofs.len() >= 12, "{} roof slabs", roofs.len());
    for r in &roofs {
        let name = r.name.replace("roof_", "low_");
        let z = sc
            .low_ceilings
            .iter()
            .find(|z| z.name == name)
            .unwrap_or_else(|| panic!("no zone {name}"));
        // The slab's underside is the zone's height.
        assert!((r.pos[1] - r.scale[1] / 2.0 - z.height_m).abs() < 1e-4);
        assert!((r.pos[0] - z.center[0]).abs() < 1e-4 && (r.pos[2] - z.center[1]).abs() < 1e-4);
    }
    assert_eq!(roofs.len(), sc.low_ceilings.len());
    let w = moss();
    for (what, at) in [
        ("the mouth hall", [0.0, 0.0, -90.0]),
        ("the Knife's alcove", [22.0, 0.0, 75.0]),
        ("the lift's doorway", [38.0, 0.0, 59.0]),
    ] {
        assert!(low_ceiling::zone_at(&w, at).is_none(), "{what} is open");
    }
    for (what, at) in [
        ("the Low Gallery", [0.0, 0.0, -60.0]),
        ("the Spore Hall", [0.0, 0.0, -28.0]),
        ("the Nursery", [-34.0, 0.0, -24.0]),
        ("the Warren", [0.0, 0.0, 4.0]),
        ("B04's chamber", [0.0, 0.0, 70.0]),
        ("the return corridor", [42.0, 0.0, -10.0]),
    ] {
        assert!(low_ceiling::zone_at(&w, at).is_some(), "{what} is low");
    }
}

// ---------------------------------------------------------------------------------------
// Light beams

fn mirror(w: &World, id: &str) -> usize {
    w.layout.mirrors.iter().position(|m| m.id == id).unwrap()
}

fn receptor(w: &World, id: &str) -> usize {
    w.layout.receptors.iter().position(|m| m.id == id).unwrap()
}

/// Turns mirror `id` a quarter turn from beside it.
fn turn(w: &mut World, id: &str) {
    let m = w.layout.mirrors[mirror(w, id)].pos;
    interact_at(w, [m[0] + 1.0, 0.0, m[2]]);
    // One more tick lets the receptors the new beam reaches light.
    step(w, 0, 0, 0);
}

fn turn_to(w: &mut World, id: &str, facing: u8) {
    let i = mirror(w, id);
    let mut n = 0;
    while light_beam::facing(w, i) != facing {
        turn(w, id);
        n += 1;
        assert!(n <= 4, "{id} never reached facing {facing}");
    }
}

/// A beam traveling `dir` that strikes a mirror's face bounces to a compass direction, and one
/// that strikes the back stops: of the 16 pairs, each direction reflects off exactly two facings
/// and is swallowed by the other two, and a reflection never keeps the direction or reverses it.
#[test]
fn a_mirror_reflects_a_beam_off_its_face_and_swallows_it_on_its_back() {
    for dir in 0..4u8 {
        let bounced: Vec<_> = (0..4u8)
            .filter_map(|f| light_beam::reflect(dir, f))
            .collect();
        assert_eq!(bounced.len(), 2, "direction {dir}");
        for out in bounced {
            assert_ne!(out, dir, "a mirror turns the beam");
            assert_ne!(out, (dir + 2) % 4, "and never sends it back");
        }
    }
    // The cases the Grotto's solutions use: east goes north off facing 3 and south off facing 2;
    // north goes east off facing 1; south goes west off facing 3.
    assert_eq!(light_beam::reflect(1, 3), Some(0));
    assert_eq!(light_beam::reflect(1, 2), Some(2));
    assert_eq!(light_beam::reflect(0, 1), Some(1));
    assert_eq!(light_beam::reflect(2, 3), Some(3));
    // The back of those faces.
    assert_eq!(light_beam::reflect(1, 0), None);
    assert_eq!(light_beam::reflect(1, 1), None);
}

/// The Mirror Hall: the beam starts out stopped at the back of the mirror, two turns put its
/// face to it, the beam goes north to the socket, the socket lights for good, and its door opens.
#[test]
fn turning_the_teaching_mirror_lights_the_socket_and_opens_the_door() {
    let mut w = glass();
    let src = w.layout.beam_sources[0].clone();
    assert_eq!(src.id, "beam_m1");
    let m = mirror(&w, "mirror_m1");
    let beam = light_beam::trace(&w, 0);
    assert_eq!(
        beam.end,
        light_beam::End::MirrorBack(m),
        "stopped at the mirror's back"
    );
    assert_eq!(beam.legs.len(), 1);
    assert!(
        (light_beam::length(&beam) - 13.0).abs() < 0.01,
        "the beam runs 13 m to the mirror"
    );
    let door = flags::door_open("door_m1");
    assert!(!w.state.flags.is_set(&door));
    let r = receptor(&w, "recv_m1");
    // One turn is still the back of the mirror.
    turn(&mut w, "mirror_m1");
    assert_eq!(light_beam::facing(&w, m), 2);
    // Facing 2 sends it south, down the hall's passage and out of the cave: no socket.
    assert!(!matches!(
        light_beam::trace(&w, 0).end,
        light_beam::End::Receptor(_)
    ));
    assert!(!light_beam::is_lit(&w, r));
    // The second turn aims it at the socket.
    turn(&mut w, "mirror_m1");
    assert_eq!(light_beam::facing(&w, m), 3);
    let beam = light_beam::trace(&w, 0);
    assert_eq!(beam.end, light_beam::End::Receptor(r));
    assert_eq!(beam.legs.len(), 2);
    assert!(
        (light_beam::length(&beam) - 23.0).abs() < 0.01,
        "13 m, then 10 m to the socket"
    );
    assert!(light_beam::is_lit(&w, r), "the socket is lit");
    assert!(w.state.flags.is_set(&door), "and its door opens");
    // A lit socket stays lit when the mirror is turned away again, and the door stays open.
    turn(&mut w, "mirror_m1");
    assert_ne!(light_beam::trace(&w, 0).end, light_beam::End::Receptor(r));
    assert!(light_beam::is_lit(&w, r));
    assert!(w.state.flags.is_set(&door));
}

/// The Prism Hall's mirrors reach two sockets from one emitter; each socket opens its own door.
#[test]
fn the_prism_hall_has_two_sockets_for_one_emitter() {
    let mut w = glass();
    let s = w
        .layout
        .beam_sources
        .iter()
        .position(|s| s.id == "beam_m2")
        .unwrap();
    // The main door: mirrors a and b.
    turn_to(&mut w, "mirror_m2b", 1);
    turn_to(&mut w, "mirror_m2a", 3);
    let beam = light_beam::trace(&w, s);
    assert_eq!(beam.end, light_beam::End::Receptor(receptor(&w, "recv_m2")));
    assert!(w.state.flags.is_set(&flags::door_open("door_m2")));
    assert!(
        !w.state.flags.is_set(&flags::door_open("door_branch")),
        "the branch is still sealed"
    );
    // The branch door: mirrors a and c.
    turn_to(&mut w, "mirror_m2c", 3);
    turn_to(&mut w, "mirror_m2a", 2);
    let beam = light_beam::trace(&w, s);
    assert_eq!(
        beam.end,
        light_beam::End::Receptor(receptor(&w, "recv_branch"))
    );
    assert!(w.state.flags.is_set(&flags::door_open("door_branch")));
    assert!(
        w.state.flags.is_set(&flags::door_open("door_m2")),
        "the first door stays open"
    );
}

/// A shut door stops a beam, because the beam is a ray against the level's colliders; an open
/// one doesn't. (Here the Mirror Hall's own door, with a beam sent through it.)
#[test]
fn a_shut_door_stops_a_beam_and_an_open_one_lets_it_through() {
    let mut w = glass();
    // A throwaway emitter in the Mirror Hall's north passage, aimed at the door from the south.
    w.layout
        .beam_sources
        .push(lf_content::scenario::BeamSource {
            id: "probe".into(),
            pos: [1.0, 1.2, -49.0],
            dir: 0,
        });
    let probe = w.layout.beam_sources.len() - 1;
    let shut = light_beam::trace(&w, probe);
    assert_eq!(shut.end, light_beam::End::Wall);
    assert!(
        light_beam::length(&shut) < 6.0,
        "stopped at the door at z = -44"
    );
    w.state.flags.raise(&flags::door_open("door_m1"));
    super::mechanics::sync_colliders(&mut w);
    let open = light_beam::trace(&w, probe);
    assert!(
        light_beam::length(&open) > 30.0,
        "an open door lets the beam on to the next room"
    );
}

/// The twist: the Gallery of Facets' sleepers wake when the first mirror is turned, not before.
#[test]
fn turning_the_first_facet_mirror_wakes_the_nests() {
    let mut w = glass();
    let names = [
        "moth_facet_a",
        "moth_facet_b",
        "mite_facet_a",
        "mite_facet_b",
    ];
    for n in names {
        assert!(
            super::mechanics::slumber::asleep(&w, enemy(&w, n)),
            "{n} should sleep"
        );
    }
    turn(&mut w, "mirror_m5b");
    assert!(
        super::mechanics::slumber::asleep(&w, enemy(&w, names[0])),
        "the second mirror doesn't wake them"
    );
    turn(&mut w, "mirror_m5a");
    for n in names {
        assert!(
            !super::mechanics::slumber::asleep(&w, enemy(&w, n)),
            "{n} should be awake"
        );
    }
}

/// The renderer's words end with the mirrors' facings, the sockets, and each beam's legs.
#[test]
fn the_renderer_gets_the_mirrors_sockets_and_beam_legs() {
    let mut w = glass();
    let words = light_beam::render_words(&w);
    let (nm, nr, ns) = (
        w.layout.mirrors.len(),
        w.layout.receptors.len(),
        w.layout.beam_sources.len(),
    );
    assert_eq!(
        words.len(),
        nm + nr + ns * (1 + 4 * light_beam::MAX_SEGMENTS)
    );
    // The facings come first, in the level's order.
    for (i, m) in w.layout.mirrors.iter().enumerate() {
        assert_eq!(words[i], f32::from(m.facing));
    }
    // The first beam: one leg, from the emitter to the mirror.
    let beams = nm + nr;
    assert_eq!(words[beams], 1.0);
    assert_eq!(&words[beams + 1..beams + 5], &[-13.0, -60.0, 0.0, -60.0]);
    turn_to(&mut w, "mirror_m1", 3);
    let words = light_beam::render_words(&w);
    assert_eq!(words[beams], 2.0);
    assert_eq!(&words[beams + 5..beams + 9], &[0.0, -60.0, 0.0, -50.0]);
    assert_eq!(
        words[nm + receptor(&w, "recv_m1")],
        1.0,
        "the socket word is lit"
    );
    // The whole state is the old words, then these: the full vector ends with them.
    let all = super::mechanics::render_state(&w);
    assert_eq!(&all[all.len() - words.len()..], words.as_slice());
    // The player stands beside the mirror it was just turned from.
    step(&mut w, 0, 0, 0);
    assert_eq!(super::mechanics::prompt(&w), light_beam::PROMPT_TURN);
    put_player(&mut w, [0.0, 0.0, -80.0]);
    step(&mut w, 0, 0, 0);
    assert_eq!(super::mechanics::prompt(&w), 0);
}

// ---------------------------------------------------------------------------------------
// The scripted walks

/// Walks the nav path from `at` to `to`, leaving off the first point (where it stands).
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

/// The ticks a lift takes to run its whole travel, with a margin.
fn lift_ticks(w: &World) -> u16 {
    let l = &w.layout.lifts[0];
    let t = (l.top_y - l.bottom_y) / w.locomotion.mechanics.lift_speed_mps / DT;
    t.ceil() as u16 + 30
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

/// Mossgut Cave's route: the Low Gallery and the Spore Hall, the Nursery's Wick Bead and chest, the
/// hollow wall and the closet behind it, the Warren's lever (which wakes the nests), the Gate Hall,
/// B04's chamber (through its fog gate), the Fungal Knife, and the lift down.
fn moss_legs(w: &World) -> Vec<Leg> {
    let mut legs = Vec::new();
    let at = std::cell::Cell::new(w.layout.spawn);
    let to = |legs: &mut Vec<Leg>, p: [f32; 3]| {
        goto(w, legs, at.get(), p);
        at.set(p);
    };
    // The galleries, to the north end of the Spore Hall.
    to(&mut legs, [0.0, 0.0, -16.0]);
    // The Nursery: the Wick Bead, then the chest.
    to(&mut legs, [-32.0, 0.0, -25.6]);
    press(&mut legs);
    to(&mut legs, [-37.4, 0.0, -18.9]);
    press(&mut legs);
    // The hollow wall: walk at it from the north and swing, then take the closet's two items.
    to(&mut legs, [-35.0, 0.0, -29.0]);
    legs.push(Leg::Goto(-35.0, -31.8));
    legs.push(Leg::Swing);
    legs.push(Leg::Wait(40));
    legs.push(Leg::Goto(-35.0, -38.0));
    legs.push(Leg::Goto(-37.0, -41.0));
    press(&mut legs);
    legs.push(Leg::Goto(-33.0, -41.0));
    press(&mut legs);
    at.set([-33.0, 0.0, -41.0]);
    // The Warren: across the room to the lever, and pull it.
    to(&mut legs, [-3.5, 0.0, 15.8]);
    press(&mut legs);
    legs.push(Leg::Wait(120));
    // The Gate Hall, the fog gate's passage, the chamber, and the Fungal Knife in its alcove.
    to(&mut legs, [0.0, 0.0, 33.0]);
    to(&mut legs, [0.0, 0.0, 52.0]);
    to(&mut legs, [20.9, 0.0, 75.0]);
    press(&mut legs);
    // The lift: call it, step on, send it down, and walk to the connection.
    to(&mut legs, [34.2, 0.0, 60.0]);
    press(&mut legs);
    legs.push(Leg::Wait(lift_ticks(w)));
    legs.push(Leg::Goto(36.3, 60.2));
    press(&mut legs);
    legs.push(Leg::Wait(lift_ticks(w)));
    legs.push(Leg::Goto(46.0, 59.0));
    legs.push(Leg::Goto(49.0, 59.0));
    legs
}

/// Glasswing Grotto's route: the Mirror Hall, the Prism Hall (both sockets: the main door, then the
/// Nook's), the Nook's pickups, the Gallery of Facets, and the Guardian Hall's two Stalkers and the
/// Shardglass Spear.
fn glass_legs(w: &World) -> Vec<Leg> {
    let mut legs = Vec::new();
    let at = std::cell::Cell::new(w.layout.spawn);
    let to = |legs: &mut Vec<Leg>, p: [f32; 3]| {
        goto(w, legs, at.get(), p);
        at.set(p);
    };
    let turns = |legs: &mut Vec<Leg>, n: u32| {
        for _ in 0..n {
            press(legs);
        }
    };
    // The Mirror Hall: two turns of the mirror light the socket.
    to(&mut legs, [1.0, 0.0, -60.0]);
    turns(&mut legs, 2);
    // The Prism Hall, the main door: mirror b once, mirror a three times.
    to(&mut legs, [-3.0, 0.0, -20.0]);
    turns(&mut legs, 1);
    to(&mut legs, [-3.0, 0.0, -30.0]);
    turns(&mut legs, 3);
    // The branch's socket: mirror c three times, mirror a three more (3 -> 0 -> 1 -> 2).
    to(&mut legs, [-3.0, 0.0, -38.0]);
    turns(&mut legs, 3);
    to(&mut legs, [-3.0, 0.0, -30.0]);
    turns(&mut legs, 3);
    // The Nook: the Estoc, Glasswing Swarm, and the Starglass chest.
    to(&mut legs, [-38.0, 0.0, -25.0]);
    press(&mut legs);
    to(&mut legs, [-32.0, 0.0, -27.2]);
    press(&mut legs);
    to(&mut legs, [-30.5, 0.0, -17.0]);
    press(&mut legs);
    // Back through the Prism Hall's north door to the Gallery of Facets: the first mirror wakes the
    // nests, the second finishes the beam.
    to(&mut legs, [-3.0, 0.0, 14.0]);
    turns(&mut legs, 2);
    legs.push(Leg::Wait(60));
    to(&mut legs, [-3.0, 0.0, 20.0]);
    turns(&mut legs, 2);
    // The Guardian Hall: the Stalkers, and the Spear.
    to(&mut legs, [0.0, 0.0, 36.0]);
    to(&mut legs, [0.0, 0.0, 62.9]);
    press(&mut legs);
    legs
}

fn moss_walk() -> Run {
    let w = moss();
    let legs = moss_legs(&w);
    walk(w, legs)
}

fn glass_walk() -> Run {
    let w = glass();
    let legs = glass_legs(&w);
    walk(w, legs)
}

fn taken(w: &World, id: &str) -> bool {
    w.state.flags.is_set(&flags::pickup_taken(id))
}

fn mechanic_count(r: &Run, kind: u8) -> usize {
    r.events
        .iter()
        .filter(|e| matches!(e, Event::Mechanic(m) if m.kind == kind))
        .count()
}

/// The bot walks Mossgut Cave's critical path alive: the Nursery's Wick Bead and chest, the
/// hollow wall and its closet, the Warren's lever (which wakes the nests and opens the gate), the
/// Fungal Knife, and the lift down to the pit's floor, where the connection to Hollowdeep is.
#[test]
fn the_mossgut_critical_path_is_walkable() {
    let r = moss_walk();
    assert_walked(&r);
    let w = &r.world;
    for id in [
        "wick_nursery",
        "spell_veil",
        "charm_quickening",
        "knife_fungal",
    ] {
        assert!(taken(w, id), "{id} was never taken");
    }
    assert!(w.state.flags.is_set(&flags::chest_opened("chest_nursery")));
    assert!(
        w.state
            .flags
            .is_set(&flags::wall_revealed("wall_nursery_closet"))
    );
    assert!(w.state.flags.is_set(&flags::lever_pulled("lever_warren")));
    assert!(w.state.flags.is_set(&flags::door_open("door_warren")));
    // The nests woke.
    for n in [
        "crawler_nest_a",
        "crawler_nest_b",
        "crawler_nest_c",
        "crawler_nest_d",
    ] {
        assert!(
            !super::mechanics::slumber::asleep(w, enemy(w, n)),
            "{n} should be awake"
        );
    }
    // The lift came up, went down with the player, and the player is on the pit's floor, at the
    // connection to Hollowdeep.
    assert_eq!(
        mechanic_count(&r, crate::events::MechanicEvent::LIFT_STARTED),
        2
    );
    let p = player_pos(w);
    assert!(p[1] < -17.0, "the player is on the pit's floor: {p:?}");
    let to = content_connection(MOSS, "hollowdeep");
    let d = sqrt((p[0] - to[0]).powi(2) + (p[2] - to[2]).powi(2));
    assert!(d < 3.0, "{d:.1} m from the connection to Hollowdeep");
    assert!(w.tick < 6_000, "took {} ticks", w.tick);
}

fn content_connection(area: &str, to: &str) -> [f32; 3] {
    let content = Content::load_embedded().unwrap();
    content
        .level(area)
        .unwrap()
        .area
        .connections
        .iter()
        .find(|c| c.to == to)
        .unwrap_or_else(|| panic!("{area} has no connection to {to}"))
        .pos
}

/// The bot walks Glasswing Grotto's critical path alive: all four sockets, the Nook, the Gallery
/// of Facets, both elite Stalkers, and the Shardglass Spear.
#[test]
fn the_glasswing_critical_path_is_walkable() {
    let r = glass_walk();
    assert_walked(&r);
    let w = &r.world;
    for id in ["recv_m1", "recv_m2", "recv_branch", "recv_m5"] {
        assert!(
            w.state.flags.is_set(&light_beam::lit_flag(id)),
            "{id} was never lit"
        );
    }
    for door in ["door_m1", "door_m2", "door_branch", "door_m5"] {
        assert!(w.state.flags.is_set(&flags::door_open(door)), "{door}");
    }
    for id in ["estoc_nook", "spell_glasswing_swarm", "spear_shardglass"] {
        assert!(taken(w, id), "{id} was never taken");
    }
    assert!(
        w.state
            .flags
            .is_set(&flags::chest_opened("chest_starglass"))
    );
    for n in ["stalker_guard_a", "stalker_guard_b"] {
        let a = w.actors.get(enemy(w, n)).unwrap();
        assert!(a.dead, "{n} survived");
    }
    assert_eq!(mechanic_count(&r, light_beam::RECEPTOR_LIT), 4);
    assert!(mechanic_count(&r, light_beam::MIRROR_TURNED) >= 15);
    assert!(w.tick < 6_000, "took {} ticks", w.tick);
}

/// What the spec puts in D1 is placed: Marsh Crawlers and Husks, the Wick Bead, Veil, the
/// Quickening Charm, the Fungal Knife, Forgeglass Shards, a hollow wall, a lift with a call lever
/// at each stop, a one-way shortcut, B04's Echo Stone inside its chamber, no Lantern (the spec's
/// L05 is outside, in the Verge), and B04's boss entity (WP-072) at the chamber's wall.
#[test]
fn mossgut_cave_has_what_the_spec_puts_in_d1() {
    let content = Content::load_embedded().unwrap();
    let sc = content.scenario(MOSS).unwrap();
    assert!(sc.lanterns.is_empty(), "no Lantern inside a mini-dungeon");
    for kind in ["marsh_crawler", "husk"] {
        assert!(sc.enemies.iter().any(|e| e.enemy == kind), "no {kind}");
    }
    assert!(sc.enemies.len() <= 20, "{} enemies", sc.enemies.len());
    let items: Vec<String> = sc
        .pickups
        .iter()
        .map(|p| p.item.clone())
        .chain(sc.chests.iter().filter_map(|c| c.loot.item.clone()))
        .collect();
    for item in [
        "itm_wick_bead",
        "spl_veil",
        "chm_quickening",
        "wpn_fungal_knife",
        "itm_forgeglass_shard",
    ] {
        assert!(items.iter().any(|i| i == item), "no {item}");
    }
    assert_eq!(sc.hollow_walls.len(), 1);
    assert_eq!(sc.lifts.len(), 1);
    for stop in 0..=1u8 {
        assert!(sc.levers.iter().any(|l| matches!(&l.target,
            lf_content::scenario::LeverTarget::Lift { id, stop: s } if id == "mossgut_lift" && *s == stop)));
    }
    assert!(
        sc.lifts[0].top_y - sc.lifts[0].bottom_y < 20.0,
        "a fall into the pit isn't a death"
    );
    // The Echo Stone is inside B04's chamber (x -15..15, z 46..88), past the gate at z = 44.
    assert_eq!(sc.echo_stones.len(), 1);
    let s = &sc.echo_stones[0];
    assert_eq!(s.id, "stone_b04");
    assert!(
        s.pos[0].abs() < 15.0 && s.pos[2] > 46.0 && s.pos[2] < 88.0,
        "{:?}",
        s.pos
    );
    assert_eq!(world(MOSS, 1).echo_table.stones.len(), 1);
    // B04, the Mossgut Brood: one boss, in the north wall of the chamber, behind the fog gate
    // at z = 44 (WP-072).
    assert_eq!(sc.bosses.len(), 1);
    let b = &sc.bosses[0];
    assert_eq!(b.boss, "b04_mossgut_brood");
    assert!(b.pos[2] > 46.0 && b.pos[2] < 88.0, "{:?}", b.pos);
    assert_eq!(b.gate.pos, [0.0, 0.0, 44.0]);
    // The exits: out to the Verge, down to Hollowdeep (whose stub names this area).
    let area = &content.level(MOSS).unwrap().area;
    assert!(area.connections.iter().any(|c| c.to == "verge"));
    assert!(area.connections.iter().any(|c| c.to == "hollowdeep"));
    let deep = &content.level("hollowdeep").unwrap().area;
    assert!(
        deep.connections.iter().any(|c| c.to == MOSS),
        "Hollowdeep's stub names this cave"
    );
    // The nests wake on the lever's own flag.
    let wakes: Vec<_> = sc
        .enemies
        .iter()
        .filter_map(|e| e.wakes_on.as_deref())
        .collect();
    assert_eq!(wakes.len(), 4);
    assert!(
        wakes
            .iter()
            .all(|f| *f == flags::lever_pulled("lever_warren"))
    );
}

/// What the spec puts in D5 is placed: mirrors, sockets with doors, Cave Moths and Lamp Mites,
/// the two elite Stalkers, the Estoc, Glasswing Swarm, the Starglass, the Shardglass Spear, a
/// one-way shortcut, no Lantern, and no darkness rule (the grotto is lit by its crystals).
#[test]
fn glasswing_grotto_has_what_the_spec_puts_in_d5() {
    let content = Content::load_embedded().unwrap();
    let sc = content.scenario(GLASS).unwrap();
    assert!(sc.lanterns.is_empty(), "no Lantern inside a mini-dungeon");
    assert!(sc.darkness.is_none());
    assert_eq!(sc.beam_sources.len(), 3);
    assert_eq!(sc.mirrors.len(), 6);
    assert_eq!(sc.receptors.len(), 4);
    for r in &sc.receptors {
        assert!(r.opens.is_some(), "{} opens nothing", r.id);
    }
    for kind in ["cave_moth", "lamp_mite", "glasswing_stalker_elite"] {
        assert!(sc.enemies.iter().any(|e| e.enemy == kind), "no {kind}");
    }
    assert_eq!(
        sc.enemies
            .iter()
            .filter(|e| e.enemy == "glasswing_stalker_elite")
            .count(),
        2,
        "two guardians"
    );
    let items: Vec<String> = sc
        .pickups
        .iter()
        .map(|p| p.item.clone())
        .chain(sc.chests.iter().filter_map(|c| c.loot.item.clone()))
        .collect();
    for item in [
        "wpn_glassneedle_estoc",
        "spl_glasswing_swarm",
        "itm_starglass",
        "wpn_shardglass_spear",
    ] {
        assert!(items.iter().any(|i| i == item), "no {item}");
    }
    assert!(
        sc.bosses.is_empty() && sc.echo_stones.is_empty(),
        "D5 has no boss"
    );
    // The elite has the base Stalker's moves and shimmer, and more of everything else.
    let base = content.enemy_by_id("glasswing_stalker").unwrap();
    let elite = content.enemy_by_id("glasswing_stalker_elite").unwrap();
    assert_eq!(base.moveset, elite.moveset);
    assert!(elite.shimmers && elite.hp > 2 * base.hp && elite.attack > base.attack);
    assert!(elite.poise > base.poise && elite.defense.flat > base.defense.flat);
    let area = &content.level(GLASS).unwrap().area;
    assert!(area.connections.iter().any(|c| c.to == "hollowdeep"));
    let deep = &content.level("hollowdeep").unwrap().area;
    assert!(
        deep.connections.iter().any(|c| c.to == GLASS),
        "Hollowdeep's stub names this grotto"
    );
}

/// Each cave's shortcut is a one-way door: it opens from the corridor and stays shut from the
/// mouth hall.
#[test]
fn each_caves_shortcut_door_opens_from_the_corridor_only() {
    for (scenario, seed) in [(MOSS, MOSS_SEED), (GLASS, GLASS_SEED)] {
        let mut w = world(scenario, seed);
        let open = flags::door_open("door_return");
        interact_at(&mut w, [10.0, 0.0, -87.0]);
        assert!(
            !w.state.flags.is_set(&open),
            "{scenario}: it opened from the wrong side"
        );
        interact_at(&mut w, [16.5, 0.0, -87.0]);
        assert!(
            w.state.flags.is_set(&open),
            "{scenario}: it didn't open from the corridor"
        );
    }
}

/// The Warren's lever opens the gate, and the nests wake while it is pulled: not before.
#[test]
fn the_warren_lever_opens_the_gate_and_wakes_the_nests() {
    let mut w = moss();
    let sleepers = [
        "crawler_nest_a",
        "crawler_nest_b",
        "crawler_nest_c",
        "crawler_nest_d",
    ];
    for n in sleepers {
        assert!(super::mechanics::slumber::asleep(&w, enemy(&w, n)));
    }
    assert!(!w.state.flags.is_set(&flags::door_open("door_warren")));
    interact_at(&mut w, [-3.5, 0.0, 15.8]);
    assert!(w.state.flags.is_set(&flags::door_open("door_warren")));
    for n in sleepers {
        assert!(!super::mechanics::slumber::asleep(&w, enemy(&w, n)), "{n}");
    }
}

/// The lift down: called from the doorway, it carries the player 18 m to the pit's floor, and
/// each stop's lever sends it back.
#[test]
fn the_mossgut_lift_carries_the_player_down_and_back() {
    let mut w = moss();
    let trip = u32::from(lift_ticks(&w));
    let top = w.layout.lifts[0].top_y - w.layout.lifts[0].bottom_y;
    assert!((top - 18.0).abs() < 0.01);
    interact_at(&mut w, [34.4, 0.0, 60.4]);
    idle(&mut w, trip);
    assert!(
        (w.mech.lifts[0].rise - top).abs() < 0.01,
        "the lift came up"
    );
    // Onto the platform, and send it down from the same lever.
    interact_at(&mut w, [36.3, 0.0, 60.2]);
    idle(&mut w, trip + 60);
    assert_eq!(w.mech.lifts[0].rise, 0.0, "the lift went down");
    let p = player_pos(&w);
    assert!(p[1] < -17.5, "the player rode it to the floor: {p:?}");
    // The bottom lever, reached from the platform's edge, sends it up with the player.
    interact_at(&mut w, [39.4, p[1], 59.0]);
    idle(&mut w, trip + 60);
    assert!(
        player_pos(&w)[1] > -0.5,
        "and back up: {:?}",
        player_pos(&w)
    );
}

/// The Wayfarer's sword swing, the one that hits the nest in `swing_at_the_nest`, is the same
/// light attack the bot fights with, so the bot's whole route is a fight under the low roof: its
/// route, ending alive, is the proof that the thrust and the light chain are enough.
#[test]
fn the_bot_fights_the_caves_under_the_roof_with_light_swings() {
    let r = moss_walk();
    assert_walked(&r);
    let struck = mechanic_count(&r, low_ceiling::CEILING_STRUCK);
    assert_eq!(struck, 0, "the bot never swings over its head");
    let dead = r
        .world
        .layout
        .enemies
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            r.world
                .actors
                .get(r.world.state.ai_enemies[*i])
                .is_some_and(|a| a.dead)
        })
        .count();
    assert!(dead >= 8, "the bot killed {dead} enemies under the roof");
}

fn script_text(r: &Run, scenario: &str, seed: u64, what: &str) -> String {
    let mut text = format!(
        "# WP-063 golden: {what}.\n\
         # Generated by the ignored test `record_the_caves_goldens` in\n\
         # game/crates/core/src/world_state/cave_tests.rs.\n\
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

/// Writes `game/tests/replays/m6_mossgut_cave.script` and `m6_glasswing_grotto.script`. See the
/// module comment.
#[test]
#[ignore = "writes game/tests/replays/m6_*.script"]
fn record_the_caves_goldens() {
    let r = moss_walk();
    assert_walked(&r);
    write_script(
        "m6_mossgut_cave",
        &script_text(
            &r,
            MOSS,
            MOSS_SEED,
            "Mossgut Cave's critical path: the galleries, the Nursery and its closet, the Warren's lever, B04's chamber, the Fungal Knife, and the lift down",
        ),
    );
    let r = glass_walk();
    assert_walked(&r);
    write_script(
        "m6_glasswing_grotto",
        &script_text(
            &r,
            GLASS,
            GLASS_SEED,
            "Glasswing Grotto's critical path: the mirrors and sockets, the Nook, the Facets, the two elite Stalkers, and the Shardglass Spear",
        ),
    );
}
