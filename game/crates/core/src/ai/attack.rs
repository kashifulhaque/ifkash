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
pub fn rating(world: &World, a: &Actor) -> Option<AttackRating> {
    let d = &world.ai.defs[usize::from(a.ai.as_ref()?.def)].data;
    (d.attack > 0).then(|| AttackRating {
        // The rating is in thousandths.
        phys_milli: u64::from(d.attack) * 1000,
        ..AttackRating::default()
    })
}

/// The block stats of an enemy that can guard.
pub fn guard_stats(world: &World, a: &Actor) -> Option<GuardStats> {
    world.ai.defs[usize::from(a.ai.as_ref()?.def)].data.guard
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

/// True if `a` and `b` are both AI enemies: enemies don't hurt each other, so a crowd's
/// swings and arrows pass through its own ranks (WP-032).
pub fn allied(a: &Actor, b: &Actor) -> bool {
    a.ai.is_some() && b.ai.is_some()
}
