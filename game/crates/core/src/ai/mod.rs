//! Enemy AI: awareness, perception, option selection, built-in behaviors, attack tokens,
//! leashing, and return-and-heal. See docs/game/05-enemies-and-bosses.md, "AI framework",
//! and docs/game/01-architecture.md, "Tick schedule" (system 3, AI think).
//!
//! # Per tick
//!
//! [`run`] is tick system 3. For every live AI actor, in arena order, it:
//!
//! 1. Releases a finished attack's token (see [`tokens`]) and counts down timers.
//! 2. Listens: the player's noise this tick (see [`perception::player_noise`]) is heard
//!    if the actor is inside its radius.
//! 3. Thinks, on one tick in `think_every_ticks` (10 Hz), staggered by arena index (see
//!    [`thinks_on`]): sight, awareness changes, leashing, and in Combat a new option once
//!    the last one is done (see [`options`]).
//! 4. Executes the current awareness state or behavior: a velocity and a facing for
//!    locomotion (see [`behaviors`]). An attack move owns the velocity until it ends.
//!
//! The target is always the player for now; Echoes (summoned allies) can add targets.
//!
//! # Extension points
//!
//! - Bosses (WP-033, [`boss`]): [`AiState::phase`] feeds `Condition::Phase`, and
//!   `EnemyDef::ignores_tokens` skips the token pools. [`run`] calls [`boss::run`] first.
//! - Special rules (WP-032, `ai/special/`): run after [`run`]'s think for an actor whose
//!   definition needs one, for example the skeleton's revive.
//! - Navigation (WP-032 with WP-031's grid): [`behaviors::desired_move`] is the steering
//!   seam; it moves straight at the goal and stops at a wall.

pub mod attack;
pub mod behaviors;
pub mod boss;
pub mod options;
pub mod pathing;
pub mod perception;
pub mod special;
pub mod tokens;

use lf_content::enemy::{AiTuning, Behavior, Condition, Perception};
use lf_content::scenario::EnemySpawn;
use lf_content::{Content, EnemyDef, NumId};
use serde::{Deserialize, Serialize};

use crate::actor::stats::{Defense, Stats};
use crate::actor::{Actor, Motor};
use crate::arena::Id;
use crate::combat::hurtbox::Hurtbox;
use crate::combat::moves::{MoveNum, MoveTable};
use crate::combat::poise::PoiseState;
use crate::combat::status::StatusState;
use crate::events::{AwarenessEvent, Event};
use crate::world::World;

/// The awareness states (docs "Awareness states").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Awareness {
    /// Stands or patrols. Suspicious on a sound; Aware on sight or a hit.
    #[default]
    Idle,
    /// Walks to the last noise for up to `suspicious_ticks`. Aware on sight; Idle after.
    Suspicious,
    /// The notice: stands facing the target for 15 to 30 ticks, then Combat.
    Aware,
    /// Picks options until leashed or the target is lost.
    Combat,
    /// Walks home, ignoring the player, and heals to full on arrival. Then Idle.
    Return,
}

impl Awareness {
    /// The code in the WASM event stream (kind 9).
    pub fn code(self) -> u32 {
        match self {
            Self::Idle => 0,
            Self::Suspicious => 1,
            Self::Aware => 2,
            Self::Combat => 3,
            Self::Return => 4,
        }
    }
}

/// What a Combat actor is doing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Doing {
    /// Nothing: the next think picks an option.
    #[default]
    Nothing,
    /// A built-in behavior, for `AiState::doing_ticks` more ticks.
    Behavior(Behavior),
    /// Option `option`'s attack move, until the move ends.
    Attack { option: u16, move_num: MoveNum },
}

/// The attack token an actor holds (docs "Attack tokens").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Token {
    #[default]
    None,
    Melee,
    Ranged,
}

/// One AI actor's state. Simulation state: part of the state hash.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiState {
    /// The enemy's registry number. Not an index into `AiTable::defs`: an index shifts
    /// when an enemy is added, which would change the hash of unrelated fights.
    pub def: NumId,
    /// Index into `Layout::enemies`: the spawn it leashes to and its patrol.
    pub spawn: u16,
    pub awareness: Awareness,
    /// Ticks in the current awareness state.
    pub state_ticks: u16,
    /// How long this Aware notice lasts, drawn on entering Aware.
    pub notice_ticks: u16,
    /// Who it's engaging, from Aware until it returns.
    pub target: Option<Id>,
    /// Where it last saw its target. Behaviors steer and face toward this, not the
    /// target itself, so a target out of sight is lost.
    pub last_seen: [f32; 3],
    /// Ticks since it last saw its target (Combat).
    pub unseen_ticks: u16,
    /// The noise a Suspicious actor investigates.
    pub noise_at: [f32; 3],
    /// The latest noise heard since the last think.
    pub heard: Option<[f32; 3]>,
    pub doing: Doing,
    /// Ticks left of a `Doing::Behavior`.
    pub doing_ticks: u16,
    /// Strafe direction for this behavior: true is to its left.
    pub strafe_left: bool,
    /// Ticks left on each option's cooldown, by option index.
    pub cooldowns: Vec<u16>,
    pub token: Token,
    /// Whom the token is held against. Kept until the token is released, even if the
    /// actor leashes mid-attack.
    pub token_target: Option<Id>,
    /// The patrol waypoint it's walking to.
    pub patrol_i: u16,
    /// HP at the last think, to notice being hit.
    pub last_hp: u32,
    /// Boss phase for `Condition::Phase`, from 1; 0 for regular enemies (see [`boss`]).
    pub phase: u8,
    /// The guard is held up (an enemy's `Guard` behavior); `combat::moves::step` reads it
    /// like the player's guard button (WP-032).
    pub guarding: bool,
    /// Ticks since it last lost HP, saturating (`Condition::HurtWithin`). `u16::MAX` at spawn.
    pub since_hurt: u16,
    /// The nav path it follows: waypoints still to walk, nearest first (WP-032, see
    /// [`pathing`]). Empty when it steers straight.
    pub path: Vec<[f32; 3]>,
    /// The goal `path` was requested for.
    pub path_goal: [f32; 3],
    /// A path request in the queue, by ticket.
    pub path_ticket: Option<u32>,
    /// Ticks until it may ask for a path again after a failed request.
    pub path_retry: u16,
    /// Ticks lying dead, for a reviver (see [`special`]).
    pub dead_ticks: u16,
    /// The killing blow keeps it down for good (see [`special`]).
    pub stays_dead: bool,
    /// Hidden in a reed patch (see [`special::reeds`]).
    pub hidden: bool,
    /// Ticks left that an attack keeps it revealed.
    pub reveal: u16,
    /// Ticks in a row in contact with its target (see [`special::swarm`]).
    pub contact_ticks: u16,
    /// Its blast move is playing; it dies when the move ends.
    pub exploding: bool,
    /// Ticks left that a Blunt hit keeps its crystals cracked (see [`special::reflect`]).
    pub cracked: u16,
}

/// What an option does once picked.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum OptionKind {
    /// Start this move. `ranged` (it launches projectiles) picks the ranged token pool.
    Move {
        num: MoveNum,
        ranged: bool,
    },
    Behavior(Behavior),
}

/// An option with its move resolved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptionDef {
    pub kind: OptionKind,
    pub when: Vec<Condition>,
    pub weight: u16,
    pub cooldown: u16,
}

/// An enemy definition with its references and defaults resolved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiDef {
    pub num: NumId,
    pub data: EnemyDef,
    pub perception: Perception,
    pub leash_m: f32,
    pub options: Vec<OptionDef>,
}

/// The AI tuning and every enemy definition, from content. Not hashed: the content hash
/// covers it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiTable {
    /// `None` when the content has no `ai.ron`; then no AI actor can exist.
    pub tuning: Option<AiTuning>,
    /// Sorted by `num`.
    pub defs: Vec<AiDef>,
}

impl AiTable {
    /// Resolves every enemy definition. Fails on an option move that isn't defined (the
    /// validator reports those too).
    pub fn from_content(content: &Content, moves: &MoveTable) -> Result<Self, String> {
        let mut defs = Vec::new();
        for e in content.enemies.values() {
            let num = content
                .registry
                .num(&e.id)
                .ok_or_else(|| format!("enemy \"{}\" is missing from registry.ron", e.id))?;
            let tuning = content.ai;
            let mut opts = Vec::new();
            for o in &e.options {
                let kind = match (&o.move_id, o.behavior) {
                    (Some(id), None) => {
                        let num = moves
                            .num_of(id)
                            .ok_or_else(|| format!("enemy \"{}\": unknown move \"{id}\"", e.id))?;
                        let ranged = moves
                            .get(num)
                            .is_some_and(|d| !d.data.projectiles.is_empty());
                        OptionKind::Move { num, ranged }
                    }
                    (None, Some(b)) => OptionKind::Behavior(b),
                    _ => {
                        return Err(format!(
                            "enemy \"{}\": an option needs exactly one of move and behavior",
                            e.id
                        ));
                    }
                };
                opts.push(OptionDef {
                    kind,
                    when: o.when.clone(),
                    weight: o.weight,
                    cooldown: o.cooldown,
                });
            }
            let (perception, leash_m) = match tuning {
                Some(t) => (
                    e.perception.unwrap_or(t.perception),
                    e.leash_m.unwrap_or(t.leash_m),
                ),
                None => (
                    e.perception.unwrap_or(Perception {
                        sight_deg: 0.0,
                        sight_m: 0.0,
                        hearing_mult: 0.0,
                    }),
                    e.leash_m.unwrap_or(0.0),
                ),
            };
            defs.push(AiDef {
                num,
                data: e.clone(),
                perception,
                leash_m,
                options: opts,
            });
        }
        defs.sort_by_key(|d| d.num);
        Ok(Self {
            tuning: content.ai,
            defs,
        })
    }

    /// The definition of the enemy with registry number `num`.
    pub fn get(&self, num: NumId) -> Option<&AiDef> {
        self.defs
            .binary_search_by_key(&num, |d| d.num)
            .ok()
            .map(|i| &self.defs[i])
    }

    /// The definition of the enemy with registry number `num`. Panics if there is none:
    /// an `AiState` is only built from a defined enemy.
    pub fn def(&self, num: NumId) -> &AiDef {
        self.get(num)
            .unwrap_or_else(|| panic!("no enemy definition with registry number {num}"))
    }

    /// The index in `defs` of the enemy with this string ID.
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.defs.iter().position(|d| d.data.id == id)
    }
}

/// True if the actor at arena index `index` thinks on `tick`: once every `every` ticks,
/// staggered by index so a crowd's thinks spread over the ticks.
pub fn thinks_on(tick: u64, index: u32, every: u8) -> bool {
    (tick + u64::from(index)).is_multiple_of(u64::from(every.max(1)))
}

/// A fresh AI actor at its spawn: full HP, Idle, unaware. `spawn` is the spawn's index
/// in `Layout::enemies`. `None` if the enemy isn't defined.
pub fn enemy_actor(
    spawn: &EnemySpawn,
    index: u16,
    table: &AiTable,
    hurtbox: Hurtbox,
) -> Option<Actor> {
    let def_i = table.index_of(&spawn.enemy)?;
    let d = &table.defs[def_i].data;
    Some(Actor {
        pos: spawn.pos,
        yaw: spawn.yaw,
        hp: d.hp,
        max_hp: d.hp,
        poise: PoiseState {
            poise: d.poise,
            stance: d.stance,
            unaware: true,
            ..PoiseState::default()
        },
        stamina: f32::from(d.stamina),
        max_stamina: f32::from(d.stamina),
        stats: Stats {
            defense: Defense::from_data(&d.defense),
            ..Stats::default()
        },
        status: StatusState::for_enemy(&d.resist, false, &d.immune, d.can_go_mad),
        motor: Motor {
            grounded: true,
            ground_y: spawn.pos[1],
            fall_peak_y: spawn.pos[1],
            ..Motor::default()
        },
        hurtbox,
        ai: Some(AiState {
            def: table.defs[def_i].num,
            spawn: index,
            cooldowns: vec![0; table.defs[def_i].options.len()],
            last_hp: d.hp,
            since_hurt: u16::MAX,
            ..AiState::default()
        }),
        ..Actor::default()
    })
}

/// Tick system 3's AI half (the scripted dummies run first; see `systems::ai_think`).
pub fn run(world: &mut World) {
    let Some(tuning) = world.ai.tuning else {
        return;
    };
    boss::run(world);
    let ids: Vec<Id> = world
        .actors
        .iter()
        .filter(|(_, a)| a.ai.is_some())
        .map(|(id, _)| id)
        .collect();
    if ids.is_empty() {
        return;
    }
    let noise = perception::player_noise(world, &tuning);
    pathing::deliver(world);
    for id in ids {
        // A boss member waits for its fight (see `boss`).
        if boss::benched(world, id) {
            continue;
        }
        tokens::release_finished(world, id);
        if world.actors.get(id).is_some_and(|a| a.dead) {
            special::tick_dead(world, id);
            continue;
        }
        if world.actors.get(id).is_none() {
            continue;
        }
        // A disguised enemy is a chest until something wakes it (WP-037).
        if special::mimic::dormant(world, id) {
            special::mimic::wake(world, id);
            if special::mimic::dormant(world, id) {
                continue;
            }
        }
        attack::regen_stamina(world, id);
        perception::listen(world, id, noise);
        if thinks_on(world.tick, id.index, tuning.think_every_ticks) {
            think(world, id, &tuning);
        }
        behaviors::execute(world, id, &tuning);
        special::batch2::tick(world, id);
    }
}

/// Moves an actor to a new awareness state and reports it.
pub(crate) fn set_awareness(world: &mut World, id: Id, to: Awareness) {
    let Some(brain) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) else {
        return;
    };
    let from = brain.awareness;
    if from == to {
        return;
    }
    brain.awareness = to;
    brain.state_ticks = 0;
    world.events.push(Event::Awareness(AwarenessEvent {
        actor: id,
        from: from.code() as u8,
        to: to.code() as u8,
    }));
}

/// One think: perception, awareness changes, leashing, and option selection.
fn think(world: &mut World, id: Id, t: &AiTuning) {
    let player = world.player;
    let Some(a) = world.actors.get(id) else {
        return;
    };
    let Some(brain) = a.ai.as_ref() else {
        return;
    };
    let def = &world.ai.def(brain.def);
    let target = world.actors.get(player).filter(|p| !p.dead);
    let sees = target.is_some_and(|p| perception::sees(&world.physics, a, p, &def.perception, t));
    let faces = target.is_some_and(|p| perception::in_cone(a, p.pos, def.perception.sight_deg));
    let hurt = a.hp < brain.last_hp;
    let lantern_lit = def.data.backstab_immune;
    let home = world
        .layout
        .enemies
        .get(usize::from(brain.spawn))
        .map_or(a.pos, |s| s.pos);
    let from_home = perception::dist_xz(a.pos, home);
    let awareness = brain.awareness;
    let heard = brain.heard;
    let leash = def.leash_m;
    let unseen = brain.unseen_ticks;
    let state_ticks = brain.state_ticks;
    let notice = brain.notice_ticks;
    let target_alive = target.is_some();
    let target_pos = target.map(|p| p.pos);

    if let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
        b.heard = None;
    }
    if let Some(a) = world.actors.get_mut(id) {
        let hp = a.hp;
        if let Some(b) = a.ai.as_mut() {
            if hp < b.last_hp {
                b.since_hurt = 0;
            }
            b.last_hp = hp;
        }
    }

    match awareness {
        Awareness::Idle | Awareness::Suspicious => {
            if (sees || hurt) && target_alive {
                engage(world, id, player, t);
            } else if let Some(at) = heard {
                if let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut()) {
                    b.noise_at = at;
                    // A new noise restarts the investigation.
                    b.state_ticks = 0;
                }
                set_awareness(world, id, Awareness::Suspicious);
            } else if awareness == Awareness::Suspicious && state_ticks >= t.suspicious_ticks {
                set_awareness(world, id, Awareness::Idle);
            }
            // Docs "Perception", Unaware: Idle or Suspicious and not facing the player.
            let still = world
                .actors
                .get(id)
                .and_then(|a| a.ai.as_ref())
                .is_some_and(|b| matches!(b.awareness, Awareness::Idle | Awareness::Suspicious));
            if let Some(a) = world.actors.get_mut(id) {
                // A carried lit lantern gives it away (the Miner husk).
                a.poise.unaware = still && !faces && !lantern_lit;
            }
        }
        Awareness::Aware => {
            if !target_alive {
                go_home(world, id);
            } else if state_ticks >= notice {
                set_awareness(world, id, Awareness::Combat);
            }
        }
        Awareness::Combat => {
            if sees
                && let Some(tp) = target_pos
                && let Some(b) = world.actors.get_mut(id).and_then(|a| a.ai.as_mut())
            {
                b.unseen_ticks = 0;
                b.last_seen = tp;
            }
            // Docs "Perception", Leash: too far from home or out of sight too long.
            let lost = !sees && unseen >= t.lost_sight_ticks;
            if !target_alive || from_home > leash || lost {
                go_home(world, id);
            } else {
                options::decide(world, id, player, t);
            }
        }
        Awareness::Return => {
            if from_home <= t.arrive_m {
                return_home(world, id);
            }
        }
    }
}

/// Idle or Suspicious to Aware: picks the target and draws the notice length.
pub(crate) fn engage(world: &mut World, id: Id, target: Id, t: &AiTuning) {
    let (lo, hi) = t.notice_ticks;
    let span = u32::from(hi.saturating_sub(lo)) + 1;
    let notice = lo + (world.rng.ai.next_u32() % span) as u16;
    let at = world.actors.get(target).map(|p| p.pos);
    if let Some(a) = world.actors.get_mut(id) {
        a.poise.unaware = false;
        a.lock_target = Some(target);
        if let Some(b) = a.ai.as_mut() {
            b.target = Some(target);
            b.notice_ticks = notice;
            b.unseen_ticks = 0;
            b.doing = Doing::Nothing;
            if let Some(p) = at {
                b.last_seen = p;
            }
        }
    }
    set_awareness(world, id, Awareness::Aware);
}

/// Leashed or lost the target: drop it and walk home.
fn go_home(world: &mut World, id: Id) {
    if let Some(a) = world.actors.get_mut(id) {
        a.lock_target = None;
        if let Some(b) = a.ai.as_mut() {
            b.target = None;
            b.doing = Doing::Nothing;
            b.doing_ticks = 0;
        }
    }
    set_awareness(world, id, Awareness::Return);
}

/// Home again: heal to full (docs "Awareness states", Return), forget the fight, and go
/// back to Idle facing the spawn's way.
fn return_home(world: &mut World, id: Id) {
    let yaw = world
        .actors
        .get(id)
        .and_then(|a| a.ai.as_ref())
        .and_then(|b| {
            world
                .layout
                .enemies
                .get(usize::from(b.spawn))
                .map(|s| s.yaw)
        });
    if let Some(a) = world.actors.get_mut(id) {
        a.hp = a.max_hp;
        a.poise.accum = 0;
        a.poise.stance_dmg = 0;
        crate::combat::status::cure_all(a);
        a.vel[0] = 0.0;
        a.vel[2] = 0.0;
        if let Some(y) = yaw {
            a.yaw = y;
        }
        let hp = a.hp;
        if let Some(b) = a.ai.as_mut() {
            b.last_hp = hp;
            b.cooldowns.iter_mut().for_each(|c| *c = 0);
            b.patrol_i = 0;
        }
    }
    set_awareness(world, id, Awareness::Idle);
}

#[cfg(test)]
mod b01_tests;
#[cfg(test)]
mod batch2_tests;
#[cfg(test)]
mod enemy_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod wp37_tests;
