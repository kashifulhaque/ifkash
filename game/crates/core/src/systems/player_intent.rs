//! Player intent: maps the `ActionFrame` to a planar velocity, facing, sprint, and jump
//! requests (see `actor::controller::intent`).

use crate::actor::controller;
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, input: &TickInput) {
    let tuning = world.locomotion;
    if let Some(actor) = world.actors.get_mut(world.player) {
        controller::intent(actor, input.frame, &tuning);
    }
}
