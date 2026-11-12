//! Apply commands: item and equipment commands and the cycle buttons (see `items::run`).
//! The rest of the commands arrive with the systems that consume them.

use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, input: &TickInput) {
    crate::items::run(world, input);
}
