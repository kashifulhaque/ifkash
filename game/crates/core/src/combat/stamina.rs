//! Stamina: move costs, spending, and regeneration. See docs/game/03-combat.md, "Combat
//! verbs" (the Stamina column and Sprint) and "Regeneration".
//!
//! Stamina is an `f32` on the actor. Spending resets `Motor::stamina_idle`; regeneration
//! starts once it reaches `stamina_regen_delay_ticks`. Sprint drain (0.2 a tick) and the
//! jump cost are spent inline by `actor::controller::intent`, with the same reset; its
//! sprint check needs stamina above 0 and stops at 0.

use lf_content::{Locomotion, Move};

use crate::actor::Actor;
use crate::combat::dodge::LoadTier;

/// Subtracts `amount` (never below 0) and restarts the regeneration delay. A zero
/// amount spends nothing and doesn't restart it.
pub fn spend(a: &mut Actor, amount: f32) {
    if amount > 0.0 {
        a.stamina = (a.stamina - amount).max(0.0);
        a.motor.stamina_idle = 0;
    }
}

/// True if the actor has stamina to start something that costs `cost`. Souls-like: any
/// stamina above 0 starts it, and the cost can drain the bar to 0 (the spec doesn't ask
/// for the full cost up front, and the sprint and jump rules in the controller work the
/// same way).
pub fn can_start(a: &Actor, cost: f32) -> bool {
    cost <= 0.0 || a.stamina > 0.0
}

/// Pays for starting a move: its stamina (see [`can_start`]) and its FP, which must be
/// covered in full. Returns false, spending nothing, if the actor can't pay.
pub fn try_spend(a: &mut Actor, d: &Move) -> bool {
    let stamina = f32::from(d.stamina);
    let fp = f32::from(d.fp);
    if !can_start(a, stamina) || a.fp < fp {
        return false;
    }
    spend(a, stamina);
    a.fp -= fp;
    true
}

/// Regenerates stamina for one tick (tick system 8): `stamina_regen_per_tick` once
/// `stamina_regen_delay_ticks` have passed without spending. While guarding the rate is
/// `guard.regen_percent` of that, and at Heavy equip load `equip_load.heavy_regen_percent`
/// of it (both apply together when both hold).
pub fn regen(a: &mut Actor, t: &Locomotion, tier: LoadTier) {
    if a.dead {
        return;
    }
    if a.motor.stamina_idle < t.stamina_regen_delay_ticks {
        a.motor.stamina_idle += 1;
        return;
    }
    let mut rate = t.stamina_regen_per_tick;
    if a.moves.guard.active {
        rate *= f32::from(t.guard.regen_percent) / 100.0;
    }
    if tier == LoadTier::Heavy {
        rate *= f32::from(t.equip_load.heavy_regen_percent) / 100.0;
    }
    a.stamina = (a.stamina + rate).min(a.max_stamina);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{ActionFrame, TickInput, buttons};
    use crate::tick::tick;
    use crate::world::World;
    use lf_content::Content;

    fn flat() -> World {
        let content = Content::load_embedded().expect("embedded content");
        World::new(5, &content, "m0_flat").expect("m0_flat exists")
    }

    fn tick_with(w: &mut World, buttons: u16, z: i8) {
        let frame = ActionFrame {
            move_x: 0,
            move_z: z,
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

    fn stamina(w: &World) -> f32 {
        w.actors.get(w.player).unwrap().stamina
    }

    #[test]
    fn regenerates_1_5_a_tick_after_15_idle_ticks() {
        let mut w = flat();
        w.actors.get_mut(w.player).unwrap().stamina = 20.0;
        w.actors.get_mut(w.player).unwrap().motor.stamina_idle = 0;
        for _ in 0..15 {
            tick_with(&mut w, 0, 0);
            assert_eq!(stamina(&w), 20.0);
        }
        tick_with(&mut w, 0, 0);
        assert_eq!(stamina(&w), 21.5);
        tick_with(&mut w, 0, 0);
        assert_eq!(stamina(&w), 23.0);
    }

    #[test]
    fn guarding_regenerates_at_40_percent_and_heavy_load_at_80_percent() {
        let t = |guard: bool, tier| {
            let mut w = flat();
            let a = w.actors.get_mut(w.player).unwrap();
            a.stamina = 20.0;
            a.motor.stamina_idle = 15;
            a.moves.guard.active = guard;
            let tuning = w.locomotion;
            regen(w.actors.get_mut(w.player).unwrap(), &tuning, tier);
            stamina(&w) - 20.0
        };
        let near = |got: f32, want: f32| assert!((got - want).abs() < 1e-5, "{got} != {want}");
        near(t(false, LoadTier::Medium), 1.5);
        near(t(true, LoadTier::Medium), 0.6);
        near(t(false, LoadTier::Heavy), 1.2);
        near(t(true, LoadTier::Heavy), 0.48);
        near(t(false, LoadTier::Light), 1.5);
    }

    #[test]
    fn sprint_drains_0_2_a_tick_and_stops_at_zero() {
        let mut w = flat();
        // Sprint starts once DODGE has been held for more than 8 ticks.
        for _ in 0..9 {
            tick_with(&mut w, buttons::DODGE, 127);
        }
        assert!(w.actors.get(w.player).unwrap().motor.sprinting);
        let before = stamina(&w);
        tick_with(&mut w, buttons::DODGE, 127);
        assert!((before - stamina(&w) - 0.2).abs() < 1e-5);
        // No regeneration while it drains; at 0 the sprint ends.
        w.actors.get_mut(w.player).unwrap().stamina = 0.1;
        tick_with(&mut w, buttons::DODGE, 127);
        assert_eq!(stamina(&w), 0.0);
        tick_with(&mut w, buttons::DODGE, 127);
        assert!(!w.actors.get(w.player).unwrap().motor.sprinting);
    }

    #[test]
    fn moves_start_with_any_stamina_and_drain_to_zero() {
        let mut w = flat();
        w.actors.get_mut(w.player).unwrap().stamina = 5.0;
        tick_with(&mut w, buttons::LIGHT, 0);
        assert_eq!(stamina(&w), 0.0);
        assert!(w.actors.get(w.player).unwrap().moves.current.is_some());

        let mut w = flat();
        w.actors.get_mut(w.player).unwrap().stamina = 0.0;
        tick_with(&mut w, buttons::LIGHT, 0);
        assert!(w.actors.get(w.player).unwrap().moves.current.is_none());
    }
}
