//! B02, the Briar Stag (WP-070): the special rules the boss framework doesn't have. See
//! docs/game/05-enemies-and-bosses.md, "B02 The Briar Stag".
//!
//! The Stag's moves, options, and phases are data (`data/bosses/b02_briar_stag.ron` and
//! `data/moves/bosses/b02.ron`). Two rules need code, and each is switched on by a vfx name
//! in a move's data, so no move id is written here:
//!
//! - **A charge that hits a tree stuns the Stag** ([`VFX_CHARGE`]). From the move frame that
//!   plays `stag_charge`, while the move's root motion runs faster than a run, the Stag looks
//!   ahead for something solid in the way (a tree, a clump, a hedge; the level's colliders).
//!   When it is within [`IMPACT_REACH_M`] of its head, the charge ends: the move stops and the
//!   Stag kneels for [`TREE_STUN_TICKS`], the same stun as a bell (`poise.kneel`), so the
//!   player can riposte it. [`MechanicEvent::BRIAR_CHARGE_STARTED`] and
//!   [`MechanicEvent::BRIAR_TREE_STRUCK`] tell the renderer.
//! - **Bramble patches slow the player** ([`VFX_PATCHES`], phase 2's thorn burst). On the
//!   move frame that plays `bramble_patches`, [`PATCH_COUNT`] patches grow in a ring around the
//!   Stag. A player who walks or runs in one (not rolling, not attacking) moves at a reduced
//!   speed ([`before_locomotion`], tick system 5). Patches last [`PATCH_TICKS`], and all go
//!   when the fight ends or resets ("Reset: dying resets the boss fully, including adds and
//!   hazards"). They are in [`World::briar_patches`], so they are in the state hash.
//!   [`BriarEvent`] (WASM event kind 40) tells the renderer where they grow and fade.
//!
//! The spec fixes the stun (60 ticks) and that the patches slow. The rest of the numbers
//! below are tuning.

use serde::{Deserialize, Serialize};

use super::super::boss::Stage;
use crate::actor::Actor;
use crate::arena::Id;
use crate::combat::moves::root_motion_delta;
use crate::events::{BriarEvent, Event, MechanicEvent, StanceBreak};
use crate::math::dmath::{cos, sin};
use crate::world::World;

/// The boss's ID, in `data/bosses/b02_briar_stag.ron`.
pub const BOSS_ID: &str = "b02_briar_stag";
/// The vfx a charge move plays on the frame its run starts.
pub const VFX_CHARGE: &str = "stag_charge";
/// The vfx a move plays on the frame it leaves bramble patches.
pub const VFX_PATCHES: &str = "bramble_patches";

/// How long a charge that hits a tree stuns the Stag, in ticks (docs "B02 The Briar Stag":
/// "a charge that hits a tree stuns it for 60 ticks").
pub const TREE_STUN_TICKS: u8 = 60;
/// A move step shorter than this, in meters per tick, isn't a charge (a skid at the end of
/// one, or a walk). Tuning: a run is 0.15 m per tick, the charge 0.5.
const MIN_CHARGE_STEP_M: f32 = 0.25;
/// How far past the Stag's center the head reaches, in meters: the charge ends when
/// something solid is this close ahead. Tuning (the antlers and snout reach about this far).
const IMPACT_REACH_M: f32 = 1.5;
/// How far to each side of its center the Stag also looks, in meters, so a clump that only
/// meets a shoulder still stops it. Tuning.
const IMPACT_SIDE_M: f32 = 0.45;

/// Patches a thorn burst leaves, in a ring. Tuning: the spec says only "bramble patches".
const PATCH_COUNT: u16 = 6;
/// The ring's radius around the Stag, in meters. Tuning: inside the aura's 4 m.
const PATCH_RING_M: f32 = 2.6;
/// How far a patch slows, in meters. Tuning.
const PATCH_RADIUS_M: f32 = 1.6;
/// How long a patch lasts, in ticks (20 s at 30 ticks per second). Tuning.
const PATCH_TICKS: u16 = 600;
/// The walking and running speed inside a patch, in percent of normal. Tuning.
const PATCH_SPEED_PERCENT: u32 = 55;
/// The most patches on the ground; the oldest wither first. Tuning.
const MAX_PATCHES: usize = 12;
/// A patch reaches this far above and below its center, in meters. Tuning.
const PATCH_HEIGHT_M: f32 = 1.5;
/// A full turn, in radians.
const TAU: f32 = core::f32::consts::TAU;

/// A bramble patch on the ground. Simulation state: part of the state hash while any exists.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Patch {
    /// Unique among the patches on the ground (the renderer tracks them by it).
    pub id: u16,
    /// Where it lies, in meters.
    pub pos: [f32; 3],
    /// Ticks left before it withers.
    pub left: u16,
}

/// The body actor of boss `b`.
fn body(world: &World, b: usize) -> Option<Id> {
    let spawn = *world.layout.bosses.get(b)?.members.first()?;
    world.state.ai_enemies.get(usize::from(spawn)).copied()
}

fn emit(world: &mut World, b: usize, what: u8, id: u16, pos: [f32; 3], ticks: u16) {
    let radius = if what == BriarEvent::CLEARED {
        0.0
    } else {
        PATCH_RADIUS_M
    };
    world.events.push(Event::Briar(BriarEvent {
        what,
        boss: b as u16,
        index: id,
        pos,
        radius,
        ticks,
    }));
}

fn mechanic(world: &mut World, kind: u8, b: usize) {
    world.events.push(Event::Mechanic(MechanicEvent {
        kind,
        index: b as u16,
    }));
}

/// The Stag's rules for every boss with [`BOSS_ID`]; `super::run` calls it once a tick, in
/// every stage (it clears the patches when a fight resets).
pub fn run(world: &mut World) {
    for b in 0..world.layout.bosses.len() {
        if world.layout.bosses[b].def.id == BOSS_ID {
            tick(world, b);
        }
    }
}

/// The fight of boss `b` just reset (`ai::boss::reset`): the bramble patches go now, not on the
/// next tick, so a player who respawns can't step into one.
pub fn reset(world: &mut World, b: usize) {
    if world.layout.bosses[b].def.id == BOSS_ID {
        clear(world, b);
    }
}

/// One tick of the Stag's rules for boss `b`.
fn tick(world: &mut World, b: usize) {
    if world.bosses[b].stage != Stage::Fight {
        clear(world, b);
        return;
    }
    age(world, b);
    let Some(id) = body(world, b) else {
        return;
    };
    let Some(a) = world.actors.get(id) else {
        return;
    };
    if a.dead || a.hp == 0 {
        return;
    }
    let Some(c) = a.moves.current else {
        return;
    };
    let Some(d) = world.moves.get(c.num) else {
        return;
    };
    let frame_of = |name: &str| d.data.vfx.iter().find(|(_, n)| n == name).map(|&(f, _)| f);
    let charge_from = frame_of(VFX_CHARGE);
    let patches_at = frame_of(VFX_PATCHES);
    let step = root_motion_delta(&d.data.root_motion, c.frame);
    let (pos, yaw) = (a.pos, a.yaw);
    if let Some(from) = charge_from {
        if c.frame == from {
            mechanic(world, MechanicEvent::BRIAR_CHARGE_STARTED, b);
        }
        if c.frame >= from && step >= MIN_CHARGE_STEP_M && blocked(world, pos, yaw) {
            stun(world, b, id);
            return;
        }
    }
    if patches_at == Some(c.frame) {
        grow(world, b, pos, yaw);
    }
}

/// True if something solid is within [`IMPACT_REACH_M`] ahead of an actor at `pos` facing
/// `yaw`, at its center's height, on the line it runs along or `IMPACT_SIDE_M` to either side.
fn blocked(world: &World, pos: [f32; 3], yaw: f32) -> bool {
    let lift = world.locomotion.capsule_half_height + world.locomotion.capsule_radius;
    let dir = [sin(yaw), 0.0, cos(yaw)];
    // Perpendicular to the facing; both sides are tried, so which is which doesn't matter.
    let side = [cos(yaw), 0.0, -sin(yaw)];
    [-IMPACT_SIDE_M, 0.0, IMPACT_SIDE_M].iter().any(|&s| {
        let origin = [pos[0] + side[0] * s, pos[1] + lift, pos[2] + side[2] * s];
        world.physics.raycast(origin, dir, IMPACT_REACH_M).is_some()
    })
}

/// The charge ran into something: the move ends and the Stag kneels.
fn stun(world: &mut World, b: usize, id: Id) {
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    knock_out(a, TREE_STUN_TICKS);
    world.events.push(Event::StanceBreak(StanceBreak {
        target: id,
        ticks: TREE_STUN_TICKS,
    }));
    mechanic(world, MechanicEvent::BRIAR_TREE_STRUCK, b);
}

/// Ends the actor's move and locks it, kneeling, for `ticks` (as a bell does:
/// `world_state::mechanics::bell`).
fn knock_out(a: &mut Actor, ticks: u8) {
    a.moves.interrupt();
    a.moves.guard.drop_guard();
    a.moves.guard.counter = 0;
    a.moves.guard.stagger = a.moves.guard.stagger.max(ticks);
    a.poise.kneel = ticks;
    a.poise.accum = 0;
    a.poise.unaware = false;
    a.vel[0] = 0.0;
    a.vel[2] = 0.0;
    if let Some(brain) = a.ai.as_mut() {
        brain.doing = crate::ai::Doing::Nothing;
        brain.doing_ticks = 0;
    }
}

/// Grows [`PATCH_COUNT`] patches in a ring around `at`, the first one dead ahead.
fn grow(world: &mut World, b: usize, at: [f32; 3], yaw: f32) {
    for k in 0..PATCH_COUNT {
        let angle = yaw + TAU * f32::from(k) / f32::from(PATCH_COUNT);
        let pos = [
            at[0] + sin(angle) * PATCH_RING_M,
            at[1],
            at[2] + cos(angle) * PATCH_RING_M,
        ];
        // Unique among the live patches: the spawn tick (wrapped well past a patch's life)
        // and the slot in the ring.
        let id = ((world.tick % 4000) as u16) * 16 + k;
        world.briar_patches.push(Patch {
            id,
            pos,
            left: PATCH_TICKS,
        });
        emit(world, b, BriarEvent::SPAWNED, id, pos, PATCH_TICKS);
        if world.briar_patches.len() > MAX_PATCHES {
            let old = world.briar_patches.remove(0);
            emit(world, b, BriarEvent::FADED, old.id, old.pos, 0);
        }
    }
}

/// Counts every patch down and withers the ones that are out of time.
fn age(world: &mut World, b: usize) {
    if world.briar_patches.is_empty() {
        return;
    }
    let mut gone = Vec::new();
    world.briar_patches.retain_mut(|p| {
        p.left = p.left.saturating_sub(1);
        if p.left == 0 {
            gone.push(*p);
        }
        p.left > 0
    });
    for p in gone {
        emit(world, b, BriarEvent::FADED, p.id, p.pos, 0);
    }
}

/// Every patch goes at once: the fight is over or reset. [`reset`] calls it the moment a fight
/// resets, so no patch outlives the reset by a tick.
fn clear(world: &mut World, b: usize) {
    if world.briar_patches.is_empty() {
        return;
    }
    world.briar_patches.clear();
    emit(world, b, BriarEvent::CLEARED, 0, [0.0; 3], 0);
}

/// Slows a player who walks or runs in a bramble patch (tick system 5, before the controller
/// moves them). A roll, an attack, or any other move keeps its speed: the patch punishes
/// standing in the wrong place, not rolling through it.
pub fn before_locomotion(world: &mut World) {
    if world.briar_patches.is_empty() {
        return;
    }
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    if p.dead || !p.motor.grounded || p.moves.current.is_some() {
        return;
    }
    let (x, y, z) = (p.pos[0], p.pos[1], p.pos[2]);
    let inside = world.briar_patches.iter().any(|q| {
        let (dx, dz) = (q.pos[0] - x, q.pos[2] - z);
        dx * dx + dz * dz <= PATCH_RADIUS_M * PATCH_RADIUS_M
            && (q.pos[1] - y).abs() <= PATCH_HEIGHT_M
    });
    if inside && let Some(p) = world.actors.get_mut(world.player) {
        let k = PATCH_SPEED_PERCENT as f32 / 100.0;
        p.vel[0] *= k;
        p.vel[2] *= k;
    }
}
