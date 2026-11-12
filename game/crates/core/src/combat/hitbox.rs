//! Hitbox shapes in actor space, placed in the world, and the swept overlap test between
//! a hitbox and a hurtbox. See docs/game/03-combat.md, "Move data schema", and
//! docs/game/01-architecture.md, "Physics".
//!
//! # Actor space
//!
//! Move data gives shapes in the actor's local space: x right, y up, z forward, origin at
//! the feet. An actor's yaw grows from +Z toward +X, and +X is to the left of someone
//! facing +Z (see `actor::targeting`), so the actor's right is `(-cos yaw, 0, sin yaw)`
//! and its forward is `(sin yaw, 0, cos yaw)`.
//!
//! # Sweeps
//!
//! Rapier never resolves combat, and actors aren't Rapier bodies, so the tests here are
//! plain capsule-versus-capsule distance checks. To keep a fast swing or lunge from
//! passing through a thin target between two ticks, [`swept_overlap`] interpolates both
//! capsules from their previous-tick placement to their current one and tests substeps
//! no further apart than half the sum of the two radii.

use lf_content::moves::Shape;
use serde::{Deserialize, Serialize};

use crate::math::dmath::{cos, lerp, sin, sqrt};

pub type V3 = [f32; 3];

/// Most substeps one sweep tests. Even a 0.1 m blade against a 0.1 m hurtbox (0.1 m
/// spacing) gets full coverage over 6.4 m of travel in a tick, far more than any move
/// covers. Not a gameplay number: a cost cap.
pub const MAX_SUBSTEPS: u32 = 64;

/// Smallest substep spacing, in meters, so two zero-radius shapes still get a finite
/// substep count. Not a gameplay number.
const MIN_SPACING_M: f32 = 0.01;

/// Where an actor stands: feet position and yaw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Pose {
    pub pos: V3,
    pub yaw: f32,
}

impl Pose {
    /// Maps a point in actor space (x right, y up, z forward) to world space.
    pub fn to_world(&self, p: V3) -> V3 {
        let (s, c) = (sin(self.yaw), cos(self.yaw));
        [
            self.pos[0] - c * p[0] + s * p[2],
            self.pos[1] + p[1],
            self.pos[2] + s * p[0] + c * p[2],
        ]
    }

    /// The actor's forward direction in world space.
    pub fn forward(&self) -> V3 {
        [sin(self.yaw), 0.0, cos(self.yaw)]
    }
}

/// A capsule in world space: every point within `radius` of the segment `a`-`b`. A
/// sphere is a capsule with `a == b`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Capsule {
    pub a: V3,
    pub b: V3,
    pub radius: f32,
}

impl Capsule {
    /// A move-data shape placed at `pose`.
    pub fn from_shape(shape: &Shape, pose: &Pose) -> Self {
        match *shape {
            Shape::Capsule { a, b, radius } => Self {
                a: pose.to_world(a),
                b: pose.to_world(b),
                radius,
            },
            Shape::Sphere { center, radius } => {
                let c = pose.to_world(center);
                Self { a: c, b: c, radius }
            }
        }
    }

    /// The capsule `t` of the way from `self` to `other` (endpoints interpolated
    /// linearly; the radius is `other`'s).
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            a: lerp3(self.a, other.a, t),
            b: lerp3(self.b, other.b, t),
            radius: other.radius,
        }
    }
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add_scaled(a: V3, d: V3, t: f32) -> V3 {
    [a[0] + d[0] * t, a[1] + d[1] * t, a[2] + d[2] * t]
}

fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn lerp3(a: V3, b: V3, t: f32) -> V3 {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

fn len(a: V3) -> f32 {
    sqrt(dot(a, a))
}

/// The closest points between segments `p1`-`q1` and `p2`-`q2`, and their squared
/// distance (Ericson, "Real-Time Collision Detection", 5.1.9). Degenerate segments
/// (points) are handled.
pub fn closest_points(p1: V3, q1: V3, p2: V3, q2: V3) -> (V3, V3, f32) {
    // Below this squared length a segment counts as a point. Float noise, not gameplay.
    const EPS: f32 = 1.0e-12;
    let d1 = sub(q1, p1);
    let d2 = sub(q2, p2);
    let r = sub(p1, p2);
    let a = dot(d1, d1);
    let e = dot(d2, d2);
    let f = dot(d2, r);
    let (s, t) = if a <= EPS && e <= EPS {
        (0.0, 0.0)
    } else if a <= EPS {
        (0.0, (f / e).clamp(0.0, 1.0))
    } else {
        let c = dot(d1, r);
        if e <= EPS {
            ((-c / a).clamp(0.0, 1.0), 0.0)
        } else {
            let b = dot(d1, d2);
            let denom = a * e - b * b;
            let mut s = if denom > EPS {
                ((b * f - c * e) / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let mut t = (b * s + f) / e;
            if t < 0.0 {
                t = 0.0;
                s = (-c / a).clamp(0.0, 1.0);
            } else if t > 1.0 {
                t = 1.0;
                s = ((b - c) / a).clamp(0.0, 1.0);
            }
            (s, t)
        }
    };
    let c1 = add_scaled(p1, d1, s);
    let c2 = add_scaled(p2, d2, t);
    let d = sub(c1, c2);
    (c1, c2, dot(d, d))
}

/// If the capsules touch, a contact point between them: on the line joining the closest
/// points of their axes, where the two surfaces would meet.
pub fn overlap(x: &Capsule, y: &Capsule) -> Option<V3> {
    let (c1, c2, d2) = closest_points(x.a, x.b, y.a, y.b);
    let reach = x.radius + y.radius;
    if d2 > reach * reach {
        return None;
    }
    let k = if reach > 0.0 { x.radius / reach } else { 0.0 };
    Some(lerp3(c1, c2, k))
}

/// Tests a hitbox moving from `hit0` to `hit1` against a hurtbox moving from `hurt0` to
/// `hurt1` over one tick. Returns the earliest touching substep's fraction of the tick
/// (in `(0, 1]`) and contact point. The previous placement itself (fraction 0) isn't
/// tested: it belongs to the previous tick.
///
/// Substeps are spaced so that no endpoint moves more than half the sum of the radii
/// between two of them, relative to the other capsule. A target whose axis the hitbox's
/// axis passes within 0.97 times the sum of the radii is caught, so a fast swing can't
/// tunnel through a thin target.
pub fn swept_overlap(
    hit0: &Capsule,
    hit1: &Capsule,
    hurt0: &Capsule,
    hurt1: &Capsule,
) -> Option<(f32, V3)> {
    let travel = len(sub(hit1.a, hit0.a)).max(len(sub(hit1.b, hit0.b)))
        + len(sub(hurt1.a, hurt0.a)).max(len(sub(hurt1.b, hurt0.b)));
    let spacing = (0.5 * (hit1.radius + hurt1.radius)).max(MIN_SPACING_M);
    // `ceil` is exact in IEEE 754, so the count is the same on every target.
    let n = ((travel / spacing).ceil() as u32).clamp(1, MAX_SUBSTEPS);
    (1..=n).find_map(|k| {
        let t = k as f32 / n as f32;
        overlap(&hit0.lerp(hit1, t), &hurt0.lerp(hurt1, t)).map(|p| (t, p))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2;

    fn close(a: V3, b: V3) -> bool {
        len(sub(a, b)) < 1e-5
    }

    #[test]
    fn actor_space_axes() {
        let p = Pose {
            pos: [1.0, 2.0, 3.0],
            yaw: 0.0,
        };
        // Facing +Z: right is -X, forward is +Z.
        assert!(close(p.to_world([1.0, 0.0, 0.0]), [0.0, 2.0, 3.0]));
        assert!(close(p.to_world([0.0, 1.0, 2.0]), [1.0, 3.0, 5.0]));
        let p = Pose {
            pos: [0.0, 0.0, 0.0],
            yaw: FRAC_PI_2,
        };
        // Facing +X: right is +Z.
        assert!(close(p.to_world([0.0, 0.0, 1.0]), [1.0, 0.0, 0.0]));
        assert!(close(p.to_world([1.0, 0.0, 0.0]), [0.0, 0.0, 1.0]));
        assert!(close(p.forward(), [1.0, 0.0, 0.0]));
    }

    #[test]
    fn segment_distances() {
        // Crossing segments.
        let (_, _, d2) = closest_points(
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0],
            [0.0, 1.0, 1.0],
        );
        assert!((d2 - 1.0).abs() < 1e-6);
        // Parallel segments, offset.
        let (_, _, d2) = closest_points(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [2.0, 0.0, 0.5],
            [2.0, 0.0, 3.0],
        );
        assert!((d2 - 4.0).abs() < 1e-6);
        // A point and a segment.
        let (c1, c2, d2) = closest_points(
            [0.0, 3.0, 0.0],
            [0.0, 3.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!(close(c1, [0.0, 3.0, 0.0]) && close(c2, [0.0, 1.0, 0.0]));
        assert!((d2 - 4.0).abs() < 1e-6);
    }

    #[test]
    fn fast_hitbox_does_not_tunnel_through_a_thin_pole() {
        // A 0.2 m blade moving 3 m sideways in one tick past a 2 cm pole at the origin.
        let blade = |x: f32| Capsule {
            a: [x, 1.0, 0.0],
            b: [x, 1.0, 1.5],
            radius: 0.2,
        };
        let pole = Capsule {
            a: [0.0, 0.0, 0.5],
            b: [0.0, 1.8, 0.5],
            radius: 0.02,
        };
        let (start, end) = (blade(-1.5), blade(1.5));
        // Neither end of the tick touches the pole...
        assert_eq!(overlap(&start, &pole), None);
        assert_eq!(overlap(&end, &pole), None);
        // ...but the sweep between them does, about halfway through the tick.
        let (t, p) = swept_overlap(&start, &end, &pole, &pole).expect("sweep hits");
        assert!((t - 0.5).abs() < 0.1, "{t}");
        assert!(p[0].abs() < 0.2 && (p[2] - 0.5).abs() < 1e-5, "{p:?}");
        // A blade that stops short of the pole misses.
        assert_eq!(swept_overlap(&start, &blade(-0.5), &pole, &pole), None);
    }
}
