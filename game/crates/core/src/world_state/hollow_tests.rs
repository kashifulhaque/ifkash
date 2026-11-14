//! Tests for Waking Hollow (WP-043): a scripted bot walks the critical path from the
//! Lantern to the B01 arena, fighting what stands in the way, and the golden replay
//! `m4_waking_hollow` is its input.
//!
//! The bot is deterministic. It reads the world each tick, fights the nearest awake enemy
//! with the same roll-through-the-hit rule as the B01 bot (`ai/b01_tests.rs`), and walks the
//! route between fights. It stops once the player has stepped through the fog gate and the
//! boss fight has begun: the route is the critical path, not a win over B01 (WP-034 proves
//! that fight is fair).
//!
//! Record the golden: `cargo test -p lf-core record_the_hollow_golden -- --ignored`, then
//! `cargo run -p lf-tools -- replay record-script tests/replays/m4_waking_hollow.script
//! --out tests/replays/m4_waking_hollow.lfr` (from `game/`).

use lf_content::Content;

use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::events::{Event, LifeEvent, MechanicEvent};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

const SCENARIO: &str = "waking_hollow";
const SEED: u64 = 43;
/// The longest run, in ticks (the route takes about 3,000; this only stops a stuck run).
const MAX_TICKS: u64 = 20_000;
/// How close an enemy has to be before the bot stops walking and fights, in meters.
const ENGAGE_M: f32 = 8.5;
/// Every message the cave teaches a control with, in the order of the route.
const MESSAGES: [&str; 10] = [
    "msg_move",
    "msg_lantern",
    "msg_jump",
    "msg_attack",
    "msg_items",
    "msg_dodge",
    "msg_guard",
    "msg_crack",
    "msg_flask",
    "msg_sentry",
];

/// One leg of the route.
#[derive(Clone, Copy)]
pub(super) enum Leg {
    /// Walk to a point (x, z).
    Goto(f32, f32),
    /// Press Interact (read a message, take a pickup, open a chest).
    Interact,
    /// Rest at the Lantern in reach.
    Rest,
    /// Run forward and jump (the corridor's ledge).
    Jump,
    /// Swing once (the hollow wall).
    Swing,
    /// Let the world settle for a few ticks.
    Wait(u16),
}

fn route() -> Vec<Leg> {
    use Leg::*;
    vec![
        // Wake: the Lantern first, then the movement message.
        Goto(0.0, -3.0),
        Rest,
        Goto(0.0, 3.0),
        Interact,
        // Corridor 1: the jump.
        Goto(0.0, 12.0),
        Goto(0.0, 13.0),
        Interact,
        Goto(0.0, 14.2),
        Jump,
        Goto(0.0, 21.0),
        // First fight: the husk, then the alcove's salts.
        Goto(0.0, 25.0),
        Interact,
        Goto(0.0, 29.0),
        Goto(11.0, 30.0),
        Interact,
        Goto(12.4, 30.0),
        Interact,
        Goto(0.0, 36.0),
        Goto(0.0, 41.0),
        // Hall: the two husks, then the hollow wall and its chest.
        Goto(0.0, 47.0),
        Interact,
        Goto(0.0, 51.0),
        Interact,
        Goto(0.0, 54.0),
        Goto(-6.0, 52.0),
        Interact,
        Goto(-8.6, 52.0),
        Swing,
        Wait(40),
        Goto(-13.4, 52.0),
        Interact,
        Wait(5),
        Goto(0.0, 56.0),
        Goto(0.0, 65.0),
        // Twist: the husk and the hound.
        Goto(0.0, 70.0),
        Interact,
        Goto(0.0, 80.0),
        Goto(0.0, 85.0),
        // Antechamber and the fog gate.
        Goto(0.0, 91.0),
        Interact,
        Goto(0.0, 96.5),
        Goto(0.0, 102.0),
    ]
}

/// The scripted player.
pub(super) struct Bot {
    pub(super) leg: usize,
    pub(super) legs: Vec<Leg>,
    /// Ticks left in a `Wait`.
    wait: u16,
    /// Buttons sent last tick, because every button is a press edge.
    last: u16,
    /// The enemy move and frame last seen, to tell when a new move starts.
    seen: Option<(crate::combat::moves::MoveNum, u16)>,
    rolled: bool,
    /// The jump leg's phase: 0 not started, then ticks since the press.
    jump_at: Option<u32>,
    /// Walk toward an enemy that is in range but not yet lockable, instead of pressing the
    /// lock button where it can't work (an enemy that hasn't noticed the player).
    pub(super) approach: bool,
}

impl Bot {
    fn new() -> Self {
        Self::with_legs(route())
    }

    /// A bot that walks `legs` (the Verge's walk reuses it).
    pub(super) fn with_legs(legs: Vec<Leg>) -> Self {
        Self {
            leg: 0,
            legs,
            wait: 0,
            last: 0,
            seen: None,
            rolled: false,
            jump_at: None,
            approach: false,
        }
    }

    pub(super) fn done(&self) -> bool {
        self.leg >= self.legs.len()
    }

    /// The nearest living enemy within `ENGAGE_M`, not counting the boss.
    fn target<'a>(w: &'a World, p: &Actor) -> Option<(&'a Actor, f32)> {
        let boss: Vec<_> = w
            .layout
            .bosses
            .iter()
            .flat_map(|b| {
                b.members
                    .iter()
                    .map(|&m| w.state.ai_enemies[usize::from(m)])
            })
            .collect();
        w.state
            .ai_enemies
            .iter()
            .filter(|id| !boss.contains(id))
            .filter_map(|&id| w.actors.get(id))
            .filter(|a| a.hp > 0 && !a.dead)
            .map(|a| {
                let (dx, dz) = (a.pos[0] - p.pos[0], a.pos[2] - p.pos[2]);
                (a, sqrt(dx * dx + dz * dz))
            })
            .filter(|&(_, d)| d <= ENGAGE_M)
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }

    pub(super) fn act(&mut self, w: &World) -> TickInput {
        let (frame, commands) = self.decide(w);
        let mut f = frame;
        // At a boss's fog gate, Interact passes through it, as a player would (issue #82).
        if crate::ai::boss::walking_into_fog(w, f.move_x, f.move_z) {
            f.buttons |= crate::input::buttons::INTERACT;
        }
        f.buttons &= !self.last;
        self.last = f.buttons;
        TickInput { frame: f, commands }
    }

    fn decide(&mut self, w: &World) -> (ActionFrame, Vec<Command>) {
        let none = ActionFrame::default();
        let Some(p) = w.actors.get(w.player) else {
            return (none, vec![]);
        };
        let press = |mask: u16| {
            (
                ActionFrame {
                    move_x: 0,
                    move_z: 0,
                    buttons: mask,
                },
                vec![],
            )
        };
        if p.dead || p.hp == 0 {
            return (none, vec![]);
        }
        // A fight takes priority over the route.
        if let Some((e, dist)) = Self::target(w, p) {
            return self.fight(w, p, e, dist);
        }
        self.seen = None;
        self.rolled = false;
        if self.done() {
            return (none, vec![]);
        }
        // Drink a flask when hurt and nobody is near.
        if p.hp * 100 < p.max_hp * 40 && p.inventory.flasks.vigil > 0 && p.motor.grounded {
            return press(buttons::USE_ITEM);
        }
        match self.legs[self.leg] {
            Leg::Goto(x, z) => {
                let (dx, dz) = (x - p.pos[0], z - p.pos[2]);
                let d = sqrt(dx * dx + dz * dz);
                if d < 0.3 {
                    self.leg += 1;
                    return (none, vec![]);
                }
                let mag = if d < 1.0 { 50.0 } else { 127.0 };
                (
                    ActionFrame {
                        move_x: (dx / d * mag) as i8,
                        move_z: (dz / d * mag) as i8,
                        buttons: 0,
                    },
                    vec![],
                )
            }
            Leg::Interact => {
                self.leg += 1;
                press(buttons::INTERACT)
            }
            Leg::Rest => {
                self.leg += 1;
                (none, vec![Command::Rest])
            }
            Leg::Jump => {
                // Press jump while running at the ledge, then keep running until over it.
                let t = *self.jump_at.get_or_insert(0);
                self.jump_at = Some(t + 1);
                if t == 0 {
                    return (
                        ActionFrame {
                            move_x: 0,
                            move_z: 127,
                            buttons: buttons::JUMP,
                        },
                        vec![],
                    );
                }
                if p.pos[2] > 19.0 && p.motor.grounded {
                    self.leg += 1;
                }
                (
                    ActionFrame {
                        move_x: 0,
                        move_z: 127,
                        buttons: 0,
                    },
                    vec![],
                )
            }
            Leg::Swing => {
                self.leg += 1;
                press(buttons::LIGHT)
            }
            Leg::Wait(n) => {
                if self.wait == 0 {
                    self.wait = n;
                }
                self.wait -= 1;
                if self.wait == 0 {
                    self.leg += 1;
                }
                (none, vec![])
            }
        }
    }

    fn fight(&mut self, w: &World, p: &Actor, e: &Actor, dist: f32) -> (ActionFrame, Vec<Command>) {
        let toward = {
            let (dx, dz) = (e.pos[0] - p.pos[0], e.pos[2] - p.pos[2]);
            let d = dist.max(0.001);
            ((dx / d * 127.0) as i8, (dz / d * 127.0) as i8)
        };
        let go = |x: i8, z: i8, b: u16| {
            (
                ActionFrame {
                    move_x: x,
                    move_z: z,
                    buttons: b,
                },
                vec![],
            )
        };
        if p.lock_target.is_none() {
            if self.approach && dist > 6.0 {
                return go(toward.0, toward.1, 0);
            }
            // Face the enemy before locking on. Lock-on also takes a nearby enemy behind the
            // player now (issue #80), which may not be the one this bot means to fight.
            let off = crate::actor::targeting::wrap_pi(
                crate::actor::targeting::bearing(p.pos, e.pos, p.yaw) - p.yaw,
            );
            if off.abs() > 0.5 {
                return go(toward.0 / 8, toward.1 / 8, 0);
            }
            return go(0, 0, buttons::LOCK);
        }
        // Read the enemy's move: its first hit frame decides when to roll.
        let mut hit: Option<u16> = None;
        let mut frame = 0u16;
        let mut attacking = false;
        if let Some(m) = e.moves.current {
            attacking = true;
            if let Some(def) = w.moves.get(m.num) {
                hit = def.data.hitboxes.iter().map(|h| h.frames.0).min();
                frame = m.frame;
            }
            if self.seen.is_none_or(|(n, f)| n != m.num || f > m.frame) {
                self.rolled = false;
            }
            self.seen = Some((m.num, m.frame));
        } else {
            self.seen = None;
            self.rolled = false;
        }
        if p.moves
            .current
            .and_then(|c| w.moves.player_moveset.as_ref().map(|s| (c.num, s)))
            .is_some_and(|(n, s)| n == s.roll_light || n == s.roll_medium || n == s.roll_heavy)
        {
            self.rolled = true;
        }
        // Roll through a hit that is about to land.
        if let Some(first) = hit
            && !self.rolled
            && frame < first
            && first - frame <= 7
            && dist < 4.0
        {
            return go(toward.0, toward.1, buttons::DODGE);
        }
        if dist > 2.0 {
            return go(toward.0, toward.1, 0);
        }
        if !attacking && p.stamina >= 25.0 {
            return go(0, 0, buttons::LIGHT);
        }
        go(0, 0, 0)
    }
}

/// What a run produced.
struct Run {
    frames: Vec<ActionFrame>,
    commands: Vec<(u64, Command)>,
    world: World,
    events: Vec<Event>,
    bot: Bot,
}

fn walk() -> Run {
    let content = Content::load_embedded().expect("embedded content");
    let mut w = World::new(SEED, &content, SCENARIO).expect("waking_hollow exists");
    let mut bot = Bot::new();
    let (mut frames, mut commands, mut events) = (Vec::new(), Vec::new(), Vec::new());
    let mut in_arena = 0u32;
    while w.tick < MAX_TICKS && in_arena < 30 {
        let input = bot.act(&w);
        frames.push(input.frame);
        for c in &input.commands {
            commands.push((w.tick, c.clone()));
        }
        tick(&mut w, &input);
        events.extend(w.events.drain());
        if w.bosses[0].stage != Stage::Dormant {
            in_arena += 1;
        }
        if events
            .iter()
            .any(|e| matches!(e, Event::Life(LifeEvent::Died { .. })))
        {
            break;
        }
    }
    Run {
        frames,
        commands,
        world: w,
        events,
        bot,
    }
}

fn player(w: &World) -> &Actor {
    w.actors.get(w.player).unwrap()
}

#[test]
fn the_critical_path_reaches_the_lantern_and_the_arena() {
    let r = walk();
    let w = &r.world;
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, Event::Life(LifeEvent::Died { .. }))),
        "the bot died at tick {} near {:?}",
        w.tick,
        player(w).pos
    );
    // The Lantern was lit.
    assert!(w.state.flags.is_set(&flags::lantern_lit("lantern_l01")));
    // The player walked through the fog gate: the boss fight began.
    assert_ne!(
        w.bosses[0].stage,
        Stage::Dormant,
        "never entered the arena; leg {} of {}, at {:?}, tick {}",
        r.bot.leg,
        r.bot.legs.len(),
        player(w).pos,
        w.tick
    );
    let z = player(w).pos[2];
    assert!(z > 98.0, "inside the arena, z = {z}");
    // Every enemy on the way is down, and the path stayed in about three minutes.
    assert!(w.tick < 12_000, "took {} ticks", w.tick);
    // The run ends a moment after the fight starts, on the last leg.
    assert!(
        r.bot.leg + 1 >= r.bot.legs.len(),
        "stopped at leg {} of {}",
        r.bot.leg,
        r.bot.legs.len()
    );
}

/// Each message the cave teaches a control with can be read from where it is placed, and
/// each lies on the walkable floor.
#[test]
fn every_tutorial_message_can_be_read() {
    let content = Content::load_embedded().unwrap();
    let w = World::new(SEED, &content, SCENARIO).unwrap();
    assert!(w.layout.messages.len() >= 10);
    for (i, m) in w.layout.messages.iter().enumerate() {
        let mut v = World::new(SEED, &content, SCENARIO).unwrap();
        let id = v.player;
        let a = v.actors.get_mut(id).unwrap();
        a.pos = m.pos;
        a.motor.grounded = true;
        let frame = ActionFrame {
            move_x: 0,
            move_z: 0,
            buttons: buttons::INTERACT,
        };
        tick(
            &mut v,
            &TickInput {
                frame: ActionFrame::default(),
                commands: vec![],
            },
        );
        tick(
            &mut v,
            &TickInput {
                frame,
                commands: vec![],
            },
        );
        assert!(
            v.state.flags.is_set(&flags::message_read(&m.id)),
            "message {} at {:?} was not readable (index {i})",
            m.id,
            m.pos
        );
        assert!(!m.text.is_empty() && m.text.len() < 400, "{}", m.id);
    }
    // The ones the cave names.
    for id in MESSAGES {
        assert!(w.layout.messages.iter().any(|m| m.id == id), "{id}");
    }
}

/// The cave's controls, by message: every verb the card names appears in a message.
#[test]
fn the_messages_teach_every_control() {
    let content = Content::load_embedded().unwrap();
    let w = World::new(SEED, &content, SCENARIO).unwrap();
    let all: String = w
        .layout
        .messages
        .iter()
        .map(|m| m.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    for needle in [
        "W, A, S, and D",
        "mouse",
        "sprint",
        "Lantern",
        "press E",
        "Press F to jump",
        "Press Q to lock on",
        "Left mouse",
        "heavy",
        "stamina",
        "Press R",
        "Tap Space to roll",
        "right mouse button to guard",
        "parry",
        "flask",
    ] {
        assert!(all.contains(needle), "no message teaches: {needle}");
    }
}

#[test]
fn the_hollow_wall_and_chest_open_on_the_route() {
    let r = walk();
    let w = &r.world;
    assert!(w.state.flags.is_set(&flags::wall_revealed("wall_closet")));
    assert!(w.state.flags.is_set(&flags::chest_opened("chest_closet")));
    assert!(w.state.flags.is_set(&flags::pickup_taken("salts_alcove")));
    let kinds: Vec<u8> = r
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Mechanic(m) => Some(m.kind),
            _ => None,
        })
        .collect();
    assert!(kinds.contains(&MechanicEvent::WALL_REVEALED));
}

/// Writes `game/tests/replays/m4_waking_hollow.script`. See the module comment.
#[test]
#[ignore = "writes game/tests/replays/m4_waking_hollow.script"]
fn record_the_hollow_golden() {
    let r = walk();
    assert_ne!(r.world.bosses[0].stage, Stage::Dormant);
    let mut text = String::from(
        "# WP-043 golden: the critical path of Waking Hollow, from the Lantern to the B01 arena.\n\
         # Generated by the ignored test `record_the_hollow_golden` in\n\
         # game/crates/core/src/world_state/hollow_tests.rs.\n\
         # Each line: <ticks> <move_x> <move_z> [BUTTONS].\n",
    );
    text += &format!("seed {SEED}\nscenario {SCENARIO}\n");
    // Runs of equal frames, split where a command lands: `cmd` lines queue on the tick of the
    // next frame line.
    let names = [
        (buttons::LIGHT, "LIGHT"),
        (buttons::DODGE, "DODGE"),
        (buttons::JUMP, "JUMP"),
        (buttons::USE_ITEM, "USE_ITEM"),
        (buttons::INTERACT, "INTERACT"),
        (buttons::LOCK, "LOCK"),
    ];
    let mut at = 0usize;
    while at < r.frames.len() {
        for (tick, c) in &r.commands {
            if *tick == at as u64 {
                assert_eq!(*c, Command::Rest);
                text += "cmd Rest\n";
            }
        }
        let f = r.frames[at];
        let mut n = 1;
        while at + n < r.frames.len()
            && r.frames[at + n] == f
            && !r.commands.iter().any(|(t, _)| *t == (at + n) as u64)
        {
            n += 1;
        }
        text += &format!("{n} {} {}", f.move_x, f.move_z);
        let held: Vec<&str> = names
            .iter()
            .filter(|(m, _)| f.buttons & m != 0)
            .map(|(_, s)| *s)
            .collect();
        if !held.is_empty() {
            text += &format!(" {}", held.join("+"));
        }
        text.push('\n');
        at += n;
    }
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/replays/m4_waking_hollow.script"
    );
    std::fs::write(path, text).unwrap();
}
