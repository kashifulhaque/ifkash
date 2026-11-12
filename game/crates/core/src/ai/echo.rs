//! Echo summons: the Echo Bell, Echo Stones, and the ally AI (WP-059). See
//! docs/game/05-enemies-and-bosses.md, "Echo summons".
//!
//! # Summoning
//!
//! The player puts the Echo Bell (`ItemKind::EchoBell`) in a quick slot and uses it
//! (`Command::UseItem`) within `stone_ring_m` of an Echo Stone. The selected Echo's bodies
//! appear around the stone and the Echo's FP cost is spent. The selection is the owned Echo
//! at index [`EchoRun::selected`] of the owned list (sorted by registry number); a press of
//! `CYCLE_SPELL` while the Bell is the selected quick item steps it on. An Echo is owned if
//! the scenario lists it in `echoes_unlocked` or its `unlock_flag` is a raised world flag.
//! One Echo is out at a time: ringing again while one stands does nothing but report why.
//! A summoning that fails emits `EchoEvent::REFUSED` with a reason. The Echo is removed when
//! every body has fallen (after `fallen_ticks`), when the player dies, or when they rest.
//!
//! # The ally AI
//!
//! A body is an ordinary AI actor (the Echo's `body` is an enemy definition, so its options,
//! moves, guard, and speed work as an enemy's) marked an ally (`Actor::is_ally`). [`run`] drives it
//! each tick in place of `ai::run`'s enemy loop, on the same 10 Hz stagger:
//!
//! - **Following.** With no target it walks to within `follow_m` of the player, running when
//!   it is `run_beyond_m` past that, along the nav grid when a wall is in the way. Past
//!   `catch_up_m` it is put behind the player.
//! - **Engaging the player's target.** Its target is the enemy the player is locked on to, if
//!   it is within `engage_m` of the player, or else the nearest enemy in Combat that is.
//!   Enemies that haven't noticed anyone and bosses that haven't started stay alone. Past
//!   `leash_m` from the player it gives the chase up and follows.
//! - **Reverse attack tokens.** It picks and starts options with the same code as an enemy
//!   (`options::decide`), so it takes a melee or ranged token against its target from the
//!   same pools: at most `melee_tokens` Echoes attack one enemy at once. A body whose
//!   definition sets `ignores_tokens` skips them.
//! - **Drawing aggro.** Enemies in Combat within the Echo's `aggro_m` of a living body turn on
//!   it instead of the player ([`aggro_target`]; bosses through [`redirect`]).
//!
//! The Echo's attacks and arrows pass through the player and the other bodies, and enemies'
//! hit the bodies (`ai::attack::allied`). The player can't lock on to a body or riposte it.
//!
//! # The bow
//!
//! `Command::Gesture(bow_gesture)` within `bow_window_ticks` of the summoning, with the player
//! within `bow_range_m` of a living body whose Echo has a `courtesy`, sets the Echo's attack
//! to the courtesy's `damage_percent` ([`damage_percent`]) and turns its bodies to face the
//! player. The gesture's own move (WP-088) plays whatever it plays: the bow here is only the
//! command.
//!
//! # State
//!
//! [`EchoRun`] is in [`World::echo`], hashed only while an Echo is out or the selection has
//! moved, so every world that never rings the Bell hashes as before. Echo bodies are in the
//! actor arena like any AI actor, with `ally` set (hashed only while true).

use lf_content::enemy::AiTuning;
use lf_content::scenario::{EchoStone, EnemySpawn};
use lf_content::{Content, EchoDef, EchoRules, NumId, Scenario};
use serde::{Deserialize, Serialize};

use super::perception::dist_xz;
use super::{Awareness, Doing, attack, behaviors, boss, options, pathing, thinks_on, tokens};
use crate::actor::targeting::bearing;
use crate::arena::Id;
use crate::combat::hurtbox::Hurtbox;
use crate::combat::moves::turn_toward;
use crate::events::{EchoEvent, Event};
use crate::input::{Command, TickInput, buttons};
use crate::items::slot;
use crate::math::dmath::{cos, sin};
use crate::world::World;

/// The spawn index of an Echo body. No scenario spawn has it: an Echo has no home to leash
/// to or patrol, and the code that looks a spawn up gets nothing. It is also what marks an
/// actor as an Echo (`Actor::is_ally`).
pub const NO_SPAWN: u16 = u16::MAX;
/// Degrees to radians. Plain multiplication, no transcendental call.
const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;
/// How far in front of an Echo Stone the bodies appear, in meters. A placement detail, not a
/// gameplay number: it only keeps them off the stone itself.
const SUMMON_AHEAD_M: f32 = 1.5;

/// One Echo, resolved from content.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EchoInfo {
    /// The Echo's registry number.
    pub num: NumId,
    pub def: EchoDef,
}

/// The Echo rules, the Echoes, and the scenario's Echo Stones. Not hashed: the content hash
/// covers them.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EchoTable {
    /// `None` when the content has no `echoes/rules.ron`; then no Echo can be summoned.
    pub rules: Option<EchoRules>,
    /// The Echo Bell's registry number.
    pub bell: Option<NumId>,
    /// Sorted by `num`.
    pub echoes: Vec<EchoInfo>,
    pub stones: Vec<EchoStone>,
    /// The Echoes the scenario gives the player outright.
    pub unlocked: Vec<NumId>,
}

impl EchoTable {
    /// Resolves the Echo rules and definitions and the scenario's stones.
    pub fn from_content(content: &Content, level: &Scenario) -> Result<Self, String> {
        let Some(rules) = content.echo_rules.clone() else {
            return Ok(Self::default());
        };
        let reg = &content.registry;
        let bell = reg.num(&rules.bell).ok_or_else(|| {
            format!(
                "echo rules: the bell \"{}\" is not in the registry",
                rules.bell
            )
        })?;
        let mut echoes = Vec::new();
        for def in content.echoes.values() {
            let num = reg
                .num(&def.id)
                .ok_or_else(|| format!("Echo \"{}\" is missing from registry.ron", def.id))?;
            echoes.push(EchoInfo {
                num,
                def: def.clone(),
            });
        }
        echoes.sort_by_key(|e| e.num);
        let unlocked = level
            .echoes_unlocked
            .iter()
            .map(|id| {
                reg.num(id)
                    .ok_or_else(|| format!("echoes_unlocked: \"{id}\" is not in the registry"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            rules: Some(rules),
            bell: Some(bell),
            echoes,
            stones: level.echo_stones.clone(),
            unlocked,
        })
    }

    /// The Echo with registry number `num`.
    pub fn get(&self, num: NumId) -> Option<&EchoInfo> {
        self.echoes
            .binary_search_by_key(&num, |e| e.num)
            .ok()
            .map(|i| &self.echoes[i])
    }
}

/// One body of a summoned Echo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    pub id: Id,
    /// Its fall has been reported.
    pub fell: bool,
}

/// The Echo that is out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summoned {
    /// The Echo's registry number.
    pub echo: NumId,
    pub members: Vec<Member>,
    /// Ticks since the summoning.
    pub age: u32,
    /// The player bowed in time (the courtesy applies).
    pub bowed: bool,
    /// Ticks since the Echo's heal last fired.
    pub heal_clock: u16,
    /// Ticks every body has been down.
    pub fallen_ticks: u16,
}

/// The Echo state of a world. Simulation state: part of the state hash while it isn't at rest.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EchoRun {
    /// Which owned Echo the Bell calls: an index into the owned list, wrapped.
    pub selected: u16,
    /// `CYCLE_SPELL` was held last tick with the Bell selected (for the press edge).
    pub prev_cycle: bool,
    pub active: Option<Summoned>,
}

impl EchoRun {
    /// True when nothing has happened: the state a world that never rings the Bell stays in.
    pub fn is_inert(&self) -> bool {
        *self == Self::default()
    }
}

/// The indexes (into [`EchoTable::echoes`]) of the Echoes the player owns.
pub fn owned(world: &World) -> Vec<usize> {
    let t = &world.echo_table;
    t.echoes
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            t.unlocked.contains(&e.num) || world.state.flags.is_set(&e.def.unlock_flag)
        })
        .map(|(i, _)| i)
        .collect()
}

/// The Echo the Bell would call now: an index into [`EchoTable::echoes`].
pub fn selected(world: &World) -> Option<usize> {
    let owned = owned(world);
    (!owned.is_empty()).then(|| owned[usize::from(world.echo.selected) % owned.len()])
}

fn emit(world: &mut World, kind: u8, echo: u32, actor: Option<Id>, reason: u8) {
    world.events.push(Event::Echo(EchoEvent {
        kind,
        echo,
        actor,
        reason,
    }));
}

/// The item in the player's quick slot `slot` (0 to 4).
fn quick_item(world: &World, slot: u8) -> Option<NumId> {
    let p = world.actors.get(world.player)?;
    p.inventory
        .equipment
        .slots
        .get(usize::from(slot::QUICK_1) + usize::from(slot))
        .copied()
        .flatten()
}

/// Tick system 1's Echo half: the Bell (`UseItem` on its quick slot, and `CYCLE_SPELL` to
/// pick an Echo) and the bow gesture. Quick and free when the scenario has no Echo Stone and
/// no Echo is out.
pub fn apply_commands(world: &mut World, input: &TickInput) {
    let (Some(rules), Some(bell)) = (world.echo_table.rules.as_ref(), world.echo_table.bell) else {
        return;
    };
    let bow = rules.bow_gesture;
    if world.echo_table.stones.is_empty() && world.echo.active.is_none() {
        return;
    }
    // The player's death or a rest at a Lantern sends the Echo away.
    let down = world.actors.get(world.player).is_none_or(|p| p.dead);
    if down || world.state.resting.is_some() {
        dismiss(world);
        return;
    }
    // CYCLE_SPELL picks the next owned Echo while the Bell is the selected quick item.
    let selected_quick = world
        .actors
        .get(world.player)
        .map(|p| p.inventory.equipment.quick_active);
    let bell_out = selected_quick.is_some_and(|q| quick_item(world, q) == Some(bell));
    let held = bell_out && input.frame.pressed(buttons::CYCLE_SPELL);
    if held && !world.echo.prev_cycle {
        world.echo.selected = world.echo.selected.wrapping_add(1);
        let n = owned(world).len();
        if n > 0 {
            world.echo.selected %= n as u16;
        }
    }
    world.echo.prev_cycle = held;

    for c in &input.commands {
        match *c {
            Command::UseItem { slot } if slot < slot::QUICK_COUNT => {
                if quick_item(world, slot) == Some(bell) {
                    summon(world);
                }
            }
            Command::Gesture(g) if g == bow => bow_to(world),
            _ => {}
        }
    }
}

/// Rings the Bell: summons the selected Echo at the Echo Stone in reach.
fn summon(world: &mut World) {
    let Some(rules) = world.echo_table.rules.as_ref() else {
        return;
    };
    let ring_m = rules.stone_ring_m;
    let refuse = |world: &mut World, echo: u32, reason: u8| {
        emit(world, EchoEvent::REFUSED, echo, None, reason);
    };
    if world.echo.active.is_some() {
        let echo = world.echo.active.as_ref().map_or(0, |a| a.echo);
        refuse(world, echo, EchoEvent::REASON_ALREADY);
        return;
    }
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    let (ppos, fp) = (p.pos, p.fp);
    let stone = world
        .echo_table
        .stones
        .iter()
        .filter(|s| dist_xz(ppos, s.pos) <= ring_m)
        .min_by(|a, b| dist_xz(ppos, a.pos).total_cmp(&dist_xz(ppos, b.pos)))
        .cloned();
    let Some(stone) = stone else {
        refuse(world, 0, EchoEvent::REASON_NO_STONE);
        return;
    };
    let Some(idx) = selected(world) else {
        refuse(world, 0, EchoEvent::REASON_NO_ECHO);
        return;
    };
    let info = world.echo_table.echoes[idx].clone();
    if fp < f32::from(info.def.fp_cost) {
        refuse(world, info.num, EchoEvent::REASON_NO_FP);
        return;
    }
    let hurtbox = Hurtbox::from_locomotion(&world.locomotion);
    // The bodies stand in a row across the stone's facing, a step in front of it.
    let (fwd, right) = (
        [sin(stone.yaw), cos(stone.yaw)],
        [cos(stone.yaw), -sin(stone.yaw)],
    );
    let n = info.def.count;
    let mut members = Vec::new();
    for i in 0..n {
        let across = (f32::from(i) - f32::from(n - 1) * 0.5) * info.def.spacing_m;
        let spawn = EnemySpawn {
            name: format!("{}#{i}", info.def.id),
            enemy: info.def.body.id.clone(),
            pos: [
                stone.pos[0] + fwd[0] * SUMMON_AHEAD_M + right[0] * across,
                stone.pos[1],
                stone.pos[2] + fwd[1] * SUMMON_AHEAD_M + right[1] * across,
            ],
            yaw: stone.yaw,
            patrol: Vec::new(),
            wakes_on: None,
        };
        let Some(mut actor) = super::enemy_actor(&spawn, NO_SPAWN, &world.ai, hurtbox) else {
            continue;
        };
        // An Echo is nobody's backstab, and it starts out following.
        actor.poise.unaware = false;
        if let Some(b) = actor.ai.as_mut() {
            b.awareness = Awareness::Suspicious;
        }
        members.push(Member {
            id: world.actors.insert(actor),
            fell: false,
        });
    }
    if members.is_empty() {
        refuse(world, info.num, EchoEvent::REASON_NO_ECHO);
        return;
    }
    if let Some(p) = world.actors.get_mut(world.player) {
        p.fp = (p.fp - f32::from(info.def.fp_cost)).max(0.0);
    }
    let first = members.first().map(|m| m.id);
    world.echo.active = Some(Summoned {
        echo: info.num,
        members,
        age: 0,
        bowed: false,
        heal_clock: 0,
        fallen_ticks: 0,
    });
    emit(world, EchoEvent::SUMMONED, info.num, first, 0);
}

/// The player bows: if it is in time and the Echo has a courtesy, the Echo hits harder.
fn bow_to(world: &mut World) {
    let Some(rules) = world.echo_table.rules.as_ref() else {
        return;
    };
    let (window, range) = (rules.bow_window_ticks, rules.bow_range_m);
    let Some(active) = world.echo.active.as_ref() else {
        return;
    };
    let Some(info) = world.echo_table.get(active.echo) else {
        return;
    };
    if active.bowed || info.def.courtesy.is_none() || active.age > u32::from(window) {
        return;
    }
    let Some(ppos) = world.actors.get(world.player).map(|p| p.pos) else {
        return;
    };
    let echo = active.echo;
    let near: Vec<Id> = active
        .members
        .iter()
        .map(|m| m.id)
        .filter(|&id| {
            world
                .actors
                .get(id)
                .is_some_and(|a| !a.dead && dist_xz(a.pos, ppos) <= range)
        })
        .collect();
    let Some(&first) = near.first() else {
        return;
    };
    if let Some(a) = world.echo.active.as_mut() {
        a.bowed = true;
    }
    // Every body turns to the player: the bow is returned.
    for id in near {
        if let Some(a) = world.actors.get_mut(id) {
            a.yaw = bearing(a.pos, ppos, a.yaw);
        }
    }
    emit(world, EchoEvent::BOWED, echo, Some(first), 0);
}

/// Removes the Echo's bodies and clears the summoning.
fn dismiss(world: &mut World) {
    let Some(active) = world.echo.active.take() else {
        return;
    };
    for m in &active.members {
        world.actors.remove(m.id);
    }
    // Their arrows go with them.
    let shot: Vec<Id> = world
        .projectiles
        .iter()
        .filter(|(_, p)| active.members.iter().any(|m| m.id == p.shooter))
        .map(|(id, _)| id)
        .collect();
    for id in shot {
        world.projectiles.remove(id);
    }
    emit(world, EchoEvent::DISMISSED, active.echo, None, 0);
}

/// The attack rating `id` hits with, as a percent of its definition's: the courtesy's after
/// a bow, 100 for everyone else.
pub fn damage_percent(world: &World, id: Id) -> u16 {
    let Some(active) = world.echo.active.as_ref() else {
        return 100;
    };
    if !active.bowed || !active.members.iter().any(|m| m.id == id) {
        return 100;
    }
    world
        .echo_table
        .get(active.echo)
        .and_then(|i| i.def.courtesy)
        .map_or(100, |c| c.damage_percent)
}

/// True if `id` is a body of the Echo that is out.
pub fn is_member(world: &World, id: Id) -> bool {
    world
        .echo
        .active
        .as_ref()
        .is_some_and(|a| a.members.iter().any(|m| m.id == id))
}

/// The living body of the Echo that is out that `enemy` should turn on, if any: the nearest
/// one within the Echo's `aggro_m` of it.
pub fn aggro_target(world: &World, enemy: Id) -> Option<Id> {
    let active = world.echo.active.as_ref()?;
    let reach = world.echo_table.get(active.echo)?.def.aggro_m;
    if reach <= 0.0 {
        return None;
    }
    let e = world.actors.get(enemy).filter(|e| !e.is_ally())?;
    let mut best: Option<(f32, Id)> = None;
    for m in &active.members {
        let Some(a) = world.actors.get(m.id).filter(|a| !a.dead && a.hp > 0) else {
            continue;
        };
        let d = dist_xz(e.pos, a.pos);
        if d <= reach && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, m.id));
        }
    }
    best.map(|(_, id)| id)
}

/// Who `enemy` fights and where it last saw them: the Echo it is drawn to, or else `player`
/// at `at`. For the boss framework, which holds its members on a target itself.
pub fn redirect(world: &World, enemy: Id, player: Id, at: [f32; 3]) -> (Id, [f32; 3]) {
    match aggro_target(world, enemy).and_then(|e| world.actors.get(e).map(|a| (e, a.pos))) {
        Some(x) => x,
        None => (player, at),
    }
}

/// `enemy` turns on `target` (an Echo): it steers and attacks toward it, and can't lose
/// sight of it.
pub(crate) fn turn_on(world: &mut World, enemy: Id, target: Id) {
    let Some(at) = world.actors.get(target).map(|a| a.pos) else {
        return;
    };
    let Some(a) = world.actors.get_mut(enemy) else {
        return;
    };
    // A move in progress keeps the lock it started with (a boss's attack stops tracking
    // before its first active frame).
    if a.moves.current.is_none() {
        a.lock_target = Some(target);
    }
    if let Some(b) = a.ai.as_mut() {
        b.target = Some(target);
        b.last_seen = at;
        b.unseen_ticks = 0;
    }
}

/// `enemy` goes back to the player after fighting an Echo.
pub(crate) fn turn_back(world: &mut World, enemy: Id, player: Id) {
    let Some(a) = world.actors.get_mut(enemy) else {
        return;
    };
    let Some(b) = a.ai.as_mut() else {
        return;
    };
    if b.target.is_some() && b.target != Some(player) {
        b.target = Some(player);
        if a.moves.current.is_none() {
            a.lock_target = Some(player);
        }
    }
}

/// What the ally AI reads from the rules each tick.
#[derive(Clone, Copy)]
struct Knobs {
    follow_m: f32,
    catch_up_m: f32,
    engage_m: f32,
    leash_m: f32,
    run_beyond_m: f32,
    fallen_ticks: u16,
}

/// The Echo half of tick system 3: reports falls, heals, clears a fallen Echo, and thinks
/// and moves every living body.
pub fn run(world: &mut World, t: &AiTuning) {
    let Some(r) = world.echo_table.rules.as_ref() else {
        return;
    };
    let k = Knobs {
        follow_m: r.follow_m,
        catch_up_m: r.catch_up_m,
        engage_m: r.engage_m,
        leash_m: r.leash_m,
        run_beyond_m: r.run_beyond_m,
        fallen_ticks: r.fallen_ticks,
    };
    let Some(active) = world.echo.active.as_mut() else {
        return;
    };
    active.age = active.age.saturating_add(1);
    let echo = active.echo;
    let members: Vec<Member> = active.members.clone();
    // Falls, one event each.
    let mut fell = Vec::new();
    for (i, m) in members.iter().enumerate() {
        let down = world.actors.get(m.id).is_none_or(|a| a.dead || a.hp == 0);
        if down && !m.fell {
            fell.push((i, m.id));
        }
    }
    for &(i, id) in &fell {
        if let Some(a) = world.echo.active.as_mut() {
            a.members[i].fell = true;
        }
        emit(world, EchoEvent::FELL, echo, Some(id), 0);
    }
    let all_down = world
        .echo
        .active
        .as_ref()
        .is_some_and(|a| a.members.iter().all(|m| m.fell));
    if all_down {
        let done = world.echo.active.as_mut().is_some_and(|a| {
            a.fallen_ticks = a.fallen_ticks.saturating_add(1);
            a.fallen_ticks >= k.fallen_ticks
        });
        if done {
            dismiss(world);
        }
        return;
    }
    heal(world, echo);
    for m in members {
        if world
            .echo
            .active
            .as_ref()
            .is_some_and(|a| a.members.iter().any(|x| x.id == m.id && x.fell))
        {
            continue;
        }
        tick_body(world, m.id, t, &k);
    }
}

/// The Choir Wisp's slow heal: every `every_ticks`, the player gains `hp` if a living body
/// is within `range_m` of them.
fn heal(world: &mut World, echo: NumId) {
    let Some(h) = world.echo_table.get(echo).and_then(|i| i.def.heal) else {
        return;
    };
    let Some(active) = world.echo.active.as_mut() else {
        return;
    };
    active.heal_clock = active.heal_clock.saturating_add(1);
    if active.heal_clock < h.every_ticks {
        return;
    }
    let ppos = world.actors.get(world.player).map(|p| p.pos);
    let Some(ppos) = ppos else {
        return;
    };
    let members: Vec<Id> = active.members.iter().map(|m| m.id).collect();
    let near = members.iter().any(|&id| {
        world
            .actors
            .get(id)
            .is_some_and(|a| !a.dead && a.hp > 0 && dist_xz(a.pos, ppos) <= f32::from(h.range_m))
    });
    if !near {
        // Wait for it to get close: the heal fires on the first tick it can.
        return;
    }
    if let Some(a) = world.echo.active.as_mut() {
        a.heal_clock = 0;
    }
    if let Some(p) = world.actors.get_mut(world.player)
        && !p.dead
    {
        p.hp = (p.hp + h.hp).min(p.max_hp);
    }
}

/// One tick of one body: tokens, stamina, a think on its stagger, then movement.
fn tick_body(world: &mut World, id: Id, t: &AiTuning, k: &Knobs) {
    tokens::release_finished(world, id);
    if world.actors.get(id).is_none_or(|a| a.dead) {
        return;
    }
    attack::regen_stamina(world, id);
    if thinks_on(world.tick, id.index, t.think_every_ticks) {
        think_body(world, id, t, k);
    }
    let fighting = world
        .actors
        .get(id)
        .and_then(|a| a.ai.as_ref())
        .is_some_and(|b| b.awareness == Awareness::Combat);
    if fighting {
        behaviors::execute(world, id, t);
    } else {
        follow(world, id, t, k);
    }
}

/// True if `e` is an enemy a body may fight: alive, not an Echo, not waiting for its boss
/// fight, and not a dormant mimic.
fn fightable(world: &World, e: Id) -> bool {
    world
        .actors
        .get(e)
        .is_some_and(|a| !a.dead && a.hp > 0 && !a.is_ally() && a.ai.is_some())
        && !boss::benched(world, e)
        && !super::special::mimic::dormant(world, e)
}

/// The enemy `me` fights: the player's lock target, or else the nearest enemy in Combat,
/// among those within `engage_m` of the player. Ties go to the lower arena index.
fn pick_target(world: &World, me: Id, k: &Knobs) -> Option<Id> {
    let p = world.actors.get(world.player)?;
    let near_player = |e: Id| {
        world
            .actors
            .get(e)
            .is_some_and(|a| dist_xz(a.pos, p.pos) <= k.engage_m)
    };
    if let Some(lt) = p.lock_target
        && fightable(world, lt)
        && near_player(lt)
    {
        return Some(lt);
    }
    let mine = world.actors.get(me)?.pos;
    let mut best: Option<(f32, Id)> = None;
    for (id, a) in world.actors.iter() {
        let in_combat =
            a.ai.as_ref()
                .is_some_and(|b| b.awareness == Awareness::Combat);
        if !in_combat || !fightable(world, id) || !near_player(id) {
            continue;
        }
        let d = dist_xz(mine, a.pos);
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, id));
        }
    }
    best.map(|(_, id)| id)
}

/// A body's think: catch up, pick a target, and start an option against it, or go back to
/// following.
fn think_body(world: &mut World, id: Id, t: &AiTuning, k: &Knobs) {
    let player = world.player;
    let Some(p) = world.actors.get(player).map(|p| (p.pos, p.yaw)) else {
        return;
    };
    let Some(me) = world.actors.get(id).map(|a| a.pos) else {
        return;
    };
    let d = dist_xz(me, p.0);
    if d > k.catch_up_m {
        // Put it behind the player.
        let back = p.1 + core::f32::consts::PI;
        if let Some(a) = world.actors.get_mut(id) {
            a.pos = [
                p.0[0] + sin(back) * k.follow_m,
                p.0[1],
                p.0[2] + cos(back) * k.follow_m,
            ];
            a.vel = [0.0; 3];
            a.hit.prev = None;
        }
    }
    let target = if d > k.leash_m {
        None
    } else {
        pick_target(world, id, k)
    };
    match target {
        Some(e) => {
            let at = world.actors.get(e).map_or([0.0; 3], |a| a.pos);
            if let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
                b.awareness = Awareness::Combat;
                b.target = Some(e);
                b.last_seen = at;
                b.unseen_ticks = 0;
            }
            options::decide(world, id, e, t);
        }
        None => {
            if let Some(a) = world.actors.get_mut(id) {
                if a.moves.current.is_none() {
                    a.lock_target = None;
                }
                if let Some(b) = a.ai.as_mut() {
                    b.awareness = Awareness::Suspicious;
                    b.target = None;
                    if matches!(b.doing, Doing::Behavior(_)) {
                        b.doing = Doing::Nothing;
                        b.doing_ticks = 0;
                    }
                }
            }
        }
    }
}

/// Walks a body to within `follow_m` of the player, and faces them once it gets there. Does
/// what `behaviors::execute` does for its timers and for staggers and moves.
fn follow(world: &mut World, id: Id, t: &AiTuning, k: &Knobs) {
    let lift = world.locomotion.capsule_half_height + world.locomotion.capsule_radius;
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    let Some(b) = a.ai.as_mut() else {
        return;
    };
    b.state_ticks = b.state_ticks.saturating_add(1);
    for c in &mut b.cooldowns {
        *c = c.saturating_sub(1);
    }
    b.since_hurt = b.since_hurt.saturating_add(1);
    b.guarding = false;
    if a.moves.current.is_some() {
        return;
    }
    if a.moves.guard.stagger > 0 || !a.motor.grounded {
        if a.motor.grounded {
            a.vel[0] = 0.0;
            a.vel[2] = 0.0;
        }
        return;
    }
    let (pos, def) = (a.pos, a.ai.as_ref().map_or(0, |b| b.def));
    let Some(ppos) = world.actors.get(world.player).map(|p| p.pos) else {
        return;
    };
    let d = dist_xz(pos, ppos);
    let scale = f32::from(world.ai.get(def).map_or(100, |d| d.data.speed_percent)) / 100.0;
    let vel = if d <= k.follow_m {
        None
    } else {
        // Stop `follow_m` short of the player, on the line between them.
        let f = k.follow_m / d;
        let goal = [
            ppos[0] + (pos[0] - ppos[0]) * f,
            ppos[1],
            ppos[2] + (pos[2] - ppos[2]) * f,
        ];
        let speed = if d > k.follow_m + k.run_beyond_m {
            t.run_speed
        } else {
            t.walk_speed
        };
        pathing::steer(world, id, lift, pos, goal, speed * scale, t)
    };
    let Some(a) = world.actors.get_mut(id) else {
        return;
    };
    let [vx, vz] = vel.unwrap_or([0.0, 0.0]);
    a.vel[0] = vx;
    a.vel[2] = vz;
    // Face the way it walks, or the player when it stands.
    let want = if vel.is_some() {
        bearing([0.0; 3], [vx, 0.0, vz], a.yaw)
    } else {
        bearing(pos, ppos, a.yaw)
    };
    a.yaw = turn_toward(a.yaw, want, t.turn_deg_per_tick * DEG_TO_RAD);
}

#[cfg(test)]
mod tests;
