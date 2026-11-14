//! The render snapshot: a flat, `#[repr(C)]` array of per-actor records that the browser
//! host reads straight out of WASM memory. See docs/game/01-architecture.md, "Snapshot layout".
//!
//! The snapshot buffer is owned by the host (`lf-wasm`), not by `World`, so it can never
//! affect the state hash.

use crate::arena::Id;
use crate::world::World;

/// Bits of `ActorSnap::flags`, in the order of docs/game/01-architecture.md.
pub mod flags {
    pub const DEAD: u16 = 1 << 0;
    pub const STAGGERED: u16 = 1 << 1;
    pub const GUARDING: u16 = 1 << 2;
    pub const INVULNERABLE: u16 = 1 << 3;
    /// The actor the player is currently locked on to.
    pub const LOCKED_TARGET: u16 = 1 << 4;
    pub const TWO_HANDING: u16 = 1 << 5;
    /// A shimmering enemy standing still: nearly invisible (Glasswing Stalker, WP-036).
    pub const CLOAKED: u16 = 1 << 6;
    /// An enemy hidden in reeds (Reed Stalker, WP-036).
    pub const HIDDEN: u16 = 1 << 7;
    /// Off the ground: jumping, falling, or walked off an edge. Set only on actors the
    /// controller moves (the player and walking AI actors). The renderer picks the jump and
    /// fall poses from this bit, not from position deltas.
    pub const AIRBORNE: u16 = 1 << 8;
}

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
    /// Registry number of the enemy definition (7000 is the Husk), for model lookup. 0 for the
    /// player and any actor with no AI brain.
    pub kind: u16,
    /// See [`flags`]. Every bit is set: dead, staggered, guarding, invulnerable (i-frames),
    /// locked target, two-handing, cloaked, hidden, and airborne.
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

/// One projectile in flight, for the renderer (WP-027). 32 bytes, 4-byte aligned, no
/// padding. Kept in a separate buffer from the actor records, so the actor layout the
/// TypeScript reader mirrors doesn't change; WP-028 wires the buffer into `lf-wasm` and
/// the reader (`write_projectile_snapshot` is the hook).
///
/// | Offset | Field | Type |
/// |---|---|---|
/// | 0 | `id` | `u32` |
/// | 4 | `def` | `u32` |
/// | 8 | `pos` | `f32 x 3` |
/// | 20 | `vel` | `f32 x 3` |
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProjectileSnap {
    /// Arena handle, packed like [`ActorSnap::id`]. The same handle for a projectile's
    /// whole flight; it may be reused after the projectile ends.
    pub id: u32,
    /// The projectile definition's registry number (`proj_arrow` is 5000): model lookup.
    pub def: u32,
    /// Meters.
    pub pos: [f32; 3],
    /// Meters per second. Orient the model along it.
    pub vel: [f32; 3],
}

/// Size of one projectile record in bytes, and in 4-byte words.
pub const PROJECTILE_SNAP_BYTES: usize = 32;
pub const PROJECTILE_SNAP_WORDS: usize = PROJECTILE_SNAP_BYTES / 4;

const _: () = assert!(size_of::<ProjectileSnap>() == PROJECTILE_SNAP_BYTES);
const _: () = assert!(align_of::<ProjectileSnap>() == 4);

/// Clears `out` and writes one record per projectile in flight, in arena (index) order.
pub fn write_projectile_snapshot(world: &World, out: &mut Vec<ProjectileSnap>) {
    out.clear();
    for (id, p) in world.projectiles.iter() {
        out.push(ProjectileSnap {
            id: pack_id(id),
            def: p.def,
            pos: p.pos,
            vel: p.vel,
        });
    }
}

/// Packs an arena handle into one `u32` for the host: `generation (12 bits) << 20 | index (20 bits)`.
pub fn pack_id(id: Id) -> u32 {
    ((id.generation & 0xFFF) << 20) | (id.index & 0xF_FFFF)
}

/// Clears `out` and writes one record per live actor, in arena (index) order.
pub fn write_snapshot(world: &World, out: &mut Vec<ActorSnap>) {
    out.clear();
    let locked = world.actors.get(world.player).and_then(|p| p.lock_target);
    for (id, actor) in world.actors.iter() {
        let mut bits = 0;
        if actor.dead || actor.hp == 0 {
            bits |= flags::DEAD;
        }
        if locked == Some(id) {
            bits |= flags::LOCKED_TARGET;
        }
        // Staggered: any lockout from a stagger, guard break, stance break, or parry.
        if actor.moves.guard.stagger > 0 {
            bits |= flags::STAGGERED;
        }
        if actor.moves.guard.active {
            bits |= flags::GUARDING;
        }
        if crate::combat::dodge::is_invulnerable(&actor.moves) {
            bits |= flags::INVULNERABLE;
        }
        if actor.stats.two_handing {
            bits |= flags::TWO_HANDING;
        }
        if crate::ai::special::shimmer::cloaked(world, actor) {
            bits |= flags::CLOAKED;
        }
        if crate::ai::special::reeds::hidden(actor) {
            bits |= flags::HIDDEN;
        }
        // Only actors the controller moves have a grounded state (`systems::locomotion`):
        // the player and walking AI actors. Scripted dummies and flyers never touch it.
        let walks = id == world.player
            || (actor.ai.is_some() && !crate::ai::special::flying::flies(world, id));
        if walks && !actor.motor.grounded {
            bits |= flags::AIRBORNE;
        }
        let current = actor.moves.current;
        out.push(ActorSnap {
            id: pack_id(id),
            // The enemy definition's registry number (7000 is the Husk); 0 for the player and
            // any actor without an AI brain. Registry numbers below 65536 fit in a u16.
            kind: actor
                .ai
                .as_ref()
                .and_then(|b| world.ai.get(b.def))
                .map_or(0, |d| u16::try_from(d.num).unwrap_or(0)),
            flags: bits,
            pos: actor.pos,
            yaw: actor.yaw,
            // The move's registry number (0 for none); registry numbers fit in a u16.
            move_id: current.map_or(0, |c| u16::try_from(c.num).unwrap_or(u16::MAX)),
            move_frame: current.map_or(0, |c| c.frame),
            hp_frac: if actor.max_hp > 0 {
                actor.hp as f32 / actor.max_hp as f32
            } else {
                0.0
            },
            status_mask: actor.status.aura_mask(),
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
