use lf_content::{Content, NavConfig};

use super::bake::bake;
use super::*;
use crate::physics::PhysicsState;

fn baked() -> NavGrid {
    let content = Content::load_embedded().unwrap();
    let scenario = content.scenario("test_field").unwrap();
    let physics = PhysicsState::new("test_field", scenario);
    bake(&physics, scenario, &NavConfig::embedded().unwrap())
}

/// The grid is expensive to bake in a debug build; share one per test binary.
fn field() -> &'static NavGrid {
    static GRID: std::sync::OnceLock<NavGrid> = std::sync::OnceLock::new();
    GRID.get_or_init(baked)
}

fn path(from: [f32; 3], to: [f32; 3]) -> Result<Path, PathError> {
    field().find_path(&mut Search::new(), from, to, 512)
}

fn length(p: &Path) -> f32 {
    p.waypoints
        .windows(2)
        .map(|w| {
            let (dx, dz) = (w[1][0] - w[0][0], w[1][2] - w[0][2]);
            crate::math::dmath::sqrt(dx * dx + dz * dz)
        })
        .sum()
}

#[test]
fn baking_is_deterministic() {
    let a = baked();
    let b = baked();
    assert_eq!(a.hash(), b.hash());
    assert_eq!(a.to_bytes(), b.to_bytes());
    assert_eq!(a.hash(), field().hash());
    let back = NavGrid::from_bytes(&a.to_bytes()).unwrap();
    assert_eq!(back, a);
    assert!(a.cell_count() > 10_000, "{}", a.cell_count());
}

#[test]
fn obstacles_are_not_walkable() {
    let g = field();
    // The ledge's top is a floor; its face is not connected to the ramp's 9 m landing.
    let (top, exact) = g.locate([0.0, 10.0, 35.0]).unwrap();
    assert!(exact);
    assert_eq!(g.cell(top).floor_mm, 10_000);
    // Inside the ledge box at ground level there is no cell: it's solid.
    assert!(
        g.locate([0.0, 0.0, 35.0]).is_none(),
        "the ledge box is solid"
    );
    // Right against its face is too close for an agent; a meter back is walkable.
    assert!(g.locate([0.0, 9.0, 29.75]).is_some_and(|(_, exact)| !exact));
    assert!(g.locate([0.0, 9.0, 28.5]).is_some_and(|(_, exact)| exact));
    // The ridge crest (4 m) is walkable on top but not reachable from the ground.
    assert!(g.locate([24.0, 4.0, -5.0]).is_some());
}

#[test]
fn walls_cost_more_than_open_ground() {
    let g = field();
    let open = g.locate([-10.0, 0.0, -20.0]).unwrap().0;
    let by_wall = g.locate([0.0, 9.0, 29.25]).unwrap().0;
    assert_eq!(g.cell(open).cost, 1);
    assert!(g.cell(by_wall).cost > 1);
}

#[test]
fn open_ground_is_a_straight_line() {
    let p = path([-10.0, 0.0, -20.0], [10.0, 0.0, -20.0]).unwrap();
    assert_eq!(p.waypoints.len(), 2, "{:?}", p.waypoints);
    assert_eq!(p.waypoints[0], [-10.0, 0.0, -20.0]);
    assert_eq!(p.waypoints[1], [10.0, 0.0, -20.0]);
}

#[test]
fn the_path_goes_around_the_steep_ridge() {
    // The ridge spans x = 20..28 across z = -8..-2 and is too steep to walk.
    let (from, to) = ([24.0, 0.0, -12.0], [24.0, 0.0, 4.0]);
    let p = path(from, to).unwrap();
    assert!(p.waypoints.len() >= 3, "{:?}", p.waypoints);
    // Every leg stays on the ground, off the ridge's footprint.
    for w in p.waypoints.windows(2) {
        for i in 0..=40 {
            let t = i as f32 / 40.0;
            let x = w[0][0] + (w[1][0] - w[0][0]) * t;
            let z = w[0][2] + (w[1][2] - w[0][2]) * t;
            assert!(
                !((20.0..=28.0).contains(&x) && (-8.0..=-2.0).contains(&z)),
                "leg crosses the ridge at ({x}, {z}): {:?}",
                p.waypoints
            );
        }
    }
    assert!(length(&p) > 16.0 + 4.0, "must detour: {}", length(&p));
}

#[test]
fn the_path_climbs_the_ramp_and_steps() {
    let up = path([0.0, 0.0, 6.0], [0.0, 9.0, 29.0]).unwrap();
    let last = up.waypoints.last().unwrap();
    assert!((last[1] - 9.0).abs() < 0.3, "{last:?}");
    let steps = path([-23.0, 0.0, 1.0], [-23.0, 1.2, 8.0]).unwrap();
    assert!((steps.waypoints.last().unwrap()[1] - 1.2).abs() < 0.2);
}

#[test]
fn the_ledge_top_cannot_be_walked_to() {
    // 1 m above the ramp landing: needs a jump, so there is no walkable path. The search
    // either exhausts the reachable cells or runs out of budget; it never returns a path.
    let r = path([0.0, 9.0, 29.0], [0.0, 10.0, 35.0]);
    assert!(
        matches!(r, Err(PathError::NoPath | PathError::BudgetExceeded { .. })),
        "{r:?}"
    );
}

#[test]
fn off_grid_positions_fail_cleanly() {
    assert_eq!(
        path([500.0, 0.0, 0.0], [0.0, 0.0, 0.0]).unwrap_err(),
        PathError::StartOffGrid
    );
    assert_eq!(
        path([0.0, 0.0, 0.0], [0.0, 0.0, 500.0]).unwrap_err(),
        PathError::GoalOffGrid
    );
    assert_eq!(
        path([f32::NAN, 0.0, 0.0], [0.0, 0.0, 0.0]).unwrap_err(),
        PathError::StartOffGrid
    );
}

#[test]
fn a_request_over_budget_fails_without_a_partial_path() {
    let g = field();
    let (from, to) = ([-28.0, 0.0, -28.0], [28.0, 0.0, 10.0]);
    // Plenty of budget: found.
    let full = g.find_path(&mut Search::new(), from, to, 512).unwrap();
    assert!(full.expanded > 20);
    // Too little: a clean error that says how much it used.
    let err = g.find_path(&mut Search::new(), from, to, 20).unwrap_err();
    assert_eq!(err, PathError::BudgetExceeded { expanded: 20 });
    // Exactly enough works, one fewer doesn't.
    let ok = g
        .find_path(&mut Search::new(), from, to, full.expanded)
        .unwrap();
    assert_eq!(ok, full);
    assert!(matches!(
        g.find_path(&mut Search::new(), from, to, full.expanded - 1),
        Err(PathError::BudgetExceeded { .. })
    ));
}

#[test]
fn reusing_the_search_gives_the_same_answers() {
    let g = field();
    let mut s = Search::new();
    let a = g.find_path(&mut s, [-10.0, 0.0, -20.0], [10.0, 0.0, -10.0], 512);
    let _ = g.find_path(&mut s, [24.0, 0.0, -12.0], [24.0, 0.0, 4.0], 512);
    let b = g.find_path(&mut s, [-10.0, 0.0, -20.0], [10.0, 0.0, -10.0], 512);
    assert_eq!(a, b);
}

#[test]
fn the_queue_answers_four_a_tick_oldest_first() {
    let g = field();
    let cfg = NavConfig::embedded().unwrap();
    let mut q = NavQueue::new(&cfg);
    let tickets: Vec<u32> = (0..6)
        .map(|i| {
            q.request([-10.0 + i as f32, 0.0, -20.0], [10.0, 0.0, -20.0])
                .unwrap()
        })
        .collect();
    assert_eq!(q.pending(), 6);
    let first = q.run_tick(g);
    assert_eq!(
        first.iter().map(|r| r.ticket).collect::<Vec<_>>(),
        tickets[..4]
    );
    assert!(first.iter().all(|r| r.result.is_ok()));
    assert_eq!(q.pending(), 2);
    let second = q.run_tick(g);
    assert_eq!(
        second.iter().map(|r| r.ticket).collect::<Vec<_>>(),
        tickets[4..]
    );
    assert!(q.run_tick(g).is_empty());
}

#[test]
fn the_queue_refuses_requests_when_full() {
    let cfg = NavConfig::embedded().unwrap();
    let mut q = NavQueue::new(&cfg);
    for _ in 0..cfg.max_pending {
        q.request([0.0; 3], [1.0; 3]).unwrap();
    }
    assert_eq!(q.request([0.0; 3], [1.0; 3]), Err(PathError::QueueFull));
}

#[test]
fn the_queue_serializes_without_its_scratch() {
    let cfg = NavConfig::embedded().unwrap();
    let mut q = NavQueue::new(&cfg);
    q.request([0.0; 3], [1.0; 3]).unwrap();
    let bytes = postcard::to_allocvec(&q).unwrap();
    let back: NavQueue = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(back.pending(), 1);
}

/// Two 1-cell-wide floors stacked in one column pair, joined only by a link (a ladder).
#[test]
fn stacked_layers_connect_through_links() {
    let cfg = NavConfig::embedded().unwrap();
    let cell = |mm| NavCell {
        floor_mm: mm,
        cost: 1,
    };
    // Row of 4 columns along X. Columns 0..2 hold a ground cell; column 2 and 3 hold a
    // ground cell and an upper floor at 4 m; the upper floor in column 3 is the goal.
    let cols = vec![
        vec![cell(0)],
        vec![cell(0)],
        vec![cell(0), cell(4000)],
        vec![cell(0), cell(4000)],
    ];
    // Cell ids: 0, 1, 2(ground), 3(upper), 4(ground), 5(upper).
    let link = NavLink {
        from: 2,
        to: 3,
        cost: 40,
    };
    let g = NavGrid::from_columns(cfg, [0.0, 0.0], 4, 1, &cols, vec![link]);
    // Without the link the upper floor is unreachable; with it, the path climbs.
    let up = g
        .find_path(
            &mut Search::new(),
            [0.25, 0.0, 0.25],
            [1.75, 4.0, 0.25],
            512,
        )
        .unwrap();
    assert_eq!(up.waypoints.last().unwrap()[1], 4.0);
    // The link edge is kept as a waypoint (smoothing can't cut across layers).
    assert!(up.waypoints.iter().any(|w| w[1] == 0.0 && w[0] == 1.25));
    let g_no_link = NavGrid::from_columns(
        NavConfig::embedded().unwrap(),
        [0.0, 0.0],
        4,
        1,
        &cols,
        Vec::new(),
    );
    assert_eq!(
        g_no_link
            .find_path(
                &mut Search::new(),
                [0.25, 0.0, 0.25],
                [1.75, 4.0, 0.25],
                512
            )
            .unwrap_err(),
        PathError::NoPath
    );
}
