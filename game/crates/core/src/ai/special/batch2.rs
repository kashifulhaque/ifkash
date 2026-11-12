//! The per-tick hook for the batch-2 special rules (WP-036). `ai::run` calls [`tick`] for
//! each live AI actor after its steering, so each rule sees the velocity the actor chose.
//! Flight is the exception: it replaces the controller step in locomotion (see [`flying`]).

use super::{reeds, reflect, swarm, water};
use crate::arena::Id;
use crate::world::World;

/// Runs the rules the actor's definition selects.
pub fn tick(world: &mut World, id: Id) {
    water::confine(world, id);
    reeds::tick(world, id);
    swarm::tick(world, id);
    reflect::tick(world, id);
}
