//! Locomotion: gravity and the Rapier character controller move the player and every AI
//! actor against the level (see `actor::controller::step`).

use crate::actor::controller;
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    let id = world.player;
    if let Some(actor) = world.actors.get_mut(id) {
        controller::step(actor, &world.physics, &world.locomotion);
    }
    // AI actors share the player's capsule and controller tuning. Scripted dummies stand.
    let ai: Vec<_> = world
        .actors
        .iter()
        .filter(|(_, a)| a.ai.is_some())
        .map(|(id, _)| id)
        .collect();
    for id in ai {
        if let Some(actor) = world.actors.get_mut(id) {
            controller::step(actor, &world.physics, &world.locomotion);
        }
    }
}
