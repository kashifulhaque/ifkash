//! The eleven tick systems, one module each, in schedule order (see `tick.rs`).

pub mod ai_think;
pub mod apply_commands;
pub mod deaths_drops;
pub mod hit_resolution;
pub mod locomotion;
pub mod move_state;
pub mod player_intent;
pub mod projectiles;
pub mod snapshot;
pub mod status_regen;
pub mod world_logic;
