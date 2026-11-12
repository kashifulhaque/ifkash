//! Assertions about the WP-036 (enemy batch 2) goldens in `game/tests/replays/`. The golden
//! test in `golden.rs` replays every `.lfr` file against its recorded hashes; each test
//! here runs one replay and checks the fight did what its script says.

use std::path::PathBuf;

use lf_content::Content;
use lf_core::arena::Id;
use lf_core::events::{Event, Hit, ProjectileEventKind};
use lf_core::replay::{Replay, expand};
use lf_core::{World, tick};

fn replays_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/replays")
}

fn content() -> Content {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    Content::load_dir(&data).expect("content loads")
}

/// Runs the replay `name` on `scenario` and returns the final world and every event.
fn run_duel(name: &str, scenario: &str) -> (World, Vec<Event>) {
    use lf_core::input::TickInput;

    let content = content();
    let replay =
        Replay::decode(&std::fs::read(replays_dir().join(format!("{name}.lfr"))).unwrap()).unwrap();
    assert_eq!(replay.header.scenario, scenario);
    let mut w = World::new(replay.header.seed, &content, scenario).unwrap();
    let mut events = Vec::new();
    for frame in expand(&replay.frames) {
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        events.extend(w.events.drain());
    }
    (w, events)
}

fn hits(events: &[Event]) -> Vec<&Hit> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Hit(h) => Some(h),
            _ => None,
        })
        .collect()
}

/// The hits `by` landed on the player.
fn on_player<'a>(w: &World, events: &'a [Event], by: Id) -> Vec<&'a Hit> {
    hits(events)
        .into_iter()
        .filter(|h| h.attacker == by && h.target == w.player)
        .collect()
}

fn has_status(w: &World, kind: lf_content::moves::StatusKind) -> bool {
    let p = w.actors.get(w.player).unwrap();
    let i = lf_core::combat::status::index(kind);
    p.status.meter_milli[i] > 0 || p.status.active[i] > 0
}

/// The Acolyte glows and casts: the bolt lands (82 damage) and the Acolyte dies to lights.
#[test]
fn m3_acolyte_casts_a_bolt_and_dies() {
    let (w, events) = run_duel("m3_acolyte", "fight_acolyte");
    let ac = w.state.ai_enemies[0];
    let casts = events
        .iter()
        .filter(|e| matches!(e, Event::Projectile(p) if p.shooter == ac && p.kind == ProjectileEventKind::Spawned))
        .count();
    assert!(casts >= 1, "{casts}");
    let cast = w.moves.num_of("ac_cast").unwrap();
    let bolts: Vec<_> = on_player(&w, &events, ac)
        .into_iter()
        .filter(|h| h.move_num == cast)
        .collect();
    assert!(!bolts.is_empty());
    assert!(bolts.iter().all(|h| h.damage == 82), "{bolts:?}");
    assert!(w.actors.get(ac).unwrap().dead);
}

/// The Drowned wades to its pool's edge and grabs: the hit can't be blocked and costs about a
/// fifth of the player's 570 HP. It never leaves the pool (z 3 to 9 in the scenario), apart
/// from the step its grab's root motion carries it.
#[test]
fn m3_drowned_grabs_from_its_pool() {
    let (w, events) = run_duel("m3_drowned", "fight_drowned");
    let d = w.state.ai_enemies[0];
    let grab = w.moves.num_of("dr_grab").unwrap();
    let grabs: Vec<_> = on_player(&w, &events, d)
        .into_iter()
        .filter(|h| h.move_num == grab)
        .collect();
    assert_eq!(grabs.len(), 1, "{grabs:?}");
    assert_eq!(grabs[0].damage, 118);
    let max = w.actors.get(w.player).unwrap().max_hp;
    assert!((20..=21).contains(&(grabs[0].damage * 100 / max)));
    let a = w.actors.get(d).unwrap();
    assert!(a.pos[2] >= 2.8, "it stayed in the water: {:?}", a.pos);
    assert!(a.dead, "the player cut it down");
}

/// The Reed Stalker stays hidden until the player is within 4.5 m, lunges exactly once, and
/// is hidden in the reeds again at the end.
#[test]
fn m3_reed_stalker_lunges_once_then_burrows() {
    let (w, events) = run_duel("m3_reed_stalker", "fight_reed_stalker");
    let rs = w.state.ai_enemies[0];
    let lunges = on_player(&w, &events, rs);
    assert_eq!(lunges.len(), 1, "{lunges:?}");
    assert_eq!(lunges[0].move_num, w.moves.num_of("rs_lunge").unwrap());
    assert_eq!(lunges[0].damage, 154);
    let b = w.actors.get(rs).unwrap().ai.as_ref().unwrap();
    assert!(b.hidden, "back in the reeds");
}

/// Five Marsh Crawlers bite with Poison, and the player kills them all.
#[test]
fn m3_marsh_crawlers_swarm_and_poison() {
    let (w, events) = run_duel("m3_marsh_crawlers", "fight_marsh_crawlers");
    let bite = w.moves.num_of("mc_bite").unwrap();
    let bites: Vec<_> = hits(&events)
        .into_iter()
        .filter(|h| h.target == w.player && h.move_num == bite)
        .collect();
    assert!(bites.len() >= 3, "{bites:?}");
    let killed = hits(&events).iter().filter(|h| h.killed).count();
    assert_eq!(killed, 5);
    assert!(has_status(&w, lf_content::moves::StatusKind::Poison));
}

/// The Cave Moth hovers 0.9 m up, dives (45 damage), and shakes Drowse dust (12 damage and
/// buildup).
#[test]
fn m3_cave_moth_dives_and_dusts() {
    let (w, events) = run_duel("m3_cave_moth", "fight_cave_moth");
    let moth = w.state.ai_enemies[0];
    let hs = on_player(&w, &events, moth);
    let by = |id: &str| {
        let n = w.moves.num_of(id).unwrap();
        hs.iter().filter(|h| h.move_num == n).count()
    };
    assert!(by("cm_dive") >= 1, "{hs:?}");
    assert!(by("cm_dust") >= 1, "{hs:?}");
    assert!(hs.iter().all(|h| h.damage == 45 || h.damage == 12));
    let a = w.actors.get(moth).unwrap();
    assert!((a.pos[1] - 0.9).abs() < 0.05, "it hovers: {:?}", a.pos);
    assert!(has_status(&w, lf_content::moves::StatusKind::Drowse));
}

/// All six Lamp Mites cling, blow up for Fire (39 each), and die doing it.
#[test]
fn m3_lamp_mites_cling_and_explode() {
    let (w, events) = run_duel("m3_lamp_mites", "fight_lamp_mites");
    let burst = w.moves.num_of("lm_burst").unwrap();
    let blasts: Vec<_> = hits(&events)
        .into_iter()
        .filter(|h| h.target == w.player && h.move_num == burst)
        .collect();
    assert_eq!(blasts.len(), 6, "{blasts:?}");
    assert!(blasts.iter().all(|h| h.damage == 39));
    let who: std::collections::BTreeSet<_> = blasts.iter().map(|h| h.attacker).collect();
    assert_eq!(who.len(), 6, "one blast each");
    assert!(
        w.state
            .ai_enemies
            .iter()
            .all(|&id| w.actors.get(id).unwrap().dead)
    );
}

/// The Shard Golem slams (190 damage) and takes the player's lights.
#[test]
fn m3_shard_golem_slams() {
    let (w, events) = run_duel("m3_shard_golem", "fight_shard_golem");
    let g = w.state.ai_enemies[0];
    let slam = w.moves.num_of("sg_slam").unwrap();
    let hs = on_player(&w, &events, g);
    assert!(
        hs.iter().any(|h| h.move_num == slam && h.damage == 190),
        "{hs:?}"
    );
    let a = w.actors.get(g).unwrap();
    assert!(a.hp < a.max_hp);
}

/// The Glasswing Stalker lunges (118) and slashes (75).
#[test]
fn m3_glasswing_lunges_and_slashes() {
    let (w, events) = run_duel("m3_glasswing", "fight_glasswing");
    let gw = w.state.ai_enemies[0];
    let hs = on_player(&w, &events, gw);
    let lunge = w.moves.num_of("gw_lunge").unwrap();
    let slash = w.moves.num_of("gw_slash").unwrap();
    assert!(
        hs.iter().any(|h| h.move_num == lunge && h.damage == 118),
        "{hs:?}"
    );
    assert!(
        hs.iter().any(|h| h.move_num == slash && h.damage == 75),
        "{hs:?}"
    );
}

/// The Bell Acolyte's Rime bolt lands and builds Rime on the player.
#[test]
fn m3_acolyte_bell_casts_a_rime_bolt() {
    let (w, events) = run_duel("m3_acolyte_bell", "fight_acolyte_bell");
    let ac = w.state.ai_enemies[0];
    let cast = w.moves.num_of("ac_bell_cast").unwrap();
    let bolts: Vec<_> = on_player(&w, &events, ac)
        .into_iter()
        .filter(|h| h.move_num == cast)
        .collect();
    assert!(!bolts.is_empty(), "no bolt landed");
    assert!(has_status(&w, lf_content::moves::StatusKind::Rime));
}
