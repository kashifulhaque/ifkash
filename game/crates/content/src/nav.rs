//! Navigation tuning from `nav.ron` (docs/game/01-architecture.md, "Navigation").
//!
//! Kept out of [`crate::Content`] so the nav grid work package doesn't reshape the shared
//! content struct; load it with [`NavConfig::load_dir`] or [`NavConfig::embedded`]. The
//! baked grid stores the config it was baked with, so its hash covers these numbers.

use std::path::Path;

use serde::{Deserialize, Serialize};

const EMBEDDED: &str = include_str!("../../../data/nav.ron");

/// Navigation grid and pathfinding numbers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavConfig {
    /// Cell edge length in meters.
    pub cell_size: f32,
    /// Largest floor-height change between adjacent cells that can be crossed, in meters.
    pub max_edge_climb: f32,
    /// Radius of the sphere that must fit in a walkable cell, in meters.
    pub agent_radius: f32,
    /// Headroom a floor needs, in meters.
    pub agent_height: f32,
    /// How far above the floor the clearance sphere sits, in meters.
    pub step_lift: f32,
    /// Distance from a wall inside which a cell costs `edge_cost`, in meters.
    pub edge_margin: f32,
    /// Cost multiplier (1 or more) for cells near a wall.
    pub edge_cost: u8,
    /// Floors kept per column.
    pub max_layers: u8,
    /// Node expansions one path request may use before it fails.
    pub expansion_budget: u32,
    /// Path requests answered per tick.
    pub requests_per_tick: u32,
    /// Requests that can wait in the queue; more are refused.
    pub max_pending: u32,
    /// How many cells off the grid a start or goal can snap to a walkable cell.
    pub snap_radius_cells: u32,
    /// Vertical distance within which a position uses a floor, in meters.
    pub snap_height: f32,
}

impl NavConfig {
    /// Parses `nav.ron` text.
    pub fn from_ron(text: &str) -> Result<Self, String> {
        let cfg: Self = ron::from_str(text).map_err(|e| format!("nav.ron: {e}"))?;
        cfg.check()?;
        Ok(cfg)
    }

    /// The `nav.ron` baked into this crate.
    pub fn embedded() -> Result<Self, String> {
        Self::from_ron(EMBEDDED)
    }

    /// Reads `nav.ron` from a data directory laid out like `game/data/`.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self, String> {
        let path = dir.as_ref().join("nav.ron");
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::from_ron(&text)
    }

    fn check(&self) -> Result<(), String> {
        let positive = [
            ("cell_size", self.cell_size),
            ("max_edge_climb", self.max_edge_climb),
            ("agent_radius", self.agent_radius),
            ("agent_height", self.agent_height),
            ("step_lift", self.step_lift),
        ];
        for (name, v) in positive {
            if v.is_nan() || v <= 0.0 {
                return Err(format!("nav.ron: {name} must be positive"));
            }
        }
        if self.edge_cost == 0 || self.max_layers == 0 {
            return Err("nav.ron: edge_cost and max_layers must be at least 1".into());
        }
        if self.expansion_budget == 0 || self.requests_per_tick == 0 || self.max_pending == 0 {
            return Err("nav.ron: budget, request limit and queue size must be at least 1".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_file_parses_with_spec_numbers() {
        let c = NavConfig::embedded().unwrap();
        assert_eq!(c.cell_size, 0.5);
        assert_eq!(c.expansion_budget, 512);
        assert_eq!(c.requests_per_tick, 4);
    }
}
