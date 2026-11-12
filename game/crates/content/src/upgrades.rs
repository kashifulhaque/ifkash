//! Weapon upgrades and tempering: the Smith's numbers (docs/game/04-arsenal-and-builds.md,
//! "Temper" and "Upgrades"). The rules live in `upgrades.ron`; the commands and the state
//! live in `core::items::upgrade`.

use serde::{Deserialize, Serialize};

use crate::moves::{Element, StatusBuildup};

/// One band of the Forgeglass costs: the levels up to and including `through` cost
/// `count` of `item`. Bands are listed in level order and cover every common level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ForgeCost {
    pub through: u8,
    pub item: String,
    pub count: u16,
}

/// A temper (docs/game/04-arsenal-and-builds.md, "Temper"). The scaling changes are
/// per attribute in Might, Finesse, Insight, Devotion order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Temper {
    pub id: String,
    pub name: String,
    /// Each scaling is multiplied by this percent (100 leaves it alone, 0 removes it).
    pub scaling_percent: [u16; 4],
    /// Then this many thousandths is added to each scaling.
    pub scaling_add_permille: [u16; 4],
    /// The element the temper converts part of the base damage into, if any.
    #[serde(default)]
    pub element: Option<Element>,
    /// The percent of the base damage that becomes `element`. 0 with no element.
    #[serde(default)]
    pub element_percent: u8,
    /// Status buildup the temper adds to each hit. The combat pipeline doesn't apply it yet.
    #[serde(default)]
    pub status: Vec<StatusBuildup>,
}

/// The whole `upgrades.ron` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct UpgradeRules {
    /// Glimmer per upgrade is `glimmer_base * level^1.5`, rounded down, where `level` is
    /// the effective common level being reached.
    pub glimmer_base: u32,
    /// Forgeglass bands for common levels 1 up to the maximum (`attributes.ron` damage
    /// rules).
    pub common_costs: Vec<ForgeCost>,
    /// The Starglass item: one per level of a unique or relic weapon.
    pub starglass_item: String,
    /// How many common levels one Starglass level counts as in the attack rating.
    pub starglass_common_levels: u8,
    /// The highest Starglass level.
    pub starglass_max: u8,
    /// Effective levels above this need the Forgeheart Ember (a key item) held.
    pub ember_above: u8,
    pub ember_item: String,
    /// The Temper Whetstone a temper consumes.
    pub whetstone_item: String,
    pub tempers: Vec<Temper>,
}
