//! Scenarios: a static level (a heightfield plus box colliders) and a player spawn.
//! Loaded from `scenarios/<name>.ron`. See docs/game/01-architecture.md, "Physics".

use serde::{Deserialize, Serialize};

/// A level the simulation can start in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    /// Player feet position at spawn, in meters.
    pub spawn: [f32; 3],
    /// Player facing at spawn, in radians from +Z toward +X.
    pub spawn_yaw: f32,
    /// An actor whose feet drop below this height dies (falling out of the world).
    pub kill_y: f32,
    #[serde(default)]
    pub heightfield: Option<Heightfield>,
    #[serde(default)]
    pub boxes: Vec<BoxCollider>,
}

/// A terrain heightfield centered on `center`. Vertex `(row, col)` sits at
/// `x = center.x - size_x / 2 + col * size_x / (cols - 1)` and
/// `z = center.z - size_z / 2 + row * size_z / (rows - 1)`, at height
/// `center.y + heights[row * cols + col]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Heightfield {
    pub center: [f32; 3],
    /// Number of vertex rows, along Z. At least 2.
    pub rows: u32,
    /// Number of vertex columns, along X. At least 2.
    pub cols: u32,
    /// Total extent along X, in meters.
    pub size_x: f32,
    /// Total extent along Z, in meters.
    pub size_z: f32,
    /// Row-major heights in meters: `rows * cols` values.
    pub heights: Vec<f32>,
}

/// An axis-aligned box collider (a kit piece).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoxCollider {
    /// Label for tools and error messages.
    pub name: String,
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
}

impl Scenario {
    /// Checks shape invariants the physics builder relies on.
    pub fn check(&self) -> Result<(), String> {
        if let Some(h) = &self.heightfield {
            if h.rows < 2 || h.cols < 2 {
                return Err(format!(
                    "heightfield needs at least 2x2 vertices, got {}x{}",
                    h.rows, h.cols
                ));
            }
            let want = h.rows as usize * h.cols as usize;
            if h.heights.len() != want {
                return Err(format!(
                    "heightfield has {} heights, expected rows * cols = {want}",
                    h.heights.len()
                ));
            }
            if h.size_x <= 0.0 || h.size_z <= 0.0 {
                return Err("heightfield size_x and size_z must be positive".to_owned());
            }
        }
        for b in &self.boxes {
            if b.half_extents.iter().any(|&e| e <= 0.0) {
                return Err(format!("box {:?}: half_extents must be positive", b.name));
            }
        }
        Ok(())
    }
}
