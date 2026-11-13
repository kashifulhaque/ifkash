//! Tests for B09, the Lamplight Matriarch (WP-077), and the scripted bot that beats her.
//!
//! The bot plays the fight as the spec describes it (docs/game/05-enemies-and-bosses.md, "B09 The
//! Lamplight Matriarch"): it rolls out of each telegraphed hit (a dive's shadow, the dust, a beam,
//! the pass, a buffet, the lunge, the spin), works at her with the Lantern Sword whenever she is
//! low enough to reach (the ground phase and the foot of a dive), and, once the cavern is drained
//! in phase 2, uses the time she spends in the air to light the lamps with the sword's Fire. It is
//! deterministic (no randomness, no clock), and [`run_bot`] and [`run_idle`] play a whole fight
//! from any seed.
//!
//! The goldens `m7_b09_bot_win.lfr` and `m7_b09_idle_dies.lfr` come from the bot's inputs:
//! `record_the_b09_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;

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

use super::b09::State;

const SCENARIO: &str = "boss_b09";
const BOSS_ID: &str = "b09_lamplight_matriarch";
/// The seed the goldens use.
const SEED: u64 = 77;
/// The longest fight the runners play, in ticks (at 30 ticks a second, thirteen minutes). This only
/// stops a stuck run.
const MAX_TICKS: u64 = 24_000;
/// Where the six lamps stand: the boss's spawn (112, 120) plus the offsets in her definition.
const LAMPS: [[f32; 2]; 6] = [
    [132.0, 120.0],
    [122.0, 137.32],
    [102.0, 137.32],
    [92.0, 120.0],
    [102.0, 102.68],
    [122.0, 102.68],
];

fn body_of(w: &World) -> Id {
    let spawn = w.layout.bosses[0].members[0];
    w.state.ai_enemies[usize::from(spawn)]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    sqrt((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2))
}

/// The stick (-127 to 127) that points along `(dx, dz)`, scaled to `power` (0 to 1).
fn stick(dx: f32, dz: f32, power: f32) -> (i8, i8) {
    let d = sqrt(dx * dx + dz * dz).max(0.001);
    let s = |v: f32| (v / d * 127.0 * power) as i8;
    (s(dx), s(dz))
}

fn toward(from: [f32; 3], to: [f32; 3], power: f32) -> (i8, i8) {
    stick(to[0] - from[0], to[2] - from[2], power)
}

/// The scripted player.
pub struct Bot {
    /// How many ticks before a hit's first frame the bot presses DODGE.
    lead: u16,
    seen: Option<(MoveNum, u16)>,
    rolled: bool,
    /// Buttons sent last tick, because every button is a press edge.
    last: u16,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            lead: 8,
            seen: None,
            rolled: false,
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

        // The boss's move, and when it would first hit the bot where it stands.
        let mut frame = 0u16;
        if let Some(m) = body.moves.current {
            frame = m.frame;
            let key = (m.num, m.frame);
            if self.seen.is_none_or(|(n, f)| n != key.0 || f > key.1) {
                self.rolled = false;
            }
            self.seen = Some(key);
        } else {
            self.seen = None;
            self.rolled = false;
        }
        let hit = threat(w, body, pp);
        if player
            .moves
            .current
            .and_then(|c| w.moves.player_moveset.as_ref().map(|s| (c.num, s)))
            .is_some_and(|(n, s)| n == s.roll_light || n == s.roll_medium || n == s.roll_heavy)
        {
            self.rolled = true;
        }

        // 1. Roll out of a hit that is about to land.
        if let Some((first, out)) = hit
            && !self.rolled
            && frame < first
            && first - frame <= self.lead
            && player.stamina >= 25.0
        {
            return (stick(out[0], out[1], 1.0), buttons::DODGE);
        }
        // 2. Drink a flask in a long recovery, when hurt.
        if player.hp * 100 < player.max_hp * 50
            && player.inventory.flasks.vigil > 0
            && recovery_left(w, body) >= 36
            && player.moves.current.is_none()
        {
            return ((0, 0), buttons::USE_ITEM);
        }
        // 3. Work at her when she is low enough to reach; otherwise, in phase 2, light a lamp.
        let low = body.pos[1] - body.motor.ground_y <= 1.6;
        let st = w.b09;
        let (goal, reach) = if low {
            (Some(body.pos), 2.0)
        } else if st.dark {
            (
                (0..6)
                    .filter(|&i| st.lamps & (1 << i) == 0)
                    .map(|i| [LAMPS[i][0], 0.0, LAMPS[i][1]])
                    .min_by(|a, b| dist(pp, *a).total_cmp(&dist(pp, *b))),
                1.5,
            )
        } else {
            (None, 0.0)
        };
        let Some(goal) = goal else {
            return ((0, 0), 0);
        };
        if dist(pp, goal) > reach {
            return (toward(pp, goal, 1.0), 0);
        }
        if hit.is_none() && player.stamina >= 30.0 && player.moves.current.is_none() {
            return (toward(pp, goal, 0.1), buttons::LIGHT);
        }
        ((0, 0), 0)
    }
}

/// Ticks left of the body's move once its last hit is over, if the move ends the combo; 0
/// otherwise (and while it isn't in a move, which is no time to drink).
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

/// Meters the move has carried the actor forward by `frame` (the move's `root_motion` keys are
/// cumulative).
fn root_at(keys: &[(u16, f32)], frame: u16) -> f32 {
    let mut prev = (0u16, 0.0f32);
    for &(f, d) in keys {
        if frame <= f {
            let span = f32::from((f - prev.0).max(1));
            return prev.1 + (d - prev.1) * f32::from(frame - prev.0.min(frame)) / span;
        }
        prev = (f, d);
    }
    prev.1
}

/// The first frame at which the boss's current move would hit a player standing at `pp`, and
/// the direction (on the ground plane) that leads out of it: the earliest of its hitboxes still
/// to come that reach `pp` (with a margin, for the body turning and the player moving), laid
/// where the body will be by then. `None` if nothing would.
fn threat(w: &World, body: &Actor, pp: [f32; 3]) -> Option<(u16, [f32; 2])> {
    /// Meters added to each hitbox's radius.
    const MARGIN_M: f32 = 0.9;
    let m = body.moves.current?;
    let def = w.moves.get(m.num)?;
    let flat = |v: [f32; 3]| [v[0], 0.0, v[2]];
    let mut best: Option<(u16, [f32; 2])> = None;
    for h in def.data.hitboxes.iter().filter(|h| h.frames.1 >= m.frame) {
        // Where the body will be when this hitbox starts.
        let ahead = root_at(&def.data.root_motion, h.frames.0.max(m.frame))
            - root_at(&def.data.root_motion, m.frame);
        let pose = Pose {
            pos: [
                body.pos[0] + sin(body.yaw) * ahead,
                body.pos[1],
                body.pos[2] + cos(body.yaw) * ahead,
            ],
            yaw: body.yaw,
        };
        let c = Capsule::from_shape(&h.shape, &pose);
        let (near, _, d2) = closest_points(flat(c.a), flat(c.b), flat(pp), flat(pp));
        if sqrt(d2) > c.radius + MARGIN_M {
            continue;
        }
        if best.is_none_or(|(f, _)| h.frames.0 < f) {
            // Out of the hitbox: away from its nearest point (or along the body's facing if the
            // bot stands on it).
            let (dx, dz) = (pp[0] - near[0], pp[2] - near[2]);
            let out = if dx * dx + dz * dz < 0.01 {
                [sin(body.yaw), cos(body.yaw)]
            } else {
                [dx, dz]
            };
            best = Some((h.frames.0, out));
        }
    }
    best
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
            let st = w.b09;
            eprintln!(
                "t{} {:?} ph{} player hp {} st {:.0} ({:.1},{:.1}) body hp {} y {:.1} kneel {} stance {}/{} mv {:?} ground {} dives {} lamps {:06b}",
                w.tick,
                w.bosses[0].stage,
                w.bosses[0].phase,
                p.hp,
                p.stamina,
                p.pos[0],
                p.pos[2],
                b.hp,
                b.pos[1],
                b.poise.kneel,
                b.poise.stance_dmg,
                b.poise.stance,
                b.moves
                    .current
                    .and_then(|m| w.moves.get(m.num).map(|d| (d.data.id.clone(), m.frame))),
                st.ground,
                st.dives,
                st.lamps,
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
                Event::Critical(c) => eprintln!("  t{} CRIT {:?}", w.tick, c.kind),
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

fn idle(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut out = Vec::new();
    for _ in 0..ticks {
        tick(w, &TickInput::default());
        out.extend(w.events.drain());
    }
    out
}

fn fight_world_with(content: &Content) -> World {
    let mut w = World::new(SEED, content, SCENARIO).unwrap();
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

/// A world with the fight on: the player walked through the gate and the intro is over.
fn fight_world() -> World {
    fight_world_with(&Content::load_embedded().unwrap())
}

/// The same, with a player who can't die and stands far from the boss, so a test can watch
/// her fly and land without a fight.
fn watch_world() -> World {
    let mut w = fight_world();
    let p = w.actors.get_mut(w.player).unwrap();
    p.hp = 1_000_000;
    p.max_hp = 1_000_000;
    w
}

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.motor.ground_y = pos[1];
    p.motor.fall_peak_y = pos[1];
}

fn mechanics(events: &[Event], kind: u8) -> Vec<u16> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Mechanic(m) if m.kind == kind => Some(m.index),
            _ => None,
        })
        .collect()
}

fn move_id(w: &World, a: &Actor) -> Option<String> {
    let c = a.moves.current?;
    Some(w.moves.get(c.num)?.data.id.clone())
}

/// Starts phase 2: the body just under 50 percent, and ticks until the drain is on.
fn phase_two(w: &mut World) -> Vec<Event> {
    let body = body_of(w);
    w.actors.get_mut(body).unwrap().hp = 3200;
    let mut events = Vec::new();
    for _ in 0..80 {
        events.extend(idle(w, 1));
        if w.bosses[0].phase == 2 && w.b09.dark {
            break;
        }
    }
    assert_eq!(w.bosses[0].phase, 2);
    assert!(w.b09.dark, "the light is drained");
    events
}

#[test]
fn the_matriarch_is_a_flying_boss_with_a_big_body() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    tick(&mut w, &TickInput::default());
    let body = actor(&w, body_of(&w));
    assert!(body.hurtbox.radius > 1.0, "bigger than a person");
    assert_eq!(body.max_hp, 6500);
    assert_eq!(w.layout.bosses[0].def.id, BOSS_ID);
    assert_eq!(w.layout.bosses[0].def.phases[0].below, 0.5);
    assert!(
        crate::ai::special::flying::flies(&w, body_of(&w)),
        "she flies"
    );
    // No state until the fight begins, and the light is the area's own.
    assert_eq!(w.b09, State::default());
    assert_eq!(super::b09::light_at(&w, 112.0, 120.0), None);
}

#[test]
fn she_rises_for_the_air_phase_and_only_air_moves_start_while_she_flies() {
    let mut w = watch_world();
    let id = body_of(&w);
    let (base, mut peak) = (actor(&w, id).motor.ground_y, 0.0f32);
    // (the move that started, whether she was walking when it did)
    let mut started: Vec<(String, bool)> = Vec::new();
    let mut last = None;
    let mut events = Vec::new();
    for _ in 0..4000 {
        events.extend(idle(&mut w, 1));
        let a = actor(&w, id);
        peak = peak.max(a.pos[1] - base);
        let now = move_id(&w, a);
        if now != last
            && let Some(n) = &now
        {
            started.push((n.clone(), w.b09.ground));
        }
        last = now;
    }
    assert!(peak > 3.0, "she rose to the air height: {peak}");
    assert!(!started.is_empty());
    for (id, ground) in &started {
        if id.starts_with("b09_air_") {
            assert!(!ground, "{id} started on the ground");
        }
        if id.starts_with("b09_gnd_") {
            assert!(*ground, "{id} started in the air");
        }
    }
    // The cycle came round: she landed after her dives, walked, and took off again.
    let landed = mechanics(&events, MechanicEvent::MATRIARCH_LANDED);
    assert!(landed.contains(&super::b09::LANDED_DIVES), "{landed:?}");
    assert!(!mechanics(&events, MechanicEvent::MATRIARCH_TOOK_FLIGHT).is_empty());
    assert!(started.iter().any(|(id, _)| id.starts_with("b09_air_dive")));
    assert!(started.iter().any(|(id, _)| id.starts_with("b09_gnd_")));
}

#[test]
fn a_dive_takes_her_to_the_floor_and_back_up() {
    let mut w = watch_world();
    let id = body_of(&w);
    let base = actor(&w, id).motor.ground_y;
    let (mut low, mut dived) = (f32::MAX, false);
    for _ in 0..3000 {
        idle(&mut w, 1);
        let a = actor(&w, id);
        if move_id(&w, a).is_some_and(|m| m.starts_with("b09_air_dive")) {
            dived = true;
            low = low.min(a.pos[1] - base);
        }
        if dived && a.moves.current.is_none() && a.pos[1] - base > 3.0 {
            break;
        }
    }
    assert!(dived, "she never dived");
    assert!(low < 0.6, "the dive reached the floor: {low}");
}

#[test]
fn she_lands_after_four_dives() {
    let mut w = watch_world();
    let id = body_of(&w);
    let mut dives = 0u32;
    let mut was = false;
    let mut events = Vec::new();
    for _ in 0..4000 {
        events.extend(idle(&mut w, 1));
        let now = move_id(&w, actor(&w, id)).is_some_and(|m| m.starts_with("b09_air_dive"));
        if now && !was {
            dives += 1;
        }
        was = now;
        if w.b09.ground {
            break;
        }
    }
    assert!(w.b09.ground, "she never landed");
    assert_eq!(dives, 4, "four dives, then she lands");
    assert_eq!(
        mechanics(&events, MechanicEvent::MATRIARCH_LANDED),
        [super::b09::LANDED_DIVES]
    );
    // She settles to the floor.
    idle(&mut w, 30);
    let a = actor(&w, id);
    assert!(a.pos[1] - a.motor.ground_y < 0.8);
}

#[test]
fn a_stance_break_in_the_air_lands_her_at_once() {
    let mut w = watch_world();
    let id = body_of(&w);
    idle(&mut w, 60);
    assert!(!w.b09.ground);
    // The stance break as the combat code leaves it: she kneels for 60 ticks.
    w.actors.get_mut(id).unwrap().poise.kneel = 60;
    let events = idle(&mut w, 1);
    assert!(w.b09.ground);
    assert_eq!(
        mechanics(&events, MechanicEvent::MATRIARCH_LANDED),
        [super::b09::LANDED_STANCE]
    );
    let stunned = w.moves.num_of("b09_stunned");
    assert_eq!(actor(&w, id).moves.current.map(|m| m.num), stunned);
    // She comes down to the floor within a second, kneeling, open to a riposte.
    idle(&mut w, 20);
    let a = actor(&w, id);
    assert!(a.pos[1] - a.motor.ground_y < 0.8, "down: {}", a.pos[1]);
    assert!(a.poise.kneel > 0);
}

#[test]
fn she_stays_on_the_ground_for_the_ground_phase_then_takes_off() {
    let mut w = watch_world();
    let id = body_of(&w);
    idle(&mut w, 60);
    w.actors.get_mut(id).unwrap().poise.kneel = 60;
    idle(&mut w, 2);
    let from = w.b09.mode_from;
    assert!(w.b09.ground);
    // 590 ticks on, she is still walking; a while later she has taken off (she finishes the move
    // she is in first, and a player who stands under her keeps her busy).
    while w.tick < from + 590 {
        idle(&mut w, 1);
    }
    assert!(w.b09.ground);
    let mut flew = false;
    while w.tick < from + 1000 {
        idle(&mut w, 1);
        flew |= !w.b09.ground;
    }
    assert!(flew, "she took off after the ground phase");
}

#[test]
fn phase_two_drains_the_light_of_the_cavern() {
    let mut w = watch_world();
    // Before: the area's own light.
    assert_eq!(super::b09::light_at(&w, 112.0, 120.0), None);
    let events = phase_two(&mut w);
    assert_eq!(mechanics(&events, MechanicEvent::MATRIARCH_DRAINED), [0]);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Boss(b) if b.kind == BossEvent::PHASE_STARTED))
            .count(),
        1
    );
    // The middle of the cavern is dark now; the corridor outside it is not.
    assert_eq!(super::b09::light_at(&w, 112.0, 120.0), Some(false));
    assert_eq!(super::b09::light_at(&w, 104.0, 85.0), None);
    put_player(&mut w, [112.0, 0.0, 110.0]);
    idle(&mut w, 1);
    assert_eq!(
        crate::world_state::mechanics::darkness::visibility_m(&w),
        Some(6.0)
    );
    // She restarts the flight in the air.
    assert!(!w.b09.ground);
}

#[test]
fn a_lit_lamp_lights_the_ground_around_it() {
    let mut w = watch_world();
    phase_two(&mut w);
    w.b09.lamps = 1; // lamp 0, at (132, 120)
    assert_eq!(super::b09::light_at(&w, 126.0, 120.0), Some(true));
    assert_eq!(super::b09::light_at(&w, 112.0, 120.0), Some(false));
    put_player(&mut w, [128.0, 0.0, 120.0]);
    idle(&mut w, 1);
    assert_eq!(
        crate::world_state::mechanics::darkness::visibility_m(&w),
        None
    );
}

/// Walks the player next to lamp `i`, facing it, and returns the lamp's world position.
fn stand_by_lamp(w: &mut World, i: usize, gap: f32) -> [f32; 3] {
    let lamp = [LAMPS[i][0], 0.0, LAMPS[i][1]];
    // Toward the middle of the cavern from the lamp.
    let (dx, dz) = (112.0 - lamp[0], 120.0 - lamp[2]);
    let d = sqrt(dx * dx + dz * dz);
    let at = [lamp[0] + dx / d * gap, 0.0, lamp[2] + dz / d * gap];
    put_player(w, at);
    let p = w.actors.get_mut(w.player).unwrap();
    p.yaw = crate::math::dmath::atan2(lamp[0] - at[0], lamp[2] - at[2]);
    lamp
}

#[test]
fn the_smoltorch_lights_a_lamp_after_a_wait_beside_it() {
    let mut w = watch_world();
    phase_two(&mut w);
    stand_by_lamp(&mut w, 2, 1.5);
    // Without the torch, standing there does nothing.
    let events = idle(&mut w, 120);
    assert!(mechanics(&events, MechanicEvent::MATRIARCH_LAMP_LIT).is_empty());
    w.state.flags.raise("codex.projects.smoltorch");
    let events = idle(&mut w, 40);
    assert!(
        mechanics(&events, MechanicEvent::MATRIARCH_LAMP_LIT).is_empty(),
        "not yet: the hold is 45 ticks"
    );
    let events = idle(&mut w, 10);
    assert_eq!(mechanics(&events, MechanicEvent::MATRIARCH_LAMP_LIT), [2]);
    assert_eq!(w.b09.lamps, 1 << 2);
}

#[test]
fn a_walk_past_with_the_smoltorch_lights_nothing() {
    let mut w = watch_world();
    phase_two(&mut w);
    w.state.flags.raise("codex.projects.smoltorch");
    stand_by_lamp(&mut w, 2, 1.5);
    idle(&mut w, 20);
    put_player(&mut w, [112.0, 0.0, 110.0]);
    let events = idle(&mut w, 100);
    assert!(mechanics(&events, MechanicEvent::MATRIARCH_LAMP_LIT).is_empty());
    assert_eq!(w.b09.torch_ticks, 0, "the hold starts over");
}

#[test]
fn lantern_flare_lights_the_lamps_near_the_player() {
    let mut w = watch_world();
    phase_two(&mut w);
    // Between lamp 0 (132, 120) and lamp 1 (122, 137.32), 10 m from each; lamp 5 is 20 m off.
    put_player(&mut w, [127.0, 0.0, 128.66]);
    crate::world_state::mechanics::darkness::flare(&mut w, 600);
    let events = idle(&mut w, 2);
    assert_eq!(
        mechanics(&events, MechanicEvent::MATRIARCH_LAMP_LIT),
        [0, 1]
    );
    assert_eq!(w.b09.lamps, 0b000011);
    // The flare still burning doesn't light more as the player walks on.
    put_player(&mut w, [97.0, 0.0, 128.66]);
    let events = idle(&mut w, 20);
    assert!(mechanics(&events, MechanicEvent::MATRIARCH_LAMP_LIT).is_empty());
    // A second flare does.
    w.mech.flare_until = 0;
    crate::world_state::mechanics::darkness::flare(&mut w, 600);
    let events = idle(&mut w, 2);
    assert_eq!(
        mechanics(&events, MechanicEvent::MATRIARCH_LAMP_LIT),
        [2, 3]
    );
}

#[test]
fn nothing_lights_a_lamp_before_the_light_is_drained() {
    let mut w = watch_world();
    assert!(!w.b09.dark);
    stand_by_lamp(&mut w, 0, 4.0);
    w.state.flags.raise("codex.projects.smoltorch");
    crate::world_state::mechanics::darkness::flare(&mut w, 600);
    let events = idle(&mut w, 100);
    assert!(mechanics(&events, MechanicEvent::MATRIARCH_LAMP_LIT).is_empty());
    assert_eq!(w.b09.lamps, 0);
}

/// A world whose player wields the Lantern Sword (Fire).
fn sword_world() -> World {
    let mut content = Content::load_embedded().unwrap();
    content
        .scenarios
        .get_mut("scenarios/boss_b09.ron")
        .unwrap()
        .player_weapon = Some("wpn_lantern_sword".to_owned());
    let mut w = fight_world_with(&content);
    let p = w.actors.get_mut(w.player).unwrap();
    p.hp = 1_000_000;
    p.max_hp = 1_000_000;
    w
}

#[test]
fn a_swing_of_the_lantern_sword_lights_the_lamp_it_strikes() {
    let mut w = sword_world();
    phase_two(&mut w);
    stand_by_lamp(&mut w, 3, 1.4);
    let mut events = Vec::new();
    for _ in 0..30 {
        let frame = ActionFrame {
            buttons: buttons::LIGHT,
            ..ActionFrame::default()
        };
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        events.extend(w.events.drain());
        let off = ActionFrame::default();
        tick(
            &mut w,
            &TickInput {
                frame: off,
                commands: Vec::new(),
            },
        );
        events.extend(w.events.drain());
        if w.b09.lamps != 0 {
            break;
        }
    }
    assert_eq!(
        mechanics(&events, MechanicEvent::MATRIARCH_LAMP_LIT),
        [3],
        "the sword's Fire lit lamp 3"
    );
}

#[test]
fn a_swing_that_misses_the_lamp_lights_nothing() {
    let mut w = sword_world();
    phase_two(&mut w);
    // Facing the lamp from 6 m: a sword's reach doesn't get there.
    stand_by_lamp(&mut w, 3, 6.0);
    for _ in 0..20 {
        let frame = ActionFrame {
            buttons: buttons::LIGHT,
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
        tick(&mut w, &TickInput::default());
        w.events.drain();
    }
    assert_eq!(w.b09.lamps, 0);
}

#[test]
fn a_lit_lamp_stuns_her_as_she_flies_past_it() {
    let mut w = watch_world();
    phase_two(&mut w);
    let id = body_of(&w);
    w.b09.lamps = 1; // lamp 0, at (132, 120)
    // She flies over it.
    {
        let a = w.actors.get_mut(id).unwrap();
        a.pos[0] = 131.0;
        a.pos[2] = 120.0;
    }
    let events = idle(&mut w, 1);
    assert_eq!(mechanics(&events, MechanicEvent::MATRIARCH_LAMP_STUN), [0]);
    assert_eq!(
        mechanics(&events, MechanicEvent::MATRIARCH_LANDED),
        [super::b09::LANDED_LAMP]
    );
    let a = actor(&w, id);
    assert!(a.poise.kneel >= 58, "she kneels: {}", a.poise.kneel);
    assert_eq!(
        a.moves.current.map(|m| m.num),
        w.moves.num_of("b09_stunned")
    );
    assert!(w.b09.ground);
    // She is down on the floor shortly, and stays down for the ground phase.
    idle(&mut w, 20);
    assert!(actor(&w, id).pos[1] - actor(&w, id).motor.ground_y < 0.8);
}

#[test]
fn an_unlit_lamp_doesnt_stun_her_and_a_lit_one_cools_down() {
    let mut w = watch_world();
    phase_two(&mut w);
    let id = body_of(&w);
    {
        let a = w.actors.get_mut(id).unwrap();
        a.pos[0] = 131.0;
        a.pos[2] = 120.0;
    }
    let events = idle(&mut w, 3);
    assert!(mechanics(&events, MechanicEvent::MATRIARCH_LAMP_STUN).is_empty());
    // Lit: it stuns her once. After she takes off again, a pass over it within the cooldown
    // doesn't.
    w.b09.lamps = 1;
    let events = idle(&mut w, 1);
    assert_eq!(
        mechanics(&events, MechanicEvent::MATRIARCH_LAMP_STUN).len(),
        1
    );
    let ready = w.b09.stun_ready;
    assert_eq!(ready, w.tick + 449, "450 ticks from the stun");
    w.b09.ground = false;
    w.actors.get_mut(id).unwrap().poise.kneel = 0;
    {
        let a = w.actors.get_mut(id).unwrap();
        a.pos[0] = 131.0;
        a.pos[2] = 120.0;
    }
    let events = idle(&mut w, 5);
    assert!(mechanics(&events, MechanicEvent::MATRIARCH_LAMP_STUN).is_empty());
}

#[test]
fn the_players_death_relights_the_cavern_and_resets_the_fight() {
    let mut w = watch_world();
    phase_two(&mut w);
    w.b09.lamps = 0b101;
    let p = w.actors.get_mut(w.player).unwrap();
    p.hp = 0;
    p.dead = true;
    let events = idle(&mut w, 3);
    assert_eq!(w.bosses[0].stage, Stage::Dormant);
    assert_eq!(w.b09, State::default());
    assert_eq!(
        mechanics(&events, MechanicEvent::MATRIARCH_LAMPS_DOUSED),
        [0]
    );
    assert_eq!(super::b09::light_at(&w, 112.0, 120.0), None);
    let body = actor(&w, body_of(&w));
    assert_eq!(body.hp, body.max_hp);
}

#[test]
fn the_lamplight_hollow_in_the_level_has_the_boss_and_its_fog_gate() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(62, &content, "hollowdeep").unwrap();
    assert_eq!(w.layout.bosses.len(), 1);
    let lay = w.layout.bosses[0].clone();
    assert_eq!(lay.def.id, BOSS_ID);
    assert_eq!(lay.members.len(), 1);
    assert_eq!(lay.gate.pos, [104.0, 0.0, 92.0]);
    // The player at the gate seals it, and the fight starts after the intro.
    put_player(&mut w, [104.0, 0.0, 93.0]);
    idle(&mut w, 5);
    assert!(matches!(w.bosses[0].stage, Stage::Intro { .. }));
    idle(&mut w, 95);
    assert_eq!(w.bosses[0].stage, Stage::Fight);
    let spawn = w.layout.enemies[usize::from(lay.members[0])].pos;
    assert_eq!([spawn[0], spawn[2]], [112.0, 120.0]);
    // The lamps in the level (layout.ron) are where the definition puts them.
    for (i, l) in LAMPS.iter().enumerate() {
        let off = lay.def.params[&format!("lamp_{i}_dx_m")];
        assert!((112.0 + off - l[0]).abs() < 0.01, "lamp {i}");
    }
}

// ---------------------------------------------------------------------------------------
// The fight
// ---------------------------------------------------------------------------------------

#[test]
fn the_scripted_bot_beats_b09_in_both_phases() {
    let f = run_bot(SEED);
    assert!(
        f.boss_defeated(),
        "the boss stands at tick {}",
        f.world.tick
    );
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    // The fight used the mechanic: she flew and landed, phase 2 came, and the lamps were lit.
    assert!(f.mechanic_count(MechanicEvent::MATRIARCH_LANDED) >= 1);
    assert_eq!(f.boss_events(BossEvent::PHASE_STARTED), 1);
    assert_eq!(f.mechanic_count(MechanicEvent::MATRIARCH_DRAINED), 1);
    assert!(
        f.mechanic_count(MechanicEvent::MATRIARCH_LAMP_LIT) >= 1,
        "the bot lit a lamp"
    );
    // The Great Sigil and the Glimmer paid out.
    let inv = &f.world.actors.get(f.world.player).unwrap().inventory;
    assert_eq!(inv.glimmer, 4500);
    let content = Content::load_embedded().unwrap();
    let sigil = content.registry.num("itm_great_sigil_matriarch").unwrap();
    assert!(inv.count(sigil) >= 1, "no Great Sigil of the Deep Lamp");
    // The cavern is lit again.
    assert_eq!(f.world.b09, State::default());
}

#[test]
fn a_player_who_does_nothing_dies_to_b09() {
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
fn the_bot_golden_replay_beats_b09_with_the_player_alive() {
    let f = play_golden("m7_b09_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b09_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b09_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b09_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b09_goldens` in game/crates/core/src/ai/bosses/b09_tests.rs.\n\
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
        "m7_b09_bot_win",
        "WP-077 golden: the scripted bot beats B09: it fights her on the ground, and lights the lamps with the Lantern Sword's Fire in phase 2.",
        &run_bot(SEED),
    );
    write(
        "m7_b09_idle_dies",
        "WP-077 golden: a player who walks in and does nothing dies to B09.",
        &run_idle(SEED),
    );
}
