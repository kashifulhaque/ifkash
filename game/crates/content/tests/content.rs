use std::path::{Path, PathBuf};

use lf_content::{Content, ContentError, Severity, Validate};

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn real_data_has_no_errors() {
    let content = Content::load_dir(data_dir()).expect("loads");
    let issues = content.validate();
    let errors: Vec<_> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn embedded_and_dir_hashes_match() {
    let dir = Content::load_dir(data_dir()).expect("dir loads");
    let embedded = Content::load_embedded().expect("embedded loads");
    assert_eq!(dir, embedded);
    assert_eq!(dir.content_hash(), embedded.content_hash());
}

#[test]
fn embedded_matches_data_dir() {
    // Every .ron file under data/moves, data/weapons, and data/scenarios must be embedded.
    let dir = Content::load_dir(data_dir()).expect("dir loads");
    let embedded = Content::load_embedded().expect("embedded loads");
    assert_eq!(
        dir.moves.keys().collect::<Vec<_>>(),
        embedded.moves.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        dir.weapons.keys().collect::<Vec<_>>(),
        embedded.weapons.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        dir.scenarios.keys().collect::<Vec<_>>(),
        embedded.scenarios.keys().collect::<Vec<_>>()
    );
    assert!(embedded.locomotion.is_some());
    assert!(embedded.scenario("m0_flat").is_some());
    assert!(embedded.scenario("test_field").is_some());
}

#[test]
fn hash_ignores_comments_and_whitespace() {
    let base = Content::load_embedded().expect("loads");
    let registry = std::fs::read_to_string(data_dir().join("registry.ron")).unwrap();
    let noisy = format!("// extra comment\n\n{}\n\n", registry.replace(",", " ,  "));
    let attrs = std::fs::read_to_string(data_dir().join("attributes.ron")).unwrap();
    let moves = std::fs::read_to_string(data_dir().join("moves/straight_sword.ron")).unwrap();
    let common = std::fs::read_to_string(data_dir().join("moves/common.ron")).unwrap();
    let weapons = std::fs::read_to_string(data_dir().join("weapons/straight_sword.ron")).unwrap();
    let other = Content::from_sources(
        &noisy,
        &attrs,
        vec![
            ("moves/common.ron".into(), common),
            ("moves/straight_sword.ron".into(), moves),
        ],
        vec![("weapons/straight_sword.ron".into(), weapons)],
    )
    .and_then(|c| {
        let read = |p: &str| std::fs::read_to_string(data_dir().join(p)).unwrap();
        let scenarios = ["scenarios/m0_flat.ron", "scenarios/test_field.ron"]
            .map(|p| (p.to_owned(), format!("// noise\n{}", read(p))))
            .to_vec();
        c.with_world(Some(&read("locomotion.ron")), scenarios)
    })
    .expect("loads");
    assert_eq!(base.content_hash(), other.content_hash());
}

#[test]
fn hash_changes_with_data() {
    let a = Content::load_embedded().unwrap();
    let mut b = a.clone();
    b.attributes.max_hp.base_tenths += 1;
    assert_ne!(a.content_hash(), b.content_hash());
}

#[test]
fn broken_fixture_fails_validation_with_names() {
    let content = Content::load_dir(fixture("broken_frames")).expect("fixture loads");
    let issues = content.validate();
    let hit = issues
        .iter()
        .find(|i| i.severity == Severity::Error && i.field.starts_with("hitboxes[0].frames"))
        .expect("an error on hitboxes[0].frames");
    let text = hit.to_string();
    assert!(text.contains("moves/straight_sword.ron"), "{text}");
    assert!(text.contains("ss_broken"), "{text}");
    assert!(text.contains("hitboxes[0].frames"), "{text}");
    println!("{text}");
}

fn sources_with(registry: &str, moves: &str) -> Result<Content, ContentError> {
    let attrs = std::fs::read_to_string(data_dir().join("attributes.ron")).unwrap();
    Content::from_sources(
        registry,
        &attrs,
        vec![("moves/test.ron".into(), moves.into())],
        vec![],
    )
}

const ATTR_REG: &str = r#"(id: "attr_vitality", num: 100), (id: "attr_focus", num: 101),
    (id: "attr_endurance", num: 102), (id: "attr_might", num: 103),
    (id: "attr_finesse", num: 104), (id: "attr_insight", num: 105),
    (id: "attr_devotion", num: 106),"#;

#[test]
fn loader_rejects_duplicate_numbers_and_ids() {
    let dup_num = format!(r#"(entries: [{ATTR_REG} (id: "a", num: 5), (id: "b", num: 5)])"#);
    let err = sources_with(&dup_num, "[]").unwrap_err();
    assert!(matches!(err, ContentError::Registry(_)), "{err}");
    let dup_id = format!(r#"(entries: [{ATTR_REG} (id: "a", num: 5), (id: "a", num: 6)])"#);
    let err = sources_with(&dup_id, "[]").unwrap_err();
    assert!(matches!(err, ContentError::Registry(_)), "{err}");
}

#[test]
fn loader_rejects_unregistered_ids() {
    let reg = format!(r#"(entries: [{ATTR_REG}])"#);
    let moves = r#"[(id: "ghost", anim: "x", total_frames: 10, stamina: 0, fp: 0,
        root_motion: [], track_deg_per_tick: 0.0, hyperarmor: None, iframes: None,
        cancel_attack_from: None, cancel_dodge_from: None, cancel_any_from: None,
        next_chain: None, hitboxes: [], sfx: [], vfx: [])]"#;
    let err = sources_with(&reg, moves).unwrap_err();
    assert!(
        matches!(&err, ContentError::Unregistered { id, .. } if id == "ghost"),
        "{err}"
    );
}

#[test]
fn validator_collects_all_issues_and_warns_on_stubs() {
    let reg = format!(r#"(entries: [{ATTR_REG} (id: "m1", num: 1), (id: "m2", num: 2)])"#);
    let moves = r#"[
      (id: "m1", anim: "x", total_frames: 10, stamina: 0, fp: 0,
        root_motion: [(12, 0.0)], track_deg_per_tick: 0.0,
        hyperarmor: Some((from: 2, to: 11)), iframes: None,
        cancel_attack_from: Some(30), cancel_dodge_from: None, cancel_any_from: None,
        next_chain: Some("m2"), hitboxes: [], sfx: [(11, "s")], vfx: [], stub: true),
      (id: "m2", anim: "x", total_frames: 10, stamina: 0, fp: 0,
        root_motion: [], track_deg_per_tick: 0.0, hyperarmor: None, iframes: None,
        cancel_attack_from: None, cancel_dodge_from: None, cancel_any_from: None,
        next_chain: None, hitboxes: [], sfx: [], vfx: [])]"#;
    let c = sources_with(&reg, moves).unwrap();
    let issues = c.validate();
    let fields: Vec<&str> = issues.iter().map(|i| i.field.as_str()).collect();
    for want in [
        "root_motion[0].frame",
        "hyperarmor.to",
        "cancel_attack_from",
        "sfx[0].frame",
        "stub",
    ] {
        assert!(fields.contains(&want), "missing {want} in {fields:?}");
    }
    let stub = issues.iter().find(|i| i.field == "stub").unwrap();
    assert_eq!(stub.severity, Severity::Warning);
}

#[test]
fn validator_reports_unknown_chain_target() {
    // Registered but never defined.
    let reg = format!(r#"(entries: [{ATTR_REG} (id: "m1", num: 1), (id: "nowhere", num: 2)])"#);
    let moves = r#"[(id: "m1", anim: "x", total_frames: 10, stamina: 0, fp: 0,
        root_motion: [], track_deg_per_tick: 0.0, hyperarmor: None, iframes: None,
        cancel_attack_from: None, cancel_dodge_from: None, cancel_any_from: None,
        next_chain: Some("nowhere"), hitboxes: [], sfx: [], vfx: [])]"#;
    let issues = sources_with(&reg, moves).unwrap().validate();
    assert!(
        issues
            .iter()
            .any(|i| i.field == "next_chain" && i.severity == Severity::Error)
    );
}

#[test]
fn derived_stats_match_the_spec_table() {
    let a = Content::load_embedded().unwrap().attributes;
    // (stat, value at 10, value at 40) from docs/game/03-combat.md "Derived stats".
    // Values are exact tenths; the f32 conversion is exact to well under 1e-4.
    assert_eq!((a.max_hp_tenths(10), a.max_hp_tenths(40)), (5700, 13700));
    assert_eq!((a.max_fp_tenths(10), a.max_fp_tenths(40)), (850, 1950));
    assert_eq!(
        (a.max_stamina_tenths(10), a.max_stamina_tenths(40)),
        (980, 1380)
    );
    assert_eq!(
        (a.max_equip_load_tenths(10), a.max_equip_load_tenths(40)),
        (508, 868)
    );
    assert_eq!((a.spell_slots(10), a.spell_slots(40)), (2, 5));
    assert_eq!(
        (a.resistance_tenths(10), a.resistance_tenths(40)),
        (1100, 2000)
    );
    let close = |got: i32, want: f32| (Attributes::tenths_to_f32(got) - want).abs() < 1e-4;
    assert!(close(a.max_equip_load_tenths(10), 50.8));
    assert!(close(a.max_equip_load_tenths(40), 86.8));
    // Edges: the curve starts at attribute 1 and the cap is 60.
    assert_eq!(a.max_hp_tenths(1), 3000);
    assert_eq!(a.max_hp_tenths(60), 13700 + 20 * 80);
    assert_eq!(a.max_hp_tenths(99), a.max_hp_tenths(60));
}

use lf_content::Attributes;
