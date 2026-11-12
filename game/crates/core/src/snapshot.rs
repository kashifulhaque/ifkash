//! The render snapshot: a flat, `#[repr(C)]` array of per-actor records that the browser
//! host reads straight out of WASM memory. See docs/game/01-architecture.md, "Snapshot layout".
//!
//! The snapshot buffer is owned by the host (`lf-wasm`), not by `World`, so it can never
//! affect the state hash.

use crate::arena::Id;
use crate::world::World;

/// One actor as the renderer sees it. 40 bytes, 4-byte aligned, no padding. The TypeScript
/// reader in `src/lib/latticefall/engine/snapshot.ts` mirrors these offsets exactly.
///
/// | Offset | Field | Type |
/// |---|---|---|
/// | 0 | `id` | `u32` |
/// | 4 | `kind` | `u16` |
/// | 6 | `flags` | `u16` |
/// | 8 | `pos` | `f32 x 3` |
/// | 20 | `yaw` | `f32` |
/// | 24 | `move_id` | `u16` |
/// | 26 | `move_frame` | `u16` |
/// | 28 | `hp_frac` | `f32` |
/// | 32 | `poise_frac` | `f32` |
/// | 36 | `status_mask` | `u32` |
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActorSnap {
    /// Arena handle: generation in the top 12 bits, index in the low 20 (see `pack_id`).
    pub id: u32,
    /// Content ID of the actor type, for model lookup. 0 until actor kinds exist.
    pub kind: u16,
    /// Dead, staggered, guarding, invulnerable, locked target, two-handing. None yet.
    pub flags: u16,
    /// Meters.
    pub pos: [f32; 3],
    /// Radians.
    pub yaw: f32,
    pub move_id: u16,
    pub move_frame: u16,
    pub hp_frac: f32,
    pub poise_frac: f32,
    pub status_mask: u32,
}

/// Size of one record in bytes, and in 4-byte words.
pub const ACTOR_SNAP_BYTES: usize = 40;
pub const ACTOR_SNAP_WORDS: usize = ACTOR_SNAP_BYTES / 4;

const _: () = assert!(size_of::<ActorSnap>() == ACTOR_SNAP_BYTES);
const _: () = assert!(align_of::<ActorSnap>() == 4);

/// Packs an arena handle into one `u32` for the host: `generation (12 bits) << 20 | index (20 bits)`.
pub fn pack_id(id: Id) -> u32 {
    ((id.generation & 0xFFF) << 20) | (id.index & 0xF_FFFF)
}

/// Clears `out` and writes one record per live actor, in arena (index) order.
pub fn write_snapshot(world: &World, out: &mut Vec<ActorSnap>) {
    out.clear();
    for (id, actor) in world.actors.iter() {
        out.push(ActorSnap {
            id: pack_id(id),
            pos: actor.pos,
            yaw: actor.yaw,
            hp_frac: if actor.max_hp > 0 {
                actor.hp as f32 / actor.max_hp as f32
            } else {
                0.0
            },
            ..ActorSnap::default()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_offsets() {
        assert_eq!(core::mem::offset_of!(ActorSnap, kind), 4);
        assert_eq!(core::mem::offset_of!(ActorSnap, pos), 8);
        assert_eq!(core::mem::offset_of!(ActorSnap, yaw), 20);
        assert_eq!(core::mem::offset_of!(ActorSnap, move_id), 24);
        assert_eq!(core::mem::offset_of!(ActorSnap, hp_frac), 28);
        assert_eq!(core::mem::offset_of!(ActorSnap, status_mask), 36);
    }

    #[test]
    fn snapshot_holds_the_player() {
        let content = lf_content::Content::load_embedded().unwrap();
        let mut w = World::new(1, &content, "m0_flat").unwrap();
        let id = w.player;
        w.actors.get_mut(id).unwrap().pos = [1.0, 2.0, 3.0];
        let mut out = Vec::new();
        write_snapshot(&w, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].pos, [1.0, 2.0, 3.0]);
        assert_eq!(out[0].hp_frac, 1.0);
        assert_eq!(out[0].id, pack_id(id));
    }
}
