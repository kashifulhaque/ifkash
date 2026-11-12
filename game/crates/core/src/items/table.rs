//! Item definitions from `consumables.ron`, with every ID resolved to its registry number.
//! Built from content, so it's not hashed: the content hash covers it.

use super::gear::GearTable;
use lf_content::{Content, ItemEffect, ItemKind, NumId};
use serde::{Deserialize, Serialize};

/// One item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemDef {
    pub num: NumId,
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
    /// The most the inventory holds; 0 for the flasks and the on-the-spot pickups.
    pub stack: u16,
    /// What using it does (WP-057).
    pub effect: ItemEffect,
}

/// Flask numbers with the item and move references resolved. See
/// `lf_content::consumables::Flask` for what each field means.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlaskRules {
    pub vigil: NumId,
    pub lucid: NumId,
    pub drink_move: NumId,
    pub wick_bead: NumId,
    pub flask_dew: NumId,
    pub start_charges: u8,
    pub start_vigil: u8,
    pub max_charges: u8,
    pub max_level: u8,
    pub heal_base: u16,
    pub heal_per_level: u16,
    pub fp_base: u16,
    pub fp_per_level: u16,
    pub effect_frame: u16,
    pub walk_percent: u8,
    /// The items other than the flasks that `USE_ITEM` uses on the same drink move
    /// (throwables, oils, cures, Glimmer vials, Lantern Ash), sorted (WP-057).
    pub usable: Vec<NumId>,
}

impl FlaskRules {
    /// Resolves the flask section of the content. `None` if the content has no
    /// consumables file.
    pub fn from_content(content: &Content) -> Result<Option<Self>, String> {
        let Some(c) = &content.consumables else {
            return Ok(None);
        };
        let num = |id: &str| {
            content
                .registry
                .num(id)
                .ok_or_else(|| format!("\"{id}\" is missing from registry.ron"))
        };
        let f = &c.flask;
        let mut usable = Vec::new();
        for i in &c.items {
            if matches!(
                i.kind,
                ItemKind::Throwable
                    | ItemKind::WeaponBuff
                    | ItemKind::Cure
                    | ItemKind::Glimmer
                    | ItemKind::LanternAsh
                    | ItemKind::ArtScroll
            ) && i.effect != ItemEffect::None
            {
                usable.push(num(&i.id)?);
            }
        }
        usable.sort_unstable();
        Ok(Some(Self {
            vigil: num(&f.vigil)?,
            lucid: num(&f.lucid)?,
            drink_move: num(&f.drink_move)?,
            wick_bead: num(&f.wick_bead)?,
            flask_dew: num(&f.flask_dew)?,
            start_charges: f.start_charges,
            start_vigil: f.start_vigil,
            max_charges: f.max_charges,
            max_level: f.max_level,
            heal_base: f.heal_base,
            heal_per_level: f.heal_per_level,
            fp_base: f.fp_base,
            fp_per_level: f.fp_per_level,
            effect_frame: f.effect_frame,
            walk_percent: f.walk_percent,
            usable,
        }))
    }
}

/// Every item in the content, plus the weapons (for equip checks) and the gear.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ItemTable {
    /// Sorted by `num`.
    defs: Vec<ItemDef>,
    /// Weapon registry numbers, sorted.
    weapons: Vec<NumId>,
    /// Each weapon's weight in tenths of a unit, sorted by number (equip load).
    weights: Vec<(NumId, u16)>,
    /// The weapon the player starts with: the first in the content (file and list order).
    pub default_weapon: Option<NumId>,
    pub flask: Option<FlaskRules>,
    /// Charms and armor (WP-057).
    pub gear: GearTable,
}

impl ItemTable {
    pub fn from_content(content: &Content) -> Result<Self, String> {
        let num = |id: &str| {
            content
                .registry
                .num(id)
                .ok_or_else(|| format!("\"{id}\" is missing from registry.ron"))
        };
        let mut defs = Vec::new();
        if let Some(c) = &content.consumables {
            for i in &c.items {
                defs.push(ItemDef {
                    num: num(&i.id)?,
                    id: i.id.clone(),
                    name: i.name.clone(),
                    kind: i.kind,
                    stack: i.stack,
                    effect: i.effect.clone(),
                });
            }
        }
        defs.sort_by_key(|d| d.num);
        let mut weapons = Vec::new();
        let mut weights = Vec::new();
        for (_, w) in content.all_weapons() {
            let n = num(&w.id)?;
            weapons.push(n);
            weights.push((n, w.weight_tenths));
        }
        let default_weapon = weapons.first().copied();
        weapons.sort_unstable();
        weights.sort_unstable();
        Ok(Self {
            defs,
            weapons,
            weights,
            default_weapon,
            flask: FlaskRules::from_content(content)?,
            gear: GearTable::from_content(content)?,
        })
    }

    pub fn get(&self, num: NumId) -> Option<&ItemDef> {
        self.defs
            .binary_search_by_key(&num, |d| d.num)
            .ok()
            .map(|i| &self.defs[i])
    }

    /// Finds an item by string ID. A linear scan, for tools and tests.
    pub fn num_of(&self, id: &str) -> Option<NumId> {
        self.defs.iter().find(|d| d.id == id).map(|d| d.num)
    }

    /// A weapon's weight in tenths of a unit; 0 for anything else.
    pub fn weight_tenths(&self, num: NumId) -> u16 {
        self.weights
            .binary_search_by_key(&num, |w| w.0)
            .map_or(0, |i| self.weights[i].1)
    }

    pub fn is_weapon(&self, num: NumId) -> bool {
        self.weapons.binary_search(&num).is_ok()
    }
}
