//! Tests for B03, the Bridgewarden Corvane (WP-071), and the scripted bot that beats it.
//!
//! The bot is the proof that the fight is fair (docs/game/05-enemies-and-bosses.md,
//! "B03 Bridgewarden Corvane": read the telegraph, roll through, punish). [`Bot`] reads the world
//! each tick, and returns the player's input: it rolls so that the roll's i-frames cover
//! each boss hit, punishes in the recovery after a combo's last move, and drinks a flask
//! when low in a long window. It is deterministic (no randomness, no clock) and reusable:
//! [`run_bot`] and [`run_idle`] play a whole fight from any seed, for the RL smoke test
//! (WP-039) to use as a baseline.
//!
//! The goldens `m7_b03_bot_win.lfr` and `m7_b03_idle_dies.lfr` come from the bot's inputs:
//! `record_the_b03_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;

use crate::actor::Actor;
use crate::arena::Id;
use crate::combat::moves::MoveNum;
use crate::events::{Event, LifeEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::math::dmath::{cos, sin, sqrt};
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

use super::boss::Stage;

const SCENARIO: &str = "boss_b03";
const BOSS_ID: &str = "b03_bridgewarden_corvane";
/// The seed the goldens use.
const SEED: u64 = 71;
/// The longest fight the runners play, in ticks (about 33 s of game time per 2,000 ticks at
/// 60 Hz is far more than the bot needs; this only stops a stuck run).
const MAX_TICKS: u64 = 6000;

/// Where the player stands relative to the boss.
struct Geometry {
    /// Planar distance, in meters.
    dist: f32,
    /// The stick (-127 to 127) that points at the boss.
    toward: (i8, i8),
    /// How squarely the player faces the boss: 1 is straight at it, -1 straight away.
    facing: f32,
}

/// The scripted player: rolls through hits and punishes the recovery.
pub(crate) struct Bot {
    /// How many ticks before a hit's first frame the bot presses DODGE. The light roll's
    /// i-frames are its frames 2 to 14, so this centers them on a short hit window.
    lead: u16,
    /// The boss move and frame last seen, to tell when a new move starts.
    seen: Option<(MoveNum, u16)>,
    /// The bot has already rolled for the boss move it is watching.
    rolled: bool,
    /// Buttons sent last tick, because every button is a press edge.
    last: u16,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            lead: 7,
            seen: None,
            rolled: false,
            last: 0,
        }
    }
}

fn boss_body(w: &World) -> Id {
    let spawn = w.layout.bosses[0].members[0];
    w.state.ai_enemies[usize::from(spawn)]
}

fn geometry(p: &Actor, boss: &Actor) -> Geometry {
    let (dx, dz) = (boss.pos[0] - p.pos[0], boss.pos[2] - p.pos[2]);
    let dist = sqrt(dx * dx + dz * dz).max(0.001);
    let s = |v: f32| (v / dist * 127.0) as i8;
    // Yaw 0 faces +Z and turns toward +X (the level format's convention).
    let facing = (dx * sin(p.yaw) + dz * cos(p.yaw)) / dist;
    Geometry {
        dist,
        toward: (s(dx), s(dz)),
        facing,
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

    fn decide(&mut self, w: &World) -> ((i8, i8), u16) {
        let Some(player) = w.actors.get(w.player) else {
            return ((0, 0), 0);
        };
        let Some(boss) = w.actors.get(boss_body(w)) else {
            return ((0, 0), 0);
        };
        let g = geometry(player, boss);
        match w.bosses[0].stage {
            Stage::Dormant => return ((0, 127), 0), // walk through the fog gate
            Stage::Intro { .. } | Stage::Defeated => return ((0, 0), 0),
            Stage::Fight => {}
        }
        if player.dead || player.hp == 0 {
            return ((0, 0), 0);
        }
        if player.lock_target.is_none() {
            return ((0, 0), buttons::LOCK);
        }

        // Watch the boss's move: its first and last hit frames and what follows.
        let mut hit: Option<(u16, u16)> = None; // (first, last) hit frame of this move
        let mut finisher = false; // the move ends the combo
        let mut left = 0u16; // frames until the move ends
        let mut frame = 0u16;
        if let Some(m) = boss.moves.current {
            if let Some(def) = w.moves.get(m.num) {
                let first = def.data.hitboxes.iter().map(|h| h.frames.0).min();
                let last = def.data.hitboxes.iter().map(|h| h.frames.1).max();
                hit = first.zip(last);
                finisher = def.next_chain.is_none();
                left = def.data.total_frames.saturating_sub(m.frame);
                frame = m.frame;
            }
            let key = (m.num, m.frame);
            if self.seen.is_none_or(|(n, f)| n != key.0 || f > key.1) {
                self.rolled = false; // a new move
            }
            self.seen = Some(key);
        } else {
            self.seen = None;
            self.rolled = false;
        }
        if player
            .moves
            .current
            .and_then(|c| w.moves.player_moveset.as_ref().map(|s| (c.num, s)))
            .is_some_and(|(n, s)| n == s.roll_light || n == s.roll_medium || n == s.roll_heavy)
        {
            self.rolled = true;
        }

        // 1. Roll through a hit that is about to land.
        if let Some((first, _)) = hit
            && !self.rolled
            && frame < first
            && first - frame <= self.lead
        {
            return (g.toward, buttons::DODGE);
        }
        // 2. A long window after the last hit of a move: heal if low, else punish.
        if let Some((_, last)) = hit
            && finisher
            && frame > last
            && left >= 14
        {
            let low = player.hp * 100 < player.max_hp * 45;
            let flasks = player.inventory.flasks.vigil > 0;
            if low && flasks && left >= 30 {
                return ((0, 0), buttons::USE_ITEM);
            }
            // Rolling through Corvane leaves the player behind him: walk back round and
            // face him before swinging, and never swing from inside the roll.
            let rolling = player.moves.current.is_some();
            if rolling || g.dist > 2.0 || g.facing < 0.8 {
                return (g.toward, 0);
            }
            if player.stamina >= 30.0 {
                return ((0, 0), buttons::LIGHT);
            }
            return ((0, 0), 0);
        }
        // 3. Between attacks: stay close enough that the boss starts its next one.
        if boss.moves.current.is_none() && g.dist > 3.0 {
            return (g.toward, 0);
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

#[test]
fn the_scripted_bot_beats_b03_by_rolling_and_punishing() {
    let f = run_bot(SEED);
    assert!(
        f.boss_defeated(),
        "the boss stands at tick {}",
        f.world.tick
    );
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
}

#[test]
fn a_player_who_does_nothing_dies_to_b03() {
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
fn the_bot_golden_replay_beats_b03_with_the_player_alive() {
    let f = play_golden("m7_b03_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    // The Glimmer paid out (the items go to the inventory).
    assert_eq!(
        f.world
            .actors
            .get(f.world.player)
            .unwrap()
            .inventory
            .glimmer,
        900
    );
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b03_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

#[test]
fn a_hurt_bot_drinks_a_flask_in_a_long_window_and_still_wins() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    let (hp, max) = {
        let p = w.actors.get_mut(w.player).unwrap();
        p.hp = p.max_hp * 3 / 10;
        (p.hp, p.max_hp)
    };
    let before = w.actors.get(w.player).unwrap().inventory.flasks.vigil;
    let mut lowest = before;
    let mut bot = Bot::default();
    let f = play_from(w, |w| {
        let p = w.actors.get(w.player).unwrap();
        lowest = lowest.min(p.inventory.flasks.vigil);
        bot.act(w)
    });
    assert!(f.boss_defeated() && !f.player_died());
    assert!(lowest < before, "no flask was drunk");
    assert!(hp < max);
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b03_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b03_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b03_goldens` in game/crates/core/src/ai/b03_tests.rs.\n\
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
        "m7_b03_bot_win",
        "WP-071 golden: the scripted bot beats B03 by rolling through each hit and punishing.",
        &run_bot(SEED),
    );
    write(
        "m7_b03_idle_dies",
        "WP-071 golden: a player who walks in and does nothing dies to B03.",
        &run_idle(SEED),
    );
}

/// A Corvane move, by content ID.
fn content_move(id: &str) -> lf_content::moves::Move {
    Content::load_embedded()
        .unwrap()
        .move_by_id(id)
        .unwrap_or_else(|| panic!("no move {id}"))
        .clone()
}

fn first_hit(id: &str) -> u16 {
    content_move(id)
        .hitboxes
        .iter()
        .map(|h| h.frames.0)
        .min()
        .unwrap()
}

#[test]
fn every_b03_hit_is_parryable_and_the_third_hit_of_the_combo_is_delayed() {
    use lf_content::moves::HitTag;
    for id in [
        "b03_shield_advance",
        "b03_combo_1",
        "b03_combo_2",
        "b03_combo_3",
        "b03_sweep",
        "b03_lunge",
    ] {
        let m = content_move(id);
        assert!(!m.hitboxes.is_empty(), "{id}");
        for h in &m.hitboxes {
            assert!(
                !h.tags.contains(&HitTag::Unparryable),
                "{id} is unparryable"
            );
            assert!(!h.tags.contains(&HitTag::Unblockable), "{id}");
        }
    }
    // Docs: the third hit is delayed; the first two come at a steady pace (the delay is only
    // ever on the second or later hit of a combo).
    let (a, b, c) = (
        first_hit("b03_combo_1"),
        first_hit("b03_combo_2"),
        first_hit("b03_combo_3"),
    );
    assert!(a >= 12 && b >= 12, "telegraphs {a} and {b}");
    assert!(
        c >= b + 14,
        "the third hit ({c}) isn't delayed past the second ({b})"
    );
    // The shout's telegraph is 18 ticks.
    assert_eq!(first_hit("b03_lunge"), 18);
}

/// Walks the player in, puts them `distance_m` south of Corvane, waits for a free moment, has
/// the player drink, and reports whether Corvane starts the lunge within 24 ticks.
fn lunges_after_a_flask(distance_m: f32) -> bool {
    let content = Content::load_embedded().unwrap();
    let lunge = content.registry.num("b03_lunge").unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    while w.bosses[0].stage != Stage::Fight {
        let walking = w.bosses[0].stage == Stage::Dormant;
        let frame = ActionFrame {
            move_x: 0,
            move_z: if walking { 127 } else { 0 },
            buttons: 0,
        };
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        assert!(w.tick < 2000);
    }
    let boss_pos = w.actors.get(boss_body(&w)).unwrap().pos;
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = [boss_pos[0], boss_pos[1], boss_pos[2] - distance_m];
    p.hp = p.max_hp / 2;
    for _ in 0..400 {
        let free = w.actors.get(boss_body(&w)).unwrap().moves.current.is_none();
        let frame = ActionFrame {
            move_x: 0,
            move_z: 0,
            buttons: if free { buttons::USE_ITEM } else { 0 },
        };
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        if free {
            for _ in 0..24 {
                let boss = w.actors.get(boss_body(&w)).unwrap();
                if boss.moves.current.is_some_and(|m| m.num == lunge) {
                    return true;
                }
                tick(&mut w, &TickInput::default());
            }
            return false;
        }
    }
    false
}

#[test]
fn drinking_a_flask_within_6_m_is_answered_with_the_lunge() {
    assert!(lunges_after_a_flask(4.0));
}

#[test]
fn drinking_a_flask_beyond_6_m_is_not() {
    assert!(!lunges_after_a_flask(9.0));
}
