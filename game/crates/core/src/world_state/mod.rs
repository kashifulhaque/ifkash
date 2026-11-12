//! World state that outlives a life: the Lanterns, the respawn point, the Remnant, and
//! world flags. See docs/game/06-progression-and-systems.md, "Glimmer and death" and
//! "Lanterns".
//!
//! # Where things happen
//!
//! - [`lantern::run`] is the first half of tick system 1 (`systems::apply_commands`):
//!   `Command::Rest` at a Lantern in reach, and leaving the rest.
//! - [`death::run`] is the body of tick system 9 (`systems::deaths_drops`): the player's
//!   death, Remnant drop and recovery, and respawn.
//! - [`flags`] holds the world flags (bosses beaten, doors opened, Lanterns lit).
//!
//! # What a save must keep
//!
//! Everything in [`WorldState`] (it is all in the state hash), plus the player's
//! `inventory.glimmer`. [`Layout`] is rebuilt from the scenario and isn't saved.

pub mod death;
#[cfg(test)]
mod dungeon_tests;
pub mod flags;
#[cfg(test)]
mod hollow_tests;
pub mod lantern;
pub mod lantern_menu;
pub mod mechanics;
#[cfg(test)]
mod mirrormere_tests;
pub mod quests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod verge_tests;

use lf_content::Scenario;
use lf_content::scenario::{
    Chest, Door, Dummy, EnemySpawn, HollowWall, Ladder, Lantern, Lever, Lift, Message, Pool, Zone,
};
use serde::{Deserialize, Serialize};

use crate::arena::Id;
use flags::Flags;

/// The static parts of a scenario the world state refers to. Not hashed: the scenario
/// name is, and the content hash covers the scenario.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layout {
    /// Where the player starts, and respawns until they rest at a Lantern.
    pub spawn: [f32; 3],
    pub spawn_yaw: f32,
    pub lanterns: Vec<Lantern>,
    /// The enemies that respawn, in the order `WorldState::enemies` lists them. Until
    /// real enemies exist (WP-030) these are the scenario's dummies.
    pub dummies: Vec<Dummy>,
    /// The AI enemies' spawns, in the order `WorldState::ai_enemies` lists them (WP-030).
    /// An AI actor's `AiState::spawn` indexes this list.
    pub enemies: Vec<EnemySpawn>,
    /// The boss fights (WP-033). Their members are the last entries of `enemies`; see
    /// `ai::boss::extend_layout`.
    pub bosses: Vec<crate::ai::boss::BossLayout>,
    /// Water volumes and reed patches (WP-036; see `ai::special`).
    pub water: Vec<Zone>,
    pub reeds: Vec<Zone>,
    /// Pickups with their items resolved (WP-049; see `pickups`).
    pub pickups: Vec<crate::pickups::PickupSpot>,
    /// The world mechanics (WP-042; see `mechanics`). Their state is flags, plus
    /// `mechanics::MechState` for lifts and the ladder the player is on.
    pub ladders: Vec<Ladder>,
    pub lifts: Vec<Lift>,
    pub doors: Vec<Door>,
    pub levers: Vec<Lever>,
    pub hollow_walls: Vec<HollowWall>,
    pub chests: Vec<Chest>,
    pub messages: Vec<Message>,
    pub pools: Vec<Pool>,
    pub snow: Vec<Zone>,
    /// The NPCs and stations (WP-068; see `quests`).
    pub npcs: Vec<quests::NpcSpot>,
    pub stations: Vec<quests::StationSpot>,
    /// A safe zone: no combat, weapons sheathed, walk only (WP-068).
    pub safe_zone: bool,
}

impl Layout {
    pub fn from_scenario(level: &Scenario) -> Self {
        Self {
            spawn: level.spawn,
            spawn_yaw: level.spawn_yaw,
            lanterns: level.lanterns.clone(),
            dummies: level.dummies.clone(),
            enemies: level.enemies.clone(),
            bosses: Vec::new(),
            water: level.water.clone(),
            reeds: level.reeds.clone(),
            pickups: Vec::new(),
            ladders: level.ladders.clone(),
            lifts: level.lifts.clone(),
            doors: level.doors.clone(),
            levers: level.levers.clone(),
            hollow_walls: level.hollow_walls.clone(),
            chests: level.chests.clone(),
            messages: level.messages.clone(),
            pools: level.pools.clone(),
            snow: level.snow.clone(),
            npcs: Vec::new(),
            stations: Vec::new(),
            safe_zone: false,
        }
    }

    /// The Lantern with this ID, and its index in the list.
    pub fn lantern(&self, id: &str) -> Option<(usize, &Lantern)> {
        self.lanterns.iter().enumerate().find(|(_, l)| l.id == id)
    }
}

/// An unrecovered pile of the Glimmer a dead player was holding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Remnant {
    /// Feet-level position the player touches to recover it.
    pub pos: [f32; 3],
    pub glimmer: u32,
}

/// The player's death in progress and what it left behind.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeathState {
    /// Ticks left before the respawn; `None` while the player is alive.
    pub respawn_in: Option<u16>,
    /// The last place the player's feet touched walkable, non-hazard ground (see
    /// `death::run`). Where a Remnant drops.
    pub safe_pos: [f32; 3],
    /// The Remnant waiting to be recovered, if any. At most one exists.
    pub remnant: Option<Remnant>,
    /// Deaths so far (a stat for the journal and the HUD).
    pub deaths: u32,
}

/// Simulation state: part of the state hash.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    /// Named world flags. See [`flags`].
    pub flags: Flags,
    /// ID of the Lantern the player last rested at; `None` until the first rest, when
    /// the player respawns at the scenario spawn.
    pub respawn_lantern: Option<String>,
    /// ID of the Lantern the player is resting at now: the Lantern menu is open. Ends
    /// when the player moves or acts, and on death.
    pub resting: Option<String>,
    pub death: DeathState,
    /// The actor of each `Layout::dummies` entry, in order. A respawn replaces a missing
    /// actor and updates the handle.
    pub enemies: Vec<Id>,
    /// The actor of each `Layout::enemies` spawn, in order, kept the same way.
    pub ai_enemies: Vec<Id>,
    /// With Keep Glimmer on death on (Wayfarer mode, not built yet) nothing is dropped.
    pub keep_glimmer_on_death: bool,
}

impl WorldState {
    pub fn new(layout: &Layout, enemies: Vec<Id>, ai_enemies: Vec<Id>) -> Self {
        Self {
            flags: Flags::default(),
            respawn_lantern: None,
            resting: None,
            death: DeathState {
                respawn_in: None,
                safe_pos: layout.spawn,
                remnant: None,
                deaths: 0,
            },
            enemies,
            ai_enemies,
            keep_glimmer_on_death: false,
        }
    }
}
