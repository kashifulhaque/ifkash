//! Tests for B11, the Bound Exile (WP-079), and the scripted bot that beats it.
//!
//! The bot is the proof that the fight is fair (docs/game/05-enemies-and-bosses.md, "B11 The
//! Bound Exile"): it reads the real Exile's move each tick, rolls so that the roll's i-frames
//! cover each hit about to land (the beam, the burst where it blinks in, an orb), breaks the
//! glyph ring by getting in a hit while it is lit, and punishes the Exile in the recovery of its
//! last move and while it is staggered. It never goes after the image (hits on it count for
//! nothing). It is deterministic (no randomness, no clock). [`run_bot`] and [`run_idle`] play a
//! whole fight from any seed.
//!
//! The goldens `m7_b11_bot_win.lfr` and `m7_b11_idle_dies.lfr` come from these inputs:
//! `record_the_b11_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;
use lf_content::moves::Shape;

use crate::ai::boss::Stage;
use crate::arena::Id;
use crate::combat::moves::MoveNum;
use crate::events::{Event, LifeEvent, MechanicEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::math::dmath::{cos, sin, sqrt};
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;
use crate::world_state::mechanics::brazier;

const SCENARIO: &str = "boss_b11";
const BOSS_ID: &str = "b11_bound_exile";
/// The seed the goldens use.
const SEED: u64 = 79;
/// The longest fight the runners play, in ticks (the Exile has 4,500 HP; this only stops a
/// stuck run).
const MAX_TICKS: u64 = 30000;

/// The members' actors: the Exile first, then its image.
fn members(w: &World) -> Vec<Id> {
    w.layout.bosses[0]
        .members
        .iter()
        .map(|&s| w.state.ai_enemies[usize::from(s)])
        .collect()
}

fn planar(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
    sqrt(dx * dx + dz * dz)
}

fn toward(from: [f32; 3], to: [f32; 3]) -> (i8, i8) {
    let d = planar(from, to).max(0.001);
    let s = |v: f32| (v / d * 127.0) as i8;
    (s(to[0] - from[0]), s(to[2] - from[2]))
}

fn number(w: &World, key: &str) -> f32 {
    w.layout.bosses[0].def.params[key]
}

/// True if the Exile is in `b11_beam` with the glyph ring lit: frames `glyph_from` to
/// `glyph_to`.
fn glyph_lit(w: &World) -> Option<u16> {
    let a = w.actors.get(members(w)[0])?;
    let c = a.moves.current?;
    let beam = w.moves.num_of("b11_beam")?;
    let f = f32::from(c.frame);
    (!a.dead && c.num == beam && f >= number(w, "glyph_from") && f <= number(w, "glyph_to"))
        .then_some(c.frame)
}

/// How far (planar, in meters) a move's hit volumes reach from its user; used to tell whether a
/// coming hit can reach the player.
fn reach(w: &World, num: MoveNum) -> f32 {
    let Some(def) = w.moves.get(num) else {
        return 0.0;
    };
    let mut r = 0.0f32;
    for h in &def.data.hitboxes {
        let (pts, radius) = match h.shape {
            Shape::Capsule { a, b, radius } => (vec![a, b], radius),
            Shape::Sphere { center, radius } => (vec![center], radius),
        };
        for p in pts {
            r = r.max(sqrt(p[0] * p[0] + p[2] * p[2]) + radius);
        }
    }
    r + def.data.root_motion.last().map_or(0.0, |k| k.1)
}

/// The scripted player.
pub(crate) struct Bot {
    /// How many ticks before a hit's first frame the bot presses DODGE: the light roll's
    /// i-frames are its frames 2 to 14, so this centers them on a short hit window.
    lead: u16,
    /// The tick the last roll was pressed, so one roll answers a burst.
    last_roll: Option<u64>,
    /// Buttons sent last tick, because every button is a press edge.
    last: u16,
    /// The stick sent last tick. A dodge counts on its release, with the stick of that tick,
    /// so the tick after a DODGE press repeats it.
    last_stick: (i8, i8),
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            lead: 7,
            last_roll: None,
            last: 0,
            last_stick: (0, 0),
        }
    }
}

impl Bot {
    /// The input for the next tick, from the world after the last one.
    pub(crate) fn act(&mut self, w: &World) -> ActionFrame {
        let (mut stick, mut pressed) = self.decide(w);
        if self.last & buttons::DODGE != 0 {
            // The release tick of a dodge tap: keep the direction it was pressed with.
            stick = self.last_stick;
            pressed = 0;
        }
        pressed &= !self.last;
        self.last = pressed;
        self.last_stick = stick;
        ActionFrame {
            move_x: stick.0,
            move_z: stick.1,
            buttons: pressed,
        }
    }

    /// The ticks until the nearest hit coming at the player lands: the Exile's moves (the
    /// image has none) and every shot in flight.
    fn incoming(&self, w: &World, me: [f32; 3]) -> Option<(Id, i32)> {
        let mut best: Option<(Id, i32)> = None;
        let id = members(w)[0];
        if let Some(m) = w.actors.get(id)
            && !m.dead
            && m.hp > 0
            && let Some(c) = m.moves.current
            && let Some(def) = w.moves.get(c.num)
        {
            let first = def
                .data
                .hitboxes
                .iter()
                .map(|h| h.frames.0)
                .chain(def.data.projectiles.iter().map(|p| p.frame))
                .min();
            if let Some(first) = first
                && c.frame < first
                && planar(m.pos, me) <= reach(w, c.num) + 0.6
                && def.data.projectiles.is_empty()
            {
                best = Some((id, i32::from(first - c.frame)));
            }
        }
        // Shots in flight.
        for (_, p) in w.projectiles.iter() {
            let (dx, dz) = (me[0] - p.pos[0], me[2] - p.pos[2]);
            let d = sqrt(dx * dx + dz * dz);
            let sp = sqrt(p.vel[0] * p.vel[0] + p.vel[2] * p.vel[2]) / 30.0;
            if sp < 0.01 || d > 14.0 {
                continue;
            }
            // Heading at the player?
            let along = (dx * p.vel[0] + dz * p.vel[2]) / (d.max(0.001) * sp * 30.0);
            if along < 0.8 {
                continue;
            }
            let eta = (d / sp) as i32;
            if best.is_none_or(|b| eta < b.1) {
                best = Some((p.shooter, eta));
            }
        }
        best
    }

    fn decide(&mut self, w: &World) -> ((i8, i8), u16) {
        let Some(player) = w.actors.get(w.player) else {
            return ((0, 0), 0);
        };
        let ids = members(w);
        let exile = w.actors.get(ids[0]).map_or([0.0; 3], |a| a.pos);
        match w.bosses[0].stage {
            Stage::Dormant => return (toward(player.pos, exile), 0), // walk through the gate
            Stage::Intro { .. } | Stage::Defeated => return ((0, 0), 0),
            Stage::Fight => {}
        }
        if player.dead || player.hp == 0 {
            return ((0, 0), 0);
        }
        let rolling = player
            .moves
            .current
            .and_then(|c| w.moves.player_moveset.as_ref().map(|s| (c.num, s)))
            .is_some_and(|(n, s)| n == s.roll_light || n == s.roll_medium || n == s.roll_heavy);
        let f = w.actors.get(ids[0]).unwrap();
        let target = f.pos;
        let dist = planar(player.pos, target);

        // 0. Break the glyph ring: a hit while it is lit staggers the Exile. Only worth trying
        // when the swing can land before the beam does.
        if !rolling
            && player.stamina >= 26.0
            && dist <= 2.3
            && let Some(frame) = glyph_lit(w)
            && f32::from(frame) <= number(w, "glyph_to") - 12.0
        {
            return (toward(player.pos, target), buttons::LIGHT);
        }
        // 1. Roll through a hit that is about to land.
        if !rolling
            && player.stamina >= 14.0
            && self.last_roll.is_none_or(|t| w.tick > t + 14)
            && let Some((id, eta)) = self.incoming(w, player.pos)
            && eta <= i32::from(self.lead)
        {
            self.last_roll = Some(w.tick);
            let from = w.actors.get(id).map_or(target, |a| a.pos);
            return (toward(player.pos, from), buttons::DODGE);
        }
        // A roll's recovery can be cut short by an attack (`cancel_attack_from`: the rolling
        // attack), which is how the bot punishes right after it dodged.
        let roll_frame = player.moves.current.map_or(0, |c| c.frame);
        if rolling && roll_frame < 15 {
            return ((0, 0), 0);
        }
        // 2. Punish: the Exile staggered, or in the recovery of its last move, close enough, and
        // nothing else about to land.
        let threatened = self
            .incoming(w, player.pos)
            .is_some_and(|(_, eta)| eta <= 26);
        let punish = || -> Option<u16> {
            let m = w.actors.get(ids[0])?;
            if m.dead || m.hp == 0 {
                return None;
            }
            if m.moves.guard.stagger > 8 {
                return Some(m.moves.guard.stagger.into());
            }
            // Between moves it stands open (a caster has no melee to fear).
            let Some(c) = m.moves.current else {
                return Some(30);
            };
            let def = w.moves.get(c.num)?;
            let last = def.data.hitboxes.iter().map(|h| h.frames.1).max();
            let spawns = def.data.projectiles.iter().map(|p| p.frame).max();
            let last = last.max(spawns)?;
            let left = def.data.total_frames.saturating_sub(c.frame);
            (def.next_chain.is_none() && c.frame > last && left >= 14).then_some(left)
        };
        if !threatened && let Some(left) = punish() {
            let low = player.hp * 100 < player.max_hp * 45;
            if low && player.inventory.flasks.vigil > 0 && left >= 30 {
                return ((0, 0), buttons::USE_ITEM);
            }
            if dist <= 2.2 && player.stamina >= 26.0 {
                return (toward(player.pos, target), buttons::LIGHT);
            }
        }
        // 3. Close the distance.
        if dist > 1.8 {
            return (toward(player.pos, target), 0);
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

/// Plays the fight with the scripted bot until the Exile falls or the player dies.
pub(crate) fn run_bot(seed: u64) -> Fight {
    let mut bot = Bot::default();
    play(seed, |w| bot.act(w))
}

/// Plays the fight with a player who walks in through the gate and then does nothing.
pub(crate) fn run_idle(seed: u64) -> Fight {
    play(seed, |w| {
        let (stick, _) = if w.bosses[0].stage == Stage::Dormant {
            let p = w.actors.get(w.player).map_or([0.0; 3], |a| a.pos);
            let c = w.actors.get(members(w)[0]).map_or([0.0; 3], |a| a.pos);
            (toward(p, c), 0)
        } else {
            ((0, 0), 0)
        };
        ActionFrame {
            move_x: stick.0,
            move_z: stick.1,
            buttons: 0,
        }
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

/// A world with the player through the gate and the intro over: the fight is on.
fn fight_world(seed: u64) -> World {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(seed, &content, SCENARIO).unwrap();
    let mut bot = Bot::default();
    while w.bosses[0].stage != Stage::Fight {
        let frame = bot.act(&w);
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        assert!(w.tick < 1000, "the fight never started");
    }
    let _ = w.events.drain();
    w
}

/// One tick with no input; returns the events it raised.
fn idle_tick(w: &mut World) -> Vec<Event> {
    tick(
        w,
        &TickInput {
            frame: ActionFrame::default(),
            commands: Vec::new(),
        },
    );
    w.events.drain()
}

fn start_move(w: &mut World, member: usize, name: &str, frame: u16) {
    let num = w.moves.num_of(name).unwrap();
    let id = members(w)[member];
    let a = w.actors.get_mut(id).unwrap();
    a.moves.current = Some(crate::combat::moves::ActiveMove {
        num,
        frame,
        charge_ticks: 0,
        hold_button: 0,
        free_from: 999,
    });
}

fn mechanic_kinds(events: &[Event]) -> Vec<(u8, u16)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Mechanic(m) => Some((m.kind, m.index)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_scripted_bot_beats_b11_by_rolling_and_punishing() {
    let f = run_bot(SEED);
    assert!(
        f.boss_defeated(),
        "the Exile stands at tick {} (player hp {})",
        f.world.tick,
        f.player_hp()
    );
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    assert_eq!(f.world.bosses[0].phase, 2);
    let kinds = mechanic_kinds(&f.events);
    assert!(kinds.iter().any(|k| k.0 == MechanicEvent::EXILE_BLINK));
    assert!(kinds.iter().any(|k| k.0 == MechanicEvent::EXILE_SPLIT));
}

#[test]
fn a_player_who_does_nothing_dies_to_b11() {
    let f = run_idle(SEED);
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

#[test]
fn the_bot_golden_replay_beats_b11_with_the_player_alive() {
    let f = play_golden("m7_b11_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    let p = f.world.actors.get(f.world.player).unwrap();
    assert_eq!(p.inventory.glimmer, 2500);
    assert!(f.world.state.flags.get("codex.b11_bound_exile") > 0);
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b11_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

#[test]
fn the_bot_wins_from_other_seeds_too() {
    for seed in [1, 5, 99] {
        let f = run_bot(seed);
        assert!(f.boss_defeated() && !f.player_died(), "seed {seed}");
    }
}

/// The ring's stones, as the mechanic places them: radius `ring_radius_m` about the ring's
/// center, from the Exile's spawn.
fn stones(w: &World) -> Vec<[f32; 2]> {
    let spawn = w.layout.enemies[usize::from(w.layout.bosses[0].members[0])].pos;
    let c = [
        spawn[0] + number(w, "ring_dx_m"),
        spawn[2] + number(w, "ring_dz_m"),
    ];
    let (n, r, first) = (
        number(w, "stone_count") as usize,
        number(w, "ring_radius_m"),
        number(w, "stone_first_deg"),
    );
    (0..n)
        .map(|k| {
            let a = (first + 360.0 * k as f32 / n as f32).to_radians();
            [c[0] + r * sin(a), c[1] + r * cos(a)]
        })
        .collect()
}

fn at_stone(w: &World, id: Id) -> Option<usize> {
    let p = w.actors.get(id)?.pos;
    stones(w)
        .iter()
        .position(|s| (s[0] - p[0]).abs() < 0.4 && (s[1] - p[2]).abs() < 0.4)
}

fn put(w: &mut World, id: Id, pos: [f32; 3]) {
    let a = w.actors.get_mut(id).unwrap();
    a.pos = pos;
    a.vel = [0.0; 3];
}

fn hurt_exile(w: &mut World, poise_damage: u32) -> bool {
    let t = w.locomotion.poise;
    let id = members(w)[0];
    let a = w.actors.get_mut(id).unwrap();
    crate::combat::poise::apply_hit(a, poise_damage, 1000, 100, &t)
        .stagger
        .is_some()
}

fn image_up(w: &World) -> bool {
    let a = w.actors.get(members(w)[1]).unwrap();
    !a.dead && a.hp > 0
}

fn image_hidden(w: &World) -> bool {
    let a = w.actors.get(members(w)[1]).unwrap();
    a.dead && a.ai.as_ref().is_some_and(|b| b.hidden)
}

/// Phase 2 has begun: the Exile below half HP, and the split played out.
fn phase_two_world(seed: u64) -> World {
    let mut w = fight_world(seed);
    let id = members(&w)[0];
    let a = w.actors.get_mut(id).unwrap();
    a.hp = a.max_hp / 2 - 10;
    for _ in 0..200 {
        idle_tick(&mut w);
        if w.bosses[0].phase == 2 {
            return w;
        }
    }
    panic!("phase 2 never began");
}

#[test]
fn a_hit_during_the_glyph_ring_staggers_the_exile_and_breaks_the_cast() {
    let mut w = fight_world(SEED);
    let (from, to) = (
        number(&w, "glyph_from") as u16,
        number(&w, "glyph_to") as u16,
    );
    // Anywhere in the lit window, even the smallest hit staggers (the Exile's own poise is 180).
    for frame in [from, 30, to] {
        start_move(&mut w, 0, "b11_beam", frame);
        idle_tick(&mut w);
        assert!(hurt_exile(&mut w, 1), "no stagger at frame {frame}");
        let a = w.actors.get(members(&w)[0]).unwrap();
        assert!(
            a.moves.current.is_none(),
            "the cast went on at frame {frame}"
        );
        assert!(a.moves.guard.stagger > 0);
    }
    // Before the ring is lit, and after the beam has gone out, the poise is its own.
    for frame in [from - 1, to + 1, 100] {
        let mut w = fight_world(SEED);
        start_move(&mut w, 0, "b11_beam", frame);
        // One tick moves the move on a frame; the check is on the frame it was at the tick.
        w.actors.get_mut(members(&w)[0]).unwrap().poise.accum = 0;
        idle_tick(&mut w);
        assert!(!hurt_exile(&mut w, 1), "staggered at frame {frame}");
    }
    // Another move's poise is untouched too.
    let mut w = fight_world(SEED);
    start_move(&mut w, 0, "b11_orbs", 30);
    idle_tick(&mut w);
    assert!(!hurt_exile(&mut w, 40));
}

#[test]
fn the_glyph_ring_is_a_window_of_sixty_frames_and_the_beam_lands_after_it() {
    let w = fight_world(SEED);
    let beam = w.moves.get(w.moves.num_of("b11_beam").unwrap()).unwrap();
    let first_hit = beam.data.hitboxes.iter().map(|h| h.frames.0).min().unwrap();
    let (from, to) = (
        number(&w, "glyph_from") as u16,
        number(&w, "glyph_to") as u16,
    );
    assert_eq!(to + 1, first_hit, "the window closes as the beam lands");
    assert_eq!(to - from + 1, 60);
}

#[test]
fn a_blink_lands_on_a_stone_away_from_the_player_and_the_stone_it_left() {
    let frame = {
        let w = fight_world(SEED);
        number(&w, "teleport_frame") as u16
    };
    for seed in 1..=24u64 {
        let mut w = fight_world(seed);
        let exile = members(&w)[0];
        // The player stands in the middle of the ring; the Exile on stone 0.
        let s = stones(&w);
        put(&mut w, exile, [s[0][0], 0.0, s[0][1]]);
        let from = at_stone(&w, exile).unwrap();
        let p = w.actors.get(w.player).unwrap().pos;
        start_move(&mut w, 0, "b11_blink", frame);
        let events = idle_tick(&mut w);
        let to = at_stone(&w, exile).expect("not on a stone");
        assert_ne!(to, from, "seed {seed}");
        let a = w.actors.get(exile).unwrap();
        assert!(planar(a.pos, p) >= number(&w, "blink_min_player_m") - 0.1);
        assert!(mechanic_kinds(&events).contains(&(MechanicEvent::EXILE_BLINK, to as u16)));
    }
}

#[test]
fn a_blink_that_has_not_reached_its_frame_moves_nothing() {
    let mut w = fight_world(SEED);
    let exile = members(&w)[0];
    put(&mut w, exile, [3.0, 0.0, 10.0]);
    start_move(&mut w, 0, "b11_blink", 5);
    let events = idle_tick(&mut w);
    let a = w.actors.get(exile).unwrap();
    assert!((a.pos[0] - 3.0).abs() < 0.5 && (a.pos[2] - 10.0).abs() < 0.5);
    assert!(
        !mechanic_kinds(&events)
            .iter()
            .any(|k| k.0 == MechanicEvent::EXILE_BLINK)
    );
}

#[test]
fn the_image_is_hidden_and_down_until_the_split() {
    let mut w = fight_world(SEED);
    idle_tick(&mut w);
    assert!(image_hidden(&w));
    assert!(!image_up(&w));
    // The Exile is below half: the transition runs, and the image steps out near its end.
    let id = members(&w)[0];
    let a = w.actors.get_mut(id).unwrap();
    a.hp = a.max_hp / 2 - 10;
    let left = number(&w, "split_ticks_left") as u16;
    let mut split = None;
    for _ in 0..200 {
        let events = idle_tick(&mut w);
        let r = w.bosses[0];
        if r.transition_left > left {
            assert!(!image_up(&w), "the image is up early");
        }
        if image_up(&w) && split.is_none() {
            split = Some((
                r.transition_left,
                mechanic_kinds(&events),
                at_stone(&w, members(&w)[1]),
            ));
        }
        if r.phase == 2 {
            break;
        }
    }
    let (at, kinds, stone) = split.expect("the image never appeared");
    // The framework counts the transition down after the mechanic ran, so the tick that raised
    // the image shows one fewer left.
    assert_eq!(at + 1, left);
    assert!(
        kinds
            .iter()
            .any(|k| k.0 == MechanicEvent::EXILE_SPLIT && k.1 >> 8 == 1)
    );
    // It stands on a stone, away from the player, alive, and in the fight.
    let img = members(&w)[1];
    let a = w.actors.get(img).unwrap();
    assert!(stone.is_some(), "not on a stone when it appeared");
    assert!(!image_hidden(&w));
    assert_eq!(a.hp, a.max_hp);
    assert_eq!(a.ai.as_ref().unwrap().phase, 2);
}

#[test]
fn nothing_done_to_the_image_counts() {
    let mut w = phase_two_world(SEED);
    let img = members(&w)[1];
    w.actors.get_mut(img).unwrap().hp = 5;
    idle_tick(&mut w);
    let a = w.actors.get(img).unwrap();
    assert_eq!(a.hp, a.max_hp);
    // And the fight's HP is the Exile's alone: the image never counts, so the fight goes on
    // while it stands.
    assert_eq!(w.bosses[0].stage, Stage::Fight);
}

#[test]
fn only_the_real_exile_has_a_move_that_hurts() {
    let w = fight_world(SEED);
    for name in ["b11_image_blink", "b11_image_beam"] {
        let m = w.moves.get(w.moves.num_of(name).unwrap()).unwrap();
        assert!(m.data.hitboxes.is_empty() && m.data.projectiles.is_empty());
    }
    let content = Content::load_embedded().unwrap();
    let def = content.boss_by_id(BOSS_ID).unwrap();
    assert_eq!(def.parts[0].def.attack, 0);
    assert!(def.parts[0].def.element_attack.is_none());
    assert!(!def.parts[0].required);
}

#[test]
fn the_shuffle_swaps_the_exile_and_its_image_and_only_in_phase_two() {
    let content = Content::load_embedded().unwrap();
    let def = content.boss_by_id(BOSS_ID).unwrap();
    let o = def
        .body
        .options
        .iter()
        .find(|o| o.move_id.as_deref() == Some("b11_shuffle"))
        .unwrap();
    assert!(o.when.contains(&lf_content::enemy::Condition::Phase(2)));
    let mut w = phase_two_world(SEED);
    let (exile, img) = (members(&w)[0], members(&w)[1]);
    let s = stones(&w);
    put(&mut w, exile, [s[1][0], 0.0, s[1][1]]);
    put(&mut w, img, [s[5][0], 0.0, s[5][1]]);
    let frame = number(&w, "teleport_frame") as u16;
    start_move(&mut w, 0, "b11_shuffle", frame);
    let events = idle_tick(&mut w);
    let (ep, ip) = (
        w.actors.get(exile).unwrap().pos,
        w.actors.get(img).unwrap().pos,
    );
    assert_eq!(at_stone(&w, exile), Some(5), "{ep:?} {ip:?} {s:?}");
    assert_eq!(at_stone(&w, img), Some(1));
    assert!(mechanic_kinds(&events).contains(&(MechanicEvent::EXILE_SHUFFLE, 5)));
}

#[test]
fn defeating_the_exile_hides_the_image_and_pays_the_rewards() {
    let mut w = phase_two_world(SEED);
    let id = members(&w)[0];
    let a = w.actors.get_mut(id).unwrap();
    a.hp = 0;
    a.dead = true;
    idle_tick(&mut w);
    assert_eq!(w.bosses[0].stage, Stage::Defeated);
    idle_tick(&mut w);
    assert!(image_hidden(&w));
    assert!(w.state.flags.get(&flags::boss_defeated(BOSS_ID)) > 0);
    assert!(w.state.flags.get("codex.b11_bound_exile") > 0);
}

#[test]
fn a_death_puts_the_image_away_and_resets_the_split() {
    let mut w = phase_two_world(SEED);
    assert!(image_up(&w));
    {
        let p = w.actors.get_mut(w.player).unwrap();
        p.hp = 0;
        p.dead = true;
    }
    idle_tick(&mut w);
    assert_eq!(w.bosses[0].stage, Stage::Dormant);
    idle_tick(&mut w);
    assert!(image_hidden(&w));
    let a = w.actors.get(members(&w)[0]).unwrap();
    assert_eq!(a.hp, a.max_hp);
}

// ---------------------------------------------------------------------------------------
// The Bound Ring: the seal

const RING: &str = "bound_ring";

fn ring() -> World {
    let content = Content::load_embedded().unwrap();
    World::new(68, &content, RING).unwrap()
}

fn step(w: &mut World, x: i8, z: i8, b: u16) {
    tick(
        w,
        &TickInput {
            frame: ActionFrame {
                move_x: x,
                move_z: z,
                buttons: b,
            },
            commands: Vec::new(),
        },
    );
    let _ = w.events.drain();
}

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.vel = [0.0; 3];
}

fn touch_sigil(w: &mut World) {
    put_player(w, [0.0, 0.0, -16.0]);
    step(w, 0, 0, 0);
    step(w, 0, 0, buttons::INTERACT);
    for _ in 0..3 {
        step(w, 0, 0, 0);
    }
}

#[test]
fn the_ring_holds_the_exile_and_an_armed_sigil() {
    let w = ring();
    assert_eq!(w.layout.bosses.len(), 1);
    assert_eq!(w.layout.bosses[0].def.id, BOSS_ID);
    assert!(w.layout.sigils[0].armed);
    let spawn = w.layout.enemies[usize::from(w.layout.bosses[0].members[0])].pos;
    assert_eq!((spawn[0], spawn[2]), (0.0, 16.0));
    let g = w.layout.bosses[0].gate;
    assert_eq!((g.pos[0], g.pos[2]), (0.0, -14.0));
}

#[test]
fn touching_the_sigil_seals_the_ring_and_crossing_the_gate_starts_the_fight() {
    let mut w = ring();
    touch_sigil(&mut w);
    assert!(w.state.flags.is_set(&brazier::sealed_flag("sigil_ring")));
    assert!(!w.state.flags.is_set(&flags::door_open("door_ring_seal")));
    assert_eq!(w.bosses[0].stage, Stage::Dormant);
    put_player(&mut w, [0.0, 0.0, -13.0]);
    step(&mut w, 0, 0, 0);
    assert!(matches!(w.bosses[0].stage, Stage::Intro { .. }));
}

#[test]
fn the_exiles_fall_raises_the_release_and_opens_the_seal_door() {
    let mut w = ring();
    touch_sigil(&mut w);
    assert!(!w.state.flags.is_set(&flags::door_open("door_ring_seal")));
    // A defeated boss stays down: the fight's end raises the flag the framework raises.
    w.state.flags.raise(&flags::boss_defeated(BOSS_ID));
    for _ in 0..4 {
        step(&mut w, 0, 0, 0);
    }
    assert!(w.state.flags.is_set(&brazier::released_flag("sigil_ring")));
    assert!(w.state.flags.is_set(&flags::door_open("door_ring_seal")));
    // Touching the sigil again can't shut it.
    touch_sigil(&mut w);
    assert!(w.state.flags.is_set(&flags::door_open("door_ring_seal")));
}

#[test]
fn a_fight_ended_by_a_win_releases_the_seal_in_the_same_run() {
    let mut w = ring();
    touch_sigil(&mut w);
    put_player(&mut w, [0.0, 0.0, -13.0]);
    for _ in 0..100 {
        step(&mut w, 0, 0, 0);
    }
    assert_eq!(w.bosses[0].stage, Stage::Fight);
    let id = members(&w)[0];
    let a = w.actors.get_mut(id).unwrap();
    a.hp = 0;
    a.dead = true;
    for _ in 0..3 {
        step(&mut w, 0, 0, 0);
    }
    assert!(w.state.flags.is_set(&flags::boss_defeated(BOSS_ID)));
    assert!(w.state.flags.is_set(&brazier::released_flag("sigil_ring")));
    assert!(w.state.flags.is_set(&flags::door_open("door_ring_seal")));
}

#[test]
fn a_player_who_is_back_outside_the_ring_is_not_locked_out() {
    let mut w = ring();
    touch_sigil(&mut w);
    // Inside the Ring, with the fight off, the seal holds.
    put_player(&mut w, [0.0, 0.0, -18.0]);
    for _ in 0..4 {
        step(&mut w, 0, 0, 0);
    }
    assert!(w.state.flags.is_set(&brazier::sealed_flag("sigil_ring")));
    // They died and came back to a Lantern, far down the road: the seal is lifted, the door
    // opens, and the sigil can be touched again.
    put_player(&mut w, [0.0, 0.0, -60.0]);
    for _ in 0..4 {
        step(&mut w, 0, 0, 0);
    }
    assert!(!w.state.flags.is_set(&brazier::sealed_flag("sigil_ring")));
    assert!(w.state.flags.is_set(&flags::door_open("door_ring_seal")));
    touch_sigil(&mut w);
    assert!(w.state.flags.is_set(&brazier::sealed_flag("sigil_ring")));
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b11_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b11_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b11_goldens` in game/crates/core/src/ai/bosses/b11_tests.rs.\n\
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
        "m7_b11_bot_win",
        "WP-079 golden: the scripted bot beats the Bound Exile by rolling through each hit, breaking the glyph ring, and punishing.",
        &run_bot(SEED),
    );
    write(
        "m7_b11_idle_dies",
        "WP-079 golden: a player who walks in and does nothing dies to the Bound Exile.",
        &run_idle(SEED),
    );
}

/// WP-101: the blink points match the level (and the spec): one in front of each of the twelve
/// standing stones, inside it.
#[test]
fn the_blink_points_stand_in_front_of_the_levels_standing_stones() {
    let layout = include_str!("../../../../../data/levels/bound_ring/layout.ron");
    let mut standing = Vec::new();
    for line in layout.lines().filter(|l| l.contains("\"standing_stone_")) {
        let pos = line.split("pos: (").nth(1).expect("a pos");
        let n: Vec<f32> = pos
            .split(')')
            .next()
            .unwrap()
            .split(',')
            .map(|v| v.trim().parse().unwrap())
            .collect();
        standing.push([n[0], n[2]]);
    }
    assert_eq!(standing.len(), 12);
    let w = ring();
    let blink = stones(&w);
    assert_eq!(
        blink.len(),
        standing.len(),
        "one blink point per standing stone"
    );
    for s in &standing {
        let nearest = blink
            .iter()
            .map(|b| {
                let (dx, dz) = (b[0] - s[0], b[1] - s[1]);
                sqrt(dx * dx + dz * dz)
            })
            .fold(f32::MAX, f32::min);
        // In front of the stone (it is 1.6 m wide) and not inside it.
        assert!(
            (1.0..3.0).contains(&nearest),
            "stone at {s:?}: nearest blink point {nearest} m away"
        );
    }
}
