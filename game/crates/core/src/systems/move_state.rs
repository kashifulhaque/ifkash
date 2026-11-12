//! Move state machines: advances every actor's move one frame (see `combat::moves::step`).

use crate::arena::Id;
use crate::combat::{crit, moves, poise};
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, input: &TickInput) {
    let tuning = world.locomotion;
    let ids: Vec<Id> = world.actors.iter().map(|(id, _)| id).collect();
    for &id in &ids {
        // A buffered LIGHT near a downed or unaware actor starts a critical hit instead of
        // an attack: `crit::plan` takes the press, `crit::start` runs after the step.
        let crit_plan = crit::plan(world, id, (id == world.player).then_some(input.frame));
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
        if let Some(plan) = crit_plan {
            crit::start(world, id, plan);
        }
    }
    // Poise, stance, kneel, and parried timers, after every actor's move so the order of
    // actors can't matter.
    for id in ids {
        if let Some(actor) = world.actors.get_mut(id) {
            poise::tick(actor, &tuning.poise);
        }
    }
}
