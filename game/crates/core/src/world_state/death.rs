//! Death, the Remnant, and the respawn. See docs/game/06-progression-and-systems.md,
//! "Glimmer and death".
//!
//! [`run`] is tick system 9. Each tick, in this order:
//!
//! 1. While alive, if the player's feet are on the ground and the spot isn't a hazard,
//!    that spot becomes the *safe position*, so it is the last tick their feet touched
//!    walkable, non-hazard ground. (A fall that kills on landing counts: the landing
//!    spot is ground.) Nothing is a hazard yet; [`is_hazard`] is where lava and
//!    the like go.
//! 2. If the player is dead (`actor.dead`, or HP at 0), the death begins: an unrecovered
//!    Remnant is destroyed, all held Glimmer drops as a new Remnant at the safe position
//!    (nothing if none is held, or with Keep Glimmer on death), and the respawn timer
//!    starts (`respawn_delay_ticks`).
//! 3. When the timer runs out the player respawns at the last Lantern they rested at (the
//!    scenario spawn before the first rest) with full HP, FP, stamina, and flasks, and
//!    the non-boss enemies respawn.
//! 4. A living player whose feet come within `remnant_pickup_radius_m` of the Remnant
//!    recovers its Glimmer.
//!
//! The Remnant is placed by [`remnant_position`]. Boss arenas change it to put the
//! Remnant just outside the fog gate.

use crate::actor::Motor;
use crate::combat::moves::MoveState;
use crate::combat::resolve::HitState;
use crate::events::{Event, LifeEvent};
use crate::world::World;

use super::Remnant;

/// Whether standing at `pos` counts as a hazard for the safe position. No hazards exist
/// yet.
pub fn is_hazard(_world: &World, _pos: [f32; 3]) -> bool {
    false
}

/// Where a Remnant dropped by a death at safe position `safe` goes. In a boss fight it
/// goes just outside the fog gate (`ai::boss::remnant_override`).
pub fn remnant_position(world: &World, safe: [f32; 3]) -> [f32; 3] {
    crate::ai::boss::remnant_override(world).unwrap_or(safe)
}

/// Tick system 9 for the player's life cycle.
pub fn run(world: &mut World) {
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    let (pos, grounded, dead) = (p.pos, p.motor.grounded, p.dead || p.hp == 0);
    if let Some(left) = world.state.death.respawn_in {
        let left = left.saturating_sub(1);
        if left == 0 {
            respawn(world);
        } else {
            world.state.death.respawn_in = Some(left);
        }
        return;
    }
    if grounded && !is_hazard(world, pos) {
        world.state.death.safe_pos = pos;
    }
    if dead {
        die(world);
    } else {
        recover(world);
    }
}

/// Begins the player's death: drops the Remnant and starts the respawn timer.
pub fn die(world: &mut World) {
    let tuning = world.locomotion.death;
    let player = world.player;
    let Some(p) = world.actors.get_mut(player) else {
        return;
    };
    p.hp = 0;
    p.dead = true;
    let pos = p.pos;
    let held = p.inventory.glimmer;
    world.events.push(Event::Life(LifeEvent::Died { pos }));
    world.state.death.deaths += 1;
    world.state.resting = None;
    world.state.death.respawn_in = Some(tuning.respawn_delay_ticks.max(1));
    // Dying again before recovering destroys the old Remnant.
    if let Some(old) = world.state.death.remnant.take() {
        world.events.push(Event::Life(LifeEvent::RemnantLost {
            glimmer: old.glimmer,
            pos: old.pos,
        }));
    }
    if held > 0 && !world.state.keep_glimmer_on_death {
        let at = remnant_position(world, world.state.death.safe_pos);
        if let Some(p) = world.actors.get_mut(player) {
            p.inventory.glimmer = 0;
        }
        world.state.death.remnant = Some(Remnant {
            pos: at,
            glimmer: held,
        });
        world.events.push(Event::Life(LifeEvent::RemnantDropped {
            glimmer: held,
            pos: at,
        }));
    }
}

/// Where and which way the player respawns: the last rested Lantern, else the spawn.
pub fn respawn_point(world: &World) -> ([f32; 3], f32) {
    world
        .state
        .respawn_lantern
        .as_deref()
        .and_then(|id| world.layout.lantern(id))
        .map_or((world.layout.spawn, world.layout.spawn_yaw), |(_, l)| {
            (l.pos, l.yaw)
        })
}

/// Brings the player back at the respawn point, whole, and respawns the enemies.
pub fn respawn(world: &mut World) {
    let (pos, yaw) = respawn_point(world);
    if let Some(a) = world.actors.get_mut(world.player) {
        a.pos = pos;
        a.yaw = yaw;
        a.vel = [0.0; 3];
        a.dead = false;
        a.hp = a.max_hp;
        a.stamina = a.max_stamina;
        a.fp = a.max_fp;
        a.inventory.refill_flasks();
        a.motor = Motor {
            grounded: true,
            ground_y: pos[1],
            fall_peak_y: pos[1],
            ..Motor::default()
        };
        a.lock_target = None;
        a.moves = MoveState::default();
        a.hit = HitState::default();
        crate::combat::status::cure_all(a);
        crate::combat::status::clear_buffs(a, &world.status);
    }
    world.state.death.respawn_in = None;
    world.state.death.safe_pos = pos;
    super::lantern::respawn_enemies(world);
    world.events.push(Event::Life(LifeEvent::Respawned { pos }));
}

/// Recovers the Remnant if the living player's feet are within the pickup radius.
fn recover(world: &mut World) {
    let r = world.locomotion.death.remnant_pickup_radius_m;
    let player = world.player;
    let Some(remnant) = world.state.death.remnant.as_ref() else {
        return;
    };
    let Some(p) = world.actors.get_mut(player) else {
        return;
    };
    let d = [
        remnant.pos[0] - p.pos[0],
        remnant.pos[1] - p.pos[1],
        remnant.pos[2] - p.pos[2],
    ];
    if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] > r * r {
        return;
    }
    p.inventory.glimmer = p.inventory.glimmer.saturating_add(remnant.glimmer);
    let (glimmer, pos) = (remnant.glimmer, remnant.pos);
    world.state.death.remnant = None;
    world
        .events
        .push(Event::Life(LifeEvent::RemnantRecovered { glimmer, pos }));
}
