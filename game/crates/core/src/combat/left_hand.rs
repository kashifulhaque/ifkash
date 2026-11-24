//! Left-hand attacks (docs/game/03-combat.md, "Left-hand attacks"), as in Elden Ring: with a
//! weapon in the left hand, `GUARD` swings it through its own light chain instead of
//! guarding. Both hands empty, `GUARD` throws the left fist. A shield guards, a catalyst
//! casts, a bow stays still, and two-handing the right weapon turns `GUARD` back into a
//! guard with it. The left hand has no heavy: `ART` stays the right weapon's Art.
//!
//! The left chain lives on the player's moveset (`Moveset::left_chain`, set by
//! `World::sync_player_moveset`). A left swing plays the right hand's move mirrored:
//! `MoveState::left_hand` mirrors its hitboxes (`hitbox::mirror_x`), takes the left
//! weapon's attack rating (`combat::resolve`), and sets `snapshot::flags::LEFT_HAND` so the
//! renderer plays the mirrored clip.

#[cfg(test)]
mod tests {
    use crate::events::Event;
    use crate::input::{ActionFrame, TickInput, buttons};
    use crate::items::inventory::slot;
    use crate::snapshot::{self, flags};
    use crate::tick::tick;
    use crate::world::World;
    use lf_content::Content;

    fn content() -> Content {
        Content::load_embedded().expect("embedded content")
    }

    /// The `wp052_colossal` yard with `left` in the left hand (`None`: empty) and `right` in
    /// the right (`None`: empty), the dummy in reach.
    fn world(right: Option<&str>, left: Option<&str>) -> World {
        let c = content();
        let mut w = World::new(5205, &c, "wp052_colossal").expect("wp052_colossal exists");
        let num = |id: &str| c.registry.num(id).expect("in the registry");
        let p = w.actors.get_mut(w.player).unwrap();
        let eq = &mut p.inventory.equipment;
        eq.slots[usize::from(slot::RIGHT_1)] = right.map(num);
        eq.slots[usize::from(slot::RIGHT_2)] = None;
        eq.slots[usize::from(slot::LEFT_1)] = left.map(num);
        eq.slots[usize::from(slot::LEFT_2)] = None;
        eq.right_active = 0;
        eq.left_active = 0;
        w.sync_player_moveset();
        let id = w.actors.iter().find(|(id, _)| *id != w.player).unwrap().0;
        w.actors.get_mut(id).unwrap().pos = [0.0, 0.0, 1.2];
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

    fn left_chain(w: &World) -> Vec<crate::combat::moves::MoveNum> {
        w.moves
            .player_moveset
            .as_ref()
            .map(|s| s.left_chain.clone())
            .unwrap_or_default()
    }

    fn player_flags(w: &World) -> u16 {
        let mut snaps = Vec::new();
        snapshot::write_snapshot(w, &mut snaps);
        let me = snapshot::pack_id(w.player);
        snaps.iter().find(|s| s.id == me).unwrap().flags
    }

    #[test]
    fn a_left_weapon_swings_its_own_chain() {
        let mut w = world(Some("wpn_wayfarer_blade"), Some("wpn_fungal_knife"));
        let chain = left_chain(&w);
        assert!(chain.len() >= 2, "a dagger has a light chain: {chain:?}");

        press(&mut w, buttons::GUARD);
        let p = w.actors.get(w.player).unwrap();
        let c = p.moves.current.expect("GUARD swings the left weapon");
        assert_eq!(c.num, chain[0]);
        assert!(p.moves.left_hand_now());
        assert!(!p.moves.guard.active, "a left swing doesn't guard");
        assert!(player_flags(&w) & flags::LEFT_HAND != 0);

        // Pressing again once the attack cancel opens goes on to the second swing.
        let mut second = None;
        for i in 0..40 {
            press(&mut w, if i % 2 == 0 { 0 } else { buttons::GUARD });
            let p = w.actors.get(w.player).unwrap();
            if let Some(c) = p.moves.current
                && c.num != chain[0]
            {
                second = Some((c.num, p.moves.left_hand_now()));
                break;
            }
        }
        assert_eq!(second, Some((chain[1], true)));
    }

    #[test]
    fn the_right_hand_starts_its_own_chain_after_a_left_swing() {
        let mut w = world(Some("wpn_wayfarer_blade"), Some("wpn_fungal_knife"));
        let right_first = w.moves.player_moveset.as_ref().unwrap().light_chain[0];
        press(&mut w, buttons::GUARD);
        for i in 0..40 {
            press(&mut w, if i % 2 == 0 { 0 } else { buttons::LIGHT });
            let p = w.actors.get(w.player).unwrap();
            if let Some(c) = p.moves.current
                && !p.moves.left_hand_now()
            {
                assert_eq!(c.num, right_first);
                return;
            }
        }
        panic!("LIGHT never started the right hand's chain");
    }

    #[test]
    fn a_left_swing_hits_with_the_left_weapon() {
        let mut w = world(Some("wpn_wayfarer_blade"), Some("wpn_fungal_knife"));
        press(&mut w, buttons::GUARD);
        for _ in 0..60 {
            press(&mut w, 0);
            for e in w.events.drain() {
                if let Event::Hit(h) = e
                    && h.attacker == w.player
                {
                    assert!(h.damage >= 1);
                    return;
                }
            }
        }
        panic!("the left swing never hit");
    }

    #[test]
    fn a_shield_or_two_handing_guards_instead() {
        let w = world(Some("wpn_wayfarer_blade"), Some("wpn_plank_buckler"));
        assert!(left_chain(&w).is_empty(), "a shield guards");

        let mut w = world(Some("wpn_wayfarer_blade"), Some("wpn_fungal_knife"));
        press(&mut w, buttons::TWO_HAND);
        press(&mut w, 0);
        assert!(w.actors.get(w.player).unwrap().stats.two_handing);
        for _ in 0..20 {
            press(&mut w, buttons::GUARD);
        }
        let p = w.actors.get(w.player).unwrap();
        assert!(p.moves.current.is_none(), "two-handed, GUARD doesn't swing");
        assert!(
            p.moves.guard.active,
            "two-handed, GUARD guards with the right weapon"
        );
    }

    #[test]
    fn an_empty_left_hand_guards_beside_a_weapon_and_punches_when_both_are_empty() {
        let w = world(Some("wpn_wayfarer_blade"), None);
        assert!(left_chain(&w).is_empty());

        let mut w = world(None, None);
        let fists = w.moves.unarmed.as_ref().unwrap().light_chain.clone();
        assert_eq!(left_chain(&w), fists);
        press(&mut w, buttons::GUARD);
        let p = w.actors.get(w.player).unwrap();
        assert_eq!(p.moves.current.map(|c| c.num), Some(fists[0]));
        assert!(p.moves.left_hand_now());
    }

    #[test]
    fn a_left_hitbox_mirrors_across_the_center_line() {
        use crate::combat::hitbox::mirror_x;
        use lf_content::moves::Shape;
        let s = Shape::Capsule {
            a: [0.2, 1.2, 0.3],
            b: [0.3, 1.0, 1.2],
            radius: 0.3,
        };
        assert_eq!(
            mirror_x(&s),
            Shape::Capsule {
                a: [-0.2, 1.2, 0.3],
                b: [-0.3, 1.0, 1.2],
                radius: 0.3,
            }
        );
    }
}
