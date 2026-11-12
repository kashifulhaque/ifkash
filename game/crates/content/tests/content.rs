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
    assert_eq!(dir.consumables, embedded.consumables);
    assert!(embedded.consumables.is_some());
    assert!(embedded.scenario("m0_flat").is_some());
    assert!(embedded.scenario("test_field").is_some());
    assert_eq!(
        dir.enemies.keys().collect::<Vec<_>>(),
        embedded.enemies.keys().collect::<Vec<_>>()
    );
    assert_eq!(dir.ai, embedded.ai);
    assert!(embedded.ai.is_some());
    assert_eq!(dir.kits, embedded.kits);
    assert_eq!(
        dir.levels.keys().collect::<Vec<_>>(),
        embedded.levels.keys().collect::<Vec<_>>()
    );
    assert_eq!(dir.levels, embedded.levels);
}

#[test]
fn hash_ignores_comments_and_whitespace() {
    let base = Content::load_embedded().expect("loads");
    let registry = std::fs::read_to_string(data_dir().join("registry.ron")).unwrap();
    let noisy = format!("// extra comment\n\n{}\n\n", registry.replace(",", " ,  "));
    let attrs = std::fs::read_to_string(data_dir().join("attributes.ron")).unwrap();
    let moves = std::fs::read_to_string(data_dir().join("moves/straight_sword.ron")).unwrap();
    let common = std::fs::read_to_string(data_dir().join("moves/common.ron")).unwrap();
    let crits = std::fs::read_to_string(data_dir().join("moves/crits.ron")).unwrap();
    let bow_moves = std::fs::read_to_string(data_dir().join("moves/stub_bow.ron")).unwrap();
    // Every .ron file in a data subdirectory, as (relative path, text), sorted.
    let ron_files = |sub: &str| -> Vec<(String, String)> {
        let mut names: Vec<String> = std::fs::read_dir(data_dir().join(sub))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".ron"))
            .collect();
        names.sort();
        names
            .into_iter()
            .map(|n| {
                let text = std::fs::read_to_string(data_dir().join(sub).join(&n)).unwrap();
                (format!("{sub}/{n}"), text)
            })
            .collect()
    };
    let test_enemy = std::fs::read_to_string(data_dir().join("moves/test_enemy.ron")).unwrap();
    let mut all_moves = vec![
        ("moves/common.ron".to_owned(), common),
        ("moves/crits.ron".to_owned(), crits),
        ("moves/straight_sword.ron".to_owned(), moves),
        ("moves/stub_bow.ron".to_owned(), bow_moves),
        ("moves/test_enemy.ron".to_owned(), test_enemy),
    ];
    // WP-051: the six class movesets.
    for class in [
        "dagger",
        "greatsword",
        "curved_sword",
        "thrusting_sword",
        "axe",
        "hammer",
    ] {
        let text = std::fs::read_to_string(data_dir().join(format!("moves/{class}.ron"))).unwrap();
        all_moves.push((format!("moves/{class}.ron"), text));
    }
    // WP-052: the other classes' movesets.
    for class in [
        "colossal", "spear", "halberd", "fist", "bow", "shield", "catalyst",
    ] {
        let text = std::fs::read_to_string(data_dir().join(format!("moves/{class}.ron"))).unwrap();
        all_moves.push((format!("moves/{class}.ron"), text));
    }
    all_moves.extend(ron_files("moves/enemies"));
    all_moves.extend(ron_files("moves/bosses"));
    // The Arts' moves (WP-054).
    let arts_moves = std::fs::read_to_string(data_dir().join("moves/arts.ron")).unwrap();
    all_moves.push(("moves/arts.ron".to_owned(), arts_moves));
    let other = Content::from_sources(
        &noisy,
        &attrs,
        all_moves,
        // Every weapon file (WP-053 added the wpn_* files), each with a noise comment.
        ron_files("weapons")
            .into_iter()
            .map(|(p, t)| (p, format!("// noise\n{t}")))
            .collect(),
    )
    .and_then(|c| {
        let read = |p: &str| std::fs::read_to_string(data_dir().join(p)).unwrap();
        let scenarios: Vec<(String, String)> = ron_files("scenarios")
            .into_iter()
            .map(|(p, t)| (p, format!("// noise\n{t}")))
            .collect();
        c.with_world(Some(&read("locomotion.ron")), scenarios)?
            .with_consumables(Some(&format!("// noise\n{}", read("consumables.ron"))))?
            .with_projectiles(Some(&format!("// noise\n{}", read("projectiles.ron"))))?
            .with_status(Some(&format!("// noise\n{}", read("status.ron"))))?
            .with_charms(Some(&format!("// noise\n{}", read("charms.ron"))))?
            .with_armor(Some(&format!("// noise\n{}", read("armor.ron"))))?
            .with_arts(Some(&format!("// noise\n{}", read("arts.ron"))))?
            .with_upgrades(Some(&format!("// noise\n{}", read("upgrades.ron"))))?
            .with_spells(
                ron_files("spells")
                    .into_iter()
                    .map(|(p, t)| (p, format!("// noise\n{t}")))
                    .collect(),
            )?
            .with_enemies(
                Some(&format!("// noise\n{}", read("ai.ron"))),
                ron_files("enemies")
                    .into_iter()
                    .map(|(p, t)| (p, format!("// noise\n{t}")))
                    .collect(),
            )?
            .with_bosses(
                ron_files("bosses")
                    .into_iter()
                    .map(|(p, t)| (p, format!("// noise\n{t}")))
                    .collect(),
            )?
            .with_echoes(
                ron_files("echoes")
                    .into_iter()
                    .map(|(p, t)| (p, format!("// noise\n{t}")))
                    .collect(),
            )?
            .with_npcs(
                ron_files("npcs")
                    .into_iter()
                    .map(|(p, t)| (p, format!("// noise\n{t}")))
                    .collect(),
            )?
            .with_levels(
                ron_files("kits")
                    .into_iter()
                    .map(|(p, t)| (p, format!("// noise\n{t}")))
                    .collect(),
                ron_files("levels/test_field")
                    .into_iter()
                    .chain(ron_files("levels/mechanics_yard"))
                    .chain(ron_files("levels/waking_hollow"))
                    .chain(ron_files("levels/verge"))
                    .chain(ron_files("levels/hollowhearth"))
                    .map(|(p, t)| (p, format!("// noise\n{t}")))
                    .collect(),
            )
    })
    .expect("loads");
    assert_eq!(base.content_hash(), other.content_hash());
}

#[test]
fn projectile_spawns_are_validated() {
    let mut c = Content::load_embedded().unwrap();
    let m = c
        .moves
        .get_mut("moves/stub_bow.ron")
        .and_then(|v| v.first_mut())
        .unwrap();
    m.projectiles[0].projectile = "proj_nope".to_owned();
    m.charge = Some(lf_content::moves::Charge {
        charge_from: m.projectiles[0].frame,
        max_frames: 10,
        motion_value_max: 1.0,
    });
    let issues = c.validate();
    let has = |field: &str| issues.iter().any(|i| i.field.starts_with(field));
    assert!(has("projectiles[0].projectile"), "{issues:?}");
    assert!(
        issues
            .iter()
            .any(|i| i.field == "projectiles[0].frame" && i.message.contains("charge_from")),
        "{issues:?}"
    );
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

#[test]
fn enemy_attacks_must_telegraph_for_12_frames() {
    // The shipped enemies pass (`real_data_has_no_errors`). Shorten the first hitbox of
    // the Husk's follow-up swing, which only a chained move reaches, and of a projectile
    // move, and the validator names each one.
    let mut c = Content::load_embedded().unwrap();
    let moves = c.moves.get_mut("moves/enemies/husk.ron").unwrap();
    let swipe_2 = moves.iter_mut().find(|m| m.id == "husk_swipe_2").unwrap();
    swipe_2.hitboxes[0].frames = (11, 14);
    let moves = c.moves.get_mut("moves/enemies/gray_archer.ron").unwrap();
    let shoot = moves.iter_mut().find(|m| m.id == "ga_shoot").unwrap();
    shoot.projectiles[0].frame = 5;
    let errors: Vec<String> = c
        .validate()
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| i.to_string())
        .collect();
    // The Husk and the Servant share husk_swipe_2.
    assert_eq!(errors.len(), 3, "{errors:#?}");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("husk_swipe_2") && e.contains("11"))
    );
    assert!(
        errors
            .iter()
            .any(|e| e.contains("ga_shoot") && e.contains("5"))
    );
}

/// Every issue the validator reports for `item`, with its message.
fn issues_for(content: &Content, item: &str) -> Vec<String> {
    content
        .validate()
        .into_iter()
        .filter(|i| i.severity == Severity::Error && i.item == item)
        .map(|i| format!("{}: {}", i.field, i.message))
        .collect()
}

fn weapon_mut<'a>(content: &'a mut Content, id: &str) -> &'a mut lf_content::Weapon {
    content
        .weapons
        .values_mut()
        .flatten()
        .find(|w| w.id == id)
        .expect("weapon exists")
}

#[test]
fn a_weapon_must_take_its_moves_from_its_class() {
    let mut c = Content::load_embedded().expect("loads");
    // A dagger that swings the straight sword's light attack.
    weapon_mut(&mut c, "wpn_fungal_knife").moveset.light_chain = vec!["ss_light_1".to_owned()];
    let issues = issues_for(&c, "wpn_fungal_knife");
    assert!(
        issues
            .iter()
            .any(|m| m.contains("moves/straight_sword.ron") && m.contains("Dagger")),
        "{issues:#?}"
    );
}

#[test]
fn a_weapon_must_name_an_art_its_class_can_use() {
    let mut c = Content::load_embedded().expect("loads");
    // Bastion is a shield's Art, so a dagger can't use it.
    weapon_mut(&mut c, "wpn_fungal_knife").default_art = Some("art_bastion".to_owned());
    let issues = issues_for(&c, "wpn_fungal_knife");
    assert!(
        issues
            .iter()
            .any(|m| m.contains("default_art") && m.contains("can't be used by a Dagger")),
        "{issues:#?}"
    );
}

#[test]
fn a_weapon_must_name_an_art_unless_it_is_a_catalyst() {
    let mut c = Content::load_embedded().expect("loads");
    weapon_mut(&mut c, "wpn_fungal_knife").default_art = None;
    assert!(
        issues_for(&c, "wpn_fungal_knife")
            .iter()
            .any(|m| m.contains("needs an Art")),
    );
    // A catalyst casts spells, so it can't name an Art.
    weapon_mut(&mut c, "wpn_scholars_stylus").default_art = Some("art_feint_step".to_owned());
    assert!(
        issues_for(&c, "wpn_scholars_stylus")
            .iter()
            .any(|m| m.contains("has no Art")),
    );
}

#[test]
fn an_art_must_be_known_and_list_the_class() {
    let mut c = Content::load_embedded().expect("loads");
    weapon_mut(&mut c, "wpn_fungal_knife").default_art = Some("art_not_yet".to_owned());
    assert!(
        issues_for(&c, "wpn_fungal_knife")
            .iter()
            .any(|m| m.contains("unknown Art \"art_not_yet\"")),
    );
}

#[test]
fn every_real_weapon_passes_the_class_moveset_and_art_checks() {
    let c = Content::load_embedded().expect("loads");
    let errors: Vec<_> = c
        .validate()
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    // Every weapon that isn't a test weapon names an Art, unless it is a catalyst.
    for w in c
        .all_weapons()
        .map(|(_, w)| w)
        .filter(|w| !w.id.starts_with("wpn_test_"))
    {
        let catalyst = w.class == lf_content::weapons::WeaponClass::Catalyst;
        assert_eq!(catalyst, w.default_art.is_none(), "{}", w.id);
    }
}
