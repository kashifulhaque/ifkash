//! Locomotion. Placeholder until WP-010 (physics): plain Euler integration, no collision.

use crate::input::TickInput;
use crate::world::{DT, World};

pub fn run(world: &mut World, _input: &TickInput) {
    let id = world.player;
    if let Some(a) = world.actors.get_mut(id) {
        for i in 0..3 {
            a.pos[i] += a.vel[i] * DT;
        }
    }
}
