//! Weapon definitions. A minimal slice: enough to hold a moveset and attack rating
//! inputs (docs/game/03-combat.md, "Attack rating"). Later weapon WPs extend it.

use serde::{Deserialize, Serialize};

use crate::moves::PhysType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum WeaponClass {
    Dagger,
    StraightSword,
    Greatsword,
    CurvedSword,
    ThrustingSword,
    Axe,
    Hammer,
    Colossal,
    Spear,
    Halberd,
    Fist,
    Bow,
    Shield,
    Catalyst,
}

/// Scaling coefficients per attribute (`scaling_a` in the attack-rating formula).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Scaling {
    pub might: f32,
    pub finesse: f32,
    pub insight: f32,
    pub devotion: f32,
}

/// Attribute requirements. Zero means none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Requirements {
    pub might: u16,
    pub finesse: u16,
    pub insight: u16,
    pub devotion: u16,
}

/// The move each input maps to for a weapon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Moveset {
    pub light_chain: Vec<String>,
    pub heavy: String,
    pub charged_heavy: String,
    pub running: String,
    pub rolling: String,
    pub backstep: String,
    pub jump_light: String,
    pub jump_heavy: String,
    pub plunge: String,
    pub guard_counter: String,
}

impl Moveset {
    /// Every referenced move with the field path that names it, for validation.
    pub fn references(&self) -> Vec<(String, &str)> {
        let mut out: Vec<(String, &str)> = self
            .light_chain
            .iter()
            .enumerate()
            .map(|(i, id)| (format!("moveset.light_chain[{i}]"), id.as_str()))
            .collect();
        for (field, id) in [
            ("heavy", &self.heavy),
            ("charged_heavy", &self.charged_heavy),
            ("running", &self.running),
            ("rolling", &self.rolling),
            ("backstep", &self.backstep),
            ("jump_light", &self.jump_light),
            ("jump_heavy", &self.jump_heavy),
            ("plunge", &self.plunge),
            ("guard_counter", &self.guard_counter),
        ] {
            out.push((format!("moveset.{field}"), id.as_str()));
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub class: WeaponClass,
    pub base_damage: u16,
    pub phys: PhysType,
    pub scaling: Scaling,
    pub requirements: Requirements,
    /// Weight in tenths of a unit (4.0 is `40`), to avoid float drift.
    pub weight_tenths: u16,
    #[serde(default)]
    pub unique: bool,
    pub moveset: Moveset,
    /// A placeholder for an item a later WP implements; the validator warns on it.
    #[serde(default)]
    pub stub: bool,
}
