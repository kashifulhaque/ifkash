//! Status effects and buffs. Loaded from `status.ron`.
//! See docs/game/03-combat.md, "Status effects" and "Buffs".
//!
//! Percent and permille values are integers so the simulation's status math is exact
//! (see `lf_core::combat::status`).

use serde::{Deserialize, Serialize};

use crate::moves::{Element, StatusBuildup};

/// A burst of damage: a share of max HP plus a flat amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Burst {
    /// Permille of max HP: 12 percent is 120.
    pub max_hp_permille: u16,
    pub flat: u32,
}

impl Burst {
    /// The damage to a target with `max_hp`.
    pub fn damage(&self, max_hp: u32) -> u32 {
        // u64 so a large max HP can't overflow the product.
        (u64::from(max_hp) * u64::from(self.max_hp_permille) / 1000) as u32 + self.flat
    }
}

/// Rime: a burst, then a debuff for a while.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RimeRules {
    pub burst: Burst,
    pub duration_ticks: u16,
    /// Damage taken while chilled, in percent: 115 is the spec's +15%.
    pub damage_taken_percent: u16,
    /// Stamina regeneration while chilled, in percent of normal: 50 is the spec's -50%.
    pub stamina_regen_percent: u16,
}

/// Poison and Blight: damage once per `dot_interval_ticks` for a while. The spec gives
/// the damage per second; `StatusRules::dot_interval_ticks` is one second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DotRules {
    /// Each application is this much, as a [`Burst`] (so 0.2 percent is 2 permille).
    pub per_second: Burst,
    pub duration_ticks: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DrowseRules {
    /// Enemies sleep this long, or until hit.
    pub sleep_ticks: u16,
    /// A hit on a sleeper deals this percent: 120 is the spec's 1.2x.
    pub asleep_hit_percent: u16,
    /// The player is knocked down (locked) this long.
    pub knockdown_ticks: u8,
    /// The player loses this percent of max FP.
    pub fp_loss_percent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliriumRules {
    /// The player loses this percent of max FP and of max HP.
    pub fp_loss_percent: u8,
    pub hp_loss_percent: u8,
    pub player_stagger_ticks: u8,
    /// Enemies that can go mad stagger this long.
    pub enemy_stagger_ticks: u8,
}

/// Which buff slot a buff fills (docs/game/03-combat.md, "Buffs").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffSlot {
    Weapon,
    Body,
}

/// A buff: a multiplier, a duration, and a VFX tag. A weapon buff can also add status
/// buildup to every hit its holder lands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuffDef {
    pub id: String,
    pub slot: BuffSlot,
    /// Damage dealt, in percent: 100 changes nothing, 115 is +15%. The slots' product is
    /// capped by `damage.buff_cap_percent` in `attributes.ron`.
    pub damage_percent: u16,
    /// Buildup added to every hit (weapon buffs only).
    pub status: Vec<StatusBuildup>,
    pub duration_ticks: u32,
    /// The renderer's effect tag.
    pub vfx: String,
    /// A weapon buff can add an element to every hit: the element and the percent of the
    /// hit's physical attack rating it adds (an oil's "Fire at 20% of AR", WP-057).
    #[serde(default)]
    pub element: Option<(Element, u8)>,
}

/// The whole `status.ron` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusRules {
    /// After a proc, the status can't build up again for this long.
    pub lockout_ticks: u16,
    /// A meter starts to decay this many ticks after its last buildup.
    pub decay_delay_ticks: u16,
    /// Decay per tick, in thousandths of a buildup point: 1500 is the spec's 1.5.
    pub decay_milli_per_tick: u32,
    /// A boss's resistance grows by this percent per proc this fight (25).
    pub boss_growth_percent: u16,
    /// Extra buildup from a riposte, in percent (the formula's `crit_or_stealth` 0.3).
    pub crit_bonus_percent: u16,
    /// Extra buildup from a backstab, in percent (20, "Parry and critical hits").
    pub backstab_bonus_percent: u16,
    /// Buildup a blocked hit adds, in percent of the full amount. The spec doesn't say; 0.
    pub guarded_percent: u16,
    /// Ticks in a second, for Poison and Blight.
    pub dot_interval_ticks: u16,
    pub rupture: Burst,
    pub rime: RimeRules,
    pub poison: DotRules,
    pub blight: DotRules,
    pub drowse: DrowseRules,
    pub delirium: DeliriumRules,
    pub buffs: Vec<BuffDef>,
}

impl StatusRules {
    /// Every buff ID, with the field path that names it.
    pub fn references(&self) -> Vec<(String, &str)> {
        self.buffs
            .iter()
            .enumerate()
            .map(|(i, b)| (format!("buffs[{i}].id"), b.id.as_str()))
            .collect()
    }

    /// Problems in the file, as `(field, message)`.
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let mut bad = |field: &str, message: &str| out.push((field.to_owned(), message.to_owned()));
        if self.lockout_ticks == 0 {
            bad("lockout_ticks", "must be at least 1");
        }
        if self.dot_interval_ticks == 0 {
            bad("dot_interval_ticks", "must be at least 1");
        }
        if self.poison.duration_ticks == 0 || self.blight.duration_ticks == 0 {
            bad("poison/blight.duration_ticks", "must be at least 1");
        }
        let mut seen = std::collections::BTreeSet::new();
        for (i, b) in self.buffs.iter().enumerate() {
            if !seen.insert(b.id.as_str()) {
                out.push((format!("buffs[{i}].id"), "duplicate buff ID".to_owned()));
            }
            if b.duration_ticks == 0 {
                out.push((
                    format!("buffs[{i}].duration_ticks"),
                    "must be at least 1".to_owned(),
                ));
            }
            if b.damage_percent == 0 {
                out.push((
                    format!("buffs[{i}].damage_percent"),
                    "must be at least 1".to_owned(),
                ));
            }
            if b.vfx.is_empty() {
                out.push((format!("buffs[{i}].vfx"), "needs a VFX tag".to_owned()));
            }
            if b.slot == BuffSlot::Body && b.element.is_some() {
                out.push((
                    format!("buffs[{i}].element"),
                    "only a weapon buff adds an element".to_owned(),
                ));
            }
            if b.slot == BuffSlot::Body && !b.status.is_empty() {
                out.push((
                    format!("buffs[{i}].status"),
                    "only a weapon buff adds buildup".to_owned(),
                ));
            }
        }
        out
    }
}
