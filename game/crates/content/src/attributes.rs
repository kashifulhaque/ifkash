//! Attributes and derived-stat curves (docs/game/03-combat.md, "Derived stats").
//!
//! Curve values are fixed-point tenths, so evaluation is exact integer math with no
//! float drift: 50.8 equip load is `508`. Use the `*_f32` helpers only at the display
//! boundary.

use serde::{Deserialize, Serialize};

/// One of the seven attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct AttributeDef {
    pub id: String,
    pub name: String,
}

/// A run of attribute points that all add the same increment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Segment {
    /// The last attribute value this rate applies to (inclusive).
    pub until: u16,
    pub per_point_tenths: i32,
}

/// A piecewise-linear curve: `base_tenths` at the minimum attribute, then each
/// segment's rate for the points up to its `until` breakpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Curve {
    pub base_tenths: i32,
    pub segments: Vec<Segment>,
}

impl Curve {
    /// Evaluates the curve at `attr`, where the curve starts at attribute `from`.
    pub fn eval_tenths(&self, from: u16, attr: u16) -> i32 {
        let mut value = self.base_tenths;
        let mut prev = from;
        for seg in &self.segments {
            if attr <= prev {
                break;
            }
            let top = attr.min(seg.until);
            if top > prev {
                value += i32::from(top - prev) * seg.per_point_tenths;
                prev = top;
            }
        }
        value
    }
}

/// A stepped stat: `base` from attribute `from`, plus one every `every` points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Steps {
    pub base: u16,
    pub from: u16,
    pub every: u16,
}

impl Steps {
    pub fn eval(&self, attr: u16) -> u16 {
        if attr < self.from || self.every == 0 {
            self.base
        } else {
            self.base + (attr - self.from) / self.every
        }
    }
}

/// The Glimmer cost of the level from `L` to `L + 1`:
/// `floor(base + per_level * L + quad_tenths * L^2 / 10)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct LevelCost {
    pub base: u32,
    pub per_level: u32,
    /// The quadratic term in tenths: 12 is 1.2 * L^2.
    pub quad_tenths: u32,
}

/// The contents of `attributes.ron`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Attributes {
    pub min: u16,
    pub max: u16,
    pub start_min: u16,
    pub start_max: u16,
    pub level_offset: u16,
    /// The Glimmer cost of each level (docs/game/06-progression-and-systems.md, "Leveling").
    pub level_cost: LevelCost,
    pub attributes: Vec<AttributeDef>,
    pub max_hp: Curve,
    pub max_fp: Curve,
    pub max_stamina: Curve,
    pub max_equip_load: Curve,
    pub spell_slots: Steps,
    pub resistance: Curve,
    /// Attack rating and damage numbers (docs/game/03-combat.md, "Attack rating" and
    /// "Damage dealt").
    pub damage: DamageRules,
}

/// A run of attribute points that add the same scaling-curve increment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ScalingSegment {
    /// The last attribute value this rate applies to (inclusive).
    pub until: u16,
    /// Millionths of the curve gained per attribute point.
    pub per_point_ppm: u32,
}

/// The damage formulas' constants. Percent and permille values are integers so the
/// simulation's damage math is exact (see `lf_core::combat::damage`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct DamageRules {
    /// The highest weapon upgrade level.
    pub max_upgrade: u8,
    /// `base(U) = base_damage * (1 + 0.06 * U)`, in permille per level.
    pub base_per_upgrade_permille: u16,
    /// `k_a(U) = scaling_a * (1 + 0.03 * U)`, in percent per level.
    pub scaling_per_upgrade_percent: u16,
    /// The `curve(x)` of the attack-rating formula, a piecewise-linear curve from 0 at
    /// attribute 0. The last segment's `until` is where it reaches 1.0.
    pub scaling_curve: Vec<ScalingSegment>,
    /// Base damage multiplier while a requirement isn't met (scaling is zero too).
    pub unwieldy_base_percent: u16,
    /// Extra recovery frames while a requirement isn't met, in percent. Consumed by the
    /// move system (not applied by the damage pipeline).
    pub unwieldy_recovery_percent: u16,
    /// Might counts as this percent of its value while two-handing, capped at `max`.
    pub two_hand_might_percent: u16,
    /// `counter_mult` when the target is in the startup or active frames of its attack.
    pub counter_percent: u16,
    /// The most the product of damage buffs can multiply raw damage, in percent.
    pub buff_cap_percent: u16,
    /// The most a target's armor absorbs of a damage type, in percent.
    pub absorption_cap_percent: u16,
}

impl DamageRules {
    /// `curve(x)` in millionths, for an attribute value given in half points (twice the
    /// value) so that 1.5x Might from two-handing stays exact. `attr_halves` is clamped
    /// by the caller.
    pub fn scaling_curve_ppm(&self, attr_halves: u32) -> u64 {
        let mut value = 0u64;
        let mut prev = 0u32;
        for seg in &self.scaling_curve {
            let top = attr_halves.min(u32::from(seg.until) * 2);
            if top > prev {
                // Every per-point rate is even in practice; half a point is half a rate.
                value += u64::from(top - prev) * u64::from(seg.per_point_ppm) / 2;
                prev = top;
            }
        }
        value
    }
}

impl Attributes {
    fn clamp(&self, attr: u16) -> u16 {
        attr.clamp(self.min, self.max)
    }

    /// Max HP in tenths, from Vitality.
    pub fn max_hp_tenths(&self, vitality: u16) -> i32 {
        self.max_hp.eval_tenths(self.min, self.clamp(vitality))
    }

    /// Max FP in tenths, from Focus.
    pub fn max_fp_tenths(&self, focus: u16) -> i32 {
        self.max_fp.eval_tenths(self.min, self.clamp(focus))
    }

    /// Max stamina in tenths, from Endurance.
    pub fn max_stamina_tenths(&self, endurance: u16) -> i32 {
        self.max_stamina
            .eval_tenths(self.min, self.clamp(endurance))
    }

    /// Max equip load in tenths, from Endurance.
    pub fn max_equip_load_tenths(&self, endurance: u16) -> i32 {
        self.max_equip_load
            .eval_tenths(self.min, self.clamp(endurance))
    }

    /// Spell slots, from Focus.
    pub fn spell_slots(&self, focus: u16) -> u16 {
        self.spell_slots.eval(self.clamp(focus))
    }

    /// A resistance stat in tenths, from the attribute that raises it.
    pub fn resistance_tenths(&self, attr: u16) -> i32 {
        self.resistance.eval_tenths(self.min, self.clamp(attr))
    }

    /// Converts a tenths value to `f32` for display. Not for simulation math.
    pub fn tenths_to_f32(tenths: i32) -> f32 {
        tenths as f32 / 10.0
    }
}
