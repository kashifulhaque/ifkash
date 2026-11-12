//! Consumables, flasks, and pickups. Loaded from `consumables.ron`.
//! See docs/game/04-arsenal-and-builds.md, "Consumables", and
//! docs/game/06-progression-and-systems.md, "Flasks" and "Equipment slots".

use serde::{Deserialize, Serialize};

/// What an item is. Only the flasks, Wick Bead, and Flask Dew have effects in the
/// simulation so far; the other kinds are inventory entries until WP-057 gives them effects.
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
