//! World flags: named values the level and the story read and write (a boss is beaten,
//! a door is open, a Lantern is lit). Ordered (a `BTreeMap`), so hashing and saving are
//! deterministic.
//!
//! Keys are dotted lowercase IDs, `<kind>.<id>.<what>`, for example
//! `lantern.lantern_field.lit`. A flag that was never set reads as 0.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Flag name for "this Lantern has been rested at".
pub fn lantern_lit(lantern_id: &str) -> String {
    format!("lantern.{lantern_id}.lit")
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flags(BTreeMap<String, u32>);

impl Flags {
    /// The flag's value, 0 if unset.
    pub fn get(&self, key: &str) -> u32 {
        self.0.get(key).copied().unwrap_or(0)
    }

    pub fn is_set(&self, key: &str) -> bool {
        self.get(key) > 0
    }

    /// Sets the flag. Setting 0 clears it, so an unset flag and a zero flag hash the same.
    pub fn set(&mut self, key: &str, value: u32) {
        if value == 0 {
            self.0.remove(key);
        } else {
            self.0.insert(key.to_owned(), value);
        }
    }

    /// Sets the flag to 1.
    pub fn raise(&mut self, key: &str) {
        self.set(key, 1);
    }

    pub fn clear(&mut self, key: &str) {
        self.0.remove(key);
    }

    /// Every set flag, in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, u32)> {
        self.0.iter().map(|(k, v)| (k.as_str(), *v))
    }
}
