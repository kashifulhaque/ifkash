//! World logic. Triggers, doors, and so on arrive in later work packages.

use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, input: &TickInput) {
    crate::pickups::run(world);
    crate::world_state::mechanics::after_hits(world);
    // Gestures and Marks (WP-088). Last, so it sees this tick's events and flags.
    crate::world_state::marks::run(world, input);
    // Placeholder: one draw per tick so the RNG state, and therefore the state hash,
    // evolves. Remove when real world logic lands.
    let _ = world.rng.world.next_u32();
}
