//! Steering along navigation paths (docs/game/05-enemies-and-bosses.md, "Navigation").
//!
//! An actor walks straight at its goal when nothing is in the way. When a wall is, it asks
//! the world's [`NavQueue`](crate::nav::NavQueue) for a path, keeps standing or steering
//! straight meanwhile, and follows the waypoints once the answer arrives (a later tick: the
//! queue answers a few requests per tick). A request that fails (no path, or the search
//! ran out of budget) leaves it steering straight, and it waits `path_retry_ticks` before
//! asking again. A path is replanned when its goal has moved `repath_m`, and dropped when
//! the straight line is clear again or a waypoint turns out to be walled off.

use lf_content::enemy::AiTuning;

use super::behaviors::desired_move;
use super::perception::dist_xz;
use crate::arena::Id;
use crate::math::dmath::sqrt;
use crate::physics::PhysicsState;
use crate::world::World;

/// True if a wall lies on the straight line from `from` to `goal` (at the capsule's
/// center height, `lift`).
fn blocked(physics: &PhysicsState, lift: f32, from: [f32; 3], goal: [f32; 3]) -> bool {
    let (dx, dz) = (goal[0] - from[0], goal[2] - from[2]);
    let d = sqrt(dx * dx + dz * dz);
    if d <= 0.0 {
        return false;
    }
    let origin = [from[0], from[1] + lift, from[2]];
    physics.raycast(origin, [dx / d, 0.0, dz / d], d).is_some()
}

/// The planar velocity for `id` to go from `from` to `goal` at `speed`, along a nav path
/// if a wall is in the way, and the path bookkeeping that needs. `None` stops it.
pub fn steer(
    world: &mut World,
    id: Id,
    lift: f32,
    from: [f32; 3],
    goal: [f32; 3],
    speed: f32,
    t: &AiTuning,
) -> Option<[f32; 2]> {
    if world.nav.is_none() || !blocked(&world.physics, lift, from, goal) {
        if let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
            b.path.clear();
        }
        return desired_move(&world.physics, lift, from, goal, speed, t);
    }
    let b = world.actors.get_mut(id).and_then(|a| a.ai.as_mut())?;
    b.path_retry = b.path_retry.saturating_sub(1);
    let fresh = !b.path.is_empty() && dist_xz(b.path_goal, goal) <= t.repath_m;
    if !fresh && b.path_ticket.is_none() && b.path_retry == 0 {
        // Replan. The old path (if any) keeps being followed until the answer arrives.
        if let Ok(ticket) = world.nav_queue.request(from, goal) {
            b.path_ticket = Some(ticket);
            b.path_goal = goal;
        }
    }
    // Drop the waypoints it has reached.
    while b
        .path
        .first()
        .is_some_and(|w| dist_xz(from, *w) <= t.arrive_m)
    {
        b.path.remove(0);
    }
    let next = b.path.first().copied();
    match next {
        Some(w) => {
            let v = desired_move(&world.physics, lift, from, w, speed, t);
            if v.is_none() {
                // Walled off from the waypoint: forget the path and ask again.
                if let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
                    b.path.clear();
                }
            }
            v
        }
        None => desired_move(&world.physics, lift, from, goal, speed, t),
    }
}

/// Answers this tick's path requests and hands each result to the actor that asked.
pub fn deliver(world: &mut World) {
    let Some(grid) = world.nav.as_ref() else {
        return;
    };
    let retry = world.ai.tuning.map_or(0, |t| t.path_retry_ticks);
    for r in world.nav_queue.run_tick(grid) {
        let asker = world
            .actors
            .iter()
            .find(|(_, a)| {
                a.ai.as_ref()
                    .is_some_and(|b| b.path_ticket == Some(r.ticket))
            })
            .map(|(id, _)| id);
        let Some(b) = asker
            .and_then(|id| world.actors.get_mut(id))
            .and_then(|a| a.ai.as_mut())
        else {
            continue;
        };
        b.path_ticket = None;
        match r.result {
            Ok(p) => {
                // The first waypoint is where the actor stood when it asked.
                b.path = p.waypoints.into_iter().skip(1).collect();
                b.path_retry = 0;
            }
            Err(_) => {
                b.path.clear();
                b.path_retry = retry;
            }
        }
    }
}
