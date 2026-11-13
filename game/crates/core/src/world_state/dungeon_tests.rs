//! Tests for the Mirrormere mini-dungeons (WP-061): the Sunken Ossuary (D2) and the Drowned
//! Belfry (D3). Each has a scripted critical-path walk, the golden replays `m6_sunken_ossuary`
//! and `m6_drowned_belfry`, and tests of its signature mechanic: the water-level lever that
//! wakes the Drowned, and the bell that stuns and opens the bell-shaft lift.
//!
//! The bot is the Waking Hollow bot (`hollow_tests::Bot`), with hand-placed waypoints: the
//! dungeons are rooms joined by doorways, so a list of points through the doorways is the
//! route.
//!
//! Record the goldens: `cargo test -p lf-core record_the_dungeon_goldens -- --ignored`, then
//! for each, `cargo run -p lf-tools -- replay record-script tests/replays/m6_<name>.script
//! --out tests/replays/m6_<name>.lfr` (from `game/`).

use lf_content::Content;

use super::hollow_tests::{Bot, Leg};
use super::mechanics::water_level;
use crate::events::{Event, LifeEvent};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

/// The longest run, in ticks (a stuck-run stop, not a gameplay number).
const MAX_TICKS: u64 = 40_000;
/// Water deeper than this is a wall (`locomotion.ron`, `mechanics.water_deep_over_m`).
const DEEP_M: f32 = 1.4;

fn world(scenario: &str, seed: u64) -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(seed, &content, scenario).unwrap_or_else(|e| panic!("{scenario} exists: {e}"))
}

fn pool(w: &World, name: &str) -> usize {
    w.layout
        .pools
        .iter()
        .position(|p| p.name == name)
        .unwrap_or_else(|| panic!("no pool {name}"))
}

fn depth(w: &World, name: &str) -> f32 {
    water_level::depth(w, pool(w, name))
}

fn player_pos(w: &World) -> [f32; 3] {
    w.actors.get(w.player).unwrap().pos
}

fn enemy(w: &World, name: &str) -> crate::arena::Id {
    let i = w
        .layout
        .enemies
        .iter()
        .position(|e| e.name == name)
        .unwrap_or_else(|| panic!("no enemy {name}"));
    w.state.ai_enemies[i]
}

/// Puts the player at `pos` and presses `INTERACT` for one tick (then lets go).
fn interact_at(w: &mut World, pos: [f32; 3]) {
    w.actors.get_mut(w.player).unwrap().pos = pos;
    for buttons in [buttons::INTERACT, 0, 0] {
        tick(
            w,
            &TickInput {
                frame: ActionFrame {
                    move_x: 0,
                    move_z: 0,
                    buttons,
                },
                commands: Vec::new(),
            },
        );
        w.events.drain();
    }
}

/// Drives the player toward (x, z) for `ticks` ticks with no buttons.
fn push_toward(w: &mut World, x: f32, z: f32, ticks: u32) {
    for _ in 0..ticks {
        let p = player_pos(w);
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

/// Ticks with no input.
fn idle(w: &mut World, ticks: u32) {
    for _ in 0..ticks {
        tick(w, &TickInput::default());
        w.events.drain();
    }
}

/// What a scripted run produced.
struct Run {
    frames: Vec<ActionFrame>,
    commands: Vec<(u64, Command)>,
    world: World,
    events: Vec<Event>,
    bot: Bot,
}

fn walk(scenario: &str, seed: u64, legs: Vec<Leg>) -> Run {
    let mut w = world(scenario, seed);
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

/// Asserts a run finished its route alive.
fn assert_finished(r: &Run) {
    let w = &r.world;
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, Event::Life(LifeEvent::Died { .. }))),
        "the bot died at tick {} near {:?}, on leg {} of {}",
        w.tick,
        player_pos(w),
        r.bot.leg,
        r.bot.legs.len()
    );
    assert!(
        r.bot.done(),
        "stopped at leg {} of {} (tick {}, at {:?})",
        r.bot.leg,
        r.bot.legs.len(),
        w.tick,
        player_pos(w)
    );
}

/// Writes a golden's `.script` text: see `mirrormere_tests::record_the_mirrormere_golden`.
fn script_text(title: &str, scenario: &str, seed: u64, r: &Run) -> String {
    let mut text = format!(
        "# WP-061 golden: {title}\n\
         # Generated by the ignored test `record_the_dungeon_goldens` in\n\
         # game/crates/core/src/world_state/dungeon_tests.rs.\n\
         # Each line: <ticks> <move_x> <move_z> [BUTTONS].\n"
    );
    text += &format!("seed {seed}\nscenario {scenario}\n");
    let names = [
        (buttons::LIGHT, "LIGHT"),
        (buttons::HEAVY, "HEAVY"),
        (buttons::GUARD, "GUARD"),
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

// ---------------------------------------------------------------------------------------
// The Sunken Ossuary

mod ossuary {
    use super::*;

    pub const SCENARIO: &str = "sunken_ossuary";
    pub const SEED: u64 = 61;

    /// The critical path: E -> lever 1 -> T -> H -> L (lever 2) -> H -> F -> V.
    pub fn legs() -> Vec<Leg> {
        use Leg::*;
        vec![
            // E, then T: pull lever 1 and wait for the basin's water to go down.
            Goto(0.0, -60.0),
            Goto(0.0, -56.0),
            Goto(0.0, -52.0),
            Goto(-8.5, -49.8),
            Interact,
            Wait(10),
            // Across the drained basin, into the hall.
            Goto(0.0, -48.0),
            Goto(0.0, -34.0),
            Goto(0.0, -30.0),
            Goto(0.0, -27.0),
            // H: the sleepers. Through the east doorway to the lever room.
            Goto(5.0, -19.0),
            Goto(14.0, -16.0),
            Goto(19.0, -16.0),
            Goto(26.0, -19.5),
            Goto(30.0, -19.0),
            Interact,
            Wait(10),
            // Back through the hall, now awake, and north into the Flooded Crypt.
            Goto(19.0, -16.0),
            Goto(14.0, -16.0),
            Goto(5.0, -14.0),
            Goto(0.0, -8.0),
            Goto(0.0, -3.0),
            Goto(0.0, 4.0),
            Goto(0.0, 16.0),
            Goto(0.0, 21.5),
            // The vault's fog gate (B05, WP-073) is the plane z = 23: stop short of it, back out,
            // and take the shortcut: F's east door, the long corridor, and the one-way door
            // into E.
            Goto(0.0, 20.0),
            Goto(8.0, 12.0),
            Goto(15.0, 10.0),
            Goto(20.0, 10.0),
            Goto(38.0, 10.0),
            Goto(38.0, -30.0),
            Goto(38.0, -62.0),
            Goto(38.0, -66.0),
            Goto(16.0, -66.0),
            Interact,
            Wait(30),
            Goto(10.0, -66.0),
            Goto(0.0, -70.0),
        ]
    }
}

// ---------------------------------------------------------------------------------------
// The Drowned Belfry

mod belfry {
    use super::*;

    pub const SCENARIO: &str = "drowned_belfry";
    pub const SEED: u64 = 62;

    /// The critical path: A -> T1 (bell A) -> G (bell B) -> TH (bell C) -> LO (bell D, the
    /// Choir Knight, the sluice lever, Choir Litany) -> the return corridor and its door.
    pub fn legs() -> Vec<Leg> {
        use Leg::*;
        vec![
            // A, then T1: ring bell A (it opens door_t1) once the Husk is dealt with.
            Goto(0.0, -60.0),
            Goto(0.0, -52.0),
            Goto(1.3, -43.0),
            Goto(1.3, -42.2),
            Interact,
            Wait(20),
            // G: the choir. Ring bell B in the middle of it; it opens door_g.
            Goto(0.0, -34.0),
            Goto(0.0, -28.0),
            Goto(1.3, -18.0),
            Goto(1.3, -16.4),
            Interact,
            Wait(20),
            // TH: down the causeway to bell C; it opens the shaft's gate.
            Goto(0.0, -8.0),
            Goto(0.0, -2.0),
            Goto(0.0, 6.0),
            Goto(0.0, 14.0),
            Goto(1.2, 16.0),
            Interact,
            Wait(20),
            // LO: ring bell D (it stuns the Knight), fight it, pull the sluice lever, and take
            // Choir Litany from its pedestal.
            Goto(0.0, 20.0),
            Goto(0.0, 26.0),
            Goto(-8.0, 27.0),
            Goto(-10.0, 27.6),
            Interact,
            Goto(-6.0, 32.0),
            Goto(10.0, 44.0),
            Goto(12.0, 47.4),
            Interact,
            Wait(10),
            Goto(-8.0, 46.0),
            Goto(-10.4, 48.0),
            Interact,
            Wait(10),
            // The return corridor, and its one-way door into A.
            Goto(10.0, 38.0),
            Goto(20.0, 38.0),
            Goto(44.0, 38.0),
            Goto(44.0, -30.0),
            Goto(44.0, -62.0),
            Goto(44.0, -66.0),
            Goto(16.0, -66.0),
            Interact,
            Wait(30),
            Goto(8.0, -66.0),
            Goto(0.0, -70.0),
        ]
    }
}

// ---------------------------------------------------------------------------------------
// Ossuary tests

/// The level has what the spec puts in D2: two drain levers, the Drowned (most of them
/// asleep), a shortcut door, a hollow-wall secret, B05's Echo Stone, and no Lantern inside.
#[test]
fn the_ossuary_has_its_levers_sleepers_secret_and_echo_stone() {
    let content = Content::load_embedded().unwrap();
    let level = content.scenario(ossuary::SCENARIO).unwrap();
    assert!(
        level.lanterns.is_empty(),
        "no Lantern inside a mini-dungeon"
    );
    assert_eq!(level.levers.len(), 2);
    assert_eq!(level.hollow_walls.len(), 1);
    assert_eq!(level.echo_stones.len(), 1);
    assert_eq!(level.echo_stones[0].id, "stone_b05");
    let drowned: Vec<_> = level
        .enemies
        .iter()
        .filter(|e| e.enemy == "drowned")
        .collect();
    assert!(drowned.len() >= 6, "{} Drowned", drowned.len());
    assert!(
        drowned
            .iter()
            .all(|e| e.wakes_on.as_deref() == Some("ossuary.drain_f")),
        "every Drowned sleeps until the crypt is drained"
    );
    // The shortcut door opens from the corridor side only.
    let door = level
        .doors
        .iter()
        .find(|d| d.id == "door_shortcut")
        .unwrap();
    assert!(matches!(door.kind, lf_content::scenario::DoorKind::OneWay));
    // Both flooded rooms are walls until their lever is pulled, and wade after.
    for (name, flag, drained) in [
        ("flood_t", "ossuary.drain_t", 0.4),
        ("flood_f", "ossuary.drain_f", 0.5),
    ] {
        let p = level.pools.iter().find(|p| p.name == name).unwrap();
        assert!(p.depth_m > DEEP_M, "{name} is deep");
        assert_eq!(p.lowered_by.as_deref(), Some(flag));
        assert_eq!(p.lowered_depth_m, Some(drained));
        assert!(drained < 0.6, "{name} wades once drained");
    }
}

/// Lever 1 teaches the water-level mechanic: the basin is a wall until it is pulled, then wades.
#[test]
fn lever_one_drains_the_teaching_basin() {
    let mut w = world(ossuary::SCENARIO, ossuary::SEED);
    assert!(depth(&w, "flood_t") > DEEP_M);
    w.actors.get_mut(w.player).unwrap().pos = [0.0, 0.0, -50.0];
    push_toward(&mut w, 0.0, -30.0, 300);
    let p = player_pos(&w);
    assert!(p[2] < -45.0, "the deep basin let the player in: {p:?}");
    interact_at(&mut w, [-8.5, 0.0, -49.8]);
    assert!(w.state.flags.is_set("ossuary.drain_t"));
    assert!(w.state.flags.is_set(&flags::lever_pulled("lever_drain_t")));
    assert!(depth(&w, "flood_t") < 0.6);
    w.actors.get_mut(w.player).unwrap().pos = [0.0, 0.0, -50.0];
    push_toward(&mut w, 0.0, -30.0, 400);
    let p = player_pos(&w);
    assert!(p[2] > -33.0, "the drained basin didn't open: {p:?}");
    // Drain 1 doesn't touch the second room or wake anyone.
    assert!(depth(&w, "flood_f") > DEEP_M);
}

/// Lowering the water wakes the Drowned: until lever 2 they stand still and ignore the
/// player; once it is pulled they come for the player, and the crypt wades.
#[test]
fn lowering_the_water_wakes_the_drowned() {
    let mut w = world(ossuary::SCENARIO, ossuary::SEED);
    let sleepers: Vec<_> = w
        .layout
        .enemies
        .iter()
        .filter(|e| e.wakes_on.is_some())
        .map(|e| e.name.clone())
        .collect();
    assert!(sleepers.len() >= 6);
    for n in &sleepers {
        assert!(
            super::mechanics::slumber::asleep(&w, enemy(&w, n)),
            "{n} should sleep"
        );
    }
    // The player stands among them in the hall, beside one, and nothing stirs.
    let hp = w.actors.get(w.player).unwrap().hp;
    let a = enemy(&w, "drowned_hall_a");
    let start = w.actors.get(a).unwrap().pos;
    w.actors.get_mut(w.player).unwrap().pos = [-6.5, 0.0, -17.0];
    idle(&mut w, 300);
    assert_eq!(
        w.actors.get(w.player).unwrap().hp,
        hp,
        "a sleeper hurt the player"
    );
    assert_eq!(w.actors.get(a).unwrap().pos, start, "a sleeper moved");
    // Lever 2: they wake.
    assert!(depth(&w, "flood_f") > DEEP_M);
    interact_at(&mut w, [30.0, 0.0, -19.0]);
    assert!(w.state.flags.is_set("ossuary.drain_f"));
    assert!(depth(&w, "flood_f") < 0.6, "the crypt drains");
    for n in &sleepers {
        assert!(
            !super::mechanics::slumber::asleep(&w, enemy(&w, n)),
            "{n} should be awake"
        );
    }
    w.actors.get_mut(w.player).unwrap().pos = [-6.5, 0.0, -17.0];
    idle(&mut w, 240);
    let moved = w.actors.get(a).unwrap().pos;
    let hurt = w.actors.get(w.player).unwrap().hp < hp;
    assert!(
        moved != start || hurt,
        "the Drowned slept on after the water went"
    );
}

/// The deep crypt is a wall until lever 2: the way to the Keeper needs the lever.
#[test]
fn the_flooded_crypt_is_a_wall_until_lever_two() {
    let mut w = world(ossuary::SCENARIO, ossuary::SEED);
    w.actors.get_mut(w.player).unwrap().pos = [0.0, 0.0, -8.0];
    push_toward(&mut w, 0.0, 20.0, 300);
    let p = player_pos(&w);
    assert!(p[2] < -3.0, "the deep crypt let the player in: {p:?}");
    interact_at(&mut w, [30.0, 0.0, -19.0]);
    w.actors.get_mut(w.player).unwrap().pos = [0.0, 0.0, -8.0];
    push_toward(&mut w, 0.0, 20.0, 600);
    let p = player_pos(&w);
    assert!(p[2] > 0.0, "the drained crypt didn't open: {p:?}");
}

/// The shortcut door opens from the corridor and stays shut from the entry hall.
#[test]
fn the_ossuary_shortcut_door_opens_from_the_corridor_only() {
    let mut w = world(ossuary::SCENARIO, ossuary::SEED);
    let open = flags::door_open("door_shortcut");
    interact_at(&mut w, [11.0, 0.0, -66.0]);
    assert!(
        !w.state.flags.is_set(&open),
        "it opened from the wrong side"
    );
    interact_at(&mut w, [15.5, 0.0, -66.0]);
    assert!(
        w.state.flags.is_set(&open),
        "it didn't open from the corridor"
    );
}

/// The critical path: both levers, the sleeping Drowned, the Flooded Crypt, the vault's door,
/// and the shortcut back to the entry hall. The bot survives it.
#[test]
fn the_ossuary_critical_path_is_walkable() {
    let r = walk(ossuary::SCENARIO, ossuary::SEED, ossuary::legs());
    assert_finished(&r);
    let w = &r.world;
    for flag in ["ossuary.drain_t", "ossuary.drain_f"] {
        assert!(w.state.flags.is_set(flag), "{flag} was never raised");
    }
    assert!(w.state.flags.is_set(&flags::door_open("door_shortcut")));
    assert!(depth(w, "flood_t") < 0.6 && depth(w, "flood_f") < 0.6);
    // It takes under two minutes of game time, and the bot ends at the foot of the stair.
    assert!(w.tick < 7_200, "took {} ticks", w.tick);
    assert!(player_pos(w)[2] < -60.0);
}

// ---------------------------------------------------------------------------------------
// Belfry tests

/// The level has what the spec puts in D3: four bells, the bell-shaft lift with a call lever
/// at each stop, the sluice lever, the Choir Knight, and Choir Litany.
#[test]
fn the_belfry_has_its_bells_lift_sluice_and_elite() {
    let content = Content::load_embedded().unwrap();
    let level = content.scenario(belfry::SCENARIO).unwrap();
    assert!(
        level.lanterns.is_empty(),
        "no Lantern inside a mini-dungeon"
    );
    let bells = level
        .levers
        .iter()
        .filter(|l| matches!(l.target, lf_content::scenario::LeverTarget::Bell { .. }))
        .count();
    assert_eq!(bells, 4);
    assert_eq!(level.lifts.len(), 1);
    let lift = &level.lifts[0];
    assert!(lift.top_y - lift.bottom_y >= 10.0, "a deep shaft");
    for stop in 0..=1u8 {
        assert!(level.levers.iter().any(|l| matches!(&l.target,
            lf_content::scenario::LeverTarget::Lift { id, stop: s } if id == &lift.id && *s == stop)));
    }
    let sluice = level
        .levers
        .iter()
        .find(|l| l.id == "lever_sluice")
        .unwrap();
    assert_eq!(
        sluice.target,
        lf_content::scenario::LeverTarget::Flag(water_level::sluice_flag("mirrormere")),
        "the sluice lever raises the flag Mirrormere's lake reads"
    );
    assert_eq!(
        level
            .enemies
            .iter()
            .filter(|e| e.enemy == "choir_knight")
            .count(),
        1
    );
    for id in [
        "spell_choir_litany",
        "wpn_greatanchor_chapel",
        "spell_mirror_ward",
    ] {
        assert!(level.pickups.iter().any(|p| p.id == id), "{id} is placed");
    }
    assert_eq!(level.hollow_walls.len(), 1);
}

/// Ringing bell A stuns the Husk beside it (it kneels and can't act), leaves an enemy out of
/// range alone, and opens the door the bell names.
#[test]
fn a_bell_stuns_enemies_in_range_and_opens_its_door() {
    let mut w = world(belfry::SCENARIO, belfry::SEED);
    let husk = enemy(&w, "husk_bell_room");
    let far = enemy(&w, "husk_gallery_a");
    let door = flags::door_open("door_t1");
    assert!(!w.state.flags.is_set(&door));
    assert_eq!(w.actors.get(husk).unwrap().poise.kneel, 0);
    interact_at(&mut w, [1.3, 0.0, -42.2]);
    let h = w.actors.get(husk).unwrap();
    assert!(h.poise.kneel > 100, "the Husk kneels: {}", h.poise.kneel);
    assert!(h.moves.guard.stagger > 100, "and can't act");
    assert_eq!(w.actors.get(far).unwrap().poise.kneel, 0, "out of range");
    assert!(w.state.flags.is_set(&door), "the bell opened its door");
    assert!(w.state.flags.is_set(&flags::lever_pulled("bell_a")));
    // The stun wears off.
    idle(&mut w, 260);
    assert_eq!(w.actors.get(husk).unwrap().poise.kneel, 0);
}

/// Bell D stuns the elite: the Choir Knight kneels for the longest stun a bell can give.
#[test]
fn the_loft_bell_stuns_the_choir_knight() {
    let mut w = world(belfry::SCENARIO, belfry::SEED);
    let knight = enemy(&w, "choir_knight_loft");
    interact_at(&mut w, [-10.0, 0.0, 27.6]);
    assert!(w.actors.get(knight).unwrap().poise.kneel > 200);
}

/// The bell-shaft lift: bell C opens the gate, then the lift carries the player down 12 m to
/// the shaft's floor and back up, each stop with its call lever.
#[test]
fn the_bell_shaft_lift_runs_after_bell_c_opens_the_gate() {
    let mut w = world(belfry::SCENARIO, belfry::SEED);
    let gate = flags::door_open("door_shaft");
    assert!(!w.state.flags.is_set(&gate));
    // Bell C opens the gate.
    interact_at(&mut w, [1.2, 0.0, 16.0]);
    assert!(w.state.flags.is_set(&gate), "bell C opens the shaft's gate");
    // The lift waits at the bottom. Call it from the doorway.
    assert_eq!(w.mech.lifts[0].rise, 0.0);
    interact_at(&mut w, [16.8, 0.0, 9.2]);
    idle(&mut w, 900);
    let top = w.layout.lifts[0].top_y - w.layout.lifts[0].bottom_y;
    assert!(
        (w.mech.lifts[0].rise - top).abs() < 0.01,
        "the lift came up"
    );
    // Step on and send it down from the platform: it carries the player to the floor.
    interact_at(&mut w, [19.0, 0.0, 9.2]);
    idle(&mut w, 1_200);
    let p = player_pos(&w);
    assert_eq!(w.mech.lifts[0].rise, 0.0, "the lift went down");
    assert!(p[1] < -11.0, "the player rode it to the floor: {p:?}");
    // Send it back up from the bottom lever.
    interact_at(&mut w, [22.0, p[1], 9.0]);
    idle(&mut w, 1_200);
    let p = player_pos(&w);
    assert!(p[1] > -0.5, "the player rode it back up: {p:?}");
}

/// The sluice lever raises the flag Mirrormere's lake reads, lowers the Belfry's own channels,
/// and the same flag lowers the lake in a Mirrormere world.
#[test]
fn the_sluice_lever_lowers_mirrormeres_lake() {
    let mut w = world(belfry::SCENARIO, belfry::SEED);
    assert!(depth(&w, "tide_w") > DEEP_M);
    assert!(!water_level::sluice_open(&w, "mirrormere"));
    interact_at(&mut w, [12.0, 0.0, 47.6]);
    assert!(water_level::sluice_open(&w, "mirrormere"));
    assert!(depth(&w, "tide_w") < 0.6, "the channels wade");
    // Pulling it again leaves the sluice open.
    interact_at(&mut w, [12.0, 0.0, 47.6]);
    assert!(water_level::sluice_open(&w, "mirrormere"));
    // World flags outlive the area: carry the flag to Mirrormere and the lake is lower.
    let mut m = world("mirrormere", 60);
    let ford = depth(&m, "ford");
    assert!(ford > DEEP_M);
    m.state.flags.raise(&water_level::sluice_flag("mirrormere"));
    assert!(depth(&m, "ford") < 0.6);
}

/// The Belfry's critical path: bell A, the choir, bell B, the Tide Hall, bell C, bell D, the
/// Choir Knight, the sluice, Choir Litany, and the shortcut. The bot survives it.
#[test]
fn the_belfry_critical_path_is_walkable() {
    let r = walk(belfry::SCENARIO, belfry::SEED, belfry::legs());
    assert_finished(&r);
    let w = &r.world;
    for id in ["bell_a", "bell_b", "bell_c", "bell_d", "lever_sluice"] {
        assert!(
            w.state.flags.is_set(&flags::lever_pulled(id)),
            "{id} was never pulled"
        );
    }
    for door in ["door_t1", "door_g", "door_shaft", "door_return"] {
        assert!(w.state.flags.is_set(&flags::door_open(door)), "{door}");
    }
    assert!(water_level::sluice_open(w, "mirrormere"));
    assert!(
        w.state
            .flags
            .is_set(&flags::pickup_taken("spell_choir_litany"))
    );
    let knight = enemy(w, "choir_knight_loft");
    assert!(w.actors.get(knight).unwrap().dead, "the Choir Knight fell");
    assert!(w.tick < 7_200, "took {} ticks", w.tick);
}

// ---------------------------------------------------------------------------------------
// Goldens

/// Writes `game/tests/replays/m6_sunken_ossuary.script` and `m6_drowned_belfry.script`. See
/// the module comment.
#[test]
#[ignore = "writes game/tests/replays/m6_*.script"]
fn record_the_dungeon_goldens() {
    let r = walk(ossuary::SCENARIO, ossuary::SEED, ossuary::legs());
    assert_finished(&r);
    write_script(
        "m6_sunken_ossuary",
        &script_text(
            "the Sunken Ossuary's critical path: both drain levers, the Flooded Crypt, the vault door, and the shortcut.",
            ossuary::SCENARIO,
            ossuary::SEED,
            &r,
        ),
    );
    let r = walk(belfry::SCENARIO, belfry::SEED, belfry::legs());
    assert_finished(&r);
    write_script(
        "m6_drowned_belfry",
        &script_text(
            "the Drowned Belfry's critical path: four bells, the Choir Knight, the sluice, and the shortcut.",
            belfry::SCENARIO,
            belfry::SEED,
            &r,
        ),
    );
}
