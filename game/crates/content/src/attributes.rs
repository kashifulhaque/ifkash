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
    pub attributes: Vec<AttributeDef>,
    pub max_hp: Curve,
    pub max_fp: Curve,
    pub max_stamina: Curve,
    pub max_equip_load: Curve,
    pub spell_slots: Steps,
    pub resistance: Curve,
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
