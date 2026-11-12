//! Sleeping enemies (WP-061; docs/game/02-world.md, D2: "lowering water opens rooms but
//! wakes the Drowned").
//!
//! An enemy spawn with `wakes_on: Some(flag)` stands still, unaware and not thinking, until
//! the world flag is set. While it sleeps it can be backstabbed. The flag is one a flag
//! lever raises ([`super::bell`]), so the wake is permanent and saved with the world.

use crate::arena::Id;
use crate::world::World;

/// True if enemy `id` is asleep: its spawn names a wake flag that isn't set yet. Summoned
/// and boss-spawned enemies (no spawn, or a spawn without a flag) never sleep.
pub fn asleep(world: &World, id: Id) -> bool {
    let Some(b) = world.actors.get(id).and_then(|a| a.ai.as_ref()) else {
        return false;
    };
    world
        .layout
        .enemies
        .get(usize::from(b.spawn))
        .and_then(|s| s.wakes_on.as_deref())
        .is_some_and(|flag| !world.state.flags.is_set(flag))
}
