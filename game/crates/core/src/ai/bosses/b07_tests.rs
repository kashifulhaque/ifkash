//! Tests for B07, Castellan Voss (WP-075), and the scripted bot that beats him.
//!
//! The bot plays the fight as the spec describes it (docs/game/05-enemies-and-bosses.md, "B07
//! Castellan Voss, the Overfit Warden": the fight rewards varied play). In phase 1 it rolls to its
//! own left through every hit, which the core counts as "roll left"; in phase 2 it rolls through
//! him instead (a roll toward him isn't counted, and his counter for rolling left is the wide
//! sweep that its roll through already beats), steps out of the chandeliers' markers, punishes the
//! recovery after the last hit of each move, and never strikes his guard. It is deterministic (no
//! randomness, no clock), and [`run_bot`] and [`run_idle`] play a whole fight from any seed.
//!
//! The goldens `m7_b07_bot_win.lfr` and `m7_b07_idle_dies.lfr` come from the bot's inputs:
//! `record_the_b07_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;

use super::b07::{self, BOSS_ID, CATEGORIES, COUNTER_MOVES};
use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::arena::Id;
use crate::combat::moves::{ActiveMove, MoveNum, flags as move_flags};
use crate::events::{BossEvent, Event, LifeEvent, MechanicEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::math::dmath::{cos, sin, sqrt};
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

const SCENARIO: &str = "boss_b07";
/// The seed the goldens use.
const SEED: u64 = 75;
/// The longest fight the runners play, in ticks. This only stops a stuck run.
const MAX_TICKS: u64 = 24_000;

fn body_of(w: &World) -> Id {
    let spawn = w.layout.bosses[0].members[0];
    w.state.ai_enemies[usize::from(spawn)]
}

fn actor(w: &World, id: Id) -> &Actor {
    w.actors.get(id).unwrap()
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    sqrt((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2))
}

/// The stick (-127 to 127) that points from `from` to `to`.
fn toward(from: [f32; 3], to: [f32; 3]) -> (i8, i8) {
    let d = dist(from, to).max(0.001);
    let s = |v: f32| (v / d * 127.0) as i8;
    (s(to[0] - from[0]), s(to[2] - from[2]))
}

/// The scripted player.
pub(crate) struct Bot {
    /// How many ticks before a hit's first frame the bot presses DODGE (the light roll's
    /// i-frames are its frames 2 to 14).
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
        let boss = actor(w, body_of(w));
        let to_boss = toward(player.pos, boss.pos);
        let d = dist(player.pos, boss.pos).max(0.001);
        // Yaw 0 faces +Z and turns toward +X (the level format's convention).
        let facing = ((boss.pos[0] - player.pos[0]) * sin(player.yaw)
            + (boss.pos[2] - player.pos[2]) * cos(player.yaw))
            / d;
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
        // Phase 1: roll to the player's own left (the core counts it). Phase 2: roll through him.
        let roll_dir = if w.bosses[0].phase <= 1 {
            // The player's left, as it faces him: the way to him turned a quarter toward -X
            // (yaw 0 faces +Z, and +X is its right).
            // The roll goes 60 degrees to the left of straight at him, which the core counts as a
            // roll left (it is past the 45-degree sector) and which leaves the bot near him.
            let (tx, tz) = (f32::from(to_boss.0), f32::from(to_boss.1));
            let (lx, lz) = (-tz, tx);
            ((0.5 * tx + 0.866 * lx) as i8, (0.5 * tz + 0.866 * lz) as i8)
        } else {
            to_boss
        };

        // Watch the boss's move: its first and last hit frames and what follows.
        let mut hit: Option<(u16, u16)> = None;
        let mut finisher = false;
        let mut left = 0u16;
        let mut frame = 0u16;
        let mut guarding = false;
        if let Some(m) = boss.moves.current {
            if let Some(def) = w.moves.get(m.num) {
                let first = def.data.hitboxes.iter().map(|h| h.frames.0).min();
                let last = def.data.hitboxes.iter().map(|h| h.frames.1).max();
                hit = first.zip(last);
                finisher = def.next_chain.is_none();
                left = def.data.total_frames.saturating_sub(m.frame);
                frame = m.frame;
                guarding = def.data.id == "b07_guard";
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
            return (roll_dir, buttons::DODGE);
        }
        // 2. A chandelier marker under the bot: step out of it (and clear of its edge).
        let radius = w.layout.bosses[0].def.params["chandelier_radius_m"] + 1.2;
        for m in w.b07.markers.iter().filter(|m| m.left > 0) {
            if dist(player.pos, [m.x, 0.0, m.z]) < radius {
                return (toward([m.x, 0.0, m.z], player.pos), 0);
            }
        }
        // 3. A long window after the last hit of a move: heal if low, else punish. Never in
        //    his guard stance (before its shove), which would be blocked.
        if let Some((_, last)) = hit
            && finisher
            && !guarding
            && frame > last
            && left >= 14
        {
            let low = player.hp * 100 < player.max_hp * 45;
            let flasks = player.inventory.flasks.vigil > 0;
            if low && flasks && left >= 30 {
                return ((0, 0), buttons::USE_ITEM);
            }
            // Already swinging: keep the light chain going while the window lasts.
            let rolling = player
                .moves
                .current
                .and_then(|c| w.moves.player_moveset.as_ref().map(|s| (c.num, s)))
                .is_some_and(|(n, s)| n == s.roll_light || n == s.roll_medium || n == s.roll_heavy);
            if player.moves.current.is_some() && !rolling {
                return ((0, 0), buttons::LIGHT);
            }
            // A roll leaves the player beside or behind him: walk back round and face him
            // before swinging, and never swing from inside the roll.
            if rolling || d > 2.0 || facing < 0.8 {
                return (to_boss, 0);
            }
            if player.stamina >= 30.0 {
                return ((0, 0), buttons::LIGHT);
            }
            return ((0, 0), 0);
        }
        // 4. Between attacks: stay close enough that the boss starts its next one, and take a
        //    swing at him while he has nothing started (he telegraphs at least 12 ticks, and the
        //    swing is a short one).
        if boss.moves.current.is_none() {
            if d > 2.0 || facing < 0.8 {
                return (to_boss, 0);
            }
            if player.moves.current.is_none() && player.stamina >= 30.0 && !self.rolled {
                return ((0, 0), buttons::LIGHT);
            }
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

    fn mechanics(&self, kind: u8) -> Vec<u16> {
        self.events
            .iter()
            .filter_map(|e| match e {
                Event::Mechanic(m) if m.kind == kind => Some(m.index),
                _ => None,
            })
            .collect()
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

// ---------------------------------------------------------------------------------------
// Helpers for the white-box tests
// ---------------------------------------------------------------------------------------

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.motor.ground_y = pos[1];
    p.motor.fall_peak_y = pos[1];
}

fn idle(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut out = Vec::new();
    for _ in 0..ticks {
        tick(w, &TickInput::default());
        out.extend(w.events.drain());
    }
    out
}

/// A world with the fight on: the player walked through the gate and the intro is over, and
/// the player stands 6 m south of Voss with plenty of HP.
fn fight_world() -> World {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    put_player(&mut w, [0.0, 0.0, -1.0]);
    idle(&mut w, 100);
    assert_eq!(w.bosses[0].stage, Stage::Fight);
    put_player(&mut w, [0.0, 0.0, 16.0]);
    let p = w.actors.get_mut(w.player).unwrap();
    p.max_hp = 1_000_000;
    p.hp = 1_000_000;
    w
}

fn move_num(w: &World, id: &str) -> MoveNum {
    w.moves.num_of(id).unwrap_or_else(|| panic!("no move {id}"))
}

fn start_move(w: &mut World, id: Id, num: MoveNum, frame: u16) {
    w.actors.get_mut(id).unwrap().moves.current = Some(ActiveMove {
        num,
        frame,
        charge_ticks: 0,
        hold_button: 0,
        free_from: 999,
    });
}

/// The fight in phase 2: counts are set as given, Voss's HP is under the threshold, and the
/// transition has played.
fn phase_two_world(counts: [u16; CATEGORIES]) -> World {
    let mut w = fight_world();
    w.b07.counts = counts;
    let body = body_of(&w);
    let a = w.actors.get_mut(body).unwrap();
    a.hp = a.max_hp * 55 / 100;
    idle(&mut w, 60);
    assert_eq!(w.bosses[0].phase, 2);
    w
}

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

// ---------------------------------------------------------------------------------------
// Content
// ---------------------------------------------------------------------------------------

#[test]
fn the_b07_arena_is_a_boss_arena_with_voss_in_it() {
    let content = Content::load_embedded().unwrap();
    let w = World::new(SEED, &content, SCENARIO).unwrap();
    let lay = &w.layout.bosses[0];
    assert_eq!(lay.def.id, BOSS_ID);
    assert_eq!(lay.def.body.hp, 6000);
    assert_eq!(lay.def.phases.len(), 1);
    assert!((lay.def.phases[0].below - 0.6).abs() < 1e-6);
    assert!(
        lay.def
            .body
            .immune
            .contains(&lf_content::moves::StatusKind::Drowse)
    );
    assert_eq!(lay.def.rewards.items[0].item, "itm_great_sigil_castellan");
    assert!(lay.def.echo_stone);
    let spawn = w.layout.enemies[usize::from(lay.members[0])].pos;
    assert_eq!(spawn, [0.0, 0.0, 22.0]);
}

#[test]
fn a_delay_is_only_ever_on_a_later_hit_and_every_counter_telegraphs() {
    // The combo's third cut is delayed past the second's pace; the first two aren't.
    let (a, b, c) = (
        first_hit("b07_gs_1"),
        first_hit("b07_gs_2"),
        first_hit("b07_gs_3"),
    );
    assert!(a >= 12 && b >= 12);
    assert!(
        c >= b + 20,
        "the third cut ({c}) isn't delayed past the second ({b})"
    );
    // The parry counter: a quick first beat, a delayed second one.
    let (d1, d2) = (first_hit("b07_c_delay_1"), first_hit("b07_c_delay_2"));
    assert!(d1 >= 12 && d2 >= d1 + 20, "{d1} {d2}");
    for id in COUNTER_MOVES {
        assert!(first_hit(id) >= 12, "{id}");
    }
    // The kick is the one Unblockable hit, and flashes before it lands.
    use lf_content::moves::HitTag;
    let kick = content_move("b07_c_kick");
    assert!(kick.hitboxes[0].tags.contains(&HitTag::Unblockable));
    assert!(
        kick.vfx
            .iter()
            .any(|(f, n)| n == "flash_unblockable" && *f < kick.hitboxes[0].frames.0)
    );
    // The transition is 45 ticks and invulnerable throughout.
    let t = content_move("b07_overfit");
    assert_eq!(t.total_frames, 45);
    assert!(t.iframes.is_some_and(|w| w.from == 0 && w.to == 44));
}

#[test]
fn the_wide_sweeps_reach_far_toward_their_side() {
    use lf_content::moves::Shape;
    // `+x` is the boss's right, which is the player's left as they face each other.
    for (id, sign) in [("b07_c_sweep_pl", 1.0_f32), ("b07_c_sweep_pr", -1.0)] {
        let m = content_move(id);
        let reach = m
            .hitboxes
            .iter()
            .map(|h| match h.shape {
                Shape::Capsule { b, .. } => b[0] * sign,
                Shape::Sphere { center, .. } => center[0] * sign,
            })
            .fold(f32::MIN, f32::max);
        assert!(reach >= 3.5, "{id} reaches {reach} m toward its side");
    }
}

// ---------------------------------------------------------------------------------------
// The counting
// ---------------------------------------------------------------------------------------

/// A roll: the stick and `DODGE` for a tick, the stick alone for the tick the button is let go
/// (a tap rolls when it is released, toward the stick), and a tick for the hook to see it. Returns
/// the counts.
fn press(w: &mut World, stick: (i8, i8), b: u16) -> [u16; CATEGORIES] {
    let frame = |buttons| TickInput {
        frame: ActionFrame {
            move_x: stick.0,
            move_z: stick.1,
            buttons,
        },
        commands: Vec::new(),
    };
    tick(w, &frame(b));
    tick(w, &frame(0));
    tick(w, &TickInput::default());
    tick(w, &TickInput::default());
    w.events.drain();
    w.b07.counts
}

#[test]
fn rolls_are_counted_by_where_they_go_as_the_player_sees_it() {
    // The player faces +Z (toward Voss, who is north). The player's left is -X.
    let cases: [((i8, i8), [u16; CATEGORIES]); 4] = [
        ((-127, 0), [1, 0, 0, 0, 0, 0]),
        ((127, 0), [0, 1, 0, 0, 0, 0]),
        ((0, -127), [0, 0, 1, 0, 0, 0]),
        ((0, 127), [0, 0, 0, 0, 0, 0]), // toward him: not counted
    ];
    for (stick, want) in cases {
        let mut w = fight_world();
        // Voss stays out of it: park his options.
        let counts = press(&mut w, stick, buttons::DODGE);
        assert_eq!(counts, want, "stick {stick:?}");
    }
}

#[test]
fn a_backstep_counts_as_a_roll_back_and_a_second_roll_counts_again() {
    let mut w = fight_world();
    assert_eq!(press(&mut w, (0, 0), buttons::DODGE)[2], 1);
    idle(&mut w, 60);
    assert_eq!(press(&mut w, (0, -127), buttons::DODGE)[2], 2);
}

#[test]
fn a_parry_a_block_and_a_shot_are_each_counted_once() {
    let mut w = fight_world();
    let body = body_of(&w);
    // A parry: Voss enters the parried state (the first tick it shows).
    w.actors.get_mut(body).unwrap().poise.parried = 20;
    b07::run(&mut w);
    b07::run(&mut w);
    assert_eq!(w.b07.counts[3], 1);
    // A block: the guard counter window opens.
    w.actors.get_mut(w.player).unwrap().moves.guard.counter = 12;
    b07::run(&mut w);
    b07::run(&mut w);
    assert_eq!(w.b07.counts[4], 1);
    // A shot: the player starts a move that fires a projectile.
    let shot = move_num(&w, "stub_bow_shot");
    let player = w.player;
    start_move(&mut w, player, shot, 0);
    b07::run(&mut w);
    assert_eq!(w.b07.counts[5], 1);
    // The same move going on isn't a new shot.
    w.actors
        .get_mut(w.player)
        .unwrap()
        .moves
        .current
        .as_mut()
        .unwrap()
        .frame = 3;
    b07::run(&mut w);
    assert_eq!(w.b07.counts, [0, 0, 0, 1, 1, 1]);
}

#[test]
fn counting_is_announced_with_the_category_and_the_count() {
    let mut w = fight_world();
    w.actors.get_mut(w.player).unwrap().moves.guard.counter = 12;
    b07::run(&mut w);
    let events: Vec<u16> = w
        .events
        .drain()
        .into_iter()
        .filter_map(|e| match e {
            Event::Mechanic(m) if m.kind == MechanicEvent::VOSS_COUNTED => Some(m.index),
            _ => None,
        })
        .collect();
    assert_eq!(events, vec![(4 << 8) | 1]);
}

#[test]
fn nothing_is_counted_outside_the_fight_and_the_state_rests_there() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, SCENARIO).unwrap();
    let rest = w.state_hash();
    w.actors.get_mut(w.player).unwrap().moves.guard.counter = 12;
    b07::run(&mut w);
    assert!(w.b07.is_inert());
    w.actors.get_mut(w.player).unwrap().moves.guard.counter = 0;
    assert_eq!(w.state_hash(), rest);
    // In the fight the state moves the hash; a death resets it.
    let mut w = fight_world();
    w.actors.get_mut(w.player).unwrap().moves.guard.counter = 12;
    b07::run(&mut w);
    assert!(!w.b07.is_inert());
    w.actors.get_mut(w.player).unwrap().hp = 0;
    w.actors.get_mut(w.player).unwrap().dead = true;
    idle(&mut w, 3);
    assert_eq!(w.bosses[0].stage, Stage::Dormant);
    assert!(w.b07.is_inert());
}

// ---------------------------------------------------------------------------------------
// The counters
// ---------------------------------------------------------------------------------------

/// The counters Voss started in `ticks` ticks of standing in front of an idle player, as a set
/// of category numbers.
fn counters_used(w: &mut World, ticks: u32) -> Vec<usize> {
    let nums: Vec<MoveNum> = COUNTER_MOVES.iter().map(|m| move_num(w, m)).collect();
    let delay_2 = move_num(w, "b07_c_delay_2");
    let body = body_of(w);
    let mut used = Vec::new();
    for _ in 0..ticks {
        idle(w, 1);
        // Keep the player standing and Voss's HP above the phase and his death.
        let a = w.actors.get_mut(body).unwrap();
        a.hp = a.hp.max(a.max_hp / 2);
        if let Some(c) = w.actors.get(body).unwrap().moves.current
            && c.frame == 1
        {
            let hit = nums
                .iter()
                .position(|&n| n == c.num)
                .or_else(|| (c.num == delay_2).then_some(3));
            if let Some(i) = hit
                && !used.contains(&i)
            {
                used.push(i);
            }
        }
    }
    used.sort();
    used
}

#[test]
fn phase_two_enables_the_counters_for_the_two_most_counted_categories() {
    // Rolled left 5 times, parried 3, rolled back once (below the minimum), guarded 4 times.
    let mut w = phase_two_world([5, 0, 1, 3, 4, 0]);
    assert_eq!(w.b07.favored, [Some(0), Some(4)]);
    let used = counters_used(&mut w, 2400);
    assert!(
        used.iter().all(|&c| c == 0 || c == 4),
        "a counter that isn't earned was used: {used:?}"
    );
    assert!(
        used.contains(&0),
        "the sweep for rolling left was never used: {used:?}"
    );
    assert!(
        used.contains(&4),
        "the kick for guarding was never used: {used:?}"
    );
}

#[test]
fn the_counters_follow_the_counts_for_every_pair() {
    // The sweep toward the right side and the dash against shots.
    let mut w = phase_two_world([0, 6, 0, 0, 0, 5]);
    assert_eq!(w.b07.favored, [Some(1), Some(5)]);
    let used = counters_used(&mut w, 2400);
    assert!(used.iter().all(|&c| c == 1 || c == 5), "{used:?}");
    assert!(used.contains(&1) || used.contains(&5), "{used:?}");
    // Rolling back and parrying: the lunge, and the delayed swing (its second beat counts too).
    let mut w = phase_two_world([0, 0, 4, 9, 0, 0]);
    assert_eq!(w.b07.favored, [Some(3), Some(2)]);
    let used = counters_used(&mut w, 2400);
    assert!(used.iter().all(|&c| c == 2 || c == 3), "{used:?}");
    assert!(!used.is_empty());
}

#[test]
fn ties_go_to_the_lower_category_and_a_category_needs_its_minimum() {
    let w = phase_two_world([3, 3, 3, 3, 3, 3]);
    assert_eq!(w.b07.favored, [Some(0), Some(1)]);
    // Nothing counted: no counter at all, only phase 1's moves and the chandeliers.
    let mut w = phase_two_world([0, 0, 0, 0, 0, 0]);
    assert_eq!(w.b07.favored, [None, None]);
    assert!(counters_used(&mut w, 1500).is_empty());
    // One action in a category isn't enough.
    let w = phase_two_world([0, 1, 0, 0, 0, 0]);
    assert_eq!(w.b07.favored, [None, None]);
}

#[test]
fn the_overfit_event_names_the_favored_pair() {
    let mut w = fight_world();
    w.b07.counts = [0, 0, 0, 7, 0, 2];
    let body = body_of(&w);
    let a = w.actors.get_mut(body).unwrap();
    a.hp = a.max_hp * 55 / 100;
    let events = idle(&mut w, 60);
    let overfit: Vec<u16> = events
        .iter()
        .filter_map(|e| match e {
            Event::Mechanic(m) if m.kind == MechanicEvent::VOSS_OVERFIT => Some(m.index),
            _ => None,
        })
        .collect();
    assert_eq!(overfit, vec![(3 << 8) | 5]);
    // The framework's phase event came with it, and counting stopped.
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Boss(b) if b.kind == BossEvent::PHASE_STARTED))
            .count(),
        1
    );
    let before = w.b07.counts;
    w.actors.get_mut(w.player).unwrap().moves.guard.counter = 12;
    idle(&mut w, 2);
    assert_eq!(w.b07.counts, before, "counting went on in phase 2");
}

// ---------------------------------------------------------------------------------------
// The guard
// ---------------------------------------------------------------------------------------

#[test]
fn a_hit_on_the_guard_does_no_damage_and_brings_the_counter() {
    let mut w = fight_world();
    let body = body_of(&w);
    let guard = move_num(&w, "b07_guard");
    let counter = move_num(&w, "b07_guard_counter");
    idle(&mut w, 1); // Voss's HP is noted
    start_move(&mut w, body, guard, 30);
    let hp = actor(&w, body).hp;
    w.actors.get_mut(body).unwrap().hp = hp - 400;
    let events = idle(&mut w, 1);
    assert_eq!(actor(&w, body).hp, hp, "the blow cost HP");
    assert_eq!(actor(&w, body).moves.current.map(|c| c.num), Some(counter));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Mechanic(m) if m.kind == MechanicEvent::VOSS_BLOCKED))
    );
}

#[test]
fn a_hit_outside_the_guard_window_lands() {
    for frame in [4, 70] {
        let mut w = fight_world();
        let body = body_of(&w);
        let guard = move_num(&w, "b07_guard");
        idle(&mut w, 1);
        start_move(&mut w, body, guard, frame);
        let hp = actor(&w, body).hp;
        w.actors.get_mut(body).unwrap().hp = hp - 400;
        idle(&mut w, 1);
        assert_eq!(actor(&w, body).hp, hp - 400, "frame {frame}");
    }
}

// ---------------------------------------------------------------------------------------
// The chandeliers
// ---------------------------------------------------------------------------------------

fn chandelier_events(events: &[Event], kind: u8) -> Vec<u16> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Mechanic(m) if m.kind == kind => Some(m.index),
            _ => None,
        })
        .collect()
}

#[test]
fn a_marker_shows_under_the_player_and_the_chandelier_falls_after_the_telegraph() {
    let mut w = phase_two_world([0; CATEGORIES]);
    let first = w.layout.bosses[0].def.params["chandelier_first_ticks"] as u32;
    let telegraph = w.layout.bosses[0].def.params["chandelier_telegraph_ticks"] as u32;
    let body = body_of(&w);
    // Keep the player alive and still, and Voss out of the way.
    let mut marked_at = None;
    let mut fell_at = None;
    let mut hit = false;
    for t in 0..(first + telegraph + 40) {
        let events = idle(&mut w, 1);
        let a = w.actors.get_mut(body).unwrap();
        a.hp = a.max_hp * 55 / 100;
        a.moves.interrupt();
        a.pos = [0.0, 0.0, 22.0];
        put_player(&mut w, [6.0, 0.0, 12.0]);
        if !chandelier_events(&events, MechanicEvent::VOSS_CHANDELIER_MARKED).is_empty() {
            marked_at = Some(t);
            // The marker is on the player, snapped to half-meters.
            let m = w.b07.markers[0];
            assert!(
                (m.x - 6.0).abs() <= 0.26 && (m.z - 12.0).abs() <= 0.26,
                "{m:?}"
            );
        }
        if !chandelier_events(&events, MechanicEvent::VOSS_CHANDELIER_FELL).is_empty() {
            fell_at = Some(t);
        }
        hit |= !chandelier_events(&events, MechanicEvent::VOSS_CHANDELIER_HIT).is_empty();
    }
    let (m, f) = (marked_at.expect("no marker"), fell_at.expect("no fall"));
    assert!(f - m >= telegraph, "the telegraph was {} ticks", f - m);
    assert!(hit, "the chandelier missed a player standing on its marker");
    let p = actor(&w, w.player);
    assert!(p.hp < p.max_hp, "no damage");
}

#[test]
fn the_chandelier_misses_a_player_who_steps_out_or_rolls() {
    for mode in ["away", "i-frames"] {
        let mut w = phase_two_world([0; CATEGORIES]);
        let body = body_of(&w);
        let mut hit = false;
        let mut fell = false;
        for _ in 0..500 {
            let events = idle(&mut w, 1);
            let a = w.actors.get_mut(body).unwrap();
            a.hp = a.max_hp * 55 / 100;
            a.moves.interrupt();
            a.pos = [0.0, 0.0, 22.0];
            if let Some(m) = w.b07.markers.iter().find(|m| m.left > 0).copied() {
                match mode {
                    "away" if m.left == 1 => put_player(&mut w, [m.x + 6.0, 0.0, m.z]),
                    "i-frames" if m.left == 1 => {
                        w.actors.get_mut(w.player).unwrap().moves.flags |= move_flags::IFRAMES;
                    }
                    _ => {}
                }
            } else {
                put_player(&mut w, [6.0, 0.0, 12.0]);
            }
            fell |= !chandelier_events(&events, MechanicEvent::VOSS_CHANDELIER_FELL).is_empty();
            hit |= !chandelier_events(&events, MechanicEvent::VOSS_CHANDELIER_HIT).is_empty();
            if fell {
                break;
            }
        }
        assert!(fell, "{mode}: nothing fell");
        assert!(!hit, "{mode}: the chandelier hit");
    }
}

#[test]
fn chandeliers_fall_only_in_phase_two() {
    let mut w = fight_world();
    let events = idle(&mut w, 600);
    assert!(chandelier_events(&events, MechanicEvent::VOSS_CHANDELIER_MARKED).is_empty());
}

#[test]
fn a_marker_event_carries_its_slot_and_its_place_from_the_boss_spawn() {
    let mut w = phase_two_world([0; CATEGORIES]);
    let body = body_of(&w);
    let mut index = None;
    for _ in 0..400 {
        let events = idle(&mut w, 1);
        let a = w.actors.get_mut(body).unwrap();
        a.hp = a.max_hp * 55 / 100;
        a.moves.interrupt();
        a.pos = [0.0, 0.0, 22.0];
        put_player(&mut w, [-5.0, 0.0, 10.0]);
        if let Some(i) = chandelier_events(&events, MechanicEvent::VOSS_CHANDELIER_MARKED).first() {
            index = Some(*i);
            break;
        }
    }
    let i = index.expect("no marker");
    let (slot, x, z) = (i >> 14, (i >> 7) & 127, i & 127);
    // Offsets from the spawn (0, 22): x -5 m, z -12 m, in half-meters plus 64.
    assert_eq!((slot, x, z), (0, 54, 40));
}

// ---------------------------------------------------------------------------------------
// The back gate
// ---------------------------------------------------------------------------------------

#[test]
fn defeating_voss_raises_the_flag_that_opens_the_back_gate() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, "castle").unwrap();
    let flag = flags::door_open("gate_castle_back");
    assert!(!w.state.flags.is_set(&flag));
    // A fight with Voss in the hall does not open it.
    let b = w
        .layout
        .bosses
        .iter()
        .position(|l| l.def.id == BOSS_ID)
        .expect("Voss is placed in the castle");
    idle(&mut w, 5);
    assert!(!w.state.flags.is_set(&flag));
    // Defeated (here, by the stage the framework sets), the flag goes up on its own.
    w.bosses[b].stage = Stage::Defeated;
    idle(&mut w, 2);
    assert!(w.state.flags.is_set(&flag));
    // B03's defeat alone never would have: the other boss's stage is still Dormant.
    let other = 1 - b;
    assert_ne!(w.bosses[other].stage, Stage::Defeated);
}

#[test]
fn a_voss_beaten_in_the_save_opens_the_gate_too() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(SEED, &content, "castle").unwrap();
    w.state.flags.raise(&flags::boss_defeated(BOSS_ID));
    idle(&mut w, 3);
    assert!(w.state.flags.is_set(&flags::door_open("gate_castle_back")));
}

// ---------------------------------------------------------------------------------------
// The fight
// ---------------------------------------------------------------------------------------

#[test]
fn the_scripted_bot_beats_b07_through_both_phases() {
    let f = run_bot(SEED);
    assert!(
        f.boss_defeated(),
        "the boss stands at tick {}, phase {}, player hp {}",
        f.world.tick,
        f.world.bosses[0].phase,
        f.player_hp()
    );
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    assert_eq!(f.boss_events(BossEvent::PHASE_STARTED), 1);
    // The bot rolled left all through phase 1, so rolling left is what he overfit to, and the
    // chandeliers came down in phase 2.
    let overfit = f.mechanics(MechanicEvent::VOSS_OVERFIT);
    assert_eq!(overfit.len(), 1);
    assert_eq!(overfit[0] >> 8, 0, "the most-counted category is roll left");
    assert_eq!(
        f.world.b07.counts, [0; CATEGORIES],
        "the state rests after the fight"
    );
    assert!(!f.mechanics(MechanicEvent::VOSS_COUNTED).is_empty());
    assert!(
        !f.mechanics(MechanicEvent::VOSS_CHANDELIER_MARKED)
            .is_empty()
    );
    // The Sigil and the Glimmer paid out.
    let inv = &f.world.actors.get(f.world.player).unwrap().inventory;
    assert_eq!(inv.glimmer, 3500);
    let content = Content::load_embedded().unwrap();
    let sigil = content.registry.num("itm_great_sigil_castellan").unwrap();
    assert!(inv.count(sigil) >= 1, "no Sigil");
    assert!(
        f.world
            .state
            .flags
            .is_set(&flags::codex("b07_castellan_voss"))
    );
}

#[test]
fn a_player_who_does_nothing_dies_to_b07() {
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
fn the_bot_golden_replay_beats_b07_with_the_player_alive() {
    let f = play_golden("m7_b07_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b07_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b07_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b07_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b07_goldens` in game/crates/core/src/ai/bosses/b07_tests.rs.\n\
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
        "m7_b07_bot_win",
        "WP-075 golden: the scripted bot beats B07 by rolling through each hit, stepping out of the chandeliers, and punishing.",
        &run_bot(SEED),
    );
    write(
        "m7_b07_idle_dies",
        "WP-075 golden: a player who walks in and does nothing dies to B07.",
        &run_idle(SEED),
    );
}
