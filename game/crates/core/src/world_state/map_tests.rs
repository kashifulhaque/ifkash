//! Tests for the map and fast travel (WP-069): charting, markers, travel to another area, and
//! that a trip replays to the same hashes.

use lf_content::Content;

use super::flags;
use super::map::*;
use crate::events::{Event, MechanicEvent};
use crate::input::{ActionFrame, Command, TickInput};
use crate::save;
use crate::world::World;

fn content() -> Content {
    Content::load_embedded().expect("embedded content")
}

fn world(content: &Content, area: &str) -> World {
    World::new(5, content, area).expect("area exists")
}

fn input(commands: Vec<Command>) -> TickInput {
    TickInput {
        frame: ActionFrame::default(),
        commands,
    }
}

fn put_player(w: &mut World, pos: [f32; 3]) {
    let id = w.player;
    let p = w.actors.get_mut(id).unwrap();
    p.pos = pos;
    p.motor.ground_y = pos[1];
    p.motor.fall_peak_y = pos[1];
}

fn place_of(c: &Content, id: &str) -> u16 {
    places(c).iter().position(|p| p.lantern.id == id).unwrap() as u16
}

#[test]
fn places_cover_every_area_in_name_order() {
    let c = content();
    let ps = places(&c);
    // More areas arrive with their work packages; the places always include several areas.
    assert!(ps.len() >= 10, "the built areas' Lanterns are placed");
    assert!(ps.iter().any(|p| p.area == "verge") && ps.iter().any(|p| p.area == "mirrormere"));
    let areas: Vec<&str> = ps.iter().map(|p| p.area.as_str()).collect();
    let mut sorted = areas.clone();
    sorted.sort();
    assert_eq!(areas, sorted);
    let mut ids: Vec<&str> = ps.iter().map(|p| p.lantern.id.as_str()).collect();
    ids.sort();
    let n = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), n, "Lantern IDs are unique across areas");
}

#[test]
fn walking_charts_the_cells_once() {
    let c = content();
    let mut w = world(&c, "verge");
    put_player(&mut w, [-40.0, 0.0, -60.0]);
    crate::tick::tick(&mut w, &input(vec![]));
    let area = w.physics.scenario.clone();
    let (ix, iz) = cell_of(-40.0, -60.0);
    assert!(is_visited(&w.state.flags, &area, ix, iz));
    assert!(!is_visited(&w.state.flags, &area, ix + 1, iz));
    put_player(&mut w, [-20.0, 0.0, -60.0]);
    crate::tick::tick(&mut w, &input(vec![]));
    let cells = visited_cells(&w.state.flags, &area);
    assert!(cells.contains(&(ix, iz)) && cells.contains(&cell_of(-20.0, -60.0)));
    let unique: std::collections::BTreeSet<_> = cells.iter().collect();
    assert_eq!(cells.len(), unique.len());
}

#[test]
fn cells_round_trip_through_the_row_flags() {
    // Negative cells and cells past the first row chunk both decode.
    let mut flags = flags::Flags::default();
    flags.set("map.x.row.-3.-1", 1 << 31);
    flags.set("map.x.row.-3.2", 1 << 4);
    assert_eq!(visited_cells(&flags, "x"), vec![(-1, -3), (68, -3)]);
    assert!(is_visited(&flags, "x", -1, -3) && is_visited(&flags, "x", 68, -3));
    assert!(!is_visited(&flags, "x", 0, -3));
}

#[test]
fn flat_scenarios_without_lanterns_are_not_charted() {
    let c = content();
    let mut w = world(&c, "m0_flat");
    let before = w.state.flags.clone();
    crate::tick::tick(&mut w, &input(vec![]));
    assert_eq!(w.state.flags, before);
}

#[test]
fn markers_toggle_and_stop_at_twenty() {
    let c = content();
    let mut w = world(&c, "verge");
    crate::tick::tick(&mut w, &input(vec![Command::MapMarker { x: 100, z: -250 }]));
    let area = w.physics.scenario.clone();
    assert_eq!(markers(&w.state.flags, &area), vec![[10.0, -25.0]]);
    // A press within 3 m removes it.
    crate::tick::tick(&mut w, &input(vec![Command::MapMarker { x: 110, z: -240 }]));
    assert!(markers(&w.state.flags, &area).is_empty());
    for i in 0..25 {
        crate::tick::tick(
            &mut w,
            &input(vec![Command::MapMarker { x: i * 100, z: 0 }]),
        );
    }
    assert_eq!(markers(&w.state.flags, &area).len(), MAX_MARKERS);
}

/// A character that has lit one Lantern in the Verge and one in Mirrormere, as a save.
fn two_areas_lit(c: &Content) -> World {
    let mut w = world(c, "verge");
    w.state.flags.raise(&flags::lantern_lit("lantern_l02"));
    w.state.flags.raise(&flags::lantern_lit("lantern_l06"));
    let data = save::capture(&w, 5, c);
    save::restore(&data, c).unwrap()
}

fn rest_at_l02(w: &mut World) {
    put_player(w, [-40.0, 0.0, -60.0]);
    crate::tick::tick(w, &input(vec![Command::Rest]));
    assert_eq!(w.state.resting.as_deref(), Some("lantern_l02"));
}

#[test]
fn travel_to_another_area_swaps_the_world_and_keeps_the_character() {
    let c = content();
    let mut w = two_areas_lit(&c);
    rest_at_l02(&mut w);
    w.actors.get_mut(w.player).unwrap().inventory.glimmer = 321;
    let tick_before = w.tick;
    let go = Command::TravelFar {
        place: place_of(&c, "lantern_l06"),
    };
    tick_world(&mut w, &input(vec![go]), &c);
    assert_eq!(w.physics.scenario, "mirrormere");
    assert_eq!(w.tick, tick_before + 1);
    assert_eq!(w.state.resting.as_deref(), Some("lantern_l06"));
    assert_eq!(w.state.respawn_lantern.as_deref(), Some("lantern_l06"));
    assert!(w.state.flags.is_set(&flags::lantern_lit("lantern_l02")));
    assert!(!w.state.flags.is_set(TRAVEL_FLAG));
    let p = w.actors.get(w.player).unwrap();
    assert_eq!(p.inventory.glimmer, 321);
    let l = w.layout.lantern("lantern_l06").unwrap().1;
    assert_eq!(p.pos, l.pos);
    // The area-changed event reached the host's queue.
    let events = w.events.drain();
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Mechanic(m) if m.kind == MechanicEvent::AREA_CHANGED
    )));
}

#[test]
fn travel_is_refused_unlit_here_or_with_a_remnant_and_needs_a_rest() {
    let c = content();
    let l09 = place_of(&c, "lantern_l09"); // never lit
    let l06 = place_of(&c, "lantern_l06");
    let mut w = two_areas_lit(&c);
    // Not resting: the command does nothing.
    tick_world(&mut w, &input(vec![Command::TravelFar { place: l06 }]), &c);
    assert_eq!(w.physics.scenario, "verge");
    assert!(!w.state.flags.is_set(TRAVEL_FLAG));
    rest_at_l02(&mut w);
    tick_world(&mut w, &input(vec![Command::TravelFar { place: l09 }]), &c);
    assert_eq!(w.physics.scenario, "verge", "an unlit Lantern");
    let here = place_of(&c, "lantern_l02");
    tick_world(&mut w, &input(vec![Command::TravelFar { place: here }]), &c);
    assert_eq!(w.physics.scenario, "verge", "this area");
    tick_world(
        &mut w,
        &input(vec![Command::TravelFar { place: 60_000 }]),
        &c,
    );
    assert_eq!(w.physics.scenario, "verge", "no such place");
    assert!(!w.state.flags.is_set(TRAVEL_FLAG));
    w.state.death.remnant = Some(super::Remnant {
        pos: [0.0; 3],
        glimmer: 9,
    });
    tick_world(&mut w, &input(vec![Command::TravelFar { place: l06 }]), &c);
    assert_eq!(w.physics.scenario, "verge", "a Remnant lies behind");
}

/// The script: rest, mark, travel, travel back, walk. Returns the hash after every
/// tick and the final world.
fn run_script(c: &Content) -> (Vec<u64>, World) {
    let mut w = two_areas_lit(c);
    let mut hashes = Vec::new();
    let idle = ActionFrame::default();
    let walk = ActionFrame {
        move_x: 0,
        move_z: 100,
        buttons: 0,
    };
    let mut script: Vec<(Vec<Command>, ActionFrame)> = vec![
        (vec![Command::Rest], idle),
        (vec![Command::MapMarker { x: -400, z: -600 }], idle),
        (
            vec![Command::TravelFar {
                place: place_of(c, "lantern_l06"),
            }],
            idle,
        ),
    ];
    // A tick in Mirrormere charts the Lantern's cell, then the trip back.
    script.push((vec![], idle));
    script.push((
        vec![Command::TravelFar {
            place: place_of(c, "lantern_l02"),
        }],
        idle,
    ));
    script.extend((0..90).map(|_| (vec![], walk)));
    put_player(&mut w, [-40.0, 0.0, -60.0]);
    for (commands, frame) in script {
        tick_world(&mut w, &TickInput { frame, commands }, c);
        hashes.push(w.state_hash());
    }
    (hashes, w)
}

#[test]
fn a_trip_across_areas_replays_to_the_same_hashes_and_survives_a_save() {
    let c = content();
    let (a, wa) = run_script(&c);
    let (b, wb) = run_script(&c);
    assert_eq!(a, b);
    assert_eq!(wb.state_hash(), *a.last().unwrap());
    assert_eq!(
        wa.physics.scenario, "verge",
        "the trip back landed in the Verge"
    );
    assert!(!visited_cells(&wa.state.flags, "mirrormere").is_empty());
    assert_eq!(markers(&wa.state.flags, "verge").len(), 1);
    let bytes = save::save(&wa, 5, &c);
    let loaded = save::load(&bytes, &c).unwrap();
    assert_eq!(
        save::persistent_hash(&loaded, 5, &c),
        save::persistent_hash(&wa, 5, &c)
    );
}

#[test]
fn the_view_lists_every_place_and_the_charted_cells() {
    let c = content();
    let mut w = two_areas_lit(&c);
    rest_at_l02(&mut w);
    let ps = places(&c);
    let json = view_json(&w, &ps, Some([-200.0, -200.0, 200.0, 200.0]));
    assert!(json.starts_with("{\"area\":\"verge\""));
    assert!(json.contains("\"resting\":\"lantern_l02\""));
    assert!(json.contains("\"id\":\"lantern_l06\"") && json.contains("\"lit\":true"));
    assert!(json.contains("\"cells\":[["));
}
