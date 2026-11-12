//! Special rules: behavior that one enemy needs and the shared AI loop doesn't have. Each
//! rule is selected by a field of the enemy's definition. See `ai/mod.rs`, "Extension
//! points".
//!
//! - [`revive`]: the Whitewarden Skeleton gets back up after dying.

pub mod revive;

// WP-036 (enemy batch 2). `batch2::tick` is the per-tick hook; the rest are the rules.
pub mod batch2;
pub mod flying;
pub mod reeds;
pub mod reflect;
pub mod shimmer;
pub mod swarm;
pub mod water;

pub use revive::{note_death_blow, tick_dead};
