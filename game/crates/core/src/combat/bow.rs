//! Bow aiming (docs/game/04-arsenal-and-builds.md, "Weapon classes": "aim while locked on or
//! free-aim").
//!
//! A bow's moves are ordinary moves that launch a projectile (`Move::projectiles`), so
//! aiming is the move's tracking, which turns the actor until the first arrow leaves
//! (`MoveDef::track_until`; a charged shot keeps aiming while it holds at `charge_from`):
//!
//! - **Locked on:** `moves::step` turns the actor toward the lock target at the move's
//!   `track_deg_per_tick`. Nothing here.
//! - **Free aim:** with no lock target, a move that sets `free_aim` turns the actor toward
//!   the stick instead ([`free_aim`]), at the same rate. The stick holds a heading in world
//!   axes, as it does for walking (`actor::controller::intent`): the core doesn't see the
//!   camera. With the stick at rest the actor keeps its facing.
//!
//! The arrow flies along the facing at the frame it spawns (`projectile::spawn`), so the
//! aim you hold when the arrow leaves is the aim it takes. Ammunition is not spent here:
//! arrows are WP-057's.

use crate::actor::Actor;
use crate::combat::moves::turn_toward;
use crate::input::ActionFrame;
use crate::math::dmath::atan2;

/// Turns `a` toward the stick heading by at most `max_step` radians. Does nothing with the
/// stick at rest.
pub fn free_aim(a: &mut Actor, f: ActionFrame, max_step: f32) {
    if f.move_x == 0 && f.move_z == 0 {
        return;
    }
    let want = atan2(f32::from(f.move_x), f32::from(f.move_z));
    a.yaw = turn_toward(a.yaw, want, max_step);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Event, ProjectileEventKind};
    use crate::input::{TickInput, buttons};
    use crate::tick::tick;
    use crate::world::World;
    use lf_content::Content;

    const TO_RAD: f32 = core::f32::consts::PI / 180.0;

    fn world() -> World {
        let content = Content::load_embedded().expect("embedded content");
        World::new(5202, &content, "wp052_bow").expect("wp052_bow exists")
    }

    fn frame(x: i8, z: i8, b: u16) -> TickInput {
        TickInput {
            frame: ActionFrame {
                move_x: x,
                move_z: z,
                buttons: b,
            },
            commands: Vec::new(),
        }
    }

    /// Fires one light shot with the stick held, and returns the player's yaw on the tick
    /// the arrow spawned.
    fn shoot(w: &mut World, x: i8, z: i8, lock_first: bool) -> f32 {
        if lock_first {
            tick(w, &frame(0, 0, buttons::LOCK));
            tick(w, &frame(0, 0, 0));
            assert!(w.actors.get(w.player).unwrap().lock_target.is_some());
        }
        // The press tick has the stick at rest, so the walk's own facing doesn't snap the actor
        // to it before the move starts: the move's aim does the turning.
        tick(w, &frame(0, 0, buttons::LIGHT));
        for _ in 0..30 {
            tick(w, &frame(x, z, 0));
            for e in w.events.drain() {
                if let Event::Projectile(p) = e
                    && p.kind == ProjectileEventKind::Spawned
                {
                    return w.actors.get(w.player).unwrap().yaw;
                }
            }
        }
        panic!("the shot never launched an arrow");
    }

    #[test]
    fn free_aim_turns_toward_the_stick_before_the_arrow_leaves() {
        let mut w = world();
        // The stick points 45 degrees to the left of the facing.
        let yaw = shoot(&mut w, 70, 70, false);
        assert!(
            (yaw - 45.0 * TO_RAD).abs() < 2.0 * TO_RAD,
            "yaw {} did not reach the stick heading",
            yaw / TO_RAD
        );
    }

    #[test]
    fn free_aim_is_limited_to_the_turn_rate() {
        let mut w = world();
        // Straight behind: 180 degrees away, more than the draw lets the bow turn (12 degrees
        // a tick, for the 8 startup ticks after the press).
        let yaw = shoot(&mut w, 0, -100, false);
        assert!(
            (yaw.abs() - 96.0 * TO_RAD).abs() < 2.0 * TO_RAD,
            "yaw {} should be 96 degrees round",
            yaw / TO_RAD
        );
    }

    #[test]
    fn a_resting_stick_keeps_the_facing() {
        let mut w = world();
        assert_eq!(shoot(&mut w, 0, 0, false), 0.0);
    }

    #[test]
    fn locked_on_aim_follows_the_target_and_ignores_the_stick() {
        let mut w = world();
        let me = w.actors.get(w.player).unwrap().pos;
        let dummy = w
            .actors
            .iter()
            .find(|(id, _)| *id != w.player)
            .map(|(_, a)| a.pos)
            .unwrap();
        let want = crate::actor::targeting::bearing(me, dummy, 0.0);
        let yaw = shoot(&mut w, -100, 0, true);
        assert!(
            (yaw - want).abs() < 1.0 * TO_RAD,
            "yaw {} should face the dummy at {}",
            yaw / TO_RAD,
            want / TO_RAD
        );
    }
}
