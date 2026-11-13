//! B07, Castellan Voss, the Overfit Warden (WP-075). Source: docs/game/05-enemies-and-bosses.md,
//! "B07 Castellan Voss, the Overfit Warden", and `data/bosses/b07_castellan_voss.ron`, where
//! every number below lives (`params`). Voss is one ordinary AI actor (the body), and the
//! framework plays his moves and the phase change at 60% HP. This file adds what it lacks:
//!
//! 1. **The counting.** In phase 1 the core counts the player's actions in six categories
//!    ([`Category`]). A *roll* is counted on its first tick, by where its direction points
//!    against the line from the player to Voss, in quarter-turn sectors: within 45 degrees of
//!    the way away is a roll back (so is a backstep), within 45 degrees of toward him is not
//!    counted, and either side is a roll left or right *as the player sees it*. A *parry* is
//!    counted when a parry lands (the attacker enters the parried state); a *guard* when a block
//!    lands (the guard counter window opens) or breaks; *ranged or spell* when the player starts
//!    a move that fires a projectile or casts a spell. Counting stops when the phase changes.
//! 2. **The counters.** When phase 2 starts, the two most-counted categories (at least
//!    `params.favor_min_count` each; ties go to the lower category number) earn a counter. The
//!    body's options for the other counters are kept on cooldown, and a counter the framework
//!    picked in the one tick before that took hold is cancelled the tick it starts. The counters
//!    are the `Phase(2)` options of the boss definition named in [`COUNTER_MOVES`].
//! 3. **The guard.** In `b07_guard`, frames `guard_from` to `guard_to`, a hit that costs Voss HP
//!    costs him nothing: the HP is put back and he plays `b07_guard_counter` at once. The stance
//!    damage of the hit stays, so breaking his stance through the guard is the other answer. The
//!    move's own `hyperarmor` keeps him from staggering.
//! 4. **The chandeliers.** From the start of phase 2, a marker appears on the floor under the
//!    player every `chandelier_interval_ticks` (the first after `chandelier_first_ticks`), and
//!    `chandelier_telegraph_ticks` later a chandelier falls on a circle of `chandelier_radius_m`
//!    meters. It costs a player standing in it `chandelier_damage_percent` of their max HP, unless
//!    the player is in the i-frames of a roll. At most two markers are out at once.
//! 5. **The back gate.** When Voss is defeated the flag `door.gate_castle_back.open` is raised,
//!    which opens the castle's back gate toward Rimeshelf.
//!
//! State: [`B07Run`], in `World::b07`: the six counts, what was seen last tick (to find the
//! edges), Voss's HP last tick, the chosen counters, and the markers. It is reset whenever the
//! fight isn't on, and hashed only while it isn't at rest, so no other scenario's hash changes.
//! The dev overlay reads the counts from the `VOSS_COUNTED` and `VOSS_OVERFIT` mechanic events.

use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::ai::{Doing, Token};
use crate::arena::Id;
use crate::combat::moves::{ActiveMove, MoveNum, flags as move_flags};
use crate::events::{Event, MechanicEvent};
use crate::math::dmath::{cos, sin, sqrt};
use crate::world::World;
use crate::world_state::flags;

/// The boss definition this mechanic belongs to.
pub const BOSS_ID: &str = "b07_castellan_voss";
/// The door the back gate is: `gate_castle_back` in the castle level.
const BACK_GATE: &str = "gate_castle_back";
/// The move Voss plays in guard, and the one that answers a hit on it.
const GUARD_MOVE: &str = "b07_guard";
const GUARD_COUNTER_MOVE: &str = "b07_guard_counter";
/// The counter for each category, by the category's number ([`Category`]).
pub const COUNTER_MOVES: [&str; CATEGORIES] = [
    "b07_c_sweep_pl",
    "b07_c_sweep_pr",
    "b07_c_lunge",
    "b07_c_delay_1",
    "b07_c_kick",
    "b07_c_dash",
];
/// How many action categories there are (docs "B07": six).
pub const CATEGORIES: usize = 6;
/// At most this many chandeliers are out at once.
const SLOTS: usize = 2;
/// The cosine of 45 degrees: the roll sectors are quarter turns (docs "B07", the six categories
/// split the roll into left, right, and back).
const SECTOR_COS: f32 = core::f32::consts::FRAC_1_SQRT_2;
/// The counters that Voss gains: the top two categories (docs "B07").
const FAVORED: usize = 2;
/// A marker's position is sent in half-meters, offset by 64, from the boss's spawn.
const MARKER_STEP_M: f32 = 0.5;
const MARKER_OFFSET: f32 = 64.0;
const MARKER_MAX: f32 = 127.0;

/// The player actions B07 counts, in the order of [`COUNTER_MOVES`] and of the event index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    RollLeft = 0,
    RollRight = 1,
    RollBack = 2,
    Parry = 3,
    Guard = 4,
    Ranged = 5,
}

/// One chandelier marker. `left` is the ticks until it falls, 0 for a free slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub left: u16,
    /// World position of its center.
    pub x: f32,
    pub z: f32,
}

/// B07's state (`World::b07`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct B07Run {
    /// Counted actions, per [`Category`].
    pub counts: [u16; CATEGORIES],
    /// The player's move and frame last tick, to tell when a move starts.
    pub seen: Option<(MoveNum, u16)>,
    /// The player's block was up (or broken) last tick, and Voss was parried last tick.
    pub was_blocked: bool,
    pub was_parried: bool,
    /// Voss's HP last tick, to find a hit on his guard. 0 before the fight's first tick.
    pub boss_hp: u32,
    /// Phase 2 has begun and the counters are chosen.
    pub overfit: bool,
    /// The categories whose counters are enabled, most-counted first.
    pub favored: [Option<u8>; FAVORED],
    /// The tick the next marker is placed at, 0 until phase 2.
    pub next_fall: u64,
    pub markers: [Marker; SLOTS],
}

impl B07Run {
    /// At rest: no fight on. Such a state is left out of the state hash.
    pub fn is_inert(&self) -> bool {
        *self == Self::default()
    }
}

/// The numbers from `BossDef::params`, by name (0 if missing).
fn number(world: &World, b: usize, key: &str) -> f32 {
    world.layout.bosses[b]
        .def
        .params
        .get(key)
        .copied()
        .unwrap_or(0.0)
}

/// The body actor.
fn body(world: &World, b: usize) -> Option<Id> {
    let spawn = *world.layout.bosses.get(b)?.members.first()?;
    world.state.ai_enemies.get(usize::from(spawn)).copied()
}

fn is_down(a: &Actor) -> bool {
    a.dead || a.hp == 0
}

fn mechanic(world: &mut World, kind: u8, index: u16) {
    world
        .events
        .push(Event::Mechanic(MechanicEvent { kind, index }));
}

/// One tick of B07's mechanic, for every placed B07 (the hook is [`super::run`]).
pub fn run(world: &mut World) {
    for b in 0..world.bosses.len() {
        if world
            .layout
            .bosses
            .get(b)
            .is_none_or(|l| l.def.id != BOSS_ID)
        {
            continue;
        }
        step(world, b);
    }
}

fn step(world: &mut World, b: usize) {
    match world.bosses[b].stage {
        Stage::Defeated => {
            // Voss fell (now, or in an earlier life of the save): the back gate opens.
            let open = flags::door_open(BACK_GATE);
            if !world.state.flags.is_set(&open) {
                world.state.flags.raise(&open);
            }
            world.b07 = B07Run::default();
            return;
        }
        Stage::Fight => {}
        _ => {
            // Dormant (including after a reset) or in the intro: nothing is counted.
            world.b07 = B07Run::default();
            return;
        }
    }
    let Some(id) = body(world, b) else {
        return;
    };
    let run = world.bosses[b];
    if run.phase <= 1 {
        if run.transition_left == 0 {
            observe(world, id);
        }
    } else {
        adapt(world, b, id);
        chandeliers(world, b);
    }
    guard(world, b, id);
    if let Some(a) = world.actors.get(id) {
        world.b07.boss_hp = a.hp;
    }
}

// ---------------------------------------------------------------------------------------
// 1. Counting
// ---------------------------------------------------------------------------------------

fn count(world: &mut World, category: Category) {
    let c = category as usize;
    let n = world.b07.counts[c].saturating_add(1);
    world.b07.counts[c] = n;
    let index = ((c as u16) << 8) | n.min(255);
    mechanic(world, MechanicEvent::VOSS_COUNTED, index);
}

/// Looks at the player and at Voss for the actions of the category list.
fn observe(world: &mut World, id: Id) {
    let (Some(p), Some(boss)) = (world.actors.get(world.player), world.actors.get(id)) else {
        return;
    };
    let cur = p.moves.current.map(|c| (c.num, c.frame));
    let prev = world.b07.seen;
    // A move starts when it isn't the one of last tick, or its frame is lower (it began again).
    let started = match (cur, prev) {
        (Some((n, f)), Some((pn, pf))) => n != pn || f < pf,
        (Some(_), None) => true,
        _ => false,
    };
    let blocked = p.moves.guard.counter > 0 || p.moves.guard.stagger > 0;
    let parried = boss.poise.parried > 0;
    let mut counted = Vec::new();
    if let (true, Some((num, _))) = (started, cur) {
        if let Some(c) = roll_category(world, p, boss, num) {
            counted.push(c);
        } else if fires_or_casts(world, num) {
            counted.push(Category::Ranged);
        }
    }
    if blocked && !world.b07.was_blocked {
        counted.push(Category::Guard);
    }
    if parried && !world.b07.was_parried {
        counted.push(Category::Parry);
    }
    world.b07.seen = cur;
    world.b07.was_blocked = blocked;
    world.b07.was_parried = parried;
    for c in counted {
        count(world, c);
    }
}

/// The category of the player's move `num` if it is a roll or a backstep: where the roll
/// goes, against the line to Voss. `None` for a roll toward him, and for any other move.
fn roll_category(world: &World, p: &Actor, boss: &Actor, num: MoveNum) -> Option<Category> {
    let set = world.moves.player_moveset.as_ref()?;
    if num == set.dodge_backstep {
        return Some(Category::RollBack);
    }
    if num != set.roll_light && num != set.roll_medium && num != set.roll_heavy {
        return None;
    }
    // The roll's direction is the player's yaw from its first tick (`combat::moves`).
    let dir = [sin(p.yaw), cos(p.yaw)];
    let (dx, dz) = (boss.pos[0] - p.pos[0], boss.pos[2] - p.pos[2]);
    let len = sqrt(dx * dx + dz * dz);
    if len < 0.001 {
        return None;
    }
    // `f` is the way to Voss; the player's right is `f` turned a quarter toward +X (yaw 0 faces
    // +Z and turns toward +X, so right = (f.z, -f.x)).
    let f = [dx / len, dz / len];
    let ahead = dir[0] * f[0] + dir[1] * f[1];
    let right = dir[0] * f[1] - dir[1] * f[0];
    if ahead <= -SECTOR_COS {
        Some(Category::RollBack)
    } else if ahead >= SECTOR_COS {
        None
    } else if right > 0.0 {
        Some(Category::RollRight)
    } else {
        Some(Category::RollLeft)
    }
}

/// True if move `num` fires a projectile or casts the left-hand spell.
fn fires_or_casts(world: &World, num: MoveNum) -> bool {
    let casts = world
        .moves
        .player_moveset
        .as_ref()
        .and_then(|s| s.left_cast)
        .is_some_and(|c| c == num);
    casts
        || world
            .moves
            .get(num)
            .is_some_and(|d| !d.data.projectiles.is_empty())
}

// ---------------------------------------------------------------------------------------
// 2. The counters
// ---------------------------------------------------------------------------------------

/// Chooses the counters when phase 2 first shows, and keeps the others on cooldown.
fn adapt(world: &mut World, b: usize, id: Id) {
    if !world.b07.overfit {
        let min = (number(world, b, "favor_min_count") as u16).max(1);
        let counts = world.b07.counts;
        let mut order: Vec<usize> = (0..CATEGORIES).filter(|&c| counts[c] >= min).collect();
        // Most counted first; a tie goes to the lower category.
        order.sort_by(|&a, &c| counts[c].cmp(&counts[a]).then(a.cmp(&c)));
        let pick = |i: usize| order.get(i).map(|&c| c as u8);
        world.b07.favored = [pick(0), pick(1)];
        world.b07.overfit = true;
        let code = |f: Option<u8>| u16::from(f.unwrap_or(0xFF));
        let index = (code(world.b07.favored[0]) << 8) | code(world.b07.favored[1]);
        mechanic(world, MechanicEvent::VOSS_OVERFIT, index);
    }
    let enabled = |c: usize| world.b07.favored.contains(&Some(c as u8));
    let options = &world.layout.bosses[b].def.body.options;
    // The options of every counter that is off, and the move numbers of those counters.
    let mut off_options = Vec::new();
    let mut off_moves = Vec::new();
    for (c, name) in COUNTER_MOVES.iter().enumerate() {
        if enabled(c) {
            continue;
        }
        for (i, o) in options.iter().enumerate() {
            if o.move_id.as_deref() == Some(*name) {
                off_options.push(i);
            }
        }
        if let Some(n) = world.moves.num_of(name) {
            off_moves.push(n);
        }
    }
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    // A counter that is off and has just started (the framework picked it in the tick the
    // phase began, before the cooldowns could hold) is cancelled.
    if let Some(c) = a.moves.current
        && c.frame <= 1
        && off_moves.contains(&c.num)
    {
        a.moves.interrupt();
        if let Some(brain) = a.ai.as_mut() {
            brain.doing = Doing::Nothing;
            brain.token = Token::None;
            brain.token_target = None;
        }
    }
    if let Some(brain) = a.ai.as_mut() {
        for i in off_options {
            if let Some(cd) = brain.cooldowns.get_mut(i) {
                *cd = u16::MAX;
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// 3. The guard
// ---------------------------------------------------------------------------------------

/// Blocks a hit on the guard stance and plays the counter.
fn guard(world: &mut World, b: usize, id: Id) {
    let (Some(guard_num), Some(counter_num)) = (
        world.moves.num_of(GUARD_MOVE),
        world.moves.num_of(GUARD_COUNTER_MOVE),
    ) else {
        return;
    };
    let (from, to) = (number(world, b, "guard_from"), number(world, b, "guard_to"));
    let Some(free_from) = world
        .moves
        .get(counter_num)
        .map(|d| d.data.cancel_any_from.unwrap_or(d.data.total_frames))
    else {
        return;
    };
    let before = world.b07.boss_hp;
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    let Some(c) = a.moves.current else {
        return;
    };
    let frame = f32::from(c.frame);
    if is_down(a) || c.num != guard_num || frame < from || frame > to {
        return;
    }
    if before == 0 || a.hp >= before {
        return;
    }
    // The hit is blocked: no damage, and the counter starts at once.
    a.hp = before;
    a.moves.interrupt();
    a.vel[0] = 0.0;
    a.vel[2] = 0.0;
    a.moves.current = Some(ActiveMove {
        num: counter_num,
        frame: 0,
        charge_ticks: 0,
        hold_button: 0,
        free_from,
    });
    if let Some(brain) = a.ai.as_mut() {
        brain.doing = Doing::Attack {
            option: 0,
            move_num: counter_num,
        };
        brain.doing_ticks = 0;
        brain.token = Token::None;
        brain.token_target = None;
    }
    mechanic(world, MechanicEvent::VOSS_BLOCKED, 0);
}

// ---------------------------------------------------------------------------------------
// 4. The chandeliers
// ---------------------------------------------------------------------------------------

/// A coordinate offset from the spawn, in meters, as the half-meter code (0 to 127).
fn quantize(d: f32) -> u16 {
    ((d / MARKER_STEP_M).round() + MARKER_OFFSET).clamp(0.0, MARKER_MAX) as u16
}

/// The offset in meters a half-meter code stands for.
fn unquantize(q: u16) -> f32 {
    (f32::from(q) - MARKER_OFFSET) * MARKER_STEP_M
}

/// The event index for a marker in `slot` at codes `(x, z)`.
fn marker_index(slot: usize, x: u16, z: u16) -> u16 {
    ((slot as u16) << 14) | (x << 7) | z
}

fn chandeliers(world: &mut World, b: usize) {
    let run = world.bosses[b];
    if run.transition_left > 0 {
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
    let telegraph = number(world, b, "chandelier_telegraph_ticks").max(1.0) as u16;
    let interval = number(world, b, "chandelier_interval_ticks").max(1.0) as u64;
    let first = number(world, b, "chandelier_first_ticks").max(1.0) as u64;
    let radius = number(world, b, "chandelier_radius_m");
    let percent = number(world, b, "chandelier_damage_percent").max(0.0) as u64;
    if world.b07.next_fall == 0 {
        world.b07.next_fall = world.tick + first;
    }
    // Markers that are out count down and fall.
    for slot in 0..SLOTS {
        let m = world.b07.markers[slot];
        if m.left == 0 {
            continue;
        }
        world.b07.markers[slot].left = m.left - 1;
        if m.left > 1 {
            continue;
        }
        let code = (quantize(m.x - home[0]), quantize(m.z - home[2]));
        let index = marker_index(slot, code.0, code.1);
        mechanic(world, MechanicEvent::VOSS_CHANDELIER_FELL, index);
        let Some(p) = world.actors.get_mut(world.player) else {
            continue;
        };
        let (dx, dz) = (p.pos[0] - m.x, p.pos[2] - m.z);
        let dodged = p.moves.flags & move_flags::IFRAMES != 0;
        if is_down(p) || dodged || sqrt(dx * dx + dz * dz) > radius {
            continue;
        }
        let loss = (u64::from(p.max_hp) * percent / 100).max(1);
        // The status tick marks the player dead at 0 HP, as for any damage over time.
        p.hp = p.hp.saturating_sub(u32::try_from(loss).unwrap_or(u32::MAX));
        mechanic(world, MechanicEvent::VOSS_CHANDELIER_HIT, slot as u16);
    }
    // A new marker under the player, when it is time and a slot is free.
    if world.tick < world.b07.next_fall {
        return;
    }
    let Some(slot) = world.b07.markers.iter().position(|m| m.left == 0) else {
        world.b07.next_fall = world.tick + 1;
        return;
    };
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    if is_down(p) {
        return;
    }
    // The marker's center is snapped to the grid the event can carry.
    let (qx, qz) = (quantize(p.pos[0] - home[0]), quantize(p.pos[2] - home[2]));
    world.b07.markers[slot] = Marker {
        left: telegraph,
        x: home[0] + unquantize(qx),
        z: home[2] + unquantize(qz),
    };
    world.b07.next_fall = world.tick + interval;
    mechanic(
        world,
        MechanicEvent::VOSS_CHANDELIER_MARKED,
        marker_index(slot, qx, qz),
    );
}
