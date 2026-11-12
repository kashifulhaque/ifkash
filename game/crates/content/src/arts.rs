//! Arts: the weapon skills that cost FP (docs/game/04-arsenal-and-builds.md, "Arts").
//! A weapon names its Art by ID (`Weapon::default_art`), and the validator checks that the
//! Art exists and lists the weapon's class. Each Art plays a move from `moves/arts.ron`; its
//! effects (a buff, a guard stance, a parry window) are data on the Art, so the move system
//! reads them without a per-Art branch.

use serde::{Deserialize, Serialize};

use crate::moves::Element;
use crate::weapons::WeaponClass;

/// One Art (`arts.ron`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Art {
    pub id: String,
    pub name: String,
    /// The weapon classes that can use the Art. A weapon may name only an Art that lists
    /// its class.
    pub classes: Vec<WeaponClass>,
    /// The FP the Art costs. Must equal the FP of its move (the validator checks).
    pub fp: u16,
    /// The move the Art plays, from `moves/arts.ron`.
    pub move_id: String,
    /// Moves that replace `move_id` for a weapon whose elemental part is the element. Toll
    /// is one Art: the Bell Toll is the base move, and a Radiance weapon (the Choir Glaive)
    /// plays the Choir Toll.
    #[serde(default)]
    pub variants: Vec<ArtVariant>,
    /// A buff the move applies on one of its frames.
    #[serde(default)]
    pub buff: Option<ArtBuff>,
    /// The guard stays up while the move plays (a stance).
    #[serde(default)]
    pub stance: bool,
    /// While a stance holds, a block starts the guard counter at once, with no `HEAVY`.
    #[serde(default)]
    pub auto_counter: bool,
    /// The parry window in ticks while the stance holds, which replaces the weapon's own
    /// window. 0 for none.
    #[serde(default)]
    pub parry_ticks: u8,
    /// A placeholder for an Art a later WP implements; the validator warns on it.
    #[serde(default)]
    pub stub: bool,
}

/// A move an Art plays for a weapon with this elemental part.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ArtVariant {
    pub element: Element,
    pub move_id: String,
}

/// A buff an Art's move applies on move frame `frame`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ArtBuff {
    pub frame: u16,
    /// A buff from `status.ron`.
    pub buff: String,
}

/// The whole `arts.ron` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtFile {
    pub arts: Vec<Art>,
}
