//! Status and regeneration: status meters, effects, and buffs (`combat::status`), then
//! stamina regeneration.

use crate::combat::dodge::LoadTier;
use crate::combat::{stamina, status};
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    status::tick(world);
    let tuning = world.locomotion;
    // Only the player has equipment (a stand-in tier) until AI actors get movesets.
    let tier = world
        .moves
        .player_moveset
        .as_ref()
        .map_or(LoadTier::Medium, |s| s.load_tier);
    if let Some(actor) = world.actors.get_mut(world.player) {
        let before = actor.stamina;
        stamina::regen(actor, &tuning, tier);
        // Quickening Charm: the tick's gain is larger (WP-057).
        let more = world
            .items
            .gear
            .stamina_regen_percent(&actor.inventory.equipment);
        if more > 0 {
            let gain = actor.stamina - before;
            actor.stamina = (before + gain * (100 + more) as f32 / 100.0).min(actor.max_stamina);
        }
        // Rime cuts regeneration (a scaled share of this tick's gain).
        if actor.status.chilled() {
            let pct = f32::from(world.status.rules.rime.stamina_regen_percent) / 100.0;
            actor.stamina = before + (actor.stamina - before) * pct;
        }
    }
}
