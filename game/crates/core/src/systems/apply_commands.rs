//! Apply commands: resting at a Lantern (see `world_state::lantern::run`), then item and
//! equipment commands and the cycle buttons (see `items::run`). The rest of the commands
//! arrive with the systems that consume them.

use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, input: &TickInput) {
    crate::world_state::lantern::run(world, input);
    crate::items::run(world, input);
}
