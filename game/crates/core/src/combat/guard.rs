//! Guarding: raising and lowering the guard, absorbing hits, guard break, the guard
//! counter window, and the one entry point the hit resolver calls for a landed hit. See
//! docs/game/03-combat.md, "Guard and guard counter", "Combat verbs" (Guard and Guard
//! counter), and "Regeneration".
//!
//! # For the hit resolver
//!
//! WP-021 calls [`apply_incoming_hit`] once per hit that overlaps a victim, after
//! `combat::moves::step` has run for the tick (the i-frame flag describes the frame the
//! victim executed this tick). The outcome says what damage to apply:
//! [`HitOutcome::Ignored`] takes none (i-frames, or already dead), and the other
//! outcomes carry the damage to take. The function changes only stamina and guard state,
//! and interrupts the victim's move on a guard break; it never touches HP.

use lf_content::{Guard, GuardStats};
use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::combat::{dodge, stamina};

/// Per-actor guard state. Simulation state: part of the state hash.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardState {
    /// Consecutive ticks `GUARD` has been held with nothing else going on.
    pub held: u8,
    /// Blocks count: `held` has reached `raise_ticks`.
    pub active: bool,
    /// Ticks left after releasing a raised guard, in which it can't be raised again.
    pub lowering: u8,
    /// Ticks left in which `HEAVY` is a guard counter; 0 when there is no window.
    pub counter: u8,
    /// Ticks left of the stagger from a guard break; no moves and no guard until 0.
    pub stagger: u8,
}

impl GuardState {
    /// True if the actor can start a move: not lowering the guard and not staggered.
    pub fn can_act(&self) -> bool {
        self.lowering == 0 && self.stagger == 0
    }

    /// Drops the guard at once (no lowering): starting a move does this.
    pub fn drop_guard(&mut self) {
        self.held = 0;
        self.active = false;
        self.lowering = 0;
    }

    /// Advances the guard one tick. `wants` is `GUARD` held; `free` is no move running.
    /// A guard raises after `raise_ticks` of `wants` while free, and releasing a raised
    /// guard locks it for `lower_ticks`.
    pub fn update(&mut self, wants: bool, free: bool, t: &Guard) {
        if self.stagger > 0 {
            self.stagger -= 1;
            self.held = 0;
            self.active = false;
            self.lowering = 0;
            return;
        }
        if wants && free && self.lowering == 0 {
            self.held = self.held.saturating_add(1);
            self.active = self.held >= t.raise_ticks;
        } else {
            if self.active {
                self.lowering = t.lower_ticks;
            } else {
                self.lowering = self.lowering.saturating_sub(1);
            }
            self.held = 0;
            self.active = false;
        }
    }
}

/// A hit as the guard sees it. WP-021 fills it from the hitbox and WP-023 computes
/// `damage`; this module doesn't know the damage formula.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IncomingHit {
    /// Damage before the guard's absorption.
    pub damage: u32,
    /// The hitbox's `poise_damage`: sets the stamina a block costs.
    pub poise_damage: u32,
    /// The hit carries an element, so the elemental absorption applies (a hit with both
    /// is treated as elemental until WP-023 splits the damage).
    pub elemental: bool,
    /// The hit has the `Unblockable` tag: guards and i-frames aside, it always connects.
    pub unblockable: bool,
}

/// What a hit did to its victim.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitOutcome {
    /// I-frames or a dead victim: nothing happens.
    Ignored,
    /// No guard: the full damage lands.
    Connected { damage: u32 },
    /// Blocked: the reduced damage lands and the block cost `stamina_cost`.
    Guarded { damage: u32, stamina_cost: f32 },
    /// The block cost more stamina than the victim had: stamina is 0, the victim
    /// staggers, and the part of the hit its stamina couldn't pay for lands unblocked.
    GuardBroken { damage: u32 },
}

/// Rounds `damage * pct / 100` to the nearest whole number, halves up.
fn percent_of(damage: u32, pct: u32) -> u32 {
    ((u64::from(damage) * u64::from(pct) + 50) / 100) as u32
}

/// Applies `hit` to `a`: i-frames skip it, a raised guard blocks it, and otherwise it
/// connects.
///
/// A block deals `damage * (1 - absorption)` and costs `poise_damage * 100 /
/// guard_boost` stamina, and opens the guard counter window. If that cost is more than
/// the victim's stamina, the guard breaks. Stamina pays for the share of the hit it
/// covers (damage reduced as for a block), and the rest of the damage is taken in
/// full ("takes the remaining damage unblocked": the spec doesn't say whether that is
/// the whole hit or the uncovered part; this reading makes stamina matter at the edge).
pub fn apply_incoming_hit(
    a: &mut Actor,
    hit: IncomingHit,
    stats: &GuardStats,
    t: &Guard,
) -> HitOutcome {
    if a.dead || dodge::is_invulnerable(&a.moves) {
        return HitOutcome::Ignored;
    }
    if !a.moves.guard.active || hit.unblockable {
        return HitOutcome::Connected { damage: hit.damage };
    }
    let absorb = u32::from(if hit.elemental {
        stats.absorb_elem_percent
    } else {
        stats.absorb_phys_percent
    })
    .min(100);
    // The spec's formula: `poise_damage * 100 / guard_boost`.
    let cost = hit.poise_damage as f32 * 100.0 / f32::from(stats.guard_boost.max(1));
    if a.stamina >= cost {
        stamina::spend(a, cost);
        a.moves.guard.counter = t.counter_window_ticks;
        return HitOutcome::Guarded {
            damage: percent_of(hit.damage, 100 - absorb),
            stamina_cost: cost,
        };
    }
    // Guard break. `cost > stamina >= 0`, so the division is safe.
    let covered = a.stamina / cost;
    let covered_damage = ((hit.damage as f32 * covered + 0.5) as u32).min(hit.damage);
    let damage = percent_of(covered_damage, 100 - absorb) + (hit.damage - covered_damage);
    a.stamina = 0.0;
    a.motor.stamina_idle = 0;
    a.moves.interrupt();
    a.moves.guard = GuardState {
        stagger: t.break_stagger_ticks,
        ..GuardState::default()
    };
    HitOutcome::GuardBroken { damage }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::dodge::LoadTier;
    use crate::input::{ActionFrame, TickInput, buttons};
    use crate::snapshot::{self, write_snapshot};
    use crate::tick::tick;
    use crate::world::World;
    use lf_content::Content;

    fn flat() -> World {
        let content = Content::load_embedded().expect("embedded content");
        World::new(5, &content, "m0_flat").expect("m0_flat exists")
    }

    fn tick_with(w: &mut World, buttons: u16) {
        let frame = ActionFrame {
            move_x: 0,
            move_z: 0,
            buttons,
        };
        tick(
            w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
    }

    fn player(w: &mut World) -> &mut Actor {
        w.actors.get_mut(w.player).unwrap()
    }

    /// Holds `GUARD` until the guard is raised.
    fn raise(w: &mut World) {
        for _ in 0..3 {
            tick_with(w, buttons::GUARD);
        }
        assert!(player(w).moves.guard.active);
    }

    /// A 12-poise physical hit of 100 damage, like a straight-sword light attack.
    const HIT: IncomingHit = IncomingHit {
        damage: 100,
        poise_damage: 12,
        elemental: false,
        unblockable: false,
    };

    fn hit(w: &mut World, h: IncomingHit) -> HitOutcome {
        let t = w.locomotion.guard;
        let stats = w.moves.player_moveset.as_ref().unwrap().guard;
        apply_incoming_hit(player(w), h, &stats, &t)
    }

    #[test]
    fn the_guard_raises_after_3_ticks_and_lowers_for_4() {
        let mut w = flat();
        tick_with(&mut w, buttons::GUARD);
        tick_with(&mut w, buttons::GUARD);
        assert!(!player(&mut w).moves.guard.active);
        // Not raised yet: a hit connects in full.
        assert_eq!(hit(&mut w, HIT), HitOutcome::Connected { damage: 100 });
        tick_with(&mut w, buttons::GUARD);
        assert!(player(&mut w).moves.guard.active);
        // Releasing drops it at once and locks the raise for 4 ticks.
        tick_with(&mut w, 0);
        assert!(!player(&mut w).moves.guard.active);
        for _ in 0..4 {
            tick_with(&mut w, buttons::GUARD);
            assert!(!player(&mut w).moves.guard.active);
        }
        // The raise ticks start counting again after the lowering.
        for _ in 0..2 {
            tick_with(&mut w, buttons::GUARD);
            assert!(!player(&mut w).moves.guard.active);
        }
        tick_with(&mut w, buttons::GUARD);
        assert!(player(&mut w).moves.guard.active);
    }

    #[test]
    fn a_block_absorbs_by_the_weapon_and_costs_stamina() {
        let mut w = flat();
        raise(&mut w);
        let before = player(&mut w).stamina;
        // 50 percent physical absorption; cost = 12 * 100 / 40 = 30.
        assert_eq!(
            hit(&mut w, HIT),
            HitOutcome::Guarded {
                damage: 50,
                stamina_cost: 30.0
            }
        );
        assert_eq!(player(&mut w).stamina, before - 30.0);
        assert_eq!(player(&mut w).moves.guard.counter, 12);
        assert_eq!(player(&mut w).motor.stamina_idle, 0);
        // Elemental damage: the weapon absorbs none.
        let fire = IncomingHit {
            elemental: true,
            ..HIT
        };
        assert!(matches!(
            hit(&mut w, fire),
            HitOutcome::Guarded { damage: 100, .. }
        ));
        // Unblockable hits skip the guard.
        let grab = IncomingHit {
            unblockable: true,
            ..HIT
        };
        assert_eq!(hit(&mut w, grab), HitOutcome::Connected { damage: 100 });
    }

    #[test]
    fn guard_break_triggers_at_zero_stamina() {
        let mut w = flat();
        raise(&mut w);
        player(&mut w).stamina = 0.0;
        assert_eq!(hit(&mut w, HIT), HitOutcome::GuardBroken { damage: 100 });
        let p = player(&mut w);
        assert_eq!(p.stamina, 0.0);
        assert!(!p.moves.guard.active);
        assert_eq!(p.moves.guard.stagger, 30);
        assert_eq!(p.moves.guard.counter, 0);
        assert!(p.moves.locks_movement());
        // Exactly enough stamina holds; a little less breaks.
        let mut w = flat();
        raise(&mut w);
        player(&mut w).stamina = 30.0;
        assert!(matches!(hit(&mut w, HIT), HitOutcome::Guarded { .. }));
        assert_eq!(player(&mut w).stamina, 0.0);
        assert!(player(&mut w).moves.guard.active);
        assert_eq!(hit(&mut w, HIT), HitOutcome::GuardBroken { damage: 100 });
    }

    #[test]
    fn a_broken_guard_pays_the_covered_share_and_takes_the_rest_unblocked() {
        let mut w = flat();
        raise(&mut w);
        // Cost 30, stamina 6: stamina covers a fifth, 20 damage, absorbed to 10; the
        // other 80 lands in full.
        player(&mut w).stamina = 6.0;
        assert_eq!(hit(&mut w, HIT), HitOutcome::GuardBroken { damage: 90 });
        assert_eq!(player(&mut w).stamina, 0.0);
    }

    #[test]
    fn a_guard_break_interrupts_the_move_and_staggers_for_30_ticks() {
        let mut w = flat();
        // A guard break while attacking (a guarding actor can't be mid-move, but a stagger
        // from another source calls the same hook): the move ends.
        tick_with(&mut w, buttons::LIGHT);
        assert!(player(&mut w).moves.current.is_some());
        player(&mut w).moves.guard.active = true;
        player(&mut w).stamina = 0.0;
        assert!(matches!(hit(&mut w, HIT), HitOutcome::GuardBroken { .. }));
        assert!(player(&mut w).moves.current.is_none());
        // 29 idle ticks, then a press on the 30th staggered tick doesn't start a move.
        for _ in 0..29 {
            tick_with(&mut w, 0);
        }
        tick_with(&mut w, buttons::LIGHT);
        assert!(player(&mut w).moves.current.is_none());
        assert_eq!(player(&mut w).moves.guard.stagger, 0);
        // The 31st tick is free.
        tick_with(&mut w, 0);
        tick_with(&mut w, buttons::LIGHT);
        assert!(player(&mut w).moves.current.is_some());
    }

    #[test]
    fn heavy_within_12_ticks_after_a_block_is_the_guard_counter() {
        for (idle, counters) in [(11, true), (12, false)] {
            let mut w = flat();
            raise(&mut w);
            assert!(matches!(hit(&mut w, HIT), HitOutcome::Guarded { .. }));
            for _ in 0..idle {
                tick_with(&mut w, buttons::GUARD);
            }
            tick_with(&mut w, buttons::GUARD | buttons::HEAVY);
            let want = w
                .moves
                .num_of(if counters {
                    "ss_guard_counter"
                } else {
                    "ss_heavy"
                })
                .unwrap();
            let c = player(&mut w).moves.current.expect("a move started");
            assert_eq!((c.num, c.frame), (want, 0), "idle {idle}");
            // Starting it dropped the guard and used the window up.
            assert!(!player(&mut w).moves.guard.active);
            assert_eq!(player(&mut w).moves.guard.counter, 0);
        }
    }

    #[test]
    fn i_frames_beat_a_raised_guard_and_the_dead_ignore_hits() {
        let mut w = flat();
        raise(&mut w);
        player(&mut w).moves.flags = crate::combat::moves::flags::IFRAMES;
        let before = player(&mut w).stamina;
        assert_eq!(hit(&mut w, HIT), HitOutcome::Ignored);
        assert_eq!(player(&mut w).stamina, before);
        player(&mut w).moves.flags = 0;
        player(&mut w).dead = true;
        assert_eq!(hit(&mut w, HIT), HitOutcome::Ignored);
    }

    #[test]
    fn the_snapshot_flags_guarding() {
        let mut w = flat();
        raise(&mut w);
        let mut snap = Vec::new();
        write_snapshot(&w, &mut snap);
        assert_ne!(snap[0].flags & snapshot::flags::GUARDING, 0);
        assert_eq!(snap[0].flags & snapshot::flags::INVULNERABLE, 0);
        assert_eq!(
            w.moves.player_moveset.as_ref().unwrap().load_tier,
            LoadTier::Medium
        );
    }
}
