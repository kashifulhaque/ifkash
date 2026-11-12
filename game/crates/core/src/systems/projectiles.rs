//! Projectiles, tick system 6 (see `combat::projectile`).

use crate::combat::{arts, projectile, spells};
use crate::input::TickInput;
use crate::world::World;

pub fn run(world: &mut World, _input: &TickInput) {
    // Spell effects and channels for the frame the moves system just executed (WP-055).
    spells::run(world);
    // An Art's buff on its move frame (WP-054).
    arts::run(world);
    projectile::run(world);
}
