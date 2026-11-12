//! Two-handing (docs/game/03-combat.md, "Attack rating": "While two-handing, Might counts as
//! 1.5 times its value (capped at 60) for both requirements and scaling").
//!
//! `TWO_HAND` toggles two-handing the right weapon (docs/game/01-architecture.md, "Inputs").
//! The damage pipeline already reads `Stats::two_handing`; this module is the toggle.
//!
//! A press toggles it only while the player is free to act: no move running, no guard
//! break, not dead, and a weapon in the right hand. A press while busy is dropped, not
//! buffered: the toggle is a stance, not an action. Equipping a different right-hand weapon
//! ends it (`World::sync_player_moveset`), and the snapshot reports it in
//! `snapshot::flags::TWO_HANDING`.

use crate::actor::Actor;
use crate::input::{ActionFrame, buttons};

/// Applies a `TWO_HAND` press edge to `a`. Call before `moves::step` runs for the tick, so
/// `a.moves.prev_buttons` still holds last tick's buttons. Returns true if it toggled.
pub fn update(a: &mut Actor, f: ActionFrame) -> bool {
    let pressed = f.buttons & !a.moves.prev_buttons & buttons::TWO_HAND != 0;
    if !pressed || a.dead || a.moves.current.is_some() || !a.moves.guard.can_act() {
        return false;
    }
    if a.inventory.equipment.right_weapon().is_none() {
        return false;
    }
    a.stats.two_handing = !a.stats.two_handing;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Event;
    use crate::input::TickInput;
    use crate::tick::tick;
    use crate::world::World;
    use lf_content::Content;

    fn world() -> World {
        let content = Content::load_embedded().expect("embedded content");
        let mut w = World::new(5203, &content, "wp052_colossal").expect("wp052_colossal exists");
        // The dummy stands in reach of the first swing.
        let id = w.actors.iter().find(|(id, _)| *id != w.player).unwrap().0;
        w.actors.get_mut(id).unwrap().pos = [0.0, 0.0, 2.4];
        w
    }

    fn press(w: &mut World, b: u16) {
        tick(
            w,
            &TickInput {
                frame: ActionFrame {
                    move_x: 0,
                    move_z: 0,
                    buttons: b,
                },
                commands: Vec::new(),
            },
        );
    }

    fn two_handing(w: &World) -> bool {
        w.actors.get(w.player).unwrap().stats.two_handing
    }

    /// The damage of the player's first hit on the dummy, swinging the light attack.
    fn first_hit(w: &mut World) -> u32 {
        press(w, buttons::LIGHT);
        for _ in 0..80 {
            press(w, 0);
            for e in w.events.drain() {
                if let Event::Hit(h) = e
                    && h.attacker == w.player
                {
                    return h.damage;
                }
            }
        }
        panic!("the swing never hit");
    }

    #[test]
    fn a_press_toggles_and_a_hold_does_not_repeat() {
        let mut w = world();
        assert!(!two_handing(&w));
        press(&mut w, buttons::TWO_HAND);
        assert!(two_handing(&w));
        for _ in 0..5 {
            press(&mut w, buttons::TWO_HAND);
        }
        assert!(two_handing(&w), "holding the button toggles once");
        press(&mut w, 0);
        press(&mut w, buttons::TWO_HAND);
        assert!(!two_handing(&w));
    }

    #[test]
    fn a_press_during_a_move_is_dropped() {
        let mut w = world();
        press(&mut w, buttons::LIGHT);
        assert!(w.actors.get(w.player).unwrap().moves.current.is_some());
        press(&mut w, buttons::LIGHT | buttons::TWO_HAND);
        assert!(!two_handing(&w));
    }

    #[test]
    fn two_handing_raises_the_damage_of_a_might_weapon() {
        let mut one = world();
        let mut two = world();
        press(&mut two, buttons::TWO_HAND);
        press(&mut two, 0);
        let (a, b) = (first_hit(&mut one), first_hit(&mut two));
        assert!(b > a, "two-handed {b} should beat one-handed {a}");
    }

    #[test]
    fn the_snapshot_flags_it() {
        let mut w = world();
        press(&mut w, buttons::TWO_HAND);
        let mut snaps = Vec::new();
        crate::snapshot::write_snapshot(&w, &mut snaps);
        let me = crate::snapshot::pack_id(w.player);
        let mine = snaps.iter().find(|s| s.id == me).unwrap();
        assert!(mine.flags & crate::snapshot::flags::TWO_HANDING != 0);
    }
}
