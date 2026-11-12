//! Tests for world_state::mechanics, on the `mechanics_yard` level.

use lf_content::Content;

use crate::actor::Actor;
use crate::events::{Event, MechanicEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

fn yard() -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(42, &content, "mechanics_yard").expect("mechanics_yard exists")
}

/// Drives a world and keeps every frame and event.
struct Bot {
    w: World,
    frames: Vec<ActionFrame>,
    events: Vec<Event>,
}

impl Bot {
    fn new() -> Self {
        Self {
            w: yard(),
            frames: Vec::new(),
            events: Vec::new(),
        }
    }

    fn player(&self) -> &Actor {
        self.w.actors.get(self.w.player).unwrap()
    }

    fn tick(&mut self, move_x: i8, move_z: i8, buttons: u16) {
        let frame = ActionFrame {
            move_x,
            move_z,
            buttons,
        };
        self.frames.push(frame);
        tick(
            &mut self.w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        self.events.extend(self.w.events.drain());
    }

    fn idle(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.tick(0, 0, 0);
        }
    }

    /// Walks the stick toward (x, z) until the feet are within 0.3 m.
    fn goto(&mut self, x: f32, z: f32) {
        for _ in 0..900 {
            let p = self.player().pos;
            let (dx, dz) = (x - p[0], z - p[2]);
            let d = crate::math::dmath::sqrt(dx * dx + dz * dz);
            if d < 0.3 {
                return;
            }
            let mag = if d < 1.0 { 50.0 } else { 127.0 };
            self.tick((dx / d * mag) as i8, (dz / d * mag) as i8, 0);
        }
        panic!("never reached ({x}, {z}); at {:?}", self.player().pos);
    }

    /// Presses a button for one tick, then releases it.
    fn press(&mut self, mask: u16) {
        self.tick(0, 0, mask);
        self.tick(0, 0, 0);
    }

    fn mechanic_kinds(&self) -> Vec<(u8, u16)> {
        self.events
            .iter()
            .filter_map(|e| match e {
                Event::Mechanic(m) => Some((m.kind, m.index)),
                _ => None,
            })
            .collect()
    }

    fn saw(&self, kind: u8) -> bool {
        self.mechanic_kinds().iter().any(|&(k, _)| k == kind)
    }
}

/// The route the golden replay walks: ride the lift, climb the ladder, open the one-way
/// door, reveal the hollow wall, and take what is behind it.
fn golden_route(b: &mut Bot) {
    // The lift: stand on its east edge and pull the lever beside it.
    b.goto(-6.9, 10.5);
    b.press(buttons::INTERACT);
    b.idle(80);
    assert!(
        (b.player().pos[1] - 3.0).abs() < 0.1,
        "{:?}",
        b.player().pos
    );
    assert!(b.saw(MechanicEvent::LIFT_STARTED) && b.saw(MechanicEvent::LIFT_ARRIVED));
    b.goto(-8.0, 14.0);
    // Down onto the ground, then up the ladder.
    b.goto(-2.0, 13.0);
    b.goto(0.0, 10.0);
    b.goto(8.0, 10.5);
    b.goto(8.0, 11.4);
    b.press(buttons::INTERACT);
    assert!(
        b.w.mech.climb.is_some(),
        "{:?} {:?}",
        b.player().pos,
        b.player().motor
    );
    for _ in 0..400 {
        if b.w.mech.climb.is_none() {
            break;
        }
        b.tick(0, 127, 0);
    }
    assert!(b.w.mech.climb.is_none(), "never reached the top");
    assert!(
        (b.player().pos[1] - 3.0).abs() < 0.1,
        "{:?}",
        b.player().pos
    );
    b.idle(5);
    // Off the terrace, to the one-way door.
    b.goto(2.0, 10.0);
    b.goto(0.0, 22.0);
    b.goto(0.0, 24.0);
    b.press(buttons::INTERACT);
    assert!(b.w.state.flags.is_set(&flags::door_open("door_oneway")));
    b.goto(0.0, 28.5);
    // The hollow wall.
    b.goto(10.0, 28.5);
    b.goto(10.0, 30.4);
    b.press(buttons::LIGHT);
    b.idle(40);
    assert!(
        b.w.state.flags.is_set(&flags::wall_revealed("wall_closet")),
        "the swing never touched the wall; at {:?}",
        b.player().pos
    );
    b.goto(10.0, 34.0);
    b.press(buttons::INTERACT);
    b.idle(5);
}

#[test]
fn the_golden_route_rides_climbs_opens_and_reveals() {
    let mut b = Bot::new();
    golden_route(&mut b);
    for kind in [
        MechanicEvent::LIFT_STARTED,
        MechanicEvent::LIFT_ARRIVED,
        MechanicEvent::LADDER_GRABBED,
        MechanicEvent::LADDER_RELEASED,
        MechanicEvent::DOOR_OPENED,
        MechanicEvent::WALL_REVEALED,
    ] {
        assert!(
            b.saw(kind),
            "no event of kind {kind}: {:?}",
            b.mechanic_kinds()
        );
    }
    assert!(b.w.state.flags.is_set(&flags::pickup_taken("salts_closet")));
    assert!(b.player().hp > 0);
}

/// Records the golden script: `cargo test -p lf-core record_the_yard_golden -- --ignored`,
/// then `lf-tools replay record-script tests/replays/m4_mechanics_yard.script --out
/// tests/replays/m4_mechanics_yard.lfr`.
#[test]
#[ignore = "writes game/tests/replays/m4_mechanics_yard.script"]
fn record_the_yard_golden() {
    let mut b = Bot::new();
    golden_route(&mut b);
    let mut text = String::from(
        "# WP-042 golden: ride the lift, climb the ladder, open the one-way door, and reveal\n\
         # the hollow wall on mechanics_yard. Generated by the ignored test\n\
         # `record_the_yard_golden` in game/crates/core/src/world_state/mechanics/tests.rs.\n\
         seed 42\nscenario mechanics_yard\n",
    );
    let names = [(buttons::LIGHT, "LIGHT"), (buttons::INTERACT, "INTERACT")];
    let mut i = 0;
    while i < b.frames.len() {
        let f = b.frames[i];
        let mut n = 1;
        while i + n < b.frames.len() && b.frames[i + n] == f {
            n += 1;
        }
        let mask: Vec<&str> = names
            .iter()
            .filter(|(m, _)| f.buttons & m != 0)
            .map(|(_, s)| *s)
            .collect();
        text.push_str(&format!("{n} {} {}", f.move_x, f.move_z));
        if !mask.is_empty() {
            text.push(' ');
            text.push_str(&mask.join("+"));
        }
        text.push('\n');
        i += n;
    }
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/replays/m4_mechanics_yard.script"
    );
    std::fs::write(path, text).unwrap();
}

#[test]
fn a_one_way_door_stays_shut_from_the_wrong_side() {
    let mut b = Bot::new();
    // The gap's far side: walk around through the open space east of the wall? The wall
    // line is solid, so stand the player there directly.
    let id = b.w.player;
    b.w.actors.get_mut(id).unwrap().pos = [0.0, 0.0, 28.0];
    b.w.actors.get_mut(id).unwrap().motor.grounded = true;
    b.idle(3);
    b.press(buttons::INTERACT);
    assert!(!b.w.state.flags.is_set(&flags::door_open("door_oneway")));
    assert!(b.saw(MechanicEvent::DOOR_ONE_WAY));
}

#[test]
fn a_locked_door_needs_its_lever_and_the_lever_toggles_it() {
    let mut b = Bot::new();
    b.goto(-6.0, 24.0);
    b.press(buttons::INTERACT);
    assert!(b.saw(MechanicEvent::DOOR_LOCKED));
    b.goto(-4.0, 22.2);
    b.press(buttons::INTERACT);
    assert!(b.w.state.flags.is_set(&flags::door_open("gate_locked")));
    b.press(buttons::INTERACT);
    assert!(!b.w.state.flags.is_set(&flags::door_open("gate_locked")));
}

#[test]
fn a_door_blocks_until_it_opens() {
    let mut b = Bot::new();
    let closed = b.w.physics.raycast([0.0, 1.0, 20.0], [0.0, 0.0, 1.0], 20.0);
    assert!((closed.unwrap() - 5.75).abs() < 0.01, "{closed:?}");
    b.goto(0.0, 24.0);
    b.press(buttons::INTERACT);
    assert!(
        b.w.physics
            .raycast([0.0, 1.0, 20.0], [0.0, 0.0, 1.0], 20.0)
            .is_none()
    );
}

#[test]
fn the_key_opens_the_vault_and_is_used_up() {
    let mut b = Bot::new();
    b.goto(16.0, 1.0);
    b.press(buttons::INTERACT);
    let key = b.w.items.num_of("itm_yard_key").unwrap();
    assert_eq!(b.player().inventory.count(key), 1);
    b.goto(18.0, 6.0);
    b.press(buttons::INTERACT);
    assert!(b.w.state.flags.is_set(&flags::door_open("door_vault")));
    assert_eq!(b.player().inventory.count(key), 0);
    // Without a key the door says so.
    let mut c = Bot::new();
    c.goto(18.0, 6.0);
    c.press(buttons::INTERACT);
    assert!(c.saw(MechanicEvent::DOOR_NEEDS_KEY));
    // The chest pays out its item and Glimmer once.
    b.goto(21.4, 6.0);
    let before = b.player().inventory.glimmer;
    b.press(buttons::INTERACT);
    assert_eq!(b.player().inventory.glimmer, before + 200);
    let clot = b.w.items.num_of("itm_clotweed").unwrap();
    assert_eq!(b.player().inventory.count(clot), 3);
    b.press(buttons::INTERACT);
    assert_eq!(b.player().inventory.glimmer, before + 200);
}

#[test]
fn a_message_is_read_with_interact() {
    let mut b = Bot::new();
    b.goto(3.0, 3.0);
    b.press(buttons::INTERACT);
    assert!(b.w.state.flags.is_set(&flags::message_read("msg_welcome")));
    assert!(b.events.iter().any(|e| matches!(e, Event::Message(_))));
}

#[test]
fn a_kick_down_ladder_can_only_be_climbed_after_the_kick() {
    let mut b = Bot::new();
    // At the foot of the raised ladder nothing grabs.
    let id = b.w.player;
    b.w.actors.get_mut(id).unwrap().pos = [12.6, 0.0, 16.0];
    b.w.actors.get_mut(id).unwrap().motor.grounded = true;
    b.idle(3);
    b.press(buttons::INTERACT);
    assert!(b.w.mech.climb.is_none());
    // From the top, the kick drops it, then it grabs.
    let top = [11.7, 3.0, 16.0];
    b.w.actors.get_mut(id).unwrap().pos = top;
    b.idle(3);
    b.press(buttons::INTERACT);
    assert!(b.saw(MechanicEvent::LADDER_KICKED));
    b.press(buttons::INTERACT);
    assert!(b.w.mech.climb.is_some());
    // No attacking on a ladder.
    b.tick(0, 0, buttons::LIGHT);
    assert!(b.player().moves.current.is_none());
    // Jumping lets go.
    b.tick(0, 0, buttons::JUMP);
    assert!(b.w.mech.climb.is_none());
    assert!(b.saw(MechanicEvent::LADDER_RELEASED));
}

#[test]
fn shallow_water_and_snow_slow_the_player() {
    let speed = |at: [f32; 3]| {
        let mut b = Bot::new();
        let id = b.w.player;
        b.w.actors.get_mut(id).unwrap().pos = at;
        b.w.actors.get_mut(id).unwrap().motor.grounded = true;
        b.idle(3);
        let x0 = b.player().pos[0];
        for _ in 0..10 {
            b.tick(127, 0, 0);
        }
        b.player().pos[0] - x0
    };
    let open = speed([-4.0, 0.0, 4.0]);
    let wade = speed([-16.0, 0.0, -6.0]);
    let drift = speed([15.0, 0.0, 20.0]);
    assert!((wade / open - 0.8).abs() < 0.03, "{wade} {open}");
    assert!((drift / open - 0.75).abs() < 0.03, "{drift} {open}");
}

#[test]
fn deep_water_is_a_wall_and_a_lever_raises_the_pond_into_one() {
    let mut b = Bot::new();
    // The mere (depth 1.8) from its west side.
    let id = b.w.player;
    b.w.actors.get_mut(id).unwrap().pos = [8.0, 0.0, -6.0];
    b.w.actors.get_mut(id).unwrap().motor.grounded = true;
    for _ in 0..90 {
        b.tick(-127, 0, 0);
    }
    // Walking in -x here is away; walk +x instead, into the mere's west edge at x = 11.
    for _ in 0..90 {
        b.tick(127, 0, 0);
    }
    assert!(b.player().pos[0] < 11.0, "{:?}", b.player().pos);
    // The pond starts shallow and wadeable, and a lever raises it to 2 m.
    assert_eq!(super::pool_depth(&b.w, 0), 0.4);
    b.goto(-9.5, -6.0);
    b.press(buttons::INTERACT);
    assert_eq!(super::pool_depth(&b.w, 0), 2.0);
}

#[test]
fn blight_water_builds_blight() {
    let mut b = Bot::new();
    let id = b.w.player;
    b.w.actors.get_mut(id).unwrap().pos = [0.0, 0.0, -12.0];
    b.w.actors.get_mut(id).unwrap().motor.grounded = true;
    b.idle(20);
    assert!(
        b.player().status.meter_milli[1] > 0
            || b.player()
                .status
                .active_ticks(lf_content::moves::StatusKind::Blight)
                > 0
    );
}

#[test]
fn a_world_without_mechanics_hashes_as_before() {
    // The mechanics state is left out of the hash until a lift moves or a ladder is held.
    let w = yard();
    assert!(w.mech.is_inert());
    let mut moved = w.clone();
    moved.mech.lifts[0].target = 1;
    assert_ne!(w.state_hash(), moved.state_hash());
}
