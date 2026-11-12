//! Status and regeneration. Stamina regeneration only, so far.

use crate::combat::dodge::LoadTier;
use crate::combat::stamina;
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    let tuning = world.locomotion;
    // Only the player has equipment (a stand-in tier) until AI actors get movesets.
    let tier = world
        .moves
        .player_moveset
        .as_ref()
        .map_or(LoadTier::Medium, |s| s.load_tier);
    if let Some(actor) = world.actors.get_mut(world.player) {
        stamina::regen(actor, &tuning, tier);
    }
}
