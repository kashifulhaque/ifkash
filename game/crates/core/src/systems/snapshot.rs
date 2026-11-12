//! Snapshot. Will also write the render snapshot; for now it only advances the tick.

use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    world.tick += 1;
}
