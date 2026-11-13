//! Tests for B12, the Unspooled (WP-080), and the scripted bot that beats it.
//!
//! The bot plays the fight as the spec describes it (docs/game/05-enemies-and-bosses.md, "B12 The
//! Unspooled"): it stands in front of the boss, rolls through each hit a few ticks before it lands
//! (the stitching grab included: it's unblockable), steps off the loom lines while they glow and
//! before they become beams, and strikes when no hit is coming. It is deterministic (no randomness,
//! no clock), and [`run_bot`] and [`run_idle`] play a whole fight from any seed.
//!
//! The goldens `m7_b12_bot_win.lfr` and `m7_b12_idle_dies.lfr` come from the bot's inputs:
//! `record_the_b12_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;

use super::b12::{self, B12Run, BORROWED, Line};
use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::arena::Id;
use crate::combat::hitbox::{Capsule, Pose, closest_points};
use crate::combat::moves::MoveNum;
use crate::events::{BossEvent, Event, LifeEvent, LoomEvent, MechanicEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::math::dmath::{cos, sin, sqrt};
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

const SCENARIO: &str = "boss_b12";
const BOSS_ID: &str = "b12_the_unspooled";
/// The seed the goldens use.
const SEED: u64 = 80;
/// The longest fight the runners play, in ticks (at 30 ticks a second, twenty minutes). This only
/// stops a stuck run.
const MAX_TICKS: u64 = 36_000;

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

/// How far `p` is from the loom line `l`, sideways, or infinity past its ends.
fn off_line(w: &World, l: &Line, p: [f32; 3]) -> f32 {
    let (rx, rz) = (p[0] - l.x, p[2] - l.z);
    let along = rx * l.dx + rz * l.dz;
    if along.abs() > number(w, "loom_half_length_m") {
        return f32::INFINITY;
    }
    (rx * l.dz - rz * l.dx).abs()
}

/// True if `p` stands clear of every line that is out, with `margin` meters to spare.
fn clear_of_lines(w: &World, p: [f32; 3], margin: f32) -> bool {
    let hw = number(w, "loom_half_width_m");
    w.b12
        .lines
        .iter()
        .filter(|l| l.is_out())
        .all(|l| off_line(w, l, p) > hw + margin)
}

/// The scripted player.
pub struct Bot {
    /// How many ticks before a hit's first frame the bot presses DODGE.
    lead: u16,
    /// The frame of the hit the bot already rolled through (in the move it is rolling in).
    rolled_for: Option<(MoveNum, u16)>,
    pending_hit: Option<(MoveNum, u16)>,
    /// Where the bot goes to be off the loom lines, while they are out.
    refuge: Option<[f32; 3]>,
    /// Buttons sent last tick, because every button is a press edge.
    last: u16,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            lead: 7,
            rolled_for: None,
            pending_hit: None,
            refuge: None,
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
            Stage::Dormant => return ((0, 127), 0), // through the fog gate, north
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
        if rolling && let Some(h) = self.pending_hit.take() {
            self.rolled_for = Some(h);
        }
        // Where the bot stands: in front of the boss, toward the player, a little farther off the
        // larger Unspooled.
        let reach = if w.bosses[0].phase >= 2 { 2.9 } else { 2.2 };
        let fwd = [sin(body.yaw), cos(body.yaw)];
        let post = [
            body.pos[0] + fwd[0] * reach,
            pp[1],
            body.pos[2] + fwd[1] * reach,
        ];

        // 1. The loom lines: while any is out and the bot stands on one, go to the nearest spot
        // clear of them all.
        let lines_out = w.b12.lines.iter().any(Line::is_out);
        if !lines_out {
            self.refuge = None;
        }
        if lines_out && (self.refuge.is_some() || !clear_of_lines(w, pp, 0.5)) {
            let refuge = match self.refuge {
                Some(r) if clear_of_lines(w, r, 0.5) => r,
                _ => {
                    let r = refuge(w, pp, body.pos);
                    self.refuge = Some(r);
                    r
                }
            };
            let soon = w
                .b12
                .lines
                .iter()
                .filter(|l| {
                    l.is_out() && off_line(w, l, pp) <= number(w, "loom_half_width_m") + 0.4
                })
                .any(|l| l.beam > 0 || l.warn <= 5);
            // A beam about to fire under the bot: roll through it.
            if soon && !rolling && player.stamina >= 25.0 {
                return (roll_toward(w, pp, refuge), buttons::DODGE);
            }
            if dist(pp, refuge) > 0.3 {
                // Still roll a hit on the way.
                if let Some(r) = self.roll_hit(w, body, pp, rolling, refuge) {
                    return r;
                }
                return (toward(pp, refuge, 1.0), 0);
            }
        }
        // 2. Roll through a hit that is about to land, toward the post.
        if let Some(r) = self.roll_hit(w, body, pp, rolling, post) {
            return r;
        }
        if lines_out && self.refuge.is_some() {
            return ((0, 0), 0);
        }
        // 3. Drink a flask in a long recovery, when hurt.
        if player.hp * 100 < player.max_hp * 50
            && player.inventory.flasks.vigil > 0
            && recovery_left(w, body) >= 36
            && player.moves.current.is_none()
        {
            return ((0, 0), buttons::USE_ITEM);
        }
        // 4. Get to the post, and strike when no hit is coming.
        if dist(pp, post) > 0.6 {
            return (toward(pp, post, 1.0), 0);
        }
        let hit = threat(w, body, pp);
        if hit.is_none() && player.stamina >= 30.0 && player.moves.current.is_none() {
            return (toward(pp, body.pos, 0.1), buttons::LIGHT);
        }
        ((0, 0), 0)
    }

    /// Presses DODGE toward `goal` when one of the boss's hits is about to land on the bot.
    fn roll_hit(
        &mut self,
        w: &World,
        body: &Actor,
        pp: [f32; 3],
        rolling: bool,
        goal: [f32; 3],
    ) -> Option<((i8, i8), u16)> {
        let Some(m) = body.moves.current else {
            self.rolled_for = None;
            return None;
        };
        if self.rolled_for.is_some_and(|(n, _)| n != m.num) {
            self.rolled_for = None;
        }
        let first = threat(w, body, pp)?;
        let player = w.actors.get(w.player)?;
        if !rolling
            && self.rolled_for != Some((m.num, first))
            && m.frame < first
            && first - m.frame <= self.lead
            && player.stamina >= 25.0
        {
            self.pending_hit = Some((m.num, first));
            // A long spell of hits (the spin) outlasts a roll's i-frames: roll out of its reach.
            let goal = if threat_last(w, body, pp).is_some_and(|l| l > first + 9) {
                let d = dist(pp, body.pos).max(0.001);
                let away = [(pp[0] - body.pos[0]) / d, (pp[2] - body.pos[2]) / d];
                [pp[0] + away[0] * 4.0, pp[1], pp[2] + away[1] * 4.0]
            } else {
                goal
            };
            return Some((roll_toward(w, pp, goal), buttons::DODGE));
        }
        None
    }
}

/// How far a roll carries the bot, about.
const ROLL_M: f32 = 3.4;

/// The stick for a roll toward `goal` that ends clear of the loom lines that are out (the roll's
/// i-frames cover the way, not the landing).
fn roll_toward(w: &World, pp: [f32; 3], goal: [f32; 3]) -> (i8, i8) {
    if !w.b12.lines.iter().any(Line::is_out) {
        return toward(pp, goal, 1.0);
    }
    let d = dist(pp, goal).max(0.001);
    let want = [(goal[0] - pp[0]) / d, (goal[2] - pp[2]) / d];
    let mut best: Option<([f32; 3], f32)> = None;
    for k in 0..24 {
        let a = k as f32 * core::f32::consts::TAU / 24.0;
        let dir = [sin(a), cos(a)];
        let land = [pp[0] + dir[0] * ROLL_M, pp[1], pp[2] + dir[1] * ROLL_M];
        if !clear_of_lines(w, land, 0.6) || !in_arena(land) {
            continue;
        }
        let score = dir[0] * want[0] + dir[1] * want[1];
        if best.is_none_or(|(_, s)| score > s) {
            best = Some((land, score));
        }
    }
    best.map_or(toward(pp, goal, 1.0), |(land, _)| toward(pp, land, 1.0))
}

/// True if `p` is on the roof, away from the rim and out of the corridor.
fn in_arena(p: [f32; 3]) -> bool {
    p[0].abs() <= 16.0 && p[2] <= 16.0 && p[2] >= -4.5
}

/// The nearest spot to `pp` clear of every loom line that is out, not on top of the boss.
fn refuge(w: &World, pp: [f32; 3], boss: [f32; 3]) -> [f32; 3] {
    let mut best = pp;
    let mut best_d = f32::INFINITY;
    for ring in 1..=8 {
        let r = ring as f32 * 0.5;
        for k in 0..24 {
            let a = k as f32 * core::f32::consts::TAU / 24.0;
            let p = [pp[0] + r * sin(a), pp[1], pp[2] + r * cos(a)];
            if !clear_of_lines(w, p, 0.7) || dist(p, boss) < 2.2 || !in_arena(p) {
                continue;
            }
            if r < best_d {
                best_d = r;
                best = p;
            }
        }
        if best_d.is_finite() {
            break;
        }
    }
    best
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
    const MARGIN_M: f32 = 1.0;
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

/// The last active frame of the boss's hits still to come that would reach `pp`.
fn threat_last(w: &World, body: &Actor, pp: [f32; 3]) -> Option<u16> {
    const MARGIN_M: f32 = 1.0;
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
        .map(|h| h.frames.1)
        .max()
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

fn actor(w: &World, id: Id) -> &Actor {
    w.actors.get(id).unwrap()
}

/// A trace of the bot's fight, for tuning: `cargo test -p lf-core trace_the_b12_bot -- --ignored
/// --nocapture`. Prints the state every 120 ticks and every hit on the player.
#[test]
#[ignore]
fn trace_the_b12_bot() {
    let mut bot = Bot::default();
    let content = Content::load_embedded().unwrap();
    let seed = std::env::var("B12_SEED").map_or(SEED, |s| s.parse().unwrap());
    let mut w = World::new(seed, &content, SCENARIO).unwrap();
    let (mut over, mut won) = (0, false);
    while w.tick < MAX_TICKS && over < 30 {
        if w.tick.is_multiple_of(120) {
            let p = actor(&w, w.player);
            let b = actor(&w, body_of(&w));
            eprintln!(
                "t{} {:?} ph{} player hp {} st {:.0} ({:.1},{:.1}) body hp {} ({:.1},{:.1}) mv {:?}",
                w.tick,
                w.bosses[0].stage,
                w.bosses[0].phase,
                p.hp,
                p.stamina,
                p.pos[0],
                p.pos[2],
                b.hp,
                b.pos[0],
                b.pos[2],
                b.moves.current.map(|m| (m.num, m.frame)),
            );
        }
        let frame = bot.act(&w);
        if std::env::var("B12_DEBUG").is_ok_and(|v| {
            let mut it = v.split(',').map(|x| x.parse::<u64>().unwrap());
            let (a, b) = (it.next().unwrap(), it.next().unwrap());
            w.tick >= a && w.tick <= b
        }) {
            let p = actor(&w, w.player);
            let lines: Vec<_> = w
                .b12
                .lines
                .iter()
                .filter(|l| l.is_out())
                .map(|l| {
                    (
                        l.warn,
                        l.beam,
                        (l.x * 10.0).round() / 10.0,
                        (l.z * 10.0).round() / 10.0,
                    )
                })
                .collect();
            eprintln!(
                "    t{} b ({:.2},{:.2} y{:.2} {:?}) p ({:.2},{:.2}) st {:.0} mv {:?} if {} refuge {:?} in {:?} lines {:?}",
                w.tick,
                actor(&w, body_of(&w)).pos[0],
                actor(&w, body_of(&w)).pos[2],
                actor(&w, body_of(&w)).yaw,
                actor(&w, body_of(&w)).moves.current.map(|m| m.frame),
                p.pos[0],
                p.pos[2],
                p.stamina,
                p.moves.current.map(|m| (m.num, m.frame)),
                p.moves.flags & crate::combat::moves::flags::IFRAMES != 0,
                bot.refuge,
                frame,
                lines
            );
        }
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
                Event::Mechanic(m) => eprintln!("  t{} MECH {} {}", w.tick, m.kind, m.index),
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
            ..ActionFrame::default()
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

fn boss_kinds(events: &[(u64, Event)]) -> Vec<(u8, u8)> {
    events
        .iter()
        .filter_map(|(_, e)| match e {
            Event::Boss(b) => Some((b.kind, b.phase)),
            _ => None,
        })
        .collect()
}

/// Kills the body outright, as a last hit would.
fn kill_body(w: &mut World) {
    let id = body_of(w);
    let a = w.actors.get_mut(id).unwrap();
    a.hp = 0;
    a.dead = true;
}

/// Makes the body start move `name` now, as the framework would.
fn start_move(w: &mut World, name: &str) {
    let num = w.moves.num_of(name).unwrap();
    let d = w.moves.get(num).unwrap();
    let total = d.data.total_frames;
    let id = body_of(w);
    let a = w.actors.get_mut(id).unwrap();
    a.moves.current = Some(crate::combat::moves::ActiveMove {
        num,
        frame: 0,
        charge_ticks: 0,
        hold_button: 0,
        free_from: total,
    });
}

#[test]
fn the_first_death_is_a_rebirth_with_full_hp_and_the_second_is_the_defeat() {
    let mut w = fight_world();
    let id = body_of(&w);
    kill_body(&mut w);
    let events = run_ticks(&mut w, 1, true, |_| ActionFrame::default());
    // It stood back up, and phase 2's transition began instead of a defeat.
    assert_eq!(boss_kinds(&events), [(BossEvent::PHASE_STARTED, 2)]);
    assert!(!actor(&w, id).dead);
    let unravel = w.moves.num_of("b12_unravel").unwrap();
    assert_eq!(actor(&w, id).moves.current.map(|c| c.num), Some(unravel));
    // The HP fills over the 45 invulnerable ticks, and phase 2 starts at the full 8,100.
    let mut hp = Vec::new();
    let mut events = Vec::new();
    for _ in 0..46 {
        events.extend(run_ticks(&mut w, 1, true, |_| ActionFrame::default()));
        hp.push(actor(&w, id).hp);
    }
    assert!(hp.windows(2).all(|p| p[1] >= p[0]), "{hp:?}");
    assert_eq!(w.bosses[0].phase, 2);
    assert_eq!(actor(&w, id).hp, 8100);
    assert_eq!(actor(&w, id).max_hp, 8100);
    assert_eq!(mech(&events, MechanicEvent::UNSPOOLED_REBORN).len(), 1);
    assert!(w.b12.reborn);
    // The second death ends the fight and pays the ending's letter.
    kill_body(&mut w);
    let events = run_ticks(&mut w, 2, true, |_| ActionFrame::default());
    assert_eq!(boss_kinds(&events), [(BossEvent::DEFEATED, 0)]);
    assert_eq!(w.bosses[0].stage, Stage::Defeated);
    assert!(w.state.flags.is_set(&flags::boss_defeated(BOSS_ID)));
    let letter = w.items.num_of("itm_couriers_letter").unwrap();
    assert_eq!(actor(&w, w.player).inventory.count(letter), 1);
    assert!(w.b12.is_inert(), "the fight is over: rest");
}

#[test]
fn the_unspooled_is_bigger_than_the_weaver() {
    let mut w = fight_world();
    let id = body_of(&w);
    let small = actor(&w, id).hurtbox;
    assert_eq!(small.radius, number(&w, "p1_body_radius_m"));
    kill_body(&mut w);
    run_ticks(&mut w, 50, true, |_| ActionFrame::default());
    let big = actor(&w, id).hurtbox;
    assert_eq!(big.radius, number(&w, "p2_body_radius_m"));
    assert!(big.radius > small.radius);
}

#[test]
fn phase_2_borrows_only_the_moves_of_beaten_great_sigil_bosses() {
    // The scenario's player holds the Choir's and the Castellan's Sigils; the Matriarch's flag is
    // up as if she were beaten in an earlier visit.
    let mut w = fight_world();
    w.state
        .flags
        .raise(&flags::boss_defeated("b09_lamplight_matriarch"));
    assert_eq!(b12::beaten_mask(&w), 0b0111);
    kill_body(&mut w);
    run_ticks(&mut w, 50, true, |_| ActionFrame::default());
    assert_eq!(w.b12.borrowed, 0b0111);
    // The Bellwright's option stays on cooldown; the others are free to come up.
    let opts = &w.layout.bosses[0].def.body.options;
    let brain = actor(&w, body_of(&w)).ai.as_ref().unwrap();
    for (i, (name, _, _)) in BORROWED.iter().enumerate() {
        let o = opts
            .iter()
            .position(|o| o.move_id.as_deref() == Some(*name))
            .unwrap();
        let held = brain.cooldowns[o] > u16::MAX - 100;
        assert_eq!(held, i == 3, "{name}");
    }
    // Over a long phase 2 with the player close, the borrowed moves it may use show up, and the
    // Bellwright's never does.
    let flail = w.moves.num_of("b10_flail_slam").unwrap();
    let mut used = std::collections::BTreeSet::new();
    for _ in 0..6000 {
        let at = actor(&w, body_of(&w)).pos;
        let yaw = actor(&w, body_of(&w)).yaw;
        let r = 3.5 + 2.5 * sin(w.tick as f32 * 0.01);
        put_player(&mut w, [at[0] + r * sin(yaw), 0.0, at[2] + r * cos(yaw)]);
        run_ticks(&mut w, 1, true, |_| ActionFrame::default());
        if let Some(c) = actor(&w, body_of(&w)).moves.current {
            used.insert(c.num);
        }
    }
    assert!(!used.contains(&flail), "a masked move was used");
    let leap = w.moves.num_of("b07_leap_slam").unwrap();
    let flare = w.moves.num_of("b09_gnd_flare").unwrap();
    assert!(used.contains(&leap) && used.contains(&flare), "{used:?}");
}

#[test]
fn without_any_great_sigil_phase_2_has_only_its_own_moves() {
    let mut w = fight_world();
    let p = w.actors.get_mut(w.player).unwrap();
    for (_, _, sigil) in BORROWED {
        let n = w.items.num_of(sigil).unwrap();
        let had = p.inventory.count(n);
        if had > 0 {
            p.inventory.remove(n, had);
        }
    }
    assert_eq!(b12::beaten_mask(&w), 0);
    kill_body(&mut w);
    run_ticks(&mut w, 50, true, |_| ActionFrame::default());
    assert_eq!(w.b12.borrowed, 0);
}

#[test]
fn loom_lines_glow_for_30_ticks_then_burn_a_player_who_stays() {
    let mut w = fight_world();
    let at = actor(&w, body_of(&w)).pos;
    let spot = [at[0], 0.0, at[2] - 6.0];
    put_player(&mut w, spot);
    start_move(&mut w, "b12_loom");
    let mut events = Vec::new();
    let mut hp = Vec::new();
    for _ in 0..60 {
        put_player(&mut w, spot);
        events.extend(run_ticks(&mut w, 1, false, |_| ActionFrame::default()));
        hp.push(actor(&w, w.player).hp);
    }
    let loom: Vec<(u64, u8, u16, u16)> = events
        .iter()
        .filter_map(|(t, e)| match e {
            Event::Loom(l) => Some((*t, l.what, l.slot, l.ticks)),
            _ => None,
        })
        .collect();
    let laid: Vec<_> = loom
        .iter()
        .filter(|l| l.1 == LoomEvent::LAID)
        .copied()
        .collect();
    let beams: Vec<_> = loom
        .iter()
        .filter(|l| l.1 == LoomEvent::BEAM)
        .copied()
        .collect();
    assert_eq!(laid.len(), 3, "{loom:?}");
    assert_eq!(beams.len(), 3, "{loom:?}");
    // The spec's 30 ticks between the warning and the beam.
    assert_eq!(beams[0].0 - laid[0].0, 30);
    assert!(laid.iter().all(|l| l.3 == 30));
    // The middle line ran through the player: one hit for the cast.
    let hits = mech(&events, MechanicEvent::LOOM_BEAM_HIT);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].0 >= beams[0].0);
    let max = actor(&w, w.player).max_hp;
    assert!(hp.last().copied().unwrap() < max);
}

#[test]
fn a_player_between_the_lines_or_rolling_is_not_burned() {
    for roll in [false, true] {
        let mut w = fight_world();
        let at = actor(&w, body_of(&w)).pos;
        let spot = [at[0], 0.0, at[2] - 6.0];
        put_player(&mut w, spot);
        start_move(&mut w, "b12_loom");
        let mut events = Vec::new();
        for t in 0..60u32 {
            if !roll {
                // Two meters back off the middle line, between it and the next.
                if t > 8 {
                    put_player(&mut w, [spot[0], 0.0, spot[2] - 2.0]);
                }
                events.extend(run_ticks(&mut w, 1, false, |_| ActionFrame::default()));
            } else {
                put_player(&mut w, spot);
                // Roll as the beam lights: the press reaches the roll a tick later, and its
                // i-frames (from its frame 2) cover the beam.
                let fire = w.b12.lines.iter().any(|l| l.warn == 5);
                events.extend(run_ticks(&mut w, 1, false, |_| ActionFrame {
                    move_x: 127,
                    move_z: 0,
                    buttons: if fire { buttons::DODGE } else { 0 },
                }));
            }
        }
        let hits = mech(&events, MechanicEvent::LOOM_BEAM_HIT);
        let loom: Vec<_> = events
            .iter()
            .filter_map(|(t, e)| match e {
                Event::Loom(l) => Some((*t, l.what, l.center)),
                _ => None,
            })
            .collect();
        assert!(hits.is_empty(), "roll {roll}: {hits:?} {loom:?}");
    }
}

#[test]
fn the_loom_storm_lays_a_grid_both_ways() {
    let mut w = fight_world();
    kill_body(&mut w);
    run_ticks(&mut w, 50, true, |_| ActionFrame::default());
    let at = actor(&w, body_of(&w)).pos;
    put_player(&mut w, [at[0], 0.0, at[2] - 6.0]);
    start_move(&mut w, "b12_loom_storm");
    run_ticks(&mut w, 8, true, |_| ActionFrame::default());
    let out: Vec<&Line> = w.b12.lines.iter().filter(|l| l.is_out()).collect();
    assert_eq!(out.len(), 6);
    // Three lines one way and three at right angles.
    let dot = out[0].dx * out[3].dx + out[0].dz * out[3].dz;
    assert!(dot.abs() < 1e-4, "{dot}");
}

#[test]
fn an_echo_out_at_the_rebirth_is_unspooled() {
    use crate::ai::echo::{Member, Summoned};
    let mut w = fight_world();
    // A stand-in Echo body: a copy of the boss's actor, named as the Echo that is out.
    let copy = actor(&w, body_of(&w)).clone();
    let ally = w.actors.insert(copy);
    w.echo.active = Some(Summoned {
        echo: 0,
        members: vec![Member {
            id: ally,
            fell: false,
        }],
        age: 0,
        bowed: false,
        heal_clock: 0,
        fallen_ticks: 0,
    });
    // In phase 1 the Echo fights on.
    let events = run_ticks(&mut w, 5, true, |_| ActionFrame::default());
    assert!(mech(&events, MechanicEvent::ECHO_UNSPOOLED).is_empty());
    assert!(!actor(&w, ally).dead);
    // The rebirth unspools it.
    kill_body(&mut w);
    let events = run_ticks(&mut w, 3, true, |_| ActionFrame::default());
    assert_eq!(mech(&events, MechanicEvent::ECHO_UNSPOOLED).len(), 1);
    assert!(actor(&w, ally).dead);
}

#[test]
fn the_players_death_resets_the_rebirth_and_the_lines() {
    let mut w = fight_world();
    kill_body(&mut w);
    run_ticks(&mut w, 50, true, |_| ActionFrame::default());
    assert!(w.b12.reborn);
    let at = actor(&w, body_of(&w)).pos;
    put_player(&mut w, [at[0], 0.0, at[2] - 6.0]);
    start_move(&mut w, "b12_loom_storm");
    run_ticks(&mut w, 10, true, |_| ActionFrame::default());
    assert!(w.b12.lines.iter().any(Line::is_out));
    w.actors.get_mut(w.player).unwrap().hp = 0;
    w.actors.get_mut(w.player).unwrap().dead = true;
    let events = run_ticks(&mut w, 5, false, |_| ActionFrame::default());
    assert!(
        events
            .iter()
            .any(|(_, e)| matches!(e, Event::Loom(l) if l.what == LoomEvent::CLEARED))
    );
    assert_eq!(w.b12, B12Run::default(), "the fight is over: rest");
    // The Weaver is back, at full HP and in its own size.
    let id = body_of(&w);
    assert_eq!(actor(&w, id).hp, 8100);
    assert_eq!(w.bosses[0].phase, 0);
}

#[test]
fn b12_state_is_in_the_state_hash_during_the_fight_and_out_of_it_at_rest() {
    let mut w = fight_world();
    kill_body(&mut w);
    run_ticks(&mut w, 3, true, |_| ActionFrame::default());
    assert!(!w.b12.is_inert(), "the fight is on");
    let mut other = w.clone();
    other.b12 = B12Run::default();
    assert_ne!(w.state_hash(), other.state_hash());
    w.actors.get_mut(w.player).unwrap().hp = 0;
    w.actors.get_mut(w.player).unwrap().dead = true;
    run_ticks(&mut w, 5, false, |_| ActionFrame::default());
    assert!(w.b12.is_inert());
}

#[test]
fn the_arena_in_the_level_has_the_boss_and_its_fog_gate() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(20, &content, "spire").unwrap();
    assert_eq!(w.layout.bosses.len(), 1);
    let lay = &w.layout.bosses[0];
    assert_eq!(lay.def.id, BOSS_ID);
    assert_eq!(lay.members.len(), 1);
    assert_eq!(lay.gate.pos, [6.5, 22.0, -5.25]);
    // The stairs below pass under the gate: a player a floor down doesn't walk through it.
    put_player(&mut w, [6.5, 17.0, -4.5]);
    run_ticks(&mut w, 2, true, |_| ActionFrame::default());
    assert_eq!(w.bosses[0].stage, Stage::Dormant);
    // The player through the gate seals it, and the fight starts after the intro.
    put_player(&mut w, [6.5, 22.0, -4.5]);
    run_ticks(&mut w, 5, true, |_| ActionFrame::default());
    assert!(matches!(w.bosses[0].stage, Stage::Intro { .. }));
    run_ticks(&mut w, 95, true, |_| ActionFrame::default());
    assert_eq!(w.bosses[0].stage, Stage::Fight);
    // A player who steps back out is put back on the gate's plane.
    put_player(&mut w, [6.5, 22.0, -7.0]);
    run_ticks(&mut w, 1, true, |_| ActionFrame::default());
    assert!(actor(&w, w.player).pos[2] >= -5.3);
}

// ---------------------------------------------------------------------------------------
// The fight
// ---------------------------------------------------------------------------------------

#[test]
fn the_scripted_bot_beats_b12_through_both_lives() {
    let f = run_bot(SEED);
    assert!(f.boss_defeated(), "the bot lost");
    assert!(!f.player_died());
    assert_eq!(f.boss_events(BossEvent::PHASE_STARTED), 1);
    assert_eq!(f.mechanic_count(MechanicEvent::UNSPOOLED_REBORN), 1);
    let laid = f
        .events
        .iter()
        .filter(|e| matches!(e, Event::Loom(l) if l.what == LoomEvent::LAID))
        .count();
    assert!(laid >= 3, "no loom lines");
    let inv = &f.world.actors.get(f.world.player).unwrap().inventory;
    let content = Content::load_embedded().unwrap();
    let letter = content.registry.num("itm_couriers_letter").unwrap();
    assert_eq!(inv.count(letter), 1, "no Courier's Letter");
    assert_eq!(inv.glimmer, 8000);
}

#[test]
fn a_player_who_does_nothing_dies_to_b12() {
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
fn the_bot_golden_replay_beats_b12_with_the_player_alive() {
    let f = play_golden("m7_b12_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b12_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b12_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b12_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b12_goldens` in game/crates/core/src/ai/bosses/b12_tests.rs.\n\
             # Each line: <ticks> <move_x> <move_z> [BUTTONS].\nseed {SEED}\nscenario {SCENARIO}\n"
        );
        for (frame, n) in crate::replay::compress(&f.frames) {
            let mut names = Vec::new();
            for (mask, name) in [
                (buttons::LIGHT, "LIGHT"),
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
        "m7_b12_bot_win",
        "WP-080 golden: the scripted bot beats B12 through both lives: it rolls each hit, steps off the loom lines, and strikes in the gaps.",
        &run_bot(SEED),
    );
    write(
        "m7_b12_idle_dies",
        "WP-080 golden: a player who walks in and does nothing dies to B12.",
        &run_idle(SEED),
    );
}
