//! Consumables, flasks, and pickups. Loaded from `consumables.ron`.
//! See docs/game/04-arsenal-and-builds.md, "Consumables", and
//! docs/game/06-progression-and-systems.md, "Flasks" and "Equipment slots".

use serde::{Deserialize, Serialize};

use crate::moves::StatusKind;

/// What an item is. The kind says where it can go (a quick slot, the arrow slot); the
/// item's [`ItemEffect`] says what using it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    /// The HP flask. Charges are shared with the Lucid Flask, so it has no stack.
    VigilFlask,
    /// The FP flask.
    LucidFlask,
    Throwable,
    WeaponBuff,
    Cure,
    Glimmer,
    LanternAsh,
    Ammunition,
    /// Picked up and used on the spot: adds one flask charge.
    WickBead,
    /// Picked up and used on the spot: raises both flasks' potency by one level.
    FlaskDew,
    /// A key item: it sits in the inventory until a keyed door takes it (WP-042).
    Key,
    /// The Echo Bell (WP-059): in a quick slot, using it at an Echo Stone summons an Echo.
    /// Keeps a stack of one.
    EchoBell,
    /// An Art Scroll (WP-054): used from a quick slot, it changes the held weapon's Art.
    ArtScroll,
}

/// What using an item does (WP-057). `phys` and `elem` are attack rating in whole
/// points, so a thrown item's damage doesn't scale with the thrower's weapon.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemEffect {
    /// Nothing happens on use (flasks, pickups, keys, and items nobody has wired yet).
    #[default]
    None,
    /// Throw a projectile that deals `phys` physical and `elem` elemental damage; the
    /// projectile definition says which type and element, and its status buildup.
    Throw {
        projectile: String,
        phys: u32,
        elem: u32,
    },
    /// Put the weapon buff `buff` on the player's weapon.
    Oil { buff: String },
    /// Clear the meters and effects of these statuses.
    Cure { kinds: Vec<StatusKind> },
    /// Add Glimmer.
    Glimmer { amount: u32 },
    /// Return to the last Lantern rested at.
    LanternAsh,
    /// Ammunition: a bow fires `projectile` instead of its arrow while this item sits in
    /// the arrow slot. `elem_percent` adds the projectile's element at that percent of
    /// the shooter's physical attack rating.
    Ammo {
        projectile: String,
        elem_percent: u8,
    },
    /// Changes the right-hand weapon's Art to `art` (WP-054). The weapon must be common and
    /// its class must be one the Art lists; otherwise using the scroll does nothing.
    ArtScroll { art: String },
}

impl ItemEffect {
    /// The projectile ID this effect names, if any.
    pub fn projectile(&self) -> Option<&str> {
        match self {
            Self::Throw { projectile, .. } | Self::Ammo { projectile, .. } => Some(projectile),
            _ => None,
        }
    }
}

/// One item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
    /// The most the inventory holds. 0 for items with no stack (the flasks, and the
    /// pickups that apply on the spot).
    pub stack: u16,
    /// What using it does. Omitted for items with no effect.
    #[serde(default)]
    pub effect: ItemEffect,
}

/// Flask numbers (docs/game/06-progression-and-systems.md, "Flasks"; docs/game/03-combat.md,
/// "Combat verbs", Drink a flask).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flask {
    /// The Vigil Flask item ID.
    pub vigil: String,
    /// The Lucid Flask item ID.
    pub lucid: String,
    /// The move a flask drink plays.
    pub drink_move: String,
    /// The Wick Bead item ID.
    pub wick_bead: String,
    /// The Flask Dew item ID.
    pub flask_dew: String,
    /// Charges the player starts with.
    pub start_charges: u8,
    /// How many of the starting charges go to the Vigil Flask; the rest go to the Lucid Flask.
    pub start_vigil: u8,
    /// Most charges a Wick Bead can raise the total to.
    pub max_charges: u8,
    /// Highest potency level a Flask Dew can raise the flasks to.
    pub max_level: u8,
    /// HP healed at level +0, and added per level.
    pub heal_base: u16,
    pub heal_per_level: u16,
    /// FP restored at level +0, and added per level.
    pub fp_base: u16,
    pub fp_per_level: u16,
    /// The drink move's frame on which the effect lands (and the charge is spent).
    pub effect_frame: u16,
    /// Walk speed while drinking, as a percent of the walk speed.
    pub walk_percent: u8,
}

/// The whole `consumables.ron` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Consumables {
    pub flask: Flask,
    pub items: Vec<Item>,
}

impl Consumables {
    /// Every item ID the file references, with the field path that names it.
    pub fn references(&self) -> Vec<(String, &str)> {
        let f = &self.flask;
        vec![
            ("flask.vigil".to_owned(), f.vigil.as_str()),
            ("flask.lucid".to_owned(), f.lucid.as_str()),
            ("flask.wick_bead".to_owned(), f.wick_bead.as_str()),
            ("flask.flask_dew".to_owned(), f.flask_dew.as_str()),
        ]
    }

    pub fn item(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Problems in the file, as `(field, message)`.
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let f = &self.flask;
        let mut seen = std::collections::BTreeSet::new();
        for (i, item) in self.items.iter().enumerate() {
            if !seen.insert(item.id.as_str()) {
                out.push((format!("items[{i}].id"), "duplicate item ID".to_owned()));
            }
            let stackable = !matches!(
                item.kind,
                ItemKind::VigilFlask
                    | ItemKind::LucidFlask
                    | ItemKind::WickBead
                    | ItemKind::FlaskDew
            );
            if stackable && item.stack == 0 {
                out.push((format!("items[{i}].stack"), "must be at least 1".to_owned()));
            }
            // WP-057: a usable kind needs the matching effect, and other kinds none.
            let consistent = match (item.kind, &item.effect) {
                (ItemKind::Throwable, ItemEffect::Throw { .. })
                | (ItemKind::WeaponBuff, ItemEffect::Oil { .. })
                | (ItemKind::Cure, ItemEffect::Cure { .. })
                | (ItemKind::Glimmer, ItemEffect::Glimmer { .. })
                | (ItemKind::LanternAsh, ItemEffect::LanternAsh)
                | (ItemKind::Ammunition, ItemEffect::Ammo { .. })
                | (ItemKind::ArtScroll, ItemEffect::ArtScroll { .. }) => true,
                (
                    ItemKind::Throwable
                    | ItemKind::WeaponBuff
                    | ItemKind::Cure
                    | ItemKind::Glimmer
                    | ItemKind::LanternAsh
                    | ItemKind::Ammunition
                    | ItemKind::ArtScroll,
                    _,
                ) => false,
                (_, e) => *e == ItemEffect::None,
            };
            if !consistent {
                out.push((
                    format!("items[{i}].effect"),
                    "doesn't match the item's kind".to_owned(),
                ));
            }
        }
        for (field, id, want) in [
            ("flask.vigil", &f.vigil, ItemKind::VigilFlask),
            ("flask.lucid", &f.lucid, ItemKind::LucidFlask),
            ("flask.wick_bead", &f.wick_bead, ItemKind::WickBead),
            ("flask.flask_dew", &f.flask_dew, ItemKind::FlaskDew),
        ] {
            match self.item(id) {
                Some(item) if item.kind == want => {}
                Some(_) => out.push((
                    field.to_owned(),
                    format!("item \"{id}\" has the wrong kind"),
                )),
                None => out.push((field.to_owned(), format!("item \"{id}\" is not defined"))),
            }
        }
        if f.start_vigil > f.start_charges {
            out.push((
                "flask.start_vigil".to_owned(),
                "more than start_charges".to_owned(),
            ));
        }
        if f.start_charges > f.max_charges {
            out.push((
                "flask.start_charges".to_owned(),
                "more than max_charges".to_owned(),
            ));
        }
        if f.walk_percent > 100 {
            out.push(("flask.walk_percent".to_owned(), "more than 100".to_owned()));
        }
        out
    }
}
