//! Lanterns: resting, the respawn point, and respawning enemies. See
//! docs/game/06-progression-and-systems.md, "Lanterns".
//!
//! Resting (`Command::Rest` with a Lantern in reach) restores HP, FP, stamina, and flask
//! charges, respawns the non-boss enemies, sets the respawn point, lights the Lantern
//! (flag `lantern.<id>.lit`), and opens the Lantern menu, which is the `resting` state:
//! `Command::SplitFlasks` applies only while it lasts. The rest ends when the player
//! pushes the stick or presses any button other than `INTERACT`, or dies.
//!
//! Resting also cures every status (meters and effects; buffs run on).
//!
//! Not built yet: the other menu entries (level up, attune, travel, loadouts), and Lantern
//! Ash.

use lf_content::scenario::Dummy;

use crate::actor::stats::{Defense, Stats};
use crate::actor::{Actor, Motor};
use crate::combat::dummy::DummyScript;
use crate::combat::hurtbox::Hurtbox;
use crate::combat::moves::MoveTable;
use crate::combat::poise::PoiseState;
use crate::combat::status::StatusState;
use crate::events::{Event, LifeEvent};
use crate::input::{Command, TickInput, buttons};
use crate::world::World;

use super::flags;

/// The Lantern the player could rest at now: the closest one within the rest radius
/// horizontally and the rest height vertically, as an index into `Layout::lanterns`
/// (the lowest index on a tie).
pub fn in_reach(world: &World) -> Option<usize> {
    let d = world.locomotion.death;
    let p = world.actors.get(world.player)?;
    let r2 = d.lantern_rest_radius_m * d.lantern_rest_radius_m;
    let mut best: Option<(usize, f32)> = None;
    for (i, l) in world.layout.lanterns.iter().enumerate() {
        let (dx, dz) = (l.pos[0] - p.pos[0], l.pos[2] - p.pos[2]);
        let d2 = dx * dx + dz * dz;
        let near = d2 <= r2 && (l.pos[1] - p.pos[1]).abs() <= d.lantern_rest_height_m;
        if near && best.is_none_or(|(_, b)| d2 < b) {
            best = Some((i, d2));
        }
    }
    best.map(|(i, _)| i)
}

/// Tick system 1, first half: ends a rest the player walked away from, then applies
/// `Command::Rest`.
pub fn run(world: &mut World, input: &TickInput) {
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    if p.dead {
        world.state.resting = None;
        return;
    }
    let f = input.frame;
    if f.move_x != 0 || f.move_z != 0 || f.buttons & !buttons::INTERACT != 0 {
        world.state.resting = None;
    }
    if input.commands.contains(&Command::Rest) {
        try_rest(world);
    }
}

/// Rests at the Lantern in reach, if the player is standing still enough to: on the
/// ground and not in the middle of a move. Returns whether they rested.
pub fn try_rest(world: &mut World) -> bool {
    let Some(p) = world.actors.get(world.player) else {
        return false;
    };
    if p.dead || !p.motor.grounded || p.moves.current.is_some() {
        return false;
    }
    match in_reach(world) {
        Some(i) => {
            rest(world, i);
            true
        }
        None => false,
    }
}

/// Rests at the Lantern at index `lantern` of the layout, whatever the player is doing.
pub fn rest(world: &mut World, lantern: usize) {
    let l = world.layout.lanterns[lantern].clone();
    let player = world.player;
    let pos = world.actors.get(player).map_or(l.pos, |p| p.pos);
    if let Some(a) = world.actors.get_mut(player) {
        a.hp = a.max_hp;
        a.stamina = a.max_stamina;
        a.fp = a.max_fp;
        a.inventory.refill_flasks();
        crate::combat::status::cure_all(a);
    }
    world.state.flags.raise(&flags::lantern_lit(&l.id));
    world.state.respawn_lantern = Some(l.id.clone());
    world.state.resting = Some(l.id);
    respawn_enemies(world);
    world.events.push(Event::Life(LifeEvent::Rested {
        lantern: lantern as u16,
        pos,
    }));
}

/// A fresh dummy at its scenario position, at full HP, with its poise, stance, and
/// scripted attack (WP-024). `World::new` rejects a dummy whose attack move is unknown.
pub fn dummy_actor(d: &Dummy, hurtbox: Hurtbox, moves: &MoveTable) -> Actor {
    let script = d.attack.as_ref().and_then(|at| {
        Some(DummyScript {
            move_num: moves.num_of(&at.move_id)?,
            first_tick: u64::from(at.first_tick),
            every_ticks: u64::from(at.every_ticks),
        })
    });
    Actor {
        poise: PoiseState {
            poise: d.poise,
            stance: d.stance,
            unaware: d.unaware,
            ..PoiseState::default()
        },
        script,
        pos: d.pos,
        yaw: d.yaw,
        hp: d.hp,
        max_hp: d.hp,
        stats: Stats {
            defense: Defense::from_data(&d.defense),
            ..Stats::default()
        },
        status: StatusState::for_enemy(&d.resist, d.boss, &d.immune, d.can_go_mad),
        motor: Motor {
            grounded: true,
            ground_y: d.pos[1],
            fall_peak_y: d.pos[1],
            ..Motor::default()
        },
        hurtbox,
        ..Actor::default()
    }
}

/// Puts every non-boss enemy back at its spawn at full HP, inserting any that were
/// removed. Called when the player rests and when they respawn.
///
/// This is the hook WP-030 replaces: once enemies come from the level format it should
/// walk the level's spawn table instead of `Layout::dummies`, skip bosses, and respect
/// flags such as "boss defeated". Keep the signature.
pub fn respawn_enemies(world: &mut World) {
    let hurtbox = Hurtbox::from_locomotion(&world.locomotion);
    for (i, d) in world.layout.dummies.iter().enumerate() {
        let fresh = dummy_actor(d, hurtbox, &world.moves);
        let slot = world
            .state
            .enemies
            .get(i)
            .and_then(|&id| world.actors.get_mut(id));
        match slot {
            Some(a) => *a = fresh,
            None => {
                let id = world.actors.insert(fresh);
                match world.state.enemies.get_mut(i) {
                    Some(h) => *h = id,
                    None => world.state.enemies.push(id),
                }
            }
        }
    }
}
