//! Projectiles, tick system 6 (see `combat::projectile`).

use crate::combat::{projectile, spells};
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    // Spell effects and channels for the frame the moves system just executed (WP-055).
    spells::run(world);
    projectile::run(world);
}
