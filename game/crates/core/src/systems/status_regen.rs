//! Status and regeneration. Stamina regeneration only, so far.

use crate::actor::controller;
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    let tuning = world.locomotion;
    if let Some(actor) = world.actors.get_mut(world.player) {
        controller::regen(actor, &tuning);
    }
}
