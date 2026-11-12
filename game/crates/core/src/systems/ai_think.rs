//! AI think: the scripted dummies' attack loops (`combat::dummy`), then every AI actor's
//! perception, decisions, and behaviors (`ai::run`).

use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    crate::combat::dummy::run(world);
    crate::ai::run(world);
}
