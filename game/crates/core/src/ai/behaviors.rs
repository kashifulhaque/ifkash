//! Built-in behaviors and the per-tick execution of awareness states
//! (docs/game/05-enemies-and-bosses.md, "Awareness states" and "Decision making").
//!
//! [`execute`] runs every tick for every live AI actor, after its think. It sets the
//! actor's planar velocity and turns its facing; tick system 5 (locomotion) then moves it
//! with the character controller. It leaves an actor in a move alone (the move's root
//! motion and tracking own it) and holds a staggered one still.
//!
//! | State or behavior | Moves | Faces |
//! |---|---|---|
//! | Idle | Walks its patrol, or stands | Where it walks |
//! | Suspicious | Walks to the noise | Where it walks |
//! | Aware | Stands | The target |
//! | Approach | Runs to `approach_stop_m` from the target | The target |
//! | Strafe | Walks sideways round the target | The target |
//! | Retreat | Walks away from the target | The target |
//! | Guard, Wait | Stands | The target |
//! | Flee | Runs away from the target | Where it runs |
//! | Return | Runs home | Where it runs |
//! | Burrow | Walks to the nearest reed patch (WP-036) | Where it walks |
//!
//! "The target" is where the actor last saw it (`AiState::last_seen`), so a target that
//! slips out of sight is lost rather than tracked through walls.
//!
//! Guard raises the guard (`AiState::guarding`, read by `combat::moves::step`) for as long
//! as the behavior lasts; an enemy blocks only if its definition has `guard` stats.
//!
//! Walking goals (the target, a noise, a waypoint, home) go through [`super::pathing`]:
//! straight when clear, along a nav path when a wall is in the way. Strafing, retreating,
//! and fleeing only pick a direction, so they steer straight and stop at a wall. A
//! definition's `speed_percent` scales every speed.

use lf_content::enemy::{AiTuning, Behavior};

use super::perception::dist_xz;
use super::{Awareness, Doing};
use crate::actor::targeting::bearing;
use crate::arena::Id;
use crate::combat::moves::turn_toward;
use crate::math::dmath::{cos, sin, sqrt};
use crate::physics::PhysicsState;
use crate::world::World;

/// Degrees to radians. Plain multiplication, no transcendental call.
const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;
/// How far ahead a direction-only behavior (strafe, retreat, flee) aims its goal. Any
/// distance past `arrive_m` gives the same velocity; not a gameplay number.
const AIM_AHEAD_M: f32 = 2.0;

/// The steering seam: the planar velocity that takes an actor at `from` toward `goal` at
/// `speed`, or `None` once it's within `arrive_m` or a wall is in the way. It moves in a
/// straight line and probes `probe_m` ahead at the capsule's center height (`lift`, above
/// the steps the controller climbs). [`super::pathing::steer`] calls it for each straight
/// stretch and for each waypoint of a nav path.
pub fn desired_move(
    physics: &PhysicsState,
    lift: f32,
    from: [f32; 3],
    goal: [f32; 3],
    speed: f32,
    t: &AiTuning,
) -> Option<[f32; 2]> {
    let (dx, dz) = (goal[0] - from[0], goal[2] - from[2]);
    let d = sqrt(dx * dx + dz * dz);
    if d <= t.arrive_m {
        return None;
    }
    let dir = [dx / d, 0.0, dz / d];
    let origin = [from[0], from[1] + lift, from[2]];
    if physics.raycast(origin, dir, t.probe_m.min(d)).is_some() {
        return None;
    }
    Some([dir[0] * speed, dir[2] * speed])
}

/// A point `AIM_AHEAD_M` from `from` along `yaw`.
fn ahead(from: [f32; 3], yaw: f32) -> [f32; 3] {
    [
        from[0] + sin(yaw) * AIM_AHEAD_M,
        from[1],
        from[2] + cos(yaw) * AIM_AHEAD_M,
    ]
}

/// Where to go and which way to face.
enum Plan {
    Stand {
        face: Option<[f32; 3]>,
    },
    Go {
        goal: [f32; 3],
        speed: f32,
        face: Option<[f32; 3]>,
        /// The goal is a place to reach (so a wall means a nav path), not a direction.
        nav: bool,
    },
}

/// One tick of the actor's state: timers, then velocity and facing.
pub fn execute(world: &mut World, id: Id, t: &AiTuning) {
    let lift = world.locomotion.capsule_half_height + world.locomotion.capsule_radius;
    {
        let Some(a) = world.actors.get_mut(id) else {
            return;
        };
        let Some(b) = a.ai.as_mut() else {
            return;
        };
        b.state_ticks = b.state_ticks.saturating_add(1);
        if b.awareness == Awareness::Combat {
            b.unseen_ticks = b.unseen_ticks.saturating_add(1);
        }
        for c in &mut b.cooldowns {
            *c = c.saturating_sub(1);
        }
        if matches!(b.doing, Doing::Behavior(_)) {
            b.doing_ticks = b.doing_ticks.saturating_sub(1);
        }
        b.since_hurt = b.since_hurt.saturating_add(1);
        b.guarding = b.awareness == Awareness::Combat
            && b.doing_ticks > 0
            && matches!(b.doing, Doing::Behavior(Behavior::Guard { .. }));
        if a.moves.current.is_some() {
            return;
        }
        if a.moves.guard.stagger > 0 || !a.motor.grounded {
            if a.motor.grounded {
                a.vel[0] = 0.0;
                a.vel[2] = 0.0;
            }
            return;
        }
    }

    let Some(a) = world.actors.get(id) else {
        return;
    };
    let Some(b) = a.ai.as_ref() else {
        return;
    };
    let mut next_patrol = None;
    // A flyer with nothing else to do drifts toward a lit Lantern (WP-036).
    let light = super::special::flying::light_goal(world, id);
    let target = b.target.map(|_| b.last_seen);
    let spawn = world.layout.enemies.get(usize::from(b.spawn));
    let home = spawn.map_or(a.pos, |s| s.pos);
    let pos = a.pos;
    let plan = match b.awareness {
        Awareness::Idle => match spawn.filter(|s| !s.patrol.is_empty()) {
            Some(s) => {
                let i = usize::from(b.patrol_i) % s.patrol.len();
                let goal = s.patrol[i];
                if dist_xz(pos, goal) <= t.arrive_m {
                    next_patrol = Some(((i + 1) % s.patrol.len()) as u16);
                    Plan::Stand { face: None }
                } else {
                    Plan::Go {
                        goal,
                        speed: t.walk_speed,
                        face: None,
                        nav: true,
                    }
                }
            }
            None => match light {
                Some(goal) => Plan::Go {
                    goal,
                    speed: t.walk_speed,
                    face: None,
                    nav: false,
                },
                None => Plan::Stand { face: None },
            },
        },
        Awareness::Suspicious => Plan::Go {
            goal: b.noise_at,
            speed: t.walk_speed,
            face: None,
            nav: true,
        },
        Awareness::Aware => Plan::Stand { face: target },
        Awareness::Return => Plan::Go {
            goal: home,
            speed: t.run_speed,
            face: None,
            nav: true,
        },
        Awareness::Combat => match (b.doing, target) {
            (Doing::Behavior(beh), Some(tp)) if b.doing_ticks > 0 => {
                let to = bearing(pos, tp, a.yaw);
                match beh {
                    Behavior::Approach { .. } => {
                        if dist_xz(pos, tp) <= t.approach_stop_m {
                            Plan::Stand { face: target }
                        } else {
                            Plan::Go {
                                goal: tp,
                                speed: t.run_speed,
                                face: target,
                                nav: true,
                            }
                        }
                    }
                    Behavior::Strafe { .. } => {
                        let side = if b.strafe_left { 1.0 } else { -1.0 };
                        Plan::Go {
                            goal: ahead(pos, to + side * core::f32::consts::FRAC_PI_2),
                            speed: t.walk_speed,
                            face: target,
                            nav: false,
                        }
                    }
                    Behavior::Retreat { .. } => Plan::Go {
                        goal: ahead(pos, to + core::f32::consts::PI),
                        speed: t.walk_speed,
                        face: target,
                        nav: false,
                    },
                    Behavior::Flee { .. } => Plan::Go {
                        goal: ahead(pos, to + core::f32::consts::PI),
                        speed: t.run_speed,
                        face: None,
                        nav: false,
                    },
                    Behavior::Burrow { .. } => {
                        match super::special::reeds::nearest_patch(world, pos) {
                            Some(goal) if dist_xz(pos, goal) > t.arrive_m => Plan::Go {
                                goal,
                                speed: t.walk_speed,
                                face: None,
                                nav: false,
                            },
                            _ => Plan::Stand { face: target },
                        }
                    }
                    Behavior::Guard { .. } | Behavior::Wait { .. } | Behavior::Patrol => {
                        Plan::Stand { face: target }
                    }
                }
            }
            _ => Plan::Stand { face: target },
        },
    };

    let speed_percent = world
        .actors
        .get(id)
        .and_then(|a| a.ai.as_ref())
        .and_then(|b| world.ai.get(b.def))
        .map_or(100, |d| d.data.speed_percent);
    let speed_scale = f32::from(speed_percent) / 100.0;
    let (vel, face) = match plan {
        Plan::Stand { face } => (None, face),
        Plan::Go {
            goal,
            speed,
            face,
            nav: true,
        } => (
            super::pathing::steer(world, id, lift, pos, goal, speed * speed_scale, t),
            face,
        ),
        Plan::Go {
            goal, speed, face, ..
        } => (
            desired_move(&world.physics, lift, pos, goal, speed * speed_scale, t),
            face,
        ),
    };
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    if let (Some(n), Some(b)) = (next_patrol, a.ai.as_mut()) {
        b.patrol_i = n;
    }
    let [vx, vz] = vel.unwrap_or([0.0, 0.0]);
    a.vel[0] = vx;
    a.vel[2] = vz;
    let want = match face {
        Some(p) => Some(bearing(a.pos, p, a.yaw)),
        None if vel.is_some() => Some(bearing([0.0; 3], [vx, 0.0, vz], a.yaw)),
        None => None,
    };
    if let Some(y) = want {
        a.yaw = turn_toward(a.yaw, y, t.turn_deg_per_tick * DEG_TO_RAD);
    }
}
