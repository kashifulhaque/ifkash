//! The move data schema (docs/game/03-combat.md, "Move data schema").

use serde::{Deserialize, Serialize};

/// An inclusive frame window `from..=to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct FrameWindow {
    pub from: u16,
    pub to: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum PhysType {
    Neutral,
    Slash,
    Blunt,
    Pierce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Element {
    Aether,
    Fire,
    Storm,
    Radiance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum StatusKind {
    Poison,
    Blight,
    Drowse,
    Delirium,
    Rupture,
    Rime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum HitTag {
    Unparryable,
    Unblockable,
    Grab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct StatusBuildup {
    pub kind: StatusKind,
    pub amount: u16,
}

/// A hit volume in the actor's local space: x right, y up, z forward, origin at the feet.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Shape {
    Capsule {
        a: [f32; 3],
        b: [f32; 3],
        radius: f32,
    },
    Sphere {
        center: [f32; 3],
        radius: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Hitbox {
    /// Active frames, inclusive.
    pub frames: (u16, u16),
    pub shape: Shape,
    pub motion_value: f32,
    pub phys: PhysType,
    pub element: Option<Element>,
    pub poise_damage: u16,
    pub status: Vec<StatusBuildup>,
    pub knockback: f32,
    /// Render-only freeze frames.
    pub hitstop: u8,
    pub tags: Vec<HitTag>,
}

/// A chargeable move. The move holds at `charge_from` for up to `max_frames` ticks and
/// the hitbox motion value rises from its listed value to `motion_value_max`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Charge {
    pub charge_from: u16,
    pub max_frames: u16,
    pub motion_value_max: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Move {
    pub id: String,
    /// Renderer clip name; visual only.
    pub anim: String,
    pub total_frames: u16,
    pub stamina: u16,
    pub fp: u16,
    /// `(frame, meters forward)` keys.
    pub root_motion: Vec<(u16, f32)>,
    /// Turn toward the lock target, startup only.
    pub track_deg_per_tick: f32,
    pub hyperarmor: Option<FrameWindow>,
    pub iframes: Option<FrameWindow>,
    pub cancel_attack_from: Option<u16>,
    pub cancel_dodge_from: Option<u16>,
    pub cancel_any_from: Option<u16>,
    pub next_chain: Option<String>,
    #[serde(default)]
    pub charge: Option<Charge>,
    pub hitboxes: Vec<Hitbox>,
    pub sfx: Vec<(u16, String)>,
    pub vfx: Vec<(u16, String)>,
    /// A placeholder for a move a later WP implements; the validator warns on it.
    #[serde(default)]
    pub stub: bool,
    /// Multiplies the hitboxes' `poise_damage` into stance damage (docs/game/03-combat.md,
    /// "Poise and stance"): 2.0 for guard counters, charged heavies, and jump heavies.
    #[serde(default = "default_stance_mult")]
    pub stance_mult: f32,
}

fn default_stance_mult() -> f32 {
    1.0
}
