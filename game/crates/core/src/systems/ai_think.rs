//! AI think. Only the scripted dummies act until WP-030 fills it in.

use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    // The scenario dummies' scripted attacks stand in for AI until WP-030.
    crate::combat::dummy::run(world);
}
