//! Echo tests (WP-059): the Bell, the stones, the ally AI, the bow, and the golden replay in
//! which an Echo helps beat the test boss.
//!
//! The golden `echo_helps_beat_test_boss.lfr` comes from the closed-loop bot in this file:
//! `record_the_echo_golden` (ignored) plays it and writes the replay. The replay's script
//! format has no `cmd` lines for the Bell's commands, so the recorder writes the `.lfr`
//! directly with `Replay::record`.

use lf_content::Content;

use super::*;

use crate::events::{BossEvent, HitResult};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::items::slot;
use crate::replay::{Replay, expand};
use crate::tick::tick;
use crate::world_state::flags;

const SCENARIO: &str = "echo_boss_gate";
const SEED: u64 = 59;
const GOLDEN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/replays/m5_echo_helps_beat_test_boss.lfr"
);
/// Registry numbers of the Bell and the four Echoes (see `data/registry.ron`).
const BELL: u32 = 17000;
const POLITELLAMA: u32 = 17010;
const BONE_SENTINEL: u32 = 17011;
const CHOIR_WISP: u32 = 17012;
const BRIAR_HOUNDS: u32 = 17013;
/// The quick slot the Bell goes in (slots 0 and 1 hold the flasks), as an equipment slot and
/// as the index `Command::UseItem` takes.
const BELL_SLOT: u8 = slot::QUICK_1 + 2;
const BELL_QUICK: u8 = 2;
/// Where the player stands to ring the Bell: 1.8 m from the stone at (-4, 0, 3).
const RING_AT: [f32; 3] = [-2.5, 0.0, 2.0];
const LONGEST_FIGHT: u64 = 3000;

fn arena() -> World {
    let content = Content::load_embedded().unwrap();
    World::new(SEED, &content, SCENARIO).unwrap()
}

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.motor.ground_y = pos[1];
    p.motor.fall_peak_y = pos[1];
}

/// Gives the player the Bell in a quick slot and plenty of FP.
fn arm(w: &mut World) {
    let bell = w.echo_table.bell.unwrap();
    let items = w.items.clone();
    let p = w.actors.get_mut(w.player).unwrap();
    assert_eq!(p.inventory.pick_up(bell, 1, &items), 1);
    assert!(p.inventory.equip(BELL_SLOT, bell, &items));
    p.max_fp = 300.0;
    p.fp = 300.0;
}

fn idle() -> TickInput {
    TickInput::default()
}

fn cmd(c: Command) -> TickInput {
    TickInput {
        frame: ActionFrame::default(),
        commands: vec![c],
    }
}

fn run(w: &mut World, input: &TickInput, n: u32) -> Vec<Event> {
    let mut out = Vec::new();
    for _ in 0..n {
        tick(w, input);
        out.extend(w.events.drain());
    }
    out
}

fn echo_events(events: &[Event]) -> Vec<(u8, u32, u8)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Echo(x) => Some((x.kind, x.echo, x.reason)),
            _ => None,
        })
        .collect()
}

fn members(w: &World) -> Vec<Id> {
    w.echo
        .active
        .as_ref()
        .map(|a| a.members.iter().map(|m| m.id).collect())
        .unwrap_or_default()
}

/// Rings the Bell at the stone with the Echo at `index` of the owned list selected.
fn summon(w: &mut World, index: u16) -> Vec<Event> {
    arm(w);
    put_player(w, RING_AT);
    w.echo.selected = index;
    run(w, &cmd(Command::UseItem { slot: BELL_QUICK }), 1)
}

/// Rings the Bell at an Echo Stone moved outside the fog gate, so the boss stays Dormant.
fn summon_outside(w: &mut World) {
    w.echo_table.stones[0].pos = [-4.0, 0.0, -3.0];
    arm(w);
    put_player(w, [-3.0, 0.0, -3.0]);
    run(w, &cmd(Command::UseItem { slot: BELL_QUICK }), 1);
    assert!(w.echo.active.is_some());
}

fn boss_body(w: &World) -> Id {
    w.state.ai_enemies[usize::from(w.layout.bosses[0].members[0])]
}

/// Starts the boss fight with the player at the stone, and returns the events.
fn start_fight(w: &mut World) -> Vec<Event> {
    // Through the gate and into the arena, then out of the intro.
    put_player(w, [0.0, 0.0, -0.2]);
    let mut events = run(
        w,
        &TickInput {
            frame: ActionFrame {
                move_x: 0,
                move_z: 127,
                // Interact passes the fog gate (issue #82).
                buttons: buttons::INTERACT,
            },
            commands: Vec::new(),
        },
        12,
    );
    events.extend(run(w, &idle(), 100));
    assert_eq!(w.bosses[0].stage, crate::ai::boss::Stage::Fight);
    events
}

// ---- The Bell and the stones ----------------------------------------------------------

#[test]
fn the_bell_and_the_echoes_are_in_the_content() {
    let w = arena();
    assert_eq!(w.echo_table.bell, Some(BELL));
    let nums: Vec<u32> = w.echo_table.echoes.iter().map(|e| e.num).collect();
    assert_eq!(nums, [POLITELLAMA, BONE_SENTINEL, CHOIR_WISP, BRIAR_HOUNDS]);
    assert_eq!(w.echo_table.stones.len(), 1);
    assert_eq!(owned(&w).len(), 4, "the arena unlocks all four");
}

#[test]
fn ringing_the_bell_at_a_stone_summons_the_echo_and_spends_fp() {
    let mut w = arena();
    let events = summon(&mut w, 0);
    assert_eq!(
        echo_events(&events),
        [(EchoEvent::SUMMONED, POLITELLAMA, 0)]
    );
    let ids = members(&w);
    assert_eq!(ids.len(), 1);
    let echo = w.actors.get(ids[0]).unwrap();
    assert!(echo.is_ally() && !echo.dead);
    assert_eq!(w.actors.get(w.player).unwrap().fp, 300.0 - 40.0);
    // It stands in front of the stone.
    assert!(dist_xz(echo.pos, [-4.0, 0.0, 3.0]) < 3.0, "{:?}", echo.pos);
    // The Bell does not start a flask drink or anything else on the player.
    assert!(w.actors.get(w.player).unwrap().moves.current.is_none());
}

#[test]
fn the_bell_refuses_without_a_stone_without_fp_and_while_an_echo_is_out() {
    let mut w = arena();
    arm(&mut w);
    // Too far from the stone.
    put_player(&mut w, [4.0, 0.0, 6.0]);
    let e = run(&mut w, &cmd(Command::UseItem { slot: BELL_QUICK }), 1);
    assert_eq!(
        echo_events(&e),
        [(EchoEvent::REFUSED, 0, EchoEvent::REASON_NO_STONE)]
    );
    assert!(w.echo.active.is_none());
    // Not enough FP.
    put_player(&mut w, RING_AT);
    w.actors.get_mut(w.player).unwrap().fp = 39.0;
    let e = run(&mut w, &cmd(Command::UseItem { slot: BELL_QUICK }), 1);
    assert_eq!(
        echo_events(&e),
        [(EchoEvent::REFUSED, POLITELLAMA, EchoEvent::REASON_NO_FP)]
    );
    assert_eq!(w.actors.get(w.player).unwrap().fp, 39.0);
    // Enough, and then a second ring does nothing more.
    w.actors.get_mut(w.player).unwrap().fp = 100.0;
    run(&mut w, &cmd(Command::UseItem { slot: BELL_QUICK }), 1);
    assert!(w.echo.active.is_some());
    let e = run(&mut w, &cmd(Command::UseItem { slot: BELL_QUICK }), 1);
    assert_eq!(
        echo_events(&e),
        [(EchoEvent::REFUSED, POLITELLAMA, EchoEvent::REASON_ALREADY)]
    );
    assert_eq!(members(&w).len(), 1);
    assert_eq!(w.actors.get(w.player).unwrap().fp, 60.0);
}

#[test]
fn using_something_other_than_the_bell_summons_nothing() {
    let mut w = arena();
    arm(&mut w);
    put_player(&mut w, RING_AT);
    // Quick slot 0 is the Vigil Flask.
    let e = run(&mut w, &cmd(Command::UseItem { slot: 0 }), 1);
    assert!(echo_events(&e).is_empty());
    assert!(w.echo.active.is_none());
}

#[test]
fn an_echo_not_yet_earned_is_not_offered() {
    let content = Content::load_embedded().unwrap();
    // The Test field has no Echo Stone and unlocks nothing: its table is empty of stones, and
    // the Echoes are only owned through their flags.
    let mut w = World::new(1, &content, "test_field").unwrap();
    assert!(w.echo_table.stones.is_empty());
    assert!(owned(&w).is_empty());
    w.state.flags.set("echo.politellama.unlocked", 1);
    assert_eq!(owned(&w).len(), 1);
    assert_eq!(
        w.echo_table.echoes[selected(&w).unwrap()].def.id,
        "echo_politellama"
    );
}

#[test]
fn cycle_spell_picks_the_next_echo_while_the_bell_is_selected() {
    let mut w = arena();
    arm(&mut w);
    put_player(&mut w, RING_AT);
    let press = TickInput {
        frame: ActionFrame {
            buttons: buttons::CYCLE_SPELL,
            ..ActionFrame::default()
        },
        commands: Vec::new(),
    };
    // The Bell isn't the selected quick item yet: the button does nothing.
    run(&mut w, &press, 2);
    run(&mut w, &idle(), 1);
    assert_eq!(w.echo.selected, 0);
    // Select it (the Vigil Flask is quick slot 0, the Lucid Flask 1, the Bell 2).
    w.actors
        .get_mut(w.player)
        .unwrap()
        .inventory
        .equipment
        .quick_active = BELL_QUICK;
    // A held button steps once, not every tick.
    run(&mut w, &press, 5);
    assert_eq!(w.echo.selected, 1);
    run(&mut w, &idle(), 1);
    run(&mut w, &press, 1);
    run(&mut w, &idle(), 1);
    run(&mut w, &press, 1);
    run(&mut w, &idle(), 1);
    run(&mut w, &press, 1);
    run(&mut w, &idle(), 1);
    // Wraps past the fourth.
    assert_eq!(w.echo.selected, 0);
    run(&mut w, &press, 1);
    run(&mut w, &idle(), 1);
    run(&mut w, &press, 1);
    run(&mut w, &idle(), 1);
    run(&mut w, &press, 1);
    run(&mut w, &idle(), 1);
    assert_eq!(
        selected(&w).map(|i| w.echo_table.echoes[i].num),
        Some(BRIAR_HOUNDS)
    );
    let e = run(&mut w, &cmd(Command::UseItem { slot: BELL_QUICK }), 1);
    assert_eq!(echo_events(&e), [(EchoEvent::SUMMONED, BRIAR_HOUNDS, 0)]);
    assert_eq!(members(&w).len(), 3, "three hounds");
}

// ---- Allies ---------------------------------------------------------------------------

#[test]
fn an_echo_follows_the_player() {
    let mut w = arena();
    summon_outside(&mut w);
    let id = members(&w)[0];
    // The player walks away along the floor; the Echo trails them.
    let walk = TickInput {
        frame: ActionFrame {
            move_x: 127,
            move_z: 0,
            buttons: 0,
        },
        commands: Vec::new(),
    };
    run(&mut w, &walk, 60);
    run(&mut w, &idle(), 120);
    let (p, e) = (
        w.actors.get(w.player).unwrap().pos,
        w.actors.get(id).unwrap().pos,
    );
    let d = dist_xz(p, e);
    assert!(d <= 4.0 && d > 1.0, "it stops near the player: {d}");
    assert!(p[0] > 2.0, "the player did walk: {p:?}");
}

#[test]
fn an_echo_far_behind_is_put_beside_the_player() {
    let mut w = arena();
    summon(&mut w, 0);
    let id = members(&w)[0];
    put_player(&mut w, [0.0, 0.0, -13.0]);
    run(&mut w, &idle(), 12);
    let d = dist_xz(
        w.actors.get(w.player).unwrap().pos,
        w.actors.get(id).unwrap().pos,
    );
    assert!(d < 6.0, "{d}");
}

#[test]
fn echoes_are_not_lockable_hit_by_the_player_or_hurting_them() {
    let mut w = arena();
    summon(&mut w, 0);
    let id = members(&w)[0];
    // Beside the player, with the player swinging: the swing passes through.
    let near = w.actors.get(w.player).unwrap().pos;
    {
        let e = w.actors.get_mut(id).unwrap();
        e.pos = [near[0], near[1], near[2] + 1.0];
    }
    let hp = w.actors.get(id).unwrap().hp;
    let mut events = Vec::new();
    for i in 0..30 {
        let light = TickInput {
            frame: ActionFrame {
                move_x: 0,
                move_z: 0,
                buttons: if i % 2 == 0 { buttons::LIGHT } else { 0 },
            },
            commands: Vec::new(),
        };
        tick(&mut w, &light);
        events.extend(w.events.drain());
        // Hold it in place so the swing really reaches it.
        let e = w.actors.get_mut(id).unwrap();
        e.pos = [near[0], near[1], near[2] + 1.0];
    }
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Hit(h) if h.target == id)),
        "the player's swings pass through an Echo"
    );
    assert_eq!(w.actors.get(id).unwrap().hp, hp);
    // The lock-on skips it.
    let lock = TickInput {
        frame: ActionFrame {
            buttons: buttons::LOCK,
            ..ActionFrame::default()
        },
        commands: Vec::new(),
    };
    run(&mut w, &lock, 1);
    assert_ne!(w.actors.get(w.player).unwrap().lock_target, Some(id));
}

/// One tick of the fight with the player standing still, collecting events.
fn fight_ticks(w: &mut World, n: u32) -> Vec<Event> {
    run(w, &idle(), n)
}

fn lock_on_boss(w: &mut World) {
    let boss = boss_body(w);
    w.actors.get_mut(w.player).unwrap().lock_target = Some(boss);
}

#[test]
fn an_echo_fights_the_boss_the_player_is_locked_on_to() {
    let mut w = arena();
    start_fight(&mut w);
    summon(&mut w, 1); // the Bone Sentinel
    lock_on_boss(&mut w);
    let boss = boss_body(&w);
    let hp = w.actors.get(boss).unwrap().hp;
    let events = fight_ticks(&mut w, 300);
    let id = members(&w)[0];
    let hits_on_boss = events
        .iter()
        .filter(|e| matches!(e, Event::Hit(h) if h.attacker == id && h.target == boss))
        .count();
    assert!(hits_on_boss >= 1, "the Echo hit the boss");
    assert!(w.actors.get(boss).unwrap().hp < hp);
    // It never hurts the player.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Hit(h) if h.attacker == id && h.target == w.player))
    );
}

#[test]
fn an_echo_leaves_a_boss_that_has_not_started_alone() {
    let mut w = arena();
    summon_outside(&mut w);
    assert_eq!(w.bosses[0].stage, crate::ai::boss::Stage::Dormant);
    lock_on_boss(&mut w);
    let boss = boss_body(&w);
    let hp = w.actors.get(boss).unwrap().hp;
    run(&mut w, &idle(), 200);
    assert_eq!(w.actors.get(boss).unwrap().hp, hp);
}

#[test]
fn a_bone_sentinel_draws_the_boss_off_the_player() {
    let mut w = arena();
    start_fight(&mut w);
    summon(&mut w, 1);
    let boss = boss_body(&w);
    let id = members(&w)[0];
    // The Sentinel (12 m aggro) next to the boss; the player well away.
    let bpos = w.actors.get(boss).unwrap().pos;
    w.actors.get_mut(id).unwrap().pos = [bpos[0], bpos[1], bpos[2] - 4.0];
    run(&mut w, &idle(), 12);
    let target = w
        .actors
        .get(boss)
        .and_then(|a| a.ai.as_ref())
        .unwrap()
        .target;
    assert_eq!(target, Some(id), "the boss turned on the Sentinel");
    // The Llama's taunt is shorter than the Sentinel's hold, and the Wisp has none.
    let info = |n: u32| w.echo_table.get(n).unwrap().def.aggro_m;
    assert!(info(BONE_SENTINEL) > info(POLITELLAMA));
    assert!(info(POLITELLAMA) > 0.0);
    assert_eq!(info(CHOIR_WISP), 0.0);
    assert_eq!(info(BRIAR_HOUNDS), 0.0);
}

#[test]
fn the_boss_goes_back_to_the_player_when_the_aggro_ends() {
    let mut w = arena();
    start_fight(&mut w);
    summon(&mut w, 1);
    let boss = boss_body(&w);
    let id = members(&w)[0];
    let bpos = w.actors.get(boss).unwrap().pos;
    w.actors.get_mut(id).unwrap().pos = [bpos[0], bpos[1], bpos[2] - 4.0];
    run(&mut w, &idle(), 12);
    assert_eq!(
        w.actors.get(boss).unwrap().ai.as_ref().unwrap().target,
        Some(id)
    );
    // The Sentinel falls back out of reach.
    w.actors.get_mut(id).unwrap().pos = [-9.0, 0.0, -9.0];
    run(&mut w, &idle(), 3);
    assert_eq!(
        w.actors.get(boss).unwrap().ai.as_ref().unwrap().target,
        Some(w.player)
    );
}

#[test]
fn briar_hounds_take_attack_tokens_in_reverse_two_at_a_time() {
    let mut w = arena();
    start_fight(&mut w);
    summon(&mut w, 3);
    lock_on_boss(&mut w);
    let boss = boss_body(&w);
    assert_eq!(members(&w).len(), 3);
    let tuning = w.ai.tuning.unwrap();
    let mut most = 0;
    for _ in 0..600 {
        run(&mut w, &idle(), 1);
        let held = tokens::held(&w, boss, crate::ai::Token::Melee, boss);
        most = most.max(held);
        assert!(held <= usize::from(tuning.melee_tokens), "{held} tokens");
    }
    assert_eq!(most, 2, "two hounds attacked at once at some point");
}

#[test]
fn a_choir_wisp_heals_the_player_slowly_and_shoots() {
    let mut w = arena();
    start_fight(&mut w);
    summon(&mut w, 2);
    lock_on_boss(&mut w);
    let p = w.actors.get_mut(w.player).unwrap();
    p.hp = p.max_hp / 2;
    let before = p.hp;
    let events = run(&mut w, &idle(), 180);
    let after = w.actors.get(w.player).unwrap().hp;
    // Three heals of 10 in three seconds, less the boss's 1 damage hits.
    assert!(after >= before + 20, "{before} -> {after}");
    assert!(after <= before + 30 + 5, "slowly: {before} -> {after}");
    // The wisp's bolt is an Echo projectile.
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::Projectile(p) if members(&w).contains(&p.shooter)
        )),
        "it cast"
    );
}

// ---- The bow --------------------------------------------------------------------------

#[test]
fn a_bow_after_the_summoning_raises_the_politellama_to_120_percent() {
    let mut w = arena();
    summon(&mut w, 0);
    let id = members(&w)[0];
    assert_eq!(damage_percent(&w, id), 100);
    let bow = w.echo_table.rules.as_ref().unwrap().bow_gesture;
    // The player is close to the Echo (it stands by the stone).
    let e = run(&mut w, &cmd(Command::Gesture(bow)), 1);
    assert_eq!(echo_events(&e), [(EchoEvent::BOWED, POLITELLAMA, 0)]);
    assert_eq!(damage_percent(&w, id), 120);
    // Other actors are unaffected, and a second bow is not a second bonus.
    assert_eq!(damage_percent(&w, w.player), 100);
    let e = run(&mut w, &cmd(Command::Gesture(bow)), 1);
    assert!(echo_events(&e).is_empty());
    assert_eq!(damage_percent(&w, id), 120);
}

#[test]
fn a_bow_too_late_or_to_another_echo_does_nothing() {
    let mut w = arena();
    summon(&mut w, 0);
    let id = members(&w)[0];
    let bow = w.echo_table.rules.as_ref().unwrap().bow_gesture;
    let window = w.echo_table.rules.as_ref().unwrap().bow_window_ticks;
    run(&mut w, &idle(), u32::from(window) + 5);
    let e = run(&mut w, &cmd(Command::Gesture(bow)), 1);
    assert!(echo_events(&e).is_empty());
    assert_eq!(damage_percent(&w, id), 100);
    // Another gesture is not the bow.
    let mut w = arena();
    summon(&mut w, 0);
    let e = run(&mut w, &cmd(Command::Gesture(bow + 1)), 1);
    assert!(echo_events(&e).is_empty());
    // The Sentinel has no courtesy to earn.
    let mut w = arena();
    summon(&mut w, 1);
    let e = run(&mut w, &cmd(Command::Gesture(bow)), 1);
    assert!(echo_events(&e).is_empty());
}

#[test]
fn a_bowed_to_politellama_hits_harder() {
    // The same swing, with and without the bow: the bowed Llama hits for 20 percent more.
    let swing = |bowed: bool| -> u32 {
        let mut w = arena();
        start_fight(&mut w);
        summon(&mut w, 0);
        if bowed {
            let bow = w.echo_table.rules.as_ref().unwrap().bow_gesture;
            run(&mut w, &cmd(Command::Gesture(bow)), 1);
        }
        lock_on_boss(&mut w);
        let boss = boss_body(&w);
        let id = members(&w)[0];
        // Put the Llama in front of the boss to swing, and the boss dormant-still.
        let bpos = w.actors.get(boss).unwrap().pos;
        for _ in 0..400 {
            let e = w.actors.get_mut(id).unwrap();
            if e.moves.current.is_none() {
                e.pos = [bpos[0], bpos[1], bpos[2] - 1.6];
            }
            tick(&mut w, &idle());
            let first = w.events.drain().into_iter().find_map(|e| match e {
                Event::Hit(h) if h.attacker == id && h.target == boss => Some(h.damage),
                _ => None,
            });
            if let Some(d) = first {
                return d;
            }
        }
        panic!("the Llama never hit the boss");
    };
    let (plain, polite) = (swing(false), swing(true));
    assert!(polite > plain, "{plain} -> {polite}");
}

// ---- Dismissal ------------------------------------------------------------------------

#[test]
fn a_fallen_echo_is_cleared_and_the_bell_can_ring_again() {
    let mut w = arena();
    summon(&mut w, 0);
    let id = members(&w)[0];
    {
        let a = w.actors.get_mut(id).unwrap();
        a.hp = 0;
        a.dead = true;
    }
    let events = run(&mut w, &idle(), 1);
    assert_eq!(echo_events(&events), [(EchoEvent::FELL, POLITELLAMA, 0)]);
    let wait = w.echo_table.rules.as_ref().unwrap().fallen_ticks;
    let events = run(&mut w, &idle(), u32::from(wait));
    assert_eq!(
        echo_events(&events),
        [(EchoEvent::DISMISSED, POLITELLAMA, 0)]
    );
    assert!(w.actors.get(id).is_none() && w.echo.active.is_none());
    let events = run(&mut w, &cmd(Command::UseItem { slot: BELL_QUICK }), 1);
    assert_eq!(
        echo_events(&events),
        [(EchoEvent::SUMMONED, POLITELLAMA, 0)]
    );
}

#[test]
fn the_players_death_sends_the_echo_away() {
    let mut w = arena();
    summon(&mut w, 0);
    let id = members(&w)[0];
    {
        let p = w.actors.get_mut(w.player).unwrap();
        p.hp = 0;
        p.dead = true;
    }
    let events = run(&mut w, &idle(), 1);
    assert_eq!(
        echo_events(&events),
        [(EchoEvent::DISMISSED, POLITELLAMA, 0)]
    );
    assert!(w.actors.get(id).is_none());
}

#[test]
fn the_state_hash_is_unchanged_until_the_bell_is_rung() {
    let content = Content::load_embedded().unwrap();
    let mut a = World::new(SEED, &content, SCENARIO).unwrap();
    let mut b = a.clone();
    // Cycling does nothing without the Bell selected, and nothing is hashed for it.
    let press = TickInput {
        frame: ActionFrame {
            buttons: buttons::CYCLE_SPELL,
            ..ActionFrame::default()
        },
        commands: Vec::new(),
    };
    run(&mut a, &press, 5);
    run(&mut b, &press, 5);
    assert!(a.echo.is_inert());
    assert_eq!(a.state_hash(), b.state_hash());
    // Ringing it changes the state.
    summon(&mut a, 0);
    run(&mut b, &idle(), 1);
    assert!(!a.echo.is_inert());
    assert_ne!(a.state_hash(), b.state_hash());
}

// ---- The golden replay ----------------------------------------------------------------

/// The scripted player: picks up the Bell, walks to the stone and rings it, bows, then walks
/// to the boss, locks on, and swings. With `echo` false it never rings the Bell.
struct Bot {
    echo: bool,
    stage: u8,
    last: u16,
    toggle: bool,
}

impl Bot {
    fn new(echo: bool) -> Self {
        Self {
            echo,
            stage: 0,
            last: 0,
            toggle: false,
        }
    }

    /// The stick that walks from `from` to `to`.
    fn toward(from: [f32; 3], to: [f32; 3]) -> (i8, i8) {
        let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
        let d = crate::math::dmath::sqrt(dx * dx + dz * dz).max(0.001);
        ((dx / d * 127.0) as i8, (dz / d * 127.0) as i8)
    }

    /// The next tick's input and commands, from the world after the last tick.
    fn act(&mut self, w: &World) -> (ActionFrame, Vec<Command>) {
        let p = w.actors.get(w.player).unwrap();
        let bell = w.echo_table.bell.unwrap();
        let mut commands = Vec::new();
        let mut stick = (0, 0);
        let mut pressed = 0;
        match self.stage {
            // Walk to the Bell lying by the spawn.
            0 => {
                stick = Self::toward(p.pos, [0.0, 0.0, -8.0]);
                if p.inventory.count(bell) > 0 {
                    self.stage = 1;
                    commands.push(Command::Equip {
                        slot: BELL_SLOT,
                        item: bell,
                    });
                }
            }
            // Through the fog gate to the Echo Stone. Interact passes the fog (issue #82).
            1 => {
                stick = Self::toward(p.pos, RING_AT);
                if crate::ai::boss::walking_into_fog(w, stick.0, stick.1) {
                    pressed = buttons::INTERACT;
                }
                if dist_xz(p.pos, [-4.0, 0.0, 3.0]) <= 2.0 {
                    stick = (0, 0);
                    self.stage = 2;
                    if self.echo {
                        commands.push(Command::UseItem { slot: BELL_QUICK });
                    }
                }
            }
            // Bow to the Echo the next tick.
            2 => {
                self.stage = 3;
                if self.echo {
                    let bow = w.echo_table.rules.as_ref().unwrap().bow_gesture;
                    commands.push(Command::Gesture(bow));
                }
            }
            // Wait out the intro.
            3 => {
                if w.bosses[0].stage == crate::ai::boss::Stage::Fight {
                    self.stage = 4;
                }
            }
            // Lock on and close in, then swing.
            _ => {
                let boss = w.actors.get(boss_body(w)).unwrap();
                if p.lock_target.is_none() {
                    pressed = buttons::LOCK;
                } else {
                    let d = dist_xz(p.pos, boss.pos);
                    if d > 2.0 {
                        stick = Self::toward(p.pos, boss.pos);
                    } else {
                        self.toggle = !self.toggle;
                        if self.toggle {
                            pressed = buttons::LIGHT;
                        }
                    }
                }
            }
        }
        // A button only counts on its press edge.
        pressed &= !self.last;
        self.last = pressed;
        (
            ActionFrame {
                move_x: stick.0,
                move_z: stick.1,
                buttons: pressed,
            },
            commands,
        )
    }
}

/// What a played fight left behind.
struct Fight {
    frames: Vec<ActionFrame>,
    commands: Vec<(u64, Command)>,
    events: Vec<Event>,
    /// The tick the boss fell, if it did.
    defeated: Option<u64>,
    world: World,
}

fn play(echo: bool) -> Fight {
    let mut w = arena();
    let mut bot = Bot::new(echo);
    let (mut frames, mut commands, mut events) = (Vec::new(), Vec::new(), Vec::new());
    let mut defeated = None;
    while w.tick < LONGEST_FIGHT && defeated.is_none() {
        let (frame, cmds) = bot.act(&w);
        for c in &cmds {
            commands.push((w.tick, c.clone()));
        }
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: cmds,
            },
        );
        frames.push(frame);
        for e in w.events.drain() {
            if matches!(&e, Event::Boss(b) if b.kind == BossEvent::DEFEATED) {
                defeated = Some(w.tick);
            }
            events.push(e);
        }
    }
    Fight {
        frames,
        commands,
        events,
        defeated,
        world: w,
    }
}

#[test]
fn the_bot_beats_the_test_boss_faster_with_an_echo() {
    let with = play(true);
    let without = play(false);
    let (a, b) = (with.defeated.unwrap(), without.defeated.unwrap());
    assert!(a + 60 < b, "with an Echo {a} ticks, alone {b}");
    for f in [&with, &without] {
        assert!(
            f.world
                .state
                .flags
                .is_set(&flags::boss_defeated("test_boss"))
        );
        assert!(!f.world.actors.get(f.world.player).unwrap().dead);
    }
}

/// Checks the facts a replay of the Echo fight must show.
fn check_the_fight(events: &[Event], w: &World) {
    let boss = boss_body(w);
    let echo = echo_events(events);
    assert_eq!(echo[0], (EchoEvent::SUMMONED, POLITELLAMA, 0), "{echo:?}");
    assert_eq!(echo[1], (EchoEvent::BOWED, POLITELLAMA, 0), "{echo:?}");
    let llama = events
        .iter()
        .find_map(|e| match e {
            Event::Echo(x) if x.kind == EchoEvent::SUMMONED => x.actor,
            _ => None,
        })
        .unwrap();
    let echo_hits = events
        .iter()
        .filter(|e| {
            matches!(e, Event::Hit(h)
                if h.attacker == llama && h.target == boss && h.result == HitResult::Connected)
        })
        .count();
    let player_hits = events
        .iter()
        .filter(|e| matches!(e, Event::Hit(h) if h.attacker == w.player && h.target == boss))
        .count();
    assert!(echo_hits >= 3, "the Llama landed {echo_hits} hits");
    assert!(player_hits >= 3, "the player landed {player_hits} hits");
    assert!(w.state.flags.is_set(&flags::boss_defeated("test_boss")));
    assert!(!w.actors.get(w.player).unwrap().dead, "the player lives");
    // The Echo never hurt the player.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Hit(h) if h.attacker == llama && h.target == w.player))
    );
}

#[test]
fn the_echo_bot_fight_shows_what_the_golden_must() {
    let f = play(true);
    check_the_fight(&f.events, &f.world);
}

/// Writes the golden. Run with
/// `cargo test -p lf-core record_the_echo_golden -- --ignored`.
#[test]
#[ignore]
fn record_the_echo_golden() {
    let f = play(true);
    check_the_fight(&f.events, &f.world);
    let content = Content::load_embedded().unwrap();
    let r = Replay::record(SEED, SCENARIO, &content, &f.frames, f.commands).unwrap();
    std::fs::write(GOLDEN, r.encode()).unwrap();
}

#[test]
fn the_golden_replay_has_an_echo_help_beat_the_test_boss() {
    let replay = Replay::decode(&std::fs::read(GOLDEN).unwrap()).unwrap();
    let mut w = World::new(
        replay.header.seed,
        &Content::load_embedded().unwrap(),
        &replay.header.scenario,
    )
    .unwrap();
    let frames = expand(&replay.frames);
    let mut events = Vec::new();
    for (i, frame) in frames.into_iter().enumerate() {
        let commands = replay
            .commands
            .iter()
            .filter(|(t, _)| *t == i as u64)
            .map(|(_, c)| c.clone())
            .collect();
        tick(&mut w, &TickInput { frame, commands });
        events.extend(w.events.drain());
    }
    check_the_fight(&events, &w);
}
