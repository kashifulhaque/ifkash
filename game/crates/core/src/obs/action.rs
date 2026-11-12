//! The default `MultiDiscrete` action space (docs/game/09-rl-environment.md, "Actions"):
//! five heads decoded into one `ActionFrame` per tick of a step.
//!
//! | Head | Size | Values |
//! |---|---|---|
//! | `move_dir` | 9 | None, forward, forward-right, right, back-right, back, back-left, left, forward-left |
//! | `move_speed` | 2 | Walk, run |
//! | `action` | 10 | None, light, heavy (tap), heavy (hold), guard (hold), dodge, Art, use item, jump, interact |
//! | `lock` | 3 | Keep, toggle, switch to the next target |
//! | `quick` | 4 | None, cycle item, cycle spell, two-hand toggle |
//!
//! Movement is relative to the lock target when locked, else to the player's facing.
//!
//! Buttons: a tap presses on the step's first tick only and releases for the rest, so a
//! step of two or more ticks always leaves a press edge for the next one. Guard and heavy
//! (hold) are held for every tick of the step. Dodge taps, except when the previous step
//! also chose dodge: then it is held for the whole step, so dodge chosen on consecutive
//! steps becomes a sprint once held past `sprint_delay_ticks`, as for a human.
//!
//! With `frame_skip` 1 a tap presses on every tick, so the same tap chosen on two steps in
//! a row is one press; use `frame_skip` 2 or more (the default is 2).

use crate::input::{ActionFrame, buttons};
use crate::items::FlaskKind;
use crate::math::dmath::{cos, sin};
use crate::world::World;

use super::target_id;

/// The size of each head, in order: `move_dir`, `move_speed`, `action`, `lock`, `quick`.
pub const HEADS: [u8; 5] = [9, 2, 10, 3, 4];

/// `action` head values.
pub mod act {
    pub const NONE: u8 = 0;
    pub const LIGHT: u8 = 1;
    pub const HEAVY_TAP: u8 = 2;
    pub const HEAVY_HOLD: u8 = 3;
    pub const GUARD: u8 = 4;
    pub const DODGE: u8 = 5;
    pub const ART: u8 = 6;
    pub const USE_ITEM: u8 = 7;
    pub const JUMP: u8 = 8;
    pub const INTERACT: u8 = 9;
}

const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;

/// Remembers the previous step's `action`, for dodge-into-sprint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActionDecoder {
    prev_action: u8,
}

impl ActionDecoder {
    /// Forgets the previous step (call it on reset).
    pub fn reset(&mut self) {
        self.prev_action = act::NONE;
    }

    /// The frame for tick `i` (from 0) of a step that chose `heads`, from the world at
    /// the start of the step. Out-of-range head values count as 0. Call
    /// [`ActionDecoder::finish_step`] after the step's last tick.
    pub fn frame(&self, world: &World, heads: [u8; 5], i: u32) -> ActionFrame {
        let h = |k: usize| if heads[k] < HEADS[k] { heads[k] } else { 0 };
        let (move_x, move_z) = stick(world, h(0), h(1));
        let first = i == 0;
        let tap = |mask: u16| if first { mask } else { 0 };
        let mut b = match h(2) {
            act::LIGHT => tap(buttons::LIGHT),
            act::HEAVY_TAP => tap(buttons::HEAVY),
            act::HEAVY_HOLD => buttons::HEAVY,
            act::GUARD => buttons::GUARD,
            act::DODGE if self.prev_action == act::DODGE => buttons::DODGE,
            act::DODGE => tap(buttons::DODGE),
            act::ART => tap(buttons::ART),
            act::USE_ITEM => tap(buttons::USE_ITEM),
            act::JUMP => tap(buttons::JUMP),
            act::INTERACT => tap(buttons::INTERACT),
            _ => 0,
        };
        b |= match h(3) {
            1 => tap(buttons::LOCK),
            2 => tap(buttons::SWITCH_R),
            _ => 0,
        };
        b |= match h(4) {
            1 => tap(buttons::CYCLE_ITEM),
            2 => tap(buttons::CYCLE_SPELL),
            3 => tap(buttons::TWO_HAND),
            _ => 0,
        };
        ActionFrame {
            move_x,
            move_z,
            buttons: b,
        }
    }

    /// Records the step's `action` head for the next step.
    pub fn finish_step(&mut self, heads: [u8; 5]) {
        self.prev_action = if heads[2] < HEADS[2] { heads[2] } else { 0 };
    }
}

/// The world-space stick for `move_dir` and `move_speed`: relative to the bearing to the
/// lock target when locked, else to the player's facing.
fn stick(world: &World, dir: u8, speed: u8) -> (i8, i8) {
    let Some(p) = world.actors.get(world.player) else {
        return (0, 0);
    };
    if dir == 0 {
        return (0, 0);
    }
    let reference = p
        .lock_target
        .and(target_id(world))
        .and_then(|t| world.actors.get(t))
        .map_or(p.yaw, |t| {
            crate::actor::targeting::bearing(p.pos, t.pos, p.yaw)
        });
    // Clockwise from forward in 45 degree steps; yaw grows toward the left, so turning
    // right subtracts.
    let offset = -f32::from(dir - 1) * 45.0 * DEG_TO_RAD;
    let yaw = reference + offset;
    // A walk is half the walk threshold (docs/game/01-architecture.md, "ActionFrame":
    // magnitudes below `walk_below` walk); a run is a full stick.
    let mag = if speed == 0 {
        (world.locomotion.walk_below * 0.5).max(2.0)
    } else {
        127.0
    };
    let x = (sin(yaw) * mag).round().clamp(-127.0, 127.0) as i8;
    let z = (cos(yaw) * mag).round().clamp(-127.0, 127.0) as i8;
    (x, z)
}

/// The advisory mask per head, concatenated in head order (28 values): `true` means the
/// value is worth choosing. Attacks, Art, and dodge are masked with no stamina, use item
/// with no charges in the selected flask, and switching targets when not locked.
/// Unmasked illegal actions do nothing, as they would for a human.
pub fn action_mask(world: &World) -> [bool; 28] {
    let mut m = [true; 28];
    let Some(p) = world.actors.get(world.player) else {
        return m;
    };
    let base = usize::from(HEADS[0] + HEADS[1]);
    if p.stamina <= 0.0 {
        for a in [
            act::LIGHT,
            act::HEAVY_TAP,
            act::HEAVY_HOLD,
            act::DODGE,
            act::ART,
        ] {
            m[base + usize::from(a)] = false;
        }
    }
    let charges = world
        .moves
        .flask
        .as_ref()
        .and_then(|rules| p.inventory.selected_flask(rules))
        .map(|kind| match kind {
            FlaskKind::Vigil => p.inventory.flasks.vigil,
            FlaskKind::Lucid => p.inventory.flasks.lucid,
        });
    if charges == Some(0) {
        m[base + usize::from(act::USE_ITEM)] = false;
    }
    let lock = base + usize::from(HEADS[2]);
    if p.lock_target.is_none() {
        m[lock + 2] = false;
    }
    m
}
