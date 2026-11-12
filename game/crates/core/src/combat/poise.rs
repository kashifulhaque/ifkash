//! Poise, stance, and the parry (docs/game/03-combat.md, "Poise and stance" and "Parry
//! and critical hits"). Critical hits are in [`super::crit`].
//!
//! # Poise
//!
//! Every landed hit that isn't blocked adds its `poise_damage` to the target's
//! accumulator. When the accumulator exceeds the actor's poise the actor staggers and the
//! accumulator resets; a margin of up to half the poise is a light stagger, more is a
//! heavy one. During hyperarmor frames the poise counts as doubled. The accumulator also
//! resets after `reset_ticks` ticks without poise damage. A poise of 0 means the actor has
//! no poise meter and never staggers.
//!
//! A stagger is the same lockout a guard break causes: it interrupts the actor's move and
//! sets `guard.stagger`, which blocks starting moves, raising the guard, and walking until
//! it runs out. That keeps one stagger mechanism; [`PoiseState`] adds what the lockout
//! doesn't say (the kind of stagger, stance, and the parried state).
//!
//! # Stance
//!
//! Only actors with a stance meter (elites and bosses) have one. A hit adds
//! `poise_damage * stance_mult` of the move; reaching the meter breaks the stance: the
//! actor kneels (locked, like a stagger) for `stance_break_ticks`, and a riposte can start.
//! Stance damage decays after a quiet spell.
//!
//! # Parry
//!
//! A weapon with a parry window raises it `parry_startup_ticks` after `GUARD` is pressed,
//! and it stays open for the weapon's `parry_window_ticks`. A hit that arrives inside the
//! window from the front, and isn't `Unparryable`, is parried: it deals nothing, and the
//! attacker is interrupted and enters the *parried* state, locked for `parried_ticks`.
//! The parry costs no stamina and a missed parry has no extra recovery here; the spec's
//! recovery and stamina columns for Parry are left to the shield and dagger WPs.

use lf_content::Poise;
use serde::{Deserialize, Serialize};

use super::moves::{MoveNum, flags as move_flags};
use crate::actor::Actor;
use crate::arena::Id;
use crate::events::{self, Event};
use crate::math::dmath::{cos, sin};
use crate::world::World;

/// Per-actor poise, stance, and critical-hit state. Simulation state: part of the state
/// hash.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoiseState {
    /// The actor's poise. 0: no poise meter.
    pub poise: u16,
    /// Poise damage taken since the last reset.
    pub accum: u32,
    /// Ticks since the last poise damage.
    pub since_hit: u16,
    /// The stance meter. 0: no stance meter.
    pub stance: u16,
    /// Stance damage taken, in the same units as `stance`.
    pub stance_dmg: u32,
    /// Ticks since the last stance damage.
    pub stance_since: u16,
    /// Ticks left of a stance break; above 0 means the actor is kneeling.
    pub kneel: u8,
    /// Ticks left of the parried state.
    pub parried: u8,
    /// The kind of the current stagger, for the renderer: true for heavy.
    pub heavy_stagger: bool,
    /// Unaware of the player: a backstab can land (docs/game/03-combat.md, "Parry and
    /// critical hits"). Cleared by any hit. WP-030's AI sets and clears it as awareness
    /// changes.
    pub unaware: bool,
}

/// What a hit did to the target's poise and stance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoiseEffect {
    /// The hit staggered the target: `(heavy, ticks)`.
    pub stagger: Option<(bool, u8)>,
    /// The hit broke the target's stance: the kneel's ticks.
    pub stance_break: Option<u8>,
}

/// Applies a landed, unblocked hit to `a`'s poise and stance. `poise_damage` is the
/// hitbox's and `stance_mult_milli` the move's `stance_mult` in thousandths.
/// `poise_percent` scales the actor's poise (Steadfast: 150; 100 changes nothing).
pub fn apply_hit(
    a: &mut Actor,
    poise_damage: u32,
    stance_mult_milli: u32,
    poise_percent: u32,
    t: &Poise,
) -> PoiseEffect {
    let mut fx = PoiseEffect::default();
    a.poise.unaware = false;
    if a.dead || poise_damage == 0 {
        return fx;
    }
    let hyper = a.moves.has(move_flags::HYPERARMOR);
    let p = &mut a.poise;
    // Stance. A kneeling actor has already broken.
    if p.stance > 0 && p.kneel == 0 {
        p.stance_dmg += poise_damage * stance_mult_milli / 1000;
        p.stance_since = 0;
        if p.stance_dmg >= u32::from(p.stance) {
            p.stance_dmg = 0;
            p.kneel = t.stance_break_ticks;
            p.accum = 0;
            fx.stance_break = Some(t.stance_break_ticks);
        }
    }
    // Poise. A kneeling actor is already down.
    if p.poise > 0 && p.kneel == 0 && fx.stance_break.is_none() {
        p.accum += poise_damage;
        p.since_hit = 0;
        let mult = if hyper {
            u32::from(t.hyperarmor_mult)
        } else {
            1
        };
        let poise = u32::from(p.poise) * mult * poise_percent / 100;
        if p.accum > poise {
            let margin = p.accum - poise;
            let heavy = margin * 100 > poise * u32::from(t.heavy_margin_percent);
            let ticks = if heavy {
                t.heavy_stagger_ticks
            } else {
                t.light_stagger_ticks
            };
            p.accum = 0;
            p.heavy_stagger = heavy;
            fx.stagger = Some((heavy, ticks));
        }
    }
    // Both end the actor's move and lock it.
    let lock = match (fx.stance_break, fx.stagger) {
        (Some(k), _) => k,
        (None, Some((_, ticks))) => ticks,
        (None, None) => return fx,
    };
    a.moves.interrupt();
    a.moves.guard.drop_guard();
    a.moves.guard.counter = 0;
    a.moves.guard.stagger = a.moves.guard.stagger.max(lock);
    fx
}

/// Puts `attacker` in the parried state: its move ends, and it is locked for
/// `parried_ticks`.
pub fn apply_parried(attacker: &mut Actor, t: &Poise) {
    attacker.moves.interrupt();
    attacker.moves.guard.drop_guard();
    attacker.poise.unaware = false;
    attacker.poise.parried = t.parried_ticks;
    attacker.moves.guard.stagger = attacker.moves.guard.stagger.max(t.parried_ticks);
}

/// Advances one actor's timers by a tick: the poise reset, the stance decay, the kneel,
/// and the parried state. Run after the tick's moves, so a state set by hit resolution
/// lasts its full count of ticks.
pub fn tick(a: &mut Actor, t: &Poise) {
    let p = &mut a.poise;
    p.since_hit = p.since_hit.saturating_add(1);
    if p.since_hit >= t.reset_ticks {
        p.accum = 0;
    }
    p.stance_since = p.stance_since.saturating_add(1);
    if p.stance_since > t.stance_decay_after_ticks && p.stance_dmg > 0 {
        // 2 percent of the meter per tick; at least 1 so a small meter still drains.
        let step = (u32::from(p.stance) * u32::from(t.stance_decay_percent) / 100).max(1);
        p.stance_dmg = p.stance_dmg.saturating_sub(step);
    }
    p.kneel = p.kneel.saturating_sub(1);
    p.parried = p.parried.saturating_sub(1);
}

/// True if a guard held for `held` ticks (see `GuardState::held`) is inside the parry
/// window: it opens `startup` ticks after the press and lasts `window` ticks. A window of
/// 0 never opens.
pub fn parry_window_open(held: u8, startup: u8, window: u8) -> bool {
    held > startup && u16::from(held) <= u16::from(startup) + u16::from(window)
}

/// True if `target` parries a hit from `attacker`: it has a parry window and is inside it,
/// the hit isn't unparryable, and the attacker is in front of it. `window` is the target's
/// weapon's `parry_window_ticks`.
pub fn is_parry(
    attacker: &Actor,
    target: &Actor,
    window: u8,
    unparryable: bool,
    t: &Poise,
) -> bool {
    if unparryable || target.dead || target.moves.current.is_some() {
        return false;
    }
    if !parry_window_open(target.moves.guard.held, t.parry_startup_ticks, window) {
        return false;
    }
    // The attacker must be in the front half-plane of the target.
    let fwd = [sin(target.yaw), cos(target.yaw)];
    let (dx, dz) = (
        attacker.pos[0] - target.pos[0],
        attacker.pos[2] - target.pos[2],
    );
    fwd[0] * dx + fwd[1] * dz > 0.0
}

/// A parry decided in the hit resolver's decide step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parry {
    pub attacker: Id,
    pub target: Id,
    /// The attacker's move that was parried.
    pub move_num: MoveNum,
}

/// Applies the parries decided this tick: each attacker is parried and its move marks the
/// target struck, and a `Parry` event goes out.
pub fn apply_parries(world: &mut World, parries: &[Parry]) {
    let t = world.locomotion.poise;
    for p in parries {
        if let Some(a) = world.actors.get_mut(p.attacker) {
            apply_parried(a, &t);
            if let Err(at) = a.hit.struck.binary_search(&p.target) {
                a.hit.struck.insert(at, p.target);
            }
        }
        world.events.push(Event::Parry(events::Parry {
            attacker: p.attacker,
            target: p.target,
            move_num: p.move_num,
        }));
    }
}

/// Applies the poise and stance damage of a hit from `move_num` that connected on
/// `target`, and emits the `Stagger` and `StanceBreak` events it causes.
pub fn apply_landing(world: &mut World, target: Id, move_num: MoveNum, poise_damage: u32) {
    let t = world.locomotion.poise;
    // The move's stance multiplier, in thousandths (rounding the f32 is exact for the
    // listed values: 2.0 -> 2000).
    let mult = world.moves.get(move_num).map_or(1000, |d| {
        (d.data.stance_mult * 1000.0).round().max(0.0) as u32
    });
    // Steadfast (a body buff) raises the target's poise (WP-056).
    let poise_percent = world.actors.get(target).map_or(100, |a| {
        a.status
            .body_buff
            .and_then(|b| world.status.buff(b.num))
            .map_or(100, |d| u32::from(d.poise_percent))
    });
    let Some(a) = world.actors.get_mut(target) else {
        return;
    };
    // A target the hit killed doesn't stagger.
    let fx = if a.hp == 0 {
        PoiseEffect::default()
    } else {
        apply_hit(a, poise_damage, mult, poise_percent, &t)
    };
    if let Some(ticks) = fx.stance_break {
        world
            .events
            .push(Event::StanceBreak(events::StanceBreak { target, ticks }));
    } else if let Some((heavy, ticks)) = fx.stagger {
        world.events.push(Event::Stagger(events::Stagger {
            target,
            heavy,
            ticks,
        }));
    }
}

/// The parry window of `id`'s weapon: only the player has a moveset until AI lands.
pub fn parry_window_of(world: &World, id: Id) -> u8 {
    if id == world.player {
        let base = world
            .moves
            .player_moveset
            .as_ref()
            .map_or(0, |m| m.guard.parry_window_ticks);
        // Parrier's Eye lengthens the window of a weapon that has one (WP-057).
        let more = world.actors.get(id).map_or(0, |p| {
            world.items.gear.parry_window_bonus(&p.inventory.equipment)
        });
        if base > 0 {
            u8::try_from(u32::from(base) + more).unwrap_or(u8::MAX)
        } else {
            0
        }
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::moves::{ActiveMove, flags};
    use lf_content::Content;

    fn tuning() -> Poise {
        Content::load_embedded().unwrap().locomotion.unwrap().poise
    }

    fn actor(poise: u16, stance: u16) -> Actor {
        Actor {
            hp: 100,
            max_hp: 100,
            poise: PoiseState {
                poise,
                stance,
                ..PoiseState::default()
            },
            ..Actor::default()
        }
    }

    fn swing(a: &mut Actor) {
        a.moves.current = Some(ActiveMove {
            num: 1,
            frame: 5,
            charge_ticks: 0,
            hold_button: 0,
            free_from: 40,
        });
    }

    #[test]
    fn poise_accumulates_until_it_is_exceeded_and_resets() {
        let t = tuning();
        let mut a = actor(40, 0);
        for _ in 0..3 {
            assert_eq!(apply_hit(&mut a, 12, 1000, 100, &t).stagger, None);
        }
        // 36 so far; the fourth light makes 48, a margin of 8 of 40: light.
        let fx = apply_hit(&mut a, 12, 1000, 100, &t);
        assert_eq!(fx.stagger, Some((false, t.light_stagger_ticks)));
        assert_eq!(a.poise.accum, 0);
        assert_eq!(a.moves.guard.stagger, t.light_stagger_ticks);
    }

    #[test]
    fn exactly_the_poise_does_not_stagger() {
        let t = tuning();
        let mut a = actor(40, 0);
        assert_eq!(apply_hit(&mut a, 40, 1000, 100, &t).stagger, None);
        assert_eq!(
            apply_hit(&mut a, 1, 1000, 100, &t).stagger.map(|s| s.0),
            Some(false)
        );
    }

    #[test]
    fn light_and_heavy_stagger_split_at_half_the_poise() {
        let t = tuning();
        // Margin 20 of 40 is exactly 50 percent: still light. 21 is heavy.
        let mut a = actor(40, 0);
        assert_eq!(
            apply_hit(&mut a, 60, 1000, 100, &t).stagger,
            Some((false, t.light_stagger_ticks))
        );
        let mut b = actor(40, 0);
        assert_eq!(
            apply_hit(&mut b, 61, 1000, 100, &t).stagger,
            Some((true, t.heavy_stagger_ticks))
        );
        assert!(b.poise.heavy_stagger);
        assert_eq!(b.moves.guard.stagger, t.heavy_stagger_ticks);
    }

    #[test]
    fn a_stagger_interrupts_the_move() {
        let t = tuning();
        let mut a = actor(10, 0);
        swing(&mut a);
        apply_hit(&mut a, 12, 1000, 100, &t);
        assert_eq!(a.moves.current, None);
    }

    #[test]
    fn the_accumulator_resets_after_a_quiet_spell() {
        let t = tuning();
        let mut a = actor(40, 0);
        apply_hit(&mut a, 30, 1000, 100, &t);
        for _ in 0..t.reset_ticks - 1 {
            tick(&mut a, &t);
        }
        assert_eq!(a.poise.accum, 30);
        tick(&mut a, &t);
        assert_eq!(a.poise.accum, 0);
        // So a second 30 doesn't stagger.
        assert_eq!(apply_hit(&mut a, 30, 1000, 100, &t).stagger, None);
    }

    #[test]
    fn hyperarmor_doubles_the_poise() {
        let t = tuning();
        let mut plain = actor(40, 0);
        let mut armored = actor(40, 0);
        swing(&mut armored);
        armored.moves.flags = flags::HYPERARMOR;
        // 60 staggers the plain actor (60 > 40) but not the armored one (60 <= 80).
        assert!(apply_hit(&mut plain, 60, 1000, 100, &t).stagger.is_some());
        assert_eq!(apply_hit(&mut armored, 60, 1000, 100, &t).stagger, None);
        assert!(armored.moves.current.is_some());
        // It still staggers once the doubled poise is passed.
        assert!(apply_hit(&mut armored, 30, 1000, 100, &t).stagger.is_some());
    }

    #[test]
    fn no_poise_meter_never_staggers() {
        let t = tuning();
        let mut a = actor(0, 0);
        assert_eq!(
            apply_hit(&mut a, 500, 1000, 100, &t),
            PoiseEffect::default()
        );
    }

    #[test]
    fn stance_breaks_and_the_kneel_runs_out() {
        let t = tuning();
        let mut a = actor(1000, 120);
        // Charged heavies: 34 poise damage times 2.0.
        assert_eq!(apply_hit(&mut a, 34, 2000, 100, &t).stance_break, None);
        assert_eq!(a.poise.stance_dmg, 68);
        let fx = apply_hit(&mut a, 34, 2000, 100, &t);
        assert_eq!(fx.stance_break, Some(t.stance_break_ticks));
        assert_eq!(a.poise.kneel, t.stance_break_ticks);
        assert_eq!(a.moves.guard.stagger, t.stance_break_ticks);
        assert_eq!(a.poise.stance_dmg, 0);
        for _ in 0..t.stance_break_ticks {
            tick(&mut a, &t);
        }
        assert_eq!(a.poise.kneel, 0);
    }

    #[test]
    fn stance_damage_waits_then_decays() {
        let t = tuning();
        let mut a = actor(1000, 100);
        apply_hit(&mut a, 50, 1000, 100, &t);
        for _ in 0..t.stance_decay_after_ticks {
            tick(&mut a, &t);
        }
        // Nothing decayed through the quiet spell; the next tick takes 2 percent of 100.
        assert_eq!(a.poise.stance_dmg, 50);
        tick(&mut a, &t);
        assert_eq!(a.poise.stance_dmg, 48);
    }

    #[test]
    fn a_meterless_actor_takes_no_stance_damage() {
        let t = tuning();
        let mut a = actor(1000, 0);
        apply_hit(&mut a, 50, 2000, 100, &t);
        assert_eq!(a.poise.stance_dmg, 0);
    }

    #[test]
    fn a_hit_makes_an_unaware_actor_aware() {
        let t = tuning();
        let mut a = actor(1000, 0);
        a.poise.unaware = true;
        apply_hit(&mut a, 1, 1000, 100, &t);
        assert!(!a.poise.unaware);
    }

    #[test]
    fn the_parry_window_opens_after_startup_and_lasts_its_length() {
        // Startup 2, window 6: held ticks 3 through 8.
        let open: Vec<u8> = (0..=12).filter(|&h| parry_window_open(h, 2, 6)).collect();
        assert_eq!(open, vec![3, 4, 5, 6, 7, 8]);
        // A weapon with no window never parries.
        assert!((0..=255).all(|h| !parry_window_open(h, 2, 0)));
    }

    #[test]
    fn a_parry_needs_the_window_the_front_and_a_parryable_hit() {
        let t = tuning();
        let mut target = actor(40, 0);
        target.yaw = 0.0; // faces +Z
        target.moves.guard.held = 4;
        let mut attacker = actor(40, 0);
        attacker.pos = [0.0, 0.0, 1.0];
        assert!(is_parry(&attacker, &target, 6, false, &t));
        assert!(!is_parry(&attacker, &target, 6, true, &t), "unparryable");
        assert!(!is_parry(&attacker, &target, 0, false, &t), "no parry tool");
        attacker.pos = [0.0, 0.0, -1.0];
        assert!(!is_parry(&attacker, &target, 6, false, &t), "from behind");
        attacker.pos = [0.0, 0.0, 1.0];
        target.moves.guard.held = 9;
        assert!(!is_parry(&attacker, &target, 6, false, &t), "window closed");
        target.moves.guard.held = 2;
        assert!(!is_parry(&attacker, &target, 6, false, &t), "not open yet");
    }

    #[test]
    fn the_parried_state_locks_the_attacker_and_runs_out() {
        let t = tuning();
        let mut a = actor(40, 0);
        swing(&mut a);
        apply_parried(&mut a, &t);
        assert_eq!(a.moves.current, None);
        assert_eq!(a.poise.parried, t.parried_ticks);
        assert_eq!(a.moves.guard.stagger, t.parried_ticks);
        for _ in 0..t.parried_ticks {
            tick(&mut a, &t);
        }
        assert_eq!(a.poise.parried, 0);
    }
}
