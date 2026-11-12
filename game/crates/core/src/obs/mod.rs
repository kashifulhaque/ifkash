//! Observation encoders and the `MultiDiscrete` action decoder for RL hosts (WP-039).
//! See docs/game/09-rl-environment.md, "Observations" and "Actions".
//!
//! Everything here is a pure read of the world: encoding an observation never changes
//! simulation state, so the browser can compute the same vectors for a policy running on
//! the site. The encoders follow the determinism rules (no `std` transcendental calls, no
//! hash-map iteration), so the same world gives bit-identical vectors on every platform.
//!
//! This first cut has the `proprio`, `target`, and `move` keys and the default five-head
//! action space. The feature layout of each vector is listed on its encoder. Slots the
//! spec names but no system implements yet (equip load, buffs other than the two buff
//! slots) are zero, and the unused tail of each vector is zero padding, so later WPs can
//! fill them without changing the shapes.
//!
//! The scales that map world units into roughly `[-1, 1]` are observation scaling, not
//! gameplay numbers: see [`POS_SCALE_M`] and friends.

pub mod action;
mod encode;

pub use action::{ActionDecoder, HEADS, action_mask};
pub use encode::{boss_body, move_ids, proprio, target, target_id};

/// Length of the `proprio` vector (docs/game/09-rl-environment.md, "Observations").
pub const PROPRIO_LEN: usize = 48;
/// Length of the `target` vector (docs/game/09-rl-environment.md, "Observations").
pub const TARGET_LEN: usize = 32;

/// Positions are divided by this many meters (about the width of a boss arena), so a
/// target across the arena reads about 1. Observation scaling only.
pub const POS_SCALE_M: f32 = 20.0;
/// Velocities are divided by this many m/s (about the player's sprint speed).
/// Observation scaling only.
pub const VEL_SCALE: f32 = 6.0;
/// Frame counts until a hit lands are divided by this many ticks (one second at 30 Hz)
/// and clipped to 1. Observation scaling only.
pub const HIT_SCALE_TICKS: f32 = 30.0;
/// Frame counts until a move ends are divided by this many ticks (two seconds) and
/// clipped to 1. Observation scaling only.
pub const LEFT_SCALE_TICKS: f32 = 60.0;
/// Status and buff timers are divided by this many ticks (one minute) and clipped to 1.
/// Observation scaling only.
pub const TIMER_SCALE_TICKS: f32 = 1800.0;

#[cfg(test)]
mod tests;
