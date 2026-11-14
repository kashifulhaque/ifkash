//! Tests for B08, the Forgeheart Colossus (WP-076), and the scripted bot that beats it.
//!
//! The bot plays the fight as the spec describes it (docs/game/05-enemies-and-bosses.md, "B08
//! Forgeheart Colossus"): it works at one leg until the leg's stance breaks, then walks to the
//! front of the kneeling Colossus and ripostes the open furnace. It rolls through the stomp, the
//! sweep, and the flare, and leaves the creeping lava's edge alone. It is deterministic (no
//! randomness, no clock), and [`run_bot`] and [`run_idle`] play a whole fight from any seed.
//!
//! The goldens `m7_b08_bot_win.lfr` and `m7_b08_idle_dies.lfr` come from the bot's inputs:
//! `record_the_b08_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;

use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::arena::Id;
use crate::combat::hitbox::{Capsule, Pose, closest_points};
use crate::combat::moves::MoveNum;
use crate::events::{BossEvent, Event, LifeEvent, MechanicEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::math::dmath::{atan2, cos, sin, sqrt};
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

const SCENARIO: &str = "boss_b08";
const BOSS_ID: &str = "b08_forgeheart_colossus";
/// The seed the goldens use.
const SEED: u64 = 76;
/// The longest fight the runners play, in ticks (at 30 ticks a second, ten minutes). This only
/// stops a stuck run.
const MAX_TICKS: u64 = 18_000;

fn body_of(w: &World) -> Id {
    member(w, 0)
}

fn member(w: &World, i: usize) -> Id {
    let spawn = w.layout.bosses[0].members[i];
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

/// The scripted player.
pub(crate) struct Bot {
    /// How many ticks before a hit's first frame the bot presses DODGE.
    lead: u16,
    seen: Option<(MoveNum, u16)>,
    rolled: bool,
    /// Buttons sent last tick, because every button is a press edge.
    last: u16,
    /// The leg the bot is working on.
    leg: usize,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            lead: 7,
            seen: None,
            rolled: false,
            last: 0,
            leg: 0,
        }
    }
}

impl Bot {
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

    fn decide(&mut self, w: &World) -> ((i8, i8), u16) {
        let Some(player) = w.actors.get(w.player) else {
            return ((0, 0), 0);
        };
        match w.bosses[0].stage {
            Stage::Dormant => return ((127, 0), crate::ai::boss::fog_buttons(w)), // through the fog gate, east
            Stage::Intro { .. } | Stage::Defeated => return ((0, 0), 0),
            Stage::Fight => {}
        }
        if player.dead || player.hp == 0 {
            return ((0, 0), 0);
        }
        let Some(body) = w.actors.get(body_of(w)) else {
            return ((0, 0), 0);
        };
        let legs = [member(w, 1), member(w, 2)].map(|id| w.actors.get(id));
        let (Some(l0), Some(l1)) = (legs[0], legs[1]) else {
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

        // Where the bot is headed, and what it does when it gets there.
        let leg = if self.leg == 0 { l0 } else { l1 };
        let kneeling = body.poise.kneel > 0;
        let fwd = [sin(body.yaw), cos(body.yaw)];
        let front = [body.pos[0] + fwd[0] * 1.1, 0.0, body.pos[2] + fwd[1] * 1.1];
        let goal = if kneeling { front } else { leg.pos };

        // 1. Roll through a hit that is about to land, toward where the bot is going.
        if let Some(first) = hit
            && !self.rolled
            && frame < first
            && first - frame <= self.lead
            && player.stamina >= 25.0
        {
            return (toward(pp, goal, 1.0), buttons::DODGE);
        }
        // 2. The Colossus kneels: the furnace is open. Riposte it from the front.
        if kneeling {
            if dist(pp, front) > 0.5 {
                return (toward(pp, front, 1.0), 0);
            }
            return (toward(pp, body.pos, 0.1), buttons::LIGHT);
        }
        // 3. Drink a flask in a long recovery, when hurt.
        if player.hp * 100 < player.max_hp * 50
            && player.inventory.flasks.vigil > 0
            && recovery_left(w, body) >= 36
            && player.moves.current.is_none()
        {
            return ((0, 0), buttons::USE_ITEM);
        }
        // 4. Work at a leg until its stance breaks, but never start a swing that a coming
        // hit would catch the bot in.
        let d = dist(pp, leg.pos);
        if d > 1.5 {
            return (toward(pp, leg.pos, 1.0), 0);
        }
        if hit.is_none() && player.stamina >= 30.0 && player.moves.current.is_none() {
            return (toward(pp, leg.pos, 0.1), buttons::LIGHT);
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
        move_x: if w.bosses[0].stage == Stage::Dormant {
            127
        } else {
            0
        },
        move_z: 0,
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
            let (l0, l1) = (actor(&w, member(&w, 1)), actor(&w, member(&w, 2)));
            eprintln!(
                "t{} {:?} ph{} player hp {} st {:.0} ({:.1},{:.1}) body hp {} kneel {} mv {:?} stance {}/{} {}/{}",
                w.tick,
                w.bosses[0].stage,
                w.bosses[0].phase,
                p.hp,
                p.stamina,
                p.pos[0],
                p.pos[2],
                b.hp,
                b.poise.kneel,
                b.moves.current.map(|m| (m.num, m.frame)),
                l0.poise.stance_dmg,
                l0.poise.stance,
                l1.poise.stance_dmg,
                l1.poise.stance,
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

/// A world with the fight on: the player walked through the gate and the intro is over.
fn fight_world() -> World {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    while w.bosses[0].stage != Stage::Fight {
        let frame = ActionFrame {
            move_x: if w.bosses[0].stage == Stage::Dormant {
                127
            } else {
                0
            },
            buttons: crate::ai::boss::fog_buttons(&w),
            move_z: 0,
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

fn mechanics(events: &[Event], kind: u8) -> Vec<u16> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Mechanic(m) if m.kind == kind => Some(m.index),
            _ => None,
        })
        .collect()
}

#[test]
fn the_colossus_stands_on_its_legs() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    tick(&mut w, &TickInput::default());
    let body = actor(&w, body_of(&w));
    assert!(
        body.hurtbox.radius > 1.0,
        "the body is bigger than a person"
    );
    // The legs flank the body, one on each side. It faces -x, so its left is +z.
    let (l, r) = (actor(&w, member(&w, 1)), actor(&w, member(&w, 2)));
    assert!((dist(l.pos, body.pos) - 2.4).abs() < 0.01);
    assert!((dist(r.pos, body.pos) - 2.4).abs() < 0.01);
    assert!((l.pos[2] - body.pos[2] - 2.4).abs() < 0.01);
    assert!((r.pos[2] - body.pos[2] + 2.4).abs() < 0.01);
    assert_eq!(l.poise.stance, 240);
    assert_eq!(r.poise.stance, 240);
    assert_eq!(body.poise.stance, 0, "the stance meters are in the legs");
}

#[test]
fn the_legs_follow_the_body_when_it_turns() {
    let mut w = fight_world();
    let id = body_of(&w);
    w.actors.get_mut(id).unwrap().yaw = 0.0; // facing +z: its left is +x
    tick(&mut w, &TickInput::default());
    let body = actor(&w, id).pos;
    let (l, r) = (actor(&w, member(&w, 1)), actor(&w, member(&w, 2)));
    assert!(l.pos[0] > body[0] + 2.0 && r.pos[0] < body[0] - 2.0);
}

#[test]
fn a_hit_on_a_leg_hurts_the_colossus() {
    let mut w = fight_world();
    let (body, leg) = (body_of(&w), member(&w, 1));
    let before = actor(&w, body).hp;
    let max = actor(&w, leg).max_hp;
    w.actors.get_mut(leg).unwrap().hp = max - 500;
    idle(&mut w, 1);
    assert_eq!(actor(&w, body).hp, before - 500);
    assert_eq!(actor(&w, leg).hp, max, "the leg is whole again");
}

#[test]
fn a_hit_on_a_leg_does_nothing_while_the_colossus_changes_phase() {
    let mut w = fight_world();
    let (body, leg) = (body_of(&w), member(&w, 2));
    // Just under 40 percent: the next tick starts the transition move.
    w.actors.get_mut(body).unwrap().hp = 2700;
    idle(&mut w, 2);
    assert!(w.bosses[0].transition_left > 0, "the heart is igniting");
    let max = actor(&w, leg).max_hp;
    w.actors.get_mut(leg).unwrap().hp = max - 1000;
    idle(&mut w, 1);
    assert_eq!(actor(&w, body).hp, 2700, "invulnerable");
    assert_eq!(actor(&w, leg).hp, max);
    // Breaking a leg's stance now doesn't make it kneel either.
    w.actors.get_mut(leg).unwrap().poise.kneel = 60;
    idle(&mut w, 1);
    assert_eq!(actor(&w, body).poise.kneel, 0);
}

#[test]
fn a_broken_leg_kneels_the_colossus_and_opens_the_furnace() {
    let mut w = fight_world();
    let (body, left, right) = (body_of(&w), member(&w, 1), member(&w, 2));
    // The stance break as the combat code leaves it: the leg kneels for 60 ticks.
    {
        let a = w.actors.get_mut(left).unwrap();
        a.poise.kneel = 60;
        a.poise.stance_dmg = 0;
    }
    let events = idle(&mut w, 1);
    assert_eq!(
        mechanics(&events, MechanicEvent::COLOSSUS_LEG_BROKE),
        [0],
        "the left leg broke"
    );
    let b = actor(&w, body);
    let kneeling = b.poise.kneel;
    assert!(kneeling >= 58, "the body kneels: {kneeling}");
    let kneel = w.moves.num_of("b08_kneel").unwrap();
    assert_eq!(b.moves.current.map(|m| m.num), Some(kneel));
    // Only the body can be riposted: the leg's own kneel is used up at once, and the leg
    // plays its buckle.
    let l = actor(&w, left);
    assert_eq!(l.poise.kneel, 0);
    assert_eq!(
        l.moves.current.map(|m| m.num),
        w.moves.num_of("b08_leg_buckle")
    );
    // The furnace is open to a riposte from the front.
    let t = w.locomotion.poise;
    let front = [
        b.pos[0] + sin(b.yaw) * 1.1,
        b.pos[1],
        b.pos[2] + cos(b.yaw) * 1.1,
    ];
    let attacker = Actor {
        pos: front,
        yaw: atan2(b.pos[0] - front[0], b.pos[2] - front[2]),
        hp: 1,
        max_hp: 1,
        ..Actor::default()
    };
    assert!(
        crate::combat::crit::kind_against(&attacker, b, &t).is_some(),
        "a riposte from the front"
    );
    // The other leg can't be worked while the Colossus kneels, and can't make it kneel again.
    {
        let a = w.actors.get_mut(right).unwrap();
        a.poise.stance_dmg = 200;
        a.poise.kneel = 60;
    }
    let events = idle(&mut w, 1);
    assert!(mechanics(&events, MechanicEvent::COLOSSUS_LEG_BROKE).is_empty());
    assert_eq!(actor(&w, right).poise.stance_dmg, 0);
    assert_eq!(actor(&w, right).poise.kneel, 0);
    assert!(
        actor(&w, body).poise.kneel < kneeling,
        "the kneel wasn't extended"
    );
    // On its feet again after the 60 ticks, a leg can break once more.
    idle(&mut w, 62);
    assert_eq!(actor(&w, body).poise.kneel, 0);
    w.actors.get_mut(right).unwrap().poise.kneel = 60;
    let events = idle(&mut w, 1);
    assert_eq!(mechanics(&events, MechanicEvent::COLOSSUS_LEG_BROKE), [1]);
}

#[test]
fn a_broken_leg_stops_the_swing_the_colossus_was_in() {
    let mut w = fight_world();
    let (body, leg) = (body_of(&w), member(&w, 1));
    // Wait until it is winding up something.
    let mut n = 0;
    while actor(&w, body).moves.current.is_none() {
        idle(&mut w, 1);
        n += 1;
        assert!(n < 300, "the Colossus never attacked");
    }
    let mid = actor(&w, body).moves.current.unwrap().num;
    w.actors.get_mut(leg).unwrap().poise.kneel = 60;
    idle(&mut w, 1);
    let now = actor(&w, body).moves.current.unwrap().num;
    assert_ne!(mid, now);
    assert_eq!(Some(now), w.moves.num_of("b08_kneel"));
}

#[test]
fn the_forge_is_dry_until_the_heart_ignites() {
    let mut w = fight_world();
    idle(&mut w, 300);
    assert_eq!(w.bosses[0].phase, 1);
    assert_eq!(w.mech.b08_lava_from, 0);
}

/// Starts phase 2: the body just under 40 percent, and ticks until the transition is over.
fn phase_two(w: &mut World) -> Vec<Event> {
    let body = body_of(w);
    w.actors.get_mut(body).unwrap().hp = 2700;
    let mut events = Vec::new();
    for _ in 0..60 {
        events.extend(idle(w, 1));
        if w.bosses[0].phase == 2 && w.mech.b08_lava_from > 0 {
            break;
        }
    }
    assert_eq!(w.bosses[0].phase, 2);
    events
}

#[test]
fn lava_creeps_in_from_the_walls_in_phase_two_and_burns() {
    let mut w = fight_world();
    let events = phase_two(&mut w);
    assert_eq!(
        mechanics(&events, MechanicEvent::COLOSSUS_LAVA_STARTED),
        [0]
    );
    let from = w.mech.b08_lava_from;
    assert!(from > 0);
    // The east wall's foot is x 78 and the body's spawn is x 62, so the lava's edge there is 16 m
    // out; the player stands 1 m inside it.
    let body = actor(&w, body_of(&w)).pos;
    put_player(&mut w, [77.0, body[1], 72.0]);
    let events = idle(&mut w, 30);
    // The lava is only a few centimeters deep: it hasn't reached the player.
    assert!(mechanics(&events, MechanicEvent::COLOSSUS_LAVA_BURNED).is_empty());
    // Run the clock on: 300 ticks at 0.5 m per 100 is 1.5 m, past the player.
    w.tick += 1000;
    w.mech.b08_lava_from = w.tick - 300;
    let events = idle(&mut w, 25);
    assert!(
        !mechanics(&events, MechanicEvent::COLOSSUS_LAVA_BURNED).is_empty(),
        "the lava reached the player"
    );
}

#[test]
fn lava_leaves_the_middle_of_the_forge_dry_however_long_the_fight_lasts() {
    let mut w = fight_world();
    phase_two(&mut w);
    // An hour of lava.
    w.mech.b08_lava_from = 1;
    w.tick = 100_000;
    let body = actor(&w, body_of(&w)).pos;
    put_player(&mut w, [body[0], body[1], body[2] - 6.0]);
    let events = idle(&mut w, 200);
    assert!(
        mechanics(&events, MechanicEvent::COLOSSUS_LAVA_BURNED).is_empty(),
        "the middle stays dry"
    );
    // The edges are all lava by then.
    put_player(&mut w, [45.0, body[1], 72.0]);
    let events = idle(&mut w, 25);
    assert!(!mechanics(&events, MechanicEvent::COLOSSUS_LAVA_BURNED).is_empty());
}

#[test]
fn the_players_death_dries_the_forge_and_resets_the_fight() {
    let mut w = fight_world();
    phase_two(&mut w);
    assert!(w.mech.b08_lava_from > 0);
    w.actors.get_mut(w.player).unwrap().hp = 0;
    w.actors.get_mut(w.player).unwrap().dead = true;
    idle(&mut w, 3);
    assert_eq!(w.bosses[0].stage, Stage::Dormant);
    assert_eq!(w.mech.b08_lava_from, 0);
    let body = actor(&w, body_of(&w));
    assert_eq!(body.hp, body.max_hp);
}

#[test]
fn the_fight_reset_dries_the_forge_on_the_same_tick() {
    let mut w = fight_world();
    phase_two(&mut w);
    assert!(w.mech.b08_lava_from > 0);
    w.actors.get_mut(w.player).unwrap().hp = 0;
    w.actors.get_mut(w.player).unwrap().dead = true;
    idle(&mut w, 1);
    assert_eq!(w.bosses[0].stage, Stage::Dormant);
    assert_eq!(w.mech.b08_lava_from, 0);
}

#[test]
fn the_forge_in_the_level_has_the_boss_and_its_fog_gate() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(62, &content, "cinderdeep_mine").unwrap();
    assert_eq!(w.layout.bosses.len(), 1);
    let lay = &w.layout.bosses[0];
    assert_eq!(lay.def.id, BOSS_ID);
    assert_eq!(lay.members.len(), 3);
    assert_eq!(lay.gate.pos, [44.0, -22.0, 82.0]);
    // The player at the gate seals it, and the fight starts after the intro.
    put_player(&mut w, [45.0, -22.0, 82.0]);
    idle(&mut w, 5);
    assert!(matches!(w.bosses[0].stage, Stage::Intro { .. }));
    idle(&mut w, 95);
    assert_eq!(w.bosses[0].stage, Stage::Fight);
    let body = actor(&w, body_of(&w)).pos;
    assert_eq!(body, [62.0, -22.0, 72.0]);
    let leg = actor(&w, member(&w, 1)).pos;
    assert!((dist(leg, body) - 2.4).abs() < 0.01);
}

// ---------------------------------------------------------------------------------------
// The fight
// ---------------------------------------------------------------------------------------

#[test]
fn the_scripted_bot_beats_b08_by_breaking_legs_and_riposting_the_furnace() {
    let f = run_bot(SEED);
    assert!(
        f.boss_defeated(),
        "the boss stands at tick {}",
        f.world.tick
    );
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    // The fight used the mechanic: legs broke, the furnace was ripposted, and phase 2 came.
    assert!(f.mechanic_count(MechanicEvent::COLOSSUS_LEG_BROKE) >= 1);
    let body = body_of(&f.world);
    let ripostes = f
        .events
        .iter()
        .filter(|e| matches!(e, Event::Critical(c) if c.target == body))
        .count();
    assert!(ripostes >= 1, "no riposte on the body");
    assert!(
        f.events
            .iter()
            .all(|e| !matches!(e, Event::Critical(c) if c.target != body)),
        "only the body is ripostable"
    );
    assert_eq!(f.boss_events(BossEvent::PHASE_STARTED), 1);
    assert_eq!(f.mechanic_count(MechanicEvent::COLOSSUS_LAVA_STARTED), 1);
    // The Maul and the Ember paid out, with the Glimmer.
    let inv = &f.world.actors.get(f.world.player).unwrap().inventory;
    assert_eq!(inv.glimmer, 3500);
    let content = Content::load_embedded().unwrap();
    let ember = content.registry.num("itm_forgeheart_ember").unwrap();
    let maul = content.registry.num("wpn_forgeheart_maul").unwrap();
    assert!(inv.count(ember) >= 1, "no Ember");
    assert!(inv.gear.contains(&maul), "no Maul");
}

#[test]
fn a_player_who_does_nothing_dies_to_b08() {
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
fn the_bot_golden_replay_beats_b08_with_the_player_alive() {
    let f = play_golden("m7_b08_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b08_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b08_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b08_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b08_goldens` in game/crates/core/src/ai/bosses/b08_tests.rs.\n\
             # Each line: <ticks> <move_x> <move_z> [BUTTONS].\nseed {SEED}\nscenario {SCENARIO}\n"
        );
        for (frame, n) in crate::replay::compress(&f.frames) {
            let mut names = Vec::new();
            for (mask, name) in [
                (buttons::LIGHT, "LIGHT"),
                (buttons::INTERACT, "INTERACT"),
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
        "m7_b08_bot_win",
        "WP-076 golden: the scripted bot beats B08 by breaking a leg's stance and riposting the furnace.",
        &run_bot(SEED),
    );
    write(
        "m7_b08_idle_dies",
        "WP-076 golden: a player who walks in and does nothing dies to B08.",
        &run_idle(SEED),
    );
}
