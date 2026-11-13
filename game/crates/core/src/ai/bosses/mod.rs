//! The bosses' own mechanics: what the shared framework in [`super::boss`] doesn't do. Each
//! boss with a mechanic has one module, `bNN`, that reads its numbers from its definition's
//! `params` map (see `lf_content::BossDef`, in `game/data/bosses/bNN_*.ron`) and does nothing
//! for any other boss. There are two hooks; a boss module uses whichever fits:
//!
//! - [`run`] runs once a tick for the whole world, from [`super::run`] just before the
//!   framework's own `boss::run`. Use it for state that lives outside one fight's stage.
//! - [`tick`] runs once a tick per boss while that boss's fight is in the Fight stage, from
//!   `boss.rs` just before the framework's fight step.
//!
//! - [`b04`]: the Mossgut Brood: egg sacs, weak point, and Crawler spawns (WP-072).
//! - [`b06`]: the Drowned Choir: the verse, the Cantor's mend, and the Lamenter's teleport
//!   (WP-074; [`tick`]).
//! - [`b08`]: the Forgeheart Colossus's legs, kneel, and lava (WP-076; [`run`]).

pub mod b04;
#[cfg(test)]
mod b04_tests;
pub mod b06;
#[cfg(test)]
mod b06_tests;
pub mod b08;
#[cfg(test)]
mod b08_tests;

use crate::world::World;

/// Runs every whole-world boss mechanic for this tick.
pub fn run(world: &mut World) {
    b08::run(world);
    b04::run(world);
}

/// Runs the in-fight mechanic of boss `b` (an index into `Layout::bosses`), if it has one.
pub fn tick(world: &mut World, b: usize) {
    b06::tick(world, b);
    b04::tick(world, b);
}
