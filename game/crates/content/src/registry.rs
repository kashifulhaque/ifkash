//! The ID registry: string ID to stable numeric ID.
//!
//! Numeric IDs are `u32`. That's wide enough for every item the game will ever define
//! and still a plain `number` in TypeScript. The registry is append-only: replays and
//! saved games store the numbers, so an entry is never renumbered, reused, or removed.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ContentError;

/// Numeric ID type stored in replays and saves.
pub type NumId = u32;

/// One line of `registry.ron`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct RegistryEntry {
    pub id: String,
    pub num: NumId,
}

/// The shape of `registry.ron`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegistryFile {
    pub entries: Vec<RegistryEntry>,
}

/// The parsed registry. A `BTreeMap` keeps iteration and hashing stable.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Registry {
    pub ids: BTreeMap<String, NumId>,
}

impl Registry {
    /// Builds a registry, failing on a repeated string ID or a repeated number.
    pub(crate) fn from_entries(entries: Vec<RegistryEntry>) -> Result<Self, ContentError> {
        let mut ids = BTreeMap::new();
        let mut by_num: BTreeMap<NumId, String> = BTreeMap::new();
        for e in entries {
            if let Some(other) = by_num.get(&e.num) {
                return Err(ContentError::Registry(format!(
                    "numeric ID {} is used by both \"{other}\" and \"{}\"",
                    e.num, e.id
                )));
            }
            if ids.insert(e.id.clone(), e.num).is_some() {
                return Err(ContentError::Registry(format!(
                    "string ID \"{}\" appears more than once",
                    e.id
                )));
            }
            by_num.insert(e.num, e.id);
        }
        Ok(Self { ids })
    }

    /// Looks up the numeric ID for a string ID.
    pub fn num(&self, id: &str) -> Option<NumId> {
        self.ids.get(id).copied()
    }

    /// Returns true when the string ID has an entry.
    pub fn contains(&self, id: &str) -> bool {
        self.ids.contains_key(id)
    }
}
