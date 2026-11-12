//! Player intent. Placeholder until WP-010 (physics): maps the move stick to a desired
//! planar velocity and a facing. Later work packages add buttons and action requests.

use crate::input::TickInput;
use crate::math::dmath::{atan2, sqrt};
use crate::world::World;

// Placeholder speeds. Spec: docs/game/01-architecture.md, "Inputs" table. They move to
// `game/data` in WP-010.
/// Walk speed in m/s (stick magnitude below `WALK_BELOW`).
pub const WALK_SPEED: f32 = 1.6;
/// Run speed in m/s (stick magnitude above `RUN_ABOVE`).
pub const RUN_SPEED: f32 = 4.5;
/// Stick magnitude (0..=127 units) below which the player walks.
pub const WALK_BELOW: f32 = 40.0;
/// Stick magnitude above which the player runs.
pub const RUN_ABOVE: f32 = 90.0;

pub fn run(world: &mut World, input: &TickInput) {
    let id = world.player;
    let Some(actor) = world.actors.get_mut(id) else {
        return;
    };
    let x = f32::from(input.frame.move_x);
    let z = f32::from(input.frame.move_z);
    let mag = sqrt(x * x + z * z);
    if mag < 1.0 {
        actor.vel = [0.0, 0.0, 0.0];
        return;
    }
    let speed = if mag > RUN_ABOVE {
        RUN_SPEED
    } else if mag < WALK_BELOW {
        WALK_SPEED
    } else {
        // Linear blend between walk and run across the dead band.
        let t = (mag - WALK_BELOW) / (RUN_ABOVE - WALK_BELOW);
        WALK_SPEED + (RUN_SPEED - WALK_SPEED) * t
    };
    actor.vel = [x / mag * speed, 0.0, z / mag * speed];
    actor.yaw = atan2(x, z);
}
