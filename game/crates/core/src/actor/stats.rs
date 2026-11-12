//! Actor stats: attribute levels, the stats derived from them, and defenses. See
//! docs/game/03-combat.md, "Attributes" and "Derived stats".
//!
//! Attribute levels are a stub loadout until leveling (WP-050): the player's come from
//! the `stub_*` fields of `locomotion.ron`. The derived curves live in
//! `data/attributes.ron`, not in code.

use lf_content::scenario::{DamageType, DefenseData};
use lf_content::{Attributes, Locomotion};
use serde::{Deserialize, Serialize};

use crate::combat::damage::TYPE_COUNT;

/// The seven attribute levels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attrs {
    pub vitality: u16,
    pub focus: u16,
    pub endurance: u16,
    pub might: u16,
    pub finesse: u16,
    pub insight: u16,
    pub devotion: u16,
}

impl Attrs {
    /// The sum of the seven levels.
    pub fn total(&self) -> u32 {
        [
            self.vitality,
            self.focus,
            self.endurance,
            self.might,
            self.finesse,
            self.insight,
            self.devotion,
        ]
        .iter()
        .map(|&a| u32::from(a))
        .sum()
    }

    /// Character level: the sum of attributes minus the offset.
    pub fn level(&self, rules: &Attributes) -> u32 {
        self.total().saturating_sub(u32::from(rules.level_offset))
    }

    /// The stats the attributes drive, from the curves in `attributes.ron`.
    pub fn derived(&self, rules: &Attributes) -> Derived {
        Derived {
            max_hp: (rules.max_hp_tenths(self.vitality) / 10).max(1) as u32,
            max_fp: rules.max_fp_tenths(self.focus) as f32 / 10.0,
            max_stamina: rules.max_stamina_tenths(self.endurance) as f32 / 10.0,
            max_equip_load_tenths: rules.max_equip_load_tenths(self.endurance),
            spell_slots: rules.spell_slots(self.focus),
        }
    }
}

/// Stats derived from attributes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Derived {
    pub max_hp: u32,
    pub max_fp: f32,
    pub max_stamina: f32,
    /// In tenths of a unit, like weapon weights.
    pub max_equip_load_tenths: i32,
    pub spell_slots: u16,
}

/// Defenses against each damage type, indexed by `DamageType as usize`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Defense {
    /// Flat defense, in the `raw * raw / (raw + defense)` term.
    pub flat: [u32; TYPE_COUNT],
    /// Percent absorbed, before the cap in `attributes.ron`.
    pub absorb_percent: [u8; TYPE_COUNT],
    /// Damage multiplier in percent: 100 is neutral, 125 is the spec's 1.25.
    pub weakness_percent: [u16; TYPE_COUNT],
}

impl Default for Defense {
    fn default() -> Self {
        Self {
            flat: [0; TYPE_COUNT],
            absorb_percent: [0; TYPE_COUNT],
            weakness_percent: [100; TYPE_COUNT],
        }
    }
}

impl Defense {
    /// The same flat defense against every type, nothing else.
    pub fn flat(flat: u32) -> Self {
        Self {
            flat: [flat; TYPE_COUNT],
            ..Self::default()
        }
    }

    /// Builds the defenses a scenario file describes.
    pub fn from_data(d: &DefenseData) -> Self {
        let mut out = Self::flat(d.flat);
        for &(t, pct) in &d.absorption {
            out.absorb_percent[t as usize] = pct;
        }
        for &(t, pct) in &d.weakness {
            out.weakness_percent[t as usize] = pct;
        }
        out
    }

    pub fn flat_of(&self, t: DamageType) -> u32 {
        self.flat[t as usize]
    }
}

/// An actor's combat stats. Simulation state: part of the state hash.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    pub attrs: Attrs,
    /// Upgrade level of the equipped weapon, 0 to `damage.max_upgrade` (stub: no
    /// upgrades exist yet).
    pub weapon_upgrade: u8,
    /// Two-handing the right weapon (`TWO_HAND` toggles it once the player intent system
    /// wires that button).
    pub two_handing: bool,
    pub defense: Defense,
    /// The product of this actor's active damage buffs, in percent; 100 with none. The
    /// pipeline caps it at `damage.buff_cap_percent`.
    pub buff_percent: u16,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            attrs: Attrs::default(),
            weapon_upgrade: 0,
            two_handing: false,
            defense: Defense::default(),
            buff_percent: 100,
        }
    }
}

impl Stats {
    /// The player's stub loadout from `locomotion.ron`.
    pub fn player_stub(loco: &Locomotion) -> Self {
        Self {
            attrs: Attrs {
                vitality: loco.stub_vitality,
                focus: loco.stub_focus,
                endurance: loco.stub_endurance,
                might: loco.stub_might,
                finesse: loco.stub_finesse,
                insight: loco.stub_insight,
                devotion: loco.stub_devotion,
            },
            weapon_upgrade: loco.stub_weapon_upgrade,
            defense: Defense::flat(loco.stub_defense),
            ..Self::default()
        }
    }
}
