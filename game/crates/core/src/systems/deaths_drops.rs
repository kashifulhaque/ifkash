//! Deaths and drops: the player's death, Remnant, and respawn (see
//! `world_state::death::run`). Enemy deaths and drops arrive with WP-023 and WP-030.

use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    crate::world_state::death::run(world);
}
