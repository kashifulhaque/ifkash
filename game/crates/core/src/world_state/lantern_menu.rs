//! The Lantern menu's commands that change the character or the player's place: level up
//! and travel (`Command::LevelUp`, `Command::Travel`). Split flasks lives in `items`. All of
//! them apply only while resting at a Lantern. See docs/game/06-progression-and-systems.md,
//! "Leveling" and "Lanterns".
//!
//! The level-up cost lives here, not in the browser, so the menu asks the core what the
//! next level costs (`level_cost`). WP-050 (leveling and respec) may move this code into
//! `actor/level.rs` and add respec.

use crate::actor::stats::{Attrs, Derived};
use crate::combat::status::StatusState;
use crate::input::{Command, TickInput};
use crate::world::World;

use super::{flags, lantern};

/// Number of attributes `Command::LevelUp` can raise.
pub const ATTRIBUTE_COUNT: u8 = 7;

/// Glimmer to go from character level `level` to the next:
/// `floor(100 + 20 L + 1.2 L^2)` (docs/game/06-progression-and-systems.md, "Leveling").
/// `12 * L^2 / 10` in integers is the exact floor of `1.2 * L^2`.
pub fn level_cost(level: u32) -> u32 {
    100 + 20 * level + 12 * level * level / 10
}

/// The attribute `index` (the order of `Attrs`), or `None` past the last.
fn attribute_mut(a: &mut Attrs, index: u8) -> Option<&mut u16> {
    Some(match index {
        0 => &mut a.vitality,
        1 => &mut a.focus,
        2 => &mut a.endurance,
        3 => &mut a.might,
        4 => &mut a.finesse,
        5 => &mut a.insight,
        6 => &mut a.devotion,
        _ => return None,
    })
}

/// The derived stats now and after raising `attribute` by one, for the menu's preview.
/// `None` if the player is gone, the attribute doesn't exist, or it is at the maximum.
/// Read-only.
pub fn level_preview(world: &World, attribute: u8) -> Option<(Derived, Derived)> {
    let mut attrs = world.actors.get(world.player)?.stats.attrs;
    let now = attrs.derived(&world.attributes);
    let v = attribute_mut(&mut attrs, attribute)?;
    if *v >= world.attributes.max {
        return None;
    }
    *v += 1;
    Some((now, attrs.derived(&world.attributes)))
}

/// Applies the tick's `LevelUp` and `Travel` commands, in order, while resting.
pub fn run(world: &mut World, input: &TickInput) {
    for c in &input.commands {
        if world.state.resting.is_none() {
            return;
        }
        match *c {
            Command::LevelUp { attribute } => {
                level_up(world, attribute);
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
/// Maximum HP, FP, and stamina grow with their attributes, and the current values grow
/// by the same amount; the resistances that attributes raise are refreshed.
pub fn level_up(world: &mut World, attribute: u8) -> bool {
    let rules = &world.attributes;
    let player = world.player;
    let Some(p) = world.actors.get_mut(player) else {
        return false;
    };
    let cost = level_cost(p.stats.attrs.level(rules));
    if p.dead || p.inventory.glimmer < cost {
        return false;
    }
    let max = rules.max;
    let mut attrs = p.stats.attrs;
    match attribute_mut(&mut attrs, attribute) {
        Some(v) if *v < max => *v += 1,
        _ => return false,
    }
    let before = p.stats.attrs.derived(rules);
    let after = attrs.derived(rules);
    p.inventory.glimmer -= cost;
    p.stats.attrs = attrs;
    p.max_hp = after.max_hp;
    p.max_fp = after.max_fp;
    p.max_stamina = after.max_stamina;
    p.hp = (p.hp + after.max_hp.saturating_sub(before.max_hp)).min(p.max_hp);
    p.fp = (p.fp + (after.max_fp - before.max_fp).max(0.0)).min(p.max_fp);
    p.stamina = (p.stamina + (after.max_stamina - before.max_stamina).max(0.0)).min(p.max_stamina);
    p.status.resist = StatusState::for_player(&attrs, rules).resist;
    true
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
    fn the_cost_curve_matches_the_leveling_table() {
        // The spec's table: cost to the next level.
        for (level, cost) in [(10, 420), (20, 980), (30, 1780), (45, 3430), (60, 5620)] {
            assert_eq!(level_cost(level), cost, "level {level}");
        }
        // The table's cumulative column: 6,518 from level 10 to 20.
        assert_eq!((10..20).map(level_cost).sum::<u32>(), 6518);
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
        let cost = level_cost(level);
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
