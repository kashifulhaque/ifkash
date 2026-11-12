//! The whole game state.

use serde::{Deserialize, Serialize};

use crate::arena::{Arena, Id};
use crate::events::EventQueue;
use crate::rng::RngStreams;

/// Seconds per tick (30 Hz).
pub const DT: f32 = 1.0 / 30.0;

/// Placeholder actor. Later work packages add the real fields.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Actor {
    // Placeholder kinematics until WP-010 (physics).
    pub pos: [f32; 3],
    pub yaw: f32,
    pub vel: [f32; 3],
    pub is_player: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub tick: u64,
    pub rng: RngStreams,
    pub actors: Arena<Actor>,
    /// The player actor, spawned by `World::new`.
    pub player: Id,
    /// Drained by the host every tick; excluded from the state hash.
    pub events: EventQueue,
}

/// Borrowed view of everything that is hashed (the world minus the event queue).
#[derive(Serialize)]
struct HashView<'a> {
    tick: u64,
    rng: &'a RngStreams,
    actors: &'a Arena<Actor>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let mut actors = Arena::new();
        let player = actors.insert(Actor {
            is_player: true,
            ..Actor::default()
        });
        Self {
            tick: 0,
            rng: RngStreams::from_seed(seed),
            actors,
            player,
            events: EventQueue::default(),
        }
    }

    /// 64-bit xxh3 of the canonical postcard serialization, excluding events.
    pub fn state_hash(&self) -> u64 {
        let view = HashView {
            tick: self.tick,
            rng: &self.rng,
            actors: &self.actors,
        };
        let bytes = postcard::to_allocvec(&view).expect("world serialization cannot fail");
        xxhash_rust::xxh3::xxh3_64(&bytes)
    }
}
