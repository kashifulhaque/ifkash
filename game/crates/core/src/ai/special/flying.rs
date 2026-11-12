//! Flight (docs/game/05-enemies-and-bosses.md, "Enemy roster", Cave Moth): an enemy with
//! `flying` skips gravity and the character controller. Its feet follow the ground below it
//! at `hover_m`, and a dive move brings it down and back up by the move's frame. It moves
//! straight (the steering already probes for walls) and drifts toward lit Lanterns when
//! it has nothing else to do.
//!
//! [`step`] replaces the controller's step for a flyer in tick system 5 (locomotion).

use lf_content::enemy::Flying;

use crate::ai::perception::dist_xz;
use crate::arena::Id;
use crate::physics::PhysicsState;
use crate::world::{DT, World};
use crate::world_state::flags::lantern_lit;

/// How far above the flyer's feet the downward ground probe starts, in meters. Keeps the
/// probe out of a floor the flyer has dipped into; not a gameplay number.
const PROBE_UP_M: f32 = 0.5;
/// How far down the ground probe reaches, in meters. Beyond it the flyer keeps the last
/// ground height it knew.
const PROBE_DOWN_M: f32 = 40.0;

/// The height of the ground below (x, y, z), if any is within reach.
fn ground_below(physics: &PhysicsState, pos: [f32; 3]) -> Option<f32> {
    let origin = [pos[0], pos[1] + PROBE_UP_M, pos[2]];
    physics
        .raycast(origin, [0.0, -1.0, 0.0], PROBE_DOWN_M + PROBE_UP_M)
        .map(|toi| origin[1] - toi)
}

/// The height above ground a flyer in a dive should be at, `frame` frames into the move.
fn dive_height(f: &Flying, d: &lf_content::enemy::Dive, frame: u16) -> f32 {
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    if frame < d.down_until {
        lerp(
            f.hover_m,
            d.low_m,
            f32::from(frame) / f32::from(d.down_until.max(1)),
        )
    } else if frame < d.up_from {
        d.low_m
    } else if frame < d.up_until {
        lerp(
            d.low_m,
            f.hover_m,
            f32::from(frame - d.up_from) / f32::from((d.up_until - d.up_from).max(1)),
        )
    } else {
        f.hover_m
    }
}

/// True if `id` is a flyer.
pub fn flies(world: &World, id: Id) -> bool {
    world
        .actors
        .get(id)
        .and_then(|a| a.ai.as_ref())
        .is_some_and(|b| world.ai.def(b.def).data.flying.is_some())
}

/// One tick of a flyer's motion. Does nothing for an actor that doesn't fly.
pub fn step(world: &mut World, id: Id) {
    let Some(a) = world.actors.get(id) else {
        return;
    };
    let Some(b) = a.ai.as_ref() else {
        return;
    };
    let Some(f) = world.ai.def(b.def).data.flying.as_ref() else {
        return;
    };
    // The dive the actor is in, as (its height now), decided before the actor is borrowed
    // mutably.
    let dive = a.moves.current.and_then(|c| {
        f.dives
            .iter()
            .find(|d| world.moves.num_of(&d.move_id) == Some(c.num))
            .map(|d| dive_height(f, d, c.frame))
    });
    let (hover, climb) = (f.hover_m, f.climb_mps);
    let ground = ground_below(&world.physics, a.pos).unwrap_or(a.motor.ground_y);
    let kill_y = world.physics.kill_y;
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    a.motor.landed = false;
    if a.dead {
        return;
    }
    a.pos[0] += a.vel[0] * DT;
    a.pos[2] += a.vel[2] * DT;
    let want = ground + dive.unwrap_or(hover);
    a.pos[1] = if dive.is_some() {
        want
    } else {
        let step = climb * DT;
        a.pos[1] + (want - a.pos[1]).clamp(-step, step)
    };
    // A flyer is always "grounded" as far as moves and the AI are concerned.
    a.vel[1] = 0.0;
    a.motor.grounded = true;
    a.motor.ground_y = ground;
    a.motor.fall_peak_y = ground;
    if a.pos[1] < kill_y {
        a.hp = 0;
        a.dead = true;
    }
}

/// Where an Idle flyer with no patrol drifts: the nearest lit Lantern within `light_m` of it.
pub fn light_goal(world: &World, id: Id) -> Option<[f32; 3]> {
    let a = world.actors.get(id)?;
    let b = a.ai.as_ref()?;
    let f = world.ai.def(b.def).data.flying.as_ref()?;
    if f.light_m <= 0.0 {
        return None;
    }
    let mut best: Option<(f32, [f32; 3])> = None;
    for l in &world.layout.lanterns {
        if !world.state.flags.is_set(&lantern_lit(&l.id)) {
            continue;
        }
        let d = dist_xz(a.pos, l.pos);
        if d <= f.light_m && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, l.pos));
        }
    }
    best.map(|(_, p)| [p[0], a.pos[1], p[2]])
}
