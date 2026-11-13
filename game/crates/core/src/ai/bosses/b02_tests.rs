//! Tests for B02, the Briar Stag (WP-070), and the scripted bot that beats it.
//!
//! The bot is the proof that the fight is fair (docs/game/05-enemies-and-bosses.md, "B02 The
//! Briar Stag"): it rolls sideways out of each charge as the Stag reaches it, rolls through
//! the toss, the stomp, and the thorn burst as their hits land, and punishes the recovery
//! after a combo's last move or a tree stun. It is deterministic (no randomness, no clock)
//! and reusable: [`run_bot`] and [`run_idle`] play a whole fight from any seed.
//!
//! The goldens `m7_b02_bot_win.lfr` and `m7_b02_idle_dies.lfr` come from the bot's inputs:
//! `record_the_b02_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;

use super::b02::{self, BOSS_ID, TREE_STUN_TICKS};
use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::arena::Id;
use crate::combat::moves::{ActiveMove, MoveNum};
use crate::events::{BriarEvent, Event, LifeEvent, MechanicEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::math::dmath::{cos, sin, sqrt};
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

const SCENARIO: &str = "boss_b02";
/// The seed the goldens use.
const SEED: u64 = 70;
/// The longest fight the runners play, in ticks (about 11 minutes of game time at 30 ticks
/// per second). It only stops a stuck run.
const MAX_TICKS: u64 = 24000;
/// The middle of the arena's width (z), which the bot rolls toward so a roll never ends in a
/// hedge.
const ARENA_MID_Z: f32 = -80.0;
/// How close the Stag must be for the bot to lock on (the lock-on reaches 20 m).
const LOCK_RANGE_M: f32 = 16.0;

fn boss_body(w: &World) -> Id {
    let spawn = w.layout.bosses[0].members[0];
    w.state.ai_enemies[usize::from(spawn)]
}

/// Planar distance and the stick that points from `p` at `boss`.
fn geometry(p: &Actor, boss: &Actor) -> (f32, (i8, i8)) {
    let (dx, dz) = (boss.pos[0] - p.pos[0], boss.pos[2] - p.pos[2]);
    let dist = sqrt(dx * dx + dz * dz).max(0.001);
    let s = |v: f32| (v / dist * 127.0) as i8;
    (dist, (s(dx), s(dz)))
}

/// What the bot sees of the boss's move this tick.
struct Watch {
    frame: u16,
    first: Option<u16>,
    last: Option<u16>,
    /// The move has `stag_charge`: the frame it runs from.
    charge_from: Option<u16>,
    /// The move ends its combo.
    finisher: bool,
    left: u16,
}

/// The briar clumps in `boss_b02` (x, z): the trees the bot baits charges into.
const CLUMPS: [(f32, f32); 5] = [
    (130.0, -70.0),
    (150.0, -92.0),
    (148.0, -66.0),
    (130.0, -94.0),
    (162.0, -80.0),
];
/// Every this many boss moves, the bot backs off behind a clump and waits for a charge.
const KITE_EVERY: u32 = 3;
/// How long the bot waits for a charge behind a clump, in ticks, before it gives up.
const KITE_TICKS: u16 = 420;
/// How far behind a clump's center the bot stands, in meters.
const BAIT_BEHIND_M: f32 = 2.4;

/// The scripted player: rolls out of hits, punishes the recovery, and every few moves baits a
/// charge into a clump.
pub(crate) struct Bot {
    /// How many ticks before a hit's first frame the bot presses DODGE (the light roll's
    /// i-frames are its frames 2 to 14).
    lead: u16,
    /// How close the Stag must be before the bot rolls out of a charge, in meters.
    charge_dist: f32,
    /// The boss move last seen, to tell when a new move starts.
    seen: Option<(MoveNum, u16)>,
    /// The bot has already rolled for the boss move it is watching.
    rolled: bool,
    /// Buttons sent last tick, because every button is a press edge.
    last: u16,
    /// Boss moves that have ended.
    cycle: u32,
    /// The boss was in a move last tick, and whether it was a charge.
    was: Option<bool>,
    /// Ticks spent baiting a charge, while doing so.
    kite: Option<u16>,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            lead: 7,
            charge_dist: 6.0,
            seen: None,
            rolled: false,
            last: 0,
            cycle: 0,
            was: None,
            // The fight opens with a bait: the Stag is far away, so it charges at once.
            kite: Some(0),
        }
    }
}

fn stick(dx: f32, dz: f32) -> (i8, i8) {
    let d = sqrt(dx * dx + dz * dz).max(0.001);
    ((dx / d * 127.0) as i8, (dz / d * 127.0) as i8)
}

/// The unit normal to `(x, z)` that points toward the middle of the arena from `z0`.
fn side_of(x: f32, z: f32, z0: f32) -> (f32, f32) {
    let side = (z, -x);
    if side.1 * (ARENA_MID_Z - z0) >= 0.0 {
        side
    } else {
        (-side.0, -side.1)
    }
}

impl Bot {
    /// The input for the next tick, from the world after the last one.
    pub(crate) fn act(&mut self, w: &World) -> ActionFrame {
        let (stick, mut pressed) = self.decide(w);
        pressed &= !self.last;
        self.last = pressed;
        ActionFrame {
            move_x: stick.0,
            move_z: stick.1,
            buttons: pressed,
        }
    }

    fn watch(&mut self, w: &World, boss: &Actor) -> Option<Watch> {
        let m = boss.moves.current?;
        let def = w.moves.get(m.num)?;
        let first = def.data.hitboxes.iter().map(|h| h.frames.0).min();
        let last = def.data.hitboxes.iter().map(|h| h.frames.1).max();
        let charge_from = def
            .data
            .vfx
            .iter()
            .find(|(_, n)| n == b02::VFX_CHARGE)
            .map(|&(f, _)| f);
        let key = (m.num, m.frame);
        if self.seen.is_none_or(|(n, f)| n != key.0 || f > key.1) {
            self.rolled = false;
        }
        self.seen = Some(key);
        Some(Watch {
            frame: m.frame,
            first,
            last,
            charge_from,
            finisher: def.next_chain.is_none(),
            left: def.data.total_frames.saturating_sub(m.frame),
        })
    }

    /// Where to stand to bait a charge: behind the clump nearest the player that is far from
    /// the Stag, as seen from the Stag.
    fn bait(player: &Actor, boss: &Actor) -> (f32, f32) {
        let far = |c: &&(f32, f32)| {
            let (dx, dz) = (c.0 - boss.pos[0], c.1 - boss.pos[2]);
            sqrt(dx * dx + dz * dz) >= 9.0
        };
        let near = |c: &&(f32, f32)| {
            let (dx, dz) = (c.0 - player.pos[0], c.1 - player.pos[2]);
            dx * dx + dz * dz
        };
        let c = CLUMPS
            .iter()
            .filter(far)
            .min_by(|a, b| near(a).total_cmp(&near(b)))
            .copied()
            .unwrap_or(CLUMPS[0]);
        let (ux, uz) = stick(c.0 - boss.pos[0], c.1 - boss.pos[2]);
        (
            c.0 + f32::from(ux) / 127.0 * BAIT_BEHIND_M,
            c.1 + f32::from(uz) / 127.0 * BAIT_BEHIND_M,
        )
    }

    fn decide(&mut self, w: &World) -> ((i8, i8), u16) {
        let Some(player) = w.actors.get(w.player) else {
            return ((0, 0), 0);
        };
        let Some(boss) = w.actors.get(boss_body(w)) else {
            return ((0, 0), 0);
        };
        let (dist, toward) = geometry(player, boss);
        match w.bosses[0].stage {
            Stage::Dormant => return ((127, 0), 0), // walk through the fog gate (it opens east)
            // The gate seals behind the player, but the arena is theirs during the intro: the
            // bot spends it walking to its bait.
            Stage::Intro { .. } if self.kite.is_some() => {
                let (bx, bz) = Self::bait(player, boss);
                return (stick(bx - player.pos[0], bz - player.pos[2]), 0);
            }
            Stage::Intro { .. } | Stage::Defeated => return ((0, 0), 0),
            Stage::Fight => {}
        }
        if player.dead || player.hp == 0 {
            return ((0, 0), 0);
        }
        if player.lock_target.is_none() {
            // Lock-on reaches 20 m; the arena is longer, so walk toward the Stag first.
            return if dist > LOCK_RANGE_M {
                match self.kite {
                    Some(_) => {
                        let (bx, bz) = Self::bait(player, boss);
                        (stick(bx - player.pos[0], bz - player.pos[2]), 0)
                    }
                    None => (toward, 0),
                }
            } else {
                // Step toward it too: the lock-on takes a target in front of the player.
                (toward, buttons::LOCK)
            };
        }
        let watch = self.watch(w, boss);
        if watch.is_none() {
            self.seen = None;
            self.rolled = false;
        }
        // Count the Stag's moves; every few, bait a charge. A finished charge ends the bait.
        let now = watch.as_ref().map(|m| m.charge_from.is_some());
        if let (Some(was_charge), None) = (self.was, now) {
            self.cycle += 1;
            if was_charge {
                self.kite = None;
            } else if self.cycle.is_multiple_of(KITE_EVERY) && self.kite.is_none() {
                self.kite = Some(0);
            }
        }
        self.was = now;
        if let Some(t) = self.kite.as_mut() {
            *t += 1;
            if *t > KITE_TICKS || boss.poise.kneel > 0 {
                self.kite = None;
            }
        }
        if player
            .moves
            .current
            .and_then(|c| w.moves.player_moveset.as_ref().map(|s| (c.num, s)))
            .is_some_and(|(n, s)| n == s.roll_light || n == s.roll_medium || n == s.roll_heavy)
        {
            self.rolled = true;
        }

        if let Some(m) = &watch {
            // 1a. A charge: wait until it is nearly on us, then roll sideways out of its line.
            if let Some(from) = m.charge_from
                && !self.rolled
                && m.frame >= from.saturating_sub(2)
                && m.last.is_some_and(|l| m.frame <= l)
                && dist < self.charge_dist
            {
                let pick = side_of(sin(boss.yaw), cos(boss.yaw), player.pos[2]);
                return (
                    ((pick.0 * 127.0) as i8, (pick.1 * 127.0) as i8),
                    buttons::DODGE,
                );
            }
            // 1b. Any other hit about to land: roll through it.
            if m.charge_from.is_none()
                && let Some(first) = m.first
                && !self.rolled
                && m.frame < first
                && first - m.frame <= self.lead
            {
                return (toward, buttons::DODGE);
            }
            // 2. A long window after the last hit of a move: heal if low, else punish.
            if let Some(last) = m.last
                && m.finisher
                && m.frame > last
                && m.left >= 14
                && m.charge_from.is_none()
            {
                return self.punish(player, dist, toward, m.left);
            }
            return ((0, 0), 0);
        }
        if boss.poise.kneel > 0 {
            // 3. A stunned Stag: close in, round whatever it ran into, and hit it.
            let eye = [player.pos[0], player.pos[1] + 0.9, player.pos[2]];
            let dir = [
                f32::from(toward.0) / 127.0,
                0.0,
                f32::from(toward.1) / 127.0,
            ];
            if w.physics.raycast(eye, dir, (dist - 1.0).max(0.1)).is_some() {
                let pick = side_of(dir[0], dir[2], player.pos[2]);
                return (stick(pick.0 * 2.4 + dir[0], pick.1 * 2.4 + dir[2]), 0);
            }
            return self.punish(player, dist, toward, boss.poise.kneel.into());
        }
        if self.kite.is_some() {
            // 4. Baiting: stand behind a clump, and wait for the charge.
            let (bx, bz) = Self::bait(player, boss);
            let (dx, dz) = (bx - player.pos[0], bz - player.pos[2]);
            if dx * dx + dz * dz > 0.8 * 0.8 {
                return (stick(dx, dz), 0);
            }
            return ((0, 0), 0);
        }
        // 5. Between attacks: stay close enough that the Stag starts its next one.
        if dist > 3.0 {
            return (toward, 0);
        }
        ((0, 0), 0)
    }

    fn punish(&self, player: &Actor, dist: f32, toward: (i8, i8), left: u16) -> ((i8, i8), u16) {
        let low = player.hp * 100 < player.max_hp * 45;
        let flasks = player.inventory.flasks.vigil > 0;
        if low && flasks && left >= 30 {
            return ((0, 0), buttons::USE_ITEM);
        }
        if dist > 2.0 {
            return (toward, 0);
        }
        if player.stamina >= 30.0 {
            return ((0, 0), buttons::LIGHT);
        }
        ((0, 0), 0)
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

    fn mechanics(&self, kind: u8) -> usize {
        self.events
            .iter()
            .filter(|e| matches!(e, Event::Mechanic(m) if m.kind == kind))
            .count()
    }

    fn phases(&self) -> Vec<u8> {
        self.events
            .iter()
            .filter_map(|e| match e {
                Event::Boss(b) if b.kind == crate::events::BossEvent::PHASE_STARTED => {
                    Some(b.phase)
                }
                _ => None,
            })
            .collect()
    }
}

fn world(seed: u64) -> World {
    let content = Content::load_embedded().unwrap();
    World::new(seed, &content, SCENARIO).unwrap()
}

fn play(seed: u64, input: impl FnMut(&World) -> ActionFrame) -> Fight {
    play_from(world(seed), input)
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
        move_x: if w.bosses[0].stage == Stage::Dormant {
            127
        } else {
            0
        },
        move_z: 0,
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

// ---------------------------------------------------------------------------------------
// The fight

#[test]
fn the_scripted_bot_beats_b02_by_rolling_and_punishing() {
    let f = run_bot(SEED);
    assert!(
        f.boss_defeated(),
        "the boss stands at tick {}",
        f.world.tick
    );
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    // The fight went through both phases.
    assert_eq!(f.phases(), [2]);
}

#[test]
fn a_player_who_does_nothing_dies_to_b02() {
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
fn the_bot_golden_replay_beats_b02_with_the_player_alive() {
    let f = play_golden("m7_b02_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    // The reward paid out: Glimmer, the Briar Axe, three Bloodbloom Oil, and the codex flag.
    let p = f.world.actors.get(f.world.player).unwrap();
    assert_eq!(p.inventory.glimmer, 1000);
    let content = Content::load_embedded().unwrap();
    let num = |id: &str| content.registry.num(id).unwrap();
    assert_eq!(p.inventory.count(num("itm_bloodbloom_oil")), 3);
    assert!(f.world.state.flags.is_set("codex.b02_briar_stag"));
    // The golden shows the Stag's tree stun.
    assert!(f.mechanics(MechanicEvent::BRIAR_TREE_STRUCK) >= 1);
    // And the patches of the second phase.
    assert!(
        f.events
            .iter()
            .any(|e| matches!(e, Event::Briar(b) if b.what == BriarEvent::SPAWNED))
    );
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b02_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

// ---------------------------------------------------------------------------------------
// The rules

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.motor.ground_y = pos[1];
    p.motor.fall_peak_y = pos[1];
}

fn step(w: &mut World, x: i8, z: i8, n: u32) -> Vec<Event> {
    let input = TickInput {
        frame: ActionFrame {
            move_x: x,
            move_z: z,
            buttons: 0,
        },
        commands: Vec::new(),
    };
    let mut out = Vec::new();
    for _ in 0..n {
        tick(w, &input);
        out.extend(w.events.drain());
    }
    out
}

/// A world with the fight running: the player just inside the gate, the intro over.
fn fight(seed: u64) -> World {
    let mut w = world(seed);
    put_player(&mut w, [113.0, 0.0, -80.0]);
    step(&mut w, 0, 0, 100);
    assert_eq!(w.bosses[0].stage, Stage::Fight);
    // Tests that stand the player near the Stag for a while shouldn't lose them to it.
    let p = w.actors.get_mut(w.player).unwrap();
    p.hp = 1_000_000;
    p.max_hp = 1_000_000;
    w
}

/// Places the Stag and starts one of its moves at frame 0, as its think would.
fn start_move(w: &mut World, pos: [f32; 3], yaw: f32, id: &str) {
    let num = w.moves.num_of(id).unwrap();
    let d = w.moves.get(num).unwrap();
    let free_from = d.data.cancel_any_from.unwrap_or(d.data.total_frames);
    let body = boss_body(w);
    let a = w.actors.get_mut(body).unwrap();
    a.pos = pos;
    a.yaw = yaw;
    a.moves.current = Some(ActiveMove {
        num,
        frame: 0,
        charge_ticks: 0,
        hold_button: 0,
        free_from,
    });
    let brain = a.ai.as_mut().unwrap();
    brain.doing = crate::ai::Doing::Attack {
        option: 0,
        move_num: num,
    };
}

fn briar(events: &[Event], what: u8) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Event::Briar(b) if b.what == what))
        .count()
}

fn mechanics(events: &[Event], kind: u8) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Event::Mechanic(m) if m.kind == kind))
        .count()
}

const WEST: f32 = -core::f32::consts::FRAC_PI_2;
const EAST: f32 = core::f32::consts::FRAC_PI_2;

#[test]
fn a_charge_that_hits_a_tree_stuns_the_stag_for_60_ticks() {
    let mut w = fight(3);
    let body = boss_body(&w);
    // Clump A stands at (130, -70). The Stag charges west along z = -70 at the player, who
    // stands behind it.
    put_player(&mut w, [119.0, 0.0, -70.0]);
    start_move(&mut w, [141.0, 0.0, -70.0], WEST, "b02_scrape_charge");
    let mut events = Vec::new();
    let mut stunned_at = None;
    for t in 0..140u32 {
        events.extend(step(&mut w, 0, 0, 1));
        let a = w.actors.get(body).unwrap();
        if stunned_at.is_none() && a.poise.kneel > 0 {
            stunned_at = Some(t);
        }
    }
    assert_eq!(mechanics(&events, MechanicEvent::BRIAR_TREE_STRUCK), 1);
    let at = stunned_at.expect("the Stag was stunned");
    // The charge starts on frame 30 and the clump is 11 m from the Stag: it runs about 20
    // frames before it is within reach.
    assert!((30..=65).contains(&at), "stunned on tick {at}");
    // It stopped in front of the clump (the face is at x = 131), not through it.
    let a = w.actors.get(body).unwrap();
    assert!(a.pos[0] > 131.0 && a.pos[0] < 134.0, "{:?}", a.pos);
    // The stun is a kneel of 60 ticks, announced as a stance break.
    let breaks: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::StanceBreak(s) if s.target == body => Some(s.ticks),
            _ => None,
        })
        .collect();
    assert_eq!(breaks, [TREE_STUN_TICKS]);
    // The player did not take the charge.
    assert!(!w.actors.get(w.player).unwrap().dead);
}

#[test]
fn the_stun_lasts_60_ticks_and_the_stag_is_down_meanwhile() {
    let mut w = fight(3);
    let body = boss_body(&w);
    put_player(&mut w, [119.0, 0.0, -70.0]);
    start_move(&mut w, [141.0, 0.0, -70.0], WEST, "b02_scrape_charge");
    // Run to the stun.
    for _ in 0..140 {
        step(&mut w, 0, 0, 1);
        if w.actors.get(body).unwrap().poise.kneel > 0 {
            break;
        }
    }
    let kneel = w.actors.get(body).unwrap().poise.kneel;
    assert!(kneel >= TREE_STUN_TICKS - 2, "{kneel}");
    // It can't act while it kneels, and it gets up on its own after the 60 ticks.
    let mut down = 0;
    for _ in 0..kneel + 2 {
        step(&mut w, 0, 0, 1);
        let a = w.actors.get(body).unwrap();
        if a.poise.kneel == 0 {
            break;
        }
        assert!(a.moves.current.is_none());
        down += 1;
    }
    assert!(down >= u32::from(TREE_STUN_TICKS) - 4, "{down}");
    assert_eq!(w.actors.get(body).unwrap().poise.kneel, 0);
}

#[test]
fn a_charge_across_open_ground_runs_its_whole_length_and_does_not_stun() {
    let mut w = fight(4);
    let body = boss_body(&w);
    // Along z = -80 from x = 150 to the west, nothing stands in the way for 18 m.
    put_player(&mut w, [114.0, 0.0, -80.0]);
    start_move(&mut w, [150.0, 0.0, -80.0], WEST, "b02_scrape_charge");
    let mut events = Vec::new();
    for _ in 0..80 {
        events.extend(step(&mut w, 0, 0, 1));
    }
    assert_eq!(mechanics(&events, MechanicEvent::BRIAR_TREE_STRUCK), 0);
    let a = w.actors.get(body).unwrap();
    assert_eq!(a.poise.kneel, 0);
    // 18.2 m of root motion in all (the last key).
    assert!((150.0 - a.pos[0] - 18.2).abs() < 0.5, "{:?}", a.pos);
    // A charge that starts announces itself once.
    assert_eq!(mechanics(&events, MechanicEvent::BRIAR_CHARGE_STARTED), 1);
}

#[test]
fn a_hedge_stops_a_charge_like_a_tree() {
    let mut w = fight(4);
    let body = boss_body(&w);
    // The east hedge is at x = 172. The Stag charges east at it from 14 m away. The player is
    // far to the west, so the Stag would turn back to face them: point it east again every
    // tick (a charge fixes its line 4 ticks before it starts, so only the facing from then on
    // matters).
    put_player(&mut w, [114.0, 0.0, -100.0]);
    start_move(&mut w, [158.0, 0.0, -100.0], EAST, "b02_scrape_charge");
    let mut events = Vec::new();
    for _ in 0..90 {
        w.actors.get_mut(body).unwrap().yaw = EAST;
        events.extend(step(&mut w, 0, 0, 1));
    }
    assert_eq!(mechanics(&events, MechanicEvent::BRIAR_TREE_STRUCK), 1);
}

#[test]
fn the_thorn_burst_of_phase_2_leaves_bramble_patches_that_slow_the_player() {
    let mut w = fight(5);
    put_player(&mut w, [140.0, 0.0, -80.0]);
    start_move(&mut w, [150.0, 0.0, -80.0], WEST, "b02_thorn_burst_2");
    // Before frame 38 there is nothing; on it, six patches grow in a ring around the Stag.
    let mut events = step(&mut w, 0, 0, 37);
    assert!(w.briar_patches.is_empty());
    events.extend(step(&mut w, 0, 0, 3));
    assert_eq!(w.briar_patches.len(), 6, "{:?}", w.briar_patches);
    assert_eq!(briar(&events, BriarEvent::SPAWNED), 6);
    let ring: Vec<[f32; 3]> = w.briar_patches.iter().map(|p| p.pos).collect();
    let body_at = w.actors.get(boss_body(&w)).unwrap().pos;
    for p in &ring {
        let (dx, dz) = (p[0] - body_at[0], p[2] - body_at[2]);
        assert!((sqrt(dx * dx + dz * dz) - 2.6).abs() < 0.05);
    }
    // They are in the state hash.
    let with = w.state_hash();
    let mut other = w.clone();
    other.briar_patches.clear();
    assert_ne!(with, other.state_hash());

    // A player who walks in one moves slower than one who walks beside it.
    let patch = ring[0];
    let walk = |w: &mut World, at: [f32; 3]| {
        put_player(w, at);
        let a = w.actors.get_mut(w.player).unwrap();
        a.moves.interrupt();
        let before = a.pos;
        step(w, 0, 127, 2);
        let after = w.actors.get(w.player).unwrap().pos;
        let (dx, dz) = (after[0] - before[0], after[2] - before[2]);
        sqrt(dx * dx + dz * dz)
    };
    // Stand the player in the patch's near half, so two ticks of walking stay inside it.
    let inside = walk(&mut w, [patch[0], 0.0, patch[2] - 1.0]);
    let outside = walk(&mut w, [patch[0] + 10.0, 0.0, patch[2] - 1.0]);
    assert!(outside > 0.0 && inside > 0.0);
    let ratio = inside / outside;
    assert!((ratio - 0.55).abs() < 0.1, "{inside} vs {outside}");
}

#[test]
fn bramble_patches_wither_after_20_seconds_and_vanish_when_the_fight_resets() {
    let mut w = fight(6);
    put_player(&mut w, [140.0, 0.0, -80.0]);
    start_move(&mut w, [150.0, 0.0, -80.0], WEST, "b02_thorn_burst_2");
    let mut events = step(&mut w, 0, 0, 40);
    assert_eq!(w.briar_patches.len(), 6);
    // After their 600 ticks, they wither (all together, as they grew together).
    events.extend(step(&mut w, 0, 0, 600));
    assert!(w.briar_patches.is_empty());
    assert_eq!(briar(&events, BriarEvent::FADED), 6);
    assert_eq!(briar(&events, BriarEvent::CLEARED), 0);

    // Grow them again; the player's death resets the fight and clears them.
    start_move(&mut w, [150.0, 0.0, -80.0], WEST, "b02_thorn_burst_2");
    put_player(&mut w, [100.0, 0.0, -80.0]);
    step(&mut w, 0, 0, 40);
    assert_eq!(w.briar_patches.len(), 6);
    w.actors.get_mut(w.player).unwrap().hp = 0;
    let events = step(&mut w, 0, 0, 3);
    assert!(w.briar_patches.is_empty());
    assert_eq!(briar(&events, BriarEvent::CLEARED), 1);
}

#[test]
fn phase_2_starts_below_half_hp_and_the_charge_then_chains_twice() {
    let mut w = fight(7);
    let body = boss_body(&w);
    put_player(&mut w, [114.0, 0.0, -80.0]);
    w.actors.get_mut(body).unwrap().hp = 1299;
    let events = step(&mut w, 0, 0, 1);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Boss(b) if b.kind == crate::events::BossEvent::PHASE_STARTED && b.phase == 2
    )));
    // The roar is the transition move.
    let roar = w.moves.num_of("b02_roar").unwrap();
    assert_eq!(
        w.actors.get(body).unwrap().moves.current.map(|c| c.num),
        Some(roar)
    );
    step(&mut w, 0, 0, 50);
    assert_eq!(w.bosses[0].phase, 2);
    // The phase 2 charge chains into `b02_charge_again` on its own.
    start_move(&mut w, [150.0, 0.0, -80.0], WEST, "b02_scrape_charge_2");
    let (first, second) = (
        w.moves.num_of("b02_scrape_charge_2").unwrap(),
        w.moves.num_of("b02_charge_again").unwrap(),
    );
    let mut seen = Vec::new();
    for _ in 0..200 {
        // Keep the player out of the way; the point is the sequence of moves.
        put_player(&mut w, [114.0, 0.0, -80.0]);
        step(&mut w, 0, 0, 1);
        let a = w.actors.get(body).unwrap();
        if let Some(c) = a.moves.current
            && seen.last() != Some(&c.num)
        {
            seen.push(c.num);
        }
    }
    assert_eq!(&seen[..2], [first, second], "{seen:?}");
}

#[test]
fn the_b02_content_passes_every_fairness_check() {
    use lf_content::validate::{Severity, Validate};
    let content = Content::load_embedded().unwrap();
    let errors: Vec<String> = content
        .validate()
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| format!("{} {} {}: {}", i.file, i.item, i.field, i.message))
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    let boss = content.boss_by_id(BOSS_ID).unwrap();
    assert_eq!(boss.body.hp, 2600);
    assert_eq!(boss.phases.len(), 1);
    assert!((boss.phases[0].below - 0.5).abs() < f32::EPSILON);
    assert!(
        boss.body
            .immune
            .contains(&lf_content::moves::StatusKind::Rupture)
    );
    assert!(boss.echo_stone);
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b02_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b02_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b02_goldens` in game/crates/core/src/ai/bosses/b02_tests.rs.\n\
             # Each line: <ticks> <move_x> <move_z> [BUTTONS].\nseed {SEED}\nscenario {SCENARIO}\n"
        );
        for (frame, n) in crate::replay::compress(&f.frames) {
            let mut names = Vec::new();
            for (mask, name) in [
                (buttons::LIGHT, "LIGHT"),
                (buttons::DODGE, "DODGE"),
                (buttons::USE_ITEM, "USE_ITEM"),
                (buttons::LOCK, "LOCK"),
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
        "m7_b02_bot_win",
        "WP-070 golden: the scripted bot beats B02, baiting a charge into a clump and rolling out of the rest.",
        &run_bot(SEED),
    );
    write(
        "m7_b02_idle_dies",
        "WP-070 golden: a player who walks in and does nothing dies to B02.",
        &run_idle(SEED),
    );
}
