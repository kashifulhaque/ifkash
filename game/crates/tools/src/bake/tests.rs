use std::path::PathBuf;

use lf_content::level::Entity;
use lf_content::scenario::{Darkness, EnemySpawn, Lantern, Zone};
use lf_content::{Content, Kits, Level, NavConfig};

use super::area::{Baked, bake_level};
use super::warnings::WarningKind;

fn data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn bake_with(content: &Content, edit: impl FnOnce(&mut Level)) -> Baked {
    let mut level = content.level("test_field").expect("test field").clone();
    edit(&mut level);
    let kits = Kits::new(content.kits.iter().map(|(f, c)| (f.as_str(), c))).unwrap();
    let nav = NavConfig::load_dir(data()).unwrap();
    bake_level("test_field", &level, &kits, &nav).unwrap()
}

fn kinds(b: &Baked) -> Vec<WarningKind> {
    b.warnings.iter().map(|w| w.kind).collect()
}

#[test]
fn the_test_field_bakes_cleanly_and_matches_the_committed_manifest() {
    let content = Content::load_dir(data()).unwrap();
    let baked = bake_with(&content, |_| {});
    assert!(baked.warnings.is_empty(), "{:?}", baked.warnings);
    assert_eq!(Some(&baked.scenario), content.scenario("test_field"));
    let committed = std::fs::read_to_string(super::default_manifest_dir().join("test_field.json"))
        .expect("static/latticefall/levels/test_field.json exists");
    assert!(
        committed == baked.manifest,
        "the committed manifest is stale: run `cargo run -p lf-tools -- bake test_field`"
    );
}

#[test]
fn problem_entities_produce_warnings() {
    let content = Content::load_dir(data()).unwrap();
    let baked = bake_with(&content, |level| {
        let es = &mut level.entities.entities;
        // On top of the steep ridge (x = 20..28, z = -6..-4, 4 m up): walkable, but its
        // 63 degree sides can't be climbed.
        es.push(Entity::Lantern(Lantern {
            id: "lantern_ridge".into(),
            pos: [24.0, 4.0, -5.0],
            yaw: 0.0,
        }));
        // Inside the ledge box.
        es.push(Entity::Enemy(EnemySpawn {
            name: "buried".into(),
            enemy: "husk".into(),
            pos: [0.0, 3.0, 35.0],
            yaw: 0.0,
            patrol: vec![],
            wakes_on: None,
        }));
        // Out past the terrain's edge, over the void.
        es.push(Entity::Lantern(Lantern {
            id: "lantern_void".into(),
            pos: [0.0, 0.0, 80.0],
            yaw: 0.0,
        }));
    });
    let k = kinds(&baked);
    let msg = |kind| {
        baked
            .warnings
            .iter()
            .filter(|w| w.kind == kind)
            .map(|w| w.message.as_str())
            .collect::<Vec<_>>()
    };
    assert_eq!(msg(WarningKind::UnreachableLantern).len(), 1, "{k:?}");
    assert!(msg(WarningKind::UnreachableLantern)[0].contains("lantern_ridge"));
    assert!(msg(WarningKind::InsideCollider)[0].contains("\"buried\""));
    assert!(msg(WarningKind::InsideCollider)[0].contains("ledge"));
    assert!(
        msg(WarningKind::OffNavGrid)
            .iter()
            .any(|m| m.contains("lantern_void"))
    );
    assert!(
        msg(WarningKind::OutsideBounds)
            .iter()
            .any(|m| m.contains("lantern_void"))
    );
}

#[test]
fn a_lantern_on_reachable_ground_is_fine() {
    let content = Content::load_dir(data()).unwrap();
    // The top of the ramp, 9 m up: reachable by walking.
    let baked = bake_with(&content, |level| {
        level.entities.entities.push(Entity::Lantern(Lantern {
            id: "lantern_ramp_top".into(),
            pos: [0.0, 9.0, 29.0],
            yaw: 0.0,
        }));
    });
    assert!(baked.warnings.is_empty(), "{:?}", baked.warnings);
}

fn dark_rule() -> Entity {
    Entity::Darkness(Darkness {
        visibility_m: 6.0,
        torch_visibility_m: 14.0,
        lantern_light_m: 9.0,
        torch_flag: "codex.projects.smoltorch".into(),
    })
}

/// WP-062: a dark area's walk from the spawn through its Lanterns must stay in glowing zones.
/// The test field has Lanterns far from its spawn, so with no glow the walk goes dark; a glow
/// over the whole field lights it.
#[test]
fn a_dark_critical_path_is_a_warning_until_glowing_zones_cover_it() {
    let content = Content::load_dir(data()).unwrap();
    // A second Lantern at the top of the ramp, 30 m on: the walk between the two Lanterns is dark.
    let far_lantern = || {
        Entity::Lantern(Lantern {
            id: "lantern_ramp_top".into(),
            pos: [0.0, 9.0, 29.0],
            yaw: 0.0,
        })
    };
    let dark = bake_with(&content, |level| {
        level.entities.entities.push(dark_rule());
        level.entities.entities.push(far_lantern());
    });
    assert!(
        kinds(&dark).contains(&WarningKind::DarkCriticalPath),
        "{:?}",
        dark.warnings
    );
    let lit = bake_with(&content, |level| {
        let b = level.area.bounds;
        level.entities.entities.push(dark_rule());
        level.entities.entities.push(far_lantern());
        level.entities.entities.push(Entity::Glow(Zone {
            name: "everywhere".into(),
            center: [(b.min[0] + b.max[0]) / 2.0, (b.min[1] + b.max[1]) / 2.0],
            half_extents: [(b.max[0] - b.min[0]) / 2.0, (b.max[1] - b.min[1]) / 2.0],
        }));
    });
    assert!(
        !kinds(&lit).contains(&WarningKind::DarkCriticalPath),
        "{:?}",
        lit.warnings
    );
    // A level without a darkness rule is never dark.
    let plain = bake_with(&content, |_| {});
    assert!(!kinds(&plain).contains(&WarningKind::DarkCriticalPath));
}

/// WP-062: Hollowdeep and the Cinderdeep Mine bake with no warnings, which includes the rule that
/// Hollowdeep's critical path stays inside its glowing zones, and the committed manifests are
/// current.
#[test]
fn the_underground_areas_bake_cleanly() {
    let content = Content::load_dir(data()).unwrap();
    let kits = Kits::new(content.kits.iter().map(|(f, c)| (f.as_str(), c))).unwrap();
    let nav = NavConfig::load_dir(data()).unwrap();
    for area in ["hollowdeep", "cinderdeep_mine"] {
        let level = content.level(area).expect("the area exists");
        let baked = bake_level(area, level, &kits, &nav).unwrap();
        assert!(baked.warnings.is_empty(), "{area}: {:?}", baked.warnings);
        let committed =
            std::fs::read_to_string(super::default_manifest_dir().join(format!("{area}.json")))
                .unwrap_or_else(|_| panic!("static/latticefall/levels/{area}.json exists"));
        assert!(
            committed == baked.manifest,
            "the committed manifest is stale: run `cargo run -p lf-tools -- bake {area}`"
        );
    }
}

/// WP-063: Mossgut Cave and Glasswing Grotto bake with no warnings (their pickups stand clear of
/// the roof slabs, and every mirror has ground at its foot), and the committed manifests are
/// current.
#[test]
fn the_caves_bake_cleanly() {
    let content = Content::load_dir(data()).unwrap();
    let kits = Kits::new(content.kits.iter().map(|(f, c)| (f.as_str(), c))).unwrap();
    let nav = NavConfig::load_dir(data()).unwrap();
    for area in ["mossgut_cave", "glasswing_grotto"] {
        let level = content.level(area).expect("the area exists");
        let baked = bake_level(area, level, &kits, &nav).unwrap();
        assert!(baked.warnings.is_empty(), "{area}: {:?}", baked.warnings);
        let committed =
            std::fs::read_to_string(super::default_manifest_dir().join(format!("{area}.json")))
                .unwrap_or_else(|_| panic!("static/latticefall/levels/{area}.json exists"));
        assert!(
            committed == baked.manifest,
            "the committed manifest is stale: run `cargo run -p lf-tools -- bake {area}`"
        );
    }
}

/// WP-066: Rimeshelf, the Whitewarden Catacombs, and the Bound Ring bake with no warnings (every
/// Lantern is reachable, no enemy or item stands in a collider or under the terrain), and the
/// committed manifests are current.
#[test]
fn the_rimeshelf_areas_bake_cleanly() {
    let content = Content::load_dir(data()).unwrap();
    let kits = Kits::new(content.kits.iter().map(|(f, c)| (f.as_str(), c))).unwrap();
    let nav = NavConfig::load_dir(data()).unwrap();
    for area in ["rimeshelf", "whitewarden", "bound_ring"] {
        let level = content.level(area).expect("the area exists");
        let baked = bake_level(area, level, &kits, &nav).unwrap();
        assert!(baked.warnings.is_empty(), "{area}: {:?}", baked.warnings);
        let committed =
            std::fs::read_to_string(super::default_manifest_dir().join(format!("{area}.json")))
                .unwrap_or_else(|_| panic!("static/latticefall/levels/{area}.json exists"));
        assert!(
            committed == baked.manifest,
            "the committed manifest is stale: run `cargo run -p lf-tools -- bake {area}`"
        );
    }
}
