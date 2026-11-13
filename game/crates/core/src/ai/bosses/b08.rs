//! B08, the Forgeheart Colossus: its legs, its kneel, and its lava. See
//! docs/game/05-enemies-and-bosses.md, "B08 Forgeheart Colossus", and
//! `data/bosses/b08_forgeheart_colossus.ron`, where every number below lives (`params`).
//!
//! # The legs
//!
//! The boss has two parts, its legs, that the framework spawns as ordinary AI actors. Each has
//! a stance meter and no attacks. [`run`] does four things with them, every tick:
//!
//! 1. **Size and place.** The body and the legs get hurtboxes the size of a colossus (the
//!    default capsule is a person's), and each leg is put at its side of the body, turned with
//!    it, so the legs follow it wherever it walks. A reset rebuilds the actors at their spawn;
//!    this puts them right again on the next tick.
//! 2. **Pass hits on.** The legs take hits like the body does. A leg's lost HP is taken off the
//!    body instead and the leg is healed, so a hit on a leg hurts the Colossus. (Not while the
//!    body plays a phase transition, which makes it invulnerable.)
//! 3. **Kneel.** A leg whose stance broke makes the Colossus kneel for the same 60 ticks: the
//!    body's own `kneel` is set, which opens the riposte (`combat::crit::kind_against`), its
//!    move is interrupted, and it plays `b08_kneel` so the renderer shows it. The leg buckles
//!    (`b08_leg_buckle`). The leg's own kneel is cleared at once, so only the body can be
//!    riposted, and a leg's stance can't be worked on while the Colossus is down, so the
//!    kneel can't be chained.
//!
//! # The lava
//!
//! When phase 2 starts (the heart ignites), lava creeps in from the four walls of the forge:
//! `lava_m_per_100_ticks` meters of depth per 100 ticks, up to `lava_max_m`, which leaves the
//! middle of the arena dry. Anyone standing in it loses `lava_damage_percent` of their max HP
//! every `lava_interval_ticks`. The tick it began at is `MechState::b08_lava_from`, the one
//! piece of state this module keeps (hashed with the world mechanics); it goes back to 0 when
//! the fight isn't on, so a reset or a defeat dries the forge.

use lf_content::BossDef;

use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::ai::{Doing, Token};
use crate::arena::Id;
use crate::combat::hitbox::Pose;
use crate::combat::hurtbox::Hurtbox;
use crate::combat::moves::ActiveMove;
use crate::events::{Event, MechanicEvent};
use crate::world::World;

/// The boss definition this module drives.
pub const BOSS_ID: &str = "b08_forgeheart_colossus";
/// The move the body plays while it kneels, and the one a broken leg plays.
const KNEEL_MOVE: &str = "b08_kneel";
const BUCKLE_MOVE: &str = "b08_leg_buckle";
/// Member indexes in `BossLayout::members`: the body, then the two legs, left first.
const BODY: usize = 0;
const LEG_L: usize = 1;
const LEG_R: usize = 2;
/// Percent in thousandths of a percent, so the damage rounds up with integers.
const PERCENT_MILLI: u64 = 1000;
const HUNDRED_PERCENT_MILLI: u64 = 100 * PERCENT_MILLI;

/// The numbers from `BossDef::params`.
struct Params {
    leg_x: f32,
    leg_z: f32,
    body: Hurtbox,
    leg: Hurtbox,
    /// How far each wall's lava edge is from the body's spawn: west, east, south, north (world
    /// axes: -x, +x, -z, +z).
    edges: [f32; 4],
    /// Lava depth gained per tick, in meters.
    lava_per_tick: f32,
    lava_max: f32,
    /// A burn takes this many thousandths of a percent of the player's max HP.
    lava_damage_milli: u64,
    lava_interval: u64,
}

impl Params {
    /// Reads the numbers. `None` if any is missing (the boss then has no mechanic).
    fn read(def: &BossDef) -> Option<Self> {
        let get = |key: &str| def.params.get(key).copied();
        Some(Self {
            leg_x: get("leg_x_m")?,
            leg_z: get("leg_z_m")?,
            body: Hurtbox {
                radius: get("body_radius_m")?,
                half_height: get("body_half_height_m")?,
            },
            leg: Hurtbox {
                radius: get("leg_radius_m")?,
                half_height: get("leg_half_height_m")?,
            },
            edges: [
                get("lava_edge_w_m")?,
                get("lava_edge_e_m")?,
                get("lava_edge_s_m")?,
                get("lava_edge_n_m")?,
            ],
            lava_per_tick: get("lava_m_per_100_ticks")? / 100.0,
            lava_max: get("lava_max_m")?,
            lava_damage_milli: (get("lava_damage_percent")?.max(0.0) * PERCENT_MILLI as f32) as u64,
            lava_interval: (get("lava_interval_ticks")?.max(1.0)) as u64,
        })
    }
}

/// The member actor `i` of boss `b`.
fn member(world: &World, b: usize, i: usize) -> Option<Id> {
    let spawn = *world.layout.bosses.get(b)?.members.get(i)?;
    world.state.ai_enemies.get(usize::from(spawn)).copied()
}

fn is_down(a: &Actor) -> bool {
    a.dead || a.hp == 0
}

/// The fight of boss `b` just reset (`ai::boss::reset`): the forge dries now, not on the next
/// tick, so the lava never outlives the reset it belongs to.
pub fn reset(world: &mut World, b: usize) {
    if world.layout.bosses[b].def.id == BOSS_ID {
        world.mech.b08_lava_from = 0;
    }
}

/// One tick of B08's mechanic, for every placed B08 (the hook is [`super::run`]).
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
        tick(world, b, &p);
    }
}

fn tick(world: &mut World, b: usize, p: &Params) {
    let (Some(body), Some(left), Some(right)) = (
        member(world, b, BODY),
        member(world, b, LEG_L),
        member(world, b, LEG_R),
    ) else {
        return;
    };
    place(world, [body, left, right], p);
    let stage = world.bosses[b].stage;
    if stage != Stage::Fight {
        // Dormant, in the intro, or defeated: the forge is dry.
        world.mech.b08_lava_from = 0;
        return;
    }
    let legs = [left, right];
    let invulnerable = world.bosses[b].transition_left > 0;
    // Hits on the legs go to the body, unless the body is invulnerable.
    for leg in legs {
        pass_hits_on(world, body, leg, invulnerable);
    }
    // A broken leg kneels the body.
    let mut down = world.actors.get(body).is_none_or(|a| a.poise.kneel > 0);
    for (i, leg) in legs.into_iter().enumerate() {
        let broke = world
            .actors
            .get_mut(leg)
            .map(|a| {
                let ticks = a.poise.kneel;
                a.poise.kneel = 0;
                a.moves.guard.stagger = 0;
                ticks
            })
            .unwrap_or(0);
        if broke > 0 && !down && !invulnerable {
            kneel(world, body, leg, i, broke);
            down = true;
        }
        if down && let Some(a) = world.actors.get_mut(leg) {
            // No chaining: a leg's stance waits until the Colossus is back on its feet.
            a.poise.stance_dmg = 0;
        }
    }
    lava(world, b, p);
}

/// Sizes the actors' hurtboxes and puts the legs at the body's sides. `ids` is the body, the
/// left leg, and the right leg.
fn place(world: &mut World, ids: [Id; 3], p: &Params) {
    let Some(at) = world.actors.get_mut(ids[0]).map(|a| {
        a.hurtbox = p.body;
        Pose {
            pos: a.pos,
            yaw: a.yaw,
        }
    }) else {
        return;
    };
    // The body's left is -x in its own space (+x is right, see `combat::hitbox`).
    for (leg, side) in [(ids[1], -1.0), (ids[2], 1.0)] {
        let spot = at.to_world([side * p.leg_x, 0.0, p.leg_z]);
        if let Some(a) = world.actors.get_mut(leg) {
            a.hurtbox = p.leg;
            a.pos = spot;
            a.yaw = at.yaw;
            a.vel = [0.0; 3];
            a.motor.ground_y = spot[1];
            a.motor.fall_peak_y = spot[1];
        }
    }
}

/// Moves the HP a leg lost since the last tick to the body, and heals the leg. With
/// `invulnerable`, the loss is only undone.
fn pass_hits_on(world: &mut World, body: Id, leg: Id, invulnerable: bool) {
    let lost = match world.actors.get_mut(leg) {
        Some(a) if !a.dead && a.hp < a.max_hp => {
            let lost = a.max_hp - a.hp;
            a.hp = a.max_hp;
            lost
        }
        _ => return,
    };
    if invulnerable {
        return;
    }
    if let Some(a) = world.actors.get_mut(body)
        && !is_down(a)
    {
        a.hp = a.hp.saturating_sub(lost);
    }
}

/// The leg `leg_i` broke: the body kneels for `ticks`, and the leg buckles.
fn kneel(world: &mut World, body: Id, leg: Id, leg_i: usize, ticks: u8) {
    let play = |world: &World, id: &str| {
        let num = world.moves.num_of(id)?;
        let d = world.moves.get(num)?;
        let (total, free) = (d.data.total_frames, d.data.cancel_any_from);
        Some(ActiveMove {
            num,
            frame: 0,
            charge_ticks: 0,
            hold_button: 0,
            free_from: free.unwrap_or(total),
        })
    };
    let (kneel_move, buckle_move) = (play(world, KNEEL_MOVE), play(world, BUCKLE_MOVE));
    if let Some(a) = world.actors.get_mut(body) {
        if is_down(a) {
            return;
        }
        a.moves.interrupt();
        a.vel[0] = 0.0;
        a.vel[2] = 0.0;
        a.poise.kneel = ticks;
        a.poise.accum = 0;
        a.moves.current = kneel_move;
        if let (Some(m), Some(brain)) = (kneel_move, a.ai.as_mut()) {
            brain.doing = Doing::Attack {
                option: 0,
                move_num: m.num,
            };
            brain.doing_ticks = 0;
            brain.token = Token::None;
            brain.token_target = None;
        }
    }
    if let Some(a) = world.actors.get_mut(leg) {
        a.moves.interrupt();
        a.moves.current = buckle_move;
    }
    world.events.push(Event::Mechanic(MechanicEvent {
        kind: MechanicEvent::COLOSSUS_LEG_BROKE,
        index: leg_i as u16,
    }));
}

/// Phase 2's creeping lava: starts it, and burns a player standing in it.
fn lava(world: &mut World, b: usize, p: &Params) {
    let run = world.bosses[b];
    if run.phase < 2 || run.transition_left > 0 {
        return;
    }
    if world.mech.b08_lava_from == 0 {
        world.mech.b08_lava_from = world.tick.max(1);
        world.events.push(Event::Mechanic(MechanicEvent {
            kind: MechanicEvent::COLOSSUS_LAVA_STARTED,
            index: b as u16,
        }));
    }
    let elapsed = world.tick.saturating_sub(world.mech.b08_lava_from);
    if elapsed == 0 || !elapsed.is_multiple_of(p.lava_interval) {
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
    let Some(player) = world.actors.get_mut(world.player) else {
        return;
    };
    if is_down(player) || !in_lava(p, elapsed, home, player.pos) {
        return;
    }
    let loss = (u64::from(player.max_hp) * p.lava_damage_milli)
        .div_ceil(HUNDRED_PERCENT_MILLI)
        .max(1);
    // The status tick marks the player dead at 0 HP, as for any damage over time.
    player.hp = player
        .hp
        .saturating_sub(u32::try_from(loss).unwrap_or(u32::MAX));
    world.events.push(Event::Mechanic(MechanicEvent {
        kind: MechanicEvent::COLOSSUS_LAVA_BURNED,
        index: b as u16,
    }));
}

/// How deep the lava is `elapsed` ticks after it began, in meters.
pub fn lava_depth_m(per_tick: f32, max: f32, elapsed: u64) -> f32 {
    (elapsed as f32 * per_tick).min(max)
}

/// True if `pos` is in the lava, `elapsed` ticks after it began: within the depth of any of
/// the four walls, whose edges are `Params::edges` from the boss's spawn `home`.
fn in_lava(p: &Params, elapsed: u64, home: [f32; 3], pos: [f32; 3]) -> bool {
    let d = lava_depth_m(p.lava_per_tick, p.lava_max, elapsed);
    let (dx, dz) = (pos[0] - home[0], pos[2] - home[2]);
    dx < -p.edges[0] + d || dx > p.edges[1] - d || dz < -p.edges[2] + d || dz > p.edges[3] - d
}
