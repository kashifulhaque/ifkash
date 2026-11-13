//! B10, the Bellwright: his beat, his bell's toll, and the cracking ice. See
//! docs/game/05-enemies-and-bosses.md, "B10 The Bellwright", and
//! `data/bosses/b10_the_bellwright.ron`, where every number below lives (`params`).
//!
//! # The beat
//!
//! The fight runs on a beat: one every `beat_ticks_p1` ticks, and one every `beat_ticks_p2` once
//! the chain snaps (phase 2). The grid starts when the fight does and starts again, at the new
//! speed, when the phase 2 transition ends. Each beat sends [`MechanicEvent::BELL_BEAT`] with its
//! place in the bar (0 to `toll_every_beats - 1`), which the music and the renderer follow.
//!
//! The Bellwright is an ordinary AI actor, and the framework lets him pick his next move whenever
//! the last one is done. This module makes him pick only on the grid: while he is free, it holds
//! him in a one-tick `Wait` until `slot_offset_ticks` after a beat, and the tick it comes he is
//! let go to decide. So a move is picked at beat + `slot_offset_ticks`, and the moves in
//! `data/moves/bosses/b10.ron` are laid out so that a hit on frame F (which lands F - 1 ticks after
//! the pick) lands on a beat: the first hit is on frame `period - offset + 1`, later hits come one
//! beat apart, and `total_frames` is a whole number of beats. Measured, not derived: the test
//! `he_starts_every_move_on_the_slot_so_each_hit_lands_on_a_beat` fails if the offset drifts. A walk (`Approach`) or `Wait` that ends between slots is held on until the next one,
//! so he is always on the grid. The hold never touches a move in progress, a stagger, or a phase
//! transition.
//!
//! # The toll
//!
//! Every `toll_every_beats` beats (not counting the first, which is the fight's start) the bell
//! tolls. [`MechanicEvent::BELL_TOLL_WARNED`] sends `toll_warn_ticks` earlier, when the bell
//! winds up. On the toll a Rime shockwave starts where the Bellwright stands and runs out at
//! `toll_speed_m_per_tick`, to `toll_max_m`. When its edge passes the player, it hits them for
//! `toll_damage_percent` of their max HP and builds Rime, unless they are in the air (a jump) or
//! inside a roll's i-frames: those are the two counters the spec gives. It can't be blocked. It is
//! not a move, so no hitbox of his: the toll comes whatever he is doing.
//!
//! # The ice
//!
//! `ice_zones` patches of ice crack in turn: patch `i` cracks at toll `ice_first_crack_toll + i *
//! ice_crack_step_tolls`, and breaks `ice_break_after_tolls` tolls later (the spec's "cracked ice
//! zones break after two bell tolls"). A broken patch is freezing water: anyone standing in it
//! loses `water_damage_percent` of their max HP every `water_interval_ticks`. It stays broken
//! until the fight resets. The patch states follow from the toll count alone ([`patch`]), so the
//! state is small.
//!
//! # State
//!
//! [`B10State`], in `MechState::b10`: the grid's start, the beats and tolls rung, and the
//! shockwave's start and origin. It goes back to rest when the fight isn't on, so a reset or a
//! defeat clears the ice, and it is left out of the world hash while at rest.

use lf_content::BossDef;
use lf_content::moves::StatusKind;
use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::ai::{Doing, Token};
use crate::arena::Id;
use crate::combat::dodge;
use crate::combat::hurtbox::Hurtbox;
use crate::combat::status;
use crate::events::{Event, MechanicEvent};
use crate::math::dmath::sqrt;
use crate::world::World;

/// The boss definition this module drives.
pub const BOSS_ID: &str = "b10_the_bellwright";
/// The body's member number: it is the only member.
const BODY: usize = 0;

/// One patch of ice's state, from the toll count (see [`patch`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ice {
    /// Not yet cracked.
    Sound,
    /// Cracked: it breaks at a later toll.
    Cracked,
    /// Broken: freezing water.
    Broken,
}

/// B10's mechanic state. Simulation state: part of the world hash while it isn't at rest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct B10State {
    /// The tick the beat grid began at, 0 while the fight isn't on.
    pub from: u64,
    /// The boss phase the grid was set up for (0 while the fight isn't on).
    pub grid_phase: u8,
    /// Beats rung since the grid began (the one at the start counts).
    pub beats: u32,
    /// Tolls rung this fight; the ice reads it.
    pub tolls: u32,
    /// The tick the latest shockwave started at, 0 for none running.
    pub toll_from: u64,
    /// Where it started: x and z in world axes.
    pub toll_at: [f32; 2],
}

impl B10State {
    /// At rest: no fight on. Such a state is left out of the world hash.
    pub fn is_rest(&self) -> bool {
        *self == Self::default()
    }
}

/// The numbers from `BossDef::params`.
struct Params {
    beat_p1: u64,
    beat_p2: u64,
    slot_offset: u64,
    toll_every: u32,
    toll_warn: u64,
    toll_speed: f32,
    toll_max: f32,
    /// Percent in thousandths of a percent, so the damage rounds up with integers.
    toll_damage_milli: u64,
    toll_rime_milli: u64,
    body: Hurtbox,
    zones: Vec<[f32; 2]>,
    ice_radius: f32,
    first_crack: u32,
    crack_step: u32,
    break_after: u32,
    water_damage_milli: u64,
    water_interval: u64,
}

/// Percent in thousandths of a percent.
const PERCENT_MILLI: f32 = 1000.0;
const HUNDRED_PERCENT_MILLI: u64 = 100_000;

impl Params {
    /// Reads the numbers. `None` if any is missing (the boss then has no mechanic).
    fn read(def: &BossDef) -> Option<Self> {
        let get = |key: &str| def.params.get(key).copied();
        let count = get("ice_zones")? as usize;
        let mut zones = Vec::with_capacity(count);
        for i in 0..count {
            zones.push([get(&format!("ice{i}_x_m"))?, get(&format!("ice{i}_z_m"))?]);
        }
        Some(Self {
            beat_p1: get("beat_ticks_p1")?.max(1.0) as u64,
            beat_p2: get("beat_ticks_p2")?.max(1.0) as u64,
            slot_offset: get("slot_offset_ticks")?.max(0.0) as u64,
            toll_every: get("toll_every_beats")?.max(1.0) as u32,
            toll_warn: get("toll_warn_ticks")?.max(0.0) as u64,
            toll_speed: get("toll_speed_m_per_tick")?,
            toll_max: get("toll_max_m")?,
            toll_damage_milli: (get("toll_damage_percent")?.max(0.0) * PERCENT_MILLI) as u64,
            toll_rime_milli: get("toll_rime_milli")?.max(0.0) as u64,
            body: Hurtbox {
                radius: get("body_radius_m")?,
                half_height: get("body_half_height_m")?,
            },
            zones,
            ice_radius: get("ice_radius_m")?,
            first_crack: get("ice_first_crack_toll")?.max(0.0) as u32,
            crack_step: get("ice_crack_step_tolls")?.max(0.0) as u32,
            break_after: get("ice_break_after_tolls")?.max(0.0) as u32,
            water_damage_milli: (get("water_damage_percent")?.max(0.0) * PERCENT_MILLI) as u64,
            water_interval: get("water_interval_ticks")?.max(1.0) as u64,
        })
    }

    /// The beat's length in the boss phase `phase`.
    fn period(&self, phase: u8) -> u64 {
        if phase >= 2 {
            self.beat_p2
        } else {
            self.beat_p1
        }
    }

    /// The toll at which patch `i` cracks.
    fn cracks_at(&self, i: usize) -> u32 {
        self.first_crack + self.crack_step * i as u32
    }
}

/// What state patch `i` is in after `tolls` tolls.
pub fn patch(tolls: u32, cracks_at: u32, break_after: u32) -> Ice {
    if tolls >= cracks_at.saturating_add(break_after) {
        Ice::Broken
    } else if tolls >= cracks_at {
        Ice::Cracked
    } else {
        Ice::Sound
    }
}

/// The body actor of boss `b`.
fn body_of(world: &World, b: usize) -> Option<Id> {
    let spawn = *world.layout.bosses.get(b)?.members.get(BODY)?;
    world.state.ai_enemies.get(usize::from(spawn)).copied()
}

fn is_down(a: &Actor) -> bool {
    a.dead || a.hp == 0
}

fn mechanic(world: &mut World, kind: u8, index: usize) {
    world.events.push(Event::Mechanic(MechanicEvent {
        kind,
        index: index as u16,
    }));
}

/// One tick of B10's mechanic, for every placed B10 (the hook is [`super::run`]).
pub fn run(world: &mut World) {
    for b in 0..world.bosses.len() {
        let Some(lay) = world.layout.bosses.get(b) else {
            continue;
        };
        if lay.def.id != BOSS_ID {
            continue;
        }
        let Some(p) = Params::read(&lay.def) else {
            continue;
        };
        step(world, b, &p);
    }
}

fn step(world: &mut World, b: usize, p: &Params) {
    let Some(body) = body_of(world, b) else {
        return;
    };
    // The giant's hurtbox (the default capsule is a person's).
    if let Some(a) = world.actors.get_mut(body) {
        a.hurtbox = p.body;
    }
    let run = world.bosses[b];
    if run.stage != Stage::Fight {
        // Dormant, in the intro, or defeated: no beat, and the ice is whole.
        world.mech.b10 = B10State::default();
        return;
    }
    if run.transition_left > 0 {
        // The chain is snapping: the Bellwright is invulnerable, and the beat waits.
        return;
    }
    let mut st = world.mech.b10;
    let tick = world.tick.max(1);
    if st.from == 0 {
        st.from = tick;
        st.grid_phase = run.phase;
        st.beats = 0;
    } else if st.grid_phase != run.phase {
        // The chain snapped: the grid starts again at the new speed.
        st.from = tick;
        st.grid_phase = run.phase;
        st.beats = 0;
        if run.phase >= 2 {
            mechanic(world, MechanicEvent::BELL_CHAIN_SNAPPED, b);
        }
    }
    let period = p.period(st.grid_phase);
    let elapsed = tick - st.from;
    let cycle = period * u64::from(p.toll_every);

    // The bell winds up, then tolls.
    if (elapsed + p.toll_warn).is_multiple_of(cycle) {
        mechanic(world, MechanicEvent::BELL_TOLL_WARNED, b);
    }
    if elapsed.is_multiple_of(period) {
        let in_bar = (st.beats % p.toll_every) as usize;
        mechanic(world, MechanicEvent::BELL_BEAT, in_bar);
        st.beats += 1;
        if in_bar == 0 && elapsed > 0 {
            toll(world, b, body, p, &mut st, tick);
        }
    }
    shockwave(world, b, p, &mut st, tick);
    water(world, b, p, &st, elapsed);
    world.mech.b10 = st;
    hold_to_slot(world, body, elapsed, period, p.slot_offset);
}

/// The bell tolls: the shockwave starts at the Bellwright, and the ice moves on.
fn toll(world: &mut World, b: usize, body: Id, p: &Params, st: &mut B10State, tick: u64) {
    let at = world.actors.get(body).map_or([0.0; 3], |a| a.pos);
    st.toll_from = tick;
    st.toll_at = [at[0], at[2]];
    st.tolls += 1;
    mechanic(world, MechanicEvent::BELL_TOLLED, b);
    for i in 0..p.zones.len() {
        let was = patch(st.tolls - 1, p.cracks_at(i), p.break_after);
        let now = patch(st.tolls, p.cracks_at(i), p.break_after);
        if was != now {
            let kind = match now {
                Ice::Cracked => MechanicEvent::ICE_CRACKED,
                _ => MechanicEvent::ICE_BROKE,
            };
            mechanic(world, kind, i);
        }
    }
}

/// Moves the shockwave on a tick, and hits the player if its edge passed them.
fn shockwave(world: &mut World, b: usize, p: &Params, st: &mut B10State, tick: u64) {
    if st.toll_from == 0 {
        return;
    }
    let e = tick - st.toll_from;
    let edge = e as f32 * p.toll_speed;
    if edge > p.toll_max {
        st.toll_from = 0;
        return;
    }
    let behind = (e as f32 - 1.0) * p.toll_speed;
    let Some(player) = world.actors.get(world.player) else {
        return;
    };
    let (dx, dz) = (player.pos[0] - st.toll_at[0], player.pos[2] - st.toll_at[1]);
    let d = sqrt(dx * dx + dz * dz);
    // The edge passes the player this tick: it was short of them, and now it is not.
    if e == 0 || d > edge || d <= behind {
        return;
    }
    // Jumped (in the air) or rolled (i-frames): the shockwave runs under or past them.
    if is_down(player) || !player.motor.grounded || dodge::is_invulnerable(&player.moves) {
        return;
    }
    let player_id = world.player;
    hurt(world, player_id, p.toll_damage_milli);
    status::add_buildup(world, player_id, StatusKind::Rime, p.toll_rime_milli);
    mechanic(world, MechanicEvent::BELL_TOLL_HIT, b);
}

/// Freezing water burns the player who stands in a broken patch.
fn water(world: &mut World, b: usize, p: &Params, st: &B10State, elapsed: u64) {
    if st.tolls == 0 || elapsed == 0 || !elapsed.is_multiple_of(p.water_interval) {
        return;
    }
    let Some(home) = world
        .layout
        .bosses
        .get(b)
        .and_then(|l| l.members.first())
        .and_then(|&s| world.layout.enemies.get(usize::from(s)))
        .map(|s| s.pos)
    else {
        return;
    };
    let Some(player) = world.actors.get(world.player) else {
        return;
    };
    if is_down(player) {
        return;
    }
    let pos = player.pos;
    let Some(i) = (0..p.zones.len()).find(|&i| {
        patch(st.tolls, p.cracks_at(i), p.break_after) == Ice::Broken && {
            let (dx, dz) = (
                pos[0] - home[0] - p.zones[i][0],
                pos[2] - home[2] - p.zones[i][1],
            );
            sqrt(dx * dx + dz * dz) <= p.ice_radius
        }
    }) else {
        return;
    };
    let player_id = world.player;
    hurt(world, player_id, p.water_damage_milli);
    mechanic(world, MechanicEvent::ICE_WATER_BURNED, i);
}

/// Takes `milli` thousandths of a percent of the actor's max HP, rounded up and at least 1. The
/// status tick marks the player dead at 0 HP, as for any damage over time.
fn hurt(world: &mut World, id: Id, milli: u64) {
    if let Some(a) = world.actors.get_mut(id) {
        let loss = (u64::from(a.max_hp) * milli)
            .div_ceil(HUNDRED_PERCENT_MILLI)
            .max(1);
        a.hp = a.hp.saturating_sub(u32::try_from(loss).unwrap_or(u32::MAX));
    }
}

/// Holds a free Bellwright until the next slot, so that he starts a move only `offset` ticks after
/// a beat. `elapsed` is the ticks since the grid began.
fn hold_to_slot(world: &mut World, body: Id, elapsed: u64, period: u64, offset: u64) {
    let Some(a) = world.actors.get_mut(body) else {
        return;
    };
    if is_down(a) || a.moves.current.is_some() {
        return;
    }
    let Some(brain) = a.ai.as_mut() else {
        return;
    };
    let at_slot = elapsed % period == offset % period;
    match (brain.doing, at_slot) {
        // A move's `Doing` outlives the move by a tick or so: it is free once the move is gone.
        (Doing::Attack { .. }, true) => {
            brain.doing = Doing::Nothing;
            brain.doing_ticks = 0;
            brain.token = Token::None;
            brain.token_target = None;
        }
        (Doing::Attack { .. }, false) => hold(brain),
        (Doing::Nothing, false) => hold(brain),
        (Doing::Nothing, true) => {}
        // A walk or wait ends at the slot: the AI thinks before it counts a behavior down, so it
        // is free to decide this very tick when its ticks are 0.
        (Doing::Behavior(_), true) => brain.doing_ticks = 0,
        (Doing::Behavior(_), false) => brain.doing_ticks = brain.doing_ticks.max(2),
    }
}

/// Keeps a free brain waiting this tick.
fn hold(brain: &mut crate::ai::AiState) {
    brain.doing = Doing::Behavior(lf_content::enemy::Behavior::Wait { dur: 2 });
    brain.doing_ticks = 2;
    brain.token = Token::None;
    brain.token_target = None;
}
