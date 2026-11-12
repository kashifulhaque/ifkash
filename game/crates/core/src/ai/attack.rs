//! What an enemy brings to the damage pipeline: its attack rating, its guard, and its
//! stamina (docs/game/03-combat.md, "Damage dealt" and "Guard and guard counter").
//!
//! The player's attack rating comes from a weapon and attributes. An enemy has neither, so
//! its definition's `attack` is the physical attack rating of every hit it lands, and the
//! hitbox's `motion_value` scales it as usual. An enemy with `attack: 0` deals the
//! formula's minimum of 1.

use lf_content::GuardStats;

use crate::actor::Actor;
use crate::arena::Id;
use crate::combat::damage::AttackRating;
use crate::combat::dodge::LoadTier;
use crate::combat::stamina;
use crate::world::World;

/// `a`'s attack rating if it is an AI enemy with an `attack`; `None` otherwise.
pub fn rating(world: &World, id: Id, a: &Actor) -> Option<AttackRating> {
    let d = &world.ai.def(a.ai.as_ref()?.def).data;
    // An Echo that was bowed to hits harder (WP-059); everyone else's 100 changes nothing.
    let percent = u64::from(super::echo::damage_percent(world, id));
    (d.attack > 0 || d.element_attack.is_some()).then(|| {
        // Aether, Fire, Storm, Radiance: the order `AttackRating::elem_milli` uses.
        let mut elem_milli = [0; 4];
        if let Some((e, rating)) = d.element_attack {
            elem_milli[e as usize] = u64::from(rating) * 1000 * percent / 100;
        }
        AttackRating {
            // The rating is in thousandths.
            phys_milli: u64::from(d.attack) * 1000 * percent / 100,
            elem_milli,
            ..AttackRating::default()
        }
    })
}

/// The block stats of an enemy that can guard.
pub fn guard_stats(world: &World, a: &Actor) -> Option<GuardStats> {
    world.ai.def(a.ai.as_ref()?.def).data.guard
}

/// Regenerates an enemy's stamina like the player's, if it has any (it only matters to a
/// guarding enemy).
pub fn regen_stamina(world: &mut World, id: Id) {
    let tuning = world.locomotion;
    if let Some(a) = world.actors.get_mut(id)
        && a.max_stamina > 0.0
    {
        stamina::regen(a, &tuning, LoadTier::Medium);
    }
}

/// True if `a` and `b` are on the same side, so neither's swings or arrows hurt the other.
/// Enemies don't hurt each other, so a crowd's swings and arrows pass through its own ranks
/// (WP-032). The player and the Echoes are the other side (WP-059): an Echo's attacks pass
/// through the player and the other Echoes, and the player's through the Echoes, while an
/// Echo and an enemy hurt each other.
pub fn allied(a: &Actor, b: &Actor) -> bool {
    if a.is_ally() || b.is_ally() {
        return (a.is_ally() || a.is_player) && (b.is_ally() || b.is_player);
    }
    a.ai.is_some() && b.ai.is_some()
}
