//! WP-051: each class moveset swung at the padded dummy in its golden replay. The replay's
//! state hashes are checked by `golden.rs`; these tests check the player's `Hit` events, so a
//! replay that drifts out of range (and so records a silent miss) fails here with a reason.

use std::path::PathBuf;

use lf_content::Content;
use lf_core::events::Event;
use lf_core::input::TickInput;
use lf_core::replay::{Replay, expand};
use lf_core::{World, tick};

fn replays_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/replays")
}

fn content() -> Content {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    Content::load_dir(&data).expect("content loads")
}

/// Replays `golden` (a file stem in tests/replays) and returns the names of the moves whose
/// hits landed on the dummy, in order.
fn player_hit_moves(golden: &str) -> Vec<String> {
    let content = content();
    let bytes = std::fs::read(replays_dir().join(format!("{golden}.lfr"))).unwrap();
    let replay = Replay::decode(&bytes).unwrap();
    let mut w = World::new(replay.header.seed, &content, &replay.header.scenario).unwrap();
    let mut names = Vec::new();
    for frame in expand(&replay.frames) {
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        for event in w.events.drain() {
            if let Event::Hit(h) = event
                && h.attacker == w.player
            {
                assert!(h.damage > 0, "a hit on the dummy did no damage: {h:?}");
                let name = content
                    .registry
                    .ids
                    .iter()
                    .find(|(_, num)| **num == h.move_num)
                    .map(|(id, _)| id.clone())
                    .unwrap_or_else(|| format!("move {}", h.move_num));
                names.push(name);
            }
        }
    }
    names
}

fn assert_swings(golden: &str, expected: &[&str]) {
    let got = player_hit_moves(golden);
    assert_eq!(got, expected, "{golden}: hits on the dummy, in order");
}

#[test]
fn wp051_dagger_golden_lands_its_chain_and_heavy() {
    assert_swings(
        "wp051_dagger",
        &["dag_light_1", "dag_light_2", "dag_light_3", "dag_heavy"],
    );
}

#[test]
fn wp051_greatsword_golden_lands_its_chain_and_heavy() {
    assert_swings(
        "wp051_greatsword",
        &["gs_light_1", "gs_light_2", "gs_light_3", "gs_heavy"],
    );
}

#[test]
fn wp051_curved_sword_golden_lands_its_chain_and_heavy() {
    assert_swings(
        "wp051_curved_sword",
        &["cs_light_1", "cs_light_2", "cs_light_3", "cs_heavy"],
    );
}

#[test]
fn wp051_thrusting_sword_golden_lands_its_chain_and_heavy() {
    assert_swings(
        "wp051_thrusting_sword",
        &["ts_light_1", "ts_light_2", "ts_light_3", "ts_heavy"],
    );
}

#[test]
fn wp051_axe_golden_lands_its_chain_and_heavy() {
    assert_swings(
        "wp051_axe",
        &["ax_light_1", "ax_light_2", "ax_light_3", "ax_heavy"],
    );
}

#[test]
fn wp051_hammer_golden_lands_its_chain_and_heavy() {
    assert_swings(
        "wp051_hammer",
        &["hm_light_1", "hm_light_2", "hm_light_3", "hm_heavy"],
    );
}
