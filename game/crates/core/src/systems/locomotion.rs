//! Locomotion: gravity and the Rapier character controller move the player and every AI
//! actor against the level (see `actor::controller::step`).

use crate::actor::controller;
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    let id = world.player;
    crate::world_state::mechanics::before_locomotion(world);
    // A player on a ladder is moved by the ladder (WP-042).
    if !crate::world_state::mechanics::climbing(world)
        && let Some(actor) = world.actors.get_mut(id)
    {
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
        // A flyer skips gravity and the controller (WP-036, `ai::special::flying`).
        if crate::ai::special::flying::flies(world, id) {
            crate::ai::special::flying::step(world, id);
        } else if let Some(actor) = world.actors.get_mut(id) {
            controller::step(actor, &world.physics, &world.locomotion);
        }
    }
}
