//! Grid baking: probes the static level's colliders column by column.
//!
//! For each 0.5 m column it casts a ray down to find floors (top to bottom), keeps a
//! floor if a sphere the size of an agent fits on it with headroom, and gives it a higher
//! cost near walls. Everything it reads is the level in [`PhysicsState`], so baking the
//! same scenario and config twice gives byte-identical grids.

use lf_content::{NavConfig, Scenario};

use super::{NavCell, NavGrid};
use crate::physics::PhysicsState;

/// How far a ray steps down when it starts inside a solid or after a rejected floor, in
/// meters. Tuning for the bake only; it doesn't change what is walkable on open floors.
const PROBE_STEP: f32 = 0.25;
/// Rays are shorter than this to avoid hitting at the very start of the cast.
const INSIDE_EPS: f32 = 1.0e-4;
/// Upper bound on probes per column, so a pathological level can't hang the bake.
const MAX_PROBES: u32 = 256;

/// The XZ extent and Y range the scenario's colliders cover: (min_x, min_z, max_x, max_z,
/// min_y, max_y). `None` for a level with no colliders.
fn bounds(scenario: &Scenario) -> Option<[f32; 6]> {
    let mut b: Option<[f32; 6]> = None;
    let mut add = |min: [f32; 3], max: [f32; 3]| {
        b = Some(match b {
            None => [min[0], min[2], max[0], max[2], min[1], max[1]],
            Some(o) => [
                o[0].min(min[0]),
                o[1].min(min[2]),
                o[2].max(max[0]),
                o[3].max(max[2]),
                o[4].min(min[1]),
                o[5].max(max[1]),
            ],
        });
    };
    if let Some(h) = &scenario.heightfield {
        let lo = h.heights.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = h.heights.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let c = h.center;
        add(
            [c[0] - h.size_x / 2.0, c[1] + lo, c[2] - h.size_z / 2.0],
            [c[0] + h.size_x / 2.0, c[1] + hi, c[2] + h.size_z / 2.0],
        );
    }
    for bx in &scenario.boxes {
        let (c, e) = (bx.center, bx.half_extents);
        add(
            [c[0] - e[0], c[1] - e[1], c[2] - e[2]],
            [c[0] + e[0], c[1] + e[1], c[2] + e[2]],
        );
    }
    b
}

/// Bakes the navigation grid for `scenario`, whose level `physics` was built from.
pub fn bake(physics: &PhysicsState, scenario: &Scenario, cfg: &NavConfig) -> NavGrid {
    let Some([min_x, min_z, max_x, max_z, min_y, max_y]) = bounds(scenario) else {
        return NavGrid::from_columns(cfg.clone(), [0.0, 0.0], 0, 0, &[], Vec::new());
    };
    let s = cfg.cell_size;
    // Align the grid to multiples of the cell size so cell centers sit on a fixed lattice.
    let ox = (min_x / s).floor() * s;
    let oz = (min_z / s).floor() * s;
    let width = ((max_x - ox) / s).ceil().max(1.0) as u32;
    let depth = ((max_z - oz) / s).ceil().max(1.0) as u32;
    let top = max_y + 1.0;
    let bottom = min_y - 1.0;
    let mut columns = Vec::with_capacity((width * depth) as usize);
    for cz in 0..depth {
        for cx in 0..width {
            let x = ox + (cx as f32 + 0.5) * s;
            let z = oz + (cz as f32 + 0.5) * s;
            columns.push(probe_column(physics, cfg, x, z, top, bottom));
        }
    }
    NavGrid::from_columns(cfg.clone(), [ox, oz], width, depth, &columns, Vec::new())
}

/// The walkable floors of one column, lowest first.
fn probe_column(
    physics: &PhysicsState,
    cfg: &NavConfig,
    x: f32,
    z: f32,
    top: f32,
    bottom: f32,
) -> Vec<NavCell> {
    let mut floors: Vec<NavCell> = Vec::new();
    let mut y = top;
    let mut probes = 0;
    while y > bottom && probes < MAX_PROBES && floors.len() < usize::from(cfg.max_layers) {
        probes += 1;
        let Some(d) = physics.raycast([x, y, z], [0.0, -1.0, 0.0], y - bottom) else {
            break;
        };
        if d < INSIDE_EPS {
            // Started inside a solid: keep stepping down until out of it.
            y -= PROBE_STEP;
            continue;
        }
        let floor = y - d;
        match classify(physics, cfg, x, floor, z) {
            Some(cost) => {
                floors.push(NavCell {
                    floor_mm: (floor * 1000.0).round() as i32,
                    cost,
                });
                // Skip the headroom: nothing walkable can be closer under this floor.
                y = floor - cfg.agent_height;
            }
            None => y = floor - PROBE_STEP,
        }
    }
    floors.reverse();
    floors
}

/// `Some(cost)` if an agent fits on the floor point, `None` if it doesn't.
fn classify(physics: &PhysicsState, cfg: &NavConfig, x: f32, floor: f32, z: f32) -> Option<u8> {
    let r = cfg.agent_radius;
    // The agent sphere is lifted so low steps and ramps don't count as walls, and swept
    // up to the top of the agent's height: a hit anywhere means a wall or ceiling.
    let rise = (cfg.agent_height - cfg.step_lift - 2.0 * r).max(0.01);
    let center = [x, floor + cfg.step_lift + r, z];
    if physics.sphere_cast(center, [0.0, rise, 0.0], r).is_some() {
        return None;
    }
    // A wider sphere: touching means a wall within the edge margin.
    let wide = r + cfg.edge_margin;
    let center = [x, floor + cfg.step_lift + wide, z];
    if physics
        .sphere_cast(center, [0.0, 0.01, 0.0], wide)
        .is_some()
    {
        Some(cfg.edge_cost)
    } else {
        Some(1)
    }
}
