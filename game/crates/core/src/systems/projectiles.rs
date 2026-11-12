//! Projectiles, tick system 6 (see `combat::projectile`).

use crate::combat::projectile;
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    projectile::run(world);
}
