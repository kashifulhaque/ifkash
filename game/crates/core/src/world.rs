//! The whole game state.

use lf_content::{Content, Locomotion};
use serde::{Deserialize, Serialize};

pub use crate::actor::Actor;
use crate::actor::Motor;
use crate::actor::stats::Stats;
use crate::ai::{AiTable, enemy_actor};
use crate::arena::{Arena, Id};
use crate::combat::damage::DamageTable;
use crate::combat::hurtbox::Hurtbox;
use crate::combat::moves::MoveTable;
use crate::combat::poise::PoiseState;
use crate::combat::projectile::{Projectile, ProjectileTable};
use crate::combat::status::{StatusState, StatusTable};
use crate::events::EventQueue;
use crate::items::{Inventory, ItemTable, slot};
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
    /// Every projectile definition, from content. Not hashed: the content hash covers it.
    pub projectile_defs: ProjectileTable,
    /// The status and buff rules, from content. Not hashed: the content hash covers them.
    pub status: StatusTable,
    pub actors: Arena<Actor>,
    /// Projectiles in flight (docs/game/01-architecture.md, "World and actors").
    pub projectiles: Arena<Projectile>,
    /// The player actor, spawned by `World::new`.
    pub player: Id,
    /// Every enemy definition and the AI tuning, from content. Not hashed: the content
    /// hash covers it.
    pub ai: AiTable,
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
    /// The content has no `status.ron`, or a buff isn't in the registry.
    Status(String),
    /// An enemy spawn or definition can't be built (WP-030).
    Ai(String),
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
            Self::Status(message) => write!(f, "{message}"),
            Self::Ai(message) => write!(f, "{message}"),
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
    /// Left out until the first projectile spawns, so a world that never fires one hashes
    /// as it did before projectiles existed. The view is only ever serialized, never read
    /// back, so a skipped field is safe.
    #[serde(skip_serializing_if = "Arena::is_unused")]
    projectiles: &'a Arena<Projectile>,
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
        let status_table = StatusTable::from_content(content).map_err(WorldError::Status)?;
        let player_status = StatusState::for_player(&stats.attrs, &content.attributes);
        let moves = MoveTable::from_content(content).map_err(WorldError::Moves)?;
        let hurtbox = Hurtbox::from_locomotion(&locomotion);
        let items = ItemTable::from_content(content).map_err(WorldError::Items)?;
        let projectile_defs = ProjectileTable::from_content(content).map_err(WorldError::Moves)?;
        let mut actors = Arena::new();
        let inventory = Inventory::starting(&items);
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
            status: player_status,
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
        let ai = AiTable::from_content(content, &moves).map_err(WorldError::Ai)?;
        if !layout.enemies.is_empty() && ai.tuning.is_none() {
            return Err(WorldError::Ai(format!(
                "scenario {scenario:?} has enemies but the content has no ai.ron"
            )));
        }
        let mut ai_enemies = Vec::new();
        for (i, s) in layout.enemies.iter().enumerate() {
            let actor = enemy_actor(s, i as u16, &ai, hurtbox).ok_or_else(|| {
                WorldError::Ai(format!(
                    "enemy spawn {:?}: unknown enemy \"{}\"",
                    s.name, s.enemy
                ))
            })?;
            ai_enemies.push(actors.insert(actor));
        }
        let state = WorldState::new(&layout, enemies, ai_enemies);
        let mut world = Self {
            tick: 0,
            rng: RngStreams::from_seed(seed),
            physics: PhysicsState::new(scenario, level),
            locomotion,
            moves,
            items,
            damage,
            projectile_defs,
            status: status_table,
            ai,
            actors,
            projectiles: Arena::new(),
            player,
            layout,
            state,
            events: EventQueue::default(),
        };
        if let Some(id) = &level.player_weapon {
            let weapon = content
                .registry
                .num(id)
                .filter(|&n| world.items.is_weapon(n))
                .ok_or_else(|| {
                    WorldError::Items(format!(
                        "scenario {scenario:?}: player_weapon \"{id}\" is not a weapon"
                    ))
                })?;
            if let Some(p) = world.actors.get_mut(player) {
                p.inventory.give_gear(weapon);
                p.inventory.equip(slot::RIGHT_1, weapon, &world.items);
            }
        }
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
            projectiles: &self.projectiles,
        };
        let bytes = postcard::to_allocvec(&view).expect("world serialization cannot fail");
        xxhash_rust::xxh3::xxh3_64(&bytes)
    }
}
