//! Tests for B05, the Ossuary Keeper (WP-073), and the scripted bot that beats it.
//!
//! The bot reads the world each tick and returns the player's input. Every attack in the
//! fight (the Keeper's, a thrall's, a Drowned's) is a hitbox on a frame window, so the bot
//! looks at each hostile actor's move (and the moves it chains into), works out when a hitbox
//! will next overlap the player, and rolls so that the roll's i-frames cover that frame. It
//! punishes in the long recovery after the Keeper's finishers and cuts down an add that
//! stands close. It is deterministic (no randomness, no clock): [`run_bot`] and [`run_idle`]
//! play a whole fight from any seed.
//!
//! The goldens `m7_b05_bot_win.lfr` and `m7_b05_idle_dies.lfr` come from the bot's inputs:
//! `record_the_b05_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;

use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::arena::Id;
use crate::combat::hitbox::{Capsule, Pose};
use crate::events::{BossEvent, Event, LifeEvent, MechanicEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;
use crate::world_state::mechanics::water_level;

use super::b05::{ARMED, DROWNED_UP, POOL, TIDE};

const SCENARIO: &str = "boss_b05";
const BOSS_ID: &str = "b05_ossuary_keeper";
/// The seed the goldens use.
const SEED: u64 = 7;
/// The longest fight the runners play, in ticks (a stuck run only).
const MAX_TICKS: u64 = 30_000;

fn boss_body(w: &World) -> Id {
    member(w, 0)
}

fn member(w: &World, i: usize) -> Id {
    let spawn = w.layout.bosses[0].members[i];
    w.state.ai_enemies[usize::from(spawn)]
}

fn dist_xz(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (dx, dz) = (a[0] - b[0], a[2] - b[2]);
    sqrt(dx * dx + dz * dz)
}

/// The shortest distance between a point and the segment `a`-`b` (in 3D).
fn point_segment(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if len2 <= 1e-6 {
        0.0
    } else {
        ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / len2).clamp(0.0, 1.0)
    };
    let c = [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t];
    let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
    sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2])
}

/// How far past a hitbox's own radius the bot still counts itself as threatened by an add,
/// in meters: the player's body, and the add stepping during its wind-up.
const ADD_MARGIN_M: f32 = 0.9;
/// The same for the Keeper, which also turns toward the player and steps through a chain.
const KEEPER_MARGIN_M: f32 = 2.0;
/// How many ticks the bot holds the stick after pressing DODGE.
const ROLL_HOLD_TICKS: u64 = 6;
/// A hit from an add is only rolled for when the Keeper has none within this many ticks, so
/// that the roll is over before the Keeper's next.
const KEEPER_CLEAR_TICKS: u16 = 30;
/// The bot starts or chains a swing only if no hit is due within this many ticks: the swing's
/// dodge cancel (frame 17) plus the lead (7) plus a margin.
const SWING_CLEAR_TICKS: u16 = 26;

/// How many ticks from now a hitbox of `attacker`'s current move, or of the moves it chains
/// into, will next overlap the player's body. `None` if none will.
fn next_threat(w: &World, attacker: &Actor, player: &Actor, margin: f32) -> Option<u16> {
    let c = attacker.moves.current?;
    let pose = Pose {
        pos: attacker.pos,
        yaw: attacker.yaw,
    };
    // The player's body, sampled at the hip and the head.
    let body = [
        [player.pos[0], player.pos[1] + 0.5, player.pos[2]],
        [player.pos[0], player.pos[1] + 1.4, player.pos[2]],
    ];
    let mut def = w.moves.get(c.num)?;
    let mut frame = c.frame;
    let mut offset = 0u16; // ticks until the move being looked at starts
    for _ in 0..4 {
        let soonest = def
            .data
            .hitboxes
            .iter()
            .filter(|h| h.frames.1 >= frame)
            .filter(|h| {
                let cap = Capsule::from_shape(&h.shape, &pose);
                body.iter()
                    .any(|&p| point_segment(p, cap.a, cap.b) <= cap.radius + margin)
            })
            .map(|h| h.frames.0.max(frame) - frame)
            .min();
        if let Some(t) = soonest {
            return Some(offset + t);
        }
        offset += def.data.total_frames.saturating_sub(frame);
        frame = 0;
        def = w.moves.get(def.next_chain?)?;
    }
    None
}

/// The scripted player: rolls through hits, punishes the recovery, and cuts down adds.
pub(crate) struct Bot {
    /// How many ticks before a hit's first frame the bot presses DODGE. The medium roll's
    /// i-frames are its frames 2 to 14, so this centers them on a short hit window.
    lead: u16,
    /// Buttons sent last tick, because every button is a press edge.
    last: u16,
    /// The tick the bot last pressed DODGE.
    rolled_at: u64,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            lead: 7,
            last: 0,
            rolled_at: 0,
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
        if pressed & buttons::DODGE != 0 {
            self.rolled_at = w.tick;
        }
        ActionFrame {
            move_x: stick.0,
            move_z: stick.1,
            buttons: pressed,
        }
    }

    fn toward(from: &Actor, to: &Actor) -> (i8, i8) {
        let (dx, dz) = (to.pos[0] - from.pos[0], to.pos[2] - from.pos[2]);
        let dist = sqrt(dx * dx + dz * dz).max(0.001);
        let s = |v: f32| (v / dist * 127.0) as i8;
        (s(dx), s(dz))
    }

    fn decide(&mut self, w: &World) -> ((i8, i8), u16) {
        let Some(player) = w.actors.get(w.player) else {
            return ((0, 0), 0);
        };
        let Some(keeper) = w.actors.get(boss_body(w)) else {
            return ((0, 0), 0);
        };
        match w.bosses[0].stage {
            Stage::Dormant => return ((0, 127), crate::ai::boss::fog_buttons(w)), // walk through the fog gate
            Stage::Intro { .. } | Stage::Defeated => return ((0, 0), 0),
            Stage::Fight => {}
        }
        if player.dead || player.hp == 0 {
            return ((0, 0), 0);
        }
        let toward = Self::toward(player, keeper);
        // A dodge is read when the roll starts, which can be a few ticks after the press (it
        // waits for the bot's own swing to be cancellable): keep the stick on the roll's
        // direction meanwhile, or the dodge becomes a backstep.
        if w.tick < self.rolled_at + ROLL_HOLD_TICKS {
            return (toward, 0);
        }
        let rolling = player
            .moves
            .current
            .and_then(|c| w.moves.player_moveset.as_ref().map(|s| (c.num, s)))
            .is_some_and(|(n, s)| n == s.roll_light || n == s.roll_medium || n == s.roll_heavy);

        // 1. Roll through the soonest hit that is about to land. The Keeper's come first.
        let n = w.layout.bosses[0].members.len();
        let threat = |i: usize| {
            let a = w.actors.get(member(w, i))?;
            if a.dead || a.hp == 0 {
                return None;
            }
            next_threat(
                w,
                a,
                player,
                if i == 0 {
                    KEEPER_MARGIN_M
                } else {
                    ADD_MARGIN_M
                },
            )
        };
        let keeper_ahead = threat(0);
        let adds_ahead = (1..n).filter_map(threat).min();
        // The Keeper's hit is rolled for when it is close; an add's only when the Keeper has
        // none soon after, so the roll is over in time.
        let want_roll = match (keeper_ahead, adds_ahead) {
            (Some(k), _) if k <= self.lead => true,
            (k, Some(a)) => a <= self.lead && k.is_none_or(|k| k > KEEPER_CLEAR_TICKS),
            _ => false,
        };
        if want_roll && !rolling && w.tick >= self.rolled_at + 12 && player.stamina >= 14.0 {
            return (toward, buttons::DODGE);
        }
        let any_ahead = keeper_ahead.is_some() || adds_ahead.is_some();
        // A swing commits the bot until the swing's dodge cancel (frame 17 of 27), and an
        // unanswered press queues the next swing of the chain. So swing only with the player
        // free, or at the cancel into the next swing, and no hit due in the next SWING_CLEAR
        // ticks.
        let soonest = keeper_ahead.into_iter().chain(adds_ahead).min();
        let free = match player.moves.current {
            None => true,
            Some(c) => w
                .moves
                .get(c.num)
                .and_then(|d| d.data.cancel_attack_from)
                .is_some_and(|from| c.frame >= from && !rolling),
        };
        let swing = |stick| {
            if free && player.stamina >= 30.0 && soonest.is_none_or(|t| t >= SWING_CLEAR_TICKS) {
                (stick, buttons::LIGHT)
            } else {
                (stick, 0)
            }
        };

        // 2. A long window after the Keeper's finisher: heal if low, else punish.
        let mut left = 0u16;
        if let Some(m) = keeper.moves.current
            && let Some(def) = w.moves.get(m.num)
            && let Some(last) = def.data.hitboxes.iter().map(|h| h.frames.1).max()
            && def.next_chain.is_none()
            && m.frame > last
        {
            left = def.data.total_frames.saturating_sub(m.frame);
        }
        if left >= 14 && !any_ahead {
            let low = player.hp * 100 < player.max_hp * 45;
            let flasks = player.inventory.flasks.vigil > 0;
            if low && flasks && left >= 30 {
                return ((0, 0), buttons::USE_ITEM);
            }
            if dist_xz(player.pos, keeper.pos) > 2.2 {
                return (toward, 0);
            }
            return swing((0, 0));
        }
        if any_ahead {
            return ((0, 0), 0);
        }
        // Quiet and hurt: drink a flask if nothing is close.
        let near_add = (1..n)
            .filter_map(|i| w.actors.get(member(w, i)))
            .any(|a| !a.dead && a.hp > 0 && dist_xz(a.pos, player.pos) < 3.0);
        if player.hp * 100 < player.max_hp * 40
            && player.inventory.flasks.vigil > 0
            && !near_add
            && dist_xz(player.pos, keeper.pos) > 3.5
        {
            return ((0, 0), buttons::USE_ITEM);
        }
        // 3. Quiet: cut down an add that is close, else keep close to the Keeper.
        let close_add = (1..n)
            .filter_map(|i| w.actors.get(member(w, i)))
            .filter(|a| !a.dead && a.hp > 0 && dist_xz(a.pos, player.pos) < 3.5)
            .min_by(|a, b| dist_xz(a.pos, player.pos).total_cmp(&dist_xz(b.pos, player.pos)));
        if let Some(add) = close_add {
            if dist_xz(add.pos, player.pos) > 1.9 {
                return (Self::toward(player, add), 0);
            }
            return swing(Self::toward(player, add));
        }
        if dist_xz(player.pos, keeper.pos) > 3.0 {
            return (toward, 0);
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
        buttons: crate::ai::boss::fog_buttons(w),
    })
}

#[test]
fn the_arena_loads_with_the_keeper_and_five_adds_down() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(1, &content, SCENARIO).unwrap();
    assert_eq!(w.layout.bosses.len(), 1);
    assert_eq!(w.layout.bosses[0].members.len(), 6);
    // The adds lie down from the first tick, before anyone walks in.
    tick(&mut w, &TickInput::default());
    assert!(!w.actors.get(member(&w, 0)).unwrap().dead);
    for i in 1..6 {
        assert!(w.actors.get(member(&w, i)).unwrap().dead, "member {i}");
    }
}

/// Prints the bot's result for many seeds. `cargo test -p lf-core b05_seeds -- --ignored
/// --nocapture`.
#[test]
#[ignore]
fn b05_seeds() {
    let from: u64 = std::env::var("FROM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let to: u64 = std::env::var("TO")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24);
    for seed in from..=to {
        let f = run_bot(seed);
        let k = f.world.actors.get(boss_body(&f.world)).unwrap();
        println!(
            "seed {seed}: won {} died {} tick {} keeper {}",
            f.boss_defeated(),
            f.player_died(),
            f.world.tick,
            k.hp
        );
    }
}

/// Prints how a bot fight goes. Run it with `cargo test -p lf-core b05_probe -- --ignored
/// --nocapture`; set `RING=1` to see the 50 ticks before each hit on the player.
#[test]
#[ignore]
fn b05_probe() {
    let content = Content::load_embedded().unwrap();
    let seed = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(SEED);
    let mut w = World::new(seed, &content, SCENARIO).unwrap();
    let mut bot = Bot::default();
    let mut ring: Vec<String> = Vec::new();
    let mut last_move = None;
    while w.tick < MAX_TICKS {
        let frame = bot.act(&w);
        {
            let p = w.actors.get(w.player).unwrap();
            let pm = p.moves.current.map(|c| (c.num, c.frame));
            let k = w.actors.get(boss_body(&w)).unwrap();
            let km = k
                .moves
                .current
                .map(|c| (w.moves.get(c.num).unwrap().data.id.clone(), c.frame));
            ring.push(format!(
                "t{} thr {:?} kpos {:?} kyaw {:.2} ppos {:?} btn {} stick {},{} pmove {:?} km {:?} dist {:.1} stam {}",
                w.tick,
                next_threat(&w, k, p, KEEPER_MARGIN_M),
                k.pos, k.yaw, p.pos,
                frame.buttons,
                frame.move_x,
                frame.move_z,
                pm,
                km,
                dist_xz(p.pos, k.pos),
                p.stamina
            ));
            if ring.len() > 50 {
                ring.remove(0);
            }
            let now = k.moves.current.map(|c| c.num);
            if now != last_move && std::env::var("MOVES").is_ok() {
                last_move = now;
                if let Some(c) = k.moves.current {
                    println!(
                        "t{} keeper move {}",
                        w.tick,
                        w.moves.get(c.num).unwrap().data.id
                    );
                }
            }
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
                Event::Hit(h) => {
                    let id = |a: Id| {
                        (0..6)
                            .find(|&i| member(&w, i) == a)
                            .map_or("player".to_owned(), |i| format!("m{i}"))
                    };
                    let name = w
                        .moves
                        .get(h.move_num)
                        .map_or("?".to_owned(), |d| d.data.id.clone());
                    if h.target == w.player && std::env::var("RING").is_ok() {
                        for l in &ring {
                            println!("   {l}");
                        }
                    }
                    println!(
                        "t{} hit {} -> {} {} dmg {} {:?} hp {}",
                        w.tick,
                        id(h.attacker),
                        id(h.target),
                        name,
                        h.damage,
                        h.result,
                        h.hp
                    );
                }
                Event::Boss(b) => println!("t{} boss {} phase {}", w.tick, b.kind, b.phase),
                Event::Mechanic(m) => println!("t{} mechanic {} {}", w.tick, m.kind, m.index),
                Event::Life(LifeEvent::Died { .. }) => println!("t{} player died", w.tick),
                _ => {}
            }
        }
        if w.bosses[0].stage == Stage::Defeated || w.actors.get(w.player).unwrap().hp == 0 {
            break;
        }
    }
    let p = w.actors.get(w.player).unwrap();
    let k = w.actors.get(boss_body(&w)).unwrap();
    println!(
        "end t{} player {}/{} keeper {}/{}",
        w.tick, p.hp, p.max_hp, k.hp, k.max_hp
    );
}

// ---------------------------------------------------------------------------------------
// The fight, the goldens, and the mechanics

#[test]
fn the_scripted_bot_beats_b05_by_rolling_punishing_and_cutting_down_adds() {
    let f = run_bot(SEED);
    assert!(
        f.boss_defeated(),
        "the Keeper stands at tick {}",
        f.world.tick
    );
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    // The whole fight happened: Raise, the tide, the Drowned, and the borrowed weapon.
    let kinds = mechanic_kinds(&f.events);
    for k in [
        MechanicEvent::B05_RAISED,
        MechanicEvent::B05_TIDE_RISING,
        MechanicEvent::B05_DROWNED_JOIN,
        MechanicEvent::B05_WEAPON_TAKEN,
    ] {
        assert!(kinds.contains(&k), "no mechanic event {k}");
    }
    // The rewards paid out: Glimmer, and the Bone Sentinel Echo.
    assert_eq!(
        f.world
            .actors
            .get(f.world.player)
            .unwrap()
            .inventory
            .glimmer,
        800
    );
    assert!(f.world.state.flags.is_set("echo.bone_sentinel.unlocked"));
    assert!(f.world.state.flags.is_set("codex.b05_ossuary_keeper"));
}

#[test]
fn a_player_who_does_nothing_dies_to_b05() {
    let f = run_idle(SEED);
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

#[test]
fn the_bot_wins_from_other_seeds_too() {
    for seed in [1, 26, 99] {
        let f = run_bot(seed);
        assert!(f.boss_defeated() && !f.player_died(), "seed {seed}");
    }
}

fn replay(name: &str) -> Replay {
    let path = format!(
        "{}/../../tests/replays/{name}.lfr",
        env!("CARGO_MANIFEST_DIR")
    );
    Replay::decode(&std::fs::read(path).unwrap()).unwrap()
}

/// Replays a golden, returning the fight.
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
fn the_bot_golden_replay_beats_b05_with_the_player_alive() {
    let f = play_golden("m7_b05_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b05_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b05_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b05_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b05_goldens` in game/crates/core/src/ai/bosses/b05_tests.rs.\n\
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
        "m7_b05_bot_win",
        "WP-073 golden: the scripted bot beats B05 by rolling through each hit, punishing the finisher, and cutting down adds.",
        &run_bot(SEED),
    );
    write(
        "m7_b05_idle_dies",
        "WP-073 golden: a player who walks in and does nothing dies to B05.",
        &run_idle(SEED),
    );
}

/// A world with the fight started and a player who can't die, standing in the vault.
fn live_fight() -> World {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    {
        let p = w.actors.get_mut(w.player).unwrap();
        p.pos = [0.0, 0.0, 23.5];
        p.hp = 1_000_000;
        p.max_hp = 1_000_000;
    }
    while w.bosses[0].stage != Stage::Fight {
        tick(&mut w, &TickInput::default());
        assert!(w.tick < 400, "the fight never started");
    }
    let _ = w.events.drain();
    w
}

fn run_ticks(w: &mut World, n: u32) -> Vec<Event> {
    let mut events = Vec::new();
    for _ in 0..n {
        tick(w, &TickInput::default());
        events.extend(w.events.drain());
    }
    events
}

fn mechanic_kinds(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Mechanic(m) => Some(m.kind),
            _ => None,
        })
        .collect()
}

fn keeper_hp_to(w: &mut World, percent: u32) {
    let id = boss_body(w);
    let a = w.actors.get_mut(id).unwrap();
    a.hp = a.max_hp * percent / 100;
}

fn pool_depth(w: &World) -> f32 {
    let i = w.layout.pools.iter().position(|p| p.name == POOL).unwrap();
    water_level::depth(w, i)
}

/// Puts every option of the Keeper except `keep` on a long cooldown.
fn only_option(w: &mut World, keep: &str) {
    let k = boss_body(w);
    let keep = w.moves.num_of(keep).unwrap();
    let def = w.actors.get(k).unwrap().ai.as_ref().unwrap().def;
    let skip: Vec<bool> =
        w.ai.def(def)
            .options
            .iter()
            .map(|o| !matches!(o.kind, crate::ai::OptionKind::Move { num, .. } if num == keep))
            .collect();
    let brain = w.actors.get_mut(k).unwrap().ai.as_mut().unwrap();
    for (i, s) in skip.into_iter().enumerate() {
        if s {
            brain.cooldowns[i] = 5000;
        }
    }
}

#[test]
fn the_adds_lie_down_until_raised_and_the_pool_starts_at_0_4_m() {
    let w = live_fight();
    for i in 1..6 {
        assert!(w.actors.get(member(&w, i)).unwrap().dead, "member {i}");
    }
    assert_eq!(pool_depth(&w), 0.4);
    assert!(!w.state.flags.is_set(TIDE));
}

#[test]
fn raise_stands_the_fallen_thralls_up_and_nothing_else() {
    let mut w = live_fight();
    only_option(&mut w, "b05_raise");
    let events = run_ticks(&mut w, 400);
    let raised: Vec<u16> = events
        .iter()
        .filter_map(|e| match e {
            Event::Mechanic(m) if m.kind == MechanicEvent::B05_RAISED => Some(m.index),
            _ => None,
        })
        .collect();
    assert_eq!(raised, [3]);
    for i in 1..=3 {
        let a = w.actors.get(member(&w, i)).unwrap();
        assert!(!a.dead && a.hp == a.max_hp, "thrall {i} is up");
    }
    // The Drowned wait for phase 2.
    for i in 4..6 {
        assert!(w.actors.get(member(&w, i)).unwrap().dead, "drowned {i}");
    }
}

#[test]
fn raise_is_locked_while_no_thrall_has_fallen() {
    let mut w = live_fight();
    only_option(&mut w, "b05_raise");
    run_ticks(&mut w, 400);
    // All three are up now; the Raise option is the only one off cooldown, and it is locked.
    let k = boss_body(&w);
    let raise = w.moves.num_of("b05_raise").unwrap();
    for _ in 0..1500 {
        run_ticks(&mut w, 1);
        let cur = w.actors.get(k).unwrap().moves.current.map(|c| c.num);
        assert_ne!(cur, Some(raise));
    }
}

#[test]
fn phase_2_raises_the_water_0_4_m_and_wakes_the_drowned_after_the_transition() {
    let mut w = live_fight();
    keeper_hp_to(&mut w, 49);
    let events = run_ticks(&mut w, 10);
    assert!(events.iter().any(
        |e| matches!(e, Event::Boss(b) if b.kind == BossEvent::PHASE_STARTED && b.phase == 2)
    ));
    // The tide is in from the start of the transition move.
    assert!(w.state.flags.is_set(TIDE));
    assert!((pool_depth(&w) - 0.8).abs() < 1e-6);
    assert!(w.state.flags.is_set(&flags::pool_drag(POOL)));
    // The Drowned wait for the transition to end (45 ticks), then stand up.
    for i in 4..6 {
        assert!(w.actors.get(member(&w, i)).unwrap().dead);
    }
    assert!(!w.state.flags.is_set(DROWNED_UP));
    let events = run_ticks(&mut w, 60);
    assert!(w.state.flags.is_set(DROWNED_UP));
    assert!(mechanic_kinds(&events).contains(&MechanicEvent::B05_DROWNED_JOIN));
    for i in 4..6 {
        assert!(!w.actors.get(member(&w, i)).unwrap().dead, "drowned {i}");
    }
}

/// How far the player walks along x in 30 ticks from (-10, 0, 30).
fn walked(w: &mut World) -> f32 {
    {
        let p = w.actors.get_mut(w.player).unwrap();
        p.pos = [-10.0, 0.0, 30.0];
        p.vel = [0.0; 3];
    }
    let input = TickInput {
        frame: ActionFrame {
            move_x: -127,
            move_z: 0,
            buttons: 0,
        },
        commands: Vec::new(),
    };
    for _ in 0..30 {
        tick(w, &input);
    }
    (w.actors.get(w.player).unwrap().pos[0] + 10.0).abs()
}

#[test]
fn the_tide_slows_the_player_more_than_the_shallows() {
    let mut shallow = live_fight();
    let mut tide = live_fight();
    keeper_hp_to(&mut tide, 49);
    run_ticks(&mut tide, 80);
    assert!(tide.state.flags.is_set(TIDE));
    let (a, b) = (walked(&mut shallow), walked(&mut tide));
    // 20% slower in the shallows, and a second 20% in the tide: 0.8 against 0.64.
    assert!(b < a * 0.9, "shallow {a} tide {b}");
    assert!(b > a * 0.6, "shallow {a} tide {b}");
}

#[test]
fn the_scavenge_arms_the_keeper_and_unlocks_the_borrowed_moves() {
    let mut w = live_fight();
    keeper_hp_to(&mut w, 49);
    run_ticks(&mut w, 10);
    let k = boss_body(&w);
    // The thralls lie down and the Keeper is unarmed: of the new moves only the scavenge can
    // start.
    assert!(!w.state.flags.is_set(ARMED));
    let borrowed: Vec<_> = ["b05_borrow_thrust", "b05_borrow_bash"]
        .iter()
        .map(|m| w.moves.num_of(m).unwrap())
        .collect();
    let scavenge = w.moves.num_of("b05_scavenge").unwrap();
    let mut started = Vec::new();
    for _ in 0..2500 {
        run_ticks(&mut w, 1);
        if let Some(c) = w.actors.get(k).unwrap().moves.current
            && started.last() != Some(&c.num)
        {
            started.push(c.num);
        }
        if w.state.flags.is_set(ARMED) {
            break;
        }
    }
    assert!(w.state.flags.is_set(ARMED), "never armed");
    assert!(started.contains(&scavenge));
    assert!(
        !started.iter().any(|n| borrowed.contains(n)),
        "a borrowed move before the weapon was taken"
    );
    // Now armed, a borrowed move comes up.
    let mut seen = false;
    for _ in 0..2500 {
        run_ticks(&mut w, 1);
        if let Some(c) = w.actors.get(k).unwrap().moves.current
            && borrowed.contains(&c.num)
        {
            seen = true;
            break;
        }
    }
    assert!(seen, "no borrowed move after arming");
}

#[test]
fn defeat_unlocks_the_echo_and_lowers_the_water() {
    let mut w = live_fight();
    keeper_hp_to(&mut w, 40);
    run_ticks(&mut w, 120);
    assert!(w.state.flags.is_set(TIDE));
    let id = boss_body(&w);
    {
        let a = w.actors.get_mut(id).unwrap();
        a.hp = 0;
        a.dead = true;
    }
    run_ticks(&mut w, 5);
    assert_eq!(w.bosses[0].stage, Stage::Defeated);
    assert!(w.state.flags.is_set("echo.bone_sentinel.unlocked"));
    assert!(w.state.flags.is_set(&flags::boss_defeated(BOSS_ID)));
    assert!(!w.state.flags.is_set(TIDE));
    assert_eq!(pool_depth(&w), 0.4);
    for i in 1..6 {
        assert!(w.actors.get(member(&w, i)).unwrap().dead);
    }
}

#[test]
fn the_players_death_resets_the_fight_and_the_adds_lie_down_again() {
    let mut w = live_fight();
    keeper_hp_to(&mut w, 49);
    run_ticks(&mut w, 120);
    assert!(w.state.flags.is_set(TIDE));
    {
        let p = w.actors.get_mut(w.player).unwrap();
        p.hp = 0;
        p.dead = true;
    }
    run_ticks(&mut w, 3);
    assert_eq!(w.bosses[0].stage, Stage::Dormant);
    assert!(!w.state.flags.is_set(TIDE));
    assert!(!w.state.flags.is_set(DROWNED_UP));
    assert!(!w.state.flags.is_set(ARMED));
    assert_eq!(pool_depth(&w), 0.4);
    let k = w.actors.get(boss_body(&w)).unwrap();
    assert_eq!(k.hp, k.max_hp);
    for i in 1..6 {
        assert!(w.actors.get(member(&w, i)).unwrap().dead, "member {i}");
    }
}
