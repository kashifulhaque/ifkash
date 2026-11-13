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

/// Flag name for "this boss is defeated" (WP-033).
pub fn boss_defeated(boss_id: &str) -> String {
    format!("boss.{boss_id}.defeated")
}

/// Flag name for "this codex entry is unlocked" (a boss reward).
pub fn codex(entry: &str) -> String {
    format!("codex.{entry}")
}

/// Flag name for "this door is open" (WP-042).
pub fn door_open(door_id: &str) -> String {
    format!("door.{door_id}.open")
}

/// Flag name for "this lever is pulled" (WP-042). Pulling a lever again clears it.
pub fn lever_pulled(lever_id: &str) -> String {
    format!("lever.{lever_id}.pulled")
}

/// Flag name for "a hit revealed this hollow wall" (WP-042).
pub fn wall_revealed(wall_id: &str) -> String {
    format!("wall.{wall_id}.revealed")
}

/// Flag name for "the player took this pickup" (WP-042).
pub fn pickup_taken(pickup_id: &str) -> String {
    format!("pickup.{pickup_id}.taken")
}

/// Flag name for "the player opened this chest" (WP-042).
pub fn chest_opened(chest_id: &str) -> String {
    format!("chest.{chest_id}.opened")
}

/// Flag name for "the player read this message" (WP-042).
pub fn message_read(message_id: &str) -> String {
    format!("message.{message_id}.read")
}

/// Flag name for "this kick-down ladder has dropped" (WP-042).
pub fn ladder_down(ladder_id: &str) -> String {
    format!("ladder.{ladder_id}.down")
}

/// Flag name for "a lever raised this pool's water" (WP-042).
pub fn pool_raised(pool_name: &str) -> String {
    format!("pool.{pool_name}.raised")
}

/// Flag name for "this pool's water drags a wading player twice" (WP-073: B05's rising tide).
pub fn pool_drag(pool_name: &str) -> String {
    format!("pool.{pool_name}.drag")
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
