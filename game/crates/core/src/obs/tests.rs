use lf_content::Content;

use super::action::act;
use super::*;
use crate::ai::boss::Stage;
use crate::input::{ActionFrame, TickInput, buttons};
use crate::tick::tick;
use crate::world::World;

fn step(w: &mut World, frame: ActionFrame) {
    tick(
        w,
        &TickInput {
            frame,
            commands: Vec::new(),
        },
    );
    w.events.drain();
}

/// The B01 arena with the player walked through the fog gate and the fight on.
fn in_fight(seed: u64) -> World {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(seed, &content, "boss_b01").unwrap();
    while w.bosses[0].stage != Stage::Fight {
        assert!(w.tick < 600, "the fight never started");
        let move_z = if w.bosses[0].stage == Stage::Dormant {
            127
        } else {
            0
        };
        step(
            &mut w,
            ActionFrame {
                move_x: 0,
                move_z,
                buttons: 0,
            },
        );
    }
    w
}

#[test]
fn encoders_are_pure_reads() {
    let w = in_fight(3);
    let before = w.state_hash();
    let (a, b, c) = (proprio(&w), target(&w), move_ids(&w));
    assert_eq!(w.state_hash(), before);
    assert_eq!(
        (proprio(&w), target(&w), move_ids(&w)),
        (a, b, c),
        "same world, same vectors"
    );
}

#[test]
fn vectors_are_finite_and_scaled_through_a_fight() {
    let mut w = in_fight(5);
    for i in 0..600u32 {
        let o = proprio(&w);
        let t = target(&w);
        for v in o.iter().chain(t.iter()) {
            assert!(v.is_finite() && (-1.0..=1.0).contains(v), "tick {i}: {v}");
        }
        let buttons = if i % 20 == 0 { buttons::LIGHT } else { 0 };
        step(
            &mut w,
            ActionFrame {
                move_x: 0,
                move_z: 127,
                buttons,
            },
        );
        if w.actors.get(w.player).unwrap().hp == 0 {
            break;
        }
    }
}

#[test]
fn the_target_is_the_boss_at_full_hp_when_the_fight_starts() {
    let w = in_fight(1);
    assert_eq!(target_id(&w), boss_body(&w, 0));
    let t = target(&w);
    assert_eq!(t[0], 1.0, "present");
    assert_eq!(t[14], 1.0, "full HP");
    assert_eq!(t[29], 1.0, "the fight is on");
    // The boss is ahead of the player, who walked in facing +Z.
    assert!(t[4] > 0.0 && t[7] > 0.9, "ahead: {t:?}");
    let p = proprio(&w);
    assert_eq!(p[0], 1.0);
    assert!(p[14] > 0.0, "flasks");
}

#[test]
fn forward_is_the_facing_and_taps_press_once_per_step() {
    let w = in_fight(1);
    let d = ActionDecoder::default();
    let yaw = w.actors.get(w.player).unwrap().yaw;
    let heads = [1, 1, act::LIGHT, 0, 0];
    let f0 = d.frame(&w, heads, 0);
    let f1 = d.frame(&w, heads, 1);
    assert_eq!(f0.buttons, buttons::LIGHT);
    assert_eq!(f1.buttons, 0);
    let expect = (
        (crate::math::dmath::sin(yaw) * 127.0).round() as i8,
        (crate::math::dmath::cos(yaw) * 127.0).round() as i8,
    );
    assert_eq!((f0.move_x, f0.move_z), expect);
    // Right of +Z is -X (yaw grows toward +X, the left).
    let right = d.frame(&w, [3, 1, 0, 0, 0], 0);
    if yaw.abs() < 0.01 {
        assert_eq!((right.move_x, right.move_z), (-127, 0));
    }
    // A walk stays under the walk threshold.
    let walk = d.frame(&w, [1, 0, 0, 0, 0], 0);
    assert!(f32::from(walk.move_z.unsigned_abs()) < w.locomotion.walk_below);
}

#[test]
fn dodge_on_consecutive_steps_is_held_and_guard_is_always_held() {
    let w = in_fight(1);
    let mut d = ActionDecoder::default();
    let dodge = [0, 0, act::DODGE, 0, 0];
    assert_eq!(d.frame(&w, dodge, 1).buttons, 0, "first dodge taps");
    d.finish_step(dodge);
    assert_eq!(d.frame(&w, dodge, 1).buttons, buttons::DODGE, "then holds");
    let guard = [0, 0, act::GUARD, 0, 0];
    assert_eq!(d.frame(&w, guard, 1).buttons, buttons::GUARD);
    // Out-of-range heads count as none.
    assert_eq!(d.frame(&w, [200, 9, 77, 9, 9], 0), ActionFrame::default());
}

#[test]
fn the_mask_drops_use_item_without_charges_and_attacks_without_stamina() {
    let mut w = in_fight(1);
    let use_item = usize::from(HEADS[0] + HEADS[1] + act::USE_ITEM);
    let light = usize::from(HEADS[0] + HEADS[1] + act::LIGHT);
    let switch = usize::from(HEADS[0] + HEADS[1] + HEADS[2] + 2);
    let m = action_mask(&w);
    assert!(m[use_item] && m[light]);
    assert!(!m[switch], "not locked on");
    let p = w.actors.get_mut(w.player).unwrap();
    p.inventory.flasks.vigil = 0;
    p.stamina = 0.0;
    let m = action_mask(&w);
    assert!(!m[use_item] && !m[light]);
    assert_eq!(
        m.len(),
        HEADS.iter().map(|&h| usize::from(h)).sum::<usize>()
    );
}
