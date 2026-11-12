//! Bake warnings: problems in a level that the content loader can't call errors,
//! because they depend on the baked navigation grid or on where things ended up
//! (docs/game/02-world.md, "Build an area", step 5).

use std::collections::VecDeque;

use lf_content::level::{MAX_HAND_PLACED_PROPS, sample_height};
use lf_content::{Level, Scenario};
use lf_core::nav::{NavGrid, Search};
use lf_core::world_state::mechanics::darkness;

/// How far below the terrain surface a feet position may sit before it counts as buried,
/// in meters. Allows for the bilinear terrain sample differing from the triangulated
/// heightfield the core collides with.
const BURIED_SLACK_M: f32 = 0.25;

/// How far above the feet the inside-a-collider test starts, so standing on a box's top
/// face isn't inside it, in meters.
const FEET_LIFT_M: f32 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningKind {
    /// A Lantern the player can't walk to from the spawn.
    UnreachableLantern,
    /// A spawn or item with no walkable cell within the grid's snap radius.
    OffNavGrid,
    /// A spawn or item inside a box collider or under the terrain.
    InsideCollider,
    /// A spawn or item outside the area's bounds.
    OutsideBounds,
    /// More hand-placed props than the dressing rules allow.
    TooManyProps,
    /// A dark area's critical path (the walk from the spawn through the Lanterns, in order)
    /// leaves the glowing zones (WP-062).
    DarkCriticalPath,
}

impl WarningKind {
    /// A short name for the printed warning.
    pub fn slug(self) -> &'static str {
        match self {
            Self::UnreachableLantern => "unreachable-lantern",
            Self::OffNavGrid => "off-nav-grid",
            Self::InsideCollider => "inside-collider",
            Self::OutsideBounds => "outside-bounds",
            Self::TooManyProps => "too-many-props",
            Self::DarkCriticalPath => "dark-critical-path",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Warning {
    pub kind: WarningKind,
    pub message: String,
}

/// Every point entity: (label, position, whether the player must reach it).
fn anchors(level: &Level) -> Vec<(String, [f32; 3], bool)> {
    let mut v = vec![(
        "the player spawn".to_owned(),
        level.entities.spawn.pos,
        false,
    )];
    for e in &level.entities.entities {
        if let Some(pos) = e.anchor() {
            v.push((
                format!("{} {:?}", e.kind(), e.label()),
                pos,
                e.must_be_reachable(),
            ));
        }
    }
    v
}

/// Cells reachable from `start` by walking: grid steps and links.
fn reachable(grid: &NavGrid, start: u32) -> Vec<bool> {
    let mut seen = vec![false; grid.cell_count()];
    let mut queue = VecDeque::from([start]);
    seen[start as usize] = true;
    while let Some(id) = queue.pop_front() {
        let steps = (-1..=1)
            .flat_map(|dx| (-1..=1).map(move |dz| (dx, dz)))
            .filter(|&d| d != (0, 0))
            .filter_map(|(dx, dz)| grid.step(id, dx, dz));
        let links = grid.links().iter().filter(|l| l.from == id).map(|l| l.to);
        for n in steps.chain(links).collect::<Vec<_>>() {
            if !seen[n as usize] {
                seen[n as usize] = true;
                queue.push_back(n);
            }
        }
    }
    seen
}

/// The node budget for a path request in the critical-path check.
const PATH_BUDGET: u32 = 4_000_000;

/// How far apart the critical path is sampled, in meters.
const LIT_SAMPLE_M: f32 = 0.5;

/// A dark area's rule (docs/game/02-world.md, "Hollowdeep": "the critical path stays inside
/// glowing zones"): walk the baked grid's shortest path from the spawn to each Lantern in order
/// and warn about the first stretch of each leg that isn't lit (a glowing zone, or within a
/// Lantern's light).
fn check_lit_path(scenario: &Scenario, grid: &NavGrid, out: &mut Vec<Warning>) {
    let Some(rule) = &scenario.darkness else {
        return;
    };
    let mut stops = vec![("the player spawn".to_owned(), scenario.spawn)];
    stops.extend(scenario.lanterns.iter().map(|l| (l.id.clone(), l.pos)));
    let mut search = Search::new();
    for pair in stops.windows(2) {
        let (from, to) = (&pair[0], &pair[1]);
        let Ok(path) = grid.find_path(&mut search, from.1, to.1, PATH_BUDGET) else {
            continue;
        };
        let dark = path.waypoints.windows(2).find_map(|seg| {
            let (dx, dz) = (seg[1][0] - seg[0][0], seg[1][2] - seg[0][2]);
            let n = ((dx * dx + dz * dz).sqrt() / LIT_SAMPLE_M).ceil().max(1.0) as u32;
            (0..=n).find_map(|k| {
                let t = k as f32 / n as f32;
                let (x, z) = (seg[0][0] + dx * t, seg[0][2] + dz * t);
                let lit = darkness::lit_by(
                    &scenario.glows,
                    &scenario.lanterns,
                    rule.lantern_light_m,
                    x,
                    z,
                );
                (!lit).then_some((x, z))
            })
        });
        if let Some((x, z)) = dark {
            out.push(Warning {
                kind: WarningKind::DarkCriticalPath,
                message: format!(
                    "the walk from {} to {} leaves the glowing zones at ({x:.1}, {z:.1})",
                    from.0, to.0
                ),
            });
        }
    }
}

/// Checks a compiled level against its baked grid.
pub fn check(level: &Level, scenario: &Scenario, grid: &NavGrid) -> Vec<Warning> {
    let mut out = Vec::new();
    let mut warn = |kind, message: String| out.push(Warning { kind, message });
    let height = grid.config.agent_height;
    let anchors = anchors(level);
    let mut located = Vec::new();
    for (label, p, _) in &anchors {
        if !level.area.bounds.contains(p[0], p[2]) {
            warn(
                WarningKind::OutsideBounds,
                format!("{label} at {p:?} is outside the area's bounds"),
            );
        }
        let (lo, hi) = (p[1] + FEET_LIFT_M, p[1] + height);
        if let Some(b) = scenario.boxes.iter().find(|b| {
            let (c, e) = (b.center, b.half_extents);
            (p[0] - c[0]).abs() < e[0]
                && (p[2] - c[2]).abs() < e[2]
                && lo < c[1] + e[1]
                && hi > c[1] - e[1]
        }) {
            warn(
                WarningKind::InsideCollider,
                format!("{label} at {p:?} is inside the collider {:?}", b.name),
            );
        }
        if let Some(h) = &scenario.heightfield {
            let ground = sample_height(h, p[0], p[2]);
            if p[1] < ground - BURIED_SLACK_M {
                warn(
                    WarningKind::InsideCollider,
                    format!("{label} at {p:?} is under the terrain (ground at {ground} m)"),
                );
            }
        }
        let cell = grid.locate(*p).map(|(id, _)| id);
        if cell.is_none() {
            warn(
                WarningKind::OffNavGrid,
                format!("{label} at {p:?} has no walkable nav cell nearby"),
            );
        }
        located.push(cell);
    }
    // anchors[0] is the player spawn.
    if let Some(start) = located[0] {
        let seen = reachable(grid, start);
        for ((label, p, must), cell) in anchors.iter().zip(&located).skip(1) {
            if let (true, Some(c)) = (*must, cell)
                && !seen[*c as usize]
            {
                warn(
                    WarningKind::UnreachableLantern,
                    format!("{label} at {p:?} can't be reached on foot from the player spawn"),
                );
            }
        }
    }
    let props = level.dressing.props.len();
    if props > MAX_HAND_PLACED_PROPS {
        warn(
            WarningKind::TooManyProps,
            format!(
                "dressing.ron places {props} props by hand; use scatter rules past {MAX_HAND_PLACED_PROPS}"
            ),
        );
    }
    // `warn` is done; the critical-path check takes `out` itself.
    check_lit_path(scenario, grid, &mut out);
    out
}
