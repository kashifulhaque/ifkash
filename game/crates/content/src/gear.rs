//! Charms and armor. Loaded from `charms.ron` and `armor.ron`.
//! See docs/game/04-arsenal-and-builds.md, "Charms" and "Armor".
//!
//! Both kinds carry a list of [`GearEffect`]s. The simulation adds up the effects of
//! everything the player has equipped (`lf_core::items::gear`); nothing here has any
//! behavior of its own. Percentages are whole percents, durations are ticks.

use serde::{Deserialize, Serialize};

use crate::moves::{Element, StatusKind};
use crate::scenario::DamageType;

/// The three resistances that gear can raise (docs/game/03-combat.md, "Status effects").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resistance {
    /// Poison and Blight.
    Immunity,
    /// Rupture and Rime.
    Robustness,
    /// Drowse and Delirium.
    Composure,
}

/// Which damage types an absorption entry covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbsorbTarget {
    /// All eight types.
    All,
    /// Neutral, Slash, Blunt, and Pierce.
    Physical,
    /// Aether, Fire, Storm, and Radiance.
    Elemental,
    Neutral,
    Slash,
    Blunt,
    Pierce,
    Aether,
    Fire,
    Storm,
    Radiance,
}

impl AbsorbTarget {
    /// The damage types this target covers.
    pub fn types(self) -> &'static [DamageType] {
        use DamageType as T;
        match self {
            Self::All => &[
                T::Neutral,
                T::Slash,
                T::Blunt,
                T::Pierce,
                T::Aether,
                T::Fire,
                T::Storm,
                T::Radiance,
            ],
            Self::Physical => &[T::Neutral, T::Slash, T::Blunt, T::Pierce],
            Self::Elemental => &[T::Aether, T::Fire, T::Storm, T::Radiance],
            Self::Neutral => &[T::Neutral],
            Self::Slash => &[T::Slash],
            Self::Blunt => &[T::Blunt],
            Self::Pierce => &[T::Pierce],
            Self::Aether => &[T::Aether],
            Self::Fire => &[T::Fire],
            Self::Storm => &[T::Storm],
            Self::Radiance => &[T::Radiance],
        }
    }
}

/// One effect of a charm or an armor piece.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GearEffect {
    /// Max HP, in percent more (8 is the spec's +8%).
    MaxHpPercent(u16),
    /// Max equip load, in percent more.
    EquipLoadPercent(u16),
    /// Stamina regeneration, in percent more.
    StaminaRegenPercent(u16),
    /// The parry window of the equipped weapon, in ticks more.
    ParryWindowTicks(u8),
    /// After a `status` proc within `radius_m` of the wearer (on anyone, the wearer
    /// included), the wearer holds the body buff `buff`.
    ProcBuff {
        status: StatusKind,
        buff: String,
        radius_m: f32,
    },
    /// Damage of one element, in percent more.
    ElementDamagePercent { element: Element, percent: u16 },
    /// Formula damage (elemental projectiles with no physical type), in percent more.
    FormulaDamagePercent(u16),
    /// Spell slots, more.
    SpellSlots(u8),
    /// Spell startup, in percent less.
    SpellStartupReducePercent(u8),
    /// The weight of equipped weapons, in percent less.
    WeaponWeightReducePercent(u8),
    /// The stamina a block costs, in percent less.
    GuardStaminaReducePercent(u8),
    /// Damage dealt, in percent more, while HP is below `below_percent` of max.
    LowHpDamage { below_percent: u8, percent: u16 },
    /// Restores `amount` FP every `every_ticks` ticks.
    FpRegen { every_ticks: u16, amount: u16 },
    /// A resistance, more. The same units as the attribute curves in `attributes.ron`.
    Resist { kind: Resistance, amount: u16 },
    /// Enemies notice the wearer at this percent of their sight range; with `dark_only`
    /// only in darkness.
    SightPercent { percent: u8, dark_only: bool },
}

/// A Charm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Charm {
    pub id: String,
    pub name: String,
    /// A relic: a charm that is an easter egg for a portfolio project.
    #[serde(default)]
    pub relic: bool,
    pub effects: Vec<GearEffect>,
}

/// Which armor slot a piece fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmorSlot {
    Head,
    Body,
}

/// Flat defense against physical and elemental damage.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefenseValues {
    pub physical: u32,
    pub elemental: u32,
}

/// One armor piece: a head or a body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmorPiece {
    pub id: String,
    pub name: String,
    /// The set it belongs to; empty for the unique relic pieces.
    pub set: String,
    pub slot: ArmorSlot,
    /// Equip load, in tenths of a unit, like a weapon's.
    pub weight_tenths: u16,
    pub defense: DefenseValues,
    /// Percent absorbed, per damage type. Entries add up, capped at 100.
    pub absorption: Vec<(AbsorbTarget, u8)>,
    /// The poise it adds.
    pub poise: u16,
    pub effects: Vec<GearEffect>,
}

/// The whole `armor.ron` file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmorFile {
    pub pieces: Vec<ArmorPiece>,
}

/// Problems in a list of gear effects, as `(field, message)`.
fn effect_problems(prefix: &str, effects: &[GearEffect], out: &mut Vec<(String, String)>) {
    for (i, e) in effects.iter().enumerate() {
        let mut bad = |m: &str| out.push((format!("{prefix}.effects[{i}]"), m.to_owned()));
        match e {
            GearEffect::ProcBuff { radius_m, buff, .. } => {
                if !radius_m.is_finite() || *radius_m <= 0.0 {
                    bad("radius_m must be positive");
                }
                if buff.is_empty() {
                    bad("needs a buff ID");
                }
            }
            GearEffect::FpRegen { every_ticks, .. } if *every_ticks == 0 => {
                bad("every_ticks must be at least 1");
            }
            GearEffect::SpellStartupReducePercent(p)
            | GearEffect::WeaponWeightReducePercent(p)
            | GearEffect::GuardStaminaReducePercent(p)
                if *p > 100 =>
            {
                bad("a reduction can't be more than 100 percent");
            }
            GearEffect::SightPercent { percent, .. } if *percent == 0 || *percent > 100 => {
                bad("percent must be 1 to 100");
            }
            _ => {}
        }
    }
}

/// Problems in the charms list, as `(field, message)`.
pub fn charm_problems(charms: &[Charm]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (i, c) in charms.iter().enumerate() {
        if !seen.insert(c.id.as_str()) {
            out.push((format!("[{i}].id"), "duplicate charm ID".to_owned()));
        }
        if c.effects.is_empty() {
            out.push((
                format!("[{i}].effects"),
                "a charm needs an effect".to_owned(),
            ));
        }
        effect_problems(&format!("[{i}]"), &c.effects, &mut out);
    }
    out
}

/// Problems in the armor file, as `(field, message)`.
pub fn armor_problems(file: &ArmorFile) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (i, p) in file.pieces.iter().enumerate() {
        if !seen.insert(p.id.as_str()) {
            out.push((format!("pieces[{i}].id"), "duplicate armor ID".to_owned()));
        }
        if p.absorption.iter().any(|&(_, pct)| pct > 100) {
            out.push((
                format!("pieces[{i}].absorption"),
                "more than 100 percent".to_owned(),
            ));
        }
        effect_problems(&format!("pieces[{i}]"), &p.effects, &mut out);
    }
    out
}

/// Every buff ID that a list of gear effects names.
pub fn effect_buffs(effects: &[GearEffect]) -> impl Iterator<Item = &str> {
    effects.iter().filter_map(|e| match e {
        GearEffect::ProcBuff { buff, .. } => Some(buff.as_str()),
        _ => None,
    })
}
