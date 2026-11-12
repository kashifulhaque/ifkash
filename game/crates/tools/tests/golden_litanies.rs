//! Assertions about the WP-056 (the Litanies) goldens in `game/tests/replays/`. The golden
//! test in `golden.rs` replays every `.lfr` file against its recorded hashes; each test here
//! runs one replay and checks the Litany did what its script says. Where a rule is a number
//! in the data, the test also runs the same replay on a copy of the content with that number
//! changed, to show the number is what made the difference. The rules have unit tests in
//! `game/crates/core/src/combat/spells/tests.rs`.

use std::path::PathBuf;

use lf_content::Content;
use lf_content::moves::StatusKind;
use lf_core::arena::Id;
use lf_core::combat::status;
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

/// The world after a replay, every event it raised, and the per-tick readings the tests need.
struct Run {
    world: World,
    /// Every event, with the tick it came on.
    events: Vec<(u64, Event)>,
    /// `(tick, player hp)` after each tick.
    hp: Vec<(u64, u32)>,
    /// `(tick, HP of each Echo)` after each tick, in actor order.
    echo_hp: Vec<(u64, Vec<u32>)>,
    /// `(tick, the player's Rupture buildup meter)` after each tick.
    rupture: Vec<(u64, u32)>,
}

/// Runs the replay `name` (commands included) on `content`, checking it is on `scenario`.
/// The recorded hashes are not checked here, because a control run changes the content on
/// purpose; `golden.rs` checks the hashes of the unchanged content.
fn run_with(content: &Content, name: &str, scenario: &str) -> Run {
    let replay =
        Replay::decode(&std::fs::read(replays_dir().join(format!("{name}.lfr"))).unwrap()).unwrap();
    assert_eq!(replay.header.scenario, scenario);
    let mut world = World::new(replay.header.seed, content, scenario).unwrap();
    let mut events = Vec::new();
    let mut hp = Vec::new();
    let mut echo_hp = Vec::new();
    let mut rupture = Vec::new();
    let rupture_index = status::index(StatusKind::Rupture);
    for frame in expand(&replay.frames) {
        let commands = replay
            .commands
            .iter()
            .filter(|(t, _)| *t == world.tick)
            .map(|(_, c)| c.clone())
            .collect();
        tick(&mut world, &TickInput { frame, commands });
        let t = world.tick;
        let player = world.actors.get(world.player);
        hp.push((t, player.map_or(0, |p| p.hp)));
        rupture.push((t, player.map_or(0, |p| p.status.meter_milli[rupture_index])));
        echo_hp.push((
            t,
            world
                .actors
                .iter()
                .filter(|(_, a)| a.is_ally())
                .map(|(_, a)| a.hp)
                .collect(),
        ));
        for e in world.events.drain() {
            events.push((t, e));
        }
    }
    Run {
        world,
        events,
        hp,
        echo_hp,
        rupture,
    }
}

fn run(name: &str, scenario: &str) -> Run {
    run_with(&content(), name, scenario)
}

/// A copy of the content with `edit` applied, for a control run.
fn edited(edit: impl FnOnce(&mut Content)) -> Content {
    let mut c = content();
    edit(&mut c);
    c
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

    /// The ticks on which a projectile of `kind` came.
    fn projectiles(&self, kind: ProjectileEventKind) -> Vec<u64> {
        self.events
            .iter()
            .filter_map(|(t, e)| match e {
                Event::Projectile(p) if p.kind == kind => Some(*t),
                _ => None,
            })
            .collect()
    }

    /// The distinct actors the player hit, sorted.
    fn targets(&self) -> Vec<Id> {
        let mut t: Vec<Id> = self.player_hits().iter().map(|(_, h)| h.target).collect();
        t.sort();
        t.dedup();
        t
    }

    /// The damage of each of the player's hits, in order.
    fn damage(&self) -> Vec<u32> {
        self.player_hits().iter().map(|(_, h)| h.damage).collect()
    }

    /// True if every player hit was the move `id`.
    fn all_hits_are_move(&self, id: &str) -> bool {
        let n = content().registry.num(id).unwrap();
        self.player_hits().iter().all(|(_, h)| h.move_num == n)
    }

    /// The dummy standing at `(x, z)`: the dummies stand where they are put.
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

    /// The Echo's max HP, after the run.
    fn echo_max_hp(&self) -> u32 {
        self.world
            .actors
            .iter()
            .find(|(_, a)| a.is_ally())
            .map(|(_, a)| a.max_hp)
            .expect("an Echo")
    }
}

/// The Staggers that landed on the player.
fn player_staggers(r: &Run) -> usize {
    r.events
        .iter()
        .filter(|(_, e)| matches!(e, Event::Stagger(s) if s.target == r.world.player))
        .count()
}

// Positions in `litany_range`: dummy_front (0, 7), which stands in the pool; dummy_near
// (1.3, 2.3); dummy_far (0, 14); dummy_left (3, 8); dummy_right (-3, 8).

#[test]
fn m5_litany_mend_heals_35_percent_of_max_hp_once() {
    let r = run("m5_litany_mend", "litany_ward");
    let max = r.world.actors.get(r.world.player).unwrap().max_hp;
    // The Husk's three hits (62, 79, 62) take the player to 367, 203 below max. The heal is
    // 35 percent of max (199), so it is not capped and lands in one tick.
    let rises: Vec<(u64, u32, u32)> =
        r.hp.windows(2)
            .filter(|w| w[1].1 > w[0].1)
            .map(|w| (w[1].0, w[0].1, w[1].1))
            .collect();
    assert_eq!(rises.len(), 1, "one heal: {rises:?}");
    let (_, before, after) = rises[0];
    assert_eq!(before, 367);
    assert_eq!(after - before, max * 35 / 100);
}

#[test]
fn m5_litany_wide_mend_heals_the_echo_within_its_radius_by_30_percent() {
    let r = run("m5_litany_wide_mend", "litany_echo");
    // One Echo is in the arena. Its HP rises once: the heal. Its hits only lower it.
    let echo: Vec<(u64, u32)> = r
        .echo_hp
        .iter()
        .filter_map(|(t, hps)| hps.first().map(|h| (*t, *h)))
        .collect();
    let rises: Vec<(u64, u32, u32)> = echo
        .windows(2)
        .filter(|w| w[1].1 > w[0].1)
        .map(|w| (w[1].0, w[0].1, w[1].1))
        .collect();
    assert_eq!(rises.len(), 1, "one heal to the Echo: {rises:?}");
    let (_, before, after) = rises[0];
    // 30 percent of the Echo's own max (700): 210, and not capped.
    let max = r.echo_max_hp();
    assert_eq!(max, 700);
    assert_eq!(after - before, max * 30 / 100);
    assert!(after <= max);
    // The Husks fight the Echo, not the player: the caster is at full HP the whole run, so
    // this golden shows the heal on the Echo only.
    assert!(r.hits_taken().is_empty(), "the Husks never hit the player");
    assert!(r.hp.iter().all(|(_, hp)| *hp == 570));
}

#[test]
fn m5_litany_ember_toss_arcs_its_fire_onto_the_dummy_ahead() {
    let r = run("m5_litany_ember_toss", "litany_range");
    assert_eq!(r.projectiles(ProjectileEventKind::Spawned).len(), 1);
    assert_eq!(r.targets(), vec![r.at(0.0, 7.0)]);
    assert_eq!(r.damage(), vec![81]);
    assert!(r.all_hits_are_move("spm_ember_toss"));
}

#[test]
fn m5_litany_pyre_breath_breathes_one_flame_every_ten_ticks_while_held() {
    let r = run("m5_litany_pyre_breath", "litany_range");
    let spawned = r.projectiles(ProjectileEventKind::Spawned);
    // The button is held for 60 ticks: a breath at each tenth tick of the hold.
    assert_eq!(spawned.len(), 6, "six breaths: {spawned:?}");
    assert!(spawned.windows(2).all(|w| w[1] - w[0] == 10), "{spawned:?}");
    // Every breath hits the dummy ahead, once each.
    assert_eq!(r.targets(), vec![r.at(0.0, 7.0)]);
    assert_eq!(r.player_hits().len(), 6);
    assert!(r.all_hits_are_move("spm_pyre_breath"));
}

#[test]
fn m5_litany_storm_spear_hits_1_5_times_harder_in_the_pool() {
    let r = run("m5_litany_storm_spear", "litany_range");
    assert_eq!(r.targets(), vec![r.at(0.0, 7.0)], "the dummy in the pool");
    let wet = r.damage();
    assert_eq!(wet, vec![138]);
    // The same replay with the spear's water bonus set to 100 percent: the dry damage.
    let dry = run_with(
        &edited(|c| {
            for p in c
                .projectiles
                .iter_mut()
                .filter(|p| p.id == "spp_storm_spear")
            {
                p.vs_water_percent = 100;
            }
        }),
        "m5_litany_storm_spear",
        "litany_range",
    )
    .damage();
    assert_eq!(dry, vec![92]);
    assert_eq!(wet[0], dry[0] * 3 / 2, "1.5 times the dry damage");
}

#[test]
fn m5_litany_radiant_edge_adds_radiance_to_the_weapons_hit() {
    let r = run("m5_litany_radiant_edge", "litany_range");
    assert_eq!(r.targets(), vec![r.at(0.0, 7.0)]);
    assert_eq!(r.damage(), vec![87]);
    assert!(r.all_hits_are_move("ss_light_1"));
    // The same replay with the buff's Radiance taken out: the blade alone.
    let plain = run_with(
        &edited(|c| {
            if let Some(s) = c.status.as_mut() {
                for b in s.buffs.iter_mut().filter(|b| b.id == "spb_radiant_edge") {
                    b.element = None;
                }
            }
        }),
        "m5_litany_radiant_edge",
        "litany_range",
    );
    assert!(
        plain.damage()[0] < r.damage()[0],
        "Radiance adds to the hit: {:?} vs {:?}",
        plain.damage(),
        r.damage()
    );
}

#[test]
fn m5_litany_choir_litany_bursts_five_meters_around_the_caster() {
    let r = run("m5_litany_choir_litany", "litany_range");
    // dummy_near is 2.6 m away, inside the burst; dummy_front is 7 m away, outside it.
    assert_eq!(r.targets(), vec![r.at(1.3, 2.3)]);
    assert_eq!(r.damage(), vec![153]);
    assert!(r.all_hits_are_move("spm_choir_litany"));
}

#[test]
fn m5_litany_purge_cures_rupture_and_resets_its_meter() {
    let r = run("m5_litany_purge", "litany_hounds");
    // The bites build the meter up (about 30 a bite).
    let peak = r.rupture.iter().map(|(_, m)| *m).max().unwrap();
    assert!(peak >= 20_000, "the hounds built Rupture up: {peak}");
    // Purge resets the meter in one tick. Decay is 1.5 a tick, so a drop of 15 or more in
    // one tick, to zero, is the cure and not decay.
    let cures: Vec<(u64, u32)> = r
        .rupture
        .windows(2)
        .filter(|w| w[0].1 >= 15_000 && w[1].1 == 0)
        .map(|w| (w[1].0, w[0].1))
        .collect();
    assert_eq!(cures.len(), 1, "one cure: {cures:?}");
}

#[test]
fn m5_litany_steadfast_keeps_the_husks_combo_from_staggering_the_player() {
    let r = run("m5_litany_steadfast", "litany_ward");
    // The combo lands in full (62, 79, 62), and none of it staggers the player.
    let taken: Vec<u32> = r.hits_taken().iter().map(|h| h.damage).collect();
    assert_eq!(taken, vec![62, 79, 62]);
    assert_eq!(player_staggers(&r), 0, "+50% poise holds");
    // The same replay with the buff's poise at 100 percent: the combo staggers the player.
    let plain = run_with(
        &edited(|c| {
            if let Some(s) = c.status.as_mut() {
                for b in s.buffs.iter_mut().filter(|b| b.id == "spb_steadfast") {
                    b.poise_percent = 100;
                }
            }
        }),
        "m5_litany_steadfast",
        "litany_ward",
    );
    assert!(
        player_staggers(&plain) >= 1,
        "without Steadfast the combo staggers"
    );
}
