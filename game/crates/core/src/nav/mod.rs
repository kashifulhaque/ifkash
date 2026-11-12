//! Navigation: the baked grid and A* pathfinding for enemies.
//! See docs/game/01-architecture.md, "Navigation".
//!
//! The grid is baked offline (`lf-tools bake nav`, or [`bake::bake`] in tests) from the
//! static level's colliders: each cell stores a floor height and a cost, stacked floors
//! are separate cells in one column, and ladder and stair edges are explicit [`NavLink`]s.
//! A [`NavQueue`] answers path requests with a per-tick request limit and an expansion
//! budget; a request that runs out of budget fails with [`PathError::BudgetExceeded`]
//! rather than returning a partial path.
//!
//! The grid is immutable data and not part of the world state hash. Nothing here reads
//! the clock or the random streams, and all search arithmetic is integer, so results
//! don't depend on the machine.

use std::ops::Range;

use lf_content::NavConfig;
use serde::{Deserialize, Serialize};

pub mod bake;
mod path;

pub use path::{NavQueue, Path, PathError, PathRequest, PathResponse, Search};

/// One walkable floor in a column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavCell {
    /// Floor height in millimeters. Heights are quantized so the baked bytes don't depend
    /// on float noise.
    pub floor_mm: i32,
    /// Cost multiplier for entering the cell (1 is open ground).
    pub cost: u8,
}

/// A non-adjacent edge between two cells: a ladder or a stair between layers. Directed;
/// bake both ways for a two-way link.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavLink {
    pub from: u32,
    pub to: u32,
    /// Cost in tenths of a cell step (10 is one orthogonal step on open ground).
    pub cost: u32,
}

/// The baked navigation grid of one area. Cell ids are indices into the cell list, which
/// is ordered by column (row-major along Z, then X) and then by floor height.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavGrid {
    pub config: NavConfig,
    /// World X and Z of the corner of column (0, 0).
    pub origin: [f32; 2],
    pub width: u32,
    pub depth: u32,
    col_start: Vec<u32>,
    cells: Vec<NavCell>,
    /// Sorted by `from`, then `to`.
    links: Vec<NavLink>,
}

impl NavGrid {
    /// Builds a grid from columns of cells (each ordered by floor height) and links.
    /// `columns` is row-major: column `cz * width + cx`.
    pub fn from_columns(
        config: NavConfig,
        origin: [f32; 2],
        width: u32,
        depth: u32,
        columns: &[Vec<NavCell>],
        mut links: Vec<NavLink>,
    ) -> Self {
        assert_eq!(
            columns.len(),
            (width * depth) as usize,
            "one column per cell"
        );
        let mut col_start = Vec::with_capacity(columns.len() + 1);
        let mut cells = Vec::new();
        for col in columns {
            col_start.push(cells.len() as u32);
            cells.extend_from_slice(col);
        }
        col_start.push(cells.len() as u32);
        links.sort_by_key(|l| (l.from, l.to));
        Self {
            config,
            origin,
            width,
            depth,
            col_start,
            cells,
            links,
        }
    }

    /// Number of walkable cells.
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn cell(&self, id: u32) -> NavCell {
        self.cells[id as usize]
    }

    pub fn links(&self) -> &[NavLink] {
        &self.links
    }

    /// 64-bit xxh3 of the canonical postcard bytes: the "same hash twice" check for baking.
    pub fn hash(&self) -> u64 {
        xxhash_rust::xxh3::xxh3_64(&self.to_bytes())
    }

    /// Canonical serialized form, as written by `lf-tools bake nav`.
    pub fn to_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("the grid serializes")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }

    fn column_cells(&self, cx: u32, cz: u32) -> Range<u32> {
        let i = (cz * self.width + cx) as usize;
        self.col_start[i]..self.col_start[i + 1]
    }

    /// The column a cell is in, as (x, z) column indices.
    pub fn column_of(&self, id: u32) -> (u32, u32) {
        let col = self.col_start.partition_point(|&s| s <= id) as u32 - 1;
        (col % self.width, col / self.width)
    }

    /// World position of a cell: the center of its column at its floor height.
    pub fn cell_pos(&self, id: u32) -> [f32; 3] {
        let (cx, cz) = self.column_of(id);
        let s = self.config.cell_size;
        [
            self.origin[0] + (cx as f32 + 0.5) * s,
            self.cell(id).floor_mm as f32 / 1000.0,
            self.origin[1] + (cz as f32 + 0.5) * s,
        ]
    }

    /// Largest crossable floor change between adjacent cells, in millimeters.
    fn max_climb_mm(&self) -> i32 {
        (self.config.max_edge_climb * 1000.0).round() as i32
    }

    /// The cell reached by stepping from `id` one column over (`dx`, `dz` each -1, 0 or 1),
    /// or `None` if there is no cell there within the climb limit. A diagonal step also
    /// needs both orthogonal steps, so paths never cut a wall corner.
    pub fn step(&self, id: u32, dx: i32, dz: i32) -> Option<u32> {
        let (cx, cz) = self.column_of(id);
        self.step_from(id, cx, cz, dx, dz)
    }

    fn step_from(&self, id: u32, cx: u32, cz: u32, dx: i32, dz: i32) -> Option<u32> {
        let nx = cx as i64 + i64::from(dx);
        let nz = cz as i64 + i64::from(dz);
        if nx < 0 || nz < 0 || nx >= i64::from(self.width) || nz >= i64::from(self.depth) {
            return None;
        }
        if dx != 0 && dz != 0 {
            self.step_from(id, cx, cz, dx, 0)?;
            self.step_from(id, cx, cz, 0, dz)?;
        }
        let here = self.cell(id).floor_mm;
        let limit = self.max_climb_mm();
        let mut best: Option<(i32, u32)> = None;
        for n in self.column_cells(nx as u32, nz as u32) {
            let d = (self.cell(n).floor_mm - here).abs();
            if d <= limit && best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, n));
            }
        }
        best.map(|(_, n)| n)
    }

    /// Links leaving `id`.
    fn links_from(&self, id: u32) -> &[NavLink] {
        let lo = self.links.partition_point(|l| l.from < id);
        let hi = self.links.partition_point(|l| l.from <= id);
        &self.links[lo..hi]
    }

    /// The walkable cell nearest a world position, or `None` if none is within the snap
    /// radius. Returns the cell and whether the position was inside it (no snapping).
    pub fn locate(&self, pos: [f32; 3]) -> Option<(u32, bool)> {
        let s = self.config.cell_size;
        let fx = ((pos[0] - self.origin[0]) / s).floor();
        let fz = ((pos[2] - self.origin[1]) / s).floor();
        if !fx.is_finite() || !fz.is_finite() || fx.abs() > 1.0e6 || fz.abs() > 1.0e6 {
            return None;
        }
        let (px, pz) = (fx as i64, fz as i64);
        let snap_mm = (self.config.snap_height * 1000.0).round() as i64;
        let y_mm = (pos[1] * 1000.0).round() as i64;
        let radius = i64::from(self.config.snap_radius_cells);
        // Best by (squared column distance, vertical distance, id).
        let mut best: Option<((i64, i64, u32), u32)> = None;
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let (x, z) = (px + dx, pz + dz);
                if x < 0 || z < 0 || x >= i64::from(self.width) || z >= i64::from(self.depth) {
                    continue;
                }
                for id in self.column_cells(x as u32, z as u32) {
                    let dy = (i64::from(self.cell(id).floor_mm) - y_mm).abs();
                    if dy > snap_mm {
                        continue;
                    }
                    let key = (dx * dx + dz * dz, dy, id);
                    if best.is_none_or(|(k, _)| key < k) {
                        best = Some((key, id));
                    }
                }
            }
        }
        best.map(|((d2, _, _), id)| (id, d2 == 0))
    }
}

#[cfg(test)]
mod tests;
