//! The map and fast travel (WP-069). See docs/game/06-progression-and-systems.md, "Map".
//!
//! # What the core keeps
//!
//! All of it is world flags, so a save keeps it and the state hash covers it:
//!
//! - **Visited cells.** The player's 16-meter cell is recorded as a bit in
//!   `map.<area>.row.<iz>.<chunk>`: one flag per row of 32 cells. The renderer draws the
//!   cells and a halo around them; the core records only where the player stood. Only
//!   areas with a Lantern are charted, so the combat test scenarios hash as before.
//! - **Markers.** Up to [`MAX_MARKERS`] (all areas together), each one flag,
//!   `map.<area>.marker.<slot>`, the position packed into the value.
//! - **A pending trip.** `map.travel` holds the destination place plus one while a trip to
//!   another area waits for the host.
//!
//! # Travel to another area
//!
//! The world can't swap its own level: building one needs the content, which the tick
//! doesn't see. So `Command::TravelFar` (while resting) only raises `map.travel`, and the
//! host finishes the trip after the tick with [`tick_world`] (or [`finish_tick`]). That
//! builds a fresh world for the destination's area from the persistent state
//! (`save::capture` and `save::restore`), keeps the tick counter and the attuned spells,
//! and rests at the destination Lantern, so the Lantern menu stays open there. A replay,
//! the browser host, and a test all go through the same function, so a trip replays to the
//! same hashes. Travel inside one area stays `Command::Travel`.
//!
//! A trip is refused (and the flag cleared) if the Lantern isn't lit, is in this area, or
//! the player has a Remnant waiting: the Remnant has no area of its own, so leaving it
//! behind would lose it.
//!
//! The 16-meter cell and the marker cap are from the Map section of the spec.

use lf_content::Content;
use lf_content::scenario::Lantern;

use crate::events::{Event, MechanicEvent};
use crate::input::{Command, TickInput};
use crate::world::World;

use super::{flags, lantern};

/// Side of a map cell, in meters (spec: 16-meter map cells).
pub const CELL_M: f32 = 16.0;
/// Most player-placed markers, over all areas (spec: up to 20).
pub const MAX_MARKERS: usize = 20;
/// A marker within this many decimeters of a placement is removed instead of adding another.
const MARKER_REMOVE_DM: i32 = 30;
/// Offset that keeps a packed marker coordinate positive.
const MARKER_BIAS: i32 = 16_384;
/// Cells per row flag.
const CHUNK: i32 = 32;
/// The flag that holds a pending trip (the place index plus one).
pub const TRAVEL_FLAG: &str = "map.travel";

/// A Lantern anywhere in the world: a place one can travel to.
#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    /// The area (the level's directory name, also its scenario name).
    pub area: String,
    pub lantern: Lantern,
}

/// Every Lantern of every compiled area, areas in name order, Lanterns in the order of the
/// level. The index of a place is what `Command::TravelFar` carries.
pub fn places(content: &Content) -> Vec<Place> {
    let mut out = Vec::new();
    for (key, scenario) in &content.scenarios {
        let Some(area) = key.strip_prefix("levels/") else {
            continue;
        };
        for l in &scenario.lanterns {
            out.push(Place {
                area: area.to_owned(),
                lantern: l.clone(),
            });
        }
    }
    out
}

/// The map cell a point on the ground is in.
pub fn cell_of(x: f32, z: f32) -> (i32, i32) {
    ((x / CELL_M).floor() as i32, (z / CELL_M).floor() as i32)
}

fn row_flag(area: &str, iz: i32, chunk: i32) -> String {
    format!("map.{area}.row.{iz}.{chunk}")
}

fn marker_flag(area: &str, slot: u32) -> String {
    format!("map.{area}.marker.{slot}")
}

/// Whether the player has stood in cell `(ix, iz)` of `area`.
pub fn is_visited(flags: &flags::Flags, area: &str, ix: i32, iz: i32) -> bool {
    let bit = ix.rem_euclid(CHUNK) as u32;
    flags.get(&row_flag(area, iz, ix.div_euclid(CHUNK))) >> bit & 1 == 1
}

/// Every visited cell of `area`, in flag order.
pub fn visited_cells(flags: &flags::Flags, area: &str) -> Vec<(i32, i32)> {
    let prefix = format!("map.{area}.row.");
    let mut out = Vec::new();
    for (key, value) in flags.iter() {
        let Some(rest) = key.strip_prefix(&prefix) else {
            continue;
        };
        let mut parts = rest.split('.');
        let (Some(iz), Some(chunk)) = (
            parts.next().and_then(|s| s.parse::<i32>().ok()),
            parts.next().and_then(|s| s.parse::<i32>().ok()),
        ) else {
            continue;
        };
        for bit in 0..CHUNK {
            if value >> bit & 1 == 1 {
                out.push((chunk * CHUNK + bit, iz));
            }
        }
    }
    out
}

/// The markers placed in `area`, in meters.
pub fn markers(flags: &flags::Flags, area: &str) -> Vec<[f32; 2]> {
    let prefix = format!("map.{area}.marker.");
    flags
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .map(|(_, v)| {
            let x = (v & 0xFFFF) as i32 - MARKER_BIAS;
            let z = (v >> 16) as i32 - MARKER_BIAS;
            [x as f32 / 10.0, z as f32 / 10.0]
        })
        .collect()
}

fn marker_total(flags: &flags::Flags) -> usize {
    flags
        .iter()
        .filter(|(k, _)| k.starts_with("map.") && k.contains(".marker."))
        .count()
}

/// Applies the tick's map commands and charts the player's cell. Called from tick system 1.
pub fn run(world: &mut World, input: &TickInput) {
    for c in &input.commands {
        match *c {
            Command::TravelFar { place } if world.state.resting.is_some() => {
                world
                    .state
                    .flags
                    .set(TRAVEL_FLAG, u32::from(place).saturating_add(1));
            }
            Command::MapMarker { x, z } => toggle_marker(world, x, z),
            _ => {}
        }
    }
    visit(world);
}

/// Records the player's cell, if the area is charted and they are alive.
fn visit(world: &mut World) {
    if world.layout.lanterns.is_empty() {
        return;
    }
    let Some(p) = world.actors.get(world.player).filter(|p| !p.dead) else {
        return;
    };
    let (ix, iz) = cell_of(p.pos[0], p.pos[2]);
    let area = world.physics.scenario.as_str();
    let key = row_flag(area, iz, ix.div_euclid(CHUNK));
    let bits = world.state.flags.get(&key);
    let bit = 1u32 << ix.rem_euclid(CHUNK);
    if bits & bit == 0 {
        world.state.flags.set(&key, bits | bit);
    }
}

/// Places a marker at `(x, z)` (in decimeters), or removes the one close to it.
fn toggle_marker(world: &mut World, x: i16, z: i16) {
    let (x, z) = (
        i32::from(x).clamp(-MARKER_BIAS + 1, MARKER_BIAS - 1),
        i32::from(z).clamp(-MARKER_BIAS + 1, MARKER_BIAS - 1),
    );
    let area = world.physics.scenario.clone();
    let prefix = format!("map.{area}.marker.");
    let limit = i64::from(MARKER_REMOVE_DM) * i64::from(MARKER_REMOVE_DM);
    let mut near: Option<(i64, String)> = None;
    for (key, v) in world.state.flags.iter() {
        if !key.starts_with(&prefix) {
            continue;
        }
        let mx = (v & 0xFFFF) as i32 - MARKER_BIAS;
        let mz = (v >> 16) as i32 - MARKER_BIAS;
        let (dx, dz) = (i64::from(mx - x), i64::from(mz - z));
        let d2 = dx * dx + dz * dz;
        if d2 <= limit && near.as_ref().is_none_or(|(b, _)| d2 < *b) {
            near = Some((d2, key.to_owned()));
        }
    }
    if let Some((_, key)) = near {
        world.state.flags.clear(&key);
        world.events.push(Event::Mechanic(MechanicEvent {
            kind: MechanicEvent::MARKER_REMOVED,
            index: 0,
        }));
        return;
    }
    if marker_total(&world.state.flags) >= MAX_MARKERS {
        return;
    }
    let slot = (0..)
        .find(|&s| !world.state.flags.is_set(&marker_flag(&area, s)))
        .unwrap_or(0);
    let packed = ((z + MARKER_BIAS) as u32) << 16 | (x + MARKER_BIAS) as u32;
    world.state.flags.set(&marker_flag(&area, slot), packed);
    world.events.push(Event::Mechanic(MechanicEvent {
        kind: MechanicEvent::MARKER_PLACED,
        index: 0,
    }));
}

/// Ticks `world`, then finishes a trip to another area if one is waiting. Hosts that handle
/// menu commands (the browser, replays) call this instead of `tick`.
pub fn tick_world(world: &mut World, input: &TickInput, content: &Content) {
    crate::tick::tick(world, input);
    finish_tick(world, content);
}

/// Finishes the trip `Command::TravelFar` started, by replacing `world` with the destination
/// area's. A trip that isn't allowed is dropped. Returns whether the world was replaced.
pub fn finish_tick(world: &mut World, content: &Content) -> bool {
    let pending = world.state.flags.get(TRAVEL_FLAG);
    if pending == 0 {
        return false;
    }
    world.state.flags.clear(TRAVEL_FLAG);
    match build_destination(world, content, (pending - 1) as usize) {
        Some(next) => {
            *world = next;
            true
        }
        None => false,
    }
}

fn build_destination(world: &mut World, content: &Content, place: usize) -> Option<World> {
    let target = places(content).into_iter().nth(place)?;
    let here = world.physics.scenario.as_str();
    if target.area == here
        || !world
            .state
            .flags
            .is_set(&flags::lantern_lit(&target.lantern.id))
        || world.state.death.remnant.is_some()
    {
        return None;
    }
    // The world doesn't keep its seed; derive the next one from the tick, so a replay does
    // the same.
    let seed = world.tick.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ place as u64;
    let mut data = crate::save::capture(world, seed, content);
    data.scenario = target.area.clone();
    data.respawn_lantern = Some(target.lantern.id.clone());
    let mut next = crate::save::restore(&data, content).ok()?;
    next.tick = world.tick;
    next.events = std::mem::take(&mut world.events);
    next.spells = world.spells;
    next.sync_player_moveset();
    let (index, _) = next.layout.lantern(&target.lantern.id)?;
    lantern::rest(&mut next, index);
    next.events.push(Event::Mechanic(MechanicEvent {
        kind: MechanicEvent::AREA_CHANGED,
        index: place as u16,
    }));
    Some(next)
}

// ---------------------------------------------------------------------------------------
// The browser's read-only view

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The map as JSON, for the DOM map and its list view. Read-only. `places` is
/// [`places`] of the content; `bounds` is the area's `[min_x, min_z, max_x, max_z]` if known.
///
/// Fields: `area`, `cell` (meters), `bounds`, `player` `{x, z, yaw}`, `cells` (visited,
/// `[ix, iz]` pairs), `resting` (the Lantern's ID or null), `remnant` (`{x, z, glimmer}` or
/// null), `markers` (`[x, z]` in this area), `markerTotal`, `bosses` (defeated ones in this
/// area: `{name, x, z}`), and `places` (every Lantern: `i` the index `TravelFar` takes,
/// `area`, `id`, `x`, `y`, `z`, `lit`, `idx` the index in this area for `Travel` or -1,
/// `here`).
pub fn view_json(world: &World, places: &[Place], bounds: Option<[f32; 4]>) -> String {
    let area = world.physics.scenario.as_str();
    let (px, pz, pyaw) = world
        .actors
        .get(world.player)
        .map_or((0.0, 0.0, 0.0), |p| (p.pos[0], p.pos[2], p.yaw));
    let mut s = String::with_capacity(1024);
    s.push_str(&format!("{{\"area\":{},\"cell\":{CELL_M}", esc(area)));
    match bounds {
        Some(b) => s.push_str(&format!(
            ",\"bounds\":[{},{},{},{}]",
            b[0], b[1], b[2], b[3]
        )),
        None => s.push_str(",\"bounds\":null"),
    }
    s.push_str(&format!(
        ",\"player\":{{\"x\":{px:.2},\"z\":{pz:.2},\"yaw\":{pyaw:.3}}},\"cells\":["
    ));
    let cells = visited_cells(&world.state.flags, area);
    for (i, (ix, iz)) in cells.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("[{ix},{iz}]"));
    }
    s.push_str("],\"resting\":");
    match &world.state.resting {
        Some(id) => s.push_str(&esc(id)),
        None => s.push_str("null"),
    }
    s.push_str(",\"remnant\":");
    match &world.state.death.remnant {
        Some(r) => s.push_str(&format!(
            "{{\"x\":{:.2},\"z\":{:.2},\"glimmer\":{}}}",
            r.pos[0], r.pos[2], r.glimmer
        )),
        None => s.push_str("null"),
    }
    s.push_str(",\"markers\":[");
    for (i, m) in markers(&world.state.flags, area).iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("[{:.1},{:.1}]", m[0], m[1]));
    }
    s.push_str(&format!(
        "],\"markerTotal\":{},\"bosses\":[",
        marker_total(&world.state.flags)
    ));
    let mut first = true;
    for b in &world.layout.bosses {
        if !world.state.flags.is_set(&b.defeated_flag()) {
            continue;
        }
        if !first {
            s.push(',');
        }
        first = false;
        s.push_str(&format!(
            "{{\"name\":{},\"x\":{:.1},\"z\":{:.1}}}",
            esc(&b.def.name),
            b.gate.pos[0],
            b.gate.pos[2]
        ));
    }
    s.push_str("],\"places\":[");
    for (i, p) in places.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let lit = world.state.flags.is_set(&flags::lantern_lit(&p.lantern.id));
        let here = p.area == area;
        let idx = if here {
            world
                .layout
                .lantern(&p.lantern.id)
                .map_or(-1, |(n, _)| n as i64)
        } else {
            -1
        };
        s.push_str(&format!(
            "{{\"i\":{i},\"area\":{},\"id\":{},\"x\":{:.1},\"y\":{:.1},\"z\":{:.1},\"lit\":{lit},\"idx\":{idx},\"here\":{here}}}",
            esc(&p.area),
            esc(&p.lantern.id),
            p.lantern.pos[0],
            p.lantern.pos[1],
            p.lantern.pos[2],
        ));
    }
    s.push_str("]}");
    s
}
