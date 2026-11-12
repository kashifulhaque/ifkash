//! Trapdoors (WP-062; docs/game/02-world.md, "Hollowdeep" Loop: the Root Stair "opens a
//! one-way trapdoor next to L22").
//!
//! A trapdoor is a hatch in the ceiling at the top of a stair. The player stands under it and
//! pushes it open with `INTERACT` (the flag `trapdoor.<id>.open`). Opening it slides the
//! hatch aside and, if the trapdoor names a ladder, drops that kick-down ladder, so the way up
//! to the next area is open. It is one-way: nothing here opens it from above (the Spire side
//! has its own prompt, "Opens from the other side"), and it never shuts again.
//!
//! The hatch is a thin box collider (a mover) that sits over its hole until it opens, then
//! slides `2 * half_extents.x` along -x. Movers are numbered after the lifts, doors, and hollow
//! walls, as in `mechanics`.

use super::{Layout, flags, mechanic};
use crate::world::World;

/// The hatch was pushed open (WASM event kind 14, Mechanic; index is the trapdoor's).
pub const TRAPDOOR_OPENED: u8 = 43;

/// The prompt code (`interact_prompt`): push the hatch open.
pub const PROMPT_OPEN: u32 = 32;

/// Half the hatch's thickness, in meters: a 0.3 m slab.
const HALF_THICK_M: f32 = 0.15;

/// Flag name for "this trapdoor is open".
pub fn open_flag(id: &str) -> String {
    format!("trapdoor.{id}.open")
}

/// The mover index of trapdoor `i`: after every lift, door, and hollow wall.
fn mover(layout: &Layout, i: usize) -> usize {
    layout.lifts.len() + layout.doors.len() + layout.hollow_walls.len() + i
}

fn center(t: &lf_content::scenario::Trapdoor, open: bool) -> [f32; 3] {
    let slide = if open { -t.half_extents[0] * 2.0 } else { 0.0 };
    [t.pos[0] + slide, t.pos[1] + HALF_THICK_M, t.pos[2]]
}

/// Adds the hatches to the physics world, after the other movers.
pub fn install(world: &mut World) {
    for t in &world.layout.trapdoors {
        world.physics.add_mover(
            center(t, false),
            [t.half_extents[0], HALF_THICK_M, t.half_extents[1]],
        );
    }
}

/// Puts every hatch where its flag says.
pub fn sync(world: &mut World) {
    for (i, t) in world.layout.trapdoors.iter().enumerate() {
        let open = world.state.flags.is_set(&open_flag(&t.id));
        world
            .physics
            .set_mover(mover(&world.layout, i), center(t, open));
    }
}

/// Whether trapdoor `i` is open.
pub fn is_open(world: &World, i: usize) -> bool {
    world
        .state
        .flags
        .is_set(&open_flag(&world.layout.trapdoors[i].id))
}

/// Where the player's `INTERACT` reaches trapdoor `i` (feet), if it is shut.
pub fn spot(world: &World, i: usize) -> Option<[f32; 3]> {
    (!is_open(world, i)).then(|| world.layout.trapdoors[i].stand)
}

/// The player pushed trapdoor `i` open.
pub fn interact(world: &mut World, i: usize) {
    let t = world.layout.trapdoors[i].clone();
    world.state.flags.raise(&open_flag(&t.id));
    if let Some(ladder) = &t.ladder {
        world.state.flags.raise(&flags::ladder_down(ladder));
    }
    mechanic(world, TRAPDOOR_OPENED, i);
    sync(world);
}

/// The renderer's words: 1 for each open trapdoor.
pub fn render_words(world: &World) -> impl Iterator<Item = f32> + '_ {
    (0..world.layout.trapdoors.len()).map(|i| f32::from(u8::from(is_open(world, i))))
}
