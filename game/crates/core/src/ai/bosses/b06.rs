//! B06, the Drowned Choir (WP-074). Source: docs/game/05-enemies-and-bosses.md, "B06 The
//! Drowned Choir". The boss is three ordinary AI actors (the Cantor is the body, the
//! Bellbearer and the Lamenter are required parts; see `lf_content::boss`), and the framework
//! plays their moves. This file adds the three things the framework lacks:
//!
//! 1. **The verse.** When a member falls, the survivors "sing a new verse and gain one move
//!    each". Their options gated on `Phase(2)` are those moves, so the first fall raises the
//!    boss's phase from 1 to 2 (the framework only changes phase by HP threshold, which can't
//!    tell a member's death from damage spread across the three). Every survivor's cooldowns
//!    clear, so the new move can come at once, and the event `CHOIR_VERSE` tells the renderer.
//! 2. **The mend.** The Cantor "heals the others": when `b06_cantor_mend` reaches
//!    `params.mend_frame`, each other living member recovers `mend_heal_percent` of its max
//!    HP. A stagger that breaks the move before that frame stops it, so interrupting the hymn
//!    is the counter, and the kill order matters (the Cantor first starves the others of
//!    healing; the others first leave the Cantor with nothing to heal).
//! 3. **The teleport-stab.** On `params.teleport_frame` of `b06_lam_veilstab` the Lamenter
//!    reappears `teleport_behind_m` past the player, on the line from where she was, facing
//!    them. The stab lands 16 frames later and she stops turning 4 frames before it (the
//!    framework's tracking rule), so the move is fair. A wall behind the player brings her
//!    nearer, never nearer than `teleport_min_m`, and never through the wall.
//!
//! Every number is read from the boss definition's `params` map. State: none of its own;
//! everything is read from the actors' moves and HP, so it is part of the world hash through
//! them.

use crate::arena::Id;
use crate::events::{BossEvent, Event, MechanicEvent};
use crate::math::dmath::{atan2, sqrt};
use crate::world::World;

/// The boss definition this mechanic belongs to.
pub const BOSS_ID: &str = "b06_drowned_choir";
/// Member numbers: the order in the boss definition (the body, then the parts).
const CANTOR: usize = 0;
const LAMENTER: usize = 2;
/// The moves the mechanic watches, by registry ID.
const MEND_MOVE: &str = "b06_cantor_mend";
const VEIL_MOVE: &str = "b06_lam_veilstab";
/// How high above the player's feet the teleport's wall probe runs, in meters (about the
/// waist; a probe at the floor would hit steps and thresholds).
const PROBE_HEIGHT_M: f32 = 1.0;
/// How far short of a wall the Lamenter stands, in meters (her body's radius and a skin).
const WALL_MARGIN_M: f32 = 0.5;

/// One tick of the mechanic, for boss `b` while it fights. Does nothing for another boss.
pub fn tick(world: &mut World, b: usize) {
    let Some(lay) = world.layout.bosses.get(b) else {
        return;
    };
    if lay.def.id != BOSS_ID {
        return;
    }
    verse(world, b);
    mend(world, b);
    veil(world, b);
}

/// The actor of member `i` of boss `b`.
fn member(world: &World, b: usize, i: usize) -> Option<Id> {
    let spawn = *world.layout.bosses.get(b)?.members.get(i)?;
    world.state.ai_enemies.get(usize::from(spawn)).copied()
}

fn down(world: &World, id: Id) -> bool {
    world.actors.get(id).is_none_or(|a| a.dead || a.hp == 0)
}

fn number(world: &World, b: usize, key: &str) -> f32 {
    world.layout.bosses[b]
        .def
        .params
        .get(key)
        .copied()
        .unwrap_or(0.0)
}

fn mechanic_event(world: &mut World, kind: u8, member: usize) {
    world.events.push(Event::Mechanic(MechanicEvent {
        kind,
        index: member as u16,
    }));
}

/// The first fall raises the phase, which gives the survivors their verse move.
fn verse(world: &mut World, b: usize) {
    if world.bosses[b].phase != 1 {
        return;
    }
    let n = world.layout.bosses[b].members.len();
    let Some(fallen) = (0..n).find(|&i| member(world, b, i).is_some_and(|id| down(world, id)))
    else {
        return;
    };
    world.bosses[b].phase = 2;
    for i in 0..n {
        let Some(id) = member(world, b, i) else {
            continue;
        };
        if down(world, id) {
            continue;
        }
        if let Some(brain) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
            brain.phase = 2;
            brain.cooldowns.iter_mut().for_each(|c| *c = 0);
        }
    }
    world.events.push(Event::Boss(BossEvent {
        boss: b as u16,
        kind: BossEvent::PHASE_STARTED,
        phase: 2,
    }));
    mechanic_event(world, MechanicEvent::CHOIR_VERSE, fallen);
}

/// The Cantor's hymn heals the other members when it completes.
fn mend(world: &mut World, b: usize) {
    let Some(cantor) = member(world, b, CANTOR) else {
        return;
    };
    let Some(num) = world.moves.num_of(MEND_MOVE) else {
        return;
    };
    let Some(a) = world.actors.get(cantor) else {
        return;
    };
    let Some(c) = a.moves.current else {
        return;
    };
    if a.dead || c.num != num || f32::from(c.frame) != number(world, b, "mend_frame") {
        return;
    }
    let percent = number(world, b, "mend_heal_percent") as u64;
    for i in 0..world.layout.bosses[b].members.len() {
        if i == CANTOR {
            continue;
        }
        let Some(id) = member(world, b, i) else {
            continue;
        };
        if down(world, id) {
            continue;
        }
        if let Some(t) = world.actors.get_mut(id) {
            let heal = (u64::from(t.max_hp) * percent / 100) as u32;
            let hp = t.hp.saturating_add(heal).min(t.max_hp);
            if hp > t.hp {
                t.hp = hp;
                mechanic_event(world, MechanicEvent::CHOIR_MEND, i);
            }
        }
    }
}

/// The Lamenter reappears just past the player.
fn veil(world: &mut World, b: usize) {
    let Some(lam) = member(world, b, LAMENTER) else {
        return;
    };
    let Some(num) = world.moves.num_of(VEIL_MOVE) else {
        return;
    };
    let Some(a) = world.actors.get(lam) else {
        return;
    };
    let Some(c) = a.moves.current else {
        return;
    };
    if a.dead || c.num != num || f32::from(c.frame) != number(world, b, "teleport_frame") {
        return;
    }
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    let (from, target) = (a.pos, p.pos);
    let (dx, dz) = (target[0] - from[0], target[2] - from[2]);
    let len = sqrt(dx * dx + dz * dz);
    if len < 0.001 {
        return;
    }
    let dir = [dx / len, dz / len];
    let behind = number(world, b, "teleport_behind_m");
    let min = number(world, b, "teleport_min_m");
    // How far the wall behind the player lets her go.
    let reach = world
        .physics
        .raycast(
            [target[0], target[1] + PROBE_HEIGHT_M, target[2]],
            [dir[0], 0.0, dir[1]],
            behind + WALL_MARGIN_M,
        )
        .map_or(behind, |t| t - WALL_MARGIN_M);
    // Past the player when there is room; otherwise beside them on her own side.
    let (ax, az) = if reach >= min {
        (target[0] + dir[0] * reach, target[2] + dir[1] * reach)
    } else {
        (target[0] - dir[0] * min, target[2] - dir[1] * min)
    };
    if let Some(a) = world.actors.get_mut(lam) {
        a.pos = [ax, target[1], az];
        a.vel = [0.0; 3];
        a.yaw = atan2(target[0] - ax, target[2] - az);
    }
    mechanic_event(world, MechanicEvent::CHOIR_TELEPORT, LAMENTER);
}
