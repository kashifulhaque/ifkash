//! The disguise rule (docs/game/02-world.md, treasure chest row: one mimic chest exists, in
//! the castle armory). An enemy with `disguise` is a chest while it is Idle: [`dormant`]
//! is true, and `ai::run` skips its perception and behaviors. [`wake`] ends the disguise
//! on any of three triggers:
//!
//! - approach: the player comes within `wake_m`;
//! - interaction: the player holds `INTERACT` within `interact_m` (tries to open it);
//! - a hit: it has lost HP since the fight began.
//!
//! It wakes straight into the Aware notice, then Combat. When it leashes or loses the
//! player it walks home, heals, and goes Idle again, so it is a chest once more.
//! Dormancy is derived from the awareness state, so the rule adds no state to hash.

use crate::ai::perception::dist_xz;
use crate::ai::{Awareness, engage};
use crate::arena::Id;
use crate::input::buttons;
use crate::world::World;

/// True if `id` is a disguised enemy lying dormant.
pub fn dormant(world: &World, id: Id) -> bool {
    let Some(b) = world.actors.get(id).and_then(|a| a.ai.as_ref()) else {
        return false;
    };
    world.ai.def(b.def).data.disguise.is_some() && b.awareness == Awareness::Idle
}

/// Checks the wake triggers for a dormant `id` and wakes it if one holds.
pub fn wake(world: &mut World, id: Id) {
    let player = world.player;
    let (Some(a), Some(p)) = (world.actors.get(id), world.actors.get(player)) else {
        return;
    };
    let Some(b) = a.ai.as_ref() else {
        return;
    };
    let Some(rule) = world.ai.def(b.def).data.disguise else {
        return;
    };
    if p.dead {
        return;
    }
    let d = dist_xz(a.pos, p.pos);
    // `prev_buttons` is this tick's held buttons: the controller has run (system 2).
    let opening = p.motor.prev_buttons & buttons::INTERACT != 0 && d <= rule.interact_m;
    let hurt = a.hp < b.last_hp;
    if (d <= rule.wake_m || opening || hurt)
        && let Some(tuning) = world.ai.tuning
    {
        engage(world, id, player, &tuning);
    }
}
