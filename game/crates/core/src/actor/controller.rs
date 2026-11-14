//! The player character controller: stick input to planar velocity, sprint, jump,
//! gravity, Rapier's `KinematicCharacterController`, and fall damage.
//!
//! Every number comes from `Locomotion` (`game/data/locomotion.ron`), which cites the
//! spec rows: docs/game/01-architecture.md "ActionFrame" and "Physics",
//! docs/game/02-world.md "Traversal and world mechanics" (jump, fall damage), and
//! docs/game/03-combat.md "Combat verbs" (sprint, jump) and "Regeneration".

use lf_content::Locomotion;
use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::prelude::{Capsule, Pose, Vector};

use super::Actor;
use crate::input::{ActionFrame, buttons};
use crate::math::dmath::{atan2, sqrt};
use crate::physics::PhysicsState;
use crate::world::DT;

/// Degrees to radians. Plain multiplication, no transcendental call.
const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;
/// Extra reach of the footing probe below the capsule's bottom, in meters. Covers the
/// gap between the capsule and a 45 degree slope under its center (about 0.15 m for the
/// player capsule); not a gameplay number.
const FOOTING_PROBE_M: f32 = 0.25;
/// Stick magnitudes below this (in 0..=127 units) count as no input: a stick at rest.
const STICK_DEAD_ZONE: f32 = 1.0;

/// Planar speed for a stick magnitude: walk below `walk_below`, run above `run_above`,
/// and a linear blend across the band between (docs/game/01-architecture.md, "ActionFrame").
pub fn stick_speed(mag: f32, t: &Locomotion) -> f32 {
    if mag > t.run_above {
        t.run_speed
    } else if mag < t.walk_below {
        t.walk_speed
    } else {
        let k = (mag - t.walk_below) / (t.run_above - t.walk_below);
        t.walk_speed + (t.run_speed - t.walk_speed) * k
    }
}

/// Initial upward speed that peaks at `jump_height`: `sqrt(2 g h)`.
pub fn jump_speed(t: &Locomotion) -> f32 {
    sqrt(2.0 * t.gravity * t.jump_height)
}

/// Player intent (tick system 2): turns the frame into a planar velocity, a facing,
/// sprint, and jump requests. `lock_pos` is the position of the locked-on target, if any:
/// while it's set the actor strafes, facing the target whichever way it moves
/// (docs/game/03-combat.md, "Lock-on"). The move direction is the stick's, unchanged.
/// `can_sprint` is false when overloaded (walk only; "Equip load and roll tiers").
pub fn intent(
    a: &mut Actor,
    frame: ActionFrame,
    lock_pos: Option<[f32; 3]>,
    can_sprint: bool,
    t: &Locomotion,
) {
    let m = &mut a.motor;
    let pressed = frame.buttons & !m.prev_buttons;
    m.prev_buttons = frame.buttons;
    m.dodge_held = if frame.pressed(buttons::DODGE) {
        m.dodge_held.saturating_add(1)
    } else {
        0
    };
    m.sprinting = false;
    if a.dead {
        a.vel[0] = 0.0;
        a.vel[2] = 0.0;
        return;
    }
    // A move owns the velocity and facing until its `cancel_any_from` frame
    // (combat::moves sets them from root motion and tracking).
    if a.moves.locks_movement() {
        a.vel[0] = 0.0;
        a.vel[2] = 0.0;
        return;
    }
    if let Some(p) = lock_pos {
        a.yaw = super::targeting::bearing(a.pos, p, a.yaw);
    }
    // In the air the takeoff velocity is kept: no air control.
    if !m.grounded && m.jump_startup == 0 {
        return;
    }
    if m.landing > 0 {
        m.landing -= 1;
        a.vel[0] = 0.0;
        a.vel[2] = 0.0;
        return;
    }

    let x = f32::from(frame.move_x);
    let z = f32::from(frame.move_z);
    let mag = sqrt(x * x + z * z);
    let mut spent = false;
    if mag < STICK_DEAD_ZONE {
        a.vel[0] = 0.0;
        a.vel[2] = 0.0;
    } else {
        let mut speed = stick_speed(mag, t);
        // Sprint starts once DODGE has been held for more than `sprint_delay_ticks`.
        if can_sprint
            && a.moves.slow_walk_percent == 0
            && m.dodge_held > t.sprint_delay_ticks
            && a.stamina > 0.0
            && m.jump_startup == 0
        {
            m.sprinting = true;
            speed = t.sprint_speed;
            a.stamina = (a.stamina - t.sprint_stamina_per_tick).max(0.0);
            spent = true;
        }
        a.vel[0] = x / mag * speed;
        a.vel[2] = z / mag * speed;
        if lock_pos.is_none() {
            a.yaw = atan2(x, z);
        }
    }

    if pressed & buttons::JUMP != 0
        && a.moves.slow_walk_percent == 0
        && m.grounded
        && m.jump_startup == 0
        && a.stamina > 0.0
    {
        m.jump_startup = t.jump_startup_ticks.max(1);
        a.stamina = (a.stamina - t.jump_stamina).max(0.0);
        spent = true;
    }
    if spent {
        m.stamina_idle = 0;
    }
}

/// The character controller for the tuning in `t`.
pub fn kcc(t: &Locomotion) -> KinematicCharacterController {
    KinematicCharacterController {
        offset: CharacterLength::Absolute(t.controller_offset),
        slide: true,
        autostep: Some(CharacterAutostep {
            max_height: CharacterLength::Absolute(t.autostep_height),
            min_width: CharacterLength::Absolute(t.autostep_min_width),
            include_dynamic_bodies: false,
        }),
        max_slope_climb_angle: t.max_slope_climb_deg * DEG_TO_RAD,
        min_slope_slide_angle: t.min_slope_slide_deg * DEG_TO_RAD,
        snap_to_ground: Some(CharacterLength::Absolute(t.snap_to_ground)),
        ..KinematicCharacterController::default()
    }
}

/// Locomotion (tick system 5): jump takeoff, gravity, the controller move against the
/// level, landing, fall damage, and the kill plane.
pub fn step(a: &mut Actor, physics: &PhysicsState, t: &Locomotion) {
    a.motor.landed = false;
    if a.dead {
        return;
    }
    let m = &mut a.motor;
    if m.jump_startup > 0 {
        m.jump_startup -= 1;
        if m.jump_startup == 0 {
            a.vel[1] = jump_speed(t);
            m.grounded = false;
            m.jumped = true;
            m.fall_peak_y = m.ground_y;
        }
    }
    let was_grounded = m.grounded;
    // Vertical move: the average of the old and new velocity, which is exact under
    // constant gravity, so the jump peaks at `jump_height` rather than one step short.
    let dy = if m.grounded {
        a.vel[1] = 0.0;
        0.0
    } else {
        let v0 = a.vel[1];
        a.vel[1] = (v0 - t.gravity * DT).max(-t.terminal_fall_speed);
        (v0 + a.vel[1]) * 0.5 * DT
    };

    let lift = t.capsule_half_height + t.capsule_radius;
    let center = Vector::new(a.pos[0], a.pos[1] + lift, a.pos[2]);
    let desired = Vector::new(a.vel[0] * DT, dy, a.vel[2] * DT);
    let shape = Capsule::new_y(t.capsule_half_height, t.capsule_radius);
    let moved = kcc(t).move_shape(
        DT,
        &physics.queries(),
        &shape,
        &Pose::from_translation(center),
        desired,
        |_| {},
    );
    a.pos[0] += moved.translation.x;
    a.pos[1] += moved.translation.y;
    a.pos[2] += moved.translation.z;

    // Bumping a ceiling ends the rise.
    if a.vel[1] > 0.0 && moved.translation.y < desired.y * 0.5 {
        a.vel[1] = 0.0;
    }
    let grounded = moved.grounded && a.vel[1] <= 0.0;
    m.grounded = grounded;
    m.landed = grounded && !was_grounded;
    if grounded {
        // Remember the height of solid footing: ground right under the capsule's center.
        // Snap to ground lets the capsule roll down over an edge before it leaves the
        // ground; those ticks don't count, so a fall is measured from the top of the edge.
        let surface = footing(a.pos, physics, t);
        if let Some(y) = surface {
            m.ground_y = y;
            // Rapier's snap to ground can push the capsule into a large box floor (0.1 to
            // 0.17 m on m0_flat, issue #77); the feet then stay sunk until the next move. Lift
            // the feet back to the surface under the center. On a slope that surface is below
            // the feet, so this only lifts out of a floor. A sink within the controller offset
            // is float noise and stays, and a lift is never more than the snap distance.
            let sink = y - a.pos[1];
            if sink > t.controller_offset && sink <= t.snap_to_ground {
                a.pos[1] = y;
            }
        }
        if !was_grounded {
            let landed_on = surface.unwrap_or(a.pos[1] - t.controller_offset);
            land(a, landed_on, t);
        }
        a.vel[1] = 0.0;
    } else {
        if was_grounded && !m.jumped {
            // Walked off an edge: the fall starts at the last solid footing.
            m.fall_peak_y = m.ground_y;
        }
        m.fall_peak_y = m.fall_peak_y.max(a.pos[1] - t.controller_offset);
    }

    if a.pos[1] < physics.kill_y {
        a.hp = 0;
        a.dead = true;
    }
}

/// Height of the ground surface right under the capsule's center, if it's within reach
/// of the feet. Heights are of level geometry, so a 10 m drop measures 10 m exactly.
fn footing(pos: [f32; 3], physics: &PhysicsState, t: &Locomotion) -> Option<f32> {
    let lift = t.capsule_half_height + t.capsule_radius;
    let origin = [pos[0], pos[1] + lift, pos[2]];
    let reach = lift + t.controller_offset + FOOTING_PROBE_M;
    physics
        .raycast(origin, [0.0, -1.0, 0.0], reach)
        .map(|toi| origin[1] - toi)
}

/// Landing on the surface at height `surface`: fall damage, then jump landing recovery.
fn land(a: &mut Actor, surface: f32, t: &Locomotion) {
    let m = &mut a.motor;
    let fall = m.fall_peak_y - surface;
    m.last_fall_m = fall;
    if m.jumped {
        m.landing = t.jump_landing_ticks;
        m.jumped = false;
    }
    let damage = fall_damage(fall, a.max_hp, t);
    a.hp = a.hp.saturating_sub(damage);
    if a.hp == 0 {
        a.dead = true;
    }
}

/// Fall damage in HP for a fall of `fall_m` meters (docs/game/02-world.md, "Traversal
/// and world mechanics", Fall damage): none up to `fall_damage_from_m`, then
/// `fall_damage_per_m` of max HP per meter above it, rounded to the nearest HP; death
/// over `fall_death_over_m`.
pub fn fall_damage(fall_m: f32, max_hp: u32, t: &Locomotion) -> u32 {
    if fall_m > t.fall_death_over_m {
        return max_hp;
    }
    if fall_m <= t.fall_damage_from_m {
        return 0;
    }
    let frac = t.fall_damage_per_m * (fall_m - t.fall_damage_from_m);
    // Round half up; the cast truncates a non-negative value.
    (max_hp as f32 * frac + 0.5) as u32
}
