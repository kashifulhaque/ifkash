//! Gestures (WP-088). See docs/game/06-progression-and-systems.md, "Gestures and photo mode".
//!
//! Six gestures: wave, bow, point forward, sit, shrug, and beckon, numbered 0 to 5 in that order
//! (`data/echoes/rules.ron` fixes the bow as number 1). A gesture is `Command::Gesture(n)`, sent
//! from the gesture menu, so it is part of the replay's command list and not a new input.
//!
//! A gesture is a move with no hitboxes: it can't hurt anyone or be hurt differently, and it
//! doesn't use the move table. The simulation only keeps the gesture's number and how long it
//! has left ([`Run`], in `Track::gesture`); the renderer plays the pose from the
//! [`GestureEvent`]s (WASM event kind 58) and the `marks_state` export. There are no gesture
//! clips in the animation library yet, so the renderer builds the poses from the idle clip and
//! bone offsets.
//!
//! A gesture starts only when the player is alive, standing on the ground, not in a move, not
//! on a ladder, not resting, and not in a conversation. It ends when its time runs out (a sit
//! has none: it lasts until the player acts), when the player moves, presses any button, starts
//! a move, leaves the ground, loses HP, dies, or rests, and when the same gesture is chosen
//! again (so choosing Sit twice stands up).

use serde::{Deserialize, Serialize};

use crate::events::{Event, GestureEvent};
use crate::input::{Command, TickInput};
use crate::world::World;
use crate::world_state::quests;

/// How many gestures there are: wave, bow, point forward, sit, shrug, beckon.
pub const COUNT: u8 = 6;
/// The sit gesture, which holds until the player acts.
pub const SIT: u8 = 3;
/// How long each gesture lasts, in ticks (0 holds until cancelled). The spec fixes none of
/// these: two to two and a half seconds each at 30 ticks a second, tuning.
pub const TICKS: [u16; COUNT as usize] = [75, 60, 60, 0, 60, 75];

/// A gesture in progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    /// The gesture's number, 0 to 5.
    pub gesture: u8,
    /// Ticks left; 0 for a gesture that holds.
    pub left: u16,
    /// The player's HP when it started: losing any ends it.
    pub hp: u32,
}

/// The gesture in progress, if any.
pub fn current(world: &World) -> Option<Run> {
    world.state.marks.gesture
}

/// True if the player is in a state where a gesture can start.
fn can_start(world: &World) -> bool {
    let Some(p) = world.actors.get(world.player) else {
        return false;
    };
    !p.dead
        && p.motor.grounded
        && p.moves.current.is_none()
        && world.state.resting.is_none()
        && world.mech.climb.is_none()
        && quests::current(world).is_none()
}

fn emit(world: &mut World, what: u8, gesture: u8) {
    world
        .events
        .push(Event::Gesture(GestureEvent { what, gesture }));
}

fn end(world: &mut World) {
    if let Some(g) = world.state.marks.gesture.take() {
        emit(world, GestureEvent::ENDED, g.gesture);
    }
}

/// One tick: keeps or ends the gesture in progress, then starts the one a command asks for.
/// Starting ignores this tick's input (the menu's tick carries none, and a held key from before
/// it must not cancel the gesture it started); the next tick's does not.
pub fn run(world: &mut World, input: &TickInput) {
    if let Some(g) = world.state.marks.gesture {
        let hp = world.actors.get(world.player).map_or(0, |p| p.hp);
        let f = input.frame;
        let acted = f.move_x != 0 || f.move_z != 0 || f.buttons != 0;
        if !can_start(world) || acted || hp < g.hp {
            end(world);
        } else if g.left == 1 {
            // The last tick of a timed gesture.
            end(world);
        } else if g.left > 1
            && let Some(run) = world.state.marks.gesture.as_mut()
        {
            run.left -= 1;
        }
    }
    for c in &input.commands {
        let Command::Gesture(n) = *c else { continue };
        let Ok(n) = u8::try_from(n) else { continue };
        if n >= COUNT {
            continue;
        }
        // Choosing the gesture that is on ends it; choosing another one switches.
        let same = world.state.marks.gesture.is_some_and(|g| g.gesture == n);
        end(world);
        if same || !can_start(world) {
            continue;
        }
        let hp = world.actors.get(world.player).map_or(0, |p| p.hp);
        world.state.marks.gesture = Some(Run {
            gesture: n,
            left: TICKS[usize::from(n)],
            hp,
        });
        emit(world, GestureEvent::STARTED, n);
    }
}
