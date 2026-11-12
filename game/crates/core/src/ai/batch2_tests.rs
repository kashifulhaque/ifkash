//! Tests for the batch-2 enemies and their special rules (WP-036): water-only movement, reed
//! hiding, flight, the swarm explosion, Formula reflection, and shimmer.

use lf_content::Content;
use lf_content::moves::StatusKind;

use super::*;
use crate::combat::projectile;
use crate::events::{Event, HitResult, ProjectileEventKind};
use crate::input::{ActionFrame, TickInput};
use crate::snapshot::{ActorSnap, flags, write_snapshot};
use crate::tick::tick;
use lf_content::moves::ProjectileSpawn;

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

fn put_player(w: &mut World, pos: [f32; 3]) {
    let p = w.actors.get_mut(w.player).unwrap();
    p.pos = pos;
    p.motor.ground_y = pos[1];
    p.hit.prev = None;
}

fn brain(w: &World, id: Id) -> &AiState {
    w.actors.get(id).unwrap().ai.as_ref().unwrap()
}

fn snap(w: &World, id: Id) -> ActorSnap {
    let mut out = Vec::new();
    write_snapshot(w, &mut out);
    *out.iter()
        .find(|s| s.id == crate::snapshot::pack_id(id))
        .unwrap()
}

#[test]
fn the_drowned_stays_in_its_pool_and_its_grab_costs_about_a_fifth_of_the_players_hp() {
    let mut w = world("fight_drowned", 1);
    let drowned = enemy(&w, "drowned");
    // Next to the pool's near edge (z 3), within its reach.
    put_player(&mut w, [0.0, 0.0, 1.2]);
    let max = w.actors.get(w.player).unwrap().max_hp;
    let mut grab = None;
    for _ in 0..600 {
        for e in step(&mut w, 1) {
            if let Event::Hit(h) = e
                && h.attacker == drowned
                && h.move_num == w.moves.num_of("dr_grab").unwrap()
            {
                grab = Some(h);
            }
        }
        let a = w.actors.get(drowned).unwrap();
        // A move's root motion can carry it a step past the edge for one tick; the next
        // tick puts it back (0.6 m for the grab).
        assert!(
            (-6.0..=6.0).contains(&a.pos[0]) && (2.3..=9.1).contains(&a.pos[2]),
            "left the pool: {:?}",
            a.pos
        );
        if grab.is_some() {
            break;
        }
        // Keep the player alive and in place so the test sees the grab.
        let p = w.actors.get_mut(w.player).unwrap();
        p.hp = p.max_hp;
    }
    let g = grab.expect("it grabbed");
    assert_eq!(g.result, HitResult::Connected, "a grab can't be blocked");
    let pct = f64::from(g.damage) * 100.0 / f64::from(max);
    assert!((15.0..=25.0).contains(&pct), "{pct}% of {max}");
}

#[test]
fn the_drowned_is_put_back_in_the_water_and_cannot_chase_onto_land() {
    let mut w = world("fight_drowned", 2);
    let drowned = enemy(&w, "drowned");
    // Standing on land far behind the pool: the player is well within sight, but the pool ends
    // at x = 6, so the drowned can only come as close as the pool lets it.
    put_player(&mut w, [14.0, 0.0, 6.0]);
    step(&mut w, 400);
    let a = w.actors.get(drowned).unwrap();
    assert!(a.pos[0] <= 6.05, "{:?}", a.pos);
    // Shoved out of the water, it's put back at once.
    w.actors.get_mut(drowned).unwrap().pos = [12.0, 0.0, 6.0];
    step(&mut w, 2);
    assert!(w.actors.get(drowned).unwrap().pos[0] <= 6.05);
}

#[test]
fn the_reed_stalker_hides_lunges_then_goes_back_to_ground() {
    let mut w = world("fight_reed_stalker", 3);
    let rs = enemy(&w, "stalker");
    step(&mut w, 5);
    assert!(brain(&w, rs).hidden, "starts hidden in the reeds");
    assert!(snap(&w, rs).flags & flags::HIDDEN != 0);
    // The player walks up to 3.5 m of it.
    put_player(&mut w, [0.0, 0.0, 5.5]);
    let lunge = w.moves.num_of("rs_lunge").unwrap();
    let mut lunged = false;
    let mut revealed = false;
    let mut hidden_again = false;
    for _ in 0..500 {
        let events = step(&mut w, 1);
        let p = w.actors.get_mut(w.player).unwrap();
        p.hp = p.max_hp;
        let b = brain(&w, rs);
        if events
            .iter()
            .any(|e| matches!(e, Event::Hit(h) if h.attacker == rs && h.move_num == lunge))
        {
            lunged = true;
        }
        if lunged && !b.hidden {
            revealed = true;
        }
        if revealed && b.hidden {
            hidden_again = true;
            break;
        }
    }
    assert!(lunged, "it lunged");
    assert!(revealed, "the lunge revealed it");
    assert!(hidden_again, "it burrowed back into the reeds");
}

#[test]
fn the_cave_moth_hovers_dives_and_is_drawn_to_a_lit_lantern() {
    let mut w = world("fight_cave_moth", 4);
    let moth = enemy(&w, "moth");
    // Out of everyone's senses: it just hovers.
    put_player(&mut w, [20.0, 0.0, -20.0]);
    step(&mut w, 60);
    let y = w.actors.get(moth).unwrap().pos[1];
    assert!((y - 0.9).abs() < 0.05, "hover height {y}");
    // A lit Lantern within 14 m draws it.
    w.state
        .flags
        .raise(&crate::world_state::flags::lantern_lit("lantern_moth"));
    let before = w.actors.get(moth).unwrap().pos;
    step(&mut w, 200);
    let a = w.actors.get(moth).unwrap();
    let to_lantern = |p: [f32; 3]| perception::dist_xz(p, [9.0, 0.0, 10.0]);
    assert!(to_lantern(a.pos) < to_lantern(before) - 5.0, "{:?}", a.pos);
    assert!((a.pos[1] - 0.9).abs() < 0.05);

    // Then a fight: it dives. Watch its lowest point while it plays cm_dive.
    let mut w = world("fight_cave_moth", 5);
    let moth = enemy(&w, "moth");
    // 3 m from it: inside the dive's range from the start.
    put_player(&mut w, [0.0, 0.0, 7.0]);
    let dive = w.moves.num_of("cm_dive").unwrap();
    let mut lowest = f32::MAX;
    let mut dived = false;
    for _ in 0..600 {
        step(&mut w, 1);
        let p = w.actors.get_mut(w.player).unwrap();
        p.hp = p.max_hp;
        let a = w.actors.get(moth).unwrap();
        if a.moves.current.is_some_and(|c| c.num == dive) {
            dived = true;
            lowest = lowest.min(a.pos[1]);
        }
        if dived && a.moves.current.is_none() {
            break;
        }
    }
    assert!(dived, "it dived");
    assert!(lowest < 0.3, "the dive's low point {lowest}");
    step(&mut w, 60);
    assert!(
        (w.actors.get(moth).unwrap().pos[1] - 0.9).abs() < 0.1,
        "it climbs back"
    );
}

#[test]
fn a_lamp_mite_explodes_after_its_fuse() {
    let mut w = world("fight_lamp_mites", 6);
    // Keep one mite and move the others far away, so only it matters.
    let keep = enemy(&w, "mite_a");
    for n in ["mite_b", "mite_c", "mite_d", "mite_e", "mite_f"] {
        let id = enemy(&w, n);
        w.actors.get_mut(id).unwrap().dead = true;
    }
    put_player(&mut w, [0.0, 0.0, 6.0]);
    let burst = w.moves.num_of("lm_burst").unwrap();
    let hp0 = w.actors.get(w.player).unwrap().hp;
    let mut contact_at = None;
    let mut started = None;
    let mut hit = None;
    for i in 0..600u32 {
        let events = step(&mut w, 1);
        let a = w.actors.get(keep).unwrap();
        let near = perception::dist_xz(a.pos, w.actors.get(w.player).unwrap().pos) <= 1.9;
        if near && contact_at.is_none() {
            contact_at = Some(i);
        }
        if started.is_none() && a.moves.current.is_some_and(|c| c.num == burst) {
            started = Some(i);
        }
        for e in events {
            if let Event::Hit(h) = e
                && h.attacker == keep
            {
                hit = Some((i, h));
            }
        }
        if hit.is_some() && a.dead {
            break;
        }
    }
    let (contact, started) = (
        contact_at.expect("contact"),
        started.expect("it started the blast"),
    );
    // 48 ticks of contact, give or take the tick it took to notice.
    assert!(
        (46..=56).contains(&(started - contact)),
        "{contact} -> {started}"
    );
    let (hit_tick, h) = hit.expect("the blast landed");
    assert_eq!(h.move_num, burst);
    // 12 frames after the blast starts.
    assert!(
        (10..=14).contains(&(hit_tick - started)),
        "{started} -> {hit_tick}"
    );
    let a = w.actors.get(keep).unwrap();
    assert!(a.dead, "the mite goes with the blast");
    assert!(w.actors.get(w.player).unwrap().hp < hp0);
    assert!(
        h.damage > 20,
        "Fire comes from its element rating: {}",
        h.damage
    );
}

#[test]
fn breaking_contact_resets_a_mites_fuse() {
    let mut w = world("fight_lamp_mites", 7);
    let keep = enemy(&w, "mite_a");
    for n in ["mite_b", "mite_c", "mite_d", "mite_e", "mite_f"] {
        let id = enemy(&w, n);
        w.actors.get_mut(id).unwrap().dead = true;
    }
    put_player(&mut w, [0.0, 0.0, 6.0]);
    let mut peak = 0;
    for _ in 0..200 {
        step(&mut w, 1);
        peak = peak.max(brain(&w, keep).contact_ticks);
        if brain(&w, keep).contact_ticks > 20 {
            break;
        }
    }
    assert!(peak > 20);
    // The player steps well away: the count falls to 0.
    put_player(&mut w, [0.0, 0.0, -6.0]);
    w.actors.get_mut(keep).unwrap().pos = [0.0, 0.0, 6.0];
    step(&mut w, 2);
    assert_eq!(brain(&w, keep).contact_ticks, 0);
}

fn spawn_bolt(w: &mut World, shooter: Id) {
    projectile::spawn(
        w,
        shooter,
        0,
        &ProjectileSpawn {
            frame: 0,
            projectile: "proj_seeker_bolt".to_owned(),
            offset: [0.0, 1.0, 0.6],
            yaw_deg: 0.0,
            pitch_deg: 0.0,
        },
    )
    .unwrap();
}

#[test]
fn a_shard_golem_reflects_a_formula_until_a_blunt_hit_cracks_it() {
    let mut w = world("fight_shard_golem", 8);
    let golem = enemy(&w, "golem");
    // Keep the golem from acting and from seeing: only the bolt matters.
    w.actors
        .get_mut(golem)
        .unwrap()
        .ai
        .as_mut()
        .unwrap()
        .awareness = Awareness::Idle;
    let max = w.actors.get(golem).unwrap().max_hp;
    let hp0 = w.actors.get(w.player).unwrap().hp;
    let player = w.player;
    spawn_bolt(&mut w, player);
    let mut events = Vec::new();
    for _ in 0..120 {
        w.actors
            .get_mut(golem)
            .unwrap()
            .ai
            .as_mut()
            .unwrap()
            .awareness = Awareness::Idle;
        events.extend(step(&mut w, 1));
    }
    assert_eq!(
        w.actors.get(golem).unwrap().hp,
        max,
        "no damage to the golem"
    );
    let back = events.iter().find_map(|e| match e {
        Event::Hit(h) if h.target == w.player => Some(h),
        _ => None,
    });
    let back = back.expect("the bolt came back at its shooter");
    assert_eq!(back.attacker, golem, "the reflector owns the bolt now");
    assert!(w.actors.get(w.player).unwrap().hp < hp0);

    // A Blunt hit cracks the crystals: the next Formula lands on the golem.
    let mut w = world("fight_shard_golem", 8);
    let golem = enemy(&w, "golem");
    super::special::reflect::note_blunt(&mut w, golem);
    assert!(brain(&w, golem).cracked > 0);
    let player = w.player;
    spawn_bolt(&mut w, player);
    let mut events = Vec::new();
    for _ in 0..60 {
        w.actors
            .get_mut(golem)
            .unwrap()
            .ai
            .as_mut()
            .unwrap()
            .awareness = Awareness::Idle;
        events.extend(step(&mut w, 1));
    }
    assert!(
        w.actors.get(golem).unwrap().hp < max,
        "a cracked golem takes it"
    );
    // The crack wears off.
    step(&mut w, 150);
    assert_eq!(brain(&w, golem).cracked, 0);
}

#[test]
fn an_arrow_is_not_a_formula_and_a_golem_takes_it() {
    let mut w = world("fight_shard_golem", 9);
    let golem = enemy(&w, "golem");
    let max = w.actors.get(golem).unwrap().max_hp;
    let player = w.player;
    projectile::spawn(
        &mut w,
        player,
        0,
        &ProjectileSpawn {
            frame: 0,
            projectile: "proj_arrow".to_owned(),
            offset: [0.0, 1.0, 0.6],
            yaw_deg: 0.0,
            pitch_deg: 0.0,
        },
    )
    .unwrap();
    step(&mut w, 40);
    assert!(w.actors.get(golem).unwrap().hp < max);
}

#[test]
fn the_acolytes_bolt_glows_for_30_ticks_and_hurts() {
    let mut w = world("fight_acolyte", 10);
    let ac = enemy(&w, "acolyte");
    let hp0 = w.actors.get(w.player).unwrap().hp;
    let cast = w.moves.num_of("ac_cast").unwrap();
    let mut cast_start = None;
    let mut spawned = None;
    let mut hit = None;
    for i in 0..400u32 {
        let events = step(&mut w, 1);
        if cast_start.is_none()
            && w.actors
                .get(ac)
                .unwrap()
                .moves
                .current
                .is_some_and(|c| c.num == cast)
        {
            cast_start = Some(i);
        }
        for e in events {
            match e {
                Event::Projectile(p)
                    if p.shooter == ac && p.kind == ProjectileEventKind::Spawned =>
                {
                    spawned.get_or_insert(i);
                }
                Event::Hit(h) if h.attacker == ac && h.target == w.player => {
                    hit.get_or_insert(h);
                }
                _ => {}
            }
        }
        if hit.is_some() {
            break;
        }
    }
    let (c, s) = (cast_start.unwrap(), spawned.unwrap());
    assert!((29..=31).contains(&(s - c)), "{c} -> {s}");
    let h = hit.expect("the bolt landed");
    assert!(h.damage > 20, "{}", h.damage);
    assert!(w.actors.get(w.player).unwrap().hp < hp0);
}

#[test]
fn marsh_crawlers_bite_with_poison() {
    let mut w = world("fight_marsh_crawlers", 11);
    let mut bit = false;
    for _ in 0..500 {
        for e in step(&mut w, 1) {
            if let Event::Hit(h) = e
                && h.target == w.player
            {
                bit = true;
            }
        }
        let p = w.actors.get_mut(w.player).unwrap();
        p.hp = p.max_hp;
    }
    assert!(bit);
    let p = w.actors.get(w.player).unwrap();
    let i = crate::combat::status::index(StatusKind::Poison);
    assert!(p.status.meter_milli[i] > 0 || p.status.active[i] > 0);
}

#[test]
fn a_glasswing_is_cloaked_while_still_and_visible_while_it_moves() {
    let mut w = world("fight_glasswing", 12);
    let gw = enemy(&w, "stalker");
    put_player(&mut w, [20.0, 0.0, -20.0]);
    step(&mut w, 5);
    assert!(snap(&w, gw).flags & flags::CLOAKED != 0, "still: cloaked");
    // It notices the player and moves: no longer cloaked.
    put_player(&mut w, [0.0, 0.0, 0.0]);
    let mut moving = false;
    for _ in 0..200 {
        step(&mut w, 1);
        let s = snap(&w, gw);
        let a = w.actors.get(gw).unwrap();
        if a.vel[0].abs() + a.vel[2].abs() > 0.5 {
            assert_eq!(s.flags & flags::CLOAKED, 0, "moving: shimmering");
            moving = true;
            break;
        }
    }
    assert!(moving);
    // A different enemy is never cloaked.
    let hw = world("fight_husk", 12);
    let husk = hw.state.ai_enemies[0];
    assert_eq!(snap(&hw, husk).flags & flags::CLOAKED, 0);
}

#[test]
fn a_hovering_cave_moth_can_be_hit_by_a_sword() {
    let mut w = world("fight_cave_moth", 13);
    let moth = enemy(&w, "moth");
    // The moth hovers 1 m in front of the player, who swings once. Pin the moth's position
    // each tick so only the reach matters.
    w.actors.get_mut(moth).unwrap().pos = [0.0, 0.9, 1.0];
    let mut hit = false;
    for i in 0..60 {
        let a = w.actors.get_mut(moth).unwrap();
        a.pos = [0.0, 0.9, 1.0];
        a.hit.prev = None;
        let frame = ActionFrame {
            buttons: if i == 1 {
                crate::input::buttons::LIGHT
            } else {
                0
            },
            ..ActionFrame::default()
        };
        tick(
            &mut w,
            &TickInput {
                frame,
                commands: Vec::new(),
            },
        );
        hit |= w
            .events
            .drain()
            .iter()
            .any(|e| matches!(e, Event::Hit(h) if h.target == moth));
    }
    assert!(hit, "a light swing reaches a moth hovering at 0.9 m");
}
