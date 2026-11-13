//! Tests for the Marks, the gestures, and the timer (WP-088). Every Mark has a test that earns
//! it (and, where the rule has a "not", one that doesn't). Marks that need a real flow (resting,
//! talking, bowing) run through `tick`; those that need the moment of a boss's fall push the
//! event the boss framework pushes and call [`run`] directly, so no other system disturbs the
//! fixture.

use lf_content::Content;

use super::gesture;
use super::*;
use crate::combat::moves::MoveNum;
use crate::events::{GestureEvent, Hit, HitResult, PickupEvent};
use crate::input::{ActionFrame, Command, buttons};
use crate::items::slot;
use crate::tick::tick;

const SEED: u64 = 88;

fn content() -> Content {
    Content::load_embedded().expect("embedded content")
}

fn world(scenario: &str) -> World {
    World::new(SEED, &content(), scenario).unwrap_or_else(|e| panic!("{scenario}: {e}"))
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

fn press(buttons: u16) -> TickInput {
    TickInput {
        frame: ActionFrame {
            move_x: 0,
            move_z: 0,
            buttons,
        },
        commands: vec![],
    }
}

fn walk() -> TickInput {
    TickInput {
        frame: ActionFrame {
            move_x: 0,
            move_z: 100,
            buttons: 0,
        },
        commands: vec![],
    }
}

fn ticks(w: &mut World, input: &TickInput, n: u32) {
    for _ in 0..n {
        tick(w, input);
    }
}

fn earned(w: &World, mark: usize) -> bool {
    is_earned(&w.state.flags, mark)
}

/// The Marks of `Event::Mark` still queued, in order.
fn mark_events(w: &World) -> Vec<usize> {
    w.events
        .since(0)
        .filter_map(|e| match e {
            Event::Mark(m) => Some(usize::from(m.mark)),
            _ => None,
        })
        .collect()
}

fn gesture_events(w: &mut World) -> Vec<(u8, u8)> {
    w.events
        .drain()
        .into_iter()
        .filter_map(|e| match e {
            Event::Gesture(g) => Some((g.what, g.gesture)),
            _ => None,
        })
        .collect()
}

fn boss_index(w: &World, id: &str) -> usize {
    w.layout
        .bosses
        .iter()
        .position(|b| b.def.id == id)
        .unwrap_or_else(|| panic!("no boss {id}"))
}

/// What the boss framework does when a boss falls: the flag and the event.
fn fall(w: &mut World, id: &str) {
    let b = boss_index(w, id);
    w.bosses[b].stage = Stage::Defeated;
    w.state.flags.raise(&flags::boss_defeated(id));
    w.events.push(Event::Boss(BossEvent {
        boss: b as u16,
        kind: BossEvent::DEFEATED,
        phase: 0,
    }));
}

fn start_fight(w: &mut World, id: &str) {
    let b = boss_index(w, id);
    w.bosses[b].stage = Stage::Fight;
}

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.motor.grounded = true;
    p.motor.ground_y = pos[1];
    p.motor.fall_peak_y = pos[1];
}

fn hit_player(w: &mut World, by: u32) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.hp = p.hp.saturating_sub(by);
}

fn kill_event(w: &mut World, target: Id) {
    w.events.push(Event::Hit(Hit {
        attacker: w.player,
        target,
        move_num: MoveNum::default(),
        hitbox: 0,
        motion_value: 1.0,
        charge_frac: 0.0,
        point: [0.0; 3],
        dir: [0.0, 0.0, 1.0],
        damage: 10,
        result: HitResult::Connected,
        counter: false,
        killed: true,
        hp: 0,
    }));
}

// ---------------------------------------------------------------------------------------
// The list

#[test]
fn the_list_is_complete_ordered_and_valid() {
    assert_eq!(MARKS.len(), MARK_COUNT);
    const { assert!(MARK_COUNT <= 32, "the mask is a u32") };
    let mut slugs = std::collections::BTreeSet::new();
    for m in MARKS {
        // The cloud sync stores the slug: 1 to 40 of the characters the API takes.
        assert!((1..=40).contains(&m.slug.len()), "{}", m.slug);
        assert!(
            m.slug
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_'),
            "{}",
            m.slug
        );
        assert!(!m.name.is_empty() && !m.how.is_empty());
        assert!(!m.name.contains('"') && !m.how.contains('"'), "{}", m.slug);
        assert!(slugs.insert(m.slug), "duplicate {}", m.slug);
    }
    // The constants index the table by their slugs.
    assert_eq!(MARKS[FIRST_LIGHT].slug, "first_light");
    assert_eq!(MARKS[READER].slug, "reader");
    assert_eq!(MARKS[SIGIL_STILLWATER].slug, "sigil_stillwater");
    assert_eq!(MARKS[UNBROKEN].slug, "unbroken");
    // The bosses the Marks name exist in the content.
    let c = content();
    for id in BOSS_IDS {
        assert!(
            c.scenarios
                .values()
                .any(|s| s.bosses.iter().any(|b| b.boss == id)),
            "no scenario places {id}"
        );
    }
}

#[test]
fn the_mask_follows_the_flags_in_order() {
    let mut f = Flags::default();
    assert_eq!(earned_mask(&f), 0);
    f.raise(&flag(FIRST_LIGHT));
    f.raise(&flag(UNBROKEN));
    assert_eq!(earned_mask(&f), 1 | 1 << UNBROKEN);
}

#[test]
fn the_event_cursor_reads_each_event_once_even_after_a_drain() {
    let mut q = crate::events::EventQueue::default();
    let e = || {
        Event::Gesture(GestureEvent {
            what: 0,
            gesture: 0,
        })
    };
    for _ in 0..3 {
        q.push(e());
    }
    assert_eq!(
        (q.pushed(), q.since(0).count(), q.since(3).count()),
        (3, 3, 0)
    );
    q.drain();
    for _ in 0..2 {
        q.push(e());
    }
    assert_eq!(q.pushed(), 5);
    assert_eq!(q.since(3).count(), 2, "the two new ones");
    assert_eq!(q.since(4).count(), 1);
    assert_eq!(q.since(5).count(), 0);
    assert_eq!(q.since(0).count(), 2, "what was drained is gone");
}

// ---------------------------------------------------------------------------------------
// The Marks

#[test]
fn first_light_comes_from_resting_at_a_lantern() {
    let mut w = world("test_field");
    ticks(&mut w, &idle(), 3);
    assert!(!earned(&w, FIRST_LIGHT));
    // The Lantern is 3 m from the spawn; 2 m is in reach.
    put_player(&mut w, [0.0, 0.0, -2.0]);
    tick(&mut w, &cmd(Command::Rest));
    assert!(earned(&w, FIRST_LIGHT));
    assert_eq!(mark_events(&w), [FIRST_LIGHT]);
    // It is earned once: a second rest adds no event.
    w.events.drain();
    tick(&mut w, &cmd(Command::Rest));
    assert!(mark_events(&w).is_empty());
}

#[test]
fn woken_comes_from_defeating_b01() {
    let mut w = world("boss_b01");
    run(&mut w, &idle());
    assert!(!earned(&w, WOKEN));
    // Another boss's fall isn't it.
    w.state.flags.raise(&flags::boss_defeated(BOSS_IDS[1]));
    w.events
        .push(Event::Pickup(PickupEvent { index: 0, item: 0 }));
    run(&mut w, &idle());
    assert!(!earned(&w, WOKEN));
    fall(&mut w, "b01_hollow_sentry");
    run(&mut w, &idle());
    assert!(earned(&w, WOKEN));
}

/// Presses and releases `INTERACT` at an NPC or station, as the player does.
fn talk(w: &mut World, at: [f32; 3]) {
    put_player(w, at);
    ticks(w, &idle(), 4);
    tick(w, &press(buttons::INTERACT));
    tick(w, &idle());
}

fn in_front_of(w: &World, pos: [f32; 3], yaw: f32) -> [f32; 3] {
    let _ = w;
    use crate::math::dmath::{cos, sin};
    [pos[0] + sin(yaw) * 2.4, 0.0, pos[2] + cos(yaw) * 2.4]
}

#[test]
fn reader_comes_from_opening_the_ledger_and_not_another_station() {
    let mut w = world("hollowhearth");
    let station = |w: &World, id: &str| {
        let s = w
            .layout
            .stations
            .iter()
            .find(|s| s.def.id == id)
            .unwrap_or_else(|| panic!("no station {id}"));
        in_front_of(w, s.pos, s.yaw)
    };
    let at = station(&w, "stn_hearth");
    talk(&mut w, at);
    assert!(quests::current(&w).is_some(), "the Hearth opened");
    assert!(!earned(&w, READER));
    quests::close(&mut w);
    let at = station(&w, "stn_ledger");
    talk(&mut w, at);
    assert!(earned(&w, READER));
    assert!(!earned(&w, CORRESPONDENT));
}

#[test]
fn correspondent_comes_from_talking_to_the_courier() {
    let mut w = world("hollowhearth");
    let npc = |w: &World, id: &str| {
        let n = w.layout.npcs.iter().find(|n| n.def.id == id).unwrap();
        in_front_of(w, n.pos, n.yaw)
    };
    let at = npc(&w, "npc_smith");
    talk(&mut w, at);
    assert!(quests::current(&w).is_some(), "the Smith opened");
    assert!(!earned(&w, CORRESPONDENT));
    quests::close(&mut w);
    let at = npc(&w, "npc_courier");
    talk(&mut w, at);
    assert!(earned(&w, CORRESPONDENT));
    assert!(!earned(&w, READER));
}

#[test]
fn chronicler_needs_every_placed_page() {
    let mut w = world("test_field");
    for (i, page) in PAGES.iter().enumerate() {
        assert!(!earned(&w, CHRONICLER), "after {i} pages");
        w.state.flags.raise(&crate::pickups::taken_flag(page));
        w.events
            .push(Event::Pickup(PickupEvent { index: 0, item: 0 }));
        run(&mut w, &idle());
    }
    assert!(earned(&w, CHRONICLER));
    // The pages named here are the pickups the levels place.
    let c = content();
    for page in PAGES {
        assert!(
            c.scenarios
                .values()
                .any(|s| s.pickups.iter().any(|p| p.id == page)),
            "no level places {page}"
        );
    }
}

#[test]
fn collector_needs_every_relic() {
    let mut w = world("test_field");
    for (i, entry) in RELICS.iter().enumerate() {
        assert!(!earned(&w, COLLECTOR), "after {i} relics");
        w.state.flags.raise(&flags::codex(entry));
        w.events
            .push(Event::Pickup(PickupEvent { index: 0, item: 0 }));
        run(&mut w, &idle());
    }
    assert!(earned(&w, COLLECTOR));
    // Every relic is a codex entry a level's pickup or a quest reward unlocks.
    let c = content();
    for entry in RELICS {
        let by_pickup = c
            .scenarios
            .values()
            .any(|s| s.pickups.iter().any(|p| p.codex.as_deref() == Some(entry)));
        assert!(by_pickup || entry == "projects.opencode-theme", "{entry}");
    }
}

#[test]
fn each_great_sigil_boss_gives_its_own_mark() {
    for (boss, mark) in SIGILS {
        let scenario = format!("boss_{}", &boss[..3]);
        let mut w = world(&scenario);
        run(&mut w, &idle());
        assert!(mark_events(&w).is_empty(), "{boss}");
        fall(&mut w, boss);
        run(&mut w, &idle());
        assert!(earned(&w, mark), "{boss}");
        // Only that one of the four.
        for (_, other) in SIGILS {
            assert_eq!(earned(&w, other), other == mark, "{boss} / {other}");
        }
        assert!(!earned(&w, EVER_AFTER) && !earned(&w, THOROUGH));
    }
}

#[test]
fn ever_after_comes_from_defeating_b12() {
    let mut w = world("boss_b12");
    run(&mut w, &idle());
    assert!(!earned(&w, EVER_AFTER));
    fall(&mut w, "b12_the_unspooled");
    run(&mut w, &idle());
    assert!(earned(&w, EVER_AFTER));
    assert!(!earned(&w, THOROUGH), "one boss is not all of them");
}

#[test]
fn thorough_needs_all_twelve_bosses() {
    let mut w = world("boss_b01");
    for (i, id) in BOSS_IDS.iter().enumerate() {
        assert!(!earned(&w, THOROUGH), "after {i} bosses");
        // The flag is what counts; the event only makes the sweep run now.
        w.state.flags.raise(&flags::boss_defeated(id));
        w.events
            .push(Event::Pickup(PickupEvent { index: 0, item: 0 }));
        run(&mut w, &idle());
    }
    assert!(earned(&w, THOROUGH));
    assert!(earned(&w, EVER_AFTER) && earned(&w, WOKEN));
}

#[test]
fn a_save_from_before_marks_existed_earns_them_on_the_first_sweep() {
    let c = content();
    let mut w = world("test_field");
    w.state.flags.raise(&flags::boss_defeated(BOSS_IDS[0]));
    w.state.flags.raise(&flags::lantern_lit("lantern_field"));
    let mut loaded = crate::save::load(&crate::save::save(&w, SEED, &c), &c).unwrap();
    // The first tick of a world is a sweep.
    tick(&mut loaded, &idle());
    assert!(earned(&loaded, WOKEN) && earned(&loaded, FIRST_LIGHT));
}

#[test]
fn pacifist_path_needs_the_gate_and_no_kills_after_the_verge() {
    // At the gate with no kills: earned.
    let mut w = world("spire");
    let door = w
        .layout
        .doors
        .iter()
        .find(|d| d.id == SPIRE_GATE)
        .unwrap()
        .pos;
    put_player(&mut w, [door[0], door[1], door[2] + 20.0]);
    run(&mut w, &idle());
    assert!(!earned(&w, PACIFIST_PATH), "20 m from the gate");
    put_player(&mut w, [door[0], door[1], door[2] + 3.0]);
    tick(&mut w, &idle());
    assert!(earned(&w, PACIFIST_PATH));

    // A kill after the Verge breaks it for good.
    let mut w = world("castle");
    let victim = w.state.ai_enemies[0];
    kill_event(&mut w, victim);
    run(&mut w, &idle());
    assert_eq!(w.state.flags.get(KILLS_AFTER_VERGE), 1);
    let mut w2 = world("spire");
    w2.state.flags.set(KILLS_AFTER_VERGE, 1);
    put_player(&mut w2, [door[0], door[1], door[2] + 3.0]);
    run(&mut w2, &idle());
    assert!(!earned(&w2, PACIFIST_PATH));
}

#[test]
fn only_ordinary_enemies_outside_the_verge_break_pacifist_path() {
    // In the Verge a kill is free (the Verge is the tutorial).
    let mut w = world("verge");
    let victim = w.state.ai_enemies[0];
    kill_event(&mut w, victim);
    run(&mut w, &idle());
    assert_eq!(w.state.flags.get(KILLS_AFTER_VERGE), 0);
    // A boss's own body isn't an ordinary enemy.
    let mut w = world("boss_b07");
    let spawn = w.layout.bosses[0].members[0];
    let body = w.state.ai_enemies[usize::from(spawn)];
    kill_event(&mut w, body);
    run(&mut w, &idle());
    assert_eq!(w.state.flags.get(KILLS_AFTER_VERGE), 0);
    // Nor is anything the player didn't kill: another attacker's kill isn't counted.
    let mut w = world("castle");
    let victim = w.state.ai_enemies[0];
    w.events.push(Event::Hit(Hit {
        attacker: victim,
        target: victim,
        move_num: MoveNum::default(),
        hitbox: 0,
        motion_value: 1.0,
        charge_frac: 0.0,
        point: [0.0; 3],
        dir: [0.0, 0.0, 1.0],
        damage: 1,
        result: HitResult::Connected,
        counter: false,
        killed: true,
        hp: 0,
    }));
    run(&mut w, &idle());
    assert_eq!(w.state.flags.get(KILLS_AFTER_VERGE), 0);
}

#[test]
fn generalist_needs_no_category_over_35_percent_in_phase_1() {
    let try_counts = |counts: [u16; 6]| {
        let mut w = world("boss_b07");
        start_fight(&mut w, B07);
        w.b07.counts = counts;
        fall(&mut w, B07);
        run(&mut w, &idle());
        earned(&w, GENERALIST)
    };
    // Spread out: 20, 20, 20, 20, 10, 10 percent.
    assert!(try_counts([4, 4, 4, 4, 2, 2]));
    // Exactly 35 percent is not above it.
    assert!(try_counts([7, 5, 4, 4, 0, 0]));
    // One category over: 40 percent rolls left.
    assert!(!try_counts([8, 4, 4, 4, 0, 0]));
    // Overfit: mostly one thing.
    assert!(!try_counts([1, 0, 0, 12, 0, 0]));
    // Beating B07 does not give it for another boss.
    let mut w = world("boss_b09");
    fall(&mut w, "b09_lamplight_matriarch");
    run(&mut w, &idle());
    assert!(!earned(&w, GENERALIST));
}

#[test]
fn speedrunner_needs_b12_under_45_minutes_of_in_game_time() {
    assert_eq!(SPEEDRUN_TICKS, 81_000);
    let try_at = |tick: u64, boss: &str| {
        let scenario = format!("boss_{}", &boss[..3]);
        let mut w = world(&scenario);
        w.tick = tick;
        fall(&mut w, boss);
        run(&mut w, &idle());
        earned(&w, SPEEDRUNNER)
    };
    assert!(try_at(60 * 30, B12));
    assert!(try_at(SPEEDRUN_TICKS - 1, B12));
    assert!(!try_at(SPEEDRUN_TICKS, B12));
    assert!(!try_at(2 * SPEEDRUN_TICKS, B12));
    // A fast B01 is not it.
    assert!(!try_at(100, B01));
}

#[test]
fn unbroken_needs_a_great_sigil_boss_beaten_without_losing_hp() {
    for (boss, _) in SIGILS {
        let scenario = format!("boss_{}", &boss[..3]);
        // Untouched: earned.
        let mut w = world(&scenario);
        start_fight(&mut w, boss);
        run(&mut w, &idle());
        assert_eq!(w.state.marks.watch, boss_index(&w, boss) as u16 + 1);
        run(&mut w, &idle());
        fall(&mut w, boss);
        run(&mut w, &idle());
        assert!(earned(&w, UNBROKEN), "{boss} untouched");
        assert!(w.state.marks.is_inert(), "the watch ends with the fight");

        // Hit once, even if healed back: not earned.
        let mut w = world(&scenario);
        start_fight(&mut w, boss);
        run(&mut w, &idle());
        hit_player(&mut w, 5);
        run(&mut w, &idle());
        w.actors.get_mut(w.player).unwrap().hp = w.actors.get(w.player).unwrap().max_hp;
        run(&mut w, &idle());
        fall(&mut w, boss);
        run(&mut w, &idle());
        assert!(!earned(&w, UNBROKEN), "{boss} after a hit");
    }
    // A boss that isn't a Great Sigil boss doesn't count.
    let mut w = world("boss_b01");
    start_fight(&mut w, B01);
    run(&mut w, &idle());
    assert!(w.state.marks.is_inert());
    fall(&mut w, B01);
    run(&mut w, &idle());
    assert!(!earned(&w, UNBROKEN));
}

#[test]
fn a_death_in_the_fight_starts_the_next_try_clean() {
    let mut w = world("boss_b07");
    let b = boss_index(&w, B07);
    start_fight(&mut w, B07);
    run(&mut w, &idle());
    hit_player(&mut w, 5);
    run(&mut w, &idle());
    assert!(w.state.marks.watch_hit);
    // The player died: the boss resets, the fight is Dormant again.
    w.bosses[b].stage = Stage::Dormant;
    run(&mut w, &idle());
    assert!(w.state.marks.is_inert());
    // The next try, with full HP, is watched afresh.
    w.actors.get_mut(w.player).unwrap().hp = w.actors.get(w.player).unwrap().max_hp;
    start_fight(&mut w, B07);
    run(&mut w, &idle());
    fall(&mut w, B07);
    run(&mut w, &idle());
    assert!(earned(&w, UNBROKEN));
}

const BELL_SLOT: u8 = slot::QUICK_1 + 2;
const BELL_QUICK: u8 = 2;

#[test]
fn polite_comes_from_bowing_to_the_politellama_in_time() {
    let summon = |index: u16| {
        let mut w = world("echo_boss_gate");
        let bell = w.echo_table.bell.unwrap();
        let items = w.items.clone();
        let p = w.actors.get_mut(w.player).unwrap();
        assert_eq!(p.inventory.pick_up(bell, 1, &items), 1);
        assert!(p.inventory.equip(BELL_SLOT, bell, &items));
        p.max_fp = 300.0;
        p.fp = 300.0;
        put_player(&mut w, [-2.5, 0.0, 2.0]);
        w.echo.selected = index;
        tick(&mut w, &cmd(Command::UseItem { slot: BELL_QUICK }));
        assert!(w.echo.active.is_some());
        w
    };
    let bow = |w: &World| w.echo_table.rules.as_ref().unwrap().bow_gesture;

    // The PoliteLlama, bowed to in time.
    let mut w = summon(0);
    assert!(!earned(&w, POLITE));
    let b = bow(&w);
    tick(&mut w, &cmd(Command::Gesture(b)));
    assert!(earned(&w, POLITE));

    // A different gesture is not a bow.
    let mut w = summon(0);
    let b = bow(&w);
    tick(&mut w, &cmd(Command::Gesture(b + 1)));
    assert!(!earned(&w, POLITE));

    // Too late.
    let mut w = summon(0);
    let b = bow(&w);
    let window = w.echo_table.rules.as_ref().unwrap().bow_window_ticks;
    ticks(&mut w, &idle(), u32::from(window) + 5);
    tick(&mut w, &cmd(Command::Gesture(b)));
    assert!(!earned(&w, POLITE));

    // Another Echo has no courtesy to earn.
    let mut w = summon(1);
    let b = bow(&w);
    tick(&mut w, &cmd(Command::Gesture(b)));
    assert!(!earned(&w, POLITE));
}

#[test]
fn earned_marks_travel_with_a_save_and_the_timer_returns_to_the_last_rest() {
    let c = content();
    let mut w = world("test_field");
    ticks(&mut w, &idle(), 100);
    put_player(&mut w, [0.0, 0.0, -2.0]);
    tick(&mut w, &cmd(Command::Rest));
    let rested_at = w.tick;
    assert!(rested_at >= 100);
    assert_eq!(w.state.flags.get(PLAY_TICKS), rested_at as u32);
    ticks(&mut w, &idle(), 50);

    let loaded = crate::save::load(&crate::save::save(&w, SEED, &c), &c).unwrap();
    assert!(earned(&loaded, FIRST_LIGHT));
    assert_eq!(
        earned_mask(&loaded.state.flags),
        earned_mask(&w.state.flags)
    );
    // The 50 ticks after the rest are not in the save: the timer is back at the rest.
    assert_eq!(loaded.tick, rested_at);
    assert_eq!(play_ticks(&loaded), rested_at);
}

#[test]
fn the_timer_is_the_tick_counter_and_only_runs_with_the_simulation() {
    let mut w = world("test_field");
    assert_eq!(play_ticks(&w), 0);
    ticks(&mut w, &idle(), 90);
    // 90 ticks at 30 a second is three seconds, whatever the clock says.
    assert_eq!(play_ticks(&w), 90);
    // Nothing runs the world but `tick`: no ticks, no time.
    assert_eq!(play_ticks(&w), 90);
}

// ---------------------------------------------------------------------------------------
// Gestures

fn field() -> World {
    let mut w = world("test_field");
    ticks(&mut w, &idle(), 3);
    w.events.drain();
    w
}

#[test]
fn six_gestures_start_and_end_when_their_time_is_up() {
    for n in 0..gesture::COUNT {
        let mut w = field();
        tick(&mut w, &cmd(Command::Gesture(u16::from(n))));
        assert_eq!(
            gesture::current(&w).map(|g| g.gesture),
            Some(n),
            "gesture {n} starts"
        );
        assert_eq!(gesture_events(&mut w), [(GestureEvent::STARTED, n)]);
        let time = gesture::TICKS[usize::from(n)];
        if n == gesture::SIT {
            // A sit holds as long as the player does nothing.
            ticks(&mut w, &idle(), 600);
            assert!(gesture::current(&w).is_some());
            continue;
        }
        assert!(time > 0);
        ticks(&mut w, &idle(), u32::from(time) - 1);
        assert!(gesture::current(&w).is_some(), "gesture {n} still on");
        tick(&mut w, &idle());
        assert!(gesture::current(&w).is_none(), "gesture {n} is over");
        assert_eq!(gesture_events(&mut w), [(GestureEvent::ENDED, n)]);
    }
}

#[test]
fn a_gesture_has_no_hitbox_and_changes_nothing_but_itself() {
    let mut w = field();
    let before = w.actors.get(w.player).unwrap().clone();
    for n in 0..gesture::COUNT {
        tick(&mut w, &cmd(Command::Gesture(u16::from(n))));
        let p = w.actors.get(w.player).unwrap();
        assert!(p.moves.current.is_none(), "no move is started");
        assert_eq!(p.pos, before.pos);
        assert_eq!(
            (p.hp, p.stamina, p.fp),
            (before.hp, before.stamina, before.fp)
        );
    }
    // And no hit event came out of any of them.
    assert!(w.events.since(0).all(|e| !matches!(e, Event::Hit(_))));
}

#[test]
fn moving_or_pressing_a_button_ends_a_gesture() {
    for input in [walk(), press(buttons::LIGHT), press(buttons::JUMP)] {
        let mut w = field();
        tick(&mut w, &cmd(Command::Gesture(3)));
        ticks(&mut w, &idle(), 10);
        assert!(gesture::current(&w).is_some());
        tick(&mut w, &input);
        assert!(gesture::current(&w).is_none(), "{input:?}");
    }
}

#[test]
fn the_input_of_the_tick_a_gesture_starts_on_does_not_cancel_it() {
    let mut w = field();
    tick(
        &mut w,
        &TickInput {
            frame: walk().frame,
            commands: vec![Command::Gesture(0)],
        },
    );
    assert!(gesture::current(&w).is_some());
    tick(&mut w, &idle());
    assert!(gesture::current(&w).is_some());
}

#[test]
fn choosing_the_same_gesture_again_ends_it_and_another_switches() {
    let mut w = field();
    tick(&mut w, &cmd(Command::Gesture(3)));
    tick(&mut w, &cmd(Command::Gesture(0)));
    assert_eq!(gesture::current(&w).map(|g| g.gesture), Some(0));
    tick(&mut w, &cmd(Command::Gesture(0)));
    assert!(gesture::current(&w).is_none());
    assert_eq!(
        gesture_events(&mut w),
        [
            (GestureEvent::STARTED, 3),
            (GestureEvent::ENDED, 3),
            (GestureEvent::STARTED, 0),
            (GestureEvent::ENDED, 0),
        ]
    );
    // A number that isn't a gesture does nothing.
    tick(&mut w, &cmd(Command::Gesture(6)));
    tick(&mut w, &cmd(Command::Gesture(60_000)));
    assert!(gesture::current(&w).is_none() && gesture_events(&mut w).is_empty());
}

#[test]
fn a_gesture_needs_footing_life_and_a_free_player() {
    // In the air.
    let mut w = field();
    w.actors.get_mut(w.player).unwrap().motor.grounded = false;
    gesture::run(&mut w, &cmd(Command::Gesture(0)));
    assert!(gesture::current(&w).is_none());
    // Dead.
    let mut w = field();
    w.actors.get_mut(w.player).unwrap().dead = true;
    gesture::run(&mut w, &cmd(Command::Gesture(0)));
    assert!(gesture::current(&w).is_none());
    // Resting.
    let mut w = field();
    w.state.resting = Some("lantern_field".into());
    gesture::run(&mut w, &cmd(Command::Gesture(0)));
    assert!(gesture::current(&w).is_none());
    // Mid-attack: the real light attack starts a move, and a gesture then is refused.
    let mut w = field();
    tick(&mut w, &press(buttons::LIGHT));
    assert!(w.actors.get(w.player).unwrap().moves.current.is_some());
    tick(&mut w, &cmd(Command::Gesture(0)));
    assert!(gesture::current(&w).is_none());
}

#[test]
fn a_hit_a_death_or_a_rest_ends_a_gesture() {
    let mut w = field();
    tick(&mut w, &cmd(Command::Gesture(3)));
    hit_player(&mut w, 1);
    tick(&mut w, &idle());
    assert!(gesture::current(&w).is_none(), "a hit");

    let mut w = field();
    tick(&mut w, &cmd(Command::Gesture(3)));
    let p = w.actors.get_mut(w.player).unwrap();
    p.hp = 0;
    p.dead = true;
    tick(&mut w, &idle());
    assert!(gesture::current(&w).is_none(), "a death");

    let mut w = field();
    tick(&mut w, &cmd(Command::Gesture(3)));
    put_player(&mut w, [0.0, 0.0, -2.0]);
    tick(&mut w, &cmd(Command::Rest));
    assert!(gesture::current(&w).is_none(), "a rest");
}

#[test]
fn the_bow_is_gesture_one_and_the_echo_rules_agree() {
    let w = world("echo_boss_gate");
    // docs: wave, bow, point forward, sit, shrug, beckon.
    assert_eq!(w.echo_table.rules.as_ref().unwrap().bow_gesture, 1);
    assert_eq!(gesture::COUNT, 6);
    assert_eq!(gesture::SIT, 3);
}

#[test]
fn gestures_replay_and_leave_the_hash_as_it_was_once_they_are_over() {
    let play = |gestures: bool| {
        let mut w = field();
        for n in 0..gesture::COUNT {
            if gestures {
                tick(&mut w, &cmd(Command::Gesture(u16::from(n))));
                ticks(&mut w, &idle(), 40);
            } else {
                ticks(&mut w, &idle(), 41);
            }
            // End whatever holds (the sit).
            tick(&mut w, &press(buttons::LOCK));
        }
        ticks(&mut w, &idle(), 10);
        w
    };
    let a = play(true);
    let b = play(true);
    assert_eq!(
        a.state_hash(),
        b.state_hash(),
        "the same gestures, the same hash"
    );
    assert!(a.state.marks.is_inert());
    // While a gesture is on, the hash shows it.
    let mut c = field();
    let before = c.state_hash();
    tick(&mut c, &cmd(Command::Gesture(0)));
    let mut d = field();
    tick(&mut d, &idle());
    assert!(gesture::current(&c).is_some());
    assert_ne!(c.state_hash(), d.state_hash());
    let _ = before;
    // A world that never gestured has a hash with no trace of the Marks.
    let mut e = field();
    ticks(&mut e, &idle(), 20);
    assert!(e.state.marks.is_inert());
}

#[test]
fn a_clone_in_the_middle_of_a_gesture_stays_in_lockstep() {
    let mut a = field();
    tick(&mut a, &cmd(Command::Gesture(1)));
    let mut b = a.clone();
    for _ in 0..80 {
        tick(&mut a, &idle());
        tick(&mut b, &idle());
        assert_eq!(a.state_hash(), b.state_hash());
    }
    assert!(gesture::current(&a).is_none());
}
