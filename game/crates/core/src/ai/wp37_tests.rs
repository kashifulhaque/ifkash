//! Tests for the batch-3 special rules (WP-037): the Mimic Chest's disguise and the Rime
//! Wolves' pack rule.

use lf_content::Content;

use super::*;
use crate::input::{ActionFrame, TickInput, buttons};
use crate::tick::tick;

fn world(scenario: &str, seed: u64) -> World {
    let content = Content::load_embedded().unwrap();
    World::new(seed, &content, scenario).unwrap()
}

fn enemy(w: &World, name: &str) -> Id {
    let i = w
        .layout
        .enemies
        .iter()
        .position(|s| s.name == name)
        .unwrap();
    w.state.ai_enemies[i]
}

fn step(w: &mut World, n: u32, buttons: u16) {
    for _ in 0..n {
        tick(
            w,
            &TickInput {
                frame: ActionFrame {
                    buttons,
                    ..ActionFrame::default()
                },
                commands: Vec::new(),
            },
        );
        w.events.drain();
    }
}

/// Puts the player `z` meters along +Z (the mimic chest is at z = 6).
fn player_at(w: &mut World, z: f32) {
    w.actors.get_mut(w.player).unwrap().pos = [0.0, 0.0, z];
}

fn awareness(w: &World, id: Id) -> Awareness {
    w.actors.get(id).unwrap().ai.as_ref().unwrap().awareness
}

#[test]
fn a_mimic_ignores_a_player_in_plain_sight_and_all_their_noise() {
    let mut w = world("fight_mimic", 1);
    let m = enemy(&w, "mimic");
    // 4 m from the chest, well inside its sight cone, and attacking (noise 10 m).
    player_at(&mut w, 2.0);
    step(&mut w, 300, buttons::LIGHT);
    assert_eq!(awareness(&w, m), Awareness::Idle);
    assert!(special::mimic::dormant(&w, m));
}

#[test]
fn a_mimic_wakes_when_the_player_comes_within_2_m() {
    let mut w = world("fight_mimic", 2);
    let m = enemy(&w, "mimic");
    player_at(&mut w, 3.9);
    step(&mut w, 5, 0);
    assert_eq!(awareness(&w, m), Awareness::Idle, "2.1 m away");
    player_at(&mut w, 4.5);
    step(&mut w, 1, 0);
    assert_eq!(awareness(&w, m), Awareness::Aware, "1.5 m away");
}

#[test]
fn trying_to_open_a_mimic_wakes_it_from_3_m_but_not_from_farther() {
    let mut w = world("fight_mimic", 3);
    let m = enemy(&w, "mimic");
    // 3.5 m away: out of reach of the chest.
    player_at(&mut w, 2.5);
    step(&mut w, 5, buttons::INTERACT);
    assert_eq!(awareness(&w, m), Awareness::Idle);
    // 2.5 m away without INTERACT: still a chest.
    player_at(&mut w, 3.5);
    step(&mut w, 5, 0);
    assert_eq!(awareness(&w, m), Awareness::Idle);
    // Holding INTERACT there wakes it.
    step(&mut w, 1, buttons::INTERACT);
    assert_eq!(awareness(&w, m), Awareness::Aware);
}

#[test]
fn hurting_a_mimic_wakes_it() {
    let mut w = world("fight_mimic", 4);
    let m = enemy(&w, "mimic");
    player_at(&mut w, 0.0);
    w.actors.get_mut(m).unwrap().hp -= 10;
    step(&mut w, 1, 0);
    assert_eq!(awareness(&w, m), Awareness::Aware);
}

#[test]
fn an_awake_mimic_that_loses_the_player_goes_home_and_is_a_chest_again() {
    let mut w = world("fight_mimic", 5);
    let m = enemy(&w, "mimic");
    player_at(&mut w, 4.5);
    step(&mut w, 1, 0);
    assert_eq!(awareness(&w, m), Awareness::Aware);
    // The player leaves for good; the mimic gives up, walks home, heals, and lies dormant.
    w.actors.get_mut(w.player).unwrap().pos = [20.0, 0.0, -20.0];
    w.actors.get_mut(m).unwrap().hp -= 100;
    step(&mut w, 900, 0);
    assert_eq!(awareness(&w, m), Awareness::Idle);
    assert!(special::mimic::dormant(&w, m));
    assert_eq!(w.actors.get(m).unwrap().hp, 900);
}

/// Puts `id` in an attack move as far as the pack rule can tell.
fn pretend_attacking(w: &mut World, id: Id) {
    let b = w.actors.get_mut(id).unwrap().ai.as_mut().unwrap();
    b.awareness = Awareness::Combat;
    b.doing = Doing::Attack {
        option: 0,
        move_num: 0,
    };
}

#[test]
fn a_wolf_waits_while_a_packmate_attacks_and_a_far_one_does_not_count() {
    let mut w = world("fight_rime_wolves", 6);
    let (a, b, c) = (
        enemy(&w, "wolf_a"),
        enemy(&w, "wolf_b"),
        enemy(&w, "wolf_c"),
    );
    for id in [a, b, c] {
        w.actors.get_mut(id).unwrap().ai.as_mut().unwrap().awareness = Awareness::Combat;
    }
    assert!(special::pack::may_attack(&w, a));
    pretend_attacking(&mut w, b);
    assert!(!special::pack::may_attack(&w, a), "b is in a move");
    assert!(!special::pack::may_attack(&w, c), "b is in a move");
    assert!(special::pack::may_attack(&w, b), "b doesn't block itself");
    // A packmate more than 12 m away doesn't count.
    w.actors.get_mut(b).unwrap().pos = [0.0, 0.0, 40.0];
    assert!(special::pack::may_attack(&w, a));
    // Nor does a dead one.
    w.actors.get_mut(b).unwrap().pos = [0.0, 0.0, 12.0];
    assert!(!special::pack::may_attack(&w, a));
    w.actors.get_mut(b).unwrap().dead = true;
    assert!(special::pack::may_attack(&w, a));
}

#[test]
fn a_new_strafe_copies_a_strafing_packmate() {
    let mut w = world("fight_rime_wolves", 7);
    let (a, b) = (enemy(&w, "wolf_a"), enemy(&w, "wolf_b"));
    for id in [a, b] {
        w.actors.get_mut(id).unwrap().ai.as_mut().unwrap().awareness = Awareness::Combat;
    }
    for left in [true, false] {
        let brain = w.actors.get_mut(a).unwrap().ai.as_mut().unwrap();
        brain.doing = Doing::Behavior(Behavior::Strafe { dur: 45 });
        brain.doing_ticks = 30;
        brain.strafe_left = left;
        options::start_behavior(&mut w, b, Behavior::Strafe { dur: 45 });
        assert_eq!(
            w.actors.get(b).unwrap().ai.as_ref().unwrap().strafe_left,
            left
        );
    }
}
