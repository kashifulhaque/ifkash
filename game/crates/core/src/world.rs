//! The whole game state.

use serde::{Deserialize, Serialize};

use crate::arena::Arena;
use crate::events::EventQueue;
use crate::rng::RngStreams;

/// Placeholder actor. Later work packages add the real fields.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Actor {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub tick: u64,
    pub rng: RngStreams,
    pub actors: Arena<Actor>,
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
        Self {
            tick: 0,
            rng: RngStreams::from_seed(seed),
            actors: Arena::new(),
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
