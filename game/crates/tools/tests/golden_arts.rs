//! Assertions about the WP-054 (Arts) goldens in `game/tests/replays/`. The golden test in
//! `golden.rs` replays every `.lfr` file against its recorded hashes; each test here runs one
//! replay and checks the Art did what its script says. Each Art's scenario is
//! `game/data/scenarios/m5_art_<name>.ron`, and the weapon is the Art's default weapon.
//! The rules have unit tests in `game/crates/core/src/combat/arts.rs`.

use std::path::PathBuf;

use lf_content::{Content, NumId};
use lf_core::combat::moves::flags;
use lf_core::events::{Event, Hit, HitResult, ProjectileEventKind};
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

/// The world after a replay, every event it raised with its tick, and the player's state
/// after each tick: FP, z, and move flags.
struct Run {
    world: World,
    events: Vec<(u64, Event)>,
    fp: Vec<f32>,
    z: Vec<f32>,
    flags: Vec<u8>,
}

/// Runs the replay `name` on `scenario` and records the player's state each tick.
fn run(name: &str, scenario: &str) -> Run {
    let content = content();
    let replay =
        Replay::decode(&std::fs::read(replays_dir().join(format!("{name}.lfr"))).unwrap()).unwrap();
    assert_eq!(replay.header.scenario, scenario);
    let mut world = World::new(replay.header.seed, &content, scenario).unwrap();
    let mut events = Vec::new();
    let (mut fp, mut z, mut fl) = (Vec::new(), Vec::new(), Vec::new());
    for frame in expand(&replay.frames) {
        let commands = replay
            .commands
            .iter()
            .filter(|(t, _)| *t == world.tick)
            .map(|(_, c)| c.clone())
            .collect();
        tick(&mut world, &TickInput { frame, commands });
        let t = world.tick;
        for e in world.events.drain() {
            events.push((t, e));
        }
        let p = world.actors.get(world.player);
        fp.push(p.map_or(0.0, |p| p.fp));
        z.push(p.map_or(0.0, |p| p.pos[2]));
        fl.push(p.map_or(0, |p| p.moves.flags));
    }
    Run {
        world,
        events,
        fp,
        z,
        flags: fl,
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

    /// The hits the player took, with the tick each landed on.
    fn hits_taken(&self) -> Vec<(u64, &Hit)> {
        self.events
            .iter()
            .filter_map(|(t, e)| match e {
                Event::Hit(h) if h.target == self.world.player => Some((*t, h)),
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

    fn parries(&self) -> Vec<(u64, NumId)> {
        self.events
            .iter()
            .filter_map(|(t, e)| match e {
                Event::Parry(p) if p.target == self.world.player => Some((*t, p.move_num)),
                _ => None,
            })
            .collect()
    }

    /// The registry number of a move by its string ID.
    fn move_num(&self, id: &str) -> NumId {
        content().registry.num(id).unwrap()
    }

    /// True if every player hit was the move `id`, and there is at least one.
    fn only_hits_from(&self, id: &str) -> bool {
        let n = self.move_num(id);
        let hits = self.player_hits();
        !hits.is_empty() && hits.iter().all(|(_, h)| h.move_num == n)
    }

    /// FP the player paid on the press tick: the FP after the tick before minus the FP after
    /// the press tick (the press is the tick index `press`).
    fn fp_paid(&self, press: usize) -> f32 {
        self.fp[press - 1] - self.fp[press]
    }
}

/// The weapon's guard counter move, by weapon ID.
fn guard_counter_of(weapon: &str) -> NumId {
    let c = content();
    let w = c
        .all_weapons()
        .map(|(_, w)| w)
        .find(|w| w.id == weapon)
        .unwrap_or_else(|| panic!("{weapon} is a weapon"));
    c.registry.num(&w.moveset.guard_counter).unwrap()
}

/// The FP each Art costs in the script, which is the press tick's index in its replay (the
/// first 10 ticks settle, so ART is pressed on tick 10).
const PRESS: usize = 10;

#[test]
fn m5_art_feint_step_dashes_with_iframes_for_6_fp() {
    let r = run("m5_art_feint_step", "m5_art_feint_step");
    assert_eq!(r.fp_paid(PRESS), 6.0);
    // Root motion covers 1.8 m forward by frame 12, and the dash's i-frames are frames 1 to 8.
    let dash = r.z[PRESS + 12] - r.z[PRESS - 1];
    assert!((dash - 1.8).abs() < 0.05, "dashed {dash} m");
    let iframes: Vec<usize> = (PRESS..PRESS + 20)
        .filter(|&i| r.flags[i] & flags::IFRAMES != 0)
        .map(|i| i - PRESS)
        .collect();
    assert_eq!(iframes, (1..=8).collect::<Vec<_>>());
    assert!(r.player_hits().is_empty());
}

#[test]
fn m5_art_rending_sweep_spins_for_10_fp_and_double_rupture() {
    let r = run("m5_art_rending_sweep", "m5_art_rending_sweep");
    assert_eq!(r.fp_paid(PRESS), 10.0);
    // The hitbox is out on frames 12 to 21 (ticks 22 to 31), and the dummy is struck once.
    assert!(r.only_hits_from("amv_rending_sweep"));
    assert_eq!(r.player_hits().len(), 1);
    let t = r.player_hits()[0].0;
    assert!((22..=31).contains(&t), "hit on tick {t}");
}

#[test]
fn m5_art_bastion_counters_the_next_block_by_itself() {
    let r = run("m5_art_bastion", "m5_art_bastion");
    assert_eq!(r.fp_paid(PRESS), 0.0);
    // The duelist's first swing (tick 52) is blocked. The next block starts the shield's guard
    // counter without a HEAVY, and the counter ends the stance, so the second swing (tick 130)
    // isn't blocked.
    let blocked: Vec<u64> = r
        .hits_taken()
        .iter()
        .filter(|(_, h)| h.result == HitResult::Guarded)
        .map(|(t, _)| *t)
        .collect();
    assert_eq!(blocked, vec![52]);
    let counter = guard_counter_of("wpn_castle_kite");
    let counters: Vec<u64> = r
        .player_hits()
        .iter()
        .filter(|(_, h)| h.move_num == counter)
        .map(|(t, _)| *t)
        .collect();
    assert_eq!(counters.len(), 1, "counters on {counters:?}");
    assert!((53..=64).contains(&counters[0]), "counter on {counters:?}");
}

#[test]
fn m5_art_windward_parry_parries_the_swing_from_its_stance() {
    let r = run("m5_art_windward_parry", "m5_art_windward_parry");
    assert_eq!(r.fp_paid(48), 0.0);
    // The stance is pressed on tick 48, so its 5-tick window covers the swing's hit on 52 to 54.
    let parries = r.parries();
    assert_eq!(parries.len(), 1, "{parries:?}");
    assert_eq!(parries[0].1, r.move_num("dummy_slash"));
    assert!(r.hits_taken().is_empty(), "a parried swing deals nothing");
}

#[test]
fn m5_art_rising_cleave_slashes_up_for_10_fp() {
    let r = run("m5_art_rising_cleave", "m5_art_rising_cleave");
    assert_eq!(r.fp_paid(PRESS), 10.0);
    assert!(r.only_hits_from("amv_rising_cleave"));
}

#[test]
fn m5_art_rallying_roar_buffs_the_body_for_12_fp() {
    let r = run("m5_art_rallying_roar", "m5_art_rallying_roar");
    assert_eq!(r.fp_paid(PRESS), 12.0);
    // The buff lands on frame 14, which is tick 24.
    let buff = r.world.status.buff_num("buf_rallying_roar").unwrap();
    let body = r.world.actors.get(r.world.player).unwrap().status.body_buff;
    assert_eq!(body.map(|b| b.num), Some(buff));
    assert!(r.player_hits().is_empty());
}

#[test]
fn m5_art_lunging_thrust_lunges_and_thrusts_for_9_fp() {
    let r = run("m5_art_lunging_thrust", "m5_art_lunging_thrust");
    assert_eq!(r.fp_paid(PRESS), 9.0);
    // The 4 m lunge carries the player most of the way to the dummy 3 m ahead.
    assert!(r.z[PRESS + 14] - r.z[PRESS - 1] > 3.0);
    assert!(r.only_hits_from("amv_lunging_thrust"));
}

#[test]
fn m5_art_earthsplit_shocks_the_ground_for_14_fp() {
    let r = run("m5_art_earthsplit", "m5_art_earthsplit");
    assert_eq!(r.fp_paid(PRESS), 14.0);
    // The shockwave is on frames 30 to 33, which are ticks 40 to 43.
    assert!(r.only_hits_from("amv_earthsplit"));
    assert!(r.player_hits().iter().all(|(t, _)| (40..=43).contains(t)));
}

#[test]
fn m5_art_rime_edge_coats_the_weapon_for_15_fp() {
    let r = run("m5_art_rime_edge", "m5_art_rime_edge");
    assert_eq!(r.fp_paid(PRESS), 15.0);
    let buff = r.world.status.buff_num("buf_rime_edge").unwrap();
    let weapon = r.world.actors.get(r.world.player).unwrap().status.weapon_buff;
    assert_eq!(weapon.map(|b| b.num), Some(buff));
}

#[test]
fn m5_art_radiant_blade_adds_radiance_for_15_fp() {
    let r = run("m5_art_radiant_blade", "m5_art_radiant_blade");
    assert_eq!(r.fp_paid(PRESS), 15.0);
    let buff = r.world.status.buff_num("buf_radiant_blade").unwrap();
    let weapon = r.world.actors.get(r.world.player).unwrap().status.weapon_buff;
    assert_eq!(weapon.map(|b| b.num), Some(buff));
}

#[test]
fn m5_art_spark_volley_fires_three_arrows_for_10_fp() {
    let r = run("m5_art_spark_volley", "m5_art_spark_volley");
    assert_eq!(r.fp_paid(PRESS), 10.0);
    assert_eq!(r.projectiles(ProjectileEventKind::Spawned), 3);
}

#[test]
fn m5_art_lantern_flare_burns_in_front_for_12_fp() {
    let r = run("m5_art_lantern_flare", "m5_art_lantern_flare");
    assert_eq!(r.fp_paid(PRESS), 12.0);
    assert!(r.only_hits_from("amv_lantern_flare"));
}

#[test]
fn m5_art_glass_barrage_fires_five_shards_for_14_fp() {
    let r = run("m5_art_glass_barrage", "m5_art_glass_barrage");
    assert_eq!(r.fp_paid(PRESS), 14.0);
    assert_eq!(r.projectiles(ProjectileEventKind::Spawned), 5);
}

#[test]
fn m5_art_toll_stuns_with_the_bell_toll_for_18_fp() {
    let r = run("m5_art_toll", "m5_art_toll");
    assert_eq!(r.fp_paid(PRESS), 18.0);
    // The shockwave is on frames 24 to 28, which are ticks 34 to 38.
    assert!(r.only_hits_from("amv_bell_toll"));
    assert!(r.player_hits().iter().all(|(t, _)| (34..=38).contains(t)));
}
