//! Locomotion: gravity and the Rapier character controller move against the level
//! (see `actor::controller::step`).

use crate::actor::controller;
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    let id = world.player;
    if let Some(actor) = world.actors.get_mut(id) {
        controller::step(actor, &world.physics, &world.locomotion);
    }
}
