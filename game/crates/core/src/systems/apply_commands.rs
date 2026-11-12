//! Apply commands: resting at a Lantern (see `world_state::lantern::run`), then item and
//! equipment commands and the cycle buttons (see `items::run`), then the world mechanics
//! (see `world_state::mechanics::run`). The rest of the commands
//! arrive with the systems that consume them.

use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, input: &TickInput) {
    // On a ladder the item commands and buttons are blocked (WP-042).
    let masked = crate::world_state::mechanics::mask_input(world, input);
    let fed = masked.as_ref().unwrap_or(input);
    crate::world_state::lantern::run(world, fed);
    crate::items::run(world, fed);
    crate::world_state::mechanics::run(world, input);
}
