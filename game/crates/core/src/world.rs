//! The whole game state.

use lf_content::{Content, Locomotion};
use serde::{Deserialize, Serialize};

pub use crate::actor::Actor;
use crate::actor::Motor;
use crate::arena::{Arena, Id};
use crate::events::EventQueue;
use crate::physics::PhysicsState;
use crate::rng::RngStreams;

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
    pub actors: Arena<Actor>,
    /// The player actor, spawned by `World::new`.
    pub player: Id,
    /// Drained by the host every tick; excluded from the state hash.
    pub events: EventQueue,
}

/// Why a world couldn't be built from content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldError {
    UnknownScenario(String),
    /// The content has no `locomotion.ron`.
    MissingLocomotion,
}

impl core::fmt::Display for WorldError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownScenario(name) => write!(
                f,
                "unknown scenario {name:?} (no scenarios/{name}.ron in the content)"
            ),
            Self::MissingLocomotion => write!(f, "the content has no locomotion.ron"),
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
}

impl World {
    /// Starts a fresh world from `seed` in the scenario named `scenario` (the file
    /// `scenarios/<scenario>.ron`), with the player at the scenario's spawn.
    pub fn new(seed: u64, content: &Content, scenario: &str) -> Result<Self, WorldError> {
        let level = content
            .scenario(scenario)
            .ok_or_else(|| WorldError::UnknownScenario(scenario.to_owned()))?;
        let locomotion = content.locomotion.ok_or(WorldError::MissingLocomotion)?;
        let attrs = &content.attributes;
        let max_hp = (attrs.max_hp_tenths(locomotion.stub_vitality) / 10).max(1) as u32;
        let max_stamina = attrs.max_stamina_tenths(locomotion.stub_endurance) as f32 / 10.0;
        let mut actors = Arena::new();
        let player = actors.insert(Actor {
            pos: level.spawn,
            yaw: level.spawn_yaw,
            is_player: true,
            hp: max_hp,
            max_hp,
            stamina: max_stamina,
            max_stamina,
            motor: Motor {
                grounded: true,
                ground_y: level.spawn[1],
                fall_peak_y: level.spawn[1],
                ..Motor::default()
            },
            ..Actor::default()
        });
        for d in &level.dummies {
            actors.insert(Actor {
                pos: d.pos,
                yaw: d.yaw,
                hp: d.hp,
                max_hp: d.hp,
                motor: Motor {
                    grounded: true,
                    ground_y: d.pos[1],
                    fall_peak_y: d.pos[1],
                    ..Motor::default()
                },
                ..Actor::default()
            });
        }
        Ok(Self {
            tick: 0,
            rng: RngStreams::from_seed(seed),
            physics: PhysicsState::new(scenario, level),
            locomotion,
            actors,
            player,
            events: EventQueue::default(),
        })
    }

    /// 64-bit xxh3 of the canonical postcard serialization, excluding events.
    pub fn state_hash(&self) -> u64 {
        let view = HashView {
            tick: self.tick,
            rng: &self.rng,
            scenario: &self.physics.scenario,
            actors: &self.actors,
        };
        let bytes = postcard::to_allocvec(&view).expect("world serialization cannot fail");
        xxhash_rust::xxh3::xxh3_64(&bytes)
    }
}
