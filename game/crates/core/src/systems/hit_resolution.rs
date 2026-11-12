//! Hit resolution: active hitboxes against hurtboxes (see `combat::resolve::run`).

use crate::combat::resolve;
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    resolve::run(world);
}
