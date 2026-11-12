//! Swarm explosion (docs/game/05-enemies-and-bosses.md, "Enemy roster", Lamp Mite): an enemy
//! with `explodes` counts the ticks it stays within `contact_m` of its target. At
//! `contact_ticks` it plays its blast move, and it dies when the move ends. Breaking
//! contact (a roll, a few steps) resets the count. The blast is an ordinary move, so it
//! has a telegraph, i-frames dodge it, and the validator checks it.

use super::super::{Awareness, Doing, options};
use crate::ai::perception::dist_xz;
use crate::arena::Id;
use crate::combat::moves::ActiveMove;
use crate::world::World;
use lf_content::enemy::Behavior;

/// One tick of the explosion rule for `id`.
pub fn tick(world: &mut World, id: Id) {
    let Some(a) = world.actors.get(id) else {
        return;
    };
    let Some(b) = a.ai.as_ref() else {
        return;
    };
    let Some(rule) = world.ai.defs[usize::from(b.def)].data.explodes.clone() else {
        return;
    };
    if a.dead {
        return;
    }
    if b.exploding {
        // The blast is done: the mite goes with it.
        if a.moves.current.is_none()
            && let Some(a) = world.actors.get_mut(id)
        {
            a.hp = 0;
            a.dead = true;
            a.vel = [0.0; 3];
        }
        return;
    }
    let in_contact = b.awareness == Awareness::Combat
        && b.target
            .and_then(|t| world.actors.get(t))
            .is_some_and(|t| !t.dead && dist_xz(a.pos, t.pos) <= rule.contact_m);
    let contact = if in_contact {
        b.contact_ticks.saturating_add(1)
    } else {
        0
    };
    let fuse_done = contact >= rule.contact_ticks && options::can_start_move(a);
    let target = b.target;
    if let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
        b.contact_ticks = if fuse_done { 0 } else { contact };
    }
    if !fuse_done {
        return;
    }
    let Some(d) = world
        .moves
        .num_of(&rule.move_id)
        .and_then(|n| world.moves.get(n).map(|d| (n, d.data.total_frames)))
    else {
        return;
    };
    let (num, frames) = d;
    if let Some(a) = world.actors.get_mut(id) {
        a.moves.guard.drop_guard();
        a.moves.current = Some(ActiveMove {
            num,
            frame: 0,
            charge_ticks: 0,
            hold_button: 0,
            free_from: frames,
        });
        a.lock_target = target;
        a.vel[0] = 0.0;
        a.vel[2] = 0.0;
        if let Some(b) = a.ai.as_mut() {
            b.exploding = true;
            b.guarding = false;
            // Holds the think step off the blast until the mite is gone.
            b.doing = Doing::Behavior(Behavior::Wait { dur: frames });
            b.doing_ticks = frames;
        }
    }
}
