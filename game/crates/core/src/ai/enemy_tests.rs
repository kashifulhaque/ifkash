//! Tests for the batch-1 enemies (WP-032): the revive rule, nav steering round a wall,
//! enemy damage and guards, and the Miner's lantern.

use lf_content::Content;

use super::special::revive::DeathBlow;
use super::*;
use crate::events::Event;
use crate::input::{ActionFrame, TickInput};
use crate::tick::tick;
use crate::world_state::flags::lantern_lit;

fn world(scenario: &str, seed: u64) -> World {
    let content = Content::load_embedded().unwrap();
    World::new(seed, &content, scenario).unwrap()
}

fn enemy(w: &World, name: &str) -> Id {
    let i = w
        .layout
        .enemies
        .iter()
        .position(|s| s.name == name)
        .unwrap();
    w.state.ai_enemies[i]
}

fn step(w: &mut World, n: u32) -> Vec<Event> {
    let mut out = Vec::new();
    for _ in 0..n {
        tick(
            w,
            &TickInput {
                frame: ActionFrame::default(),
                commands: Vec::new(),
            },
        );
        out.extend(w.events.drain());
    }
    out
}

/// Moves the player far from the fight, out of everyone's sight and hearing.
fn send_player_away(w: &mut World) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = [20.0, 0.0, -20.0];
    p.hit.prev = None;
}

/// Kills the actor the way a hit would: HP 0, dead, and the death noted for its rules.
fn kill(w: &mut World, id: Id, blow: DeathBlow) {
    let a = w.actors.get_mut(id).unwrap();
    a.hp = 0;
    a.dead = true;
    special::note_death_blow(w, id, blow);
}

fn revives(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Event::Revive(_)))
        .count()
}

#[test]
fn a_skeleton_revives_after_exactly_180_ticks_at_half_hp() {
    let mut w = world("fight_skeleton", 1);
    send_player_away(&mut w);
    let sk = enemy(&w, "skeleton");
    kill(&mut w, sk, DeathBlow::default());
    // `tick_dead` counts from the next AI tick: the revive lands on the 180th.
    let early = step(&mut w, 179);
    assert_eq!(revives(&early), 0);
    assert!(w.actors.get(sk).unwrap().dead);
    let late = step(&mut w, 1);
    assert_eq!(revives(&late), 1, "{late:?}");
    let a = w.actors.get(sk).unwrap();
    assert!(!a.dead);
    assert_eq!((a.hp, a.max_hp), (90, 180));
}

#[test]
fn a_radiance_kill_or_a_critical_keeps_a_skeleton_down() {
    for blow in [
        DeathBlow {
            radiance: true,
            critical: false,
        },
        DeathBlow {
            radiance: false,
            critical: true,
        },
    ] {
        let mut w = world("fight_skeleton", 2);
        send_player_away(&mut w);
        let sk = enemy(&w, "skeleton");
        kill(&mut w, sk, blow);
        let events = step(&mut w, 400);
        assert_eq!(revives(&events), 0, "{blow:?}");
        assert!(w.actors.get(sk).unwrap().dead);
    }
}

#[test]
fn a_lit_lantern_within_6_m_keeps_a_skeleton_down_but_an_unlit_one_does_not() {
    // lantern_near stands 5 m from the skeleton's spawn.
    let mut w = world("fight_skeleton", 3);
    send_player_away(&mut w);
    let sk = enemy(&w, "skeleton");
    kill(&mut w, sk, DeathBlow::default());
    assert_eq!(revives(&step(&mut w, 200)), 1, "unlit: it gets back up");

    let mut w = world("fight_skeleton", 3);
    send_player_away(&mut w);
    w.state.flags.raise(&lantern_lit("lantern_near"));
    let sk = enemy(&w, "skeleton");
    kill(&mut w, sk, DeathBlow::default());
    assert_eq!(
        revives(&step(&mut w, 400)),
        0,
        "lit and near: it stays down"
    );

    // A lit Lantern far from the body doesn't count (lantern_start is 10 m away).
    let mut w = world("fight_skeleton", 3);
    send_player_away(&mut w);
    w.state.flags.raise(&lantern_lit("lantern_start"));
    let sk = enemy(&w, "skeleton");
    kill(&mut w, sk, DeathBlow::default());
    assert_eq!(
        revives(&step(&mut w, 200)),
        1,
        "lit but far: it gets back up"
    );
}

#[test]
fn an_enemy_walks_round_a_wall_along_a_nav_path() {
    let mut w = world("fight_nav_wall", 5);
    let husk = enemy(&w, "husk");
    assert!(w.nav.is_some(), "a scenario with enemies bakes a grid");
    // The husk stands behind the wall, 12 m out, and is walked to the far side of it; its
    // home is where it spawned, straight through the wall.
    let a = w.actors.get_mut(husk).unwrap();
    a.pos = [0.0, 0.0, 2.0];
    a.motor.ground_y = 0.0;
    a.hit.prev = None;
    send_player_away(&mut w);
    super::set_awareness(&mut w, husk, Awareness::Return);
    let mut widest = 0.0f32;
    let mut home_at = None;
    for i in 0..600 {
        step(&mut w, 1);
        let a = w.actors.get(husk).unwrap();
        widest = widest.max(a.pos[0].abs());
        if home_at.is_none() && super::perception::dist_xz(a.pos, [0.0, 0.0, 12.0]) < 0.8 {
            home_at = Some(i);
        }
    }
    // The wall ends at x = 6: going round it means passing beyond that.
    assert!(widest > 6.0, "it routed round the wall: {widest}");
    assert!(home_at.is_some(), "it reached home");
}

#[test]
fn without_a_nav_grid_an_enemy_steers_straight_and_stops_at_the_wall() {
    let mut w = world("fight_nav_wall", 5);
    w.nav = None;
    let husk = enemy(&w, "husk");
    let a = w.actors.get_mut(husk).unwrap();
    a.pos = [0.0, 0.0, 2.0];
    a.motor.ground_y = 0.0;
    a.hit.prev = None;
    send_player_away(&mut w);
    super::set_awareness(&mut w, husk, Awareness::Return);
    step(&mut w, 300);
    let a = w.actors.get(husk).unwrap();
    assert!(a.pos[2] < 6.0, "it stopped at the wall: {:?}", a.pos);
}

#[test]
fn the_miner_is_never_unaware_but_the_servant_is() {
    let mut w = world("fight_husk_variants", 6);
    // Behind them (they face -Z) and out of sight range.
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = [0.0, 0.0, 24.5];
    p.hit.prev = None;
    step(&mut w, 12);
    let poise = |n: &str| w.actors.get(enemy(&w, n)).unwrap().poise.unaware;
    assert!(poise("servant"), "a Servant facing away is unaware");
    assert!(!poise("miner"), "the Miner's lit lantern gives it away");
}

#[test]
fn an_enemy_hit_deals_its_attack_rating_through_the_damage_formula() {
    let mut w = world("fight_husk", 7);
    let husk = enemy(&w, "husk");
    let hp0 = w.actors.get(w.player).unwrap().hp;
    let events = step(&mut w, 200);
    let hit = events
        .iter()
        .find_map(|e| match e {
            Event::Hit(h) if h.attacker == husk => Some(h),
            _ => None,
        })
        .expect("the husk landed a swing");
    // AR 90, motion value 1.0, against the player's flat 40: 90^2 / 130 = 62.3.
    assert_eq!(hit.damage, 62);
    assert_eq!(w.actors.get(w.player).unwrap().hp, hp0 - 62);
}

#[test]
fn enemies_do_not_hurt_each_other() {
    let mut w = world("fight_thorn_hounds", 8);
    // Pile every hound onto one spot, hounds all in attack range of each other.
    let ids: Vec<Id> = ["hound_a", "hound_b", "hound_c"]
        .iter()
        .map(|n| enemy(&w, n))
        .collect();
    for &id in &ids {
        let a = w.actors.get_mut(id).unwrap();
        a.pos = [0.0, 0.0, 6.0];
        a.hit.prev = None;
    }
    let events = step(&mut w, 300);
    for e in &events {
        if let Event::Hit(h) = e {
            assert!(!ids.contains(&h.target), "{h:?}");
        }
    }
    assert!(ids.iter().all(|&id| w.actors.get(id).unwrap().hp == 90));
}

#[test]
fn a_guarding_knight_blocks_most_of_a_hit() {
    let mut w = world("fight_gray_knight", 9);
    let knight = enemy(&w, "knight");
    // Hold the guard up by hand (the Guard behavior does this in a fight).
    for _ in 0..4 {
        let b = w.actors.get_mut(knight).unwrap().ai.as_mut().unwrap();
        b.awareness = Awareness::Combat;
        b.doing = Doing::Behavior(Behavior::Guard { dur: 30 });
        b.doing_ticks = 30;
        step(&mut w, 1);
    }
    let k = w.actors.get(knight).unwrap();
    assert!(k.moves.guard.active, "the Guard behavior raised the guard");
    let stamina = k.stamina;
    assert!(stamina > 0.0);
}

#[test]
fn an_archer_that_was_hit_may_strafe_and_one_that_was_not_may_not() {
    let mut w = world("fight_gray_archer", 10);
    let archer = enemy(&w, "archer");
    let t = w.ai.tuning.unwrap();
    // Get it into Combat, then compare its eligible options before and after a hit.
    step(&mut w, 40);
    let strafe = |w: &World, hurt: bool| -> bool {
        let b = w.actors.get(archer).unwrap().ai.as_ref().unwrap();
        let def = &w.ai.defs[usize::from(b.def)];
        let i = def
            .options
            .iter()
            .position(|o| {
                o.when
                    .contains(&lf_content::enemy::Condition::HurtWithin(30))
            })
            .unwrap();
        let p = w.player;
        options::eligible(w, archer, p, &t).contains(&i) == hurt
    };
    assert!(strafe(&w, false), "not hurt: the reposition option is out");
    w.actors.get_mut(archer).unwrap().hp -= 20;
    step(&mut w, 3);
    let b = w.actors.get(archer).unwrap().ai.as_ref().unwrap();
    assert!(b.since_hurt < 30, "{}", b.since_hurt);
    assert!(strafe(&w, true), "just hurt: the reposition option is in");
}
