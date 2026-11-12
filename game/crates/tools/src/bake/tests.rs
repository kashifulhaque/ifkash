use std::path::PathBuf;

use lf_content::level::Entity;
use lf_content::scenario::{EnemySpawn, Lantern};
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
