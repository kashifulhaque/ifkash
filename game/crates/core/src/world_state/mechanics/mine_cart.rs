//! Mine carts (WP-062; docs/game/02-world.md, "Mini-dungeons", D4 Cinderdeep Mine: "ride one
//! down, push one to break a wall").
//!
//! A cart waits at the first point of its rail. Two kinds:
//!
//! - `Ride`: `INTERACT` beside the cart puts the player on it. It rolls to the other end at
//!   its `speed_mps` and the player rides along, locked in place (no movement, attacks, or
//!   items: [`mask`]). At the far end they step off. The cart stays there; `INTERACT` beside
//!   it again rides it back, so a ride never strands anyone.
//! - `Push(wall)`: `INTERACT` beside the cart at its start shoves it. It rolls to the end
//!   of its rail by itself and breaks the hollow wall named `wall`: the same flag a hit
//!   raises, `wall.<id>.revealed`, so the passage opens exactly as if the player had struck
//!   it. It stays at the end, and the flag `cart.<id>.spent` is raised.
//!
//! The cart's position is its distance `s` along the rail, in [`super::super::MechState`]
//! (`carts`). A cart isn't a collider: the player walks through a waiting one. The numbers
//! (speed, rail) are in the level's `entities.ron`.

use lf_content::scenario::{Cart, CartKind};
use serde::{Deserialize, Serialize};

use super::{mechanic, sync_colliders};
use crate::events::MechanicEvent;
use crate::input::TickInput;
use crate::math::dmath::{atan2, sqrt};
use crate::world::{DT, World};
use crate::world_state::flags;

/// A ride started (WASM event kind 14, Mechanic; index is the cart's).
pub const CART_STARTED: u8 = 40;
/// A cart reached the end of its trip.
pub const CART_ARRIVED: u8 = 41;
/// A pushed cart broke its wall.
pub const CART_BROKE_WALL: u8 = 42;

/// The prompt codes (`interact_prompt`): ride, push.
pub const PROMPT_RIDE: u32 = 30;
pub const PROMPT_PUSH: u32 = 31;

/// How much farther than the usual reach the player can be from the cart's middle and still
/// board it, in meters: half the greybox cart's 2 m length (`render/props`).
pub const CART_HALF_LENGTH_M: f32 = 1.0;

/// One cart's trip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CartRun {
    /// Distance along the rail from its first point, in meters.
    pub s: f32,
    /// The end it is going to or waiting at: 0 the first point, 1 the last.
    pub target: u8,
    /// The player is on it.
    pub riding: bool,
}

/// Flag name for "this pushed cart broke its wall".
pub fn spent_flag(cart_id: &str) -> String {
    format!("cart.{cart_id}.spent")
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2])
}

/// The rail's length, in meters.
pub fn rail_len(c: &Cart) -> f32 {
    c.rail.windows(2).map(|s| dist(s[0], s[1])).sum()
}

/// Where the cart is at distance `s` along its rail (clamped), and the way it faces
/// there as a yaw.
pub fn point_at(c: &Cart, s: f32) -> ([f32; 3], f32) {
    let mut left = s.max(0.0);
    for seg in c.rail.windows(2) {
        let len = dist(seg[0], seg[1]);
        if left <= len {
            let t = left / len;
            let p = [
                seg[0][0] + (seg[1][0] - seg[0][0]) * t,
                seg[0][1] + (seg[1][1] - seg[0][1]) * t,
                seg[0][2] + (seg[1][2] - seg[0][2]) * t,
            ];
            return (p, atan2(seg[1][0] - seg[0][0], seg[1][2] - seg[0][2]));
        }
        left -= len;
    }
    let n = c.rail.len();
    let last = c.rail[n - 1];
    let prev = c.rail[n - 2];
    (last, atan2(last[0] - prev[0], last[2] - prev[2]))
}

/// Whether any cart carries the player.
pub fn riding(world: &World) -> bool {
    world.mech.carts.iter().any(|c| c.riding)
}

/// Where the player's `INTERACT` reaches cart `i`, if they could use it now: its place on the
/// rail while it waits at an end. `None` while it rolls, once a pushed cart is spent, and for a
/// pushed cart that isn't at the start.
pub fn spot(world: &World, i: usize) -> Option<[f32; 3]> {
    let c = &world.layout.carts[i];
    let run = world.mech.carts.get(i)?;
    let len = rail_len(c);
    let waiting = (run.target == 0 && run.s == 0.0) || (run.target == 1 && run.s == len);
    if !waiting || run.riding {
        return None;
    }
    if matches!(c.kind, CartKind::Push(_))
        && (run.s != 0.0 || world.state.flags.is_set(&spent_flag(&c.id)))
    {
        return None;
    }
    Some(point_at(c, run.s).0)
}

/// The HUD prompt for cart `i`.
pub fn prompt(world: &World, i: usize) -> u32 {
    match world.layout.carts[i].kind {
        CartKind::Ride => PROMPT_RIDE,
        CartKind::Push(_) => PROMPT_PUSH,
    }
}

/// The player pressed `INTERACT` beside cart `i`: board it, or shove it.
pub fn interact(world: &mut World, i: usize) {
    let kind = world.layout.carts[i].kind.clone();
    let len = rail_len(&world.layout.carts[i]);
    let run = &mut world.mech.carts[i];
    run.target = if run.s >= len { 0 } else { 1 };
    run.riding = matches!(kind, CartKind::Ride);
    mechanic(world, CART_STARTED, i);
    if let Some(p) = world.actors.get_mut(world.player)
        && matches!(kind, CartKind::Ride)
    {
        // Hold the player on the cart from this tick: no jump, no fall.
        p.vel = [0.0; 3];
        p.motor.grounded = true;
    }
}

/// Moves every moving cart one tick. A cart that carries the player sets their position.
pub fn step(world: &mut World) {
    let dead = world
        .actors
        .get(world.player)
        .is_none_or(|p| p.dead || p.hp == 0);
    for i in 0..world.layout.carts.len() {
        if dead {
            world.mech.carts[i].riding = false;
        }
        let run = world.mech.carts[i];
        let c = world.layout.carts[i].clone();
        let len = rail_len(&c);
        let goal = if run.target == 0 { 0.0 } else { len };
        if run.s == goal {
            continue;
        }
        let step = c.speed_mps * DT;
        let next = if (goal - run.s).abs() <= step {
            goal
        } else if goal > run.s {
            run.s + step
        } else {
            run.s - step
        };
        world.mech.carts[i].s = next;
        let (pos, rail_yaw) = point_at(&c, next);
        if run.riding
            && let Some(p) = world.actors.get_mut(world.player)
        {
            p.pos = pos;
            p.vel = [0.0; 3];
            // Facing the way it rolls (toward the first point when it rolls back).
            p.yaw = if run.target == 0 {
                let back = rail_yaw + core::f32::consts::PI;
                if back > core::f32::consts::PI {
                    back - core::f32::consts::TAU
                } else {
                    back
                }
            } else {
                rail_yaw
            };
            let m = &mut p.motor;
            m.grounded = true;
            m.jumped = false;
            m.jump_startup = 0;
            m.landing = 0;
            m.ground_y = pos[1];
            m.fall_peak_y = pos[1];
        }
        if next == goal {
            world.mech.carts[i].riding = false;
            mechanic(world, CART_ARRIVED, i);
            if let CartKind::Push(wall) = &c.kind
                && let Some(w) = world.layout.hollow_walls.iter().position(|w| &w.id == wall)
            {
                world.state.flags.raise(&flags::wall_revealed(wall));
                world.state.flags.raise(&spent_flag(&c.id));
                mechanic(world, MechanicEvent::WALL_REVEALED, w);
                mechanic(world, CART_BROKE_WALL, i);
                sync_colliders(world);
            }
        }
    }
}

/// While the player rides, the systems after the mechanics see no stick, no buttons, and no
/// commands. `None` means use the input as it is.
pub fn mask(world: &World, input: &TickInput) -> Option<TickInput> {
    if !riding(world) {
        return None;
    }
    let mut masked = input.clone();
    masked.frame.move_x = 0;
    masked.frame.move_z = 0;
    masked.frame.buttons = 0;
    masked.commands.clear();
    Some(masked)
}

/// The renderer's words: each cart's distance along its rail, in meters.
pub fn render_words(world: &World) -> impl Iterator<Item = f32> + '_ {
    world.mech.carts.iter().map(|r| r.s)
}
