//! Echo definitions (`echoes/<id>.ron`, one Echo per file) and the shared Echo rules
//! (`echoes/rules.ron`). See docs/game/05-enemies-and-bosses.md, "Echo summons".
//!
//! An Echo is a summoned ally. Its file holds the **body** (an [`EnemyDef`]: HP, poise,
//! guard, and the option table its AI picks from), the **moves** the options name (inline,
//! so the whole Echo lives in one file), and what makes it an Echo: its FP cost, how many
//! bodies appear, how strongly it draws enemies' attention, and the two special rules (the
//! Choir Wisp's heal and the PoliteLlama's courtesy). The core spawns each body as an AI
//! actor on the player's side (`lf_core::ai::echo`).

use serde::{Deserialize, Serialize};

use crate::enemy::EnemyDef;
use crate::moves::Move;

/// The numbers every Echo shares (`echoes/rules.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EchoRules {
    /// The Echo Bell's item ID. Ringing it (`Command::UseItem` on its quick slot) at an
    /// Echo Stone summons the selected Echo.
    pub bell: String,
    /// The gesture number (`Command::Gesture`) of the bow. The gesture menu (WP-088) sends it.
    pub bow_gesture: u16,
    /// How close to an Echo Stone the player has to stand to ring the Bell, in meters.
    pub stone_ring_m: f32,
    /// An Echo follows the player until it is this close, in meters.
    pub follow_m: f32,
    /// An Echo farther than this from the player is put next to them, in meters.
    pub catch_up_m: f32,
    /// An Echo picks a target only among enemies this close to the player, in meters.
    pub engage_m: f32,
    /// An Echo that has drifted this far from the player gives up its target and follows,
    /// in meters.
    pub leash_m: f32,
    /// A bow counts only this many ticks after the summoning.
    pub bow_window_ticks: u16,
    /// The player has to be within this many meters of the Echo to bow to it.
    pub bow_range_m: f32,
    /// An Echo runs, instead of walking, when it is this much farther than `follow_m`.
    pub run_beyond_m: f32,
    /// Once every body of a summoned Echo has fallen, they stay this many ticks before they
    /// are removed and the Bell can summon again.
    pub fallen_ticks: u16,
}

/// One Echo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EchoDef {
    /// Registry ID, for example `echo_politellama`.
    pub id: String,
    /// The name in the Echo list.
    pub name: String,
    /// What the summoning costs, in FP.
    pub fp_cost: u16,
    /// The world flag that says the player has earned this Echo. Raised by its source (a
    /// quest or a boss reward). A scenario's `echoes_unlocked` also unlocks it.
    pub unlock_flag: String,
    /// How many bodies appear (the Briar Hounds are three).
    #[serde(default = "one")]
    pub count: u8,
    /// How far apart the bodies stand when they appear, in meters.
    #[serde(default = "default_spacing")]
    pub spacing_m: f32,
    /// Enemies in Combat within this many meters of a living body of this Echo turn on it
    /// instead of the player ("holds aggro", "taunts"). 0 draws nobody.
    #[serde(default)]
    pub aggro_m: f32,
    /// The body. Its ID needs a registry entry of its own. It is also an enemy definition
    /// in the AI table, so everything an enemy option can say works here.
    pub body: EnemyDef,
    /// The moves the body's options name, with their own registry entries.
    pub moves: Vec<Move>,
    /// The Choir Wisp's slow heal.
    #[serde(default)]
    pub heal: Option<EchoHeal>,
    /// The PoliteLlama's courtesy: a bow from the player makes it hit harder.
    #[serde(default)]
    pub courtesy: Option<Courtesy>,
}

fn one() -> u8 {
    1
}

fn default_spacing() -> f32 {
    1.4
}

/// A heal over time: every `every_ticks`, a living body heals the player (and no one else
/// yet) by `hp` if they are within `range_m`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EchoHeal {
    pub hp: u32,
    pub every_ticks: u16,
    /// Meters, as the integer part of a distance.
    pub range_m: u16,
}

/// The courtesy rule (docs "Echo summons", PoliteLlama): a bow from the player within
/// `Rules::bow_window_ticks` of the summoning sets its attack to `damage_percent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Courtesy {
    pub damage_percent: u16,
}

impl EchoDef {
    /// Every registry ID the definition references, with its field path. The body's own
    /// references are checked as an enemy's.
    pub fn references(&self) -> Vec<(String, &str)> {
        let mut out = vec![("id".to_owned(), self.id.as_str())];
        for (i, m) in self.moves.iter().enumerate() {
            out.push((format!("moves[{i}].id"), m.id.as_str()));
            if let Some(n) = &m.next_chain {
                out.push((format!("moves[{i}].next_chain"), n.as_str()));
            }
        }
        out
    }

    /// Shape problems the core relies on, as `(field, message)`.
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if self.fp_cost == 0 {
            out.push(("fp_cost".to_owned(), "must be positive".to_owned()));
        }
        if self.count == 0 || self.count > 6 {
            out.push(("count".to_owned(), "must be 1 to 6".to_owned()));
        }
        if self.spacing_m <= 0.0 || self.aggro_m < 0.0 {
            out.push((
                "spacing_m".to_owned(),
                "spacing_m must be positive and aggro_m can't be negative".to_owned(),
            ));
        }
        if self.unlock_flag.is_empty() {
            out.push(("unlock_flag".to_owned(), "must not be empty".to_owned()));
        }
        if self.heal.is_some_and(|h| h.hp == 0 || h.every_ticks == 0) {
            out.push((
                "heal".to_owned(),
                "hp and every_ticks must be positive".to_owned(),
            ));
        }
        if self.courtesy.is_some_and(|c| c.damage_percent == 0) {
            out.push((
                "courtesy.damage_percent".to_owned(),
                "must be positive".to_owned(),
            ));
        }
        out
    }
}

impl EchoRules {
    /// Shape problems the core relies on, as `(field, message)`.
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for (name, v) in [
            ("stone_ring_m", self.stone_ring_m),
            ("follow_m", self.follow_m),
            ("catch_up_m", self.catch_up_m),
            ("engage_m", self.engage_m),
            ("leash_m", self.leash_m),
            ("bow_range_m", self.bow_range_m),
            ("run_beyond_m", self.run_beyond_m),
        ] {
            if !(v.is_finite() && v > 0.0) {
                out.push((name.to_owned(), "must be positive".to_owned()));
            }
        }
        if self.follow_m >= self.catch_up_m {
            out.push((
                "follow_m".to_owned(),
                "must be less than catch_up_m".to_owned(),
            ));
        }
        out
    }
}
