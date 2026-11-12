//! Assertions about the WP-055 (spell system and Formulae) goldens in `game/tests/replays/`.
//! The golden test in `golden.rs` replays every `.lfr` file against its recorded hashes;
//! each test here runs one replay and checks the spell did what its script says. The rules
//! have unit tests in `game/crates/core/src/combat/spells/tests.rs`.

use std::path::PathBuf;

use lf_content::Content;
use lf_core::arena::Id;
use lf_core::events::{Event, Hit, ProjectileEventKind};
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

/// The world after a replay, every event it raised, and the tick each event came on.
struct Run {
    world: World,
    events: Vec<(u64, Event)>,
    /// The player's z on each tick an awareness event came.
    player_z: Vec<(u64, f32)>,
}

/// Runs the replay `name` (commands included) and checks it is on `scenario`.
fn run(name: &str, scenario: &str) -> Run {
    let content = content();
    let replay =
        Replay::decode(&std::fs::read(replays_dir().join(format!("{name}.lfr"))).unwrap()).unwrap();
    assert_eq!(replay.header.scenario, scenario);
    let mut world = World::new(replay.header.seed, &content, scenario).unwrap();
    let mut events = Vec::new();
    let mut player_z = Vec::new();
    for frame in expand(&replay.frames) {
        let commands = replay
            .commands
            .iter()
            .filter(|(t, _)| *t == world.tick)
            .map(|(_, c)| c.clone())
            .collect();
        tick(&mut world, &TickInput { frame, commands });
        let t = world.tick;
        let z = world.actors.get(world.player).map_or(0.0, |p| p.pos[2]);
        for e in world.events.drain() {
            if matches!(e, Event::Awareness(_)) {
                player_z.push((t, z));
            }
            events.push((t, e));
        }
    }
    Run {
        world,
        events,
        player_z,
    }
}

impl Run {
    /// The hits the player landed, with the tick each landed on.
    fn player_hits(&self) -> Vec<(u64, &Hit)> {
        self.events
            .iter()
            .filter_map(|(t, e)| match e {
                Event::Hit(h) if h.attacker == self.world.player => Some((*t, h)),
                _ => None,
            })
            .collect()
    }

    /// The hits the player took.
    fn hits_taken(&self) -> Vec<&Hit> {
        self.events
            .iter()
            .filter_map(|(_, e)| match e {
                Event::Hit(h) if h.target == self.world.player => Some(h),
                _ => None,
            })
            .collect()
    }

    fn projectiles(&self, kind: ProjectileEventKind) -> usize {
        self.events
            .iter()
            .filter(|(_, e)| matches!(e, Event::Projectile(p) if p.kind == kind))
            .count()
    }

    /// The distinct actors the player hit, sorted.
    fn targets(&self) -> Vec<Id> {
        let mut t: Vec<Id> = self.player_hits().iter().map(|(_, h)| h.target).collect();
        t.sort();
        t.dedup();
        t
    }

    /// The dummy standing at `(x, z)` in `spell_range`: the dummies stand where they are put.
    fn at(&self, x: f32, z: f32) -> Id {
        self.world
            .actors
            .iter()
            .find(|(_, a)| {
                !a.is_player && (a.pos[0] - x).abs() < 0.01 && (a.pos[2] - z).abs() < 0.01
            })
            .map(|(id, _)| id)
            .unwrap_or_else(|| panic!("nobody at ({x}, {z})"))
    }

    /// The only enemy in `spell_ward`: the Husk.
    fn husk(&self) -> Id {
        self.world
            .actors
            .iter()
            .find(|(_, a)| !a.is_player)
            .map(|(id, _)| id)
            .expect("the Husk")
    }

    /// The damage of each of the player's hits, in order.
    fn damage(&self) -> Vec<u32> {
        self.player_hits().iter().map(|(_, h)| h.damage).collect()
    }

    /// True if every player hit was the spell move `id`.
    fn all_hits_are_move(&self, id: &str) -> bool {
        let n = content().registry.num(id).unwrap();
        self.player_hits().iter().all(|(_, h)| h.move_num == n)
    }
}

// Positions in `spell_range`: dummy_front (0, 7), dummy_near (1.3, 2.3), dummy_far (0, 14),
// dummy_left (3, 8), dummy_right (-3, 8).

#[test]
fn m5_spell_aether_dart_hits_the_dummy_ahead_once() {
    let r = run("m5_spell_aether_dart", "spell_range");
    assert_eq!(r.projectiles(ProjectileEventKind::Spawned), 1);
    assert_eq!(r.targets(), vec![r.at(0.0, 7.0)]);
    // Spell power (the stylus at Insight 10, 0.9 scaling since WP-053) times the dart's
    // 0.7 motion value, against 100 flat defense.
    assert_eq!(r.damage(), vec![32]);
    assert!(r.all_hits_are_move("spm_aether_dart"));
}

#[test]
fn m5_spell_aether_bolt_hits_harder_and_slower_than_the_dart() {
    let r = run("m5_spell_aether_bolt", "spell_range");
    assert_eq!(r.targets(), vec![r.at(0.0, 7.0)]);
    assert_eq!(r.damage(), vec![116]);
    let dart = run("m5_spell_aether_dart", "spell_range");
    assert!(r.damage()[0] > dart.damage()[0] * 3);
    // 18 m/s against 30, and a later release frame (14 against 7): it arrives later.
    assert!(r.player_hits()[0].0 > dart.player_hits()[0].0);
}

#[test]
fn m5_spell_homing_motes_launch_five_and_all_find_the_target() {
    let r = run("m5_spell_homing_motes", "spell_range");
    assert_eq!(r.projectiles(ProjectileEventKind::Spawned), 5);
    assert_eq!(r.damage(), vec![25; 5]);
    assert_eq!(r.targets().len(), 1, "they home onto one target");
    // They leave on different frames, so they arrive on different ticks.
    let ticks: Vec<u64> = r.player_hits().iter().map(|(t, _)| *t).collect();
    assert!(ticks.windows(2).all(|w| w[0] <= w[1]) && ticks[0] != ticks[4]);
}

#[test]
fn m5_spell_aether_wave_hits_what_stands_close_in_front() {
    let r = run("m5_spell_aether_wave", "spell_range");
    assert_eq!(
        r.projectiles(ProjectileEventKind::Spawned),
        0,
        "a hitbox, not a projectile"
    );
    assert_eq!(r.targets(), vec![r.at(1.3, 2.3)]);
    assert_eq!(r.damage(), vec![104]);
    assert!(r.all_hits_are_move("spm_aether_wave"));
}

#[test]
fn m5_spell_mirror_ward_absorbs_40_percent_of_three_hits() {
    let r = run("m5_spell_mirror_ward", "spell_ward");
    // The Husk's combo is 62 then 79; the ward takes 40 percent off the first three hits
    // (62, 79, 62 become 37, 47, 37), and the fourth lands in full.
    let taken: Vec<u32> = r.hits_taken().iter().map(|h| h.damage).collect();
    assert_eq!(taken, vec![37, 47, 37, 79]);
    let p = r.world.actors.get(r.world.player).unwrap();
    assert!(
        p.status.body_buff.is_none(),
        "the ward ended on the third hit"
    );
}

#[test]
fn m5_spell_veil_makes_the_husk_notice_at_half_range() {
    let r = run("m5_spell_veil", "spell_ward");
    let husk = r.husk();
    let first = r
        .events
        .iter()
        .find_map(|(t, e)| match e {
            Event::Awareness(a) if a.actor == husk => Some(*t),
            _ => None,
        })
        .expect("the Husk notices the player in the end");
    let z = r.player_z.iter().find(|(t, _)| *t == first).unwrap().1;
    // A Husk sees 14 m; with the Veil it notices at 7 m or less (the ward replay has it
    // notice at 14 m).
    let distance = 16.0 - z;
    assert!((5.0..=7.0).contains(&distance), "noticed at {distance} m");
    let ward = run("m5_spell_mirror_ward", "spell_ward");
    let ward_first = ward
        .events
        .iter()
        .find_map(|(t, e)| matches!(e, Event::Awareness(_)).then_some(*t))
        .unwrap();
    let ward_z = ward
        .player_z
        .iter()
        .find(|(t, _)| *t == ward_first)
        .unwrap()
        .1;
    assert!(
        16.0 - ward_z > 13.0,
        "without the Veil it noticed at {} m",
        16.0 - ward_z
    );
    let p = r.world.actors.get(r.world.player).unwrap();
    assert!(p.status.body_buff.is_some(), "the Veil is still up");
}

#[test]
fn m5_spell_gradient_descent_arcs_onto_the_dummy_with_heavy_stance_damage() {
    let r = run("m5_spell_gradient_descent", "spell_range");
    assert_eq!(r.projectiles(ProjectileEventKind::Spawned), 1);
    assert_eq!(r.targets(), vec![r.at(0.0, 7.0)]);
    assert_eq!(r.damage(), vec![202]);
    // It goes up first, so it lands later than a flat bolt aimed at the same dummy.
    let bolt = run("m5_spell_aether_bolt", "spell_range");
    assert!(r.player_hits()[0].0 > bolt.player_hits()[0].0);
    let content = content();
    let p = content.projectile_by_id("spp_gradient_descent").unwrap();
    assert!(p.poise_damage >= 60, "high stance damage");
}

#[test]
fn m5_spell_glasswing_swarm_fans_five_shards_across_the_group() {
    let r = run("m5_spell_glasswing_swarm", "spell_range");
    assert_eq!(r.projectiles(ProjectileEventKind::Spawned), 5);
    assert_eq!(r.targets().len(), 3, "a spread hits a group");
    assert_eq!(r.damage(), vec![22; 3]);
}

#[test]
fn m5_spell_unbinding_lance_pierces_and_the_charged_form_hits_harder() {
    let r = run("m5_spell_unbinding_lance", "spell_range");
    assert_eq!(r.projectiles(ProjectileEventKind::Spawned), 2);
    let (front, far) = (r.at(0.0, 7.0), r.at(0.0, 14.0));
    let hits = r.player_hits();
    // The plain lance goes through dummy_front and dummy_far; so does the charged one.
    let order: Vec<Id> = hits.iter().map(|(_, h)| h.target).collect();
    assert_eq!(order, vec![front, far, front, far]);
    let d = r.damage();
    assert_eq!(d[0], d[1]);
    assert_eq!(d[2], d[3]);
    assert!(d[2] > d[0], "the charged form hits harder: {d:?}");
    let n = content()
        .registry
        .num("spm_unbinding_lance_charged")
        .unwrap();
    assert_eq!(hits[2].1.move_num, n);
}
