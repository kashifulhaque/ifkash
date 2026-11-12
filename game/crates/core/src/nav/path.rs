//! A* over the grid, path smoothing, and the per-tick request queue.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

use lf_content::NavConfig;
use serde::{Deserialize, Serialize};

use super::NavGrid;

/// Step costs in tenths of a cell: orthogonal and diagonal (about 1.4 times).
const ORTHO: u32 = 10;
const DIAG: u32 = 14;

/// The eight moves, in the fixed order A* tries them (ties break by this order).
const DIRS: [(i32, i32); 8] = [
    (0, -1),
    (1, 0),
    (0, 1),
    (-1, 0),
    (1, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
];

/// A finished path: world positions from start to goal, already smoothed.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub waypoints: Vec<[f32; 3]>,
    /// Node expansions the search used.
    pub expanded: u32,
}

/// Why a request produced no path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathError {
    /// The start is not near any walkable cell.
    StartOffGrid,
    /// The goal is not near any walkable cell.
    GoalOffGrid,
    /// The search ran out of reachable cells: the goal can't be walked to.
    NoPath,
    /// The search used its whole expansion budget without reaching the goal.
    BudgetExceeded { expanded: u32 },
    /// The request queue was full, so the request was refused.
    QueueFull,
}

/// Reusable A* scratch space. It holds no state between searches that affects results;
/// stamps just avoid clearing the arrays. Create one per queue.
#[derive(Clone, Debug, Default)]
pub struct Search {
    stamp: Vec<u32>,
    closed: Vec<bool>,
    g: Vec<u32>,
    parent: Vec<u32>,
    generation: u32,
}

impl Search {
    pub fn new() -> Self {
        Self::default()
    }

    fn begin(&mut self, cells: usize) {
        if self.stamp.len() != cells {
            self.stamp = vec![0; cells];
            self.closed = vec![false; cells];
            self.g = vec![0; cells];
            self.parent = vec![0; cells];
            self.generation = 0;
        }
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.stamp.fill(0);
            self.generation = 1;
        }
    }

    fn g(&self, id: u32) -> Option<u32> {
        (self.stamp[id as usize] == self.generation).then(|| self.g[id as usize])
    }

    fn set(&mut self, id: u32, g: u32, parent: u32) {
        let i = id as usize;
        if self.stamp[i] != self.generation {
            self.stamp[i] = self.generation;
            self.closed[i] = false;
        }
        self.g[i] = g;
        self.parent[i] = parent;
    }

    fn is_closed(&self, id: u32) -> bool {
        self.stamp[id as usize] == self.generation && self.closed[id as usize]
    }
}

/// Octile distance in tenths of a cell, scaled by the cheapest cell cost (1): admissible.
fn heuristic(grid: &NavGrid, a: u32, b: u32) -> u32 {
    let (ax, az) = grid.column_of(a);
    let (bx, bz) = grid.column_of(b);
    let dx = ax.abs_diff(bx);
    let dz = az.abs_diff(bz);
    let (hi, lo) = if dx > dz { (dx, dz) } else { (dz, dx) };
    ORTHO * (hi - lo) + DIAG * lo
}

impl NavGrid {
    /// Finds a path from `from` to `to`, using at most `budget` node expansions. The
    /// result is smoothed; it never contains a partial path.
    pub fn find_path(
        &self,
        search: &mut Search,
        from: [f32; 3],
        to: [f32; 3],
        budget: u32,
    ) -> Result<Path, PathError> {
        let (start, start_exact) = self.locate(from).ok_or(PathError::StartOffGrid)?;
        let (goal, goal_exact) = self.locate(to).ok_or(PathError::GoalOffGrid)?;
        let (cells, expanded) = self.search(search, start, goal, budget)?;
        let kept = self.smooth(&cells);
        let mut waypoints: Vec<[f32; 3]> = kept.iter().map(|&c| self.cell_pos(c)).collect();
        // Start and end at the real positions when they were on the grid, so an agent
        // doesn't walk to a cell center it already passed.
        if start_exact {
            waypoints[0][0] = from[0];
            waypoints[0][2] = from[2];
        }
        if goal_exact {
            let last = waypoints.len() - 1;
            waypoints[last][0] = to[0];
            waypoints[last][2] = to[2];
        }
        Ok(Path {
            waypoints,
            expanded,
        })
    }

    /// A* between two cells. Returns the cell chain and the expansions used.
    fn search(
        &self,
        s: &mut Search,
        start: u32,
        goal: u32,
        budget: u32,
    ) -> Result<(Vec<u32>, u32), PathError> {
        s.begin(self.cell_count());
        // Ordered by (f, h, id): fully deterministic, and prefers nodes nearer the goal.
        let mut open: BinaryHeap<Reverse<(u32, u32, u32)>> = BinaryHeap::new();
        s.set(start, 0, start);
        let h0 = heuristic(self, start, goal);
        open.push(Reverse((h0, h0, start)));
        let mut expanded = 0u32;
        while let Some(Reverse((_, _, id))) = open.pop() {
            if s.is_closed(id) {
                continue;
            }
            if id == goal {
                let mut chain = vec![goal];
                let mut at = goal;
                while at != start {
                    at = s.parent[at as usize];
                    chain.push(at);
                }
                chain.reverse();
                return Ok((chain, expanded));
            }
            if expanded >= budget {
                return Err(PathError::BudgetExceeded { expanded });
            }
            expanded += 1;
            s.closed[id as usize] = true;
            let g = s.g(id).expect("a popped node has a cost");
            let (cx, cz) = self.column_of(id);
            for (dx, dz) in DIRS {
                let Some(n) = self.step_from(id, cx, cz, dx, dz) else {
                    continue;
                };
                let base = if dx != 0 && dz != 0 { DIAG } else { ORTHO };
                let ng = g + base * u32::from(self.cell(n).cost);
                self.relax(s, &mut open, n, ng, id, goal);
            }
            for link in self.links_from(id) {
                self.relax(s, &mut open, link.to, g + link.cost, id, goal);
            }
        }
        Err(PathError::NoPath)
    }

    fn relax(
        &self,
        s: &mut Search,
        open: &mut BinaryHeap<Reverse<(u32, u32, u32)>>,
        n: u32,
        ng: u32,
        parent: u32,
        goal: u32,
    ) {
        if s.is_closed(n) || s.g(n).is_some_and(|old| old <= ng) {
            return;
        }
        s.set(n, ng, parent);
        let h = heuristic(self, n, goal);
        open.push(Reverse((ng + h, h, n)));
    }

    /// Greedy line-of-sight smoothing: keeps a cell only where the straight grid line to
    /// the next one is blocked. Link edges are always kept.
    fn smooth(&self, chain: &[u32]) -> Vec<u32> {
        let mut out = vec![chain[0]];
        let mut i = 0;
        while i + 1 < chain.len() {
            let mut j = i + 1;
            while j + 1 < chain.len()
                && !self.is_link(chain[j], chain[j + 1])
                && self.line_clear(chain[i], chain[j + 1])
            {
                j += 1;
            }
            out.push(chain[j]);
            i = j;
        }
        out
    }

    fn is_link(&self, a: u32, b: u32) -> bool {
        self.links_from(a).iter().any(|l| l.to == b)
    }

    /// True if walking the Bresenham line of columns from `a` to `b` stays on connected
    /// cells and ends on `b`.
    fn line_clear(&self, a: u32, b: u32) -> bool {
        let (ax, az) = self.column_of(a);
        let (bx, bz) = self.column_of(b);
        let (mut x, mut z) = (i64::from(ax), i64::from(az));
        let (tx, tz) = (i64::from(bx), i64::from(bz));
        let (dx, dz) = ((tx - x).abs(), -(tz - z).abs());
        let (sx, sz) = ((tx - x).signum(), (tz - z).signum());
        let mut err = dx + dz;
        let mut cur = a;
        while (x, z) != (tx, tz) {
            let (px, pz) = (x, z);
            let e2 = 2 * err;
            if e2 >= dz {
                err += dz;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                z += sz;
            }
            match self.step_from(cur, px as u32, pz as u32, (x - px) as i32, (z - pz) as i32) {
                Some(n) => cur = n,
                None => return false,
            }
        }
        cur == b
    }
}

/// A path request waiting in the queue.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathRequest {
    pub ticket: u32,
    pub from: [f32; 3],
    pub to: [f32; 3],
}

/// The answer to one request.
#[derive(Clone, Debug, PartialEq)]
pub struct PathResponse {
    pub ticket: u32,
    pub result: Result<Path, PathError>,
}

/// Queues path requests and answers a limited number per tick, oldest first. Enemies
/// request a path, keep their old one meanwhile, and pick the answer up on a later tick.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NavQueue {
    pending: VecDeque<PathRequest>,
    next_ticket: u32,
    requests_per_tick: u32,
    max_pending: u32,
    expansion_budget: u32,
    #[serde(skip)]
    search: Search,
}

impl NavQueue {
    /// A queue with the limits from `nav.ron`.
    pub fn new(config: &NavConfig) -> Self {
        Self {
            pending: VecDeque::new(),
            next_ticket: 0,
            requests_per_tick: config.requests_per_tick,
            max_pending: config.max_pending,
            expansion_budget: config.expansion_budget,
            search: Search::new(),
        }
    }

    /// Queues a request and returns its ticket, or `QueueFull` if too many are waiting.
    pub fn request(&mut self, from: [f32; 3], to: [f32; 3]) -> Result<u32, PathError> {
        if self.pending.len() >= self.max_pending as usize {
            return Err(PathError::QueueFull);
        }
        let ticket = self.next_ticket;
        self.next_ticket = self.next_ticket.wrapping_add(1);
        self.pending.push_back(PathRequest { ticket, from, to });
        Ok(ticket)
    }

    /// Requests still waiting.
    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    /// Answers up to the per-tick limit of requests, oldest first. Call once per tick.
    pub fn run_tick(&mut self, grid: &NavGrid) -> Vec<PathResponse> {
        let mut out = Vec::new();
        for _ in 0..self.requests_per_tick {
            let Some(r) = self.pending.pop_front() else {
                break;
            };
            let result = grid.find_path(&mut self.search, r.from, r.to, self.expansion_budget);
            out.push(PathResponse {
                ticket: r.ticket,
                result,
            });
        }
        out
    }
}
