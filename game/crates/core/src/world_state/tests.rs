use lf_content::Content;

use super::flags::{Flags, lantern_lit};
use crate::actor::Actor;
use crate::events::{Event, LifeEvent};
use crate::input::{ActionFrame, Command, TickInput};
use crate::tick::tick;
use crate::world::World;

fn field() -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(9, &content, "test_field").expect("test_field exists")
}

fn player(w: &World) -> &Actor {
    w.actors.get(w.player).unwrap()
}

fn player_mut(w: &mut World) -> &mut Actor {
    let id = w.player;
    w.actors.get_mut(id).unwrap()
}

fn step(w: &mut World, frame: ActionFrame, commands: Vec<Command>) {
    tick(w, &TickInput { frame, commands });
}

fn idle(w: &mut World, ticks: u32) {
    for _ in 0..ticks {
        step(w, ActionFrame::default(), Vec::new());
    }
}

fn life(w: &mut World) -> Vec<LifeEvent> {
    w.events
        .drain()
        .into_iter()
        .filter_map(|e| match e {
            Event::Life(l) => Some(l),
            _ => None,
        })
        .collect()
}

/// Puts the player on the ground at `pos` (the test field is flat there).
fn stand_at(w: &mut World, pos: [f32; 3]) {
    let p = player_mut(w);
    p.pos = pos;
    p.motor.grounded = true;
}

/// Kills the player where they stand, as a hit or a fall would.
fn kill(w: &mut World) {
    let p = player_mut(w);
    p.hp = 0;
    p.dead = true;
}

fn delay(w: &World) -> u32 {
    u32::from(w.locomotion.death.respawn_delay_ticks)
}

/// Idles for the respawn delay plus `extra` ticks.
fn idle_delay(w: &mut World, extra: i32) {
    let n = delay(w) as i32 + extra;
    idle(w, n as u32);
}

const LANTERN: [f32; 3] = [0.0, 0.0, -3.0];

#[test]
fn flags_default_to_zero_and_zero_means_unset() {
    let mut f = Flags::default();
    assert_eq!(f.get("boss.warden.defeated"), 0);
    assert!(!f.is_set("boss.warden.defeated"));
    f.raise("boss.warden.defeated");
    f.set("door.gate.open", 3);
    assert!(f.is_set("boss.warden.defeated"));
    assert_eq!(f.get("door.gate.open"), 3);
    // Iteration is in key order, whatever the insertion order.
    assert_eq!(
        f.iter().collect::<Vec<_>>(),
        [("boss.warden.defeated", 1), ("door.gate.open", 3)]
    );
    f.set("door.gate.open", 0);
    f.clear("boss.warden.defeated");
    assert_eq!(f, Flags::default());
}

#[test]
fn resting_refills_sets_the_respawn_point_and_respawns_enemies() {
    let mut w = field();
    stand_at(&mut w, [0.5, 0.0, -2.0]);
    {
        let p = player_mut(&mut w);
        p.hp = 10;
        p.stamina = 1.0;
        p.fp = 0.0;
        p.inventory.flasks.vigil = 0;
        p.inventory.flasks.lucid = 0;
    }
    let dummy = w.state.enemies[0];
    {
        let d = w.actors.get_mut(dummy).unwrap();
        d.hp = 1;
        d.pos = [3.0, 0.0, 3.0];
    }
    step(&mut w, ActionFrame::default(), vec![Command::Rest]);
    let p = player(&w);
    assert_eq!((p.hp, p.stamina, p.fp), (p.max_hp, p.max_stamina, p.max_fp));
    let f = p.inventory.flasks;
    assert_eq!((f.vigil, f.lucid), (f.vigil_max, f.lucid_max()));
    assert_eq!(w.state.respawn_lantern.as_deref(), Some("lantern_field"));
    assert_eq!(w.state.resting.as_deref(), Some("lantern_field"));
    assert!(w.state.flags.is_set(&lantern_lit("lantern_field")));
    let d = w.actors.get(dummy).unwrap();
    assert_eq!((d.hp, d.pos), (100, [0.0, 0.0, 6.0]));
    assert!(matches!(
        life(&mut w)[..],
        [LifeEvent::Rested { lantern: 0, .. }]
    ));
}

#[test]
fn resting_needs_a_lantern_in_reach_and_footing() {
    let mut w = field();
    // The spawn is 3 m from the Lantern: out of reach (2 m).
    player_mut(&mut w).hp = 10;
    step(&mut w, ActionFrame::default(), vec![Command::Rest]);
    assert_eq!(player(&w).hp, 10);
    assert!(w.state.respawn_lantern.is_none() && w.state.resting.is_none());
    // In reach but in the air.
    stand_at(&mut w, [0.0, 0.0, -2.0]);
    player_mut(&mut w).motor.grounded = false;
    assert!(!super::lantern::try_rest(&mut w));
    // In reach and standing: rests.
    player_mut(&mut w).motor.grounded = true;
    assert!(super::lantern::try_rest(&mut w));
    assert_eq!(player(&w).hp, player(&w).max_hp);
}

#[test]
fn split_flasks_applies_only_while_resting() {
    let mut w = field();
    let split = |v| vec![Command::SplitFlasks { vigil: v }];
    let start = player(&w).inventory.flasks.vigil_max;
    let other = if start == 1 { 2 } else { 1 };
    // Away from any Lantern: ignored.
    step(&mut w, ActionFrame::default(), split(other));
    assert_eq!(player(&w).inventory.flasks.vigil_max, start);
    // Resting: applied, on the same tick as the rest.
    stand_at(&mut w, [0.0, 0.0, -2.0]);
    step(
        &mut w,
        ActionFrame::default(),
        vec![Command::Rest, Command::SplitFlasks { vigil: other }],
    );
    assert_eq!(player(&w).inventory.flasks.vigil_max, other);
    // Still resting on a later tick.
    step(&mut w, ActionFrame::default(), split(start));
    assert_eq!(player(&w).inventory.flasks.vigil_max, start);
    // Walking away closes the menu.
    step(
        &mut w,
        ActionFrame {
            move_x: 0,
            move_z: 100,
            buttons: 0,
        },
        Vec::new(),
    );
    assert!(w.state.resting.is_none());
    step(&mut w, ActionFrame::default(), split(other));
    assert_eq!(player(&w).inventory.flasks.vigil_max, start);
}

#[test]
fn death_drops_all_glimmer_at_the_last_safe_position_and_respawns() {
    let mut w = field();
    stand_at(&mut w, [2.0, 0.0, 1.0]);
    idle(&mut w, 1);
    let ground = player(&w).pos;
    // Airborne at death: the Remnant goes where the feet last touched ground.
    {
        let p = player_mut(&mut w);
        p.pos = [2.0, 30.0, 5.0];
        p.motor.grounded = false;
    }
    kill(&mut w);
    life(&mut w);
    idle(&mut w, 1);
    let r = w.state.death.remnant.clone().expect("a Remnant dropped");
    assert_eq!(r.glimmer, 500);
    assert_eq!(r.pos, ground);
    assert!(ground[1] < 0.1 && (ground[0] - 2.0).abs() < 0.1);
    assert_eq!(player(&w).inventory.glimmer, 0);
    assert!(player(&w).dead);
    let ev = life(&mut w);
    assert!(
        matches!(
            ev[..],
            [
                LifeEvent::Died { .. },
                LifeEvent::RemnantDropped { glimmer: 500, .. }
            ]
        ),
        "{ev:?}"
    );
    // The player stays dead for the delay, then respawns at the spawn (no Lantern yet).
    idle_delay(&mut w, -1);
    assert!(player(&w).dead);
    idle(&mut w, 1);
    let p = player(&w);
    assert!(!p.dead && p.hp == p.max_hp);
    assert_eq!(p.pos, [0.0, 0.0, 0.0]);
    assert!(matches!(life(&mut w)[..], [LifeEvent::Respawned { .. }]));
    assert_eq!(w.state.death.deaths, 1);
}

#[test]
fn respawn_is_at_the_last_rested_lantern_whole_and_with_enemies_back() {
    let mut w = field();
    stand_at(&mut w, [0.0, 0.0, -2.0]);
    step(&mut w, ActionFrame::default(), vec![Command::Rest]);
    {
        let p = player_mut(&mut w);
        p.inventory.flasks.vigil = 0;
        p.stamina = 0.0;
        p.pos = [10.0, 0.0, 10.0];
    }
    let dummy = w.state.enemies[1];
    w.actors.get_mut(dummy).unwrap().hp = 0;
    kill(&mut w);
    idle_delay(&mut w, 1);
    let p = player(&w);
    assert_eq!(p.pos, LANTERN);
    assert!(!p.dead && p.hp == p.max_hp && p.stamina == p.max_stamina);
    assert_eq!(p.inventory.flasks.vigil, p.inventory.flasks.vigil_max);
    assert!(p.motor.grounded);
    assert_eq!(w.actors.get(dummy).unwrap().hp, 100);
    // Respawning is not resting.
    assert!(w.state.resting.is_none());
}

#[test]
fn touching_the_remnant_recovers_its_glimmer() {
    let mut w = field();
    stand_at(&mut w, [4.0, 0.0, -6.0]);
    idle(&mut w, 1);
    kill(&mut w);
    idle_delay(&mut w, 1);
    life(&mut w);
    assert_eq!(player(&w).inventory.glimmer, 0);
    // Just outside the pickup radius: nothing. Inside it: recovered.
    let r = w.locomotion.death.remnant_pickup_radius_m;
    stand_at(&mut w, [4.0 + r + 0.1, 0.0, -6.0]);
    idle(&mut w, 1);
    assert!(w.state.death.remnant.is_some());
    stand_at(&mut w, [4.0 + r - 0.1, 0.0, -6.0]);
    idle(&mut w, 1);
    assert!(w.state.death.remnant.is_none());
    assert_eq!(player(&w).inventory.glimmer, 500);
    assert!(matches!(
        life(&mut w)[..],
        [LifeEvent::RemnantRecovered { glimmer: 500, .. }]
    ));
}

#[test]
fn dying_again_before_recovering_destroys_the_remnant() {
    let mut w = field();
    stand_at(&mut w, [4.0, 0.0, -6.0]);
    idle(&mut w, 1);
    kill(&mut w);
    idle_delay(&mut w, 1);
    life(&mut w);
    // Die again somewhere else, empty-handed.
    stand_at(&mut w, [-8.0, 0.0, -6.0]);
    idle(&mut w, 1);
    kill(&mut w);
    idle(&mut w, 1);
    assert!(
        w.state.death.remnant.is_none(),
        "no Remnant: nothing was held"
    );
    assert!(matches!(
        life(&mut w)[..],
        [
            LifeEvent::Died { .. },
            LifeEvent::RemnantLost { glimmer: 500, .. }
        ]
    ));
    idle_delay(&mut w, 1);
    stand_at(&mut w, [4.0, 0.0, -6.0]);
    idle(&mut w, 1);
    assert_eq!(player(&w).inventory.glimmer, 0);
    assert_eq!(w.state.death.deaths, 2);
}

#[test]
fn a_second_death_with_glimmer_replaces_the_remnant() {
    let mut w = field();
    stand_at(&mut w, [4.0, 0.0, -6.0]);
    idle(&mut w, 1);
    kill(&mut w);
    idle_delay(&mut w, 1);
    life(&mut w);
    player_mut(&mut w).inventory.glimmer = 70;
    stand_at(&mut w, [-8.0, 0.0, -6.0]);
    idle(&mut w, 1);
    kill(&mut w);
    idle(&mut w, 1);
    let r = w.state.death.remnant.clone().unwrap();
    assert_eq!((r.glimmer, r.pos), (70, [-8.0, 0.0, -6.0]));
}

#[test]
fn keep_glimmer_on_death_drops_nothing() {
    let mut w = field();
    w.state.keep_glimmer_on_death = true;
    stand_at(&mut w, [4.0, 0.0, -6.0]);
    idle(&mut w, 1);
    kill(&mut w);
    idle_delay(&mut w, 1);
    assert!(w.state.death.remnant.is_none());
    assert_eq!(player(&w).inventory.glimmer, 500);
}

#[test]
fn a_fall_to_the_void_kills_and_respawns_deterministically() {
    // Walk off the edge of the field the way the golden replays do.
    let run = || {
        let mut w = field();
        stand_at(&mut w, [0.0, 0.0, -31.0]);
        let go = ActionFrame {
            move_x: 0,
            move_z: -127,
            buttons: 0,
        };
        for _ in 0..40 {
            step(&mut w, go, Vec::new());
        }
        for _ in 0..200 {
            step(&mut w, ActionFrame::default(), Vec::new());
        }
        w
    };
    let (a, b) = (run(), run());
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.state.death.deaths, 1);
    let r = a.state.death.remnant.clone().expect("Remnant at the edge");
    assert!(r.pos[2] < -31.9 && r.pos[2] > -32.6, "{:?}", r.pos);
    assert!(!player(&a).dead);
}
