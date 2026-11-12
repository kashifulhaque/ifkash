//! Reed hiding (docs/game/05-enemies-and-bosses.md, "Enemy roster", Reed Stalker): an enemy
//! with `hides_in_reeds` is hidden while it stands in one of the scenario's reed patches. An
//! attack reveals it for `reveal_ticks` after the move ends, and the `Burrow` behavior takes
//! it back to the nearest patch. Options read the state through `Condition::Hidden` and
//! `Condition::Exposed`, and the renderer through the snapshot's `HIDDEN` flag.

use super::water;
use crate::actor::Actor;
use crate::arena::Id;
use crate::world::World;

/// The center of the reed patch nearest to `pos`, at the height of `pos`.
pub fn nearest_patch(world: &World, pos: [f32; 3]) -> Option<[f32; 3]> {
    let mut best: Option<(f32, [f32; 3])> = None;
    for z in &world.layout.reeds {
        let (dx, dz) = (z.center[0] - pos[0], z.center[1] - pos[2]);
        let d = dx * dx + dz * dz;
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, [z.center[0], pos[1], z.center[1]]));
        }
    }
    best.map(|(_, p)| p)
}

/// One tick of the reed state of `id`.
pub fn tick(world: &mut World, id: Id) {
    let Some(a) = world.actors.get(id) else {
        return;
    };
    let Some(b) = a.ai.as_ref() else {
        return;
    };
    let Some(rule) = world.ai.def(b.def).data.hides_in_reeds else {
        return;
    };
    let attacking = a.moves.current.is_some();
    let in_patch = water::in_any(&world.layout.reeds, a.pos[0], a.pos[2]);
    let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) else {
        return;
    };
    b.reveal = if attacking {
        rule.reveal_ticks
    } else {
        b.reveal.saturating_sub(1)
    };
    b.hidden = in_patch && b.reveal == 0;
}

/// True if the actor is hidden in reeds (for the snapshot).
pub fn hidden(a: &Actor) -> bool {
    a.ai.as_ref().is_some_and(|b| b.hidden)
}
