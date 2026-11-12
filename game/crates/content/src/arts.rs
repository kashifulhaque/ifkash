//! Arts: the weapon skills that cost FP (docs/game/04-arsenal-and-builds.md, "Arts").
//! A weapon names its Art by ID (`Weapon::default_art`), and the validator checks that the
//! Art exists and lists the weapon's class. The Arts' effects come in WP-054; until then
//! every entry is a stub.

use serde::{Deserialize, Serialize};

use crate::weapons::WeaponClass;

/// One Art (`arts.ron`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Art {
    pub id: String,
    pub name: String,
    /// The weapon classes that can use the Art. A weapon may name only an Art that lists
    /// its class.
    pub classes: Vec<WeaponClass>,
    /// The FP the Art costs.
    pub fp: u16,
    /// A placeholder for an Art a later WP implements; the validator warns on it.
    #[serde(default)]
    pub stub: bool,
}

/// The whole `arts.ron` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtFile {
    pub arts: Vec<Art>,
}
