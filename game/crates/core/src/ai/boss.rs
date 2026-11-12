//! The boss framework: fog gates, the intro wait, phases with transition moves,
//! multi-part bosses, resets, and rewards. See docs/game/05-enemies-and-bosses.md,
//! "Boss framework" and "Fairness rules".
//!
//! A boss is a group of ordinary AI actors (the body and its parts, see
//! `lf_content::boss`) that the scenario places with a [`FogGate`]. [`extend_layout`]
//! appends their spawns to `Layout::enemies` when the world is built, so the same code
//! makes them, and [`BossLayout::members`] names them. [`run`] is the boss half of tick
//! system 3; it runs before the actors think.
//!
//! # A fight's stages ([`Stage`])
//!
//! 1. **Dormant.** The members stand still (they skip [`super::run`]'s thinking). The
//!    player walks through the gate: across its plane, within its width and `depth_m`.
//! 2. **Intro.** The gate seals: a living player on the outside of it is put back on the
//!    plane. The core waits `intro_ticks` (90, a 3 s camera move on the renderer's side).
//!    Nothing else changes: the boss doesn't act, and isn't invulnerable.
//! 3. **Fight.** Every member goes straight to Combat against the player and stays
//!    there: it can't lose sight or leash. A phase starts when the HP of the body and
//!    the required parts, taken together, falls below the next threshold. The body then
//!    plays the phase's 45-tick transition move (its `iframes` cover every frame, which
//!    the validator checks, so nothing hurts it), and afterwards `AiState::phase` is the
//!    new phase, which `Condition::Phase` options read. Parts keep fighting meanwhile.
//! 4. **Defeated.** The body and every required part are dead. The gate opens, the other
//!    parts fall, and the rewards pay out: Glimmer, the listed items, the world flag
//!    `boss.<id>.defeated`, and the codex flag `codex.<entry>`. A defeated boss stays down:
//!    nothing respawns it.
//!
//! The player's death ends a fight at once: every member is rebuilt at its spawn at full
//! HP, the boss's projectiles are removed, and the stage goes back to Dormant. The Remnant
//! drops just outside the gate (see [`remnant_override`]).
//!
//! The attack rules that need run-time help live here too: [`TRACK_STOP_TICKS`].
//!
//! Boss state is in [`World::bosses`], hashed only once a scenario has a boss, so the
//! older scenarios' hashes are unchanged.

use lf_content::scenario::{EnemySpawn, FogGate};
use lf_content::{BossDef, Content, NumId, Scenario};
use serde::{Deserialize, Serialize};

use super::{Awareness, Doing, Token, set_awareness};
use crate::actor::Actor;
use crate::arena::Id;
use crate::combat::moves::{ActiveMove, MoveNum, MoveTable};
use crate::events::{BossEvent, Event};
use crate::math::dmath::{cos, sin};
use crate::world::World;
use crate::world_state::Layout;
use crate::world_state::flags;

/// A boss attack stops tracking the player this many ticks before its first active frame
/// (docs "Fairness rules": "Tracking stops at least 4 ticks before the active frames").
pub const TRACK_STOP_TICKS: u16 = 4;
/// How far outside the gate a Remnant drops, in meters ("just outside the fog gate",
/// docs "Glimmer and death").
const REMNANT_OUTSIDE_M: f32 = 1.5;

/// One boss in the scenario: its definition, gate, and members. Not hashed (it is built
/// from content).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BossLayout {
    /// The spawn's label.
    pub name: String,
    pub def: BossDef,
    pub gate: FogGate,
    /// The members' spawns, as indexes into `Layout::enemies`: the body first, then
    /// `def.parts` in order.
    pub members: Vec<u16>,
    /// `def.phases[i].transition`, resolved.
    pub transitions: Vec<MoveNum>,
    /// `def.rewards.items`, resolved: registry number and count.
    pub reward_items: Vec<(NumId, u16)>,
}

impl BossLayout {
    /// The flag raised when the boss is defeated.
    pub fn defeated_flag(&self) -> String {
        flags::boss_defeated(&self.def.id)
    }

    /// True if member `i` (0 is the body) counts toward the fight's HP and its end.
    fn required(&self, i: usize) -> bool {
        i == 0 || self.def.parts.get(i - 1).is_some_and(|p| p.required)
    }
}

/// Where a boss fight stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// Waiting for the player to walk through the gate.
    #[default]
    Dormant,
    /// The gate is sealed and the core is waiting; this many ticks are left.
    Intro {
        left: u16,
    },
    Fight,
    Defeated,
}

/// One boss's state. Simulation state: part of the state hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossRun {
    pub stage: Stage,
    /// The current phase, from 1. 0 until the fight starts.
    pub phase: u8,
    /// Ticks left of a phase transition move; 0 when none is playing.
    pub transition_left: u16,
    /// The phase the transition leads to.
    pub transition_to: u8,
}

impl Default for BossRun {
    fn default() -> Self {
        Self {
            stage: Stage::Dormant,
            phase: 0,
            transition_left: 0,
            transition_to: 0,
        }
    }
}

/// Adds the scenario's boss fights to the layout: every member becomes an enemy spawn
/// (so `World::new` creates it), and each boss gets a [`BossLayout`]. Fails on a boss
/// or move that isn't defined.
pub fn extend_layout(
    layout: &mut Layout,
    level: &Scenario,
    content: &Content,
    moves: &MoveTable,
) -> Result<(), String> {
    for b in &level.bosses {
        let def = content
            .boss_by_id(&b.boss)
            .ok_or_else(|| format!("boss spawn {:?}: unknown boss \"{}\"", b.name, b.boss))?;
        let mut members = Vec::new();
        for (i, actor) in def.actors().enumerate() {
            let (name, off) = match i {
                0 => (b.name.clone(), [0.0; 3]),
                _ => (format!("{}.part{}", b.name, i - 1), def.parts[i - 1].offset),
            };
            members.push(layout.enemies.len() as u16);
            layout.enemies.push(EnemySpawn {
                name,
                enemy: actor.id.clone(),
                pos: [b.pos[0] + off[0], b.pos[1] + off[1], b.pos[2] + off[2]],
                yaw: b.yaw,
                patrol: Vec::new(),
            });
        }
        let transitions = def
            .phases
            .iter()
            .map(|p| {
                moves.num_of(&p.transition).ok_or_else(|| {
                    format!("boss \"{}\": unknown move \"{}\"", def.id, p.transition)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let reward_items = def
            .rewards
            .items
            .iter()
            .map(|r| {
                content
                    .registry
                    .num(&r.item)
                    .map(|n| (n, r.count))
                    .ok_or_else(|| format!("boss \"{}\": unknown item \"{}\"", def.id, r.item))
            })
            .collect::<Result<Vec<_>, _>>()?;
        layout.bosses.push(BossLayout {
            name: b.name.clone(),
            def: def.clone(),
            gate: b.gate,
            members,
            transitions,
            reward_items,
        });
    }
    Ok(())
}

/// The gate's inward direction on the XZ plane.
fn inward(g: &FogGate) -> [f32; 2] {
    [sin(g.yaw), cos(g.yaw)]
}

/// `(along, across)`: how far `pos` is past the gate's plane (negative outside), and how
/// far to the side of its middle.
fn gate_coords(g: &FogGate, pos: [f32; 3]) -> (f32, f32) {
    let n = inward(g);
    let (dx, dz) = (pos[0] - g.pos[0], pos[2] - g.pos[2]);
    (dx * n[0] + dz * n[1], dx * n[1] - dz * n[0])
}

/// The member actor of boss `b`'s member `i`.
fn member(world: &World, b: usize, i: usize) -> Option<Id> {
    let spawn = *world.layout.bosses.get(b)?.members.get(i)?;
    world.state.ai_enemies.get(usize::from(spawn)).copied()
}

fn is_down(a: &Actor) -> bool {
    a.dead || a.hp == 0
}

/// The boss whose member the AI spawn `spawn` is, if any.
pub fn boss_of_spawn(layout: &Layout, spawn: u16) -> Option<usize> {
    layout
        .bosses
        .iter()
        .position(|b| b.members.contains(&spawn))
}

/// True if the actor is a boss member that shouldn't think now: its fight hasn't started.
/// ([`super::run`] skips it.)
pub fn benched(world: &World, id: Id) -> bool {
    let Some(b) = world
        .actors
        .get(id)
        .and_then(|a| a.ai.as_ref())
        .and_then(|brain| boss_of_spawn(&world.layout, brain.spawn))
    else {
        return false;
    };
    world.bosses.get(b).is_some_and(|r| r.stage != Stage::Fight)
}

/// Where a Remnant dropped now goes instead of the player's safe spot: just outside the
/// gate of a fight in progress. `None` outside a fight.
pub fn remnant_override(world: &World) -> Option<[f32; 3]> {
    let (b, _) = world
        .bosses
        .iter()
        .enumerate()
        .find(|(_, r)| matches!(r.stage, Stage::Intro { .. } | Stage::Fight))?;
    let g = &world.layout.bosses.get(b)?.gate;
    let n = inward(g);
    Some([
        g.pos[0] - n[0] * REMNANT_OUTSIDE_M,
        g.pos[1],
        g.pos[2] - n[1] * REMNANT_OUTSIDE_M,
    ])
}

/// True if a living player at `pos` has walked through the gate.
fn through(g: &FogGate, pos: [f32; 3]) -> bool {
    let (along, across) = gate_coords(g, pos);
    along > 0.0 && along <= g.depth_m && across.abs() <= g.half_width_m
}

/// The boss half of tick system 3: gates, intros, phases, defeat, and resets.
pub fn run(world: &mut World) {
    if world.bosses.is_empty() {
        return;
    }
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    let (alive, pos) = (!is_down(p), p.pos);
    for b in 0..world.bosses.len() {
        match world.bosses[b].stage {
            Stage::Defeated => {}
            Stage::Dormant => {
                let flag = world.layout.bosses[b].defeated_flag();
                if world.state.flags.is_set(&flag) {
                    // A boss beaten in an earlier life of the save: it stays down.
                    world.bosses[b].stage = Stage::Defeated;
                    fell_members(world, b, true);
                } else if alive && through(&world.layout.bosses[b].gate, pos) {
                    let left = world.layout.bosses[b].def.intro_ticks.max(1);
                    world.bosses[b].stage = Stage::Intro { left };
                    emit(world, b, BossEvent::SEALED, 0);
                }
            }
            Stage::Intro { left } => {
                if !alive {
                    reset(world, b);
                    continue;
                }
                seal(world, b, pos);
                if left <= 1 {
                    start_fight(world, b);
                } else {
                    world.bosses[b].stage = Stage::Intro { left: left - 1 };
                }
            }
            Stage::Fight => {
                if !alive {
                    reset(world, b);
                    continue;
                }
                seal(world, b, pos);
                fight(world, b);
            }
        }
    }
}

fn emit(world: &mut World, b: usize, kind: u8, phase: u8) {
    world.events.push(Event::Boss(BossEvent {
        boss: b as u16,
        kind,
        phase,
    }));
}

/// A sealed gate: a player on the outside of the plane, within its width, is put back on
/// it.
fn seal(world: &mut World, b: usize, pos: [f32; 3]) {
    let g = world.layout.bosses[b].gate;
    let (along, across) = gate_coords(&g, pos);
    if along >= 0.0 || across.abs() > g.half_width_m {
        return;
    }
    let n = inward(&g);
    if let Some(p) = world.actors.get_mut(world.player) {
        p.pos[0] -= n[0] * along;
        p.pos[2] -= n[1] * along;
    }
}

/// The intro is over: every living member goes to Combat against the player.
fn start_fight(world: &mut World, b: usize) {
    let player = world.player;
    let at = world.actors.get(player).map_or([0.0; 3], |p| p.pos);
    let n = world.layout.bosses[b].members.len();
    for i in 0..n {
        let Some(id) = member(world, b, i) else {
            continue;
        };
        if let Some(a) = world.actors.get_mut(id)
            && !is_down(a)
        {
            a.poise.unaware = false;
            a.lock_target = Some(player);
            if let Some(brain) = a.ai.as_mut() {
                brain.target = Some(player);
                brain.last_seen = at;
                brain.unseen_ticks = 0;
                brain.doing = Doing::Nothing;
                brain.doing_ticks = 0;
                brain.phase = 1;
            }
        }
        set_awareness(world, id, Awareness::Combat);
    }
    world.bosses[b] = BossRun {
        stage: Stage::Fight,
        phase: 1,
        ..BossRun::default()
    };
    emit(world, b, BossEvent::FIGHT_STARTED, 1);
}

/// The fight's HP fraction: the body and the required parts together.
fn fight_hp_frac(world: &World, b: usize) -> f32 {
    let (mut hp, mut max) = (0u64, 0u64);
    for i in 0..world.layout.bosses[b].members.len() {
        if !world.layout.bosses[b].required(i) {
            continue;
        }
        if let Some(a) = member(world, b, i).and_then(|id| world.actors.get(id)) {
            max += u64::from(a.max_hp);
            if !is_down(a) {
                hp += u64::from(a.hp);
            }
        }
    }
    if max == 0 {
        0.0
    } else {
        hp as f32 / max as f32
    }
}

/// One tick of a fight in progress.
fn fight(world: &mut World, b: usize) {
    let player = world.player;
    let at = world.actors.get(player).map_or([0.0; 3], |p| p.pos);
    let phase = world.bosses[b].phase;
    let n = world.layout.bosses[b].members.len();
    for i in 0..n {
        if let Some(id) = member(world, b, i) {
            hold_in_combat(world, id, player, at, phase);
        }
    }
    let defeated = (0..n)
        .filter(|&i| world.layout.bosses[b].required(i))
        .all(|i| {
            member(world, b, i)
                .and_then(|id| world.actors.get(id))
                .is_none_or(is_down)
        });
    if defeated {
        defeat(world, b);
        return;
    }
    let run = world.bosses[b];
    if run.transition_left > 0 {
        let left = run.transition_left - 1;
        world.bosses[b].transition_left = left;
        if left == 0 {
            end_transition(world, b);
        }
    } else if let Some(phase_def) = world.layout.bosses[b]
        .def
        .phases
        .get(usize::from(run.phase).saturating_sub(1))
        && fight_hp_frac(world, b) < phase_def.below
    {
        begin_transition(world, b);
    }
}

/// Keeps a member fighting: in Combat, aware of the player wherever they are, in the
/// fight's phase, and not tracking in the last ticks of an attack's telegraph.
fn hold_in_combat(world: &mut World, id: Id, player: Id, at: [f32; 3], phase: u8) {
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    if a.dead {
        return;
    }
    match a.moves.current {
        Some(c) => {
            if world
                .moves
                .get(c.num)
                .is_some_and(|d| c.frame + 1 + TRACK_STOP_TICKS >= d.track_until)
            {
                a.lock_target = None;
            }
        }
        None => a.lock_target = Some(player),
    }
    let Some(brain) = a.ai.as_mut() else {
        return;
    };
    brain.phase = phase;
    brain.target = Some(player);
    brain.last_seen = at;
    brain.unseen_ticks = 0;
    if brain.awareness != Awareness::Combat {
        set_awareness(world, id, Awareness::Combat);
    }
}

/// The body drops what it is doing and plays the next phase's transition move.
fn begin_transition(world: &mut World, b: usize) {
    let phase = world.bosses[b].phase;
    let Some(&num) = world.layout.bosses[b]
        .transitions
        .get(usize::from(phase) - 1)
    else {
        return;
    };
    let Some(d) = world.moves.get(num) else {
        return;
    };
    let (total, free_from) = (d.data.total_frames, d.data.cancel_any_from);
    let Some(id) = member(world, b, 0) else {
        return;
    };
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    if is_down(a) {
        return;
    }
    a.moves.current = Some(ActiveMove {
        num,
        frame: 0,
        charge_ticks: 0,
        hold_button: 0,
        free_from: free_from.unwrap_or(total),
    });
    a.vel[0] = 0.0;
    a.vel[2] = 0.0;
    if let Some(brain) = a.ai.as_mut() {
        brain.doing = Doing::Attack {
            option: 0,
            move_num: num,
        };
        brain.doing_ticks = 0;
        brain.token = Token::None;
        brain.token_target = None;
    }
    world.bosses[b].transition_left = total.max(1);
    world.bosses[b].transition_to = phase + 1;
    emit(world, b, BossEvent::PHASE_STARTED, phase + 1);
}

/// The transition move is over: the new phase's options take effect.
fn end_transition(world: &mut World, b: usize) {
    let to = world.bosses[b].transition_to;
    world.bosses[b].phase = to;
    world.bosses[b].transition_to = 0;
    for i in 0..world.layout.bosses[b].members.len() {
        let Some(id) = member(world, b, i) else {
            continue;
        };
        if let Some(brain) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
            brain.phase = to;
            if i == 0 {
                brain.doing = Doing::Nothing;
                brain.cooldowns.iter_mut().for_each(|c| *c = 0);
            }
        }
    }
}

/// Kills every member that is still up (a defeat takes the rest of the fight with it, or
/// a saved defeat applies). With `all`, the required members too.
fn fell_members(world: &mut World, b: usize, all: bool) {
    for i in 0..world.layout.bosses[b].members.len() {
        if !all && world.layout.bosses[b].required(i) {
            continue;
        }
        if let Some(a) = member(world, b, i).and_then(|id| world.actors.get_mut(id)) {
            a.hp = 0;
            a.dead = true;
            a.moves.current = None;
        }
    }
}

/// The boss falls: the gate opens, the other parts fall, and the rewards pay out.
fn defeat(world: &mut World, b: usize) {
    fell_members(world, b, false);
    world.bosses[b] = BossRun {
        stage: Stage::Defeated,
        phase: world.bosses[b].phase,
        ..BossRun::default()
    };
    let lay = &world.layout.bosses[b];
    let rewards = &lay.def.rewards;
    let (glimmer, items, codex) = (
        rewards.glimmer,
        lay.reward_items.clone(),
        rewards.codex.clone(),
    );
    let flag = lay.defeated_flag();
    world.state.flags.raise(&flag);
    if !codex.is_empty() {
        world.state.flags.raise(&flags::codex(&codex));
    }
    if let Some(p) = world.actors.get_mut(world.player) {
        p.inventory.glimmer = p.inventory.glimmer.saturating_add(glimmer);
        for (item, count) in items {
            p.inventory.pick_up(item, count, &world.items);
        }
    }
    emit(world, b, BossEvent::DEFEATED, 0);
}

/// The player died: every member is rebuilt at its spawn at full HP, the boss's
/// projectiles are removed, and the fight is Dormant again.
fn reset(world: &mut World, b: usize) {
    let hurtbox = crate::combat::hurtbox::Hurtbox::from_locomotion(&world.locomotion);
    let spawns = world.layout.bosses[b].members.clone();
    let mut ids = Vec::new();
    for spawn in spawns {
        let i = usize::from(spawn);
        let Some(fresh) =
            crate::ai::enemy_actor(&world.layout.enemies[i], spawn, &world.ai, hurtbox)
        else {
            continue;
        };
        match world
            .state
            .ai_enemies
            .get(i)
            .and_then(|&id| world.actors.get_mut(id))
        {
            Some(a) => *a = fresh,
            None => {
                let id = world.actors.insert(fresh);
                match world.state.ai_enemies.get_mut(i) {
                    Some(h) => *h = id,
                    None => world.state.ai_enemies.push(id),
                }
            }
        }
        ids.push(world.state.ai_enemies[i]);
    }
    let shot: Vec<Id> = world
        .projectiles
        .iter()
        .filter(|(_, p)| ids.contains(&p.shooter))
        .map(|(id, _)| id)
        .collect();
    for id in shot {
        world.projectiles.remove(id);
    }
    world.bosses[b] = BossRun::default();
    emit(world, b, BossEvent::RESET, 0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::LifeEvent;
    use crate::input::{ActionFrame, TickInput};
    use crate::replay::{Replay, expand};
    use crate::tick::tick;

    fn arena(seed: u64) -> World {
        let content = Content::load_embedded().unwrap();
        World::new(seed, &content, "boss_gate").unwrap()
    }

    fn step(w: &mut World, x: i8, z: i8, b: u16, n: u32) -> Vec<Event> {
        let input = TickInput {
            frame: ActionFrame {
                move_x: x,
                move_z: z,
                buttons: b,
            },
            commands: Vec::new(),
        };
        let mut out = Vec::new();
        for _ in 0..n {
            tick(w, &input);
            out.extend(w.events.drain());
        }
        out
    }

    fn bosses(events: &[Event]) -> Vec<(u8, u8)> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::Boss(b) => Some((b.kind, b.phase)),
                _ => None,
            })
            .collect()
    }

    fn put_player(w: &mut World, pos: [f32; 3]) {
        let p = w.actors.get_mut(w.player).unwrap();
        p.pos = pos;
        p.motor.ground_y = pos[1];
        p.motor.fall_peak_y = pos[1];
    }

    /// The body, the pillar part, and the player's Glimmer.
    fn cast(w: &World) -> (Id, Id) {
        (member(w, 0, 0).unwrap(), member(w, 0, 1).unwrap())
    }

    /// Ticks until the fight starts, with the player just inside the gate.
    fn start_fight_now(w: &mut World) -> Vec<Event> {
        put_player(w, [0.0, 0.0, -0.2]);
        let mut events = step(w, 0, 127, 0, 12);
        assert!(matches!(w.bosses[0].stage, Stage::Intro { .. }));
        events.extend(step(w, 0, 0, 0, 100));
        assert_eq!(w.bosses[0].stage, Stage::Fight);
        events
    }

    #[test]
    fn the_golden_replay_changes_phase_and_resets_after_the_players_death() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/replays/m3_boss_phase_reset.lfr"
        );
        let replay = Replay::decode(&std::fs::read(path).unwrap()).unwrap();
        let mut w = arena(replay.header.seed);
        let (body, pillar) = cast(&w);
        let mut events = Vec::new();
        let mut body_hp = Vec::new();
        for frame in expand(&replay.frames) {
            tick(
                &mut w,
                &TickInput {
                    frame,
                    commands: Vec::new(),
                },
            );
            body_hp.push((w.tick, w.actors.get(body).unwrap().hp));
            events.extend(w.events.drain());
        }
        // Sealed, the fight, phase 2, then the reset after the death.
        assert_eq!(
            bosses(&events),
            [
                (BossEvent::SEALED, 0),
                (BossEvent::FIGHT_STARTED, 1),
                (BossEvent::PHASE_STARTED, 2),
                (BossEvent::RESET, 0)
            ]
        );
        let at = |kind: u8| {
            events
                .iter()
                .position(|e| matches!(e, Event::Boss(b) if b.kind == kind))
                .unwrap()
        };
        let died = events
            .iter()
            .position(|e| matches!(e, Event::Life(LifeEvent::Died { .. })))
            .unwrap();
        assert!(at(BossEvent::PHASE_STARTED) < died && died < at(BossEvent::RESET));
        // The Remnant drops just outside the gate, not where the player fell.
        let dropped = events.iter().find_map(|e| match e {
            Event::Life(LifeEvent::RemnantDropped { glimmer, pos }) => Some((*glimmer, *pos)),
            _ => None,
        });
        let (glimmer, pos) = dropped.expect("a Remnant");
        assert_eq!(glimmer, 500); // boss_gate's start_glimmer
        assert!(pos[0].abs() < 0.01 && (pos[2] + REMNANT_OUTSIDE_M).abs() < 0.01);
        // The transition's 45 ticks are invulnerable: the body's HP holds still.
        let phase_tick = events
            .iter()
            .position(|e| matches!(e, Event::Boss(b) if b.kind == BossEvent::PHASE_STARTED))
            .unwrap();
        assert!(phase_tick > 0);
        let start = body_hp
            .iter()
            .position(|&(_, hp)| hp < 900)
            .expect("the player landed hits");
        assert!(start > 0);
        // The last and the first HP seen during the transition are equal.
        let (first_in, _) = body_hp
            .iter()
            .enumerate()
            .find(|&(_, &(_, hp))| hp < 600)
            .map(|(i, &(_, hp))| (i, hp))
            .unwrap();
        assert!(
            body_hp[first_in..first_in + 40]
                .iter()
                .all(|&(_, hp)| hp == body_hp[first_in].1),
            "the boss took damage during its transition"
        );
        // After the reset: full HP and Dormant, the pillar rebuilt, the player respawned.
        assert_eq!(w.bosses[0], BossRun::default());
        for id in [body, pillar] {
            let a = w.actors.get(id).unwrap();
            assert_eq!((a.hp, a.dead), (a.max_hp, false));
        }
        assert_eq!(w.actors.get(body).unwrap().hp, 900);
        assert!(!w.actors.get(w.player).unwrap().dead);
        assert!(!w.state.flags.is_set(&flags::boss_defeated("test_boss")));
    }

    #[test]
    fn walking_through_the_gate_seals_it_and_the_fight_starts_after_the_intro() {
        let mut w = arena(1);
        let (body, _) = cast(&w);
        // Outside the gate nothing happens, and the boss stands still.
        step(&mut w, 0, 0, 0, 40);
        assert_eq!(w.bosses[0].stage, Stage::Dormant);
        // Standing in the arena beside the gate isn't walking through it.
        put_player(&mut w, [6.0, 0.0, 1.0]);
        step(&mut w, 0, 0, 0, 5);
        assert_eq!(w.bosses[0].stage, Stage::Dormant);
        put_player(&mut w, [0.0, 0.0, -0.1]);
        let mut events = step(&mut w, 0, 127, 0, 12);
        let Stage::Intro { left } = w.bosses[0].stage else {
            panic!("{:?}", w.bosses[0].stage);
        };
        // The intro lasts 90 ticks from the tick the gate sealed.
        events.extend(step(&mut w, 0, 0, 0, u32::from(left) - 1));
        assert!(matches!(w.bosses[0].stage, Stage::Intro { left: 1 }));
        assert_eq!(w.actors.get(body).unwrap().pos[2], 12.0);
        // A player who steps back through the sealed gate is put back on its plane.
        put_player(&mut w, [1.0, 0.0, -3.0]);
        events.extend(step(&mut w, 0, 0, 0, 1));
        assert!(w.actors.get(w.player).unwrap().pos[2] >= -0.01);
        events.extend(step(&mut w, 0, 0, 0, 1));
        assert_eq!(w.bosses[0].stage, Stage::Fight);
        assert_eq!(
            bosses(&events),
            [(BossEvent::SEALED, 0), (BossEvent::FIGHT_STARTED, 1)]
        );
        let b = w.actors.get(body).unwrap().ai.as_ref().unwrap();
        assert_eq!((b.awareness, b.phase), (Awareness::Combat, 1));
        // And a sealed gate holds during the fight.
        put_player(&mut w, [0.0, 0.0, -2.0]);
        step(&mut w, 0, 0, 0, 1);
        assert!(w.actors.get(w.player).unwrap().pos[2] >= -0.01);
    }

    #[test]
    fn a_boss_never_leashes_or_loses_sight() {
        let mut w = arena(2);
        let (body, pillar) = cast(&w);
        start_fight_now(&mut w);
        // An ordinary enemy that hasn't seen the player for 240 ticks, or that is farther
        // from its spawn than its leash, goes home. A boss stays in Combat.
        for id in [body, pillar] {
            let a = w.actors.get_mut(id).unwrap();
            a.ai.as_mut().unwrap().unseen_ticks = 1000;
            a.pos[2] += 1.0;
        }
        for _ in 0..30 {
            step(&mut w, 0, 0, 0, 10);
            for id in [body, pillar] {
                let b = w.actors.get(id).unwrap().ai.as_ref().unwrap();
                assert_eq!(b.awareness, Awareness::Combat);
                assert!(b.unseen_ticks < 10);
            }
        }
    }

    #[test]
    fn defeating_the_body_pays_the_rewards_and_the_boss_stays_down() {
        let mut w = arena(3);
        let (body, pillar) = cast(&w);
        start_fight_now(&mut w);
        let before = w.actors.get(w.player).unwrap().inventory.glimmer;
        let vials = Content::load_embedded()
            .unwrap()
            .registry
            .num("itm_glimmer_vial")
            .unwrap();
        let had = w.actors.get(w.player).unwrap().inventory.count(vials);
        let a = w.actors.get_mut(body).unwrap();
        a.hp = 0;
        a.dead = true;
        let events = step(&mut w, 0, 0, 0, 2);
        assert_eq!(bosses(&events), [(BossEvent::DEFEATED, 0)]);
        assert_eq!(w.bosses[0].stage, Stage::Defeated);
        let p = w.actors.get(w.player).unwrap();
        assert_eq!(p.inventory.glimmer, before + 500);
        assert_eq!(p.inventory.count(vials), had + 2);
        assert!(w.state.flags.is_set("boss.test_boss.defeated"));
        assert!(w.state.flags.is_set("codex.test_boss"));
        // The pillar isn't required, but it falls with the body.
        assert!(w.actors.get(pillar).unwrap().dead);
        // The gate is open again.
        put_player(&mut w, [0.0, 0.0, -2.0]);
        step(&mut w, 0, 0, 0, 1);
        assert!(w.actors.get(w.player).unwrap().pos[2] < -1.0);
        // Resting, dying, and respawning don't bring it back.
        crate::world_state::lantern::respawn_enemies(&mut w);
        w.actors.get_mut(w.player).unwrap().hp = 0;
        step(&mut w, 0, 0, 0, 200);
        assert!(!w.actors.get(w.player).unwrap().dead);
        assert!(w.actors.get(body).unwrap().dead);
        assert_eq!(w.bosses[0].stage, Stage::Defeated);
        assert!(bosses(&step(&mut w, 0, 0, 0, 1)).is_empty());
    }

    #[test]
    fn a_boss_beaten_in_the_save_starts_defeated() {
        let mut w = arena(4);
        let (body, _) = cast(&w);
        w.state.flags.raise("boss.test_boss.defeated");
        step(&mut w, 0, 0, 0, 2);
        assert_eq!(w.bosses[0].stage, Stage::Defeated);
        assert!(w.actors.get(body).unwrap().dead);
    }

    #[test]
    fn a_phase_needs_the_hp_threshold_and_the_new_options_unlock_after_the_transition() {
        let mut w = arena(5);
        let (body, _) = cast(&w);
        start_fight_now(&mut w);
        // Just above the 60% threshold: no phase change.
        w.actors.get_mut(body).unwrap().hp = 541;
        step(&mut w, 0, 0, 0, 5);
        assert_eq!(w.bosses[0].phase, 1);
        assert_eq!(w.bosses[0].transition_left, 0);
        w.actors.get_mut(body).unwrap().hp = 539;
        let events = step(&mut w, 0, 0, 0, 1);
        assert_eq!(bosses(&events), [(BossEvent::PHASE_STARTED, 2)]);
        let roar = w.moves.num_of("tb_roar").unwrap();
        // Phase 1 options still apply until the transition ends, and the body is in the
        // move with i-frames on every frame.
        for _ in 0..44 {
            let a = w.actors.get(body).unwrap();
            assert_eq!(a.moves.current.map(|c| c.num), Some(roar));
            assert_eq!(w.actors.get(body).unwrap().ai.as_ref().unwrap().phase, 1);
            step(&mut w, 0, 0, 0, 1);
            let a = w.actors.get(body).unwrap();
            if let Some(c) = a.moves.current {
                assert!(
                    a.moves.flags & crate::combat::moves::flags::IFRAMES != 0,
                    "frame {} of the transition isn't invulnerable",
                    c.frame
                );
            }
        }
        step(&mut w, 0, 0, 0, 2);
        assert_eq!(w.bosses[0].phase, 2);
        assert_eq!(w.actors.get(body).unwrap().ai.as_ref().unwrap().phase, 2);
        // The new phase is picked up by `Condition::Phase`: the slam becomes possible.
        let slam = w.moves.num_of("tb_slam").unwrap();
        let mut used = false;
        put_player(&mut w, [0.0, 0.0, 6.0]);
        for _ in 0..900 {
            let at = w.actors.get(body).unwrap().pos;
            put_player(&mut w, [at[0], 0.0, at[2] - 1.8]);
            step(&mut w, 0, 0, 0, 1);
            used |= w
                .actors
                .get(body)
                .unwrap()
                .moves
                .current
                .is_some_and(|c| c.num == slam);
        }
        assert!(used, "the boss never used its phase 2 move");
    }

    #[test]
    fn a_boss_attack_stops_tracking_four_ticks_before_its_first_active_frame() {
        let mut w = arena(6);
        let (body, _) = cast(&w);
        start_fight_now(&mut w);
        let swipe = w.moves.num_of("tb_swipe").unwrap();
        let until = w.moves.get(swipe).unwrap().track_until;
        let mut yaws = Vec::new();
        let mut angle = 0.0_f32;
        for _ in 0..600 {
            // The player circles the boss at 2 m so that tracking would keep turning it.
            let at = w.actors.get(body).unwrap().pos;
            angle += 0.12;
            put_player(
                &mut w,
                [at[0] + 2.0 * sin(angle), 0.0, at[2] + 2.0 * cos(angle)],
            );
            step(&mut w, 0, 0, 0, 1);
            let a = w.actors.get(body).unwrap();
            match a.moves.current {
                Some(c) if c.num == swipe => yaws.push((c.frame, a.yaw)),
                _ if !yaws.is_empty() => break,
                _ => {}
            }
        }
        assert!(yaws.len() > usize::from(until), "{yaws:?}");
        let yaw_at = |f: u16| yaws.iter().find(|&&(fr, _)| fr == f).unwrap().1;
        // It turned while tracking, and then not at all over the last four frames.
        assert!((yaw_at(2) - yaw_at(until - 6)).abs() > 0.05, "{yaws:?}");
        assert_eq!(yaw_at(until - 3), yaw_at(until), "{yaws:?}");
    }

    #[test]
    fn dying_mid_fight_resets_every_member_at_once() {
        let mut w = arena(7);
        let (body, pillar) = cast(&w);
        start_fight_now(&mut w);
        for id in [body, pillar] {
            let a = w.actors.get_mut(id).unwrap();
            a.hp /= 2;
            a.pos[0] += 3.0;
        }
        w.actors.get_mut(w.player).unwrap().hp = 0;
        let events = step(&mut w, 0, 0, 0, 3);
        assert_eq!(bosses(&events), [(BossEvent::RESET, 0)]);
        for id in [body, pillar] {
            let a = w.actors.get(id).unwrap();
            assert_eq!(a.hp, a.max_hp);
            assert_eq!(a.ai.as_ref().unwrap().awareness, Awareness::Idle);
        }
        assert_eq!(w.actors.get(body).unwrap().pos, [0.0, 0.0, 12.0]);
    }
}
