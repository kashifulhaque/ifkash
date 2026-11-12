//! WP-052: each class moveset of batch B swung at the padded dummy in its golden replay. The
//! replay's state hashes are checked by `golden.rs`; these tests check the player's `Hit`
//! events, so a replay that drifts out of range (and so records a silent miss) fails here
//! with a reason. They also check what makes each class different: the colossal sword's
//! two-handing, the spear's and shield's guard attacks, and the bow's free and locked aim.

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

/// One landed hit of the player's: the move's string ID and the damage.
struct Landed {
    name: String,
    damage: u32,
}

/// What a replay did: the player's landed hits in order, and the ticks on which the
/// player's guard was up while it played the move `watch`.
struct Run {
    hits: Vec<Landed>,
    guarded_during_watch: usize,
    watch_ticks: usize,
}

fn run(golden: &str, watch: Option<&str>) -> Run {
    let content = content();
    let bytes = std::fs::read(replays_dir().join(format!("{golden}.lfr"))).unwrap();
    let replay = Replay::decode(&bytes).unwrap();
    let mut w = World::new(replay.header.seed, &content, &replay.header.scenario).unwrap();
    let watch_num = watch.map(|id| content.registry.num(id).expect("watched move exists"));
    let mut out = Run {
        hits: Vec::new(),
        guarded_during_watch: 0,
        watch_ticks: 0,
    };
    for frame in expand(&replay.frames) {
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        let p = w.actors.get(w.player).unwrap();
        if let (Some(c), Some(n)) = (p.moves.current, watch_num)
            && c.num == n
        {
            out.watch_ticks += 1;
            if p.moves.guard.active {
                out.guarded_during_watch += 1;
            }
        }
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
                out.hits.push(Landed {
                    name,
                    damage: h.damage,
                });
            }
        }
    }
    out
}

fn names(r: &Run) -> Vec<&str> {
    r.hits.iter().map(|h| h.name.as_str()).collect()
}

fn assert_swings(golden: &str, expected: &[&str]) -> Run {
    let r = run(golden, None);
    assert_eq!(names(&r), expected, "{golden}: hits on the dummy, in order");
    r
}

#[test]
fn wp052_colossal_golden_lands_its_chain_heavy_and_two_handed_swing() {
    let r = assert_swings(
        "wp052_colossal",
        &[
            "col_light_1",
            "col_light_2",
            "col_light_3",
            "col_heavy",
            "col_light_1",
        ],
    );
    // The same swing, two-handed (Might x1.5), hits harder.
    assert!(
        r.hits[4].damage > r.hits[0].damage,
        "two-handed {} should beat one-handed {}",
        r.hits[4].damage,
        r.hits[0].damage
    );
}

#[test]
fn wp052_spear_golden_lands_its_chain_heavy_and_guard_thrusts() {
    let r = run("wp052_spear", Some("sp_guard_thrust"));
    assert_eq!(
        names(&r),
        [
            "sp_light_1",
            "sp_light_2",
            "sp_light_3",
            "sp_heavy",
            "sp_guard_thrust",
            "sp_guard_thrust"
        ],
        "hits on the dummy, in order"
    );
    assert!(r.watch_ticks > 0, "the guard thrust ran");
    assert_eq!(
        r.guarded_during_watch, r.watch_ticks,
        "the guard stays raised through the thrust"
    );
}

#[test]
fn wp052_halberd_golden_lands_its_chain_and_heavy() {
    assert_swings(
        "wp052_halberd",
        &["hb_light_1", "hb_light_2", "hb_light_3", "hb_heavy"],
    );
}

#[test]
fn wp052_fist_golden_lands_its_string_and_heavy() {
    assert_swings(
        "wp052_fist",
        &[
            "fi_light_1",
            "fi_light_2",
            "fi_light_3",
            "fi_light_4",
            "fi_heavy",
        ],
    );
}

#[test]
fn wp052_bow_golden_lands_free_aimed_locked_and_drawn_shots() {
    assert_swings(
        "wp052_bow",
        &["bow_shot", "bow_shot", "bow_heavy", "bow_heavy_charged"],
    );
}

#[test]
fn wp052_shield_golden_lands_its_bashes_rush_and_guard_bash() {
    let r = run("wp052_shield", Some("sh_guard_bash"));
    assert_eq!(
        names(&r),
        [
            "sh_bash_1",
            "sh_bash_2",
            "sh_bash_3",
            "sh_heavy",
            "sh_guard_bash"
        ],
        "hits on the dummy, in order"
    );
    assert!(r.watch_ticks > 0, "the guard bash ran");
    assert_eq!(
        r.guarded_during_watch, r.watch_ticks,
        "the guard stays raised through the bash"
    );
}

#[test]
fn wp052_catalyst_golden_lands_its_chain_and_heavy() {
    assert_swings(
        "wp052_catalyst",
        &["ct_light_1", "ct_light_2", "ct_light_3", "ct_heavy"],
    );
}
