//! Bake warnings: problems in a level that the content loader can't call errors,
//! because they depend on the baked navigation grid or on where things ended up
//! (docs/game/02-world.md, "Build an area", step 5).

use std::collections::VecDeque;

use lf_content::level::{MAX_HAND_PLACED_PROPS, sample_height};
use lf_content::{Level, Scenario};
use lf_core::nav::NavGrid;

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
    out
}
