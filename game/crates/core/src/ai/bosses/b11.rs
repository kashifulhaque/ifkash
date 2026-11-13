//! B11, the Bound Exile (WP-079). Source: docs/game/05-enemies-and-bosses.md, "B11 The Bound
//! Exile", and `data/bosses/b11_bound_exile.ron`, where every number below lives (`params`).
//! The Exile is an ordinary AI actor (the body), and its image is a part, so the framework plays
//! their moves. This file adds what the framework lacks:
//!
//! 1. **The glyph ring.** While `b11_beam` is in frames `glyph_from` to `glyph_to` (the ring is
//!    lit and the beam hasn't landed), the Exile's poise is primed so that any hit that deals
//!    poise damage staggers it, which ends the move: "hitting the Exile during the glyph ring
//!    interrupts and staggers". Outside the window it keeps its own poise.
//! 2. **The blink.** On `teleport_frame` of `b11_blink` and `b11_image_blink` the actor
//!    reappears in front of one of the ring's twelve standing stones (`stone_count` of them, `ring_radius_m` from
//!    the ring's center, which is `ring_dx_m`, `ring_dz_m` from the Exile's spawn): not the one
//!    it stands nearest to, not within `blink_min_player_m` of the player, not within
//!    `blink_gap_m` of the other Exile. It picks at random among those (the world's AI stream),
//!    and turns to face the player. `b11_shuffle` is the same frame, but the Exile and its image
//!    swap places instead.
//! 3. **The images.** The image is a part that stays hidden and down (`dead`, and the AI's
//!    `hidden` flag, which the renderer reads as "draw nothing") until phase 2 is about to
//!    start: `split_ticks_left` ticks before the end of `b11_split` it is rebuilt at the stone
//!    farthest from the Exile and joins the fight. Its HP is topped up every tick, so nothing
//!    anyone does to it counts (only the real Exile casts, and only it can die). When the fight
//!    is not on, or is over, it is hidden again.
//! 4. **The seal.** The arena's sigil (`sigil_ring`, see `data/levels/bound_ring/entities.ron`)
//!    shuts the seal door behind the player. The Exile's defeat raises `sigil.sigil_ring.released`
//!    (the framework raises `boss.b11_bound_exile.defeated`), which opens it again. If the
//!    player isn't in the Ring while the fight isn't on (they died, and the Remnant sent them
//!    back to a Lantern, or they reloaded the save), the seal is lifted so the door doesn't lock
//!    them out for good: `sigil.sigil_ring.sealed` is cleared and the sigil can be touched again.
//!    ("In the Ring" is within `seal_inside_m` of the ring's center.)
//!
//! State: none of its own; everything is read from the actors' moves and HP, the boss's stage,
//! and flags, so it is part of the world hash through them.

use crate::ai::boss::Stage;
use crate::arena::Id;
use crate::events::{Event, MechanicEvent};
use crate::math::dmath::{atan2, cos, sin, sqrt};
use crate::world::World;
use crate::world_state::mechanics::brazier;

/// The boss definition this mechanic belongs to.
pub const BOSS_ID: &str = "b11_bound_exile";
/// Member numbers: the order in the boss definition (the body, then the part).
const EXILE: usize = 0;
const IMAGE: usize = 1;
/// The Bound Ring's seal sigil, in `data/levels/bound_ring/entities.ron`.
const SIGIL_ID: &str = "sigil_ring";
/// The moves the mechanic watches, by registry ID.
const BEAM_MOVE: &str = "b11_beam";
const SHUFFLE_MOVE: &str = "b11_shuffle";
const BLINK_MOVES: [&str; 3] = ["b11_blink", SHUFFLE_MOVE, "b11_image_blink"];
/// Degrees in a half turn, for the stones' bearings.
const HALF_TURN_DEG: f32 = 180.0;

/// One tick of the mechanic, for boss `b` while it fights. Does nothing for another boss.
pub fn tick(world: &mut World, b: usize) {
    if !is_b11(world, b) {
        return;
    }
    glyph(world, b);
    blink(world, b);
}

/// Once a tick for the whole world: the image's visibility and the arena's seal.
pub fn run(world: &mut World) {
    for b in 0..world.layout.bosses.len() {
        if is_b11(world, b) && world.bosses.len() > b {
            image(world, b);
            seal(world, b);
        }
    }
}

fn is_b11(world: &World, b: usize) -> bool {
    world
        .layout
        .bosses
        .get(b)
        .is_some_and(|lay| lay.def.id == BOSS_ID)
}

/// The actor of member `i` of boss `b`.
fn member(world: &World, b: usize, i: usize) -> Option<Id> {
    let spawn = *world.layout.bosses.get(b)?.members.get(i)?;
    world.state.ai_enemies.get(usize::from(spawn)).copied()
}

fn down(world: &World, id: Id) -> bool {
    world.actors.get(id).is_none_or(|a| a.dead || a.hp == 0)
}

fn number(world: &World, b: usize, key: &str) -> f32 {
    world.layout.bosses[b]
        .def
        .params
        .get(key)
        .copied()
        .unwrap_or(0.0)
}

fn planar(a: [f32; 3], b: [f32; 2]) -> f32 {
    let (dx, dz) = (b[0] - a[0], b[1] - a[2]);
    sqrt(dx * dx + dz * dz)
}

fn mechanic_event(world: &mut World, kind: u8, member: usize, stone: usize) {
    world.events.push(Event::Mechanic(MechanicEvent {
        kind,
        index: ((member as u16) << 8) | stone as u16,
    }));
}

/// The boss's spawn, where the stones' positions are measured from.
fn spawn_pos(world: &World, b: usize) -> [f32; 3] {
    let spawn = world.layout.bosses[b].members[EXILE];
    world.layout.enemies[usize::from(spawn)].pos
}

/// The height of the floor the stones stand on: the Exile spawns on the prisoner's plinth, which is
/// `-ring_dy_m` above it.
fn floor_y(world: &World, b: usize) -> f32 {
    spawn_pos(world, b)[1] + number(world, b, "ring_dy_m")
}

/// The blink stones' positions on the floor (x, z), in order.
fn stones(world: &World, b: usize) -> Vec<[f32; 2]> {
    let at = spawn_pos(world, b);
    let center = [
        at[0] + number(world, b, "ring_dx_m"),
        at[2] + number(world, b, "ring_dz_m"),
    ];
    let (count, radius) = (
        number(world, b, "stone_count") as usize,
        number(world, b, "ring_radius_m"),
    );
    let first = number(world, b, "stone_first_deg");
    let turn = std::f32::consts::PI / HALF_TURN_DEG;
    (0..count)
        .map(|k| {
            let deg = first + 2.0 * HALF_TURN_DEG * k as f32 / count as f32;
            let a = deg * turn;
            [center[0] + radius * sin(a), center[1] + radius * cos(a)]
        })
        .collect()
}

/// The stone nearest to `pos`.
fn nearest(stones: &[[f32; 2]], pos: [f32; 3]) -> Option<usize> {
    (0..stones.len()).min_by(|&i, &j| {
        planar(pos, stones[i])
            .partial_cmp(&planar(pos, stones[j]))
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// The stone an actor at `from` blinks to: one it isn't standing at, far enough from the player
/// and from `other` (if there is one); random among those, or the farthest from the player when
/// the ring has no such stone.
fn pick_stone(world: &mut World, b: usize, from: [f32; 3], other: Option<[f32; 3]>) -> usize {
    let stones = stones(world, b);
    let player = world.actors.get(world.player).map_or([0.0; 3], |p| p.pos);
    let (min_player, gap) = (
        number(world, b, "blink_min_player_m"),
        number(world, b, "blink_gap_m"),
    );
    let here = nearest(&stones, from);
    let ok: Vec<usize> = (0..stones.len())
        .filter(|&i| Some(i) != here)
        .filter(|&i| planar(player, stones[i]) >= min_player)
        .filter(|&i| other.is_none_or(|o| planar(o, stones[i]) >= gap))
        .collect();
    if ok.is_empty() {
        return (0..stones.len())
            .max_by(|&i, &j| {
                planar(player, stones[i])
                    .partial_cmp(&planar(player, stones[j]))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(0);
    }
    ok[(world.rng.ai.next_u32() as usize) % ok.len()]
}

/// Puts actor `id` at (x, z) on the floor, facing the player.
fn place(world: &mut World, id: Id, at: [f32; 2], floor_y: f32) {
    let player = world.actors.get(world.player).map_or([0.0; 3], |p| p.pos);
    if let Some(a) = world.actors.get_mut(id) {
        a.pos = [at[0], floor_y, at[1]];
        a.vel = [0.0; 3];
        a.yaw = atan2(player[0] - at[0], player[2] - at[1]);
    }
}

/// While the glyph ring is lit, any hit that does poise damage staggers the Exile.
fn glyph(world: &mut World, b: usize) {
    let Some(id) = member(world, b, EXILE) else {
        return;
    };
    let Some(num) = world.moves.num_of(BEAM_MOVE) else {
        return;
    };
    let (from, to) = (number(world, b, "glyph_from"), number(world, b, "glyph_to"));
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    let Some(c) = a.moves.current else {
        return;
    };
    let frame = f32::from(c.frame);
    if a.dead || c.num != num || frame < from || frame > to {
        return;
    }
    // A hit adds to `accum` and staggers once it is above the poise: start at the poise.
    a.poise.accum = u32::from(a.poise.poise);
    a.poise.since_hit = 0;
}

/// The blink moves reappear their actor at a stone (or swap the two Exiles).
fn blink(world: &mut World, b: usize) {
    let frame = number(world, b, "teleport_frame");
    let floor_y = floor_y(world, b);
    for i in [EXILE, IMAGE] {
        let Some(id) = member(world, b, i) else {
            continue;
        };
        let Some(a) = world.actors.get(id) else {
            continue;
        };
        let Some(c) = a.moves.current else {
            continue;
        };
        if a.dead || f32::from(c.frame) != frame {
            continue;
        }
        let Some(name) = BLINK_MOVES
            .into_iter()
            .find(|n| world.moves.num_of(n) == Some(c.num))
        else {
            continue;
        };
        let from = a.pos;
        // The other Exile, if it is up.
        let other_id = member(world, b, EXILE + IMAGE - i).filter(|&o| !down(world, o));
        let other = other_id.and_then(|o| world.actors.get(o)).map(|o| o.pos);
        if name == SHUFFLE_MOVE
            && let (Some(o), Some(op)) = (other_id, other)
        {
            // Swap: each stands where the other was, facing the player.
            place(world, id, [op[0], op[2]], floor_y);
            place(world, o, [from[0], from[2]], floor_y);
            let stone = nearest(&stones(world, b), op).unwrap_or(0);
            mechanic_event(world, MechanicEvent::EXILE_SHUFFLE, i, stone);
            continue;
        }
        let stone = pick_stone(world, b, from, other);
        let at = stones(world, b)[stone];
        place(world, id, at, floor_y);
        mechanic_event(world, MechanicEvent::EXILE_BLINK, i, stone);
    }
}

/// Keeps the image hidden until the split, then in the fight, then hidden again.
fn image(world: &mut World, b: usize) {
    let Some(id) = member(world, b, IMAGE) else {
        return;
    };
    let run = world.bosses[b];
    let left = number(world, b, "split_ticks_left") as u16;
    let on = run.stage == Stage::Fight
        && (run.phase >= 2
            || (run.transition_to == 2 && run.transition_left > 0 && run.transition_left <= left));
    if !on {
        if let Some(a) = world.actors.get_mut(id)
            && !(a.dead && a.hp == 0 && a.ai.as_ref().is_some_and(|brain| brain.hidden))
        {
            a.dead = true;
            a.hp = 0;
            a.moves.current = None;
            a.vel = [0.0; 3];
            if let Some(brain) = a.ai.as_mut() {
                brain.hidden = true;
            }
        }
        return;
    }
    if down(world, id) {
        split(world, b, id);
    } else if let Some(a) = world.actors.get_mut(id) {
        // Nothing done to the image counts.
        a.hp = a.max_hp;
    }
}

/// The image steps out of the Exile: rebuilt at the stone farthest from it.
fn split(world: &mut World, b: usize, id: Id) {
    let spawn = world.layout.bosses[b].members[IMAGE];
    let hurtbox = crate::combat::hurtbox::Hurtbox::from_locomotion(&world.locomotion);
    let Some(mut fresh) = crate::ai::enemy_actor(
        &world.layout.enemies[usize::from(spawn)],
        spawn,
        &world.ai,
        hurtbox,
    ) else {
        return;
    };
    let player = world.player;
    let player_at = world.actors.get(player).map_or([0.0; 3], |p| p.pos);
    fresh.poise.unaware = false;
    fresh.lock_target = Some(player);
    if let Some(brain) = fresh.ai.as_mut() {
        brain.target = Some(player);
        brain.last_seen = player_at;
        brain.phase = world.bosses[b].phase.max(1);
    }
    if let Some(a) = world.actors.get_mut(id) {
        *a = fresh;
    }
    // The stone farthest from the Exile, as far from the player as the blink rule asks.
    let exile = member(world, b, EXILE)
        .and_then(|e| world.actors.get(e))
        .map_or(player_at, |e| e.pos);
    let stones = stones(world, b);
    let min_player = number(world, b, "blink_min_player_m");
    let far = |i: &usize| planar(exile, stones[*i]);
    let stone = (0..stones.len())
        .filter(|&i| planar(player_at, stones[i]) >= min_player)
        .max_by(|i, j| {
            far(i)
                .partial_cmp(&far(j))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .or_else(|| nearest(&stones, exile))
        .unwrap_or(0);
    let floor_y = floor_y(world, b);
    place(world, id, stones[stone], floor_y);
    crate::ai::set_awareness(world, id, crate::ai::Awareness::Combat);
    mechanic_event(world, MechanicEvent::EXILE_SPLIT, IMAGE, stone);
}

/// The arena's sigil: released when the Exile falls, lifted when nobody is in the Ring.
fn seal(world: &mut World, b: usize) {
    if !world.layout.sigils.iter().any(|g| g.id == SIGIL_ID) {
        return;
    }
    let released = brazier::released_flag(SIGIL_ID);
    let sealed = brazier::sealed_flag(SIGIL_ID);
    if world
        .state
        .flags
        .is_set(&world.layout.bosses[b].defeated_flag())
    {
        if !world.state.flags.is_set(&released) {
            world.state.flags.raise(&released);
        }
        return;
    }
    // Only a seal that still holds can be lifted: once released, the touch stays on record.
    if world.bosses[b].stage != Stage::Dormant
        || !world.state.flags.is_set(&sealed)
        || world.state.flags.is_set(&released)
    {
        return;
    }
    let at = spawn_pos(world, b);
    let center = [
        at[0] + number(world, b, "ring_dx_m"),
        at[2] + number(world, b, "ring_dz_m"),
    ];
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    if planar(p.pos, center) > number(world, b, "seal_inside_m") {
        world.state.flags.clear(&sealed);
        world.state.flags.clear(&brazier::touched_flag(SIGIL_ID));
    }
}
