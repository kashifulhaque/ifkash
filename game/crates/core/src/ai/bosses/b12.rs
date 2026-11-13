//! B12, the Unspooled (WP-080), the final boss. Source: docs/game/05-enemies-and-bosses.md, "B12
//! The Unspooled", and `data/bosses/b12_the_unspooled.ron`, where every number below lives
//! (`params`). The Unspooled is one ordinary AI actor (the body), and the framework plays its moves
//! and the phase change. This file adds what the framework lacks:
//!
//! 1. **The rebirth.** The spec gives the boss two lives of 9,000 HP. The definition's one phase
//!    starts below 0.1% of the HP, a fraction no hit leaves standing, so the body dies first. On
//!    the next tick, before the framework's fight step can call it a defeat, the body stands back
//!    up at 1 HP, which starts the phase: the framework plays `b12_unravel` (45 ticks, invulnerable),
//!    and over those ticks this module refills the HP to full. The second death is the defeat. A
//!    phase-1 hit that leaves 1 to 8 HP starts the same transition, and the refill is the same.
//! 2. **The borrowed moves.** The `Phase(2)` options named in [`BORROWED`] are other bosses' moves.
//!    When the rebirth starts, the module notes which of those bosses are beaten (the boss's
//!    `boss.<id>.defeated` flag, or the player holds its Great Sigil), and keeps the others'
//!    options on cooldown for the rest of the fight. A borrowed move that the framework picked in
//!    the tick before that took hold is cancelled the tick it starts (as `b07` does).
//! 3. **The loom lines.** On frame `loom_lay_frame` of `b12_loom`, the floor lights `loom_lines`
//!    parallel lines, `loom_spacing_m` apart, across the line from the boss to the player, with
//!    the middle one through the player. `b12_loom_storm` lays a second set at right angles too.
//!    After `loom_warn_ticks` each line is a beam for `loom_beam_ticks`, and a beam takes
//!    `loom_damage_percent` of the max HP of a player on it (once a cast), unless the player is in
//!    the i-frames of a roll.
//! 4. **The size.** The hurtbox is the tall Weaver's in phase 1 and the larger Unspooled's from
//!    phase 2 (`p1_body_*`, `p2_body_*`).
//! 5. **The Echo Stone serves phase 1 only** (docs "Boss framework"): from the rebirth on, an Echo
//!    body that is out falls at once.
//!
//! The defeat raises `boss.b12_the_unspooled.defeated` (the framework), which is the ending's flag,
//! and pays the Courier's Letter.
//!
//! State: [`B12Run`], in `World::b12`. It is reset whenever the fight isn't on and hashed only
//! while it isn't at rest, so no other scenario's hash changes. The renderer draws the lines from
//! [`LoomEvent`]s (WASM event kind 50).

use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::ai::{Awareness, Doing, Token};
use crate::arena::Id;
use crate::combat::guard::GuardState;
use crate::combat::hurtbox::Hurtbox;
use crate::combat::moves::{MoveNum, flags as move_flags};
use crate::events::{Event, LoomEvent, MechanicEvent, ReviveEvent};
use crate::math::dmath::{cos, sin, sqrt};
use crate::world::World;
use crate::world_state::flags;

/// The boss definition this mechanic belongs to.
pub const BOSS_ID: &str = "b12_the_unspooled";
/// The moves that lay loom lines: one set, and one set each way.
const LOOM_MOVE: &str = "b12_loom";
const STORM_MOVE: &str = "b12_loom_storm";
/// The borrowed signature moves, one per Great Sigil boss (docs "B12": "one signature move from
/// each Great Sigil boss the player defeated"): the move, the boss, and its Great Sigil. Bit `i` of
/// [`B12Run::borrowed`] is entry `i`.
pub const BORROWED: [(&str, &str, &str); 4] = [
    (
        "b06_cantor_beam",
        "b06_drowned_choir",
        "itm_great_sigil_choir",
    ),
    (
        "b07_leap_slam",
        "b07_castellan_voss",
        "itm_great_sigil_castellan",
    ),
    (
        "b09_gnd_flare",
        "b09_lamplight_matriarch",
        "itm_great_sigil_matriarch",
    ),
    (
        "b10_flail_slam",
        "b10_the_bellwright",
        "itm_great_sigil_bellwright",
    ),
];
/// Line slots: two sets of at most three (the storm's grid).
pub const SLOTS: usize = 6;
const MAX_LINES: usize = SLOTS / 2;

/// One loom line. Free when `warn` and `beam` are both 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Line {
    /// Ticks until it becomes a beam.
    pub warn: u16,
    /// Ticks of beam left.
    pub beam: u16,
    /// Its center and its direction on the ground (a unit vector, x and z).
    pub x: f32,
    pub z: f32,
    pub dx: f32,
    pub dz: f32,
}

impl Line {
    pub fn is_out(&self) -> bool {
        self.warn > 0 || self.beam > 0
    }
}

/// B12's state (`World::b12`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct B12Run {
    /// The rebirth has begun (the phase-2 transition is playing or over).
    pub reborn: bool,
    /// Which [`BORROWED`] moves the Unspooled may use, by bit. Set at the rebirth.
    pub borrowed: u8,
    /// The body is in a loom move whose lines are already laid.
    pub laid: bool,
    /// A beam of the current cast hit the player.
    pub hit: bool,
    pub lines: [Line; SLOTS],
}

impl B12Run {
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

fn loom_event(world: &mut World, what: u8, b: usize, slot: usize, line: Line, ticks: u16) {
    let (half_length, half_width) = (
        number(world, b, "loom_half_length_m"),
        number(world, b, "loom_half_width_m"),
    );
    let y = body(world, b)
        .and_then(|id| world.actors.get(id))
        .map_or(0.0, |a| a.motor.ground_y);
    world.events.push(Event::Loom(LoomEvent {
        what,
        boss: b as u16,
        slot: slot as u16,
        center: [line.x, y, line.z],
        dir: [line.dx, line.dz],
        half_length,
        half_width,
        ticks,
    }));
}

/// One tick of B12's mechanic, for every placed B12 (the hook is [`super::run`], which runs before
/// the framework's fight step).
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
    let Some(id) = body(world, b) else {
        return;
    };
    let run = world.bosses[b];
    // The size: the Weaver until phase 2, then the Unspooled.
    let key = if run.phase >= 2 { "p2" } else { "p1" };
    let hurtbox = Hurtbox {
        radius: number(world, b, &format!("{key}_body_radius_m")),
        half_height: number(world, b, &format!("{key}_body_half_height_m")),
    };
    if let Some(a) = world.actors.get_mut(id)
        && hurtbox.radius > 0.0
    {
        a.hurtbox = hurtbox;
    }
    if run.stage != Stage::Fight {
        // Dormant (including after a reset), in the intro, or defeated: nothing is out. The
        // renderer also reads `CLEARED` as "the fight ended": the Unspooled is the Weaver again.
        if !world.b12.is_inert() {
            let defeated = usize::from(run.stage == Stage::Defeated);
            loom_event(world, LoomEvent::CLEARED, b, defeated, Line::default(), 0);
        }
        world.b12 = B12Run::default();
        return;
    }
    rebirth(world, b, id);
    if world.b12.reborn {
        borrow(world, b, id);
        unspool_echo(world, b);
    }
    loom(world, b, id);
}

// ---------------------------------------------------------------------------------------
// 1. The rebirth
// ---------------------------------------------------------------------------------------

/// Stands the Weaver back up after its first death, and refills its HP over the transition.
fn rebirth(world: &mut World, b: usize, id: Id) {
    let run = world.bosses[b];
    let down = world.actors.get(id).is_some_and(is_down);
    if run.phase == 1 && run.transition_left == 0 && down && !world.b12.reborn {
        // At 1 HP the fight's HP fraction is under the phase's threshold, so the framework's fight
        // step on this same tick plays the transition instead of calling the boss defeated.
        stand_up(world, id);
        return;
    }
    if run.transition_left == 0 || run.transition_to != 2 {
        return;
    }
    if !world.b12.reborn {
        world.b12.reborn = true;
        world.b12.borrowed = beaten_mask(world);
        let mask = u16::from(world.b12.borrowed);
        mechanic(world, MechanicEvent::UNSPOOLED_REBORN, mask);
    }
    // The HP comes back over the transition: full on its last tick.
    let total = world.layout.bosses[b]
        .transitions
        .first()
        .and_then(|&n| world.moves.get(n))
        .map_or(1, |d| d.data.total_frames.max(1));
    let done = u64::from(total.saturating_sub(run.transition_left)) + 1;
    if let Some(a) = world.actors.get_mut(id) {
        let hp = (u64::from(a.max_hp) * done / u64::from(total)).min(u64::from(a.max_hp));
        a.hp = (hp as u32).max(1);
    }
}

/// The body gets up at 1 HP, with nothing of its fall left on it (as `ai::special::revive`).
fn stand_up(world: &mut World, id: Id) {
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    a.dead = false;
    a.hp = 1;
    a.vel = [0.0; 3];
    a.moves.interrupt();
    a.moves.guard = GuardState::default();
    a.poise.accum = 0;
    a.poise.stance_dmg = 0;
    a.poise.kneel = 0;
    a.poise.parried = 0;
    a.hit.prev = None;
    crate::combat::status::cure_all(a);
    if let Some(brain) = a.ai.as_mut() {
        brain.dead_ticks = 0;
        brain.last_hp = 1;
        brain.doing = Doing::Nothing;
        brain.doing_ticks = 0;
        brain.awareness = Awareness::Combat;
        brain.token = Token::None;
        brain.token_target = None;
    }
    world
        .events
        .push(Event::Revive(ReviveEvent { actor: id, hp: 1 }));
}

/// The [`BORROWED`] entries whose boss is beaten: its flag is up, or the player holds its Sigil.
pub fn beaten_mask(world: &World) -> u8 {
    let inventory = world.actors.get(world.player).map(|p| &p.inventory);
    let mut mask = 0;
    for (i, (_, boss, sigil)) in BORROWED.iter().enumerate() {
        let flag = world.state.flags.is_set(&flags::boss_defeated(boss));
        let held = world
            .items
            .num_of(sigil)
            .zip(inventory)
            .is_some_and(|(n, inv)| inv.count(n) > 0);
        if flag || held {
            mask |= 1 << i;
        }
    }
    mask
}

// ---------------------------------------------------------------------------------------
// 2. The borrowed moves, and the Echo
// ---------------------------------------------------------------------------------------

/// Keeps the options of the borrowed moves whose boss isn't beaten on cooldown.
fn borrow(world: &mut World, b: usize, id: Id) {
    let mask = world.b12.borrowed;
    let options = &world.layout.bosses[b].def.body.options;
    let mut off_options = Vec::new();
    let mut off_moves: Vec<MoveNum> = Vec::new();
    for (i, (name, _, _)) in BORROWED.iter().enumerate() {
        if mask & (1 << i) != 0 {
            continue;
        }
        for (o, opt) in options.iter().enumerate() {
            if opt.move_id.as_deref() == Some(*name) {
                off_options.push(o);
            }
        }
        if let Some(n) = world.moves.num_of(name) {
            off_moves.push(n);
        }
    }
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
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
        for o in off_options {
            if let Some(cd) = brain.cooldowns.get_mut(o) {
                *cd = u16::MAX;
            }
        }
    }
}

/// From the rebirth on, an Echo body that is out falls (the Echo Stone serves phase 1 only).
fn unspool_echo(world: &mut World, b: usize) {
    let Some(active) = world.echo.active.as_ref() else {
        return;
    };
    let ids: Vec<Id> = active.members.iter().map(|m| m.id).collect();
    for id in ids {
        if let Some(a) = world.actors.get_mut(id)
            && !is_down(a)
        {
            a.hp = 0;
            a.dead = true;
            a.moves.current = None;
            mechanic(world, MechanicEvent::ECHO_UNSPOOLED, b as u16);
        }
    }
}

// ---------------------------------------------------------------------------------------
// 3. The loom lines
// ---------------------------------------------------------------------------------------

fn loom(world: &mut World, b: usize, id: Id) {
    let beam_ticks = number(world, b, "loom_beam_ticks").max(1.0) as u16;
    let half_length = number(world, b, "loom_half_length_m");
    let half_width = number(world, b, "loom_half_width_m");
    let percent = number(world, b, "loom_damage_percent").max(0.0) as u64;
    // The lines that are out count down first, so a line laid this tick waits its full warning.
    for slot in 0..SLOTS {
        let line = world.b12.lines[slot];
        if line.warn > 0 {
            let left = line.warn - 1;
            world.b12.lines[slot].warn = left;
            if left == 0 {
                world.b12.lines[slot].beam = beam_ticks;
                loom_event(world, LoomEvent::BEAM, b, slot, line, beam_ticks);
                burn_once(world, slot, half_length, half_width, percent);
            }
            continue;
        }
        if line.beam == 0 {
            continue;
        }
        let left = line.beam - 1;
        world.b12.lines[slot].beam = left;
        if left == 0 {
            loom_event(world, LoomEvent::FADED, b, slot, line, 0);
        } else {
            burn_once(world, slot, half_length, half_width, percent);
        }
    }
    let (loom, storm) = (
        world.moves.num_of(LOOM_MOVE),
        world.moves.num_of(STORM_MOVE),
    );
    let lay_frame = number(world, b, "loom_lay_frame").max(0.0) as u16;
    let current = world.actors.get(id).and_then(|a| a.moves.current);
    match current {
        Some(c) if Some(c.num) == loom || Some(c.num) == storm => {
            if !world.b12.laid && c.frame >= lay_frame {
                world.b12.laid = true;
                lay(world, b, id, Some(c.num) == storm);
            }
        }
        _ => world.b12.laid = false,
    }
}

/// A beam that is on burns the player, once a cast.
fn burn_once(world: &mut World, slot: usize, len: f32, width: f32, percent: u64) {
    let line = world.b12.lines[slot];
    if !world.b12.hit && burn(world, &line, len, width, percent) {
        world.b12.hit = true;
        mechanic(world, MechanicEvent::LOOM_BEAM_HIT, slot as u16);
    }
}

/// Lays a cast's lines: one set across the line from the boss to the player, and for the storm a
/// second set along it.
fn lay(world: &mut World, b: usize, id: Id, storm: bool) {
    let (Some(boss), Some(p)) = (world.actors.get(id), world.actors.get(world.player)) else {
        return;
    };
    let (bp, pp, yaw) = (boss.pos, p.pos, boss.yaw);
    let (dx, dz) = (pp[0] - bp[0], pp[2] - bp[2]);
    let len = sqrt(dx * dx + dz * dz);
    // `f` points from the boss to the player (its facing if the player stands on it).
    let f = if len > 0.001 {
        [dx / len, dz / len]
    } else {
        [sin(yaw), cos(yaw)]
    };
    let across = [f[1], -f[0]];
    let n = (number(world, b, "loom_lines").max(1.0) as usize).min(MAX_LINES);
    let spacing = number(world, b, "loom_spacing_m");
    let warn = number(world, b, "loom_warn_ticks").max(1.0) as u16;
    let mut sets = vec![(across, f)];
    if storm {
        sets.push((f, across));
    }
    world.b12.hit = false;
    for (s, (dir, step)) in sets.into_iter().enumerate() {
        for i in 0..n {
            let off = (i as f32 - (n - 1) as f32 * 0.5) * spacing;
            let line = Line {
                warn,
                beam: 0,
                x: pp[0] + step[0] * off,
                z: pp[2] + step[1] * off,
                dx: dir[0],
                dz: dir[1],
            };
            let slot = s * MAX_LINES + i;
            world.b12.lines[slot] = line;
            loom_event(world, LoomEvent::LAID, b, slot, line, warn);
        }
    }
}

/// A beam's hit on the player: `percent` of their max HP if they stand on the line and aren't in
/// a roll's i-frames. True if it hit.
fn burn(world: &mut World, line: &Line, half_length: f32, half_width: f32, percent: u64) -> bool {
    let Some(p) = world.actors.get_mut(world.player) else {
        return false;
    };
    if is_down(p) || p.moves.flags & move_flags::IFRAMES != 0 {
        return false;
    }
    let (rx, rz) = (p.pos[0] - line.x, p.pos[2] - line.z);
    let along = rx * line.dx + rz * line.dz;
    let side = (rx * line.dz - rz * line.dx).abs();
    if along.abs() > half_length || side > half_width {
        return false;
    }
    let loss = (u64::from(p.max_hp) * percent / 100).max(1);
    // The status tick marks the player dead at 0 HP, as for any damage over time.
    p.hp = p.hp.saturating_sub(u32::try_from(loss).unwrap_or(u32::MAX));
    true
}
