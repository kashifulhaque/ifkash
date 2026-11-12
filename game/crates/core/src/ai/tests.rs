//! AI framework tests on the `ai_yard` scenario (see `game/data/scenarios/ai_yard.ron`).

use lf_content::Content;
use lf_content::enemy::Condition;

use super::options::{Context, holds};
use super::perception;
use super::*;
use crate::events::Event;
use crate::input::{ActionFrame, TickInput, buttons};
use crate::tick::tick;

fn yard(seed: u64) -> World {
    let content = Content::load_embedded().unwrap();
    World::new(seed, &content, "ai_yard").unwrap()
}

/// The actor spawned for the spawn named `name`.
fn grunt(w: &World, name: &str) -> Id {
    let i = w
        .layout
        .enemies
        .iter()
        .position(|s| s.name == name)
        .unwrap();
    w.state.ai_enemies[i]
}

fn brain(w: &World, id: Id) -> &AiState {
    w.actors.get(id).unwrap().ai.as_ref().unwrap()
}

fn place_player(w: &mut World, pos: [f32; 3], yaw: f32) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.yaw = yaw;
    p.motor.ground_y = pos[1];
    p.motor.fall_peak_y = pos[1];
    p.hit.prev = None;
}

/// Ticks `n` times with `frame`, returning every awareness event.
fn run(w: &mut World, frame: ActionFrame, n: u32) -> Vec<AwarenessEvent> {
    let input = TickInput {
        frame,
        commands: Vec::new(),
    };
    let mut out = Vec::new();
    for _ in 0..n {
        tick(w, &input);
        for e in w.events.drain() {
            if let Event::Awareness(a) = e {
                out.push(a);
            }
        }
    }
    out
}

fn sees(w: &World, id: Id) -> bool {
    let a = w.actors.get(id).unwrap();
    let p = w.actors.get(w.player).unwrap();
    let def = w.ai.def(brain(w, id).def);
    perception::sees(&w.physics, a, p, &def.perception, &w.ai.tuning.unwrap())
}

#[test]
fn the_yard_spawns_its_grunts_idle_and_unaware() {
    let w = yard(1);
    assert_eq!(w.state.ai_enemies.len(), 5);
    for &id in &w.state.ai_enemies {
        let a = w.actors.get(id).unwrap();
        assert_eq!(a.hp, 200);
        assert!(a.poise.unaware);
        assert_eq!(brain(&w, id).awareness, Awareness::Idle);
    }
}

#[test]
fn an_enemy_behind_a_wall_does_not_see_the_player() {
    let mut w = yard(2);
    let hidden = grunt(&w, "grunt_hidden");
    // 8 m straight ahead of grunt_hidden, inside its cone and range, with the wall between.
    place_player(&mut w, [10.0, 0.0, 6.0], 0.0);
    assert!(!sees(&w, hidden));
    let events = run(&mut w, ActionFrame::default(), 90);
    assert!(events.iter().all(|e| e.actor != hidden), "{events:?}");
    assert_eq!(brain(&w, hidden).awareness, Awareness::Idle);

    // Control: as far away and inside the cone, but past the end of the wall.
    let mut w = yard(2);
    place_player(&mut w, [16.0, 0.0, 9.0], 0.0);
    assert!(sees(&w, hidden));
    let events = run(&mut w, ActionFrame::default(), 6);
    assert!(
        events
            .iter()
            .any(|e| e.actor == hidden && e.to == Awareness::Aware.code() as u8),
        "{events:?}"
    );
}

/// Holds the stick toward +X, with `DODGE` held too if `sprint`.
fn dash(sprint: bool) -> ActionFrame {
    ActionFrame {
        move_x: 127,
        move_z: 0,
        buttons: if sprint { buttons::DODGE } else { 0 },
    }
}

/// grunt_hidden's first awareness change with the player `behind` meters behind it,
/// running or sprinting sideways for 40 ticks. The player starts out of its cone.
fn first_reaction(behind: f32, sprint: bool) -> Option<(u8, [f32; 3])> {
    let mut w = yard(3);
    let hidden = grunt(&w, "grunt_hidden");
    place_player(&mut w, [10.0, 0.0, 14.0 + behind], 0.0);
    let events = run(&mut w, dash(sprint), 40);
    let first = events.iter().find(|e| e.actor == hidden)?;
    Some((first.to, brain(&w, hidden).noise_at))
}

#[test]
fn a_sprint_within_10_m_makes_it_suspicious() {
    let (to, noise_at) = first_reaction(9.0, true).expect("it heard the sprint");
    assert_eq!(to, Awareness::Suspicious.code() as u8);
    assert!(
        noise_at[2] > 22.0,
        "it investigates the player's spot: {noise_at:?}"
    );
    // Running (6 m) at the same distance isn't heard, and nor is a sprint past 10 m.
    assert_eq!(first_reaction(9.0, false), None);
    assert_eq!(first_reaction(11.5, true), None);
}

/// Sprints behind grunt_hidden until it turns Suspicious, then lets go of the stick.
fn make_suspicious(seed: u64) -> (World, Id) {
    let mut w = yard(seed);
    let hidden = grunt(&w, "grunt_hidden");
    place_player(&mut w, [10.0, 0.0, 23.0], 0.0);
    for _ in 0..40 {
        run(&mut w, dash(true), 1);
        if brain(&w, hidden).awareness == Awareness::Suspicious {
            return (w, hidden);
        }
    }
    panic!("grunt_hidden never heard the sprint");
}

#[test]
fn a_suspicious_enemy_turns_to_the_noise_and_finds_the_player() {
    let (mut w, hidden) = make_suspicious(4);
    // The player stops; the grunt walks toward the noise, turns, and sees them.
    let events = run(&mut w, ActionFrame::default(), 90);
    assert!(
        events
            .iter()
            .any(|e| e.actor == hidden && e.to == Awareness::Aware.code() as u8),
        "{events:?}"
    );
}

#[test]
fn a_suspicious_enemy_gives_up_after_its_timeout() {
    let (mut w, hidden) = make_suspicious(5);
    // Out of sight and silent: the player is moved far away behind the wall.
    place_player(&mut w, [10.0, 0.0, -30.0], 0.0);
    let t = w.ai.tuning.unwrap();
    let events = run(
        &mut w,
        ActionFrame::default(),
        u32::from(t.suspicious_ticks) + 6,
    );
    assert_eq!(
        events.last().filter(|e| e.actor == hidden).map(|e| e.to),
        Some(Awareness::Idle.code() as u8),
        "{events:?}"
    );
}

/// AI actors in an attack move this tick.
fn attacking(w: &World) -> Vec<Id> {
    w.actors
        .iter()
        .filter(|(_, a)| !a.dead && a.ai.is_some() && a.moves.current.is_some())
        .map(|(id, _)| id)
        .collect()
}

#[test]
fn three_enemies_never_attack_at_once() {
    for seed in [6, 7, 8, 9] {
        let mut w = yard(seed);
        let trio = [
            grunt(&w, "grunt_left"),
            grunt(&w, "grunt_mid"),
            grunt(&w, "grunt_right"),
        ];
        let mut most = 0;
        let mut attacked = [false; 3];
        let input = TickInput::default();
        for _ in 0..900 {
            tick(&mut w, &input);
            let now = attacking(&w);
            assert!(now.len() <= 2, "seed {seed}, tick {}: {now:?}", w.tick);
            most = most.max(now.len());
            for (i, id) in trio.iter().enumerate() {
                attacked[i] |= now.contains(id);
            }
        }
        // The pool did its job: two at once happened, and all three took turns.
        assert_eq!(most, 2, "seed {seed}");
        assert_eq!(attacked, [true; 3], "seed {seed}");
    }
}

#[test]
fn every_attack_holds_a_token_and_hits_the_player() {
    let mut w = yard(10);
    let input = TickInput::default();
    let mut hits = 0;
    for _ in 0..600 {
        tick(&mut w, &input);
        for e in w.events.drain() {
            if let Event::Hit(h) = e
                && h.target == w.player
            {
                hits += 1;
            }
        }
        for id in attacking(&w) {
            assert_eq!(brain(&w, id).token, Token::Melee);
        }
    }
    assert!(hits > 0);
}

#[test]
fn thinking_runs_at_10_hz_staggered_by_index() {
    for index in 0..6 {
        let ticks: Vec<u64> = (0..30).filter(|&t| thinks_on(t, index, 3)).collect();
        assert_eq!(ticks.len(), 10);
        assert!(ticks.windows(2).all(|p| p[1] - p[0] == 3));
        assert_eq!(ticks[0], u64::from((3 - index % 3) % 3));
    }
}

#[test]
fn a_leashed_enemy_walks_home_and_heals_to_full() {
    let mut w = yard(11);
    let mid = grunt(&w, "grunt_mid");
    run(&mut w, ActionFrame::default(), 60);
    assert_eq!(brain(&w, mid).awareness, Awareness::Combat);
    // Drag it past the 30 m leash and hurt it.
    let a = w.actors.get_mut(mid).unwrap();
    a.pos = [0.0, 0.0, 40.0];
    a.hp = 50;
    a.moves.current = None;
    a.hit.prev = None;
    let events = run(&mut w, ActionFrame::default(), 3);
    assert!(
        events
            .iter()
            .any(|e| e.actor == mid && e.to == Awareness::Return.code() as u8)
    );
    let home = w.layout.enemies[usize::from(brain(&w, mid).spawn)].pos;
    let input = TickInput::default();
    let mut healed = false;
    for _ in 0..600 {
        tick(&mut w, &input);
        let back = w.events.drain().into_iter().any(|e| {
            matches!(e, Event::Awareness(a)
                if a.actor == mid && a.to == Awareness::Idle.code() as u8)
        });
        if back {
            let a = w.actors.get(mid).unwrap();
            assert_eq!(a.hp, a.max_hp);
            assert!(perception::dist_xz(a.pos, home) <= w.ai.tuning.unwrap().arrive_m);
            healed = true;
            break;
        }
    }
    assert!(healed);
}

#[test]
fn losing_sight_for_240_ticks_ends_the_fight() {
    let mut w = yard(12);
    let mid = grunt(&w, "grunt_mid");
    run(&mut w, ActionFrame::default(), 60);
    assert_eq!(brain(&w, mid).awareness, Awareness::Combat);
    // The player vanishes past sight range (as if up a ladder). grunt_mid walks to where
    // it last saw them and finds nothing. grunt_hidden, which faces that way, is out of
    // the way.
    let hidden = grunt(&w, "grunt_hidden");
    let h = w.actors.get_mut(hidden).unwrap();
    h.hp = 0;
    h.dead = true;
    place_player(&mut w, [10.0, 0.0, 34.0], 0.0);
    let t = w.ai.tuning.unwrap();
    let events = run(
        &mut w,
        ActionFrame::default(),
        u32::from(t.lost_sight_ticks) + 30,
    );
    let ret = events
        .iter()
        .find(|e| e.actor == mid && e.to == Awareness::Return.code() as u8);
    assert!(ret.is_some(), "{events:?}");
}

#[test]
fn an_idle_enemy_walks_its_patrol_unaware() {
    let mut w = yard(13);
    let patrol = grunt(&w, "grunt_patrol");
    let z0 = w.actors.get(patrol).unwrap().pos[2];
    run(&mut w, ActionFrame::default(), 90);
    let a = w.actors.get(patrol).unwrap();
    assert!(a.pos[2] > z0 + 2.0, "{:?}", a.pos);
    assert_eq!(brain(&w, patrol).awareness, Awareness::Idle);
    // It walks away from the player: its back is turned, so a backstab can land.
    assert!(a.poise.unaware);
}

#[test]
fn resting_puts_ai_enemies_back_fresh() {
    let mut w = yard(14);
    let mid = grunt(&w, "grunt_mid");
    run(&mut w, ActionFrame::default(), 90);
    w.actors.get_mut(mid).unwrap().hp = 1;
    crate::world_state::lantern::rest(&mut w, 0);
    let a = w.actors.get(mid).unwrap();
    assert_eq!(a.hp, a.max_hp);
    assert_eq!(a.pos, [0.0, 0.0, 9.0]);
    assert_eq!(brain(&w, mid).awareness, Awareness::Idle);
    assert_eq!(brain(&w, mid).token, Token::None);
}

#[test]
fn conditions_read_the_context() {
    let x = Context {
        dist: 3.0,
        facing_deg: 20.0,
        target_facing_deg: 90.0,
        hp_frac: 0.25,
        target_hp_frac: 1.0,
        target_attacking: true,
        target_healing: false,
        target_guarding: false,
        phase: 0,
        since_hurt: 20,
        hidden: false,
    };
    assert!(holds(&Condition::DistBetween(2.0, 3.4), &x));
    assert!(!holds(&Condition::DistBetween(3.4, 5.0), &x));
    assert!(holds(&Condition::DistBelow(3.1), &x));
    assert!(!holds(&Condition::DistAbove(3.0), &x));
    assert!(holds(&Condition::FacingWithin(45.0), &x));
    assert!(!holds(&Condition::TargetFacingWithin(45.0), &x));
    assert!(holds(&Condition::HpBelow(0.3), &x));
    assert!(!holds(&Condition::HpAbove(0.3), &x));
    assert!(!holds(&Condition::TargetHpBelow(0.5), &x));
    assert!(holds(&Condition::TargetAttacking, &x));
    assert!(!holds(&Condition::TargetHealing, &x));
    assert!(!holds(&Condition::TargetCasting, &x));
    assert!(holds(&Condition::Phase(0), &x));
    assert!(holds(&Condition::HurtWithin(20), &x));
    assert!(!holds(&Condition::HurtWithin(19), &x));
}

#[test]
fn an_option_on_cooldown_is_not_eligible() {
    let mut w = yard(15);
    let mid = grunt(&w, "grunt_mid");
    let player = w.player;
    let t = w.ai.tuning.unwrap();
    // Stand the grunt 1 m in front of the player, facing it.
    let a = w.actors.get_mut(mid).unwrap();
    a.pos = [0.0, 0.0, 1.0];
    a.yaw = core::f32::consts::PI;
    let before = options::eligible(&w, mid, player, &t);
    assert!(before.contains(&0), "te_slash is in reach: {before:?}");
    w.actors
        .get_mut(mid)
        .unwrap()
        .ai
        .as_mut()
        .unwrap()
        .cooldowns[0] = 5;
    assert!(!options::eligible(&w, mid, player, &t).contains(&0));
}

#[test]
fn ai_worlds_stay_in_lockstep() {
    let mut a = yard(16);
    let input = TickInput::default();
    for _ in 0..300 {
        tick(&mut a, &input);
    }
    let mut b = a.clone();
    for _ in 0..300 {
        tick(&mut a, &input);
        tick(&mut b, &input);
    }
    assert_eq!(a.state_hash(), b.state_hash());
    let mut c = yard(16);
    for _ in 0..600 {
        tick(&mut c, &input);
    }
    assert_eq!(a.state_hash(), c.state_hash());
}

/// The state hash after `ticks` ticks of the yard fight, with the player standing in
/// grunt_mid's sight so the enemies wake and fight.
fn fight_hash(content: &Content, ticks: u32) -> u64 {
    let mut w = World::new(7, content, "ai_yard").unwrap();
    place_player(&mut w, [0.0, 0.0, 0.0], 0.0);
    run(&mut w, ActionFrame::default(), ticks);
    w.state_hash()
}

#[test]
fn adding_an_enemy_definition_does_not_change_an_existing_fights_hash() {
    let content = Content::load_embedded().unwrap();
    let before = fight_hash(&content, 240);

    // A new enemy whose number sorts before every existing one, so every index into the
    // sorted enemy table shifts by one.
    let mut extra = content.clone();
    let lowest = extra
        .enemies
        .values()
        .filter_map(|e| extra.registry.num(&e.id))
        .min()
        .unwrap();
    let num = lowest - 1;
    assert!(!extra.registry.ids.values().any(|&n| n == num));
    let mut def = extra
        .enemies
        .values()
        .find(|e| e.id == "test_grunt")
        .unwrap()
        .clone();
    def.id = "extra_test_enemy".to_string();
    extra.registry.ids.insert(def.id.clone(), num);
    extra.enemies.insert("extra_test_enemy".to_string(), def);

    let w = World::new(7, &extra, "ai_yard").unwrap();
    assert_eq!(w.ai.defs[0].num, num, "the new enemy sorts first");
    assert_eq!(fight_hash(&extra, 240), before);
}
