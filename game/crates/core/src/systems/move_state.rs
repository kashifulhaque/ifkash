//! Move state machines: advances every actor's move one frame (see `combat::moves::step`).

use crate::arena::Id;
use crate::combat::moves;
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, input: &TickInput) {
    let tuning = world.locomotion;
    let ids: Vec<Id> = world.actors.iter().map(|(id, _)| id).collect();
    for id in ids {
        let lock_pos = world
            .actors
            .get(id)
            .and_then(|a| a.lock_target)
            .and_then(|t| world.actors.get(t))
            .map(|t| t.pos);
        let is_player = id == world.player;
        // Only the player has input and a moveset until AI lands.
        let frame = is_player.then_some(input.frame);
        let set = if is_player {
            world.moves.player_moveset.as_ref()
        } else {
            None
        };
        if let Some(actor) = world.actors.get_mut(id) {
            moves::step(actor, frame, lock_pos, &world.moves, set, &tuning);
        }
    }
}
