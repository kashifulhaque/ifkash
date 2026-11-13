//! B05, the Ossuary Keeper (docs/game/05-enemies-and-bosses.md, "B05 Ossuary Keeper").
//!
//! The framework gives the Keeper its gate, phases, and rewards. This module adds what the
//! framework lacks:
//!
//! - **Raise.** The three bone thralls and the two Drowned are boss parts that aren't needed
//!   to win. They lie down (dead) while the fight hasn't started, and the Keeper's `b05_raise`
//!   move stands the fallen thralls up on its `bone_rise` vfx frame ("revives up to three
//!   skeletons from the floor"). The move is locked by cooldown while no thrall has fallen.
//! - **The tide (phase 2).** The water rises 0.4 m: the pool [`POOL`] goes to its
//!   `raised_depth_m` (`pool.<name>.raised`), and a second flag makes it drag a player
//!   wading in it a second time (`world_state::mechanics::before_locomotion`). When the
//!   transition ends the Drowned stand up at the vault's edges.
//! - **The borrowed weapon (phase 2).** `b05_scavenge` takes a fallen thrall's weapon on its
//!   `weapon_taken` vfx frame (flag [`ARMED`]). Until then the borrowed moves are locked; the
//!   scavenge itself is locked while the Keeper is armed or no thrall has fallen.
//! - **The Echo.** Defeat raises the Bone Sentinel Echo's unlock flag, which is how the
//!   Echo is "given by the boss's rewards" (the boss file has no field for a flag).
//!
//! Every number is in data: the moves' frames, the pool's depths, the thralls' HP. The
//! module's names for things in data (move IDs, the pool, the Echo) are below.

use crate::actor::Actor;
use crate::ai::boss::{BossRun, Stage};
use crate::ai::{Awareness, Doing, OptionKind};
use crate::arena::Id;
use crate::combat::guard::GuardState;
use crate::events::{Event, MechanicEvent, ReviveEvent};
use crate::world::World;
use crate::world_state::flags;

/// The Keeper's definition ID (also the key of `boss.<id>.defeated`).
pub const BOSS: &str = "b05_ossuary_keeper";
/// The vault's pool, in the level and in `boss_b05` (`raised_depth_m` is the phase 2 depth).
pub const POOL: &str = "vault_wade";
/// The Echo the defeat unlocks (`data/echoes/echo_bone_sentinel.ron`).
pub const ECHO: &str = "echo_bone_sentinel";
/// Raised while the Keeper holds a thrall's weapon.
pub const ARMED: &str = "boss.b05.armed";
/// Raised once the Drowned have stood up this fight.
pub const DROWNED_UP: &str = "boss.b05.drowned";
/// Raised while the tide is in (phase 2). Cleared with the fight.
pub const TIDE: &str = "boss.b05.tide";

const RAISE: &str = "b05_raise";
const SCAVENGE: &str = "b05_scavenge";
const BORROWED: [&str; 2] = ["b05_borrow_thrust", "b05_borrow_bash"];
/// The vfx names that mark a move's key frame in data.
const VFX_RISE: &str = "bone_rise";
const VFX_TAKE: &str = "weapon_taken";
/// Name prefixes of the parts, in `b05_ossuary_keeper.ron`.
const THRALL: &str = "b05_bone_thrall";
const DROWNED: &str = "b05_vault_drowned";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Keeper,
    Thrall,
    Drowned,
}

/// The actor of boss `b`'s member `i` (0 is the body).
fn member(world: &World, b: usize, i: usize) -> Option<Id> {
    let spawn = *world.layout.bosses.get(b)?.members.get(i)?;
    world.state.ai_enemies.get(usize::from(spawn)).copied()
}

fn is_down(a: &Actor) -> bool {
    a.dead || a.hp == 0
}

fn role(world: &World, id: Id) -> Option<Role> {
    let brain = world.actors.get(id)?.ai.as_ref()?;
    let name = world.ai.get(brain.def)?.data.id.as_str();
    if name.starts_with(THRALL) {
        Some(Role::Thrall)
    } else if name.starts_with(DROWNED) {
        Some(Role::Drowned)
    } else {
        Some(Role::Keeper)
    }
}

/// The members of boss `b` with `role`, in order.
fn members_of(world: &World, b: usize, r: Role) -> Vec<Id> {
    (0..world.layout.bosses[b].members.len())
        .filter_map(|i| member(world, b, i))
        .filter(|&id| role(world, id) == Some(r))
        .collect()
}

fn some_down(world: &World, ids: &[Id]) -> bool {
    ids.iter()
        .any(|&id| world.actors.get(id).is_some_and(is_down))
}

fn mechanic(world: &mut World, kind: u8, index: u16) {
    world
        .events
        .push(Event::Mechanic(MechanicEvent { kind, index }));
}

/// The Keeper's rules for every boss with [`BOSS`]; `super::after` calls it once a tick,
/// in every stage.
pub fn run(world: &mut World) {
    for b in 0..world.layout.bosses.len() {
        if world.layout.bosses[b].def.id == BOSS {
            step(world, b);
        }
    }
}

/// The boss-specific step for boss `b`, once per tick.
fn step(world: &mut World, b: usize) {
    match world.bosses[b].stage {
        Stage::Dormant | Stage::Intro { .. } => rest(world, b),
        Stage::Fight => fight(world, b),
        Stage::Defeated => settle(world, b),
    }
}

/// Clears the fight's flags and the water.
fn clear_flags(world: &mut World) {
    for key in [ARMED, DROWNED_UP, TIDE] {
        world.state.flags.clear(key);
    }
    world.state.flags.clear(&flags::pool_raised(POOL));
    world.state.flags.clear(&flags::pool_drag(POOL));
}

/// Before the fight: the adds lie down, and nothing of an earlier fight is left.
fn rest(world: &mut World, b: usize) {
    clear_flags(world);
    for i in 1..world.layout.bosses[b].members.len() {
        if let Some(a) = member(world, b, i).and_then(|id| world.actors.get_mut(id))
            && !is_down(a)
        {
            a.hp = 0;
            a.dead = true;
            a.moves.current = None;
        }
    }
}

/// After the fight: the water goes back, and the Echo is the player's.
fn settle(world: &mut World, _b: usize) {
    clear_flags(world);
    let unlock = world
        .echo_table
        .echoes
        .iter()
        .find(|e| e.def.id == ECHO)
        .map(|e| e.def.unlock_flag.clone());
    if let Some(flag) = unlock
        && !world.state.flags.is_set(&flag)
    {
        world.state.flags.raise(&flag);
    }
}

/// The frame of the first vfx called `name` in the move, or `None`.
fn vfx_frame(world: &World, move_id: &str, name: &str) -> Option<u16> {
    let num = world.moves.num_of(move_id)?;
    world
        .moves
        .get(num)?
        .data
        .vfx
        .iter()
        .find(|(_, n)| n == name)
        .map(|&(f, _)| f)
}

/// The Keeper's current move and frame, if it is in a move.
fn keeper_move(world: &World, keeper: Id) -> Option<(crate::combat::moves::MoveNum, u16)> {
    let c = world.actors.get(keeper)?.moves.current?;
    Some((c.num, c.frame))
}

fn fight(world: &mut World, b: usize) {
    let run: BossRun = world.bosses[b];
    let Some(keeper) = member(world, b, 0) else {
        return;
    };
    // The tide comes in when phase 2 is on its way, and stays until the fight ends.
    let phase2 = run.phase >= 2 || (run.transition_left > 0 && run.transition_to >= 2);
    if phase2 && !world.state.flags.is_set(TIDE) {
        world.state.flags.raise(TIDE);
        world.state.flags.raise(&flags::pool_raised(POOL));
        world.state.flags.raise(&flags::pool_drag(POOL));
        mechanic(world, MechanicEvent::B05_TIDE_RISING, 0);
    }
    let thralls = members_of(world, b, Role::Thrall);
    let drowned = members_of(world, b, Role::Drowned);
    if world.actors.get(keeper).is_some_and(|a| !is_down(a)) {
        // Key frames of the Keeper's moves.
        if let Some((num, frame)) = keeper_move(world, keeper) {
            if world.moves.num_of(RAISE) == Some(num)
                && vfx_frame(world, RAISE, VFX_RISE) == Some(frame)
            {
                let mut raised = 0u16;
                for &id in &thralls {
                    if world.actors.get(id).is_some_and(is_down) {
                        stand_up(world, id);
                        raised += 1;
                    }
                }
                if raised > 0 {
                    mechanic(world, MechanicEvent::B05_RAISED, raised);
                }
            }
            if world.moves.num_of(SCAVENGE) == Some(num)
                && vfx_frame(world, SCAVENGE, VFX_TAKE) == Some(frame)
                && some_down(world, &thralls)
                && !world.state.flags.is_set(ARMED)
            {
                world.state.flags.raise(ARMED);
                mechanic(world, MechanicEvent::B05_WEAPON_TAKEN, 0);
            }
        }
    }
    // The Drowned stand up from the vault's edges once the transition move is over.
    if run.phase >= 2 && run.transition_left == 0 && !world.state.flags.is_set(DROWNED_UP) {
        world.state.flags.raise(DROWNED_UP);
        let mut up = 0u16;
        for &id in &drowned {
            if world.actors.get(id).is_some_and(is_down) {
                stand_up(world, id);
                up += 1;
            }
        }
        mechanic(world, MechanicEvent::B05_DROWNED_JOIN, up);
    }
    // Lock the options whose condition doesn't hold.
    let armed = world.state.flags.is_set(ARMED);
    let fallen = some_down(world, &thralls);
    lock_options(world, keeper, |id| {
        if id == RAISE {
            !fallen
        } else if id == SCAVENGE {
            armed || !fallen
        } else if BORROWED.contains(&id) {
            !armed
        } else {
            false
        }
    });
}

/// Keeps every option of `keeper` whose move `locked` returns true for on cooldown. The
/// cooldown counts down once a tick after the think, so holding it at 1 each tick keeps the
/// option out of `options::eligible`; the tick after it stops, the option is free.
fn lock_options(world: &mut World, keeper: Id, locked: impl Fn(&str) -> bool) {
    let Some(brain) = world.actors.get(keeper).and_then(|a| a.ai.as_ref()) else {
        return;
    };
    let hold: Vec<usize> = world
        .ai
        .def(brain.def)
        .options
        .iter()
        .enumerate()
        .filter_map(|(i, o)| match o.kind {
            OptionKind::Move { num, .. } => world
                .moves
                .get(num)
                .filter(|d| locked(d.data.id.as_str()))
                .map(|_| i),
            OptionKind::Behavior(_) => None,
        })
        .collect();
    if let Some(brain) = world.actors.get_mut(keeper).and_then(|a| a.ai.as_mut()) {
        for i in hold {
            if let Some(c) = brain.cooldowns.get_mut(i)
                && *c == 0
            {
                *c = 1;
            }
        }
    }
}

/// Stands a fallen add up at full HP where it lies, in the fight (the revive rule's way,
/// `ai::special::revive::tick_dead`, without the countdown).
fn stand_up(world: &mut World, id: Id) {
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    let hp = a.max_hp;
    a.dead = false;
    a.hp = hp;
    a.vel = [0.0; 3];
    a.moves.interrupt();
    a.moves.guard = GuardState::default();
    a.poise.accum = 0;
    a.poise.stance_dmg = 0;
    a.poise.kneel = 0;
    a.poise.parried = 0;
    a.poise.unaware = false;
    a.hit.prev = None;
    crate::combat::status::cure_all(a);
    if let Some(brain) = a.ai.as_mut() {
        brain.dead_ticks = 0;
        brain.last_hp = hp;
        brain.doing = Doing::Nothing;
        brain.doing_ticks = 0;
        brain.guarding = false;
        brain.path.clear();
        brain.path_ticket = None;
        brain.cooldowns.iter_mut().for_each(|c| *c = 0);
        brain.awareness = Awareness::Combat;
        brain.state_ticks = 0;
    }
    world
        .events
        .push(Event::Revive(ReviveEvent { actor: id, hp }));
}
