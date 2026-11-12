//! Special rules: behavior that one enemy needs and the shared AI loop doesn't have. Each
//! rule is selected by a field of the enemy's definition. See `ai/mod.rs`, "Extension
//! points".
//!
//! - [`revive`]: the Whitewarden Skeleton gets back up after dying.

pub mod revive;

pub use revive::{note_death_blow, tick_dead};
