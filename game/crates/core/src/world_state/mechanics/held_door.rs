//! Doors that open while the player holds enough of a set of key items (WP-067; docs/game/02-world.md,
//! "Gates and progression": the Spire Gate opens when the player holds any two of the four Great
//! Sigils).
//!
//! A [`DoorKind::Held`](lf_content::scenario::DoorKind::Held) door is opened like a keyed one,
//! with `INTERACT` beside it, but it takes nothing: the Sigils stay in the inventory, and the
//! door stays open afterwards (the flag `door.<id>.open`). The Great Sigils are distinct items, so
//! "two of the four" is two different items from the list, not a stack of two.

use crate::world::World;

/// How many different items of `items` the player holds, at least one of each.
pub fn count_held(world: &World, items: &[String]) -> usize {
    let Some(p) = world.actors.get(world.player) else {
        return 0;
    };
    items
        .iter()
        .filter(|id| {
            world
                .items
                .num_of(id)
                .is_some_and(|num| p.inventory.count(num) > 0)
        })
        .count()
}

/// Whether the player holds at least `need` different items from `items`.
pub fn holds(world: &World, items: &[String], need: u8) -> bool {
    count_held(world, items) >= usize::from(need)
}
