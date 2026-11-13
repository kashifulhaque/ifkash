//! Warden Braziers and seal sigils (WP-066; docs/game/02-world.md, "Mini-dungeons", D6 the
//! Whitewarden Catacombs and D7 the Bound Ring).
//!
//! - A **Warden Brazier** is lit with `INTERACT` from beside it, for good (the flag
//!   `brazier.<id>.lit`). A Whitewarden Skeleton's revive rule lists `LitLantern(radius)`: a lit
//!   Lantern *or a lit brazier* within that many meters keeps it dead ([`lit_near`]). The rule is
//!   checked when it dies (`ai::special::revive::note_death_blow`) and again each tick it lies
//!   dead, so lighting a brazier also stops the skeletons already down beside it from rising.
//!   A skeleton that is up stays up: the brazier stops revives, it doesn't kill.
//! - A **seal sigil** is touched with `INTERACT`. That raises `sigil.<id>.touched`. If the sigil
//!   is `armed` it also raises `sigil.<id>.sealed`, and [`step`] shuts every door it names; until
//!   then those doors stand open (every tick the doors follow the flags: open unless sealed, or
//!   once `sigil.<id>.released` is raised, which M7's B11 does when the prisoner falls). An
//!   unarmed sigil is cold, a placeholder until the boss exists.
//!
//! Everything is a world flag; there is no state of its own.

use super::{flags, mechanic, set_door};
use crate::ai::perception::dist_xz;
use crate::world::World;

/// A brazier was lit (WASM event kind 14, Mechanic; index is the brazier's). WP-066 reserves
/// the kinds 100-109.
pub const BRAZIER_LIT: u8 = 100;
/// A sigil was touched (index is the sigil's). Whether it sealed is the flag
/// `sigil.<id>.sealed`.
pub const SIGIL_TOUCHED: u8 = 101;
/// A sealed sigil's doors shut (index is the sigil's).
pub const SIGIL_SEALED: u8 = 102;
/// A sigil's doors opened again, after the world raised `sigil.<id>.released` (index is the
/// sigil's).
pub const SIGIL_RELEASED: u8 = 103;

/// The prompt codes (`interact_prompt`): light a brazier, touch a sigil. WP-066 reserves 55-59.
pub const PROMPT_LIGHT: u32 = 55;
pub const PROMPT_TOUCH: u32 = 56;

/// Flag name for a lit brazier.
pub fn lit_flag(id: &str) -> String {
    format!("brazier.{id}.lit")
}

/// Flag name for "the player has touched this sigil".
pub fn touched_flag(id: &str) -> String {
    format!("sigil.{id}.touched")
}

/// Flag name for "this sigil has sealed its doors".
pub fn sealed_flag(id: &str) -> String {
    format!("sigil.{id}.sealed")
}

/// Flag name for "the world has released this sigil's seal" (raised by the boss's defeat).
pub fn released_flag(id: &str) -> String {
    format!("sigil.{id}.released")
}

/// Whether the brazier `id` is lit.
pub fn is_lit(world: &World, id: &str) -> bool {
    world.state.flags.is_set(&lit_flag(id))
}

/// Whether a lit brazier stands within `radius_m` of `pos` (horizontally).
pub fn lit_near(world: &World, pos: [f32; 3], radius_m: f32) -> bool {
    world
        .layout
        .braziers
        .iter()
        .any(|b| is_lit(world, &b.id) && dist_xz(b.pos, pos) <= radius_m)
}

/// Whether the sigil `id` can still be touched: it isn't sealed, or its seal was released.
pub fn sigil_touchable(world: &World, id: &str) -> bool {
    let f = &world.state.flags;
    !f.is_set(&sealed_flag(id)) || f.is_set(&released_flag(id))
}

/// Whether the door `id` is one a sigil seals (it opens and shuts with the sigil's flags, so
/// `INTERACT` never offers it).
pub fn is_seal_door(world: &World, id: &str) -> bool {
    world
        .layout
        .sigils
        .iter()
        .any(|g| g.doors.iter().any(|d| d == id))
}

/// The player pressed `INTERACT` beside brazier `i`: light it.
pub fn light(world: &mut World, i: usize) {
    let id = world.layout.braziers[i].id.clone();
    world.state.flags.raise(&lit_flag(&id));
    mechanic(world, BRAZIER_LIT, i);
}

/// The player pressed `INTERACT` beside sigil `i`: touch it, and, if it is armed, seal.
pub fn touch(world: &mut World, i: usize) {
    let g = world.layout.sigils[i].clone();
    world.state.flags.raise(&touched_flag(&g.id));
    mechanic(world, SIGIL_TOUCHED, i);
    if g.armed {
        world.state.flags.raise(&sealed_flag(&g.id));
    }
}

/// Each tick: the doors of every sigil follow its flags. They are open until the sigil seals
/// (so the player can walk in), shut while it is sealed, and open again once it is released.
pub fn step(world: &mut World) {
    for gi in 0..world.layout.sigils.len() {
        let g = world.layout.sigils[gi].clone();
        let f = &world.state.flags;
        let sealed = f.is_set(&sealed_flag(&g.id));
        let open = !sealed || f.is_set(&released_flag(&g.id));
        for id in &g.doors {
            let Some(d) = world.layout.doors.iter().position(|d| &d.id == id) else {
                continue;
            };
            if world.state.flags.is_set(&flags::door_open(id)) != open {
                set_door(world, d, open);
                let kind = if open { SIGIL_RELEASED } else { SIGIL_SEALED };
                // `set_door` already raised a door event; this one names the sigil.
                mechanic(world, kind, gi);
            }
        }
    }
}

/// The renderer's words: each brazier (1 once lit), then each sigil (0 cold, 1 touched, 2 while
/// its seal holds).
pub fn render_words(world: &World) -> Vec<f32> {
    let f = &world.state.flags;
    let mut out: Vec<f32> = world
        .layout
        .braziers
        .iter()
        .map(|b| f32::from(u8::from(is_lit(world, &b.id))))
        .collect();
    out.extend(world.layout.sigils.iter().map(|g| {
        let sealed = f.is_set(&sealed_flag(&g.id)) && !f.is_set(&released_flag(&g.id));
        if sealed {
            2.0
        } else {
            f32::from(u8::from(f.is_set(&touched_flag(&g.id))))
        }
    }));
    out
}
