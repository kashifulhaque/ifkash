//! Character controller behavior on the `test_field` scenario.

use lf_content::Content;

use super::controller::{fall_damage, jump_speed};
use crate::input::{ActionFrame, TickInput, buttons};
use crate::tick::tick;
use crate::world::World;

fn content() -> Content {
    Content::load_embedded().expect("embedded content")
}

fn field() -> World {
    World::new(7, &content(), "test_field").expect("test_field exists")
}

fn frame(move_x: i8, move_z: i8, buttons: u16) -> TickInput {
    TickInput {
        frame: ActionFrame {
            move_x,
            move_z,
            buttons,
        },
        commands: Vec::new(),
    }
}

fn run(w: &mut World, input: &TickInput, ticks: u32) {
    for _ in 0..ticks {
        tick(w, input);
    }
}

fn player(w: &World) -> &super::Actor {
    w.actors.get(w.player).unwrap()
}

/// Puts the player at `pos` in the air and lets it settle for a second.
fn place(w: &mut World, pos: [f32; 3]) {
    let id = w.player;
    let a = w.actors.get_mut(id).unwrap();
    a.pos = pos;
    a.motor.grounded = false;
    a.motor.fall_peak_y = pos[1];
    run(w, &frame(0, 0, 0), 30);
}

fn near(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

#[test]
fn spawns_grounded_with_full_stats() {
    let mut w = field();
    run(&mut w, &frame(0, 0, 0), 5);
    let p = player(&w);
    assert!(p.motor.grounded);
    assert!(p.max_hp > 0 && p.hp == p.max_hp);
    assert!(p.max_stamina > 0.0 && p.stamina == p.max_stamina);
    assert!(near(p.pos[1], 0.0, 0.05), "{:?}", p.pos);
}

#[test]
fn walk_and_run_speeds() {
    let t = content().locomotion.unwrap();
    for (stick, speed) in [(30, t.walk_speed), (127, t.run_speed)] {
        let mut w = field();
        place(&mut w, [-10.0, 0.0, -20.0]);
        let z0 = player(&w).pos[2];
        run(&mut w, &frame(0, stick, 0), 30);
        let dz = player(&w).pos[2] - z0;
        assert!(near(dz, speed, 0.05), "stick {stick}: moved {dz} m in 1 s");
    }
}

#[test]
fn sprint_starts_after_the_hold_delay_and_drains_stamina() {
    let t = content().locomotion.unwrap();
    let mut w = field();
    place(&mut w, [-10.0, 0.0, -25.0]);
    let s0 = player(&w).stamina;
    let held = frame(0, 127, buttons::DODGE);
    run(&mut w, &held, u32::from(t.sprint_delay_ticks));
    assert!(!player(&w).motor.sprinting);
    assert_eq!(player(&w).stamina, s0);
    let z0 = player(&w).pos[2];
    run(&mut w, &held, 30);
    let p = player(&w);
    assert!(p.motor.sprinting);
    assert!(
        near(p.pos[2] - z0, t.sprint_speed, 0.25),
        "{}",
        p.pos[2] - z0
    );
    assert!(near(s0 - p.stamina, 30.0 * t.sprint_stamina_per_tick, 1e-3));
}

#[test]
fn jump_height_and_running_distance_match_the_spec() {
    let t = content().locomotion.unwrap();
    // Standing jump: peak height.
    let mut w = field();
    place(&mut w, [-10.0, 0.0, -20.0]);
    let y0 = player(&w).pos[1];
    tick(&mut w, &frame(0, 0, buttons::JUMP));
    let p = player(&w);
    assert_eq!(p.stamina, p.max_stamina - t.jump_stamina);
    let mut peak = y0;
    for _ in 0..60 {
        tick(&mut w, &frame(0, 0, 0));
        peak = peak.max(player(&w).pos[1]);
    }
    assert!(near(peak - y0, t.jump_height, 0.08), "peak {}", peak - y0);
    assert!(player(&w).motor.grounded);

    // Running jump: horizontal distance between takeoff and landing.
    let mut w = field();
    place(&mut w, [-10.0, 0.0, -28.0]);
    run(&mut w, &frame(0, 127, 0), 10);
    tick(&mut w, &frame(0, 127, buttons::JUMP));
    let mut takeoff = None;
    let mut landing = None;
    for _ in 0..60 {
        let before = player(&w).pos[2];
        let was_grounded = player(&w).motor.grounded;
        tick(&mut w, &frame(0, 127, 0));
        let p = player(&w);
        if was_grounded && !p.motor.grounded && takeoff.is_none() {
            takeoff = Some(before);
        }
        if !was_grounded && p.motor.grounded && takeoff.is_some() {
            landing = Some(p.pos[2]);
            break;
        }
    }
    let dist = landing.unwrap() - takeoff.unwrap();
    assert!(near(dist, 3.5, 0.25), "running jump carried {dist} m");
    assert!(jump_speed(&t) > 0.0);
}

#[test]
fn walks_up_the_slope_but_not_the_steep_ridge() {
    // The ramp lane (26.6 degrees) reaches the 9 m landing.
    let mut w = field();
    run(&mut w, &frame(0, 127, 0), 30 * 8);
    let p = player(&w);
    assert!(near(p.pos[1], 9.0, 0.1), "{:?}", p.pos);
    assert!(p.pos[2] > 28.0 && p.pos[2] < 30.0, "{:?}", p.pos);

    // The 63 degree ridge stops the player near its foot.
    let mut w = field();
    place(&mut w, [24.0, 0.0, -12.0]);
    run(&mut w, &frame(0, 127, 0), 30 * 4);
    let p = player(&w);
    assert!(p.pos[1] < 1.0, "climbed the ridge: {:?}", p.pos);
    assert!(p.pos[2] < -6.0, "{:?}", p.pos);
}

#[test]
fn steps_up_stairs_but_not_the_ledge() {
    // Four 0.3 m steps, under the autostep height: walks up to 1.2 m.
    let mut w = field();
    place(&mut w, [-23.0, 0.0, 0.0]);
    run(&mut w, &frame(0, 60, 0), 30 * 2);
    let p = player(&w);
    assert!(near(p.pos[1], 1.2, 0.05), "{:?}", p.pos);

    // The 1 m ledge needs a jump.
    let mut w = field();
    run(&mut w, &frame(0, 127, 0), 30 * 9);
    let p = player(&w);
    assert!(near(p.pos[1], 9.0, 0.1), "{:?}", p.pos);
    tick(&mut w, &frame(0, 127, buttons::JUMP));
    run(&mut w, &frame(0, 127, 0), 30);
    let p = player(&w);
    assert!(near(p.pos[1], 10.0, 0.1), "{:?}", p.pos);
    assert!(p.pos[2] > 30.0, "{:?}", p.pos);
}

#[test]
fn snaps_to_ground_walking_down_the_ramp() {
    let mut w = field();
    place(&mut w, [0.0, 9.5, 29.0]);
    for _ in 0..30 * 5 {
        tick(&mut w, &frame(0, -127, 0));
        assert!(
            player(&w).motor.grounded,
            "left the ground at {:?}",
            player(&w).pos
        );
    }
    assert!(player(&w).pos[2] < 10.0);
}

#[test]
fn ten_meter_fall_applies_spec_damage() {
    let t = content().locomotion.unwrap();
    let mut w = field();
    place(&mut w, [0.0, 10.5, 38.0]);
    assert!(near(player(&w).pos[1], 10.0, 0.05), "{:?}", player(&w).pos);
    let max_hp = player(&w).max_hp;
    run(&mut w, &frame(0, 60, 0), 30 * 3);
    let p = player(&w);
    assert!(near(p.pos[1], 0.0, 0.05), "{:?}", p.pos);
    assert!(
        near(p.motor.last_fall_m, 10.0, 0.01),
        "{}",
        p.motor.last_fall_m
    );
    // 4% of max HP per meter above 8 m: 8% for 10 m.
    let expected = (max_hp as f32 * 0.08 + 0.5) as u32;
    assert_eq!(max_hp - p.hp, expected);
    assert_eq!(fall_damage(10.0, max_hp, &t), expected);
    assert!(!p.dead);
}

#[test]
fn fall_damage_thresholds() {
    let t = content().locomotion.unwrap();
    assert_eq!(fall_damage(8.0, 1000, &t), 0);
    assert_eq!(fall_damage(9.0, 1000, &t), 40);
    assert_eq!(fall_damage(10.0, 1000, &t), 80);
    assert_eq!(fall_damage(20.0, 1000, &t), 480);
    assert_eq!(fall_damage(20.5, 1000, &t), 1000);
}

#[test]
fn falling_out_of_the_world_kills() {
    let mut w = field();
    place(&mut w, [31.5, 0.0, -31.5]);
    // The player respawns after a delay (`world_state::death`), so catch the death itself.
    let f = frame(127, -127, 0);
    let died = (0..30 * 6).any(|_| {
        tick(&mut w, &f);
        player(&w).dead
    });
    let p = player(&w);
    assert!(died && p.hp == 0, "{:?}", p.pos);
}

#[test]
fn world_clones_and_serializes_in_lockstep() {
    let mut a = field();
    run(&mut a, &frame(10, 127, buttons::DODGE), 90);
    let bytes = postcard::to_allocvec(&a).unwrap();
    let mut b: World = postcard::from_bytes(&bytes).unwrap();
    let mut c = a.clone();
    assert_eq!(a.state_hash(), b.state_hash());
    for i in 0..300u32 {
        let f = frame(
            (i % 50) as i8,
            127,
            if i % 40 == 0 { buttons::JUMP } else { 0 },
        );
        tick(&mut a, &f);
        tick(&mut b, &f);
        tick(&mut c, &f);
    }
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.state_hash(), c.state_hash());
}
