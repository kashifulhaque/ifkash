//! Tests for B10, the Bellwright (WP-078), and the scripted bot that beats him.
//!
//! The bot plays the fight as the spec describes it (docs/game/05-enemies-and-bosses.md, "B10 The
//! Bellwright"): it fights by the beat. It stands in front of him, rolls through each hit a few
//! ticks before it lands (the hits land on beats), rolls through the bell's shockwave as it
//! reaches him, keeps out of the broken ice, and strikes in the windows between his swings. It is
//! deterministic (no randomness, no clock), and [`run_bot`] and [`run_idle`] play a whole fight
//! from any seed.
//!
//! The goldens `m7_b10_bot_win.lfr` and `m7_b10_idle_dies.lfr` come from the bot's inputs:
//! `record_the_b10_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;

use super::b10::{self, B10State, Ice};
use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::arena::Id;
use crate::combat::hitbox::{Capsule, Pose, closest_points};
use crate::combat::moves::MoveNum;
use crate::events::{BossEvent, Event, LifeEvent, MechanicEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::math::dmath::{cos, sin, sqrt};
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

const SCENARIO: &str = "boss_b10";
const BOSS_ID: &str = "b10_the_bellwright";
/// The seed the goldens use.
const SEED: u64 = 78;
/// The longest fight the runners play, in ticks (at 30 ticks a second, ten minutes). This only
/// stops a stuck run.
const MAX_TICKS: u64 = 18_000;

fn body_of(w: &World) -> Id {
    let spawn = w.layout.bosses[0].members[0];
    w.state.ai_enemies[usize::from(spawn)]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    sqrt((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2))
}

/// The stick (-127 to 127) that points from `from` to `to`, scaled to `power` (0 to 1).
fn toward(from: [f32; 3], to: [f32; 3], power: f32) -> (i8, i8) {
    let d = dist(from, to).max(0.001);
    let s = |v: f32| (v / d * 127.0 * power) as i8;
    (s(to[0] - from[0]), s(to[2] - from[2]))
}

/// The numbers the bot reads from the boss definition.
fn number(w: &World, key: &str) -> f32 {
    w.layout.bosses[0].def.params[key]
}

/// The centers of the ice patches, in world axes.
fn zones(w: &World) -> Vec<[f32; 3]> {
    let home = w.layout.enemies[usize::from(w.layout.bosses[0].members[0])].pos;
    (0..number(w, "ice_zones") as usize)
        .map(|i| {
            [
                home[0] + number(w, &format!("ice{i}_x_m")),
                home[1],
                home[2] + number(w, &format!("ice{i}_z_m")),
            ]
        })
        .collect()
}

/// The ice patch state after the world's tolls.
fn ice_after(w: &World, i: usize, tolls: u32) -> Ice {
    let cracks = number(w, "ice_first_crack_toll") as u32
        + number(w, "ice_crack_step_tolls") as u32 * i as u32;
    b10::patch(tolls, cracks, number(w, "ice_break_after_tolls") as u32)
}

/// When the bell's shockwave next reaches a player at `pp`, for a Bellwright at `body_pos`: the
/// toll it belongs to (its start tick, or the grid's next toll) and the ticks until its edge gets
/// there. The player can know this from the music (the beat) and the bell's wind-up. `None` before
/// the grid starts.
fn toll_eta(w: &World, pp: [f32; 3], body_pos: [f32; 3]) -> Option<(u64, f32)> {
    let st = w.mech.b10;
    if st.from == 0 {
        return None;
    }
    let speed = number(w, "toll_speed_m_per_tick");
    if st.toll_from != 0 {
        let edge = (w.tick - st.toll_from) as f32 * speed;
        let d = sqrt((pp[0] - st.toll_at[0]).powi(2) + (pp[2] - st.toll_at[1]).powi(2));
        return Some((st.toll_from, (d - edge) / speed));
    }
    let period = if st.grid_phase >= 2 {
        number(w, "beat_ticks_p2")
    } else {
        number(w, "beat_ticks_p1")
    } as u64;
    let cycle = period * number(w, "toll_every_beats") as u64;
    let next = st.from + ((w.tick - st.from) / cycle + 1) * cycle;
    Some((next, (next - w.tick) as f32 + dist(pp, body_pos) / speed))
}

/// The scripted player.
pub struct Bot {
    /// How many ticks before a hit's first frame the bot presses DODGE.
    lead: u16,
    /// The same for the bell's shockwave, in ticks before its edge reaches the bot.
    toll_lead: u16,
    /// How long a swing of the bot's own takes to be free to roll out of, in ticks.
    swing_ticks: u16,
    /// The frame of the hit the bot already rolled through (in the move it is rolling in).
    rolled_for: Option<(MoveNum, u16)>,
    /// The toll the bot already rolled through (its start tick).
    rolled_toll: u64,
    /// What the last DODGE press was for, kept until the roll starts (a press during a swing of
    /// the bot's own is ignored, and then it presses again).
    pending_toll: Option<u64>,
    pending_hit: Option<(MoveNum, u16)>,
    /// Buttons sent last tick, because every button is a press edge.
    last: u16,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            lead: 7,
            toll_lead: 5,
            swing_ticks: 24,
            rolled_for: None,
            rolled_toll: 0,
            pending_toll: None,
            pending_hit: None,
            last: 0,
        }
    }
}

impl Bot {
    pub fn act(&mut self, w: &World) -> ActionFrame {
        let (stick, mut pressed) = self.decide(w);
        pressed &= !self.last;
        self.last = pressed;
        ActionFrame {
            move_x: stick.0,
            move_z: stick.1,
            buttons: pressed,
        }
    }

    fn decide(&mut self, w: &World) -> ((i8, i8), u16) {
        let Some(player) = w.actors.get(w.player) else {
            return ((0, 0), 0);
        };
        match w.bosses[0].stage {
            Stage::Dormant => return ((0, 127), crate::ai::boss::fog_buttons(w)), // through the fog gate, north
            Stage::Intro { .. } | Stage::Defeated => return ((0, 0), 0),
            Stage::Fight => {}
        }
        if player.dead || player.hp == 0 {
            return ((0, 0), 0);
        }
        let Some(body) = w.actors.get(body_of(w)) else {
            return ((0, 0), 0);
        };
        let pp = player.pos;
        let rolling = player
            .moves
            .current
            .and_then(|c| w.moves.player_moveset.as_ref().map(|s| (c.num, s)))
            .is_some_and(|(n, s)| n == s.roll_light || n == s.roll_medium || n == s.roll_heavy);

        // Where the bot stands: in front of the Bellwright, toward the player.
        let fwd = [sin(body.yaw), cos(body.yaw)];
        let post = [body.pos[0] + fwd[0] * 2.3, 0.0, body.pos[2] + fwd[1] * 2.3];
        let goal = post;

        // A roll the bot is in counts for what it pressed it for.
        if rolling {
            if let Some(k) = self.pending_toll.take() {
                self.rolled_toll = k;
            }
            if let Some(h) = self.pending_hit.take() {
                self.rolled_for = Some(h);
            }
        }
        // 1. Roll through the bell's shockwave as its edge reaches the bot. Close in, the ring
        // arrives almost the tick it is released, so the bot rolls on the beat: it counts the grid
        // to the next toll (the music) and adds the ring's travel.
        let st = w.mech.b10;
        let toll = toll_eta(w, pp, body.pos);
        if !rolling
            && let Some((key, arrives)) = toll
            && key != self.rolled_toll
            && arrives > 0.0
            && arrives <= f32::from(self.toll_lead)
            && player.stamina >= 25.0
        {
            self.pending_toll = Some(key);
            return (toward(pp, goal, 1.0), buttons::DODGE);
        }
        // 2. Roll through a hit that is about to land, toward where the bot is going.
        let mut frame = 0u16;
        if let Some(m) = body.moves.current {
            frame = m.frame;
            if self.rolled_for.is_some_and(|(n, _)| n != m.num) {
                self.rolled_for = None;
            }
        } else {
            self.rolled_for = None;
        }
        let hit = threat(w, body, pp);
        if let (Some(first), Some(m)) = (hit, body.moves.current)
            && !rolling
            && self.rolled_for != Some((m.num, first))
            && frame < first
            && first - frame <= self.lead
            && player.stamina >= 25.0
        {
            self.pending_hit = Some((m.num, first));
            return (toward(pp, goal, 1.0), buttons::DODGE);
        }
        // 3. Out of the broken ice, toward the Bellwright.
        let at = zones(w);
        if (0..at.len()).any(|i| ice_after(w, i, st.tolls) == Ice::Broken && dist(pp, at[i]) < 3.9)
        {
            return (toward(pp, body.pos, 1.0), 0);
        }
        // 4. Drink a flask in a long recovery, when hurt.
        if player.hp * 100 < player.max_hp * 50
            && player.inventory.flasks.vigil > 0
            && recovery_left(w, body) >= 36
            && player.moves.current.is_none()
        {
            return ((0, 0), buttons::USE_ITEM);
        }
        // 5. Get to the post, and strike when no hit is coming.
        if dist(pp, post) > 0.6 {
            return (toward(pp, goal, 1.0), 0);
        }
        // A swing of the bot's own can't be rolled out of at once, so none starts just before the
        // toll (nor while a hit is coming).
        let toll_near = toll.is_some_and(|(_, t)| t > -4.0 && t < f32::from(self.swing_ticks));
        if hit.is_none() && !toll_near && player.stamina >= 30.0 && player.moves.current.is_none() {
            return (toward(pp, body.pos, 0.1), buttons::LIGHT);
        }
        ((0, 0), 0)
    }
}

/// Ticks left of the body's move once its last hit is over; 0 otherwise.
fn recovery_left(w: &World, body: &Actor) -> u16 {
    let Some(m) = body.moves.current else {
        return 0;
    };
    let Some(def) = w.moves.get(m.num) else {
        return 0;
    };
    let last = def.data.hitboxes.iter().map(|h| h.frames.1).max();
    if def.next_chain.is_some() || last.is_none_or(|l| m.frame <= l) {
        return 0;
    }
    def.data.total_frames.saturating_sub(m.frame)
}

/// The first frame at which the boss's current move would hit a player standing at `pp`: the
/// earliest of its hitboxes still to come that reach `pp` on the ground plane (with a margin,
/// for the body turning and the player moving). `None` if nothing would.
fn threat(w: &World, body: &Actor, pp: [f32; 3]) -> Option<u16> {
    /// Meters added to each hitbox's radius.
    const MARGIN_M: f32 = 0.9;
    let m = body.moves.current?;
    let def = w.moves.get(m.num)?;
    let pose = Pose {
        pos: body.pos,
        yaw: body.yaw,
    };
    let flat = |v: [f32; 3]| [v[0], 0.0, v[2]];
    def.data
        .hitboxes
        .iter()
        .filter(|h| h.frames.1 >= m.frame)
        .filter(|h| {
            let c = Capsule::from_shape(&h.shape, &pose);
            let (_, _, d2) = closest_points(flat(c.a), flat(c.b), flat(pp), flat(pp));
            sqrt(d2) <= c.radius + MARGIN_M
        })
        .map(|h| h.frames.0)
        .min()
}

/// The result of a fight.
pub(crate) struct Fight {
    pub frames: Vec<ActionFrame>,
    pub world: World,
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

    fn mechanic_count(&self, kind: u8) -> usize {
        self.events
            .iter()
            .filter(|e| matches!(e, Event::Mechanic(m) if m.kind == kind))
            .count()
    }

    fn boss_events(&self, kind: u8) -> usize {
        self.events
            .iter()
            .filter(|e| matches!(e, Event::Boss(b) if b.kind == kind))
            .count()
    }
}

fn play(seed: u64, input: impl FnMut(&World) -> ActionFrame) -> Fight {
    let content = Content::load_embedded().unwrap();
    play_from(World::new(seed, &content, SCENARIO).unwrap(), input)
}

fn play_from(mut w: World, mut input: impl FnMut(&World) -> ActionFrame) -> Fight {
    let mut frames = Vec::new();
    let mut events = Vec::new();
    let mut after = 0u32;
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
        buttons: crate::ai::boss::fog_buttons(w),
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

fn actor(w: &World, id: Id) -> &Actor {
    w.actors.get(id).unwrap()
}

/// A trace of the bot's fight, for tuning: `cargo test -p lf-core trace_the_bot -- --ignored
/// --nocapture`. Prints the state every 60 ticks and every hit on the player.
#[test]
#[ignore]
fn trace_the_bot() {
    let mut bot = Bot::default();
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    let (mut over, mut won) = (0, false);
    while w.tick < MAX_TICKS && over < 30 {
        if w.tick.is_multiple_of(60) {
            let p = actor(&w, w.player);
            let b = actor(&w, body_of(&w));
            eprintln!(
                "t{} {:?} ph{} player hp {} st {:.0} ({:.1},{:.1}) body hp {} mv {:?} tolls {}",
                w.tick,
                w.bosses[0].stage,
                w.bosses[0].phase,
                p.hp,
                p.stamina,
                p.pos[0],
                p.pos[2],
                b.hp,
                b.moves.current.map(|m| (m.num, m.frame)),
                w.mech.b10.tolls,
            );
        }
        let frame = bot.act(&w);
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        for e in w.events.drain() {
            match e {
                Event::Hit(h) if h.target == w.player => eprintln!(
                    "  t{} HIT on player: move {} hitbox {} dmg {} {:?}",
                    w.tick, h.move_num, h.hitbox, h.damage, h.result
                ),
                Event::Mechanic(m) if m.kind != MechanicEvent::BELL_BEAT => {
                    eprintln!("  t{} MECH {} {}", w.tick, m.kind, m.index)
                }
                Event::Boss(b) => eprintln!("  t{} BOSS {} {}", w.tick, b.kind, b.phase),
                Event::Life(LifeEvent::Died { .. }) => over = 1,
                _ => {}
            }
        }
        if w.bosses[0].stage == Stage::Defeated {
            won = true;
            over += 1;
        } else if over > 0 {
            over += 1;
        }
    }
    eprintln!("won {won} tick {}", w.tick);
}

// ---------------------------------------------------------------------------------------
// The mechanic
// ---------------------------------------------------------------------------------------

/// A world with the fight on: the player walked through the gate and the intro is over.
fn fight_world() -> World {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    while w.bosses[0].stage != Stage::Fight {
        let frame = ActionFrame {
            move_z: if w.bosses[0].stage == Stage::Dormant {
                127
            } else {
                0
            },
            buttons: crate::ai::boss::fog_buttons(&w),
            move_x: 0,
        };
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        w.events.drain();
        assert!(w.tick < 400, "the fight never started");
    }
    w
}

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.motor.ground_y = pos[1];
    p.motor.fall_peak_y = pos[1];
}

/// Ticks `ticks` times with the player's input from `input`, topping the player's HP up every
/// tick when `immortal`, and returns every event with the tick it came on.
fn run_ticks(
    w: &mut World,
    ticks: u32,
    immortal: bool,
    mut input: impl FnMut(&World) -> ActionFrame,
) -> Vec<(u64, Event)> {
    let mut out = Vec::new();
    for _ in 0..ticks {
        if immortal {
            let p = w.actors.get_mut(w.player).unwrap();
            p.hp = p.max_hp;
        }
        let frame = input(w);
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

fn mech(events: &[(u64, Event)], kind: u8) -> Vec<(u64, u16)> {
    events
        .iter()
        .filter_map(|(t, e)| match e {
            Event::Mechanic(m) if m.kind == kind => Some((*t, m.index)),
            _ => None,
        })
        .collect()
}

/// Puts the player at `pos` in every tick of a fight with an immortal player standing still.
fn stand_at(w: &mut World, ticks: u32, pos: [f32; 3]) -> Vec<(u64, Event)> {
    let mut out = Vec::new();
    for _ in 0..ticks {
        put_player(w, pos);
        out.extend(run_ticks(w, 1, true, |_| ActionFrame::default()));
    }
    out
}

#[test]
fn the_bellwright_is_a_giant_with_his_hurtbox_to_match() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    tick(&mut w, &TickInput::default());
    let body = actor(&w, body_of(&w));
    assert!(body.hurtbox.radius > 1.0 && body.hurtbox.half_height > 1.5);
    assert_eq!(body.max_hp, 12400);
}

#[test]
fn the_fight_runs_on_a_beat_of_40_ticks_with_a_toll_every_fourth() {
    let mut w = fight_world();
    let ev = stand_at(&mut w, 700, [0.0, 0.0, 99.0]);
    let beats = mech(&ev, MechanicEvent::BELL_BEAT);
    assert!(beats.len() >= 16, "{} beats", beats.len());
    for (i, pair) in beats.windows(2).enumerate() {
        assert_eq!(pair[1].0 - pair[0].0, 40, "beat {i} to {}", i + 1);
        assert_eq!(pair[1].1, (pair[0].1 + 1) % 4, "the bar's place");
    }
    assert_eq!(beats[0].1, 0, "the first beat is the bar's first");
    // The toll is on beat 0 of each bar after the first: 160 ticks apart, the first 160 ticks
    // after the first beat.
    let tolls = mech(&ev, MechanicEvent::BELL_TOLLED);
    assert!(tolls.len() >= 4, "{tolls:?}");
    assert_eq!(tolls[0].0, beats[0].0 + 160);
    for pair in tolls.windows(2) {
        assert_eq!(pair[1].0 - pair[0].0, 160);
    }
    // The bell winds up 20 ticks before each toll.
    let warns = mech(&ev, MechanicEvent::BELL_TOLL_WARNED);
    for (warn, toll) in warns.iter().zip(&tolls) {
        assert_eq!(toll.0 - warn.0, 20);
    }
    assert_eq!(w.mech.b10.tolls as usize, tolls.len());
}

#[test]
fn he_starts_every_move_on_the_slot_so_each_hit_lands_on_a_beat() {
    let mut w = fight_world();
    let mut starts = Vec::new();
    let mut hits = Vec::new();
    let mut beat_ticks = Vec::new();
    let mut last: Option<(MoveNum, u16)> = None;
    for _ in 0..1500 {
        put_player(&mut w, [0.0, 0.0, 101.0]);
        let ev = run_ticks(&mut w, 1, true, |_| ActionFrame::default());
        let b = actor(&w, body_of(&w));
        let now = b.moves.current.map(|m| (m.num, m.frame));
        if let Some((n, f)) = now
            && last.is_none_or(|(ln, lf)| ln != n || f < lf)
            && f <= 1
        {
            starts.push((w.tick, n));
        }
        last = now;
        beat_ticks.extend(mech(&ev, MechanicEvent::BELL_BEAT).into_iter().map(|b| b.0));
        for (t, e) in ev {
            if let Event::Hit(h) = e
                && h.target == w.player
            {
                hits.push(t);
            }
        }
    }
    let from = w.mech.b10.from;
    assert!(starts.len() >= 5, "{} moves", starts.len());
    // Every move starts the same number of ticks after a beat (the slot), and every hit lands
    // exactly on a beat.
    let slot = (starts[0].0 - from) % 40;
    for (t, _) in &starts {
        assert_eq!((t - from) % 40, slot, "move started at tick {t}");
    }
    assert!(!hits.is_empty(), "no hit landed on the player");
    // The tick a hit event carries is the tick of the beat event: the hits land on the beat.
    for t in &hits {
        assert!(
            beat_ticks.contains(t),
            "a hit landed at tick {t}, off the beat (the slot is {slot}; from {from}; beats {:?}; starts {:?}; hits {hits:?})",
            &beat_ticks[..6],
            &starts[..3]
        );
    }
}

#[test]
fn every_hit_of_every_move_lands_on_the_beat_grid() {
    let content = Content::load_embedded().unwrap();
    let def = content.boss_by_id("b10_the_bellwright").unwrap();
    let (offset, p1, p2) = (
        def.params["slot_offset_ticks"] as u16,
        def.params["beat_ticks_p1"] as u16,
        def.params["beat_ticks_p2"] as u16,
    );
    for (phase, period) in [(1u8, p1), (2u8, p2)] {
        for o in def.body.options.iter().filter(|o| o.move_id.is_some()) {
            let in_phase = o
                .when
                .iter()
                .any(|c| matches!(c, lf_content::enemy::Condition::Phase(p) if *p == phase));
            if !in_phase {
                continue;
            }
            let m = content.move_by_id(o.move_id.as_deref().unwrap()).unwrap();
            let id = &m.id;
            assert_eq!(m.total_frames % period, 0, "{id}: whole beats long");
            assert!(!m.hitboxes.is_empty(), "{id}");
            for h in &m.hitboxes {
                // A hit on frame F lands F - 1 ticks after the move is picked (see the boss file).
                assert_eq!(
                    (h.frames.0 - 1 + offset) % period,
                    0,
                    "{id}: hit at {}",
                    h.frames.0
                );
            }
        }
    }
}

#[test]
fn the_shockwave_hits_a_player_on_the_ground() {
    let mut w = fight_world();
    let at = w.mech.b10;
    assert_eq!(at.tolls, 0);
    let ev = stand_at(&mut w, 260, [0.0, 0.0, 100.0]);
    let tolls = mech(&ev, MechanicEvent::BELL_TOLLED);
    let hit = mech(&ev, MechanicEvent::BELL_TOLL_HIT);
    assert_eq!(tolls.len(), 1, "{tolls:?}");
    assert_eq!(hit.len(), 1, "the ring hit once: {hit:?}");
    // The ring runs 0.9 m a tick, and he has walked up to the player: it arrives within a few ticks.
    assert!(
        hit[0].0 >= tolls[0].0 && hit[0].0 - tolls[0].0 <= 14,
        "{hit:?} {tolls:?}"
    );
}

/// Runs a fight past its first toll with the player standing where the Bellwright walks up to
/// him, pressing the button `button` says each tick (given the world, the player's spot, and his).
fn through_the_first_toll(
    mut button: impl FnMut(&World, [f32; 3], [f32; 3]) -> u16,
) -> Vec<(u64, Event)> {
    let mut w = fight_world();
    let mut out = Vec::new();
    for _ in 0..260 {
        let pp = actor(&w, w.player).pos;
        let body = actor(&w, body_of(&w)).pos;
        let pressed = button(&w, pp, body);
        out.extend(run_ticks(&mut w, 1, true, |_| ActionFrame {
            buttons: pressed,
            ..ActionFrame::default()
        }));
    }
    out
}

#[test]
fn a_roll_goes_through_the_shockwave() {
    // The bot's own timing: it presses DODGE when the edge is 5 ticks from it.
    let mut rolled = false;
    let ev = through_the_first_toll(|w, pp, body| {
        let go = !rolled && toll_eta(w, pp, body).is_some_and(|(_, t)| t > 0.0 && t <= 5.0);
        rolled |= go;
        if go { buttons::DODGE } else { 0 }
    });
    assert!(rolled);
    assert_eq!(mech(&ev, MechanicEvent::BELL_TOLLED).len(), 1);
    assert!(
        mech(&ev, MechanicEvent::BELL_TOLL_HIT).is_empty(),
        "the roll was hit"
    );
}

#[test]
fn a_jump_goes_over_the_shockwave() {
    // Jump 7 ticks before the edge arrives (startup 4, then in the air).
    let mut jumped = false;
    let ev = through_the_first_toll(|w, pp, body| {
        let go = !jumped && toll_eta(w, pp, body).is_some_and(|(_, t)| t > 0.0 && t <= 7.0);
        jumped |= go;
        if go { buttons::JUMP } else { 0 }
    });
    assert!(jumped);
    assert_eq!(mech(&ev, MechanicEvent::BELL_TOLLED).len(), 1);
    assert!(
        mech(&ev, MechanicEvent::BELL_TOLL_HIT).is_empty(),
        "the jump was hit"
    );
}

#[test]
fn the_ice_cracks_in_turn_and_breaks_two_tolls_later() {
    let mut w = fight_world();
    let ev = stand_at(&mut w, 160 * 6 + 20, [0.0, 0.0, 99.0]);
    let cracked = mech(&ev, MechanicEvent::ICE_CRACKED);
    let broke = mech(&ev, MechanicEvent::ICE_BROKE);
    let tolls = mech(&ev, MechanicEvent::BELL_TOLLED);
    assert!(tolls.len() >= 6);
    // Patch i cracks at toll i+1 and breaks at toll i+3.
    assert_eq!(
        cracked.iter().map(|c| c.1).collect::<Vec<_>>(),
        vec![0, 1, 2, 3],
        "{cracked:?}"
    );
    assert_eq!(
        broke.iter().map(|c| c.1).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    for i in 0..4 {
        assert_eq!(
            cracked[i].0,
            tolls[i].0,
            "patch {i} cracks with toll {}",
            i + 1
        );
        assert_eq!(
            broke[i].0,
            tolls[i + 2].0,
            "patch {i} breaks two tolls later"
        );
    }
    // The state the tolls give.
    assert_eq!(ice_after(&w, 0, 0), Ice::Sound);
    assert_eq!(ice_after(&w, 0, 1), Ice::Cracked);
    assert_eq!(ice_after(&w, 0, 2), Ice::Cracked);
    assert_eq!(ice_after(&w, 0, 3), Ice::Broken);
    assert_eq!(ice_after(&w, 3, 5), Ice::Cracked);
    assert_eq!(ice_after(&w, 3, 6), Ice::Broken);
}

#[test]
fn freezing_water_burns_a_player_in_a_broken_patch_and_only_there() {
    let mut w = fight_world();
    let patch0 = zones(&w)[0];
    // Three tolls: patch 0 is broken. Wait them out away from every patch.
    stand_at(&mut w, 160 * 3 + 20, [0.0, 0.0, 99.0]);
    assert_eq!(w.mech.b10.tolls, 3);
    // Patch 1 is cracked but whole, and patch 2 is sound: no burn on either.
    let (cracked, sound) = (zones(&w)[1], zones(&w)[2]);
    let dry = stand_at(&mut w, 50, [cracked[0], 0.0, cracked[2]]);
    assert!(mech(&dry, MechanicEvent::ICE_WATER_BURNED).is_empty());
    let dry = stand_at(&mut w, 50, [sound[0], 0.0, sound[2]]);
    assert!(mech(&dry, MechanicEvent::ICE_WATER_BURNED).is_empty());
    // In the broken patch: a burn every 20 ticks, on patch 0.
    let wet = stand_at(&mut w, 100, [patch0[0], 0.0, patch0[2]]);
    let burns = mech(&wet, MechanicEvent::ICE_WATER_BURNED);
    assert_eq!(burns.len(), 5, "{burns:?}");
    assert!(burns.iter().all(|b| b.1 == 0));
}

#[test]
fn the_chain_snaps_for_phase_2_and_the_beat_speeds_up_to_32() {
    let mut w = fight_world();
    stand_at(&mut w, 100, [0.0, 0.0, 99.0]);
    // Drop him below half: the transition starts.
    let body = body_of(&w);
    {
        let a = w.actors.get_mut(body).unwrap();
        a.hp = a.max_hp * 45 / 100;
    }
    let ev = stand_at(&mut w, 400, [0.0, 0.0, 99.0]);
    assert_eq!(w.bosses[0].phase, 2);
    let snapped = mech(&ev, MechanicEvent::BELL_CHAIN_SNAPPED);
    assert_eq!(snapped.len(), 1);
    // After the snap the beat is 32 ticks, and the toll is 4 beats (128 ticks).
    let beats: Vec<_> = mech(&ev, MechanicEvent::BELL_BEAT)
        .into_iter()
        .filter(|b| b.0 >= snapped[0].0)
        .collect();
    assert!(beats.len() >= 6);
    for pair in beats.windows(2) {
        assert_eq!(pair[1].0 - pair[0].0, 32);
    }
    let tolls: Vec<_> = mech(&ev, MechanicEvent::BELL_TOLLED)
        .into_iter()
        .filter(|b| b.0 > snapped[0].0)
        .collect();
    assert!(tolls.len() >= 2);
    assert_eq!(tolls[1].0 - tolls[0].0, 128);
}

#[test]
fn the_players_death_resets_the_fight_and_the_ice() {
    let mut w = fight_world();
    stand_at(&mut w, 160 * 3 + 20, [0.0, 0.0, 99.0]);
    assert!(w.mech.b10.tolls >= 3);
    w.actors.get_mut(w.player).unwrap().hp = 0;
    w.actors.get_mut(w.player).unwrap().dead = true;
    run_ticks(&mut w, 5, false, |_| ActionFrame::default());
    assert_eq!(w.mech.b10, B10State::default(), "the fight is over: rest");
}

#[test]
fn the_arena_in_the_level_has_the_boss_and_its_fog_gate() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(20, &content, "rimeshelf").unwrap();
    assert_eq!(w.layout.bosses.len(), 1);
    let lay = &w.layout.bosses[0];
    assert_eq!(lay.def.id, BOSS_ID);
    assert_eq!(lay.members.len(), 1);
    assert_eq!(lay.gate.pos, [0.0, 6.0, 96.0]);
    // The player at the gate seals it, and the fight starts after the intro.
    put_player(&mut w, [0.0, 6.0, 97.0]);
    run_ticks(&mut w, 5, true, |_| ActionFrame::default());
    assert!(matches!(w.bosses[0].stage, Stage::Intro { .. }));
    run_ticks(&mut w, 95, true, |_| ActionFrame::default());
    assert_eq!(w.bosses[0].stage, Stage::Fight);
    assert_eq!(actor(&w, body_of(&w)).pos, [0.0, 6.0, 112.0]);
}

// ---------------------------------------------------------------------------------------
// The fight
// ---------------------------------------------------------------------------------------

#[test]
fn the_scripted_bot_beats_b10_by_the_beat() {
    let f = run_bot(SEED);
    assert!(f.boss_defeated(), "the bot lost");
    assert!(!f.player_died());
    assert_eq!(f.boss_events(BossEvent::PHASE_STARTED), 1);
    assert_eq!(f.mechanic_count(MechanicEvent::BELL_CHAIN_SNAPPED), 1);
    assert!(
        f.mechanic_count(MechanicEvent::BELL_TOLLED) >= 2,
        "the bell tolled"
    );
    // The Sigil, the Mallet, and the Glimmer paid out.
    let inv = &f.world.actors.get(f.world.player).unwrap().inventory;
    assert_eq!(inv.glimmer, 5000);
    let content = Content::load_embedded().unwrap();
    let sigil = content.registry.num("itm_great_sigil_bellwright").unwrap();
    let mallet = content.registry.num("wpn_bellwright_mallet").unwrap();
    assert!(inv.count(sigil) >= 1, "no Great Sigil");
    assert!(inv.gear.contains(&mallet), "no Mallet");
}

#[test]
fn a_player_who_does_nothing_dies_to_b10() {
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
fn the_bot_golden_replay_beats_b10_with_the_player_alive() {
    let f = play_golden("m7_b10_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b10_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b10_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b10_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b10_goldens` in game/crates/core/src/ai/bosses/b10_tests.rs.\n\
             # Each line: <ticks> <move_x> <move_z> [BUTTONS].\nseed {SEED}\nscenario {SCENARIO}\n"
        );
        for (frame, n) in crate::replay::compress(&f.frames) {
            let mut names = Vec::new();
            for (mask, name) in [
                (buttons::LIGHT, "LIGHT"),
                (buttons::INTERACT, "INTERACT"),
                (buttons::DODGE, "DODGE"),
                (buttons::JUMP, "JUMP"),
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
        "m7_b10_bot_win",
        "WP-078 golden: the scripted bot beats B10 by the beat: it rolls each hit and each toll, and strikes in the gaps.",
        &run_bot(SEED),
    );
    write(
        "m7_b10_idle_dies",
        "WP-078 golden: a player who walks in and does nothing dies to B10.",
        &run_idle(SEED),
    );
}

#[test]
fn b10_state_is_in_the_state_hash_during_the_fight_and_out_of_it_at_rest() {
    let mut w = fight_world();
    stand_at(&mut w, 160 + 20, [0.0, 0.0, 99.0]);
    assert!(!w.mech.b10.is_rest(), "the fight is on");
    let mut other = w.clone();
    other.mech.b10 = B10State::default();
    assert_ne!(
        w.state_hash(),
        other.state_hash(),
        "the beat state is hashed while the fight is on"
    );
    // After a reset the state is at rest and the hash reads the same as a world that never
    // fought.
    w.actors.get_mut(w.player).unwrap().hp = 0;
    w.actors.get_mut(w.player).unwrap().dead = true;
    run_ticks(&mut w, 5, false, |_| ActionFrame::default());
    assert!(w.mech.b10.is_rest());
    let mut rest = w.clone();
    rest.mech.b10 = B10State::default();
    assert_eq!(w.state_hash(), rest.state_hash());
}
