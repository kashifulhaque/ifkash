//! Weapon definitions. A minimal slice: enough to hold a moveset and attack rating
//! inputs (docs/game/03-combat.md, "Attack rating"). Later weapon WPs extend it.

use serde::{Deserialize, Serialize};

use crate::moves::PhysType;
use crate::spells::SpellSchool;

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
    /// Roll at Light, Medium, and Heavy equip load: `DODGE` tapped with a direction
    /// (docs/game/03-combat.md, "Combat verbs" and "Equip load and roll tiers").
    pub roll_light: String,
    pub roll_medium: String,
    pub roll_heavy: String,
    /// Backstep: `DODGE` tapped with no direction.
    pub dodge_backstep: String,
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
            ("roll_light", &self.roll_light),
            ("roll_medium", &self.roll_medium),
            ("roll_heavy", &self.roll_heavy),
            ("dodge_backstep", &self.dodge_backstep),
        ] {
            out.push((format!("moveset.{field}"), id.as_str()));
        }
        out
    }
}

/// How a weapon blocks (docs/game/03-combat.md, "Guard and guard counter").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct GuardStats {
    /// Percent of physical damage a block absorbs.
    pub absorb_phys_percent: u8,
    /// Percent of elemental damage a block absorbs.
    pub absorb_elem_percent: u8,
    /// Divides the stamina a block costs: `poise_damage * 100 / guard_boost`.
    pub guard_boost: u16,
    /// Ticks of the parry window once the parry startup has passed (docs/game/03-combat.md,
    /// "Parry and critical hits"): 6 for bucklers and parry daggers, 4 for medium shields.
    /// 0 means the weapon can't parry.
    #[serde(default)]
    pub parry_window_ticks: u8,
}

fn default_crit_mult_percent() -> u16 {
    100
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
    pub guard: GuardStats,
    /// Critical hit damage multiplier in percent: 140 for daggers, 100 for most weapons
    /// (docs/game/03-combat.md, "Parry and critical hits").
    #[serde(default = "default_crit_mult_percent")]
    pub crit_mult_percent: u16,
    pub moveset: Moveset,
    /// A placeholder for an item a later WP implements; the validator warns on it.
    #[serde(default)]
    pub stub: bool,
    /// The school a catalyst casts: a stylus casts Formulae, a reliquary Litanies. Set on
    /// every `Catalyst` weapon and on no other.
    #[serde(default)]
    pub catalyst_school: Option<SpellSchool>,
}
