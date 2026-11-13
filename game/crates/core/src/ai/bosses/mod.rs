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
//! Two more entry points exist for the bosses that need them: [`after`] runs once a tick for
//! the whole world, from [`super::run`] right after the framework's own `boss::run`, for a
//! mechanic that reads the framework's step of this tick; [`before_locomotion`] runs in tick
//! system 5, before the controller moves the player, for a boss that slows or moves the
//! player.
//!
//! - [`b02`]: the Briar Stag: the tree stun and the bramble patches (WP-070; [`run`], and
//!   [`before_locomotion`]).
//! - [`b04`]: the Mossgut Brood: egg sacs, weak point, and Crawler spawns (WP-072).
//! - [`b05`]: the Ossuary Keeper: raised thralls, the tide, and the borrowed weapon (WP-073;
//!   [`after`]).
//! - [`b06`]: the Drowned Choir: the verse, the Cantor's mend, and the Lamenter's teleport
//!   (WP-074; [`tick`]).
//! - [`b08`]: the Forgeheart Colossus's legs, kneel, and lava (WP-076; [`run`]).
//! - [`b07`]: Castellan Voss: the counts, the counters, the guard, the chandeliers, and the back
//!   gate (WP-075; [`run`]).

pub mod b02;
#[cfg(test)]
mod b02_tests;
pub mod b04;
#[cfg(test)]
mod b04_tests;
pub mod b05;
#[cfg(test)]
mod b05_tests;
pub mod b06;
#[cfg(test)]
mod b06_tests;
pub mod b07;
#[cfg(test)]
mod b07_tests;
pub mod b08;
#[cfg(test)]
mod b08_tests;

use crate::world::World;

/// Runs every whole-world boss mechanic for this tick.
pub fn run(world: &mut World) {
    b08::run(world);
    b04::run(world);
    b02::run(world);
    b07::run(world);
}

/// Runs the in-fight mechanic of boss `b` (an index into `Layout::bosses`), if it has one.
pub fn tick(world: &mut World, b: usize) {
    b06::tick(world, b);
    b04::tick(world, b);
}

/// The bosses' effects on the player's movement: tick system 5, before the controller moves
/// the player.
pub fn before_locomotion(world: &mut World) {
    b02::before_locomotion(world);
}

/// Runs every whole-world boss mechanic that follows the framework's step, for this tick.
pub fn after(world: &mut World) {
    b05::run(world);
}
