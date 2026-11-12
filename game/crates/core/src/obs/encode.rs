//! The `proprio`, `target`, and `move` encoders.

use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::arena::Id;
use crate::combat::moves::{ActiveMove, MoveDef, MoveTable, flags};
use crate::combat::status::COUNT as STATUS_COUNT;
use crate::math::dmath::{cos, sin, sqrt};
use crate::world::World;

use super::{
    HIT_SCALE_TICKS, LEFT_SCALE_TICKS, POS_SCALE_M, PROPRIO_LEN, TARGET_LEN, TIMER_SCALE_TICKS,
    VEL_SCALE,
};

fn flag(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

fn frac(num: f32, den: f32) -> f32 {
    if den > 0.0 {
        (num / den).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn clip(v: f32) -> f32 {
    v.clamp(-1.0, 1.0)
}

/// The world-space vector `(dx, dz)` in `a`'s frame: `(right, forward)`. Yaw grows from
/// +Z toward +X, and +X is to the left of an actor facing +Z (see `actor::targeting`).
fn to_local(yaw: f32, dx: f32, dz: f32) -> (f32, f32) {
    let (s, c) = (sin(yaw), cos(yaw));
    (-dx * c + dz * s, dx * s + dz * c)
}

/// What the current move says about timing: `(frame fraction, hit active now, ticks to
/// the next hit frame, ticks left, ends the combo)`. With no next hit, the ticks to it
/// read as the clip value.
struct MoveTiming {
    frac: f32,
    hit_now: bool,
    to_hit: f32,
    left: f32,
    finisher: bool,
    attack: bool,
}

fn timing(m: Option<&ActiveMove>, moves: &MoveTable) -> MoveTiming {
    let none = MoveTiming {
        frac: 0.0,
        hit_now: false,
        to_hit: 1.0,
        left: 0.0,
        finisher: false,
        attack: false,
    };
    let Some(m) = m else { return none };
    let Some(def): Option<&MoveDef> = moves.get(m.num) else {
        return none;
    };
    let total = f32::from(def.data.total_frames.max(1));
    let hits = &def.data.hitboxes;
    let hit_now = hits
        .iter()
        .any(|h| h.frames.0 <= m.frame && m.frame <= h.frames.1);
    let to_hit = hits
        .iter()
        .filter(|h| h.frames.0 > m.frame)
        .map(|h| h.frames.0 - m.frame)
        .min()
        .map_or(1.0, |t| frac(f32::from(t), HIT_SCALE_TICKS));
    MoveTiming {
        frac: frac(f32::from(m.frame), total),
        hit_now,
        to_hit,
        left: frac(
            f32::from(def.data.total_frames.saturating_sub(m.frame)),
            LEFT_SCALE_TICKS,
        ),
        finisher: !hits.is_empty() && def.next_chain.is_none(),
        attack: !hits.is_empty(),
    }
}

/// The body of boss `b` of the scenario (its first member), if it exists.
pub fn boss_body(world: &World, b: usize) -> Option<Id> {
    let spawn = *world.layout.bosses.get(b)?.members.first()?;
    world.state.ai_enemies.get(usize::from(spawn)).copied()
}

fn alive(a: &Actor) -> bool {
    !a.dead && a.hp > 0
}

/// The actor the `target` vector describes: the player's lock target, or else the
/// nearest living non-player actor (ties go to the lower arena index).
pub fn target_id(world: &World) -> Option<Id> {
    let p = world.actors.get(world.player)?;
    if let Some(t) = p.lock_target
        && world.actors.get(t).is_some_and(alive)
    {
        return Some(t);
    }
    let mut best: Option<(f32, Id)> = None;
    for (id, a) in world.actors.iter() {
        if a.is_player || a.is_ally() || !alive(a) {
            continue;
        }
        let (dx, dz) = (a.pos[0] - p.pos[0], a.pos[2] - p.pos[2]);
        let d = dx * dx + dz * dz;
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, id));
        }
    }
    best.map(|(_, id)| id)
}

/// The `proprio` vector: the player's own state. Layout (index: feature):
///
/// - 0-2: HP, FP, and stamina fractions.
/// - 3: equip load tier (0 until equip load lands).
/// - 4-6: a move is playing, its frame fraction, and its ticks left (scaled).
/// - 7-9: i-frame, hyperarmor, and charging flags of the frame just executed.
/// - 10-13: guard raised, guard being raised, guard-break stagger, guard counter window.
/// - 14-15: Vigil and Lucid flask charges, as fractions of the total charges.
/// - 16-21: status buildups (fraction of each meter), in `combat::status::KINDS` order.
/// - 22-27: status effect timers, in the same order.
/// - 28-29: weapon and body buff timers.
/// - 30-32: velocity in the player's frame (right, up, forward).
/// - 33-35: grounded, locked on, sprinting.
/// - 36-38: kneeling (stance broken), parried, poise left (fraction).
/// - 39-41: drinking (slow walk), the move locks movement, an action is buffered.
/// - 42-43: the move is an attack, and its hitbox is active now.
/// - 44: dead.
/// - 45-47: zero.
pub fn proprio(world: &World) -> [f32; PROPRIO_LEN] {
    let mut o = [0.0; PROPRIO_LEN];
    let Some(p) = world.actors.get(world.player) else {
        return o;
    };
    o[0] = frac(p.hp as f32, p.max_hp as f32);
    o[1] = frac(p.fp, p.max_fp);
    o[2] = frac(p.stamina, p.max_stamina);
    let t = timing(p.moves.current.as_ref(), &world.moves);
    o[4] = flag(p.moves.current.is_some());
    o[5] = t.frac;
    o[6] = t.left;
    o[7] = flag(p.moves.flags & flags::IFRAMES != 0);
    o[8] = flag(p.moves.flags & flags::HYPERARMOR != 0);
    o[9] = flag(p.moves.flags & flags::CHARGING != 0);
    let g = &p.moves.guard;
    o[10] = flag(g.active);
    o[11] = flag(g.held > 0 && !g.active);
    o[12] = flag(g.stagger > 0);
    o[13] = flag(g.counter > 0);
    let f = &p.inventory.flasks;
    o[14] = frac(f32::from(f.vigil), f32::from(f.max_charges));
    o[15] = frac(f32::from(f.lucid), f32::from(f.max_charges));
    let s = &p.status;
    for i in 0..STATUS_COUNT {
        o[16 + i] = frac(s.meter_milli[i] as f32, f32::from(s.resist[i]) * 1000.0);
        o[22 + i] = frac(f32::from(s.active[i]), TIMER_SCALE_TICKS);
    }
    o[28] = s
        .weapon_buff
        .map_or(0.0, |b| frac(b.ticks as f32, TIMER_SCALE_TICKS));
    o[29] = s
        .body_buff
        .map_or(0.0, |b| frac(b.ticks as f32, TIMER_SCALE_TICKS));
    let (vr, vf) = to_local(p.yaw, p.vel[0], p.vel[2]);
    o[30] = clip(vr / VEL_SCALE);
    o[31] = clip(p.vel[1] / VEL_SCALE);
    o[32] = clip(vf / VEL_SCALE);
    o[33] = flag(p.motor.grounded);
    o[34] = flag(p.lock_target.is_some());
    o[35] = flag(p.motor.sprinting);
    o[36] = flag(p.poise.kneel > 0);
    o[37] = flag(p.poise.parried > 0);
    o[38] = if p.poise.poise > 0 {
        1.0 - frac(p.poise.accum as f32, f32::from(p.poise.poise))
    } else {
        0.0
    };
    o[39] = flag(p.moves.slow_walk_percent > 0);
    o[40] = flag(p.moves.locks_movement());
    o[41] = flag(p.moves.buffer.is_some());
    o[42] = flag(t.attack);
    o[43] = flag(t.hit_now);
    o[44] = flag(!alive(p));
    o
}

/// The `target` vector: the lock target, or the nearest living enemy (see
/// [`target_id`]). All zero with no target. Layout (index: feature):
///
/// - 0-1: a target exists, and it is the lock target.
/// - 2-4: relative position in the player's frame (right, up, forward).
/// - 5: planar distance.
/// - 6-7: sine and cosine of the bearing to the target, from the player's facing
///   (positive sine: to the left).
/// - 8-9: relative planar velocity in the player's frame (right, forward).
/// - 10-11: sine and cosine of the target's facing, relative to the player's.
/// - 12-13: cosine and sine of the angle between the target's facing and its bearing to
///   the player (cosine 1: facing the player).
/// - 14-15: HP fraction, stance left (fraction).
/// - 16-17: a move is playing, its frame fraction.
/// - 18-20: its hitbox is active now, ticks to its next hit frame (scaled; 1 with no
///   hit to come), ticks left (scaled).
/// - 21: the move is the last attack of its combo.
/// - 22-26: i-frame, hyperarmor, kneeling, parried, guard-break stagger.
/// - 27-29: boss phase (phase / 4; 0 outside a boss fight), phase transition playing,
///   the fight is on.
/// - 30: fraction of status effects active.
/// - 31: zero.
pub fn target(world: &World) -> [f32; TARGET_LEN] {
    let mut o = [0.0; TARGET_LEN];
    let (Some(p), Some(tid)) = (world.actors.get(world.player), target_id(world)) else {
        return o;
    };
    let Some(t) = world.actors.get(tid) else {
        return o;
    };
    o[0] = 1.0;
    o[1] = flag(p.lock_target == Some(tid));
    let (dx, dy, dz) = (
        t.pos[0] - p.pos[0],
        t.pos[1] - p.pos[1],
        t.pos[2] - p.pos[2],
    );
    let (r, f) = to_local(p.yaw, dx, dz);
    o[2] = clip(r / POS_SCALE_M);
    o[3] = clip(dy / POS_SCALE_M);
    o[4] = clip(f / POS_SCALE_M);
    let dist = sqrt(dx * dx + dz * dz);
    o[5] = frac(dist, POS_SCALE_M);
    if dist > 1.0e-4 {
        // In the player's frame, left is -right: sine positive to the left.
        o[6] = clip(-r / dist);
        o[7] = clip(f / dist);
    } else {
        o[7] = 1.0;
    }
    let (vr, vf) = to_local(p.yaw, t.vel[0] - p.vel[0], t.vel[2] - p.vel[2]);
    o[8] = clip(vr / VEL_SCALE);
    o[9] = clip(vf / VEL_SCALE);
    let rel_yaw = t.yaw - p.yaw;
    o[10] = sin(rel_yaw);
    o[11] = cos(rel_yaw);
    if dist > 1.0e-4 {
        // The player's position in the target's frame.
        let (tr, tf) = to_local(t.yaw, -dx, -dz);
        o[12] = clip(tf / dist);
        o[13] = clip(-tr / dist);
    } else {
        o[12] = 1.0;
    }
    o[14] = frac(t.hp as f32, t.max_hp as f32);
    o[15] = if t.poise.stance > 0 {
        1.0 - frac(t.poise.stance_dmg as f32, f32::from(t.poise.stance))
    } else {
        0.0
    };
    let m = timing(t.moves.current.as_ref(), &world.moves);
    o[16] = flag(t.moves.current.is_some());
    o[17] = m.frac;
    o[18] = flag(m.hit_now);
    o[19] = m.to_hit;
    o[20] = m.left;
    o[21] = flag(m.finisher);
    o[22] = flag(t.moves.flags & flags::IFRAMES != 0);
    o[23] = flag(t.moves.flags & flags::HYPERARMOR != 0);
    o[24] = flag(t.poise.kneel > 0);
    o[25] = flag(t.poise.parried > 0);
    o[26] = flag(t.moves.guard.stagger > 0);
    let boss = (0..world.bosses.len()).find(|&b| boss_body(world, b) == Some(tid));
    if let Some(run) = boss.and_then(|b| world.bosses.get(b)) {
        o[27] = frac(f32::from(run.phase), 4.0);
        o[28] = flag(run.transition_left > 0);
        o[29] = flag(run.stage == Stage::Fight);
    }
    let active = t.status.active.iter().filter(|&&a| a > 0).count();
    o[30] = active as f32 / STATUS_COUNT as f32;
    o
}

/// The `move` key: the registry numbers of the player's current move and the target's
/// (0 for none), for embedding tables.
pub fn move_ids(world: &World) -> [i32; 2] {
    let num = |id: Option<Id>| {
        id.and_then(|id| world.actors.get(id))
            .and_then(|a| a.moves.current)
            .map_or(0, |m| m.num as i32)
    };
    [num(Some(world.player)), num(target_id(world))]
}
