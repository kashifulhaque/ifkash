//! Shimmer invisibility (docs/game/05-enemies-and-bosses.md, "Enemy roster", Glasswing
//! Stalker): an enemy with `shimmers` is nearly invisible while it stands still, and shows as
//! a shimmer while it moves or attacks. It changes nothing in the simulation: the snapshot's
//! `CLOAKED` flag carries it to the renderer.

use crate::actor::Actor;
use crate::world::World;

/// Planar speed, in meters per second, below which the actor counts as standing still.
/// Anything the controller would call movement is above it; not a gameplay number.
const STILL_SPEED: f32 = 0.05;

/// True if `a` is a shimmering enemy that is standing still and not in a move.
pub fn cloaked(world: &World, a: &Actor) -> bool {
    let Some(b) = a.ai.as_ref() else {
        return false;
    };
    let Some(d) = world.ai.get(b.def) else {
        return false;
    };
    d.data.shimmers
        && !a.dead
        && a.moves.current.is_none()
        && a.vel[0].abs() <= STILL_SPEED
        && a.vel[2].abs() <= STILL_SPEED
}
