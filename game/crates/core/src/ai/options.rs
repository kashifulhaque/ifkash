//! Option selection (docs/game/05-enemies-and-bosses.md, "Decision making").
//!
//! When a Combat actor has finished its last option, its think step filters the
//! definition's options by cooldown, conditions, and (for attack moves) whether it can
//! start a move and take an attack token, then samples one by weight from the `ai`
//! random stream. No option fitting is common (every attack is on cooldown, or the tokens
//! are taken): the actor then approaches if it's out of reach, or strafes at range,
//! for `fallback_ticks`.

use lf_content::enemy::{AiTuning, Behavior, Condition};

use super::perception::{dist_xz, off_facing_deg};
use super::{AiDef, Doing, OptionKind, Token, tokens};
use crate::actor::Actor;
use crate::arena::Id;
use crate::combat::moves::{ActiveMove, MoveTable};
use crate::world::World;

/// What conditions are tested against: the actor and its target this think.
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub dist: f32,
    /// Degrees between the actor's facing and the direction to the target.
    pub facing_deg: f32,
    /// Degrees between the target's facing and the direction to the actor.
    pub target_facing_deg: f32,
    pub hp_frac: f32,
    pub target_hp_frac: f32,
    pub target_attacking: bool,
    pub target_healing: bool,
    pub target_guarding: bool,
    pub phase: u8,
    /// Ticks since the enemy last lost HP (`u16::MAX` if it never has).
    pub since_hurt: u16,
    /// Hidden in reeds (WP-036).
    pub hidden: bool,
}

fn frac(hp: u32, max: u32) -> f32 {
    if max == 0 {
        0.0
    } else {
        hp as f32 / max as f32
    }
}

impl Context {
    pub fn new(me: &Actor, target: &Actor, moves: &MoveTable) -> Self {
        let tm = target.moves.current.and_then(|c| moves.get(c.num));
        Self {
            dist: dist_xz(me.pos, target.pos),
            facing_deg: off_facing_deg(me, target.pos),
            target_facing_deg: off_facing_deg(target, me.pos),
            hp_frac: frac(me.hp, me.max_hp),
            target_hp_frac: frac(target.hp, target.max_hp),
            target_attacking: tm
                .is_some_and(|d| !d.data.hitboxes.is_empty() || !d.data.projectiles.is_empty()),
            target_healing: match (target.moves.current, &moves.flask) {
                (Some(c), Some(f)) => c.num == f.drink_move,
                _ => false,
            },
            target_guarding: target.moves.guard.active,
            phase: me.ai.as_ref().map_or(0, |b| b.phase),
            since_hurt: me.ai.as_ref().map_or(u16::MAX, |b| b.since_hurt),
            hidden: me.ai.as_ref().is_some_and(|b| b.hidden),
        }
    }
}

/// True if the condition holds.
pub fn holds(c: &Condition, x: &Context) -> bool {
    match *c {
        Condition::DistBelow(m) => x.dist < m,
        Condition::DistAbove(m) => x.dist > m,
        Condition::DistBetween(lo, hi) => x.dist >= lo && x.dist < hi,
        Condition::FacingWithin(deg) => x.facing_deg <= deg,
        Condition::TargetFacingWithin(deg) => x.target_facing_deg <= deg,
        Condition::HpBelow(f) => x.hp_frac < f,
        Condition::HpAbove(f) => x.hp_frac >= f,
        Condition::TargetHpBelow(f) => x.target_hp_frac < f,
        Condition::TargetAttacking => x.target_attacking,
        Condition::TargetHealing => x.target_healing,
        Condition::TargetGuarding => x.target_guarding,
        // No spells yet.
        Condition::TargetCasting => false,
        Condition::Phase(p) => x.phase == p,
        Condition::HurtWithin(t) => x.since_hurt <= t,
        Condition::Hidden => x.hidden,
        Condition::Exposed => !x.hidden,
    }
}

/// True if `a` is free to start a move: alive, on the ground, not in a move, and not
/// staggered, kneeling, or parried.
pub fn can_start_move(a: &Actor) -> bool {
    !a.dead
        && a.motor.grounded
        && a.motor.landing == 0
        && a.moves.current.is_none()
        && a.moves.guard.can_act()
}

/// The token pool an option's move takes from, or `Token::None` for a behavior or an
/// enemy that ignores tokens.
fn pool(kind: OptionKind, def: &AiDef) -> Token {
    match kind {
        OptionKind::Move { .. } if def.data.ignores_tokens => Token::None,
        OptionKind::Move { ranged: true, .. } => Token::Ranged,
        OptionKind::Move { ranged: false, .. } => Token::Melee,
        OptionKind::Behavior(_) => Token::None,
    }
}

/// The indexes of the options `me` could pick now, in table order.
pub fn eligible(world: &World, me: Id, target: Id, t: &AiTuning) -> Vec<usize> {
    let (Some(a), Some(tg)) = (world.actors.get(me), world.actors.get(target)) else {
        return Vec::new();
    };
    let Some(b) = a.ai.as_ref() else {
        return Vec::new();
    };
    let def = &world.ai.defs[usize::from(b.def)];
    let x = Context::new(a, tg, &world.moves);
    let free = can_start_move(a);
    def.options
        .iter()
        .enumerate()
        .filter(|&(i, o)| {
            b.cooldowns.get(i).copied().unwrap_or(0) == 0
                && o.when.iter().all(|c| holds(c, &x))
                && match o.kind {
                    OptionKind::Move { .. } => {
                        free && tokens::available(world, me, target, pool(o.kind, def), t)
                    }
                    OptionKind::Behavior(_) => true,
                }
        })
        .map(|(i, _)| i)
        .collect()
}

/// Samples one index from `eligible` by weight, drawing once from the `ai` stream.
pub fn sample(world: &mut World, def: usize, eligible: &[usize]) -> Option<usize> {
    let opts = &world.ai.defs[def].options;
    let total: u64 = eligible
        .iter()
        .map(|&i| u64::from(opts[i].weight.max(1)))
        .sum();
    if total == 0 {
        return None;
    }
    // Scale a 32-bit draw into [0, total) without the modulo's bias toward low values.
    let mut r = (u64::from(world.rng.ai.next_u32()) * total) >> 32;
    for &i in eligible {
        let w = u64::from(world.ai.defs[def].options[i].weight.max(1));
        if r < w {
            return Some(i);
        }
        r -= w;
    }
    eligible.last().copied()
}

/// Combat think: if the last option is done, picks and starts the next.
pub fn decide(world: &mut World, me: Id, target: Id, t: &AiTuning) {
    let Some(b) = world.actors.get(me).and_then(|a| a.ai.as_ref()) else {
        return;
    };
    let busy = match b.doing {
        Doing::Nothing => false,
        Doing::Behavior(_) => b.doing_ticks > 0,
        Doing::Attack { .. } => true,
    };
    if busy {
        return;
    }
    let def = usize::from(b.def);
    let choices = eligible(world, me, target, t);
    match sample(world, def, &choices) {
        Some(i) => start_option(world, me, target, i),
        None => {
            let dist = match (world.actors.get(me), world.actors.get(target)) {
                (Some(a), Some(tg)) => dist_xz(a.pos, tg.pos),
                _ => 0.0,
            };
            let dur = t.fallback_ticks;
            let fallback = if dist > t.approach_stop_m {
                Behavior::Approach { dur }
            } else {
                Behavior::Strafe { dur }
            };
            start_behavior(world, me, fallback);
        }
    }
}

/// Starts option `i`: its move (taking a token) or its behavior, and its cooldown.
fn start_option(world: &mut World, me: Id, target: Id, i: usize) {
    let Some(b) = world.actors.get(me).and_then(|a| a.ai.as_ref()) else {
        return;
    };
    let def = &world.ai.defs[usize::from(b.def)];
    let o = def.options[i].clone();
    let token = pool(o.kind, def);
    if let Some(c) = world
        .actors
        .get_mut(me)
        .and_then(|a| a.ai.as_mut())
        .and_then(|b| b.cooldowns.get_mut(i))
    {
        *c = o.cooldown;
    }
    match o.kind {
        OptionKind::Move { num, .. } => {
            let Some(d) = world.moves.get(num) else {
                return;
            };
            let free_from = d.data.cancel_any_from.unwrap_or(d.data.total_frames);
            if let Some(a) = world.actors.get_mut(me) {
                // Starting a move drops a raised guard, like the player's.
                a.moves.guard.drop_guard();
                a.moves.current = Some(ActiveMove {
                    num,
                    frame: 0,
                    charge_ticks: 0,
                    hold_button: 0,
                    free_from,
                });
                // The move's tracking turns toward the lock target (`combat::moves`).
                a.lock_target = Some(target);
                if let Some(b) = a.ai.as_mut() {
                    b.guarding = false;
                    b.token = token;
                    b.token_target = (token != Token::None).then_some(target);
                    b.doing = Doing::Attack {
                        option: i as u16,
                        move_num: num,
                    };
                    b.doing_ticks = 0;
                }
            }
        }
        OptionKind::Behavior(beh) => start_behavior(world, me, beh),
    }
}

/// Starts a behavior for its duration. A strafe draws its direction from the `ai` stream.
pub fn start_behavior(world: &mut World, me: Id, beh: Behavior) {
    let left = matches!(beh, Behavior::Strafe { .. }) && world.rng.ai.next_u32() & 1 == 1;
    if let Some(b) = world.actors.get_mut(me).and_then(|a| a.ai.as_mut()) {
        b.doing = Doing::Behavior(beh);
        b.doing_ticks = beh.dur().unwrap_or(0);
        b.guarding = false;
        if matches!(beh, Behavior::Strafe { .. }) {
            b.strafe_left = left;
        }
    }
}
