//! The bosses' own mechanics: what the shared framework in [`super::boss`] doesn't do. Each
//! boss has a module, `bNN`, with a per-tick `run` that reads its numbers from its definition's
//! `params` (see `lf_content::BossDef`) and does nothing for any other boss. [`run`] is the one
//! hook, called from [`super::run`] just before the framework's own `boss::run`.
//!
//! - [`b08`]: the Forgeheart Colossus's legs, kneel, and lava (WP-076).

pub mod b08;
#[cfg(test)]
mod b08_tests;

use crate::world::World;

/// Runs every boss mechanic for this tick.
pub fn run(world: &mut World) {
    b08::run(world);
}
