//! Lock-on targeting (docs/game/03-combat.md, "Lock-on"): acquiring a target, breaking
//! the lock, auto-retargeting when the target dies, and switching left and right.
//!
//! The lock target is `Actor::lock_target` on the player. Every number comes from
//! `LockOn` (`game/data/locomotion.ron`). Ties are broken by arena index, so selection
//! depends only on the world state.
//!
//! Reference direction: the core doesn't see the camera, so "the camera's forward
//! direction" is the player's facing (the rule the spec gives for RL hosts). Left is
//! the side of positive angle: yaw grows from +Z toward +X, and +X is to the left of
//! someone facing +Z.

use lf_content::LockOn;

use super::Actor;
use crate::arena::{Arena, Id};
use crate::input::{ActionFrame, buttons};
use crate::math::dmath::atan2;
use crate::physics::PhysicsState;

const TAU: f32 = core::f32::consts::TAU;
const PI: f32 = core::f32::consts::PI;
/// Degrees to radians. Plain multiplication, no transcendental call.
const DEG_TO_RAD: f32 = PI / 180.0;
/// Two angles closer than this, in radians (about 0.006 degrees), count as equal when
/// breaking ties by distance. Float noise, not a gameplay number.
const ANGLE_TIE_EPS: f32 = 1.0e-4;

/// Orders candidates by angle, then distance, treating angles within `ANGLE_TIE_EPS` as
/// equal. Equal on both keeps the earlier (lower-index) candidate.
fn closer(angle: f32, dist: f32, best_angle: f32, best_dist: f32) -> bool {
    if (angle - best_angle).abs() <= ANGLE_TIE_EPS {
        dist < best_dist
    } else {
        angle < best_angle
    }
}

/// Heading from `from` to `to` in the XZ plane, in radians from +Z toward +X. Returns
/// `fallback` when the two are on the same vertical line.
pub fn bearing(from: [f32; 3], to: [f32; 3], fallback: f32) -> f32 {
    let dx = to[0] - from[0];
    let dz = to[2] - from[2];
    if dx == 0.0 && dz == 0.0 {
        fallback
    } else {
        atan2(dx, dz)
    }
}

/// Wraps an angle to `(-pi, pi]`.
pub fn wrap_pi(mut a: f32) -> f32 {
    while a > PI {
        a -= TAU;
    }
    while a <= -PI {
        a += TAU;
    }
    a
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

/// A living actor other than the player.
fn is_valid(a: &Actor) -> bool {
    !a.is_player && !a.dead && a.hp > 0
}

/// Whether the level leaves a clear line between the two actors' eyes. Actors don't
/// block sight.
fn line_of_sight(from: [f32; 3], to: [f32; 3], physics: &PhysicsState, t: &LockOn) -> bool {
    let eye = |p: [f32; 3]| [p[0], p[1] + t.eye_height_m, p[2]];
    let (a, b) = (eye(from), eye(to));
    let delta = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    physics.raycast(a, delta, 1.0).is_none()
}

/// The target `LOCK` picks: the valid actor with the smallest angle to `facing` inside
/// the acquisition range and cone, with a clear line of sight; ties go to the nearer
/// one, then the lower index.
pub fn acquire(
    actors: &Arena<Actor>,
    pos: [f32; 3],
    facing: f32,
    physics: &PhysicsState,
    t: &LockOn,
) -> Option<Id> {
    let half_cone = t.acquire_cone_deg * 0.5 * DEG_TO_RAD;
    let range_sq = t.acquire_range_m * t.acquire_range_m;
    let mut best: Option<(f32, f32, Id)> = None;
    for (id, a) in actors.iter() {
        if !is_valid(a) {
            continue;
        }
        let d = dist_sq(pos, a.pos);
        if d > range_sq {
            continue;
        }
        let angle = wrap_pi(bearing(pos, a.pos, facing) - facing).abs();
        if angle > half_cone || !line_of_sight(pos, a.pos, physics, t) {
            continue;
        }
        let better = best.is_none_or(|(ba, bd, _)| closer(angle, d, ba, bd));
        if better {
            best = Some((angle, d, id));
        }
    }
    best.map(|(_, _, id)| id)
}

/// The nearest valid actor within the retarget radius of `pos` (no cone, no line of
/// sight), picked after the locked target dies. Ties go to the lower index.
pub fn retarget(actors: &Arena<Actor>, pos: [f32; 3], t: &LockOn) -> Option<Id> {
    let range_sq = t.retarget_range_m * t.retarget_range_m;
    let mut best: Option<(f32, Id)> = None;
    for (id, a) in actors.iter() {
        if !is_valid(a) {
            continue;
        }
        let d = dist_sq(pos, a.pos);
        if d <= range_sq && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, id));
        }
    }
    best.map(|(_, id)| id)
}

/// The valid actor nearest in angle to the left (`left` true) or right of `current`,
/// seen from `pos`, within the acquisition range. Line of sight and the cone don't
/// apply. `None` when nothing lies on that side.
pub fn switch(
    actors: &Arena<Actor>,
    pos: [f32; 3],
    current: Id,
    left: bool,
    fallback_yaw: f32,
    t: &LockOn,
) -> Option<Id> {
    let from = actors.get(current)?.pos;
    let reference = bearing(pos, from, fallback_yaw);
    let range_sq = t.acquire_range_m * t.acquire_range_m;
    let mut best: Option<(f32, f32, Id)> = None;
    for (id, a) in actors.iter() {
        if id == current || !is_valid(a) || dist_sq(pos, a.pos) > range_sq {
            continue;
        }
        let rel = wrap_pi(bearing(pos, a.pos, reference) - reference);
        let side = if left { rel } else { -rel };
        if side <= 0.0 {
            continue;
        }
        let d = dist_sq(pos, a.pos);
        let better = best.is_none_or(|(bs, bd, _)| closer(side, d, bs, bd));
        if better {
            best = Some((side, d, id));
        }
    }
    best.map(|(_, _, id)| id)
}

/// Advances the player's lock-on for one tick, before the controller reads it:
/// 1. `LOCK` pressed while locked releases the lock.
/// 2. A target that died or vanished is replaced by the nearest one within the retarget
///    radius, or the lock ends; one beyond the break distance ends the lock.
/// 3. `SWITCH_L` or `SWITCH_R` moves the lock (`SWITCH_L` wins if both are pressed).
/// 4. `LOCK` pressed while unlocked acquires a target.
pub fn update(
    actors: &mut Arena<Actor>,
    player: Id,
    physics: &PhysicsState,
    frame: ActionFrame,
    t: &LockOn,
) {
    let Some(p) = actors.get(player) else {
        return;
    };
    let (pos, yaw, dead, prev) = (p.pos, p.yaw, p.dead, p.motor.prev_buttons);
    let pressed = frame.buttons & !prev;
    let mut lock = p.lock_target;

    if dead || (lock.is_some() && pressed & buttons::LOCK != 0) {
        lock = None;
    } else {
        if let Some(id) = lock {
            match actors.get(id) {
                Some(a) if is_valid(a) => {
                    if dist_sq(pos, a.pos) > t.break_range_m * t.break_range_m {
                        lock = None;
                    }
                }
                _ => lock = retarget(actors, pos, t),
            }
        }
        if let Some(id) = lock {
            let dir = if pressed & buttons::SWITCH_L != 0 {
                Some(true)
            } else if pressed & buttons::SWITCH_R != 0 {
                Some(false)
            } else {
                None
            };
            if let Some(left) = dir {
                lock = switch(actors, pos, id, left, yaw, t).or(lock);
            }
        } else if pressed & buttons::LOCK != 0 {
            lock = acquire(actors, pos, yaw, physics, t);
        }
    }

    if let Some(p) = actors.get_mut(player) {
        p.lock_target = lock;
    }
}

#[cfg(test)]
mod tests {
    use lf_content::Content;

    use super::*;
    use crate::input::TickInput;
    use crate::math::dmath::{cos, sin};
    use crate::snapshot::{ActorSnap, flags, write_snapshot};
    use crate::tick::tick;
    use crate::world::World;

    /// The `test_field` with its dummies removed, so each test places its own.
    fn field() -> World {
        let content = Content::load_embedded().expect("embedded content");
        let mut w = World::new(3, &content, "test_field").expect("test_field exists");
        // Flat ground for 30 m ahead, so nothing blocks the view.
        w.actors.get_mut(w.player).unwrap().pos = [0.0, 0.0, -25.0];
        let dummies: Vec<Id> = w
            .actors
            .iter()
            .map(|(id, _)| id)
            .filter(|&id| id != w.player)
            .collect();
        for id in dummies {
            w.actors.remove(id);
        }
        w
    }

    fn tuning(w: &World) -> LockOn {
        w.locomotion.lock_on
    }

    /// Adds a dummy `dist` meters from the player at `deg` degrees to the left (negative
    /// is right) of the facing, on the flat ground.
    fn dummy(w: &mut World, deg: f32, dist: f32) -> Id {
        let a = deg * DEG_TO_RAD;
        let p = w.actors.get(w.player).unwrap().pos;
        w.actors.insert(Actor {
            pos: [p[0] + sin(a) * dist, p[1], p[2] + cos(a) * dist],
            hp: 10,
            max_hp: 10,
            ..Actor::default()
        })
    }

    fn step(w: &mut World, move_x: i8, move_z: i8, buttons: u16) {
        let input = TickInput {
            frame: ActionFrame {
                move_x,
                move_z,
                buttons,
            },
            commands: Vec::new(),
        };
        tick(w, &input);
    }

    /// Presses and releases `button`.
    fn tap(w: &mut World, button: u16) {
        step(w, 0, 0, button);
        step(w, 0, 0, 0);
    }

    fn locked(w: &World) -> Option<Id> {
        w.actors.get(w.player).unwrap().lock_target
    }

    #[test]
    fn acquires_the_smallest_angle_then_the_nearest() {
        let mut w = field();
        let _wide = dummy(&mut w, 20.0, 5.0);
        let narrow = dummy(&mut w, -5.0, 15.0);
        tap(&mut w, buttons::LOCK);
        assert_eq!(locked(&w), Some(narrow));

        // Same angle: the nearer one wins.
        let mut w = field();
        let _far = dummy(&mut w, 10.0, 12.0);
        let near = dummy(&mut w, 10.0, 6.0);
        tap(&mut w, buttons::LOCK);
        assert_eq!(locked(&w), Some(near));
    }

    #[test]
    fn equal_angle_and_distance_go_to_the_lower_index() {
        let mut w = field();
        let first = dummy(&mut w, 10.0, 6.0);
        let _second = dummy(&mut w, 10.0, 6.0);
        tap(&mut w, buttons::LOCK);
        assert_eq!(locked(&w), Some(first));
    }

    #[test]
    fn acquisition_respects_the_cone_and_the_range() {
        let mut w = field();
        let t = tuning(&w);
        let half = t.acquire_cone_deg * 0.5;
        // Just outside the cone on either side, and just beyond range straight ahead.
        dummy(&mut w, half + 1.0, 5.0);
        dummy(&mut w, -(half + 1.0), 5.0);
        dummy(&mut w, 0.0, t.acquire_range_m + 0.5);
        tap(&mut w, buttons::LOCK);
        assert_eq!(locked(&w), None);

        // Just inside both limits.
        let inside = dummy(&mut w, half - 1.0, t.acquire_range_m - 0.5);
        tap(&mut w, buttons::LOCK);
        assert_eq!(locked(&w), Some(inside));
    }

    #[test]
    fn acquisition_needs_line_of_sight() {
        let mut w = field();
        // On the ramp top (9 m up), 16 m from a dummy on the ground behind the 10 m ledge.
        let id = w.player;
        let p = w.actors.get_mut(id).unwrap();
        p.pos = [0.0, 9.0, 26.0];
        p.motor.grounded = true;
        let behind = w.actors.insert(Actor {
            pos: [0.0, 0.0, 42.0],
            hp: 10,
            max_hp: 10,
            ..Actor::default()
        });
        assert!(!line_of_sight(
            [0.0, 9.0, 26.0],
            [0.0, 0.0, 42.0],
            &w.physics,
            &tuning(&w)
        ));
        tap(&mut w, buttons::LOCK);
        assert_eq!(locked(&w), None);
        // Line of sight isn't needed to keep a lock: it holds once set directly.
        w.actors.get_mut(id).unwrap().lock_target = Some(behind);
        step(&mut w, 0, 0, 0);
        assert_eq!(locked(&w), Some(behind));
    }

    #[test]
    fn lock_toggles_on_press_edges_only() {
        let mut w = field();
        let d = dummy(&mut w, 0.0, 5.0);
        step(&mut w, 0, 0, buttons::LOCK);
        assert_eq!(locked(&w), Some(d));
        // Holding LOCK doesn't release it.
        step(&mut w, 0, 0, buttons::LOCK);
        assert_eq!(locked(&w), Some(d));
        step(&mut w, 0, 0, 0);
        tap(&mut w, buttons::LOCK);
        assert_eq!(locked(&w), None);
    }

    #[test]
    fn breaks_beyond_30_m_and_not_before() {
        let mut w = field();
        let d = dummy(&mut w, 0.0, 5.0);
        tap(&mut w, buttons::LOCK);
        assert_eq!(locked(&w), Some(d));
        let t = tuning(&w);
        assert_eq!(t.break_range_m, 30.0);
        let pz = w.actors.get(w.player).unwrap().pos[2];
        w.actors.get_mut(d).unwrap().pos = [0.0, 0.0, pz + 29.9];
        step(&mut w, 0, 0, 0);
        assert_eq!(locked(&w), Some(d));
        w.actors.get_mut(d).unwrap().pos = [0.0, 0.0, pz + 30.1];
        step(&mut w, 0, 0, 0);
        assert_eq!(locked(&w), None);
        // It doesn't come back by itself when the target returns.
        w.actors.get_mut(d).unwrap().pos = [0.0, 0.0, pz + 5.0];
        step(&mut w, 0, 0, 0);
        assert_eq!(locked(&w), None);
    }

    #[test]
    fn a_dead_target_retargets_within_8_m() {
        let mut w = field();
        let first = dummy(&mut w, 0.0, 5.0);
        let far = dummy(&mut w, 90.0, 8.5);
        let near = dummy(&mut w, 90.0, 7.5);
        let nearer = dummy(&mut w, 170.0, 6.0);
        tap(&mut w, buttons::LOCK);
        assert_eq!(locked(&w), Some(first));
        w.actors.get_mut(first).unwrap().dead = true;
        step(&mut w, 0, 0, 0);
        // The nearest one wins, whatever its angle.
        assert_eq!(locked(&w), Some(nearer));
        w.actors.get_mut(nearer).unwrap().hp = 0;
        step(&mut w, 0, 0, 0);
        assert_eq!(locked(&w), Some(near));
        w.actors.get_mut(near).unwrap().dead = true;
        step(&mut w, 0, 0, 0);
        // 8.5 m is outside the 8 m retarget radius: the lock ends.
        assert_eq!(locked(&w), None);
        let _ = far;
    }

    #[test]
    fn a_removed_target_counts_as_dead() {
        let mut w = field();
        let first = dummy(&mut w, 0.0, 5.0);
        let other = dummy(&mut w, 40.0, 7.0);
        tap(&mut w, buttons::LOCK);
        w.actors.remove(first);
        step(&mut w, 0, 0, 0);
        assert_eq!(locked(&w), Some(other));
    }

    #[test]
    fn switching_moves_to_the_nearest_target_on_that_side() {
        let mut w = field();
        let mid = dummy(&mut w, 0.0, 6.0);
        let left_far = dummy(&mut w, 40.0, 6.0);
        let left_near = dummy(&mut w, 15.0, 9.0);
        let right = dummy(&mut w, -25.0, 6.0);
        tap(&mut w, buttons::LOCK);
        assert_eq!(locked(&w), Some(mid));
        tap(&mut w, buttons::SWITCH_L);
        assert_eq!(locked(&w), Some(left_near));
        tap(&mut w, buttons::SWITCH_L);
        assert_eq!(locked(&w), Some(left_far));
        // Nothing further left: the lock stays.
        tap(&mut w, buttons::SWITCH_L);
        assert_eq!(locked(&w), Some(left_far));
        tap(&mut w, buttons::SWITCH_R);
        assert_eq!(locked(&w), Some(left_near));
        tap(&mut w, buttons::SWITCH_R);
        assert_eq!(locked(&w), Some(mid));
        tap(&mut w, buttons::SWITCH_R);
        assert_eq!(locked(&w), Some(right));
    }

    #[test]
    fn switching_does_nothing_without_a_lock() {
        let mut w = field();
        dummy(&mut w, 10.0, 5.0);
        tap(&mut w, buttons::SWITCH_L);
        assert_eq!(locked(&w), None);
    }

    #[test]
    fn locked_movement_strafes_around_the_target() {
        let mut w = field();
        let d = dummy(&mut w, 0.0, 6.0);
        tap(&mut w, buttons::LOCK);
        let target = w.actors.get(d).unwrap().pos;
        // Hold the stick to the player's left (+X): the player slides sideways and keeps
        // facing the target.
        for _ in 0..30 {
            step(&mut w, 127, 0, 0);
        }
        let p = w.actors.get(w.player).unwrap();
        assert!(p.pos[0] > 2.0, "moved along +X: {:?}", p.pos);
        let want = atan2(target[0] - p.pos[0], target[2] - p.pos[2]);
        assert!(
            (wrap_pi(p.yaw - want)).abs() < 0.05,
            "yaw {} want {want}",
            p.yaw
        );
        assert!(p.yaw < 0.0, "turned toward -X as it moved to +X: {}", p.yaw);
        // Without a lock the same input faces the way it moves.
        tap(&mut w, buttons::LOCK);
        for _ in 0..5 {
            step(&mut w, 127, 0, 0);
        }
        let p = w.actors.get(w.player).unwrap();
        assert!((p.yaw - core::f32::consts::FRAC_PI_2).abs() < 1e-4);
    }

    #[test]
    fn the_snapshot_flags_the_locked_target_and_the_dead() {
        let mut w = field();
        let a = dummy(&mut w, 0.0, 5.0);
        let b = dummy(&mut w, 50.0, 5.0);
        tap(&mut w, buttons::LOCK);
        w.actors.get_mut(b).unwrap().dead = true;
        let mut out: Vec<ActorSnap> = Vec::new();
        write_snapshot(&w, &mut out);
        let flags_of = |id: Id| {
            out.iter()
                .find(|s| s.id == crate::snapshot::pack_id(id))
                .unwrap()
                .flags
        };
        assert_eq!(flags_of(a), flags::LOCKED_TARGET);
        assert_eq!(flags_of(b), flags::DEAD);
        assert_eq!(flags_of(w.player), 0);
    }

    #[test]
    fn the_test_field_has_three_dummies_that_stand_still() {
        let content = Content::load_embedded().unwrap();
        let mut w = World::new(1, &content, "test_field").unwrap();
        let before: Vec<[f32; 3]> = w
            .actors
            .iter()
            .filter(|(id, _)| *id != w.player)
            .map(|(_, a)| a.pos)
            .collect();
        assert_eq!(before.len(), 3);
        for _ in 0..60 {
            step(&mut w, 0, 127, 0);
        }
        let after: Vec<[f32; 3]> = w
            .actors
            .iter()
            .filter(|(id, _)| *id != w.player)
            .map(|(_, a)| a.pos)
            .collect();
        assert_eq!(before, after);
        assert!(w.actors.iter().all(|(_, a)| a.hp > 0 && a.hp == a.max_hp));
    }
}
