//! Player intent: updates the lock-on target, then maps the `ActionFrame` to a planar
//! velocity, facing, sprint, and jump requests (see `actor::targeting::update` and
//! `actor::controller::intent`).

use crate::actor::{controller, targeting};
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, input: &TickInput) {
    let tuning = world.locomotion;
    targeting::update(
        &mut world.actors,
        world.player,
        &world.physics,
        input.frame,
        &tuning.lock_on,
    );
    let target_pos = world
        .actors
        .get(world.player)
        .and_then(|p| p.lock_target)
        .and_then(|id| world.actors.get(id))
        .map(|t| t.pos);
    if let Some(actor) = world.actors.get_mut(world.player) {
        controller::intent(actor, input.frame, target_pos, &tuning);
    }
}
