//! Spells: the Formulae and Litanies (docs/game/03-combat.md, "Spells";
//! docs/game/04-arsenal-and-builds.md, "Spells"). Loaded from `spells/formulae.ron` and
//! `spells/litanies.ron`.
//!
//! A spell file is self-contained. Each spell carries its own cast moves, projectiles, and
//! buffs, written in the same schemas as `moves/`, `projectiles.ron`, and the buffs of
//! `status.ron`. [`crate::Content::with_spells`] merges them into those tables, so the move
//! system, projectile system, and buff system treat them like any other content, and a new
//! spell needs no Rust change. The spell, each move, each projectile, and each buff need an
//! entry in `registry.ron`.
//!
//! What a spell does besides launching projectiles and hitboxes (the move's own lists) is
//! in `effects`: frame-timed heals, buffs, and cures. A channeled spell (Pyre Breath) holds
//! its move at a frame and spawns a projectile every few ticks while the cast button is
//! held.

use serde::{Deserialize, Serialize};

use crate::moves::{Move, StatusKind};
use crate::projectiles::ProjectileDef;
use crate::status::BuffDef;
use crate::weapons::Requirements;

/// Which school a spell is in, and which catalyst casts it: a stylus casts Formulae, a
/// reliquary casts Litanies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SpellSchool {
    /// Scales with Insight; cast with a stylus.
    Formula,
    /// Scales with Devotion; cast with a reliquary.
    Litany,
}

/// Cast speed (docs/game/03-combat.md, "Spells": Finesse shortens cast startup by up to 25
/// percent at 60 Finesse).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CastSpeed {
    /// The most startup a cast loses, in percent.
    pub max_percent: u8,
    /// The Finesse that gives the maximum. Below it the saving rises in proportion.
    pub at_finesse: u16,
}

/// Rules shared by every spell. One spell file states them (`formulae.ron`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellRules {
    pub cast_speed: CastSpeed,
}

/// What a timed effect does.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpellEffect {
    /// Heals the caster by a share of max HP. `radius_m` above 0 also heals the Echoes
    /// (allies) within that many meters of the caster, by the same share of their max HP
    /// (Wide Mend, WP-056).
    Heal { max_hp_percent: u8, radius_m: f32 },
    /// Applies a buff defined in this spell's `buffs`, replacing the one in its slot.
    Buff { buff: String },
    /// Cures the listed statuses (meter and effect), or every status if the list is empty.
    Cure { statuses: Vec<StatusKind> },
}

/// An effect that fires on one frame of a cast move.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedEffect {
    /// The move frame it fires on. Not a frame a chargeable move holds at.
    pub frame: u16,
    pub effect: SpellEffect,
}

/// A channeled spell: the cast move must have a `charge`, which holds it at `charge_from`
/// while the cast button stays down. Every `interval_ticks` of the hold, the caster pays
/// `fp` and launches `projectile`. The channel ends when the button is released, the move's
/// `charge.max_frames` run out, or FP can't pay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    pub interval_ticks: u16,
    /// FP paid each interval (the spec's "per second" cost at 30 ticks a second).
    pub fp: u16,
    /// A projectile ID from this spell's `projectiles`.
    pub projectile: String,
    pub offset: [f32; 3],
    pub yaw_deg: f32,
    pub pitch_deg: f32,
}

/// One spell, as authored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellDef {
    pub id: String,
    pub name: String,
    pub school: SpellSchool,
    /// Attunement slots it takes: 1 or 2.
    pub slots: u8,
    /// Attributes needed to attune and cast it. Zero means none.
    pub requirements: Requirements,
    /// The move `LIGHT` starts. Its `fp` is the spell's FP cost.
    pub cast: Move,
    /// The move `HEAVY` starts (the charged form), if the spell has one.
    #[serde(default)]
    pub charged: Option<Move>,
    /// Effects of the cast move, and of the charged move.
    #[serde(default)]
    pub effects: Vec<TimedEffect>,
    #[serde(default)]
    pub charged_effects: Vec<TimedEffect>,
    #[serde(default)]
    pub channel: Option<Channel>,
    /// Projectiles the spell's moves launch.
    #[serde(default)]
    pub projectiles: Vec<ProjectileDef>,
    /// Buffs the spell's effects apply.
    #[serde(default)]
    pub buffs: Vec<BuffDef>,
}

/// The shape of a spell file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellFile {
    #[serde(default)]
    pub rules: Option<SpellRules>,
    pub spells: Vec<SpellDef>,
}

impl SpellDef {
    /// Every registry ID the spell defines, with the field that names it.
    pub fn references(&self) -> Vec<(String, &str)> {
        let mut out = vec![
            ("id".to_owned(), self.id.as_str()),
            ("cast.id".to_owned(), self.cast.id.as_str()),
        ];
        if let Some(m) = &self.charged {
            out.push(("charged.id".to_owned(), m.id.as_str()));
        }
        for (i, p) in self.projectiles.iter().enumerate() {
            out.push((format!("projectiles[{i}].id"), p.id.as_str()));
        }
        for (i, b) in self.buffs.iter().enumerate() {
            out.push((format!("buffs[{i}].id"), b.id.as_str()));
        }
        out
    }

    /// Problems in the spell, as `(field, message)`.
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        if !(1..=2).contains(&self.slots) {
            out.push(("slots".to_owned(), "must be 1 or 2".to_owned()));
        }
        let mut check = |field: &str, list: &[TimedEffect], total: u16| {
            for (i, e) in list.iter().enumerate() {
                if e.frame >= total {
                    out.push((
                        format!("{field}[{i}].frame"),
                        format!("{} is outside the move's {total} frames", e.frame),
                    ));
                }
                if let SpellEffect::Buff { buff } = &e.effect
                    && !self.buffs.iter().any(|b| &b.id == buff)
                {
                    out.push((
                        format!("{field}[{i}].effect"),
                        format!("buff \"{buff}\" is not in this spell's buffs"),
                    ));
                }
            }
        };
        check("effects", &self.effects, self.cast.total_frames);
        if let Some(c) = &self.charged {
            check("charged_effects", &self.charged_effects, c.total_frames);
        }
        if self.charged.is_none() && !self.charged_effects.is_empty() {
            out.push((
                "charged_effects".to_owned(),
                "the spell has no charged move".to_owned(),
            ));
        }
        if let Some(c) = &self.channel {
            if self.cast.charge.is_none() {
                out.push((
                    "channel".to_owned(),
                    "the cast move needs a `charge` to hold".to_owned(),
                ));
            }
            if c.interval_ticks == 0 {
                out.push((
                    "channel.interval_ticks".to_owned(),
                    "must be at least 1".to_owned(),
                ));
            }
            if !self.projectiles.iter().any(|p| p.id == c.projectile) {
                out.push((
                    "channel.projectile".to_owned(),
                    format!(
                        "projectile \"{}\" is not in this spell's projectiles",
                        c.projectile
                    ),
                ));
            }
        }
        out
    }
}
