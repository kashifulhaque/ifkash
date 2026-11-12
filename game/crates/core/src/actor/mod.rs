//! Actors: the player now; enemies, bosses, and Echoes later share the same struct.
//! See docs/game/01-architecture.md, "World and actors".

pub mod controller;
pub mod stats;
pub mod targeting;
#[cfg(test)]
mod tests;

use crate::arena::Id;
use crate::combat::hurtbox::Hurtbox;
use crate::combat::moves::MoveState;
use crate::combat::resolve::HitState;
use crate::items::Inventory;
use serde::{Deserialize, Serialize};

/// One actor. Every field is simulation state and is part of the state hash.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Actor {
    /// Feet position (the bottom of the capsule), in meters. Y is up.
    pub pos: [f32; 3],
    /// Facing, in radians from +Z toward +X.
    pub yaw: f32,
    /// X and Z: the planar velocity requested this tick, in m/s. Y: vertical velocity.
    pub vel: [f32; 3],
    pub is_player: bool,
    pub hp: u32,
    pub max_hp: u32,
    pub dead: bool,
    pub stamina: f32,
    pub max_stamina: f32,
    pub fp: f32,
    pub max_fp: f32,
    pub motor: Motor,
    /// The actor this one is locked on to (the player's lock-on target), if any.
    /// See `targeting`.
    pub lock_target: Option<Id>,
    /// The move being performed and the input buffer. See `combat::moves`.
    pub moves: MoveState,
    /// The capsule this actor is hit on. See `combat::hurtbox`.
    pub hurtbox: Hurtbox,
    /// The sweep start and the targets the current move has hit. See `combat::resolve`.
    pub hit: HitState,
    /// Equipment, items, and flasks. Only the player has any (see `items`).
    pub inventory: Inventory,
    /// Attributes, defenses, and the other inputs to damage. See `stats`.
    pub stats: stats::Stats,
}

/// Character controller state between ticks.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Motor {
    /// Standing on walkable ground after the last move.
    pub grounded: bool,
    /// Sprinting this tick.
    pub sprinting: bool,
    /// Consecutive ticks `DODGE` has been held.
    pub dodge_held: u16,
    /// Buttons of the previous tick, for press edges.
    pub prev_buttons: u16,
    /// Ticks left before a started jump leaves the ground; 0 when no jump is starting.
    pub jump_startup: u8,
    /// Ticks of landing recovery left; movement is locked while it's above 0.
    pub landing: u8,
    /// Airborne because of a jump (so landing recovery applies).
    pub jumped: bool,
    /// Height of the ground surface under the actor's center, the last time it stood
    /// with ground right under its center.
    pub ground_y: f32,
    /// Highest point of the current fall, as a ground-surface height (feet height minus
    /// the controller offset while airborne).
    pub fall_peak_y: f32,
    /// Height of the last completed fall, in meters, measured on landing.
    pub last_fall_m: f32,
    /// Ticks since stamina was last spent.
    pub stamina_idle: u16,
}
