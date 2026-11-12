//! The whole game state.

use lf_content::{Content, Locomotion};
use serde::{Deserialize, Serialize};

pub use crate::actor::Actor;
use crate::actor::Motor;
use crate::actor::stats::Stats;
use crate::arena::{Arena, Id};
use crate::combat::damage::DamageTable;
use crate::combat::hurtbox::Hurtbox;
use crate::combat::moves::MoveTable;
use crate::combat::poise::PoiseState;
use crate::events::EventQueue;
use crate::items::{Inventory, ItemTable};
use crate::physics::PhysicsState;
use crate::rng::RngStreams;
use crate::world_state::lantern::dummy_actor;
use crate::world_state::{Layout, WorldState};

/// Seconds per tick (30 Hz).
pub const DT: f32 = 1.0 / 30.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub tick: u64,
    pub rng: RngStreams,
    /// The static level and its query structures. Not hashed (see `PhysicsState`).
    pub physics: PhysicsState,
    /// Player locomotion tuning copied from content. Not hashed: the content hash covers it.
    pub locomotion: Locomotion,
    /// Every move and the player's moveset, from content. Not hashed: the content hash
    /// covers it.
    pub moves: MoveTable,
    /// Item definitions from content. Not hashed: the content hash covers them.
    pub items: ItemTable,
    /// The damage rules and every weapon's attack rating inputs, from content. Not
    /// hashed: the content hash covers them.
    pub damage: DamageTable,
    pub actors: Arena<Actor>,
    /// The player actor, spawned by `World::new`.
    pub player: Id,
    /// Lanterns and enemy spawns from the scenario, for the world state to refer to. Not
    /// hashed: the scenario name is, and the content hash covers the scenario.
    pub layout: Layout,
    /// Lanterns, the respawn point, the Remnant, and world flags (WP-029).
    pub state: WorldState,
    /// Drained by the host every tick; excluded from the state hash.
    pub events: EventQueue,
}

/// Why a world couldn't be built from content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldError {
    UnknownScenario(String),
    /// The content has no `locomotion.ron`.
    MissingLocomotion,
    /// A move or moveset references a move the content doesn't define.
    Moves(String),
    /// An item or flask references an ID the registry doesn't have.
    Items(String),
}

impl core::fmt::Display for WorldError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownScenario(name) => write!(
                f,
                "unknown scenario {name:?} (no scenarios/{name}.ron in the content)"
            ),
            Self::MissingLocomotion => write!(f, "the content has no locomotion.ron"),
            Self::Moves(message) => write!(f, "{message}"),
            Self::Items(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for WorldError {}

/// Borrowed view of everything that is hashed: the world minus the event queue, the
/// physics structures, and the tuning copy.
#[derive(Serialize)]
struct HashView<'a> {
    tick: u64,
    rng: &'a RngStreams,
    scenario: &'a str,
    actors: &'a Arena<Actor>,
    state: &'a WorldState,
}

impl World {
    /// Starts a fresh world from `seed` in the scenario named `scenario` (the file
    /// `scenarios/<scenario>.ron`), with the player at the scenario's spawn.
    pub fn new(seed: u64, content: &Content, scenario: &str) -> Result<Self, WorldError> {
        let level = content
            .scenario(scenario)
            .ok_or_else(|| WorldError::UnknownScenario(scenario.to_owned()))?;
        let locomotion = content.locomotion.ok_or(WorldError::MissingLocomotion)?;
        let stats = Stats::player_stub(&locomotion);
        let derived = stats.attrs.derived(&content.attributes);
        let (max_hp, max_stamina, max_fp) = (derived.max_hp, derived.max_stamina, derived.max_fp);
        let damage = DamageTable::from_content(content).map_err(WorldError::Items)?;
        let moves = MoveTable::from_content(content).map_err(WorldError::Moves)?;
        let hurtbox = Hurtbox::from_locomotion(&locomotion);
        let items = ItemTable::from_content(content).map_err(WorldError::Items)?;
        let mut actors = Arena::new();
        let mut inventory = Inventory::starting(&items);
        // A scenario can start the player with another weapon in the right hand.
        if let Some(id) = &level.player_weapon {
            let w = content
                .registry
                .num(id)
                .filter(|&n| items.is_weapon(n))
                .ok_or_else(|| {
                    WorldError::Items(format!("player_weapon \"{id}\" is not a weapon"))
                })?;
            inventory.give_gear(w);
            inventory.equipment.slots[usize::from(crate::items::slot::RIGHT_1)] = Some(w);
        }
        let player = actors.insert(Actor {
            pos: level.spawn,
            yaw: level.spawn_yaw,
            is_player: true,
            hp: max_hp,
            max_hp,
            stamina: max_stamina,
            max_stamina,
            fp: max_fp,
            max_fp,
            inventory: Inventory {
                glimmer: level.start_glimmer,
                ..inventory
            },
            poise: PoiseState {
                poise: locomotion.poise.stub_poise,
                ..PoiseState::default()
            },
            stats,
            motor: Motor {
                grounded: true,
                ground_y: level.spawn[1],
                fall_peak_y: level.spawn[1],
                ..Motor::default()
            },
            hurtbox,
            ..Actor::default()
        });
        for d in &level.dummies {
            if let Some(at) = &d.attack
                && moves.num_of(&at.move_id).is_none()
            {
                return Err(WorldError::Moves(format!(
                    "dummy {:?}: unknown move \"{}\"",
                    d.name, at.move_id
                )));
            }
        }
        let layout = Layout::from_scenario(level);
        let enemies = layout
            .dummies
            .iter()
            .map(|d| actors.insert(dummy_actor(d, hurtbox, &moves)))
            .collect();
        let state = WorldState::new(&layout, enemies);
        let mut world = Self {
            tick: 0,
            rng: RngStreams::from_seed(seed),
            physics: PhysicsState::new(scenario, level),
            locomotion,
            moves,
            items,
            damage,
            actors,
            player,
            layout,
            state,
            events: EventQueue::default(),
        };
        world.sync_player_moveset();
        Ok(world)
    }

    /// Points the player's active moveset (`moves.player_moveset`) at the equipped
    /// right-hand weapon. With nothing in the right hand the moveset stays as it was, a
    /// stand-in until fists exist. Called whenever equipment can change.
    pub fn sync_player_moveset(&mut self) {
        let weapon = self
            .actors
            .get(self.player)
            .and_then(|p| p.inventory.equipment.right_weapon());
        if let Some(set) = weapon.and_then(|w| self.moves.moveset_of(w)) {
            self.moves.player_moveset = Some(set.clone());
        }
    }

    /// 64-bit xxh3 of the canonical postcard serialization, excluding events.
    pub fn state_hash(&self) -> u64 {
        let view = HashView {
            tick: self.tick,
            rng: &self.rng,
            scenario: &self.physics.scenario,
            actors: &self.actors,
            state: &self.state,
        };
        let bytes = postcard::to_allocvec(&view).expect("world serialization cannot fail");
        xxhash_rust::xxh3::xxh3_64(&bytes)
    }
}
