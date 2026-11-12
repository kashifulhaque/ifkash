//! `lf-core`: the deterministic Latticefall simulation core. Later work packages add modules.

pub mod arena;
pub mod events;
pub mod input;
pub mod math;
pub mod rng;
pub mod systems;
pub mod tick;
pub mod world;

pub use input::TickInput;
pub use tick::tick;
pub use world::World;
