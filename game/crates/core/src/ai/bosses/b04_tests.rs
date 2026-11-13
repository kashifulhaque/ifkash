//! Tests for B04, the Mossgut Brood (WP-072), and the scripted bot that beats it.
//!
//! The bot is the proof that the fight is fair (docs/game/05-enemies-and-bosses.md, "B04
//! Mossgut Brood"). It does what the spec's mechanic asks: it walks in through the fog gate,
//! destroys the egg sacs one by one (cutting down any Crawler that gets close), then goes for
//! the mother, whose weak point is open. It rolls through each hit that is about to land (the
//! tendril, the Crawlers' bites, the glob when it lands, the phase 2 spray) and drinks a flask
//! in a long window when low. It is deterministic (no randomness, no clock): [`run_bot`] and
//! [`run_idle`] play a whole fight from any seed.
//!
//! The goldens `m7_b04_bot_win.lfr` and `m7_b04_idle_dies.lfr` come from these inputs:
//! `record_the_b04_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;

use crate::actor::Actor;
use crate::ai::perception::dist_xz;
use crate::arena::Id;
use crate::events::{BossEvent, Event, LifeEvent, MechanicEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;
use lf_content::scenario::DamageType;

use super::b04::{self, BOSS_ID};
use crate::ai::boss::Stage;

const SCENARIO: &str = "boss_b04";
/// The seed the goldens use.
const SEED: u64 = 72;
/// The longest fight the runners play, in ticks. Only stops a stuck run.
const MAX_TICKS: u64 = 12000;

/// A world with the boss's body and sacs found.
fn body(w: &World) -> Id {
    w.state.ai_enemies[usize::from(w.layout.bosses[0].members[0])]
}

fn sacs(w: &World) -> Vec<Id> {
    w.layout.bosses[0]
        .members
        .iter()
        .skip(1)
        .map(|&s| w.state.ai_enemies[usize::from(s)])
        .collect()
}

fn down(a: &Actor) -> bool {
    a.dead || a.hp == 0
}

/// The planar distance, and the stick (-127 to 127) that points from `from` to `to`.
fn toward(from: [f32; 3], to: [f32; 3]) -> (f32, (i8, i8)) {
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    let dist = sqrt(dx * dx + dz * dz).max(0.001);
    let s = |v: f32| (v / dist * 127.0) as i8;
    (dist, (s(dx), s(dz)))
}

/// The scripted player.
pub(crate) struct Bot {
    /// How many ticks before a hit's first frame the bot presses DODGE. The light roll's
    /// i-frames are its frames 2 to 14, so this centers them on a short hit window.
    lead: u16,
    /// Ticks until the bot may roll again.
    roll_cooldown: u16,
    /// Buttons sent last tick, because every button is a press edge.
    last: u16,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            lead: 7,
            roll_cooldown: 0,
            last: 0,
        }
    }
}

impl Bot {
    /// The input for the next tick, from the world after the last one.
    pub(crate) fn act(&mut self, w: &World) -> ActionFrame {
        let (stick, mut pressed) = self.decide(w);
        // A button only counts on its press edge: let go for a tick between presses.
        pressed &= !self.last;
        self.last = pressed;
        ActionFrame {
            move_x: stick.0,
            move_z: stick.1,
            buttons: pressed,
        }
    }

    /// A hit that is about to land on the player at `at`: the direction of the thing that
    /// will hit, if there is one.
    fn imminent(&self, w: &World, at: [f32; 3]) -> Option<(i8, i8)> {
        let boss_body = body(w);
        for (id, a) in w.actors.iter() {
            let Some(m) = a.moves.current.filter(|_| !a.dead) else {
                continue;
            };
            if a.ai.is_none() || a.is_ally() {
                continue;
            }
            let Some(def) = w.moves.get(m.num) else {
                continue;
            };
            // The mother's cone and tendrils reach far; a Crawler's bite doesn't.
            let reach = if id == boss_body { 9.0 } else { 2.6 };
            let first = def.data.hitboxes.iter().map(|h| h.frames.0).min();
            if let Some(first) = first
                && m.frame < first
                && first - m.frame <= self.lead
                && dist_xz(a.pos, at) < reach
            {
                return Some(toward(at, a.pos).1);
            }
        }
        // A glob about to land.
        for (_, p) in w.projectiles.iter() {
            if dist_xz(p.pos, at) < 4.5 && p.pos[1] < 3.0 {
                return Some(toward(at, p.pos).1);
            }
        }
        None
    }

    fn decide(&mut self, w: &World) -> ((i8, i8), u16) {
        self.roll_cooldown = self.roll_cooldown.saturating_sub(1);
        let Some(player) = w.actors.get(w.player) else {
            return ((0, 0), 0);
        };
        match w.bosses[0].stage {
            Stage::Dormant => return ((0, 127), 0), // walk through the fog gate
            Stage::Intro { .. } | Stage::Defeated => return ((0, 0), 0),
            Stage::Fight => {}
        }
        if down(player) {
            return ((0, 0), 0);
        }
        let Some(mother) = w.actors.get(body(w)) else {
            return ((0, 0), 0);
        };

        // 1. Roll through a hit that is about to land.
        if self.roll_cooldown == 0
            && let Some(dir) = self.imminent(w, player.pos)
        {
            self.roll_cooldown = 26;
            return (dir, buttons::DODGE);
        }

        // 2. In a long quiet window, with nothing close, drink a flask if low.
        let quiet = mother.moves.current.is_none_or(|m| {
            w.moves
                .get(m.num)
                .is_some_and(|d| d.data.total_frames.saturating_sub(m.frame) >= 34)
        });
        let near_crawler = w.actors.iter().any(|(_, a)| {
            a.ai.as_ref()
                .is_some_and(|b| b.def != mother.ai.as_ref().map_or(0, |m| m.def))
                && !a.is_ally()
                && !a.dead
                && dist_xz(a.pos, player.pos) < 5.0
        });
        let mother_hit_soon = mother.moves.current.is_some();
        if player.hp * 100 < player.max_hp * 45
            && player.inventory.flasks.vigil > 0
            && quiet
            && !near_crawler
            && !mother_hit_soon
            && dist_xz(mother.pos, player.pos) > 6.0
        {
            return ((0, 0), buttons::USE_ITEM);
        }

        // 3. Cut down a Crawler within reach; else the nearest sac; else the mother.
        let crawler = w
            .actors
            .iter()
            .filter(|(id, a)| {
                !down(a)
                    && !a.is_ally()
                    && a.ai.is_some()
                    && *id != body(w)
                    && !sacs(w).contains(id)
                    && dist_xz(a.pos, player.pos) < 2.2
            })
            .min_by(|a, b| dist_xz(a.1.pos, player.pos).total_cmp(&dist_xz(b.1.pos, player.pos)))
            .map(|(_, a)| a.pos);
        let sac = sacs(w)
            .into_iter()
            .filter_map(|id| w.actors.get(id))
            .filter(|a| !down(a))
            .min_by(|a, b| dist_xz(a.pos, player.pos).total_cmp(&dist_xz(b.pos, player.pos)))
            .map(|a| a.pos);
        let (goal, reach) = match (crawler, sac) {
            (Some(c), _) => (c, 1.9),
            (None, Some(s)) => (s, 1.9),
            (None, None) => (mother.pos, 2.4),
        };
        let (dist, stick) = toward(player.pos, goal);
        if dist <= reach && player.stamina >= 25.0 {
            // Face it and swing (the stick turns the swing toward the target).
            return (stick, buttons::LIGHT);
        }
        (stick, 0)
    }
}

/// The result of a fight.
pub(crate) struct Fight {
    /// The input of every tick played.
    pub frames: Vec<ActionFrame>,
    /// The world after the last tick.
    pub world: World,
    /// Every event, in order.
    pub events: Vec<Event>,
}

impl Fight {
    pub(crate) fn boss_defeated(&self) -> bool {
        self.world.state.flags.get(&flags::boss_defeated(BOSS_ID)) > 0
    }

    pub(crate) fn player_died(&self) -> bool {
        self.events
            .iter()
            .any(|e| matches!(e, Event::Life(LifeEvent::Died { .. })))
    }

    fn player_hp(&self) -> u32 {
        self.world.actors.get(self.world.player).unwrap().hp
    }

    /// The `MechanicEvent` sub-kinds raised, in order, with their indexes.
    fn mechanics(&self) -> Vec<(u8, u16)> {
        self.events
            .iter()
            .filter_map(|e| match e {
                Event::Mechanic(MechanicEvent { kind, index }) if (130..140).contains(kind) => {
                    Some((*kind, *index))
                }
                _ => None,
            })
            .collect()
    }
}

fn play(seed: u64, input: impl FnMut(&World) -> ActionFrame) -> Fight {
    let content = Content::load_embedded().unwrap();
    play_from(World::new(seed, &content, SCENARIO).unwrap(), input)
}

fn play_from(mut w: World, mut input: impl FnMut(&World) -> ActionFrame) -> Fight {
    let mut frames = Vec::new();
    let mut events = Vec::new();
    let mut after = 0u32; // ticks played since the fight ended
    while w.tick < MAX_TICKS && after < 30 {
        let frame = input(&w);
        frames.push(frame);
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        events.extend(w.events.drain());
        let over = events
            .iter()
            .any(|e| matches!(e, Event::Life(LifeEvent::Died { .. })))
            || w.bosses[0].stage == Stage::Defeated;
        if over {
            after += 1;
        }
    }
    Fight {
        frames,
        world: w,
        events,
    }
}

/// Plays the fight with the scripted bot until the boss falls or the player dies.
pub(crate) fn run_bot(seed: u64) -> Fight {
    let mut bot = Bot::default();
    play(seed, |w| bot.act(w))
}

/// Plays the fight with a player who walks in through the gate and then does nothing.
pub(crate) fn run_idle(seed: u64) -> Fight {
    play(seed, |w| ActionFrame {
        move_x: 0,
        move_z: if w.bosses[0].stage == Stage::Dormant {
            127
        } else {
            0
        },
        buttons: 0,
    })
}

fn replay(name: &str) -> Replay {
    let path = format!(
        "{}/../../tests/replays/{name}.lfr",
        env!("CARGO_MANIFEST_DIR")
    );
    Replay::decode(&std::fs::read(path).unwrap()).unwrap()
}

/// Replays a golden, returning the world and every event.
fn play_golden(name: &str) -> Fight {
    let r = replay(name);
    assert_eq!(r.header.scenario, SCENARIO);
    let frames = expand(&r.frames);
    let mut i = 0;
    let mut fight = play(r.header.seed, |_| {
        let f = frames.get(i).copied().unwrap_or_default();
        i += 1;
        f
    });
    fight.frames.truncate(frames.len());
    fight
}

// --- Helpers for the rule tests ---------------------------------------------------------------

fn fresh(seed: u64) -> World {
    let content = Content::load_embedded().unwrap();
    World::new(seed, &content, SCENARIO).unwrap()
}

fn step(w: &mut World, frame: ActionFrame, n: u32) -> Vec<(u64, Event)> {
    let mut out = Vec::new();
    for _ in 0..n {
        tick(
            w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        let t = w.tick;
        out.extend(w.events.drain().into_iter().map(|e| (t, e)));
    }
    out
}

const IDLE: ActionFrame = ActionFrame {
    move_x: 0,
    move_z: 0,
    buttons: 0,
};

fn put_player(w: &mut World, pos: [f32; 3], hp: u32) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.motor.ground_y = pos[1];
    p.motor.fall_peak_y = pos[1];
    p.hp = hp;
    p.max_hp = hp;
}

fn fell(w: &mut World, id: Id) {
    let a = w.actors.get_mut(id).unwrap();
    a.hp = 0;
    a.dead = true;
}

/// A world with the player through the gate and the fight started. The player has `hp`.
fn in_fight(seed: u64, hp: u32) -> (World, u64) {
    let mut w = fresh(seed);
    put_player(&mut w, [0.0, 0.0, 45.0], hp);
    let events = step(&mut w, IDLE, 200);
    let started = events
        .iter()
        .find_map(|(t, e)| match e {
            Event::Boss(b) if b.kind == BossEvent::FIGHT_STARTED => Some(*t),
            _ => None,
        })
        .expect("the fight starts");
    assert_eq!(w.bosses[0].stage, Stage::Fight);
    (w, started)
}

/// The Crawlers the sacs spawned (a Crawler whose spawn index is a sac's).
fn brood_crawlers(w: &World) -> Vec<Id> {
    let num = w.ai.defs[w.ai.index_of("marsh_crawler").unwrap()].num;
    let sac_spawns = &w.layout.bosses[0].members[1..];
    w.actors
        .iter()
        .filter(|(_, a)| {
            a.ai.as_ref()
                .is_some_and(|b| b.def == num && sac_spawns.contains(&b.spawn))
        })
        .map(|(id, _)| id)
        .collect()
}

fn alive_crawlers(w: &World) -> usize {
    brood_crawlers(w)
        .into_iter()
        .filter(|&id| !down(w.actors.get(id).unwrap()))
        .count()
}

fn mech(events: &[(u64, Event)], kind: u8) -> Vec<(u64, u16)> {
    events
        .iter()
        .filter_map(|(t, e)| match e {
            Event::Mechanic(m) if m.kind == kind => Some((*t, m.index)),
            _ => None,
        })
        .collect()
}

// --- Content ----------------------------------------------------------------------------------

#[test]
fn the_content_passes_every_check_and_the_level_places_the_boss() {
    use lf_content::validate::{Severity, Validate};
    let c = Content::load_embedded().unwrap();
    let errors: Vec<_> = c
        .validate()
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| format!("{} {}: {}", i.file, i.field, i.message))
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    // Two phases, 2,400 HP, four sacs that aren't required, immune to Poison and Blight.
    let def = c.boss_by_id(BOSS_ID).unwrap();
    assert_eq!(def.phases.len(), 1);
    assert_eq!(def.body.hp, 2400);
    assert_eq!(def.parts.len(), 4);
    assert!(def.parts.iter().all(|p| !p.required));
    assert_eq!(def.body.immune.len(), 2);
    // The level places the same boss at the chamber's wall, behind the fog gate.
    let w = World::new(1, &c, "mossgut_cave").unwrap();
    let b = &w.layout.bosses[0];
    assert_eq!(b.def.id, BOSS_ID);
    assert_eq!(b.gate.pos, [0.0, 0.0, 44.0]);
}

// --- The fight ----------------------------------------------------------------------------------

#[test]
fn the_scripted_bot_beats_b04_by_destroying_the_sacs_first() {
    let f = run_bot(SEED);
    assert!(
        f.boss_defeated(),
        "the boss stands at tick {}",
        f.world.tick
    );
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    // Four sacs burst, and the weak point opens right after the last of them.
    let m = f.mechanics();
    let mut bursts: Vec<u16> = m
        .iter()
        .filter(|(k, _)| *k == b04::SAC_BURST)
        .map(|(_, i)| *i)
        .collect();
    bursts.sort_unstable();
    assert_eq!(bursts, [0, 1, 2, 3]);
    let exposed: Vec<usize> = m
        .iter()
        .enumerate()
        .filter(|(_, (k, _))| *k == b04::WEAK_POINT_EXPOSED)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(exposed.len(), 1, "{m:?}");
    let before = m[..exposed[0]]
        .iter()
        .filter(|(k, _)| *k == b04::SAC_BURST)
        .count();
    assert_eq!(before, 4, "{m:?}");
    // Phase 2 began once, and the fight ended with the defeat.
    let phases: Vec<u8> = f
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Boss(b) if b.kind == BossEvent::PHASE_STARTED => Some(b.phase),
            _ => None,
        })
        .collect();
    assert_eq!(phases, [2]);
}

#[test]
fn a_player_who_does_nothing_dies_to_b04() {
    let f = run_idle(SEED);
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

#[test]
fn the_bot_wins_from_other_seeds_too() {
    for seed in [1, 7, 99] {
        let f = run_bot(seed);
        assert!(f.boss_defeated() && !f.player_died(), "seed {seed}");
    }
}

#[test]
fn the_bot_golden_replay_beats_b04_with_the_player_alive() {
    let f = play_golden("m7_b04_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    // The Glimmer paid out, and the codex entry unlocked.
    let p = f.world.actors.get(f.world.player).unwrap();
    assert_eq!(p.inventory.glimmer, 900);
    assert!(f.world.state.flags.get(&flags::codex("b04_mossgut_brood")) > 0);
    // The weak point opened on the way.
    assert!(
        f.mechanics()
            .iter()
            .any(|(k, _)| *k == b04::WEAK_POINT_EXPOSED)
    );
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b04_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

// --- The egg sacs and the Crawlers --------------------------------------------------------------

#[test]
fn each_sac_spawns_a_crawler_every_300_ticks_up_to_the_cap() {
    // A player who can't die, standing in the passage doing nothing.
    let (mut w, started) = in_fight(5, 1_000_000);
    let mut events = Vec::new();
    let mut peak = 0;
    for _ in 0..1000 {
        events.extend(step(&mut w, IDLE, 1));
        peak = peak.max(alive_crawlers(&w));
    }
    let at: Vec<(u64, u16)> = mech(&events, b04::CRAWLER_SPAWNED)
        .iter()
        .map(|&(t, i)| (t - started, i))
        .collect();
    // Sac 0 at 300 ticks, then the staggered sacs 75 ticks apart; sac 0 again at 600, sac 1
    // at 675. That is six alive, the cap, so the next one waits.
    let want: Vec<(u64, u16)> = [(0, 0), (1, 1), (2, 2), (3, 3), (4, 0), (5, 1)]
        .iter()
        .map(|&(k, i)| (300 + 75 * k, i))
        .collect();
    assert_eq!(at.len(), want.len(), "{at:?}");
    // The event is reported on the tick after the one the sac's clock reads.
    for (got, want) in at.iter().zip(&want) {
        assert!(got.0.abs_diff(want.0) <= 1 && got.1 == want.1, "{at:?}");
    }
    assert_eq!(peak, 6);
}

#[test]
fn a_destroyed_sac_stops_spawning_and_announces_itself_once() {
    let (mut w, _) = in_fight(5, 1_000_000);
    let sac = sacs(&w)[0];
    fell(&mut w, sac);
    let events = step(&mut w, IDLE, 700);
    assert_eq!(
        mech(&events, b04::SAC_BURST),
        [(mech(&events, b04::SAC_BURST)[0].0, 0)]
    );
    // Sac 0 never spawns; the other three do.
    let by_sac: Vec<u16> = mech(&events, b04::CRAWLER_SPAWNED)
        .iter()
        .map(|&(_, i)| i)
        .collect();
    assert!(!by_sac.contains(&0), "{by_sac:?}");
    assert!(by_sac.contains(&1) && by_sac.contains(&2) && by_sac.contains(&3));
    // Three sacs left: the weak point is still shut.
    assert!(mech(&events, b04::WEAK_POINT_EXPOSED).is_empty());
}

#[test]
fn the_crawlers_go_when_the_player_dies_and_when_the_boss_falls() {
    // The player dies: the fight resets and the Crawlers are removed with it.
    let f = run_idle(SEED);
    assert!(f.player_died());
    let w = &f.world;
    assert_eq!(w.bosses[0].stage, Stage::Dormant);
    assert!(brood_crawlers(w).is_empty());
    let mother = w.actors.get(body(w)).unwrap();
    assert_eq!(mother.hp, mother.max_hp);
    assert!(sacs(w).iter().all(|&id| !down(w.actors.get(id).unwrap())));

    // The boss falls: the Crawlers fall with her.
    let (mut w, _) = in_fight(5, 1_000_000);
    step(&mut w, IDLE, 400);
    assert!(alive_crawlers(&w) > 0);
    let m = body(&w);
    fell(&mut w, m);
    step(&mut w, IDLE, 3);
    assert_eq!(w.bosses[0].stage, Stage::Defeated);
    assert_eq!(alive_crawlers(&w), 0);
}

// --- The weak point ---------------------------------------------------------------------------

/// The damage of the first light attack that lands on the mother other than as a counter hit
/// (a counter hit is a hit on an enemy in its own attack, which adds its own bonus), from a
/// player standing at her chest. Tries again a little later if she was mid-attack.
fn first_hit_on_mother(w: &mut World) -> u32 {
    let m = body(w);
    let swing = ActionFrame {
        move_x: 0,
        move_z: 127,
        buttons: buttons::LIGHT,
    };
    for _ in 0..12 {
        put_player(w, [0.0, 0.0, 82.0], 1_000_000);
        let mut events = step(w, swing, 1);
        events.extend(step(
            w,
            ActionFrame {
                buttons: 0,
                ..swing
            },
            45,
        ));
        if let Some(d) = events.iter().find_map(|(_, e)| match e {
            Event::Hit(h) if h.target == m && !h.counter => Some(h.damage),
            _ => None,
        }) {
            return d;
        }
        step(w, IDLE, 17);
    }
    panic!("no swing hit the mother outside her own attack");
}

#[test]
fn destroying_all_four_sacs_doubles_the_damage_the_mother_takes() {
    let shut = {
        let (mut w, _) = in_fight(9, 1_000_000);
        first_hit_on_mother(&mut w)
    };
    let (open, weakness) = {
        let mut w = fresh(9);
        for id in sacs(&w) {
            fell(&mut w, id);
        }
        put_player(&mut w, [0.0, 0.0, 45.0], 1_000_000);
        step(&mut w, IDLE, 200);
        let weakness = w
            .actors
            .get(body(&w))
            .unwrap()
            .stats
            .defense
            .weakness_percent;
        (first_hit_on_mother(&mut w), weakness)
    };
    assert!(shut > 0);
    // Doubled, to within a point of rounding.
    assert!(open.abs_diff(shut * 2) <= 1, "shut {shut}, open {open}");
    // Her Fire weakness (1.5x) doubles to 3x along with the rest.
    assert_eq!(weakness[DamageType::Fire as usize], 300);
    assert_eq!(weakness[DamageType::Blunt as usize], 200);
}

#[test]
fn three_sacs_down_leave_the_weak_point_shut() {
    let (mut w, _) = in_fight(9, 1_000_000);
    for &id in sacs(&w).iter().take(3) {
        fell(&mut w, id);
    }
    step(&mut w, IDLE, 5);
    let weakness = w
        .actors
        .get(body(&w))
        .unwrap()
        .stats
        .defense
        .weakness_percent;
    assert_eq!(weakness[DamageType::Blunt as usize], 100);
    assert_eq!(weakness[DamageType::Fire as usize], 150);
}

// --- Phase 2 ----------------------------------------------------------------------------------

#[test]
fn she_stays_in_the_wall_in_phase_1_then_tears_free_and_sprays() {
    let (mut w, _) = in_fight(11, 1_000_000);
    let spray = w.moves.num_of("b04_spray").unwrap();
    put_player(&mut w, [0.0, 0.0, 66.0], 1_000_000);
    let m = body(&w);
    let home = w.actors.get(m).unwrap().pos;
    // Phase 1: 500 ticks, and she hasn't moved a step or sprayed.
    let mut sprayed = false;
    for _ in 0..500 {
        step(&mut w, IDLE, 1);
        let a = w.actors.get(m).unwrap();
        sprayed |= a.moves.current.is_some_and(|c| c.num == spray);
        assert_eq!(a.pos, home, "she moved at tick {}", w.tick);
    }
    assert!(!sprayed);
    assert_eq!(w.bosses[0].phase, 1);
    // Drop her under half HP: the transition starts, then phase 2.
    {
        let a = w.actors.get_mut(m).unwrap();
        a.hp = a.max_hp * 4 / 10;
    }
    let events = step(&mut w, IDLE, 1);
    assert!(events.iter().any(|(_, e)| matches!(e, Event::Boss(b)
        if b.kind == BossEvent::PHASE_STARTED && b.phase == 2)));
    let mut sprayed = false;
    for _ in 0..1500 {
        step(&mut w, IDLE, 1);
        let a = w.actors.get(m).unwrap();
        sprayed |= a.moves.current.is_some_and(|c| c.num == spray);
    }
    let a = w.actors.get(m).unwrap();
    assert_eq!(w.bosses[0].phase, 2);
    assert!(a.pos[2] < home[2] - 3.0, "she crawled to z {}", a.pos[2]);
    assert!(sprayed, "she never sprayed");
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b04_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b04_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b04_goldens` in game/crates/core/src/ai/bosses/b04_tests.rs.\n\
             # Each line: <ticks> <move_x> <move_z> [BUTTONS].\nseed {SEED}\nscenario {SCENARIO}\n"
        );
        for (frame, n) in crate::replay::compress(&f.frames) {
            let mut names = Vec::new();
            for (mask, name) in [
                (buttons::LIGHT, "LIGHT"),
                (buttons::DODGE, "DODGE"),
                (buttons::USE_ITEM, "USE_ITEM"),
            ] {
                if frame.buttons & mask != 0 {
                    names.push(name);
                }
            }
            text += &format!("{n} {} {}", frame.move_x, frame.move_z);
            if !names.is_empty() {
                text += &format!(" {}", names.join("+"));
            }
            text.push('\n');
        }
        std::fs::write(format!("{dir}/{name}.script"), text).unwrap();
    };
    write(
        "m7_b04_bot_win",
        "WP-072 golden: the scripted bot destroys B04's egg sacs, then beats the exposed mother.",
        &run_bot(SEED),
    );
    write(
        "m7_b04_idle_dies",
        "WP-072 golden: a player who walks in and does nothing dies to B04.",
        &run_idle(SEED),
    );
}
