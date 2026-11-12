//! Projectile definitions (docs/game/01-architecture.md, "Physics": projectiles are shape
//! casts, not rigid bodies; docs/game/03-combat.md, "Spells"). A move spawns one through
//! its `projectiles` list (see [`crate::moves::ProjectileSpawn`]).

use serde::{Deserialize, Serialize};

use crate::moves::{Element, HitTag, PhysType, StatusBuildup};

/// A projectile that steers toward a target while it flies.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Homing {
    /// The most the heading turns in one tick, in degrees. The projectile keeps its
    /// speed and turns its velocity toward the target by at most this much.
    pub turn_deg_per_tick: f32,
    /// How far from the projectile a target can be and still be steered toward, in
    /// meters. The shooter's lock target is followed at any range.
    pub range_m: f32,
}

/// One projectile type. Its shape is a sphere, swept along each tick's motion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectileDef {
    pub id: String,
    /// Launch speed in meters per second. A homing projectile keeps it; a falling one
    /// speeds up and slows down under `gravity`.
    pub speed: f32,
    /// Downward acceleration in meters per second squared. 0 flies straight.
    pub gravity: f32,
    /// Radius of the swept sphere, in meters.
    pub radius: f32,
    /// Ticks the projectile flies before it expires.
    pub lifetime_ticks: u16,
    /// Actors it passes through. 0 stops it at the first actor it hits; 1 lets it hit
    /// two before it stops.
    pub pierce: u8,
    pub homing: Option<Homing>,
    /// Multiplies the shooter's attack rating, like a hitbox's `motion_value`.
    pub motion_value: f32,
    pub phys: PhysType,
    pub element: Option<Element>,
    /// Poise damage on a hit; a guard turns it into a stamina cost.
    pub poise_damage: u16,
    pub knockback: f32,
    /// Render-only freeze frames.
    pub hitstop: u8,
    pub tags: Vec<HitTag>,
    /// Status buildup a hit adds (a guarded hit adds less, as for a melee hit).
    #[serde(default)]
    pub status: Option<StatusBuildup>,
    /// A projectile that bursts where it ends: everyone within this many meters of the
    /// end point takes the hit too. 0 means no burst (WP-057, the Pitch Bomb's 2 m).
    #[serde(default)]
    pub area_radius_m: f32,
}
