//! Sight and hearing (docs/game/05-enemies-and-bosses.md, "Perception").
//!
//! - **Sight:** the target is inside the actor's cone (`sight_deg` wide, centered on its
//!   facing) and range (`sight_m`, eye to chest), and a ray from the actor's eye to the
//!   target's chest reaches it without hitting the level. Darkness (6 m without a player
//!   light) and equipment that scales the range aren't built yet; `dark_sight_m` is in
//!   `ai.ron` for when they are.
//! - **Hearing:** each tick the player's loudest action sets a noise radius (walk, run,
//!   sprint, roll, attack, landing). An actor within the radius times its
//!   `hearing_mult` hears it, through walls. The noise is kept until the actor's next
//!   think, so a 10 Hz thinker doesn't miss a one-tick sound.

use lf_content::enemy::{AiTuning, Perception};

use crate::actor::Actor;
use crate::actor::targeting::{bearing, wrap_pi};
use crate::arena::Id;
use crate::math::dmath::sqrt;
use crate::physics::PhysicsState;
use crate::world::World;

/// Degrees to radians. Plain multiplication, no transcendental call.
const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;
/// A sight ray that stops this close to the chest still counts as reaching it (the hit is
/// the ray's own end, not a wall). Not a gameplay number.
const RAY_SLACK_M: f32 = 1e-3;

/// Horizontal distance.
pub fn dist_xz(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
    sqrt(dx * dx + dz * dz)
}

/// The angle, in degrees, between `a`'s facing and the direction to `to`.
pub fn off_facing_deg(a: &Actor, to: [f32; 3]) -> f32 {
    wrap_pi(bearing(a.pos, to, a.yaw) - a.yaw).abs() / DEG_TO_RAD
}

/// True if `to` is inside `a`'s cone of `cone_deg` (full angle), ignoring range.
pub fn in_cone(a: &Actor, to: [f32; 3], cone_deg: f32) -> bool {
    cone_deg >= 360.0 || off_facing_deg(a, to) <= cone_deg * 0.5
}

/// True if `me` sees `target`: inside the cone and range, with a clear line from eye to
/// chest. Read-only.
pub fn sees(
    physics: &PhysicsState,
    me: &Actor,
    target: &Actor,
    p: &Perception,
    t: &AiTuning,
) -> bool {
    if target.dead || !in_cone(me, target.pos, p.sight_deg) {
        return false;
    }
    let eye = [me.pos[0], me.pos[1] + t.eye_height_m, me.pos[2]];
    let chest = [
        target.pos[0],
        target.pos[1] + t.chest_height_m,
        target.pos[2],
    ];
    let d = [chest[0] - eye[0], chest[1] - eye[1], chest[2] - eye[2]];
    let len = sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
    if len > p.sight_m {
        return false;
    }
    if len <= RAY_SLACK_M {
        return true;
    }
    let dir = [d[0] / len, d[1] / len, d[2] / len];
    physics
        .raycast(eye, dir, len)
        .is_none_or(|toi| toi >= len - RAY_SLACK_M)
}

/// The player's noise this tick: their position and the radius of their loudest action,
/// or `None` if they make none (standing still, dead, or drinking).
pub fn player_noise(world: &World, t: &AiTuning) -> Option<([f32; 3], f32)> {
    let p = world.actors.get(world.player).filter(|p| !p.dead)?;
    let n = &t.noise;
    let mut r: f32 = 0.0;
    if p.motor.landed {
        r = r.max(n.landing_m);
    }
    if let Some(d) = p.moves.current.and_then(|c| world.moves.get(c.num)) {
        if !d.data.hitboxes.is_empty() || !d.data.projectiles.is_empty() {
            r = r.max(n.attack_m);
        } else if d.data.iframes.is_some() {
            // A dodge: the only moves with i-frames and no hitbox.
            r = r.max(n.roll_m);
        }
    }
    if p.motor.sprinting {
        r = r.max(n.sprint_m);
    }
    let speed = sqrt(p.vel[0] * p.vel[0] + p.vel[2] * p.vel[2]);
    if p.moves.current.is_none() && speed > 0.0 {
        // The stick blends walk into run (docs/game/01-architecture.md, "ActionFrame");
        // the noise switches halfway.
        let l = &world.locomotion;
        let run = speed >= (l.walk_speed + l.run_speed) * 0.5;
        r = r.max(if run { n.run_m } else { n.walk_m });
    }
    (r > 0.0).then_some((p.pos, r))
}

/// Records the player's noise in `id`'s state if it's within earshot.
pub fn listen(world: &mut World, id: Id, noise: Option<([f32; 3], f32)>) {
    let Some((at, radius)) = noise else {
        return;
    };
    let Some(a) = world.actors.get(id) else {
        return;
    };
    let Some(def) = a.ai.as_ref().map(|b| usize::from(b.def)) else {
        return;
    };
    let mult = world.ai.defs[def].perception.hearing_mult;
    if dist_xz(a.pos, at) <= radius * mult
        && let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut())
    {
        b.heard = Some(at);
    }
}
