//! Water-only movement (docs/game/05-enemies-and-bosses.md, "Navigation" and "Enemy roster",
//! Drowned and Reed Stalker): an enemy with `water_only` never leaves the scenario's water
//! volumes. Until the level format (WP-040) places water, the volumes are the scenario's
//! `water` zones. A scenario with none leaves a `water_only` enemy unconfined.
//!
//! [`confine`] runs after the actor's steering. It cancels planar velocity that would carry
//! the actor out of the water, and puts back an actor that already is out (root motion from
//! a move can overshoot by a step).

use lf_content::scenario::Zone;

use crate::arena::Id;
use crate::world::{DT, World};

/// True if (x, z) is inside the zone.
fn inside(zone: &Zone, x: f32, z: f32) -> bool {
    (x - zone.center[0]).abs() <= zone.half_extents[0]
        && (z - zone.center[1]).abs() <= zone.half_extents[1]
}

/// True if (x, z) is inside any of the zones.
pub fn in_any(zones: &[Zone], x: f32, z: f32) -> bool {
    zones.iter().any(|zone| inside(zone, x, z))
}

/// The point of the zones nearest to (x, z), as (x, z). The first zone wins a tie.
pub fn nearest_point(zones: &[Zone], x: f32, z: f32) -> Option<[f32; 2]> {
    let mut best: Option<(f32, [f32; 2])> = None;
    for zone in zones {
        let px = x.clamp(
            zone.center[0] - zone.half_extents[0],
            zone.center[0] + zone.half_extents[0],
        );
        let pz = z.clamp(
            zone.center[1] - zone.half_extents[1],
            zone.center[1] + zone.half_extents[1],
        );
        let d = (px - x) * (px - x) + (pz - z) * (pz - z);
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, [px, pz]));
        }
    }
    best.map(|(_, p)| p)
}

/// Keeps a `water_only` actor in the water.
pub fn confine(world: &mut World, id: Id) {
    let Some(a) = world.actors.get(id) else {
        return;
    };
    let Some(b) = a.ai.as_ref() else {
        return;
    };
    if !world.ai.defs[usize::from(b.def)].data.water_only || world.layout.water.is_empty() {
        return;
    }
    let zones = &world.layout.water;
    let (x, z) = (a.pos[0], a.pos[2]);
    let (nx, nz) = (x + a.vel[0] * DT, z + a.vel[2] * DT);
    let put_back = if in_any(zones, x, z) {
        None
    } else {
        nearest_point(zones, x, z)
    };
    let stop = !in_any(zones, nx, nz);
    if let Some(a) = world.actors.get_mut(id) {
        if let Some([px, pz]) = put_back {
            a.pos[0] = px;
            a.pos[2] = pz;
        }
        if stop {
            a.vel[0] = 0.0;
            a.vel[2] = 0.0;
        }
    }
}
