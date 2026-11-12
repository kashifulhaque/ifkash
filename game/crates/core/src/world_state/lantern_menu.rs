//! The Lantern menu's commands that change the character or the player's place: level up,
//! respec, and travel (`Command::LevelUp`, `Command::Respec`, `Command::Travel`). Split
//! flasks lives in `items`. All of them apply only while resting at a Lantern. See
//! docs/game/06-progression-and-systems.md, "Leveling" and "Lanterns", and
//! docs/game/04-arsenal-and-builds.md, "Respec".
//!
//! The leveling numbers and rules live in `actor::level`. This module spends the Glimmer
//! and the Rewrite Quill and applies the result to the player.

use lf_content::Attributes;

use crate::actor::Actor;
use crate::actor::level::{self, ATTRIBUTE_COUNT};
use crate::actor::stats::{Attrs, Derived, Stats};
use crate::combat::status::StatusState;
use crate::input::{Command, TickInput};
use crate::world::World;

use super::{flags, lantern};

/// The item ID of the Rewrite Quill (`consumables.ron`; registry 14000), which a respec
/// consumes.
const REWRITE_QUILL: &str = "itm_rewrite_quill";

/// The derived stats now and after raising `attribute` by one, for the menu's preview.
/// `None` if the player is gone, the attribute doesn't exist, or it is at the maximum.
/// Read-only.
pub fn level_preview(world: &World, attribute: u8) -> Option<(Derived, Derived)> {
    let attrs = world.actors.get(world.player)?.stats.attrs;
    let now = attrs.derived(&world.attributes);
    let raised = level::raised(&attrs, &world.attributes, attribute)?;
    Some((now, raised.derived(&world.attributes)))
}

/// Applies the tick's `LevelUp`, `Respec`, and `Travel` commands, in order, while resting.
pub fn run(world: &mut World, input: &TickInput) {
    for c in &input.commands {
        if world.state.resting.is_none() {
            return;
        }
        match *c {
            Command::LevelUp { attribute } => {
                level_up(world, attribute);
            }
            Command::Respec { attributes } => {
                respec(world, attributes);
            }
            Command::Travel { lantern } => {
                travel(world, usize::from(lantern));
            }
            _ => {}
        }
    }
}

/// Spends the Glimmer the next level costs to raise `attribute` by one. False (and no
/// change) if the attribute doesn't exist or is at the maximum, or the player can't pay.
pub fn level_up(world: &mut World, attribute: u8) -> bool {
    let rules = &world.attributes;
    let player = world.player;
    let Some(p) = world.actors.get_mut(player) else {
        return false;
    };
    let cost = level::level_cost(rules, p.stats.attrs.level(rules));
    if p.dead || p.inventory.glimmer < cost {
        return false;
    }
    let Some(attrs) = level::raised(&p.stats.attrs, rules, attribute) else {
        return false;
    };
    p.inventory.glimmer -= cost;
    set_attributes(p, rules, attrs);
    true
}

/// Redistributes the player's attribute points (docs/game/04-arsenal-and-builds.md,
/// "Respec"). `attributes` is the new spread, in `Attrs` order. It costs one Rewrite Quill
/// and no Glimmer, and the level stays the same. False (and no change) if the player is
/// dead or has no quill, or the spread isn't a valid respec (`level::respec_allowed`).
///
/// The floor is the starting class's values. No class can be chosen yet, so that floor is
/// the player's starting loadout (`locomotion.ron`, see `Stats::player_stub`).
pub fn respec(world: &mut World, attributes: [u16; ATTRIBUTE_COUNT]) -> bool {
    let Some(quill) = world.items.num_of(REWRITE_QUILL) else {
        return false;
    };
    let base = Stats::player_stub(&world.locomotion).attrs;
    let rules = &world.attributes;
    let player = world.player;
    let Some(p) = world.actors.get_mut(player) else {
        return false;
    };
    if p.dead || p.inventory.count(quill) == 0 {
        return false;
    }
    let target = level::from_values(attributes);
    if !level::respec_allowed(rules, &base, &p.stats.attrs, &target) {
        return false;
    }
    p.inventory.remove(quill, 1);
    set_attributes(p, rules, target);
    true
}

/// Sets the player's attributes and refreshes what they drive. The maxima follow their
/// attributes, the current values move by the same amount (clamped to the new maxima), and
/// the resistances that attributes raise are refreshed.
fn set_attributes(p: &mut Actor, rules: &Attributes, attrs: Attrs) {
    let before = p.stats.attrs.derived(rules);
    let after = attrs.derived(rules);
    p.stats.attrs = attrs;
    p.max_hp = after.max_hp;
    p.max_fp = after.max_fp;
    p.max_stamina = after.max_stamina;
    let hp_delta = i64::from(after.max_hp) - i64::from(before.max_hp);
    p.hp = (i64::from(p.hp) + hp_delta).clamp(0, i64::from(p.max_hp)) as u32;
    p.fp = (p.fp + (after.max_fp - before.max_fp)).clamp(0.0, p.max_fp);
    p.stamina = (p.stamina + (after.max_stamina - before.max_stamina)).clamp(0.0, p.max_stamina);
    p.status.resist = StatusState::for_player(&attrs, rules).resist;
}

/// Moves the player to the lit Lantern at index `to` and rests there, so the Lantern menu
/// stays open at the new place. False (and no change) if `to` isn't another Lantern of the
/// scenario, or it isn't lit.
pub fn travel(world: &mut World, to: usize) -> bool {
    let Some(l) = world.layout.lanterns.get(to).cloned() else {
        return false;
    };
    let here = world.state.resting.as_deref();
    if here == Some(l.id.as_str()) || !world.state.flags.is_set(&flags::lantern_lit(&l.id)) {
        return false;
    }
    let player = world.player;
    let Some(p) = world.actors.get_mut(player) else {
        return false;
    };
    p.pos = l.pos;
    p.yaw = l.yaw;
    p.vel = [0.0; 3];
    p.lock_target = None;
    p.motor.grounded = true;
    p.motor.ground_y = l.pos[1];
    p.motor.fall_peak_y = l.pos[1];
    lantern::rest(world, to);
    true
}

#[cfg(test)]
mod tests {
    use lf_content::Content;
    use lf_content::scenario::Lantern;

    use super::*;
    use crate::input::ActionFrame;
    use crate::tick::tick;

    fn field() -> World {
        let content = Content::load_embedded().expect("embedded content");
        World::new(9, &content, "test_field").expect("test_field exists")
    }

    fn step(w: &mut World, commands: Vec<Command>) {
        tick(
            w,
            &TickInput {
                frame: ActionFrame::default(),
                commands,
            },
        );
    }

    /// Rests at the test field's Lantern, which is 3 m behind the spawn.
    fn rest(w: &mut World) {
        let id = w.player;
        w.actors.get_mut(id).unwrap().pos = [0.0, 0.0, -2.0];
        step(w, vec![Command::Rest]);
        assert!(w.state.resting.is_some());
    }

    #[test]
    fn level_up_spends_glimmer_and_raises_the_attribute_and_maxima() {
        let mut w = field();
        rest(&mut w);
        let id = w.player;
        let (level, vit, hp) = {
            let p = w.actors.get(id).unwrap();
            (
                p.stats.attrs.level(&w.attributes),
                p.stats.attrs.vitality,
                p.max_hp,
            )
        };
        let cost = level::level_cost(&w.attributes, level);
        w.actors.get_mut(id).unwrap().inventory.glimmer = cost + 5;
        step(&mut w, vec![Command::LevelUp { attribute: 0 }]);
        let p = w.actors.get(id).unwrap();
        assert_eq!(p.inventory.glimmer, 5);
        assert_eq!(p.stats.attrs.vitality, vit + 1);
        assert_eq!(p.stats.attrs.level(&w.attributes), level + 1);
        assert!(p.max_hp > hp);
        assert_eq!(p.hp, p.max_hp, "a full bar stays full");
    }

    #[test]
    fn the_preview_matches_what_the_level_up_does() {
        let mut w = field();
        rest(&mut w);
        let id = w.player;
        let (now, next) = level_preview(&w, 0).expect("vitality can rise");
        assert!(next.max_hp > now.max_hp);
        assert!(level_preview(&w, 7).is_none());
        let hash = w.state_hash();
        let _ = level_preview(&w, 0);
        assert_eq!(w.state_hash(), hash, "a preview changes nothing");
        w.actors.get_mut(id).unwrap().inventory.glimmer = 100_000;
        step(&mut w, vec![Command::LevelUp { attribute: 0 }]);
        assert_eq!(w.actors.get(id).unwrap().max_hp, next.max_hp);
    }

    #[test]
    fn level_up_needs_a_rest_the_glimmer_and_a_real_attribute() {
        let mut w = field();
        let id = w.player;
        w.actors.get_mut(id).unwrap().inventory.glimmer = 100_000;
        let before = w.actors.get(id).unwrap().stats.attrs;
        // Not resting.
        step(&mut w, vec![Command::LevelUp { attribute: 0 }]);
        assert_eq!(w.actors.get(id).unwrap().stats.attrs, before);
        rest(&mut w);
        // No such attribute.
        step(&mut w, vec![Command::LevelUp { attribute: 7 }]);
        assert_eq!(w.actors.get(id).unwrap().stats.attrs, before);
        // Too little Glimmer.
        w.actors.get_mut(id).unwrap().inventory.glimmer = 1;
        step(&mut w, vec![Command::LevelUp { attribute: 3 }]);
        assert_eq!(w.actors.get(id).unwrap().stats.attrs, before);
        // An attribute at the maximum.
        let max = w.attributes.max;
        let p = w.actors.get_mut(id).unwrap();
        p.inventory.glimmer = 100_000;
        p.stats.attrs.might = max;
        let at_max = p.stats.attrs;
        step(&mut w, vec![Command::LevelUp { attribute: 3 }]);
        assert_eq!(w.actors.get(id).unwrap().stats.attrs, at_max);
    }

    /// The Rewrite Quill's item number in the test world.
    fn quill(w: &World) -> lf_content::registry::NumId {
        w.items
            .num_of(REWRITE_QUILL)
            .expect("the quill is registered")
    }

    /// Gives the player `n` Rewrite Quills.
    fn give_quills(w: &mut World, n: u16) {
        let q = quill(w);
        let id = w.player;
        let items = &w.items;
        let p = w.actors.get_mut(id).unwrap();
        assert_eq!(p.inventory.pick_up(q, n, items), n);
    }

    fn quills_held(w: &World) -> u16 {
        w.actors.get(w.player).unwrap().inventory.count(quill(w))
    }

    /// The starting loadout: Vitality 10, Focus 10, Endurance 10, Might 12, Finesse 12,
    /// Insight 8, Devotion 8 (level 10).
    const BASE: [u16; ATTRIBUTE_COUNT] = [10, 10, 10, 12, 12, 8, 8];

    #[test]
    fn respec_consumes_a_quill_and_redistributes_without_changing_glimmer_or_level() {
        let mut w = field();
        rest(&mut w);
        let id = w.player;
        // Three level-ups, all into Might: 420 + 465 + 512 Glimmer.
        w.actors.get_mut(id).unwrap().inventory.glimmer = 2_000;
        for _ in 0..3 {
            step(&mut w, vec![Command::LevelUp { attribute: 3 }]);
        }
        let p = w.actors.get(id).unwrap();
        assert_eq!(p.inventory.glimmer, 2_000 - 420 - 465 - 512);
        assert_eq!(p.stats.attrs.level(&w.attributes), 13);
        give_quills(&mut w, 1);
        let glimmer = w.actors.get(id).unwrap().inventory.glimmer;
        let level = w.actors.get(id).unwrap().stats.attrs.level(&w.attributes);

        // Take two Might points back and spread them over Insight and Devotion.
        let target = [10, 10, 10, 13, 12, 9, 9];
        step(&mut w, vec![Command::Respec { attributes: target }]);

        let p = w.actors.get(id).unwrap();
        assert_eq!(quills_held(&w), 0, "one quill consumed");
        assert_eq!(p.stats.attrs, level::from_values(target));
        assert_eq!(p.inventory.glimmer, glimmer, "respec costs no Glimmer");
        assert_eq!(p.stats.attrs.level(&w.attributes), level, "the level stays");
    }

    #[test]
    fn respec_resets_to_the_class_base_plus_the_free_points() {
        let mut w = field();
        rest(&mut w);
        let id = w.player;
        w.actors.get_mut(id).unwrap().inventory.glimmer = 2_000;
        for attribute in [0, 0, 5] {
            step(&mut w, vec![Command::LevelUp { attribute }]);
        }
        give_quills(&mut w, 1);
        // Vitality and Insight went up by two and one. The respec puts all three free
        // points into Focus, which resets the other attributes to the base.
        step(
            &mut w,
            vec![Command::Respec {
                attributes: [10, 13, 10, 12, 12, 8, 8],
            }],
        );
        let p = w.actors.get(id).unwrap();
        assert_eq!(p.stats.attrs.level(&w.attributes), 13, "73 points - 60");
        assert_eq!(p.stats.attrs.vitality, BASE[0]);
        assert_eq!(p.stats.attrs.insight, BASE[5]);
        assert_eq!(p.stats.attrs.focus, 13);
    }

    #[test]
    fn respec_lowers_the_maxima_and_clamps_the_current_values() {
        let mut w = field();
        rest(&mut w);
        let id = w.player;
        w.actors.get_mut(id).unwrap().inventory.glimmer = 2_000;
        for _ in 0..3 {
            step(&mut w, vec![Command::LevelUp { attribute: 0 }]);
        }
        let hp_high = w.actors.get(id).unwrap().max_hp;
        give_quills(&mut w, 1);
        // Full HP at the high maximum, then the three Vitality points go to Insight.
        w.actors.get_mut(id).unwrap().hp = hp_high;
        step(
            &mut w,
            vec![Command::Respec {
                attributes: [10, 10, 10, 12, 12, 11, 8],
            }],
        );
        let p = w.actors.get(id).unwrap();
        assert_eq!(p.max_hp, w.attributes.max_hp_tenths(10) as u32 / 10);
        assert!(p.max_hp < hp_high);
        assert_eq!(p.hp, p.max_hp, "current HP follows the lower maximum");
    }

    #[test]
    fn respec_refuses_without_a_quill_a_rest_or_a_valid_spread() {
        let mut w = field();
        let id = w.player;
        // One point above the starting loadout (Might 13, total 71), so a respec has
        // something to move. Set directly: this test is about the respec rules.
        w.actors.get_mut(id).unwrap().stats.attrs.might = 13;
        let before = w.actors.get(id).unwrap().stats.attrs;
        let valid = [10, 10, 10, 12, 12, 9, 8];
        rest(&mut w);

        // Resting, but no quill.
        step(&mut w, vec![Command::Respec { attributes: valid }]);
        assert_eq!(
            w.actors.get(id).unwrap().stats.attrs,
            before,
            "no quill, no respec"
        );

        give_quills(&mut w, 1);
        // Resting, but a changed total: a point appears from nowhere.
        step(
            &mut w,
            vec![Command::Respec {
                attributes: [10, 10, 10, 14, 12, 8, 8],
            }],
        );
        assert_eq!(w.actors.get(id).unwrap().stats.attrs, before);
        // Vitality below the starting loadout, with the total kept.
        step(
            &mut w,
            vec![Command::Respec {
                attributes: [9, 10, 10, 14, 12, 8, 8],
            }],
        );
        assert_eq!(w.actors.get(id).unwrap().stats.attrs, before);
        // No change at all.
        step(
            &mut w,
            vec![Command::Respec {
                attributes: level::values(&before),
            }],
        );
        assert_eq!(w.actors.get(id).unwrap().stats.attrs, before);
        assert_eq!(quills_held(&w), 1, "refused respecs keep the quill");

        // Not resting: refused even with a valid spread and a quill.
        w.state.resting = None;
        step(&mut w, vec![Command::Respec { attributes: valid }]);
        assert_eq!(w.actors.get(id).unwrap().stats.attrs, before);
        assert_eq!(quills_held(&w), 1);
        rest(&mut w);

        // A valid spread with the quill: it goes through.
        step(&mut w, vec![Command::Respec { attributes: valid }]);
        assert_eq!(quills_held(&w), 0);
        let after = w.actors.get(id).unwrap().stats.attrs;
        assert_eq!(after, level::from_values(valid));
        assert_eq!(after.insight, 9);
    }

    fn with_second_lantern(w: &mut World, lit: bool) {
        w.layout.lanterns.push(Lantern {
            id: "lantern_far".into(),
            pos: [10.0, 0.0, 10.0],
            yaw: 1.0,
        });
        if lit {
            w.state.flags.raise(&flags::lantern_lit("lantern_far"));
        }
    }

    #[test]
    fn travel_moves_to_a_lit_lantern_and_rests_there() {
        let mut w = field();
        with_second_lantern(&mut w, true);
        rest(&mut w);
        let id = w.player;
        w.actors.get_mut(id).unwrap().hp = 1;
        step(&mut w, vec![Command::Travel { lantern: 1 }]);
        let p = w.actors.get(id).unwrap();
        assert_eq!(w.state.resting.as_deref(), Some("lantern_far"));
        assert_eq!(w.state.respawn_lantern.as_deref(), Some("lantern_far"));
        assert!((p.pos[0] - 10.0).abs() < 0.5 && (p.pos[2] - 10.0).abs() < 0.5);
        assert_eq!(p.hp, p.max_hp);
    }

    #[test]
    fn travel_refuses_an_unlit_or_missing_lantern_or_no_rest() {
        let mut w = field();
        with_second_lantern(&mut w, false);
        let id = w.player;
        // Not resting.
        step(&mut w, vec![Command::Travel { lantern: 1 }]);
        assert!((w.actors.get(id).unwrap().pos[0]).abs() < 0.5);
        rest(&mut w);
        step(&mut w, vec![Command::Travel { lantern: 1 }]); // unlit
        step(&mut w, vec![Command::Travel { lantern: 9 }]); // missing
        step(&mut w, vec![Command::Travel { lantern: 0 }]); // where the player already is
        assert_eq!(w.state.resting.as_deref(), Some("lantern_field"));
        assert!((w.actors.get(id).unwrap().pos[0]).abs() < 0.5);
    }
}
