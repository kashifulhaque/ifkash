//! Tests for B06, the Drowned Choir (WP-074), and the scripted bot that beats it.
//!
//! The bot is the proof that the fight is fair (docs/game/05-enemies-and-bosses.md, "B06 The
//! Drowned Choir"): it reads every member's move each tick, rolls so that the roll's i-frames
//! cover each hit about to land (a knife fan, a beam, a bell slam, the stun wave, the charge, a
//! stab), and punishes a member in the recovery of its last move. It is deterministic (no
//! randomness, no clock). [`run_bot`] and [`run_idle`] play a whole fight from any seed.
//!
//! The goldens `m7_b06_bot_win.lfr` and `m7_b06_idle_dies.lfr` come from these inputs:
//! `record_the_b06_goldens` (ignored) writes the `.script` files, and
//! `lf-tools replay record-script` turns them into the `.lfr` files.

use lf_content::Content;
use lf_content::moves::Shape;

use crate::arena::Id;
use crate::combat::moves::MoveNum;
use crate::events::{Event, LifeEvent, MechanicEvent};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world::World;
use crate::world_state::flags;

use crate::ai::boss::Stage;

const SCENARIO: &str = "boss_b06";
const BOSS_ID: &str = "b06_drowned_choir";
/// The seed the goldens use.
const SEED: u64 = 74;
/// The longest fight the runners play, in ticks (the three members have 5,400 HP between
/// them; this only stops a stuck run).
const MAX_TICKS: u64 = 30000;

/// The members' actors, the Cantor first.
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

/// How far (planar, in meters) a move's hit volumes reach from its user, counting its root
/// motion; used to tell whether a coming hit can reach the player.
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
            // A sphere centered under the floor reaches less far along it (the stun wave).
            let dy = (p[1] - 0.9).abs();
            let flat = if radius > dy {
                sqrt(radius * radius - dy * dy)
            } else {
                0.0
            };
            let extent = sqrt(p[0] * p[0] + p[2] * p[2]) + if p[1] < 0.0 { flat } else { radius };
            r = r.max(extent);
        }
    }
    let travel = def.data.root_motion.last().map_or(0.0, |k| k.1);
    r + travel
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

    /// The member (and the ticks until it lands) of the nearest hit coming at the player.
    fn incoming(&self, w: &World, me: [f32; 3]) -> Option<(Id, i32)> {
        let mut best: Option<(Id, i32)> = None;
        for id in members(w) {
            let Some(m) = w.actors.get(id) else {
                continue;
            };
            if m.dead || m.hp == 0 {
                continue;
            }
            let dist = planar(m.pos, me);
            // A knife fan or an orb: time the roll by how far the nearest shot is.
            if let Some(c) = m.moves.current
                && let Some(def) = w.moves.get(c.num)
            {
                let first = def.data.hitboxes.iter().map(|h| h.frames.0).min();
                if let Some(first) = first {
                    let frame = c.frame;
                    if frame < first && dist <= reach(w, c.num) + 0.6 {
                        let eta = i32::from(first - frame);
                        if best.is_none_or(|b| eta < b.1) {
                            best = Some((id, eta));
                        }
                    } else if def.data.root_motion.last().is_some_and(|k| k.1 > 4.0)
                        && frame >= first.saturating_sub(2)
                        && frame <= first + 18
                    {
                        // The charge: it arrives when it has run the gap.
                        let eta = ((dist - 1.8).max(0.0) / 0.44) as i32;
                        if best.is_none_or(|b| eta < b.1) {
                            best = Some((id, eta));
                        }
                    }
                }
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

    /// Who the bot goes after: the Cantor first (she mends the others, so the others can't
    /// be worn down while she lives), then the Lamenter, then the Bellbearer.
    fn focus(w: &World) -> Option<Id> {
        let ids = members(w);
        [0usize, 2, 1]
            .into_iter()
            .map(|i| ids[i])
            .find(|&id| w.actors.get(id).is_some_and(|a| !a.dead && a.hp > 0))
    }

    fn decide(&mut self, w: &World) -> ((i8, i8), u16) {
        let Some(player) = w.actors.get(w.player) else {
            return ((0, 0), 0);
        };
        let ids = members(w);
        let cantor = w.actors.get(ids[0]).map_or([0.0; 3], |a| a.pos);
        match w.bosses[0].stage {
            Stage::Dormant => return (toward(player.pos, cantor), 0), // walk through the gate
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
        let Some(focus) = Self::focus(w) else {
            return ((0, 0), 0);
        };
        let f = w.actors.get(focus).unwrap();
        let target = f.pos;

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
        // 2. Punish: a member in the recovery of its last move, close enough, and nothing
        // else about to land. The focus is preferred; any other member in reach will do.
        let threatened = self
            .incoming(w, player.pos)
            .is_some_and(|(_, eta)| eta <= 26);
        let recovery = |id: Id| -> Option<(u16, [f32; 3])> {
            let m = w.actors.get(id)?;
            if m.dead || m.hp == 0 {
                return None;
            }
            let c = m.moves.current?;
            let def = w.moves.get(c.num)?;
            let last = def.data.hitboxes.iter().map(|h| h.frames.1).max();
            let spawns = def.data.projectiles.iter().map(|p| p.frame).max();
            let last = last.max(spawns)?;
            let left = def.data.total_frames.saturating_sub(c.frame);
            (def.next_chain.is_none() && c.frame > last && left >= 14).then_some((left, m.pos))
        };
        let mut order = vec![focus];
        order.extend(ids.iter().copied().filter(|&i| i != focus));
        if !threatened {
            for id in order {
                let Some((left, pos)) = recovery(id) else {
                    continue;
                };
                let dist = planar(player.pos, pos);
                let low = player.hp * 100 < player.max_hp * 45;
                if low && player.inventory.flasks.vigil > 0 && left >= 30 {
                    return ((0, 0), buttons::USE_ITEM);
                }
                if dist <= 2.2 && player.stamina >= 26.0 {
                    return (toward(player.pos, pos), buttons::LIGHT);
                }
            }
        }
        // 3. Close the distance (keeping the others' attacks in view).
        let dist = planar(player.pos, target);
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

/// Plays the fight with the scripted bot until the Choir falls or the player dies.
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

fn fell(w: &mut World, member: usize) {
    let id = members(w)[member];
    let a = w.actors.get_mut(id).unwrap();
    a.hp = 0;
    a.dead = true;
    a.moves.current = None;
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
fn the_scripted_bot_beats_b06_by_rolling_and_punishing() {
    let f = run_bot(SEED);
    assert!(
        f.boss_defeated(),
        "the Choir stands at tick {}",
        f.world.tick
    );
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    // All three members are down, and the verse was sung when the first fell.
    for id in members(&f.world) {
        assert!(f.world.actors.get(id).unwrap().hp == 0);
    }
    assert_eq!(f.world.bosses[0].phase, 2);
    assert!(mechanic_kinds(&f.events).contains(&(MechanicEvent::CHOIR_VERSE, 0)));
}

#[test]
fn a_player_who_does_nothing_dies_to_b06() {
    let f = run_idle(SEED);
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

#[test]
fn the_bot_golden_replay_beats_b06_with_the_player_alive() {
    let f = play_golden("m7_b06_bot_win");
    assert!(f.boss_defeated());
    assert!(!f.player_died());
    assert!(f.player_hp() > 0);
    // The Glimmer and both rewards paid out.
    let p = f.world.actors.get(f.world.player).unwrap();
    assert_eq!(p.inventory.glimmer, 1500);
    assert!(f.world.state.flags.get("codex.b06_drowned_choir") > 0);
}

#[test]
fn the_idle_golden_replay_kills_the_player() {
    let f = play_golden("m7_b06_idle_dies");
    assert!(f.player_died());
    assert!(!f.boss_defeated());
}

#[test]
fn the_fight_ends_only_when_all_three_members_are_down() {
    let mut w = fight_world(SEED);
    fell(&mut w, 1);
    idle_tick(&mut w);
    fell(&mut w, 2);
    idle_tick(&mut w);
    assert_eq!(w.bosses[0].stage, Stage::Fight);
    fell(&mut w, 0);
    idle_tick(&mut w);
    assert_eq!(w.bosses[0].stage, Stage::Defeated);
    assert!(w.state.flags.get(&flags::boss_defeated(BOSS_ID)) > 0);
}

#[test]
fn the_first_fall_sings_the_verse_and_the_survivors_gain_a_move() {
    let mut w = fight_world(SEED);
    assert_eq!(w.bosses[0].phase, 1);
    fell(&mut w, 1); // the Bellbearer
    let events = idle_tick(&mut w);
    assert_eq!(w.bosses[0].phase, 2);
    assert!(mechanic_kinds(&events).contains(&(MechanicEvent::CHOIR_VERSE, 1)));
    // The survivors' brains read the new phase (their `Phase(2)` options are the verse moves).
    for i in [0, 2] {
        let a = w.actors.get(members(&w)[i]).unwrap();
        assert_eq!(a.ai.as_ref().unwrap().phase, 2, "member {i}");
    }
    // A second fall doesn't sing again.
    fell(&mut w, 2);
    let events = idle_tick(&mut w);
    assert!(
        !mechanic_kinds(&events)
            .iter()
            .any(|k| k.0 == MechanicEvent::CHOIR_VERSE)
    );
}

#[test]
fn each_verse_move_is_only_in_the_options_of_phase_two() {
    let content = Content::load_embedded().unwrap();
    let def = content.boss_by_id(BOSS_ID).unwrap();
    for (actor, mv) in def
        .actors()
        .zip(["b06_cantor_orbs", "b06_bell_charge", "b06_lam_veilstab"])
    {
        let o = actor
            .options
            .iter()
            .find(|o| o.move_id.as_deref() == Some(mv))
            .unwrap_or_else(|| panic!("{} has no {mv} option", actor.id));
        assert!(
            o.when.contains(&lf_content::enemy::Condition::Phase(2)),
            "{mv} isn't a verse move"
        );
    }
}

#[test]
fn each_member_is_immune_to_a_different_status() {
    let content = Content::load_embedded().unwrap();
    let def = content.boss_by_id(BOSS_ID).unwrap();
    let immune: Vec<_> = def.actors().map(|a| a.immune.clone()).collect();
    assert_eq!(immune.len(), 3);
    for (i, a) in immune.iter().enumerate() {
        assert!(!a.is_empty(), "member {i} resists nothing");
        for b in &immune[i + 1..] {
            assert!(
                a.iter().all(|s| !b.contains(s)),
                "two members share an immunity"
            );
        }
    }
}

#[test]
fn a_finished_hymn_heals_the_others_and_a_broken_one_does_not() {
    let mut w = fight_world(SEED);
    let max = {
        let ids = members(&w);
        let a = w.actors.get_mut(ids[2]).unwrap();
        a.hp = 900;
        a.max_hp
    };
    let frame = w.layout.bosses[0].def.params["mend_frame"] as u16;
    let pct = w.layout.bosses[0].def.params["mend_heal_percent"] as u32;
    // A hymn broken before its frame (the Cantor is staggered out of it): no heal.
    start_move(&mut w, 0, "b06_cantor_mend", frame - 5);
    w.actors.get_mut(members(&w)[0]).unwrap().moves.current = None;
    let events = idle_tick(&mut w);
    assert_eq!(w.actors.get(members(&w)[2]).unwrap().hp, 900);
    assert!(
        !mechanic_kinds(&events)
            .iter()
            .any(|k| k.0 == MechanicEvent::CHOIR_MEND)
    );
    // A hymn that reaches its frame heals the Bellbearer (full already) and the Lamenter.
    start_move(&mut w, 0, "b06_cantor_mend", frame);
    let events = idle_tick(&mut w);
    assert_eq!(
        w.actors.get(members(&w)[2]).unwrap().hp,
        900 + max * pct / 100
    );
    assert_eq!(
        mechanic_kinds(&events)
            .iter()
            .filter(|k| k.0 == MechanicEvent::CHOIR_MEND)
            .collect::<Vec<_>>(),
        vec![&(MechanicEvent::CHOIR_MEND, 2)]
    );
}

#[test]
fn the_mend_never_goes_past_max_hp() {
    let mut w = fight_world(SEED);
    let ids = members(&w);
    let max = w.actors.get(ids[1]).unwrap().max_hp;
    w.actors.get_mut(ids[1]).unwrap().hp = max - 10;
    let frame = w.layout.bosses[0].def.params["mend_frame"] as u16;
    start_move(&mut w, 0, "b06_cantor_mend", frame);
    idle_tick(&mut w);
    assert_eq!(w.actors.get(ids[1]).unwrap().hp, max);
}

#[test]
fn the_teleport_stab_puts_the_lamenter_just_past_the_player_facing_them() {
    let mut w = fight_world(SEED);
    let ids = members(&w);
    let frame = w.layout.bosses[0].def.params["teleport_frame"] as u16;
    let behind = w.layout.bosses[0].def.params["teleport_behind_m"];
    // The player stands in the open middle of the nave; the Lamenter is toward the apse.
    let (px, pz) = (0.0, 20.0);
    {
        let p = w.actors.get_mut(w.player).unwrap();
        p.pos = [px, 0.02, pz];
    }
    {
        let l = w.actors.get_mut(ids[2]).unwrap();
        l.pos = [0.0, 0.02, 12.0];
    }
    start_move(&mut w, 2, "b06_lam_veilstab", frame);
    let events = idle_tick(&mut w);
    let l = w.actors.get(ids[2]).unwrap();
    let p = w.actors.get(w.player).unwrap();
    // She is `behind` meters past the player, on the far side from where she was (z larger).
    assert!(
        l.pos[2] > p.pos[2],
        "not past the player: {:?} {:?}",
        l.pos,
        p.pos
    );
    let d = planar(l.pos, p.pos);
    assert!((d - behind).abs() < 0.8, "distance {d}");
    assert!(mechanic_kinds(&events).contains(&(MechanicEvent::CHOIR_TELEPORT, 2)));
}

#[test]
fn the_teleport_stab_stops_short_of_a_wall_behind_the_player() {
    let mut w = fight_world(SEED);
    let ids = members(&w);
    let frame = w.layout.bosses[0].def.params["teleport_frame"] as u16;
    let min = w.layout.bosses[0].def.params["teleport_min_m"];
    // The player stands 0.9 m from the apse wall (z = 0 is its inner face), the Lamenter in
    // front of them: there is no room behind.
    {
        let p = w.actors.get_mut(w.player).unwrap();
        p.pos = [0.0, 0.02, 0.9];
    }
    {
        let l = w.actors.get_mut(ids[2]).unwrap();
        l.pos = [0.0, 0.02, 8.0];
    }
    start_move(&mut w, 2, "b06_lam_veilstab", frame);
    idle_tick(&mut w);
    let l = w.actors.get(ids[2]).unwrap();
    let p = w.actors.get(w.player).unwrap();
    assert!(l.pos[2] > 0.0, "inside the wall: {:?}", l.pos);
    assert!(planar(l.pos, p.pos) >= min - 0.3);
}

#[test]
fn a_death_resets_the_verse() {
    let mut w = fight_world(SEED);
    fell(&mut w, 0);
    idle_tick(&mut w);
    assert_eq!(w.bosses[0].phase, 2);
    {
        let p = w.actors.get_mut(w.player).unwrap();
        p.hp = 0;
        p.dead = true;
    }
    idle_tick(&mut w);
    assert_eq!(w.bosses[0].stage, Stage::Dormant);
    for id in members(&w) {
        let a = w.actors.get(id).unwrap();
        assert_eq!(a.hp, a.max_hp);
    }
}

/// Writes the two `.script` files the goldens are recorded from. Run it with
/// `cargo test -p lf-core record_the_b06_goldens -- --ignored`, then
/// `cargo run -p lf-tools -- replay record-script --out <lfr> <script>` for each.
#[test]
#[ignore]
fn record_the_b06_goldens() {
    let dir = format!("{}/../../tests/replays", env!("CARGO_MANIFEST_DIR"));
    let write = |name: &str, about: &str, f: &Fight| {
        let mut text = format!(
            "# {about}\n# Generated by `record_the_b06_goldens` in game/crates/core/src/ai/bosses/b06_tests.rs.\n\
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
        "m7_b06_bot_win",
        "WP-074 golden: the scripted bot beats the Drowned Choir by rolling through each hit and punishing.",
        &run_bot(SEED),
    );
    write(
        "m7_b06_idle_dies",
        "WP-074 golden: a player who walks in and does nothing dies to the Drowned Choir.",
        &run_idle(SEED),
    );
}
