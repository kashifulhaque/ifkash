//! `lf-content`: content data types, loader, and validation.
//!
//! Content is RON in `game/data/`. [`Content::load_dir`] reads it from disk (Python
//! build, tools) and [`Content::load_embedded`] bakes the same files in with
//! `include_str!` (WASM build). Both produce the same [`Content::content_hash`].
//!
//! Content types use only `Vec` and `BTreeMap` so the hash is stable. This crate must
//! not depend on `lf-core`; the core depends on it.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub mod arts;
pub mod attributes;
pub mod boss;
pub mod consumables;
pub mod echo;
pub mod enemy;
pub mod gear;
pub mod level;
pub mod locomotion;
pub mod moves;
pub mod nav;
pub mod npc;
pub mod portfolio;
pub mod projectiles;
pub mod registry;
pub mod scenario;
pub mod spells;
pub mod status;
pub mod upgrades;
pub mod validate;
pub mod weapons;

pub use arts::{Art, ArtFile};
pub use attributes::Attributes;
pub use boss::BossDef;
pub use consumables::{Consumables, Flask, Item, ItemEffect, ItemKind};
pub use echo::{EchoDef, EchoRules};
pub use enemy::{AiTuning, EnemyDef};
pub use gear::{ArmorFile, ArmorPiece, Charm, GearEffect};
pub use level::{KitCatalog, Kits, Level};
pub use locomotion::{Death, EquipLoad, Guard, LockOn, Locomotion, Poise};
pub use moves::Move;
pub use nav::NavConfig;
pub use npc::{NpcDef, StationDef};
pub use portfolio::{Portfolio, PortfolioContent};
pub use projectiles::{Homing, ProjectileDef};
pub use registry::{NumId, Registry};
pub use scenario::{Lantern, Pickup, Scenario};
pub use spells::{SpellDef, SpellRules, SpellSchool};
pub use status::StatusRules;
pub use upgrades::{ForgeCost, Temper, UpgradeRules};
pub use validate::{Issue, Severity, Validate};
pub use weapons::{GuardStats, Weapon};

use registry::RegistryFile;

/// Directory (relative to the data root) holding move files.
const MOVES_DIR: &str = "moves";
/// Directory (relative to the data root) holding weapon files.
const WEAPONS_DIR: &str = "weapons";
/// Directory (relative to the data root) holding scenario files.
const SCENARIOS_DIR: &str = "scenarios";
/// The locomotion tuning file (relative to the data root).
const LOCOMOTION_FILE: &str = "locomotion.ron";
/// The consumables file (relative to the data root).
const CONSUMABLES_FILE: &str = "consumables.ron";
/// The projectile definitions file (relative to the data root).
const PROJECTILES_FILE: &str = "projectiles.ron";
/// The status and buff rules file (relative to the data root).
const STATUS_FILE: &str = "status.ron";
/// The charms file (relative to the data root).
const CHARMS_FILE: &str = "charms.ron";
/// The armor file (relative to the data root).
const ARMOR_FILE: &str = "armor.ron";
/// The Arts file (relative to the data root).
const ARTS_FILE: &str = "arts.ron";
/// The upgrade and temper rules file (relative to the data root).
const UPGRADES_FILE: &str = "upgrades.ron";
/// Directory (relative to the data root) holding spell files.
const SPELLS_DIR: &str = "spells";
/// Directory (relative to the data root) holding enemy definitions, one per file.
const ENEMIES_DIR: &str = "enemies";
/// Directory (relative to the data root) holding boss definitions, one per file.
const BOSSES_DIR: &str = "bosses";
/// Directory (relative to the data root) holding Echo definitions, one per file, and
/// `rules.ron`.
const ECHOES_DIR: &str = "echoes";
/// The Echo rules file (relative to the data root).
const ECHO_RULES_FILE: &str = "echoes/rules.ron";
/// Directory (relative to the data root) holding NPC definitions, one per file, and
/// `stations.ron` (WP-068).
const NPCS_DIR: &str = "npcs";
/// The shared AI tuning file (relative to the data root).
const AI_FILE: &str = "ai.ron";
/// Directory (relative to the data root) holding kit catalogs, one per file.
const KITS_DIR: &str = "kits";
/// Directory (relative to the data root) holding one directory per level (area).
const LEVELS_DIR: &str = "levels";

/// Embedded copies of every data file, as `(path relative to the data root, text)`.
/// A new data file needs a line here; the `embedded_matches_data_dir` test fails if one
/// is missing.
const EMBEDDED_MOVES: &[(&str, &str)] = &[
    (
        "moves/common.ron",
        include_str!("../../../data/moves/common.ron"),
    ),
    (
        "moves/straight_sword.ron",
        include_str!("../../../data/moves/straight_sword.ron"),
    ),
    (
        "moves/dagger.ron",
        include_str!("../../../data/moves/dagger.ron"),
    ),
    (
        "moves/greatsword.ron",
        include_str!("../../../data/moves/greatsword.ron"),
    ),
    (
        "moves/curved_sword.ron",
        include_str!("../../../data/moves/curved_sword.ron"),
    ),
    (
        "moves/thrusting_sword.ron",
        include_str!("../../../data/moves/thrusting_sword.ron"),
    ),
    ("moves/axe.ron", include_str!("../../../data/moves/axe.ron")),
    (
        "moves/hammer.ron",
        include_str!("../../../data/moves/hammer.ron"),
    ),
    (
        "moves/colossal.ron",
        include_str!("../../../data/moves/colossal.ron"),
    ),
    (
        "moves/spear.ron",
        include_str!("../../../data/moves/spear.ron"),
    ),
    (
        "moves/halberd.ron",
        include_str!("../../../data/moves/halberd.ron"),
    ),
    (
        "moves/fist.ron",
        include_str!("../../../data/moves/fist.ron"),
    ),
    ("moves/bow.ron", include_str!("../../../data/moves/bow.ron")),
    (
        "moves/shield.ron",
        include_str!("../../../data/moves/shield.ron"),
    ),
    (
        "moves/catalyst.ron",
        include_str!("../../../data/moves/catalyst.ron"),
    ),
    (
        "moves/crits.ron",
        include_str!("../../../data/moves/crits.ron"),
    ),
    (
        "moves/stub_bow.ron",
        include_str!("../../../data/moves/stub_bow.ron"),
    ),
    (
        "moves/test_enemy.ron",
        include_str!("../../../data/moves/test_enemy.ron"),
    ),
    (
        "moves/enemies/husk.ron",
        include_str!("../../../data/moves/enemies/husk.ron"),
    ),
    (
        "moves/enemies/thorn_hound.ron",
        include_str!("../../../data/moves/enemies/thorn_hound.ron"),
    ),
    (
        "moves/enemies/gray_knight.ron",
        include_str!("../../../data/moves/enemies/gray_knight.ron"),
    ),
    (
        "moves/enemies/gray_archer.ron",
        include_str!("../../../data/moves/enemies/gray_archer.ron"),
    ),
    (
        "moves/enemies/whitewarden_skeleton.ron",
        include_str!("../../../data/moves/enemies/whitewarden_skeleton.ron"),
    ),
    (
        "moves/enemies/acolyte.ron",
        include_str!("../../../data/moves/enemies/acolyte.ron"),
    ),
    (
        "moves/enemies/drowned.ron",
        include_str!("../../../data/moves/enemies/drowned.ron"),
    ),
    (
        "moves/enemies/reed_stalker.ron",
        include_str!("../../../data/moves/enemies/reed_stalker.ron"),
    ),
    (
        "moves/enemies/marsh_crawler.ron",
        include_str!("../../../data/moves/enemies/marsh_crawler.ron"),
    ),
    (
        "moves/enemies/cave_moth.ron",
        include_str!("../../../data/moves/enemies/cave_moth.ron"),
    ),
    (
        "moves/enemies/lamp_mite.ron",
        include_str!("../../../data/moves/enemies/lamp_mite.ron"),
    ),
    (
        "moves/enemies/shard_golem.ron",
        include_str!("../../../data/moves/enemies/shard_golem.ron"),
    ),
    (
        "moves/enemies/glasswing_stalker.ron",
        include_str!("../../../data/moves/enemies/glasswing_stalker.ron"),
    ),
    (
        "moves/enemies/acolyte_bell.ron",
        include_str!("../../../data/moves/enemies/acolyte_bell.ron"),
    ),
    (
        "moves/enemies/lantern_thief.ron",
        include_str!("../../../data/moves/enemies/lantern_thief.ron"),
    ),
    (
        "moves/enemies/rime_wolf.ron",
        include_str!("../../../data/moves/enemies/rime_wolf.ron"),
    ),
    (
        "moves/enemies/mimic_chest.ron",
        include_str!("../../../data/moves/enemies/mimic_chest.ron"),
    ),
    (
        "moves/enemies/bramble_brute.ron",
        include_str!("../../../data/moves/enemies/bramble_brute.ron"),
    ),
    (
        "moves/enemies/choir_knight.ron",
        include_str!("../../../data/moves/enemies/choir_knight.ron"),
    ),
    (
        "moves/enemies/gray_captain.ron",
        include_str!("../../../data/moves/enemies/gray_captain.ron"),
    ),
    (
        "moves/enemies/foundry_smith.ron",
        include_str!("../../../data/moves/enemies/foundry_smith.ron"),
    ),
    (
        "moves/enemies/bell_warden.ron",
        include_str!("../../../data/moves/enemies/bell_warden.ron"),
    ),
    (
        "moves/bosses/test_boss.ron",
        include_str!("../../../data/moves/bosses/test_boss.ron"),
    ),
    (
        "moves/bosses/b01.ron",
        include_str!("../../../data/moves/bosses/b01.ron"),
    ),
    (
        "moves/arts.ron",
        include_str!("../../../data/moves/arts.ron"),
    ),
];
const EMBEDDED_WEAPONS: &[(&str, &str)] = &[
    (
        "weapons/straight_sword.ron",
        include_str!("../../../data/weapons/straight_sword.ron"),
    ),
    (
        "weapons/stub_bow.ron",
        include_str!("../../../data/weapons/stub_bow.ron"),
    ),
    (
        "weapons/stylus_reliquary.ron",
        include_str!("../../../data/weapons/stylus_reliquary.ron"),
    ),
    (
        "weapons/test_classes.ron",
        include_str!("../../../data/weapons/test_classes.ron"),
    ),
    (
        "weapons/wpn_daggers.ron",
        include_str!("../../../data/weapons/wpn_daggers.ron"),
    ),
    (
        "weapons/wpn_blades.ron",
        include_str!("../../../data/weapons/wpn_blades.ron"),
    ),
    (
        "weapons/wpn_heavy.ron",
        include_str!("../../../data/weapons/wpn_heavy.ron"),
    ),
    (
        "weapons/wpn_reach.ron",
        include_str!("../../../data/weapons/wpn_reach.ron"),
    ),
    (
        "weapons/wpn_hands.ron",
        include_str!("../../../data/weapons/wpn_hands.ron"),
    ),
    (
        "weapons/wpn_catalysts.ron",
        include_str!("../../../data/weapons/wpn_catalysts.ron"),
    ),
];
const EMBEDDED_REGISTRY: &str = include_str!("../../../data/registry.ron");
const EMBEDDED_ATTRIBUTES: &str = include_str!("../../../data/attributes.ron");
const EMBEDDED_LOCOMOTION: &str = include_str!("../../../data/locomotion.ron");
const EMBEDDED_CONSUMABLES: &str = include_str!("../../../data/consumables.ron");
/// Embedded spell files, as `(spells/<file>, text)`. `litanies.ron` is WP-056's.
const EMBEDDED_SPELLS: &[(&str, &str)] = &[
    (
        "spells/formulae.ron",
        include_str!("../../../data/spells/formulae.ron"),
    ),
    (
        "spells/litanies.ron",
        include_str!("../../../data/spells/litanies.ron"),
    ),
];
const EMBEDDED_PROJECTILES: &str = include_str!("../../../data/projectiles.ron");
const EMBEDDED_STATUS: &str = include_str!("../../../data/status.ron");
const EMBEDDED_CHARMS: &str = include_str!("../../../data/charms.ron");
const EMBEDDED_ARMOR: &str = include_str!("../../../data/armor.ron");
const EMBEDDED_ARTS: &str = include_str!("../../../data/arts.ron");
const EMBEDDED_UPGRADES: &str = include_str!("../../../data/upgrades.ron");
const EMBEDDED_SCENARIOS: &[(&str, &str)] = &[
    (
        "scenarios/m0_flat.ron",
        include_str!("../../../data/scenarios/m0_flat.ron"),
    ),
    (
        "scenarios/m2_range.ron",
        include_str!("../../../data/scenarios/m2_range.ron"),
    ),
    (
        "scenarios/duel_parry.ron",
        include_str!("../../../data/scenarios/duel_parry.ron"),
    ),
    (
        "scenarios/duel_stance.ron",
        include_str!("../../../data/scenarios/duel_stance.ron"),
    ),
    (
        "scenarios/duel_backstab.ron",
        include_str!("../../../data/scenarios/duel_backstab.ron"),
    ),
    (
        "scenarios/duel_status.ron",
        include_str!("../../../data/scenarios/duel_status.ron"),
    ),
    (
        "scenarios/ai_yard.ron",
        include_str!("../../../data/scenarios/ai_yard.ron"),
    ),
    (
        "scenarios/fight_husk.ron",
        include_str!("../../../data/scenarios/fight_husk.ron"),
    ),
    (
        "scenarios/fight_nav_wall.ron",
        include_str!("../../../data/scenarios/fight_nav_wall.ron"),
    ),
    (
        "scenarios/fight_husk_variants.ron",
        include_str!("../../../data/scenarios/fight_husk_variants.ron"),
    ),
    (
        "scenarios/fight_thorn_hounds.ron",
        include_str!("../../../data/scenarios/fight_thorn_hounds.ron"),
    ),
    (
        "scenarios/fight_gray_knight.ron",
        include_str!("../../../data/scenarios/fight_gray_knight.ron"),
    ),
    (
        "scenarios/fight_gray_archer.ron",
        include_str!("../../../data/scenarios/fight_gray_archer.ron"),
    ),
    (
        "scenarios/fight_skeleton.ron",
        include_str!("../../../data/scenarios/fight_skeleton.ron"),
    ),
    (
        "scenarios/fight_acolyte.ron",
        include_str!("../../../data/scenarios/fight_acolyte.ron"),
    ),
    (
        "scenarios/fight_drowned.ron",
        include_str!("../../../data/scenarios/fight_drowned.ron"),
    ),
    (
        "scenarios/fight_reed_stalker.ron",
        include_str!("../../../data/scenarios/fight_reed_stalker.ron"),
    ),
    (
        "scenarios/fight_marsh_crawlers.ron",
        include_str!("../../../data/scenarios/fight_marsh_crawlers.ron"),
    ),
    (
        "scenarios/fight_cave_moth.ron",
        include_str!("../../../data/scenarios/fight_cave_moth.ron"),
    ),
    (
        "scenarios/fight_lamp_mites.ron",
        include_str!("../../../data/scenarios/fight_lamp_mites.ron"),
    ),
    (
        "scenarios/fight_shard_golem.ron",
        include_str!("../../../data/scenarios/fight_shard_golem.ron"),
    ),
    (
        "scenarios/fight_glasswing.ron",
        include_str!("../../../data/scenarios/fight_glasswing.ron"),
    ),
    (
        "scenarios/fight_acolyte_bell.ron",
        include_str!("../../../data/scenarios/fight_acolyte_bell.ron"),
    ),
    (
        "scenarios/fight_lantern_thief.ron",
        include_str!("../../../data/scenarios/fight_lantern_thief.ron"),
    ),
    (
        "scenarios/fight_rime_wolves.ron",
        include_str!("../../../data/scenarios/fight_rime_wolves.ron"),
    ),
    (
        "scenarios/fight_mimic.ron",
        include_str!("../../../data/scenarios/fight_mimic.ron"),
    ),
    (
        "scenarios/fight_bramble_brute.ron",
        include_str!("../../../data/scenarios/fight_bramble_brute.ron"),
    ),
    (
        "scenarios/fight_choir_knight.ron",
        include_str!("../../../data/scenarios/fight_choir_knight.ron"),
    ),
    (
        "scenarios/fight_gray_captain.ron",
        include_str!("../../../data/scenarios/fight_gray_captain.ron"),
    ),
    (
        "scenarios/fight_foundry_smith.ron",
        include_str!("../../../data/scenarios/fight_foundry_smith.ron"),
    ),
    (
        "scenarios/fight_bell_warden.ron",
        include_str!("../../../data/scenarios/fight_bell_warden.ron"),
    ),
    (
        "scenarios/spell_range.ron",
        include_str!("../../../data/scenarios/spell_range.ron"),
    ),
    (
        "scenarios/spell_ward.ron",
        include_str!("../../../data/scenarios/spell_ward.ron"),
    ),
    (
        "scenarios/litany_range.ron",
        include_str!("../../../data/scenarios/litany_range.ron"),
    ),
    (
        "scenarios/litany_ward.ron",
        include_str!("../../../data/scenarios/litany_ward.ron"),
    ),
    (
        "scenarios/litany_hounds.ron",
        include_str!("../../../data/scenarios/litany_hounds.ron"),
    ),
    (
        "scenarios/litany_echo.ron",
        include_str!("../../../data/scenarios/litany_echo.ron"),
    ),
    (
        "scenarios/boss_gate.ron",
        include_str!("../../../data/scenarios/boss_gate.ron"),
    ),
    (
        "scenarios/boss_b01.ron",
        include_str!("../../../data/scenarios/boss_b01.ron"),
    ),
    (
        "scenarios/wp051_dagger.ron",
        include_str!("../../../data/scenarios/wp051_dagger.ron"),
    ),
    (
        "scenarios/wp051_greatsword.ron",
        include_str!("../../../data/scenarios/wp051_greatsword.ron"),
    ),
    (
        "scenarios/wp051_curved_sword.ron",
        include_str!("../../../data/scenarios/wp051_curved_sword.ron"),
    ),
    (
        "scenarios/wp051_thrusting_sword.ron",
        include_str!("../../../data/scenarios/wp051_thrusting_sword.ron"),
    ),
    (
        "scenarios/wp051_axe.ron",
        include_str!("../../../data/scenarios/wp051_axe.ron"),
    ),
    (
        "scenarios/wp051_hammer.ron",
        include_str!("../../../data/scenarios/wp051_hammer.ron"),
    ),
    (
        "scenarios/echo_boss_gate.ron",
        include_str!("../../../data/scenarios/echo_boss_gate.ron"),
    ),
    (
        "scenarios/wp052_colossal.ron",
        include_str!("../../../data/scenarios/wp052_colossal.ron"),
    ),
    (
        "scenarios/wp052_spear.ron",
        include_str!("../../../data/scenarios/wp052_spear.ron"),
    ),
    (
        "scenarios/wp052_halberd.ron",
        include_str!("../../../data/scenarios/wp052_halberd.ron"),
    ),
    (
        "scenarios/wp052_fist.ron",
        include_str!("../../../data/scenarios/wp052_fist.ron"),
    ),
    (
        "scenarios/wp052_bow.ron",
        include_str!("../../../data/scenarios/wp052_bow.ron"),
    ),
    (
        "scenarios/wp052_shield.ron",
        include_str!("../../../data/scenarios/wp052_shield.ron"),
    ),
    (
        "scenarios/wp052_catalyst.ron",
        include_str!("../../../data/scenarios/wp052_catalyst.ron"),
    ),
    (
        "scenarios/m5_art_feint_step.ron",
        include_str!("../../../data/scenarios/m5_art_feint_step.ron"),
    ),
    (
        "scenarios/m5_art_rending_sweep.ron",
        include_str!("../../../data/scenarios/m5_art_rending_sweep.ron"),
    ),
    (
        "scenarios/m5_art_bastion.ron",
        include_str!("../../../data/scenarios/m5_art_bastion.ron"),
    ),
    (
        "scenarios/m5_art_windward_parry.ron",
        include_str!("../../../data/scenarios/m5_art_windward_parry.ron"),
    ),
    (
        "scenarios/m5_art_rising_cleave.ron",
        include_str!("../../../data/scenarios/m5_art_rising_cleave.ron"),
    ),
    (
        "scenarios/m5_art_rallying_roar.ron",
        include_str!("../../../data/scenarios/m5_art_rallying_roar.ron"),
    ),
    (
        "scenarios/m5_art_lunging_thrust.ron",
        include_str!("../../../data/scenarios/m5_art_lunging_thrust.ron"),
    ),
    (
        "scenarios/m5_art_earthsplit.ron",
        include_str!("../../../data/scenarios/m5_art_earthsplit.ron"),
    ),
    (
        "scenarios/m5_art_rime_edge.ron",
        include_str!("../../../data/scenarios/m5_art_rime_edge.ron"),
    ),
    (
        "scenarios/m5_art_radiant_blade.ron",
        include_str!("../../../data/scenarios/m5_art_radiant_blade.ron"),
    ),
    (
        "scenarios/m5_art_spark_volley.ron",
        include_str!("../../../data/scenarios/m5_art_spark_volley.ron"),
    ),
    (
        "scenarios/m5_art_lantern_flare.ron",
        include_str!("../../../data/scenarios/m5_art_lantern_flare.ron"),
    ),
    (
        "scenarios/m5_art_glass_barrage.ron",
        include_str!("../../../data/scenarios/m5_art_glass_barrage.ron"),
    ),
    (
        "scenarios/m5_art_toll.ron",
        include_str!("../../../data/scenarios/m5_art_toll.ron"),
    ),
];
const EMBEDDED_BOSSES: &[(&str, &str)] = &[
    (
        "bosses/test_boss.ron",
        include_str!("../../../data/bosses/test_boss.ron"),
    ),
    (
        "bosses/b01_hollow_sentry.ron",
        include_str!("../../../data/bosses/b01_hollow_sentry.ron"),
    ),
];
const EMBEDDED_ECHOES: &[(&str, &str)] = &[
    (
        "echoes/rules.ron",
        include_str!("../../../data/echoes/rules.ron"),
    ),
    (
        "echoes/echo_politellama.ron",
        include_str!("../../../data/echoes/echo_politellama.ron"),
    ),
    (
        "echoes/echo_bone_sentinel.ron",
        include_str!("../../../data/echoes/echo_bone_sentinel.ron"),
    ),
    (
        "echoes/echo_choir_wisp.ron",
        include_str!("../../../data/echoes/echo_choir_wisp.ron"),
    ),
    (
        "echoes/echo_briar_hounds.ron",
        include_str!("../../../data/echoes/echo_briar_hounds.ron"),
    ),
];
const EMBEDDED_NPCS: &[(&str, &str)] = &[
    (
        "npcs/archivist.ron",
        include_str!("../../../data/npcs/archivist.ron"),
    ),
    (
        "npcs/cartographer.ron",
        include_str!("../../../data/npcs/cartographer.ron"),
    ),
    (
        "npcs/smith.ron",
        include_str!("../../../data/npcs/smith.ron"),
    ),
    (
        "npcs/courier.ron",
        include_str!("../../../data/npcs/courier.ron"),
    ),
    (
        "npcs/peddler.ron",
        include_str!("../../../data/npcs/peddler.ron"),
    ),
    (
        "npcs/stations.ron",
        include_str!("../../../data/npcs/stations.ron"),
    ),
];
const EMBEDDED_AI: &str = include_str!("../../../data/ai.ron");
const EMBEDDED_ENEMIES: &[(&str, &str)] = &[
    (
        "enemies/test_grunt.ron",
        include_str!("../../../data/enemies/test_grunt.ron"),
    ),
    (
        "enemies/husk.ron",
        include_str!("../../../data/enemies/husk.ron"),
    ),
    (
        "enemies/husk_servant.ron",
        include_str!("../../../data/enemies/husk_servant.ron"),
    ),
    (
        "enemies/husk_miner.ron",
        include_str!("../../../data/enemies/husk_miner.ron"),
    ),
    (
        "enemies/husk_frost.ron",
        include_str!("../../../data/enemies/husk_frost.ron"),
    ),
    (
        "enemies/thorn_hound.ron",
        include_str!("../../../data/enemies/thorn_hound.ron"),
    ),
    (
        "enemies/gray_knight.ron",
        include_str!("../../../data/enemies/gray_knight.ron"),
    ),
    (
        "enemies/gray_archer.ron",
        include_str!("../../../data/enemies/gray_archer.ron"),
    ),
    (
        "enemies/whitewarden_skeleton.ron",
        include_str!("../../../data/enemies/whitewarden_skeleton.ron"),
    ),
    (
        "enemies/acolyte.ron",
        include_str!("../../../data/enemies/acolyte.ron"),
    ),
    (
        "enemies/drowned.ron",
        include_str!("../../../data/enemies/drowned.ron"),
    ),
    (
        "enemies/reed_stalker.ron",
        include_str!("../../../data/enemies/reed_stalker.ron"),
    ),
    (
        "enemies/marsh_crawler.ron",
        include_str!("../../../data/enemies/marsh_crawler.ron"),
    ),
    (
        "enemies/cave_moth.ron",
        include_str!("../../../data/enemies/cave_moth.ron"),
    ),
    (
        "enemies/lamp_mite.ron",
        include_str!("../../../data/enemies/lamp_mite.ron"),
    ),
    (
        "enemies/shard_golem.ron",
        include_str!("../../../data/enemies/shard_golem.ron"),
    ),
    (
        "enemies/glasswing_stalker.ron",
        include_str!("../../../data/enemies/glasswing_stalker.ron"),
    ),
    (
        "enemies/acolyte_bell.ron",
        include_str!("../../../data/enemies/acolyte_bell.ron"),
    ),
    (
        "enemies/lantern_thief.ron",
        include_str!("../../../data/enemies/lantern_thief.ron"),
    ),
    (
        "enemies/rime_wolf.ron",
        include_str!("../../../data/enemies/rime_wolf.ron"),
    ),
    (
        "enemies/mimic_chest.ron",
        include_str!("../../../data/enemies/mimic_chest.ron"),
    ),
    (
        "enemies/bramble_brute.ron",
        include_str!("../../../data/enemies/bramble_brute.ron"),
    ),
    (
        "enemies/choir_knight.ron",
        include_str!("../../../data/enemies/choir_knight.ron"),
    ),
    (
        "enemies/gray_captain.ron",
        include_str!("../../../data/enemies/gray_captain.ron"),
    ),
    (
        "enemies/foundry_smith.ron",
        include_str!("../../../data/enemies/foundry_smith.ron"),
    ),
    (
        "enemies/bell_warden.ron",
        include_str!("../../../data/enemies/bell_warden.ron"),
    ),
    // WP-063: the elite Glasswing Stalker.
    (
        "enemies/glasswing_stalker_elite.ron",
        include_str!("../../../data/enemies/glasswing_stalker_elite.ron"),
    ),
];

const EMBEDDED_KITS: &[(&str, &str)] = &[
    (
        "kits/greybox.ron",
        include_str!("../../../data/kits/greybox.ron"),
    ),
    (
        "kits/meadow.ron",
        include_str!("../../../data/kits/meadow.ron"),
    ),
];
/// Level files as `(levels/<area>/<file>, text)`.
const EMBEDDED_LEVELS: &[(&str, &str)] = &[
    (
        "levels/verge/area.ron",
        include_str!("../../../data/levels/verge/area.ron"),
    ),
    (
        "levels/verge/terrain.ron",
        include_str!("../../../data/levels/verge/terrain.ron"),
    ),
    (
        "levels/verge/layout.ron",
        include_str!("../../../data/levels/verge/layout.ron"),
    ),
    (
        "levels/verge/entities.ron",
        include_str!("../../../data/levels/verge/entities.ron"),
    ),
    (
        "levels/verge/dressing.ron",
        include_str!("../../../data/levels/verge/dressing.ron"),
    ),
    (
        "levels/mirrormere/area.ron",
        include_str!("../../../data/levels/mirrormere/area.ron"),
    ),
    (
        "levels/mirrormere/terrain.ron",
        include_str!("../../../data/levels/mirrormere/terrain.ron"),
    ),
    (
        "levels/mirrormere/layout.ron",
        include_str!("../../../data/levels/mirrormere/layout.ron"),
    ),
    (
        "levels/mirrormere/entities.ron",
        include_str!("../../../data/levels/mirrormere/entities.ron"),
    ),
    (
        "levels/mirrormere/dressing.ron",
        include_str!("../../../data/levels/mirrormere/dressing.ron"),
    ),
    (
        "levels/cinderdeep_mine/area.ron",
        include_str!("../../../data/levels/cinderdeep_mine/area.ron"),
    ),
    (
        "levels/cinderdeep_mine/terrain.ron",
        include_str!("../../../data/levels/cinderdeep_mine/terrain.ron"),
    ),
    (
        "levels/cinderdeep_mine/layout.ron",
        include_str!("../../../data/levels/cinderdeep_mine/layout.ron"),
    ),
    (
        "levels/cinderdeep_mine/entities.ron",
        include_str!("../../../data/levels/cinderdeep_mine/entities.ron"),
    ),
    (
        "levels/cinderdeep_mine/dressing.ron",
        include_str!("../../../data/levels/cinderdeep_mine/dressing.ron"),
    ),
    (
        "levels/hollowdeep/area.ron",
        include_str!("../../../data/levels/hollowdeep/area.ron"),
    ),
    (
        "levels/hollowdeep/terrain.ron",
        include_str!("../../../data/levels/hollowdeep/terrain.ron"),
    ),
    (
        "levels/hollowdeep/layout.ron",
        include_str!("../../../data/levels/hollowdeep/layout.ron"),
    ),
    (
        "levels/hollowdeep/entities.ron",
        include_str!("../../../data/levels/hollowdeep/entities.ron"),
    ),
    (
        "levels/hollowdeep/dressing.ron",
        include_str!("../../../data/levels/hollowdeep/dressing.ron"),
    ),
    (
        "levels/waking_hollow/area.ron",
        include_str!("../../../data/levels/waking_hollow/area.ron"),
    ),
    (
        "levels/waking_hollow/layout.ron",
        include_str!("../../../data/levels/waking_hollow/layout.ron"),
    ),
    (
        "levels/waking_hollow/entities.ron",
        include_str!("../../../data/levels/waking_hollow/entities.ron"),
    ),
    (
        "levels/mechanics_yard/area.ron",
        include_str!("../../../data/levels/mechanics_yard/area.ron"),
    ),
    (
        "levels/mechanics_yard/terrain.ron",
        include_str!("../../../data/levels/mechanics_yard/terrain.ron"),
    ),
    (
        "levels/mechanics_yard/layout.ron",
        include_str!("../../../data/levels/mechanics_yard/layout.ron"),
    ),
    (
        "levels/mechanics_yard/entities.ron",
        include_str!("../../../data/levels/mechanics_yard/entities.ron"),
    ),
    (
        "levels/test_field/area.ron",
        include_str!("../../../data/levels/test_field/area.ron"),
    ),
    (
        "levels/test_field/terrain.ron",
        include_str!("../../../data/levels/test_field/terrain.ron"),
    ),
    (
        "levels/test_field/layout.ron",
        include_str!("../../../data/levels/test_field/layout.ron"),
    ),
    (
        "levels/test_field/entities.ron",
        include_str!("../../../data/levels/test_field/entities.ron"),
    ),
    (
        "levels/test_field/dressing.ron",
        include_str!("../../../data/levels/test_field/dressing.ron"),
    ),
    (
        "levels/hollowhearth/area.ron",
        include_str!("../../../data/levels/hollowhearth/area.ron"),
    ),
    (
        "levels/hollowhearth/layout.ron",
        include_str!("../../../data/levels/hollowhearth/layout.ron"),
    ),
    (
        "levels/hollowhearth/entities.ron",
        include_str!("../../../data/levels/hollowhearth/entities.ron"),
    ),
    // WP-061: the Mirrormere mini-dungeons.
    (
        "levels/sunken_ossuary/area.ron",
        include_str!("../../../data/levels/sunken_ossuary/area.ron"),
    ),
    (
        "levels/sunken_ossuary/terrain.ron",
        include_str!("../../../data/levels/sunken_ossuary/terrain.ron"),
    ),
    (
        "levels/sunken_ossuary/layout.ron",
        include_str!("../../../data/levels/sunken_ossuary/layout.ron"),
    ),
    (
        "levels/sunken_ossuary/entities.ron",
        include_str!("../../../data/levels/sunken_ossuary/entities.ron"),
    ),
    (
        "levels/drowned_belfry/area.ron",
        include_str!("../../../data/levels/drowned_belfry/area.ron"),
    ),
    (
        "levels/drowned_belfry/terrain.ron",
        include_str!("../../../data/levels/drowned_belfry/terrain.ron"),
    ),
    (
        "levels/drowned_belfry/layout.ron",
        include_str!("../../../data/levels/drowned_belfry/layout.ron"),
    ),
    (
        "levels/drowned_belfry/entities.ron",
        include_str!("../../../data/levels/drowned_belfry/entities.ron"),
    ),
    (
        "levels/castle/area.ron",
        include_str!("../../../data/levels/castle/area.ron"),
    ),
    (
        "levels/castle/terrain.ron",
        include_str!("../../../data/levels/castle/terrain.ron"),
    ),
    (
        "levels/castle/layout.ron",
        include_str!("../../../data/levels/castle/layout.ron"),
    ),
    (
        "levels/castle/entities.ron",
        include_str!("../../../data/levels/castle/entities.ron"),
    ),
    (
        "levels/spire/area.ron",
        include_str!("../../../data/levels/spire/area.ron"),
    ),
    (
        "levels/spire/terrain.ron",
        include_str!("../../../data/levels/spire/terrain.ron"),
    ),
    (
        "levels/spire/layout.ron",
        include_str!("../../../data/levels/spire/layout.ron"),
    ),
    (
        "levels/spire/entities.ron",
        include_str!("../../../data/levels/spire/entities.ron"),
    ),
    // WP-063: the two caves.
    (
        "levels/mossgut_cave/area.ron",
        include_str!("../../../data/levels/mossgut_cave/area.ron"),
    ),
    (
        "levels/mossgut_cave/terrain.ron",
        include_str!("../../../data/levels/mossgut_cave/terrain.ron"),
    ),
    (
        "levels/mossgut_cave/layout.ron",
        include_str!("../../../data/levels/mossgut_cave/layout.ron"),
    ),
    (
        "levels/mossgut_cave/entities.ron",
        include_str!("../../../data/levels/mossgut_cave/entities.ron"),
    ),
    (
        "levels/glasswing_grotto/area.ron",
        include_str!("../../../data/levels/glasswing_grotto/area.ron"),
    ),
    (
        "levels/glasswing_grotto/terrain.ron",
        include_str!("../../../data/levels/glasswing_grotto/terrain.ron"),
    ),
    (
        "levels/glasswing_grotto/layout.ron",
        include_str!("../../../data/levels/glasswing_grotto/layout.ron"),
    ),
    (
        "levels/glasswing_grotto/entities.ron",
        include_str!("../../../data/levels/glasswing_grotto/entities.ron"),
    ),
    // WP-066: Rimeshelf, the Whitewarden Catacombs, and the Bound Ring.
    (
        "levels/rimeshelf/area.ron",
        include_str!("../../../data/levels/rimeshelf/area.ron"),
    ),
    (
        "levels/rimeshelf/terrain.ron",
        include_str!("../../../data/levels/rimeshelf/terrain.ron"),
    ),
    (
        "levels/rimeshelf/layout.ron",
        include_str!("../../../data/levels/rimeshelf/layout.ron"),
    ),
    (
        "levels/rimeshelf/entities.ron",
        include_str!("../../../data/levels/rimeshelf/entities.ron"),
    ),
    (
        "levels/whitewarden/area.ron",
        include_str!("../../../data/levels/whitewarden/area.ron"),
    ),
    (
        "levels/whitewarden/terrain.ron",
        include_str!("../../../data/levels/whitewarden/terrain.ron"),
    ),
    (
        "levels/whitewarden/layout.ron",
        include_str!("../../../data/levels/whitewarden/layout.ron"),
    ),
    (
        "levels/whitewarden/entities.ron",
        include_str!("../../../data/levels/whitewarden/entities.ron"),
    ),
    (
        "levels/bound_ring/area.ron",
        include_str!("../../../data/levels/bound_ring/area.ron"),
    ),
    (
        "levels/bound_ring/terrain.ron",
        include_str!("../../../data/levels/bound_ring/terrain.ron"),
    ),
    (
        "levels/bound_ring/layout.ron",
        include_str!("../../../data/levels/bound_ring/layout.ron"),
    ),
    (
        "levels/bound_ring/entities.ron",
        include_str!("../../../data/levels/bound_ring/entities.ron"),
    ),
];

/// All parsed content.
///
/// `moves` and `weapons` are keyed by file path relative to the data root (for example
/// `moves/straight_sword.ron`) so validation can name the file an item came from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Content {
    pub registry: Registry,
    pub attributes: Attributes,
    pub moves: BTreeMap<String, Vec<Move>>,
    pub weapons: BTreeMap<String, Vec<Weapon>>,
    /// Player locomotion tuning from `locomotion.ron`. `None` only for partial data
    /// directories (test fixtures); the simulation needs it.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub locomotion: Option<Locomotion>,
    /// Scenarios keyed by file path relative to the data root, for example
    /// `scenarios/m0_flat.ron`, plus every compiled level under `levels/<area>`. Look one up by name with [`Content::scenario`].
    #[cfg_attr(feature = "ts", ts(skip))]
    pub scenarios: BTreeMap<String, Scenario>,
    /// Items and flask numbers from `consumables.ron`. `None` only for partial data
    /// directories (test fixtures); the player has no inventory without it.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub consumables: Option<Consumables>,
    /// Projectile definitions from `projectiles.ron`; empty if the file is missing.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub projectiles: Vec<ProjectileDef>,
    /// Status effect and buff rules from `status.ron`. `None` only for partial data
    /// directories (test fixtures); the simulation needs it.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub status: Option<StatusRules>,
    /// Charms from `charms.ron` (WP-057); empty if the file is missing.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub charms: Vec<Charm>,
    /// Armor pieces from `armor.ron` (WP-057); empty if the file is missing.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub armor: Vec<ArmorPiece>,
    /// Arts from `arts.ron` (WP-053); empty if the file is missing.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub arts: Vec<Art>,
    /// Upgrade and temper rules from `upgrades.ron` (WP-053); `None` if the file is missing.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub upgrades: Option<UpgradeRules>,
    /// Every spell from `spells/*.ron`, in file then list order (WP-055). Their moves,
    /// projectiles, and buffs are also in `moves`, `projectiles`, and `status`.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub spells: Vec<SpellDef>,
    /// The rules every spell shares (cast speed), from the spell file that states them.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub spell_rules: Option<SpellRules>,
    /// The shared AI tuning from `ai.ron`. `None` only for partial data directories;
    /// a scenario with enemies needs it.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub ai: Option<AiTuning>,
    /// Enemy definitions keyed by file path relative to the data root, for example
    /// `enemies/test_grunt.ron`. Look one up by ID with [`Content::enemy_by_id`].
    #[cfg_attr(feature = "ts", ts(skip))]
    pub enemies: BTreeMap<String, EnemyDef>,
    /// Boss definitions keyed by file path relative to the data root, for example
    /// `bosses/test_boss.ron`. Look one up by ID with [`Content::boss_by_id`]. A boss's
    /// body and parts are also in `enemies`, so the AI table builds them like any enemy.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub bosses: BTreeMap<String, BossDef>,
    /// The Echo rules from `echoes/rules.ron` (WP-059).
    #[cfg_attr(feature = "ts", ts(skip))]
    pub echo_rules: Option<EchoRules>,
    /// Echo definitions keyed by file path relative to the data root, for example
    /// `echoes/echo_politellama.ron`. Look one up by ID with [`Content::echo_by_id`]. An
    /// Echo's body is also in `enemies` (so the AI table builds it like any enemy) and its
    /// moves are in `moves`, both under keys that name the Echo's file.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub echoes: BTreeMap<String, EchoDef>,
    /// Kit catalogs keyed by file path relative to the data root, for example
    /// `kits/greybox.ron` (WP-040).
    #[cfg_attr(feature = "ts", ts(skip))]
    pub kits: BTreeMap<String, KitCatalog>,
    /// Levels keyed by directory relative to the data root, for example
    /// `levels/test_field` (WP-040). Each is also compiled into `scenarios` under the
    /// same key, so [`Content::scenario`] finds it by name.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub levels: BTreeMap<String, Level>,
    /// NPC definitions keyed by file path relative to the data root, for example
    /// `npcs/archivist.ron` (WP-068). Look one up by ID with [`Content::npc_by_id`].
    #[cfg_attr(feature = "ts", ts(skip))]
    pub npcs: BTreeMap<String, NpcDef>,
    /// The stations of `npcs/stations.ron` (WP-068), in file order.
    #[cfg_attr(feature = "ts", ts(skip))]
    pub stations: Vec<StationDef>,
}

/// A failure to load content. Validation problems are [`Issue`]s instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    Io {
        path: String,
        message: String,
    },
    Parse {
        file: String,
        message: String,
    },
    Registry(String),
    /// A content item references an ID that has no entry in `registry.ron`.
    Unregistered {
        file: String,
        field: String,
        id: String,
    },
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => write!(f, "cannot read {path}: {message}"),
            Self::Parse { file, message } => write!(f, "{file}: parse error: {message}"),
            Self::Registry(message) => write!(f, "registry.ron: {message}"),
            Self::Unregistered { file, field, id } => write!(
                f,
                "{file}: {field}: ID \"{id}\" is missing from registry.ron"
            ),
        }
    }
}

impl std::error::Error for ContentError {}

fn parse<T: serde::de::DeserializeOwned>(file: &str, text: &str) -> Result<T, ContentError> {
    ron::from_str(text).map_err(|e| ContentError::Parse {
        file: file.to_owned(),
        message: e.to_string(),
    })
}

/// Every file in `levels/<area>/` directories as `(levels/<area>/<file>, text)`, sorted.
/// Only the files in [`level::LEVEL_FILES`] are read. No `levels/` directory is no levels.
fn list_levels(dir: &Path) -> Result<Vec<(String, String)>, ContentError> {
    let io = |path: &Path, e: std::io::Error| ContentError::Io {
        path: path.display().to_string(),
        message: e.to_string(),
    };
    let root = dir.join(LEVELS_DIR);
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut areas = Vec::new();
    for entry in std::fs::read_dir(&root).map_err(|e| io(&root, e))? {
        let entry = entry.map_err(|e| io(&root, e))?;
        if entry.path().is_dir() {
            areas.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    areas.sort();
    let mut out = Vec::new();
    for area in areas {
        for file in level::LEVEL_FILES {
            let path = root.join(&area).join(file);
            if path.is_file() {
                let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
                out.push((format!("{LEVELS_DIR}/{area}/{file}"), text));
            }
        }
    }
    Ok(out)
}

impl Content {
    /// Loads content from a data directory laid out like `game/data/`.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self, ContentError> {
        let dir = dir.as_ref();
        let read = |path: &Path| {
            std::fs::read_to_string(path).map_err(|e| ContentError::Io {
                path: path.display().to_string(),
                message: e.to_string(),
            })
        };
        let list = |sub: &str| -> Result<Vec<(String, String)>, ContentError> {
            let sub_dir = dir.join(sub);
            if sub != MOVES_DIR && sub != WEAPONS_DIR && !sub_dir.is_dir() {
                return Ok(Vec::new());
            }
            let mut names = Vec::new();
            let entries = std::fs::read_dir(&sub_dir).map_err(|e| ContentError::Io {
                path: sub_dir.display().to_string(),
                message: e.to_string(),
            })?;
            for entry in entries {
                let entry = entry.map_err(|e| ContentError::Io {
                    path: sub_dir.display().to_string(),
                    message: e.to_string(),
                })?;
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.ends_with(".ron") {
                    names.push(name);
                } else if sub == MOVES_DIR && entry.path().is_dir() {
                    // One level of subfolders under moves/ (moves/bosses/, ...).
                    let nested = std::fs::read_dir(entry.path()).map_err(|e| ContentError::Io {
                        path: entry.path().display().to_string(),
                        message: e.to_string(),
                    })?;
                    for inner in nested.flatten() {
                        let inner_name = inner.file_name().to_string_lossy().into_owned();
                        if inner_name.ends_with(".ron") {
                            names.push(format!("{name}/{inner_name}"));
                        }
                    }
                }
            }
            names.sort();
            names
                .into_iter()
                .map(|name| Ok((format!("{sub}/{name}"), read(&sub_dir.join(&name))?)))
                .collect()
        };
        let locomotion_path = dir.join(LOCOMOTION_FILE);
        let locomotion = if locomotion_path.is_file() {
            Some(read(&locomotion_path)?)
        } else {
            None
        };
        let consumables_path = dir.join(CONSUMABLES_FILE);
        let consumables = if consumables_path.is_file() {
            Some(read(&consumables_path)?)
        } else {
            None
        };
        let projectiles_path = dir.join(PROJECTILES_FILE);
        let projectiles = if projectiles_path.is_file() {
            Some(read(&projectiles_path)?)
        } else {
            None
        };
        let status_path = dir.join(STATUS_FILE);
        let status = if status_path.is_file() {
            Some(read(&status_path)?)
        } else {
            None
        };
        let charms_path = dir.join(CHARMS_FILE);
        let charms = if charms_path.is_file() {
            Some(read(&charms_path)?)
        } else {
            None
        };
        let armor_path = dir.join(ARMOR_FILE);
        let armor = if armor_path.is_file() {
            Some(read(&armor_path)?)
        } else {
            None
        };
        let arts_path = dir.join(ARTS_FILE);
        let arts = if arts_path.is_file() {
            Some(read(&arts_path)?)
        } else {
            None
        };
        let upgrades_path = dir.join(UPGRADES_FILE);
        let upgrades = if upgrades_path.is_file() {
            Some(read(&upgrades_path)?)
        } else {
            None
        };
        let ai_path = dir.join(AI_FILE);
        let ai = if ai_path.is_file() {
            Some(read(&ai_path)?)
        } else {
            None
        };
        Self::from_sources(
            &read(&dir.join("registry.ron"))?,
            &read(&dir.join("attributes.ron"))?,
            list(MOVES_DIR)?,
            list(WEAPONS_DIR)?,
        )?
        .with_world(locomotion.as_deref(), list(SCENARIOS_DIR)?)?
        .with_consumables(consumables.as_deref())?
        .with_projectiles(projectiles.as_deref())?
        .with_status(status.as_deref())?
        .with_charms(charms.as_deref())?
        .with_armor(armor.as_deref())?
        .with_arts(arts.as_deref())?
        .with_upgrades(upgrades.as_deref())?
        .with_spells(list(SPELLS_DIR)?)?
        .with_enemies(ai.as_deref(), list(ENEMIES_DIR)?)?
        .with_bosses(list(BOSSES_DIR)?)?
        .with_echoes(list(ECHOES_DIR)?)?
        .with_npcs(list(NPCS_DIR)?)?
        .with_levels(list(KITS_DIR)?, list_levels(dir)?)
    }

    /// Loads the content baked into this crate with `include_str!`.
    pub fn load_embedded() -> Result<Self, ContentError> {
        let own = |files: &[(&str, &str)]| {
            files
                .iter()
                .map(|(p, t)| ((*p).to_owned(), (*t).to_owned()))
                .collect()
        };
        Self::from_sources(
            EMBEDDED_REGISTRY,
            EMBEDDED_ATTRIBUTES,
            own(EMBEDDED_MOVES),
            own(EMBEDDED_WEAPONS),
        )?
        .with_world(Some(EMBEDDED_LOCOMOTION), own(EMBEDDED_SCENARIOS))?
        .with_consumables(Some(EMBEDDED_CONSUMABLES))?
        .with_projectiles(Some(EMBEDDED_PROJECTILES))?
        .with_status(Some(EMBEDDED_STATUS))?
        .with_charms(Some(EMBEDDED_CHARMS))?
        .with_armor(Some(EMBEDDED_ARMOR))?
        .with_arts(Some(EMBEDDED_ARTS))?
        .with_upgrades(Some(EMBEDDED_UPGRADES))?
        .with_spells(own(EMBEDDED_SPELLS))?
        .with_enemies(Some(EMBEDDED_AI), own(EMBEDDED_ENEMIES))?
        .with_bosses(own(EMBEDDED_BOSSES))?
        .with_echoes(own(EMBEDDED_ECHOES))?
        .with_npcs(own(EMBEDDED_NPCS))?
        .with_levels(own(EMBEDDED_KITS), own(EMBEDDED_LEVELS))
    }

    /// Builds content from RON text. `moves` and `weapons` are `(relative path, text)`.
    pub fn from_sources(
        registry: &str,
        attributes: &str,
        moves: Vec<(String, String)>,
        weapons: Vec<(String, String)>,
    ) -> Result<Self, ContentError> {
        let registry_file: RegistryFile = parse("registry.ron", registry)?;
        let registry = Registry::from_entries(registry_file.entries)?;
        let attributes: Attributes = parse("attributes.ron", attributes)?;
        let mut move_map = BTreeMap::new();
        for (path, text) in moves {
            let items: Vec<Move> = parse(&path, &text)?;
            move_map.insert(path, items);
        }
        let mut weapon_map = BTreeMap::new();
        for (path, text) in weapons {
            let items: Vec<Weapon> = parse(&path, &text)?;
            weapon_map.insert(path, items);
        }
        let content = Self {
            registry,
            attributes,
            moves: move_map,
            weapons: weapon_map,
            locomotion: None,
            scenarios: BTreeMap::new(),
            consumables: None,
            projectiles: Vec::new(),
            status: None,
            charms: Vec::new(),
            armor: Vec::new(),
            arts: Vec::new(),
            upgrades: None,
            spells: Vec::new(),
            spell_rules: None,
            ai: None,
            enemies: BTreeMap::new(),
            bosses: BTreeMap::new(),
            echo_rules: None,
            echoes: BTreeMap::new(),
            kits: BTreeMap::new(),
            levels: BTreeMap::new(),
            npcs: BTreeMap::new(),
            stations: Vec::new(),
        };
        content.check_registered()?;
        Ok(content)
    }

    /// Adds the simulation world data: locomotion tuning (RON text, if any) and scenarios
    /// as `(relative path, text)`.
    pub fn with_world(
        mut self,
        locomotion: Option<&str>,
        scenarios: Vec<(String, String)>,
    ) -> Result<Self, ContentError> {
        self.locomotion = locomotion
            .map(|text| parse::<Locomotion>(LOCOMOTION_FILE, text))
            .transpose()?;
        for (path, text) in scenarios {
            let scenario: Scenario = parse(&path, &text)?;
            scenario.check().map_err(|message| ContentError::Parse {
                file: path.clone(),
                message,
            })?;
            self.scenarios.insert(path, scenario);
        }
        Ok(self)
    }

    /// Adds the consumables file (RON text, if any). Fails on an item that has no
    /// registry entry.
    pub fn with_consumables(mut self, text: Option<&str>) -> Result<Self, ContentError> {
        self.consumables = text
            .map(|t| parse::<Consumables>(CONSUMABLES_FILE, t))
            .transpose()?;
        if let Some(c) = &self.consumables {
            for (i, item) in c.items.iter().enumerate() {
                self.check_id(CONSUMABLES_FILE, format!("items[{i}].id"), &item.id)?;
            }
            self.check_id(
                CONSUMABLES_FILE,
                "flask.drink_move".to_owned(),
                &c.flask.drink_move,
            )?;
        }
        Ok(self)
    }

    /// Adds the projectile definitions (RON text, if any). Fails on a projectile that has
    /// no registry entry.
    pub fn with_projectiles(mut self, text: Option<&str>) -> Result<Self, ContentError> {
        self.projectiles = text
            .map(|t| parse::<Vec<ProjectileDef>>(PROJECTILES_FILE, t))
            .transpose()?
            .unwrap_or_default();
        for (i, p) in self.projectiles.iter().enumerate() {
            self.check_id(PROJECTILES_FILE, format!("[{i}].id"), &p.id)?;
        }
        Ok(self)
    }

    /// Adds the status rules file (RON text, if any). Fails on a buff that has no
    /// registry entry.
    pub fn with_status(mut self, text: Option<&str>) -> Result<Self, ContentError> {
        self.status = text
            .map(|t| parse::<StatusRules>(STATUS_FILE, t))
            .transpose()?;
        if let Some(s) = &self.status {
            for (field, id) in s.references() {
                self.check_id(STATUS_FILE, field, id)?;
            }
        }
        Ok(self)
    }

    /// Adds the charms file (RON text, if any). Fails on a charm that has no registry entry.
    pub fn with_charms(mut self, text: Option<&str>) -> Result<Self, ContentError> {
        self.charms = text
            .map(|t| parse::<Vec<Charm>>(CHARMS_FILE, t))
            .transpose()?
            .unwrap_or_default();
        for (i, c) in self.charms.iter().enumerate() {
            self.check_id(CHARMS_FILE, format!("[{i}].id"), &c.id)?;
        }
        Ok(self)
    }

    /// Adds spell files as `(spells/<file>, text)` and merges each spell's moves,
    /// projectiles, and buffs into `moves`, `projectiles`, and the status rules. Fails on
    /// an ID that has no registry entry, or on a spell with buffs when there are no
    /// status rules to take them.
    pub fn with_spells(mut self, files: Vec<(String, String)>) -> Result<Self, ContentError> {
        for (path, text) in files {
            let file: spells::SpellFile = parse(&path, &text)?;
            if let Some(r) = file.rules {
                self.spell_rules = Some(r);
            }
            let mut moves = Vec::new();
            for (i, s) in file.spells.into_iter().enumerate() {
                for (field, id) in s.references() {
                    self.check_id(&path, format!("spells[{i}].{field}"), id)?;
                }
                for m in std::iter::once(&s.cast).chain(s.charged.as_ref()) {
                    if let Some(next) = &m.next_chain {
                        self.check_id(&path, format!("{}: next_chain", m.id), next)?;
                    }
                    moves.push(m.clone());
                }
                self.projectiles.extend(s.projectiles.iter().cloned());
                if !s.buffs.is_empty() {
                    let status = self.status.as_mut().ok_or_else(|| ContentError::Parse {
                        file: path.clone(),
                        message: "a spell defines buffs but there is no status.ron".to_owned(),
                    })?;
                    status.buffs.extend(s.buffs.iter().cloned());
                }
                self.spells.push(s);
            }
            self.moves.insert(path, moves);
        }
        Ok(self)
    }

    /// Adds the armor file (RON text, if any). Fails on a piece that has no registry entry.
    pub fn with_armor(mut self, text: Option<&str>) -> Result<Self, ContentError> {
        self.armor = text
            .map(|t| parse::<ArmorFile>(ARMOR_FILE, t))
            .transpose()?
            .map(|f| f.pieces)
            .unwrap_or_default();
        for (i, p) in self.armor.iter().enumerate() {
            self.check_id(ARMOR_FILE, format!("pieces[{i}].id"), &p.id)?;
        }
        Ok(self)
    }

    /// Adds the Arts file (RON text, if any). Fails on an Art with no registry entry.
    pub fn with_arts(mut self, text: Option<&str>) -> Result<Self, ContentError> {
        self.arts = text
            .map(|t| parse::<ArtFile>(ARTS_FILE, t))
            .transpose()?
            .map(|f| f.arts)
            .unwrap_or_default();
        for (i, a) in self.arts.iter().enumerate() {
            self.check_id(ARTS_FILE, format!("arts[{i}].id"), &a.id)?;
        }
        Ok(self)
    }

    /// Adds the upgrade and temper rules (RON text, if any). Fails on an ID with no
    /// registry entry; the validator checks the rest.
    pub fn with_upgrades(mut self, text: Option<&str>) -> Result<Self, ContentError> {
        self.upgrades = text
            .map(|t| parse::<UpgradeRules>(UPGRADES_FILE, t))
            .transpose()?;
        if let Some(u) = &self.upgrades {
            for (i, c) in u.common_costs.iter().enumerate() {
                self.check_id(UPGRADES_FILE, format!("common_costs[{i}].item"), &c.item)?;
            }
            self.check_id(
                UPGRADES_FILE,
                "starglass_item".to_owned(),
                &u.starglass_item,
            )?;
            self.check_id(UPGRADES_FILE, "ember_item".to_owned(), &u.ember_item)?;
            self.check_id(
                UPGRADES_FILE,
                "whetstone_item".to_owned(),
                &u.whetstone_item,
            )?;
        }
        Ok(self)
    }

    /// Finds a spell by string ID.
    pub fn spell_by_id(&self, id: &str) -> Option<&SpellDef> {
        self.spells.iter().find(|s| s.id == id)
    }

    /// Adds the AI tuning (RON text, if any) and the enemy definitions as
    /// `(relative path, text)`. Fails on an ID that has no registry entry.
    pub fn with_enemies(
        mut self,
        ai: Option<&str>,
        enemies: Vec<(String, String)>,
    ) -> Result<Self, ContentError> {
        self.ai = ai.map(|t| parse::<AiTuning>(AI_FILE, t)).transpose()?;
        for (path, text) in enemies {
            let def: EnemyDef = parse(&path, &text)?;
            for (field, id) in def.references() {
                self.check_id(&path, field, id)?;
            }
            self.enemies.insert(path, def);
        }
        Ok(self)
    }

    /// Adds the boss definitions as `(relative path, text)`. A boss's body and parts join
    /// `enemies` under the keys `<path>#body` and `<path>#part<i>`. Fails on an ID that has
    /// no registry entry.
    pub fn with_bosses(mut self, bosses: Vec<(String, String)>) -> Result<Self, ContentError> {
        for (path, text) in bosses {
            let def: BossDef = parse(&path, &text)?;
            for (field, id) in def.references() {
                self.check_id(&path, field, id)?;
            }
            for (i, e) in def.actors().enumerate() {
                let key = if i == 0 {
                    format!("{path}#body")
                } else {
                    format!("{path}#part{}", i - 1)
                };
                for (field, id) in e.references() {
                    self.check_id(&path, format!("{key}.{field}"), id)?;
                }
                self.enemies.insert(key, e.clone());
            }
            self.bosses.insert(path, def);
        }
        Ok(self)
    }

    /// Adds the Echo rules (`echoes/rules.ron`) and the Echo definitions, as
    /// `(relative path, text)`. An Echo's body joins `enemies` under `<path>#body` and its
    /// moves join `moves` under `<path>`. Fails on an ID that has no registry entry.
    pub fn with_echoes(mut self, echoes: Vec<(String, String)>) -> Result<Self, ContentError> {
        for (path, text) in echoes {
            if path == ECHO_RULES_FILE {
                self.echo_rules = Some(parse(&path, &text)?);
                continue;
            }
            let def: EchoDef = parse(&path, &text)?;
            for (field, id) in def.references() {
                self.check_id(&path, field, id)?;
            }
            for (field, id) in def.body.references() {
                self.check_id(&path, format!("body.{field}"), id)?;
            }
            self.enemies
                .insert(format!("{path}#body"), def.body.clone());
            self.moves.insert(path.clone(), def.moves.clone());
            self.echoes.insert(path, def);
        }
        Ok(self)
    }

    /// Adds the NPC files as `(npcs/<file>, text)` (WP-068): one `NpcDef` each, and
    /// `npcs/stations.ron` (a list of `StationDef`). Fails on an ID that has no registry
    /// entry, or on an NPC or station listed twice.
    pub fn with_npcs(mut self, files: Vec<(String, String)>) -> Result<Self, ContentError> {
        for (path, text) in files {
            if path == npc::STATIONS_FILE {
                let list: Vec<StationDef> = parse(&path, &text)?;
                for (i, s) in list.iter().enumerate() {
                    self.check_id(&path, format!("[{i}].id"), &s.id)?;
                    if self.stations.iter().any(|o| o.id == s.id) {
                        return Err(ContentError::Parse {
                            file: path.clone(),
                            message: format!("station {:?} is listed twice", s.id),
                        });
                    }
                    self.stations.push(s.clone());
                }
                continue;
            }
            let def: NpcDef = parse(&path, &text)?;
            for (field, id) in def.references() {
                self.check_id(&path, field, id)?;
            }
            if self.npc_by_id(&def.id).is_some() {
                return Err(ContentError::Parse {
                    file: path,
                    message: format!("NPC {:?} is listed twice", def.id),
                });
            }
            self.npcs.insert(path, def);
        }
        Ok(self)
    }

    /// Finds an NPC definition by string ID.
    pub fn npc_by_id(&self, id: &str) -> Option<&NpcDef> {
        self.npcs.values().find(|n| n.id == id)
    }

    /// Finds a station definition by string ID.
    pub fn station_by_id(&self, id: &str) -> Option<&StationDef> {
        self.stations.iter().find(|s| s.id == id)
    }

    /// Finds an Echo definition by string ID.
    pub fn echo_by_id(&self, id: &str) -> Option<&EchoDef> {
        self.echoes.values().find(|e| e.id == id)
    }

    /// Adds kit catalogs as `(kits/<name>.ron, text)` and level files as
    /// `(levels/<area>/<file>, text)`, then compiles each level into `scenarios` under
    /// `levels/<area>`. Fails on a parse or compile error, an unregistered area or kit
    /// piece ID, or a level whose name a `scenarios/` file already uses.
    pub fn with_levels(
        mut self,
        kits: Vec<(String, String)>,
        levels: Vec<(String, String)>,
    ) -> Result<Self, ContentError> {
        for (path, text) in kits {
            let cat: KitCatalog =
                level::parse_ron(&text).map_err(|message| ContentError::Parse {
                    file: path.clone(),
                    message,
                })?;
            for (i, p) in cat.pieces.iter().enumerate() {
                self.check_id(&path, format!("pieces[{i}].id"), &p.id)?;
            }
            self.kits.insert(path, cat);
        }
        let mut by_dir: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        for (path, text) in levels {
            let (dir, file) = path.rsplit_once('/').ok_or_else(|| ContentError::Parse {
                file: path.clone(),
                message: "a level file must be levels/<area>/<file>".to_owned(),
            })?;
            by_dir
                .entry(dir.to_owned())
                .or_default()
                .insert(file.to_owned(), text);
        }
        let index =
            Kits::new(self.kits.iter().map(|(f, c)| (f.as_str(), c))).map_err(|message| {
                ContentError::Parse {
                    file: KITS_DIR.to_owned(),
                    message,
                }
            })?;
        let mut compiled = Vec::new();
        for (dir, files) in &by_dir {
            let name = dir.strip_prefix(&format!("{LEVELS_DIR}/")).unwrap_or(dir);
            let err = |message: String| ContentError::Parse {
                file: dir.clone(),
                message,
            };
            let lvl = Level::parse(|f| files.get(f).cloned()).map_err(|(f, message)| {
                ContentError::Parse {
                    file: format!("{dir}/{f}"),
                    message,
                }
            })?;
            self.check_id(&format!("{dir}/area.ron"), "id".to_owned(), &lvl.area.id)?;
            lvl.check(name, &index).map_err(err)?;
            let scenario = lvl.compile(&index).map_err(err)?;
            let entities = format!("{dir}/entities.ron");
            for c in &scenario.chests {
                if let Some(item) = &c.loot.item {
                    self.check_id(&entities, format!("chest {}: loot.item", c.id), item)?;
                }
            }
            for d in &scenario.doors {
                if let scenario::DoorKind::Keyed(key) = &d.kind {
                    self.check_id(&entities, format!("door {}: key", d.id), key)?;
                }
                if let scenario::DoorKind::Held { items, .. } = &d.kind {
                    for key in items {
                        self.check_id(&entities, format!("door {}: held item", d.id), key)?;
                    }
                }
            }
            for n in &scenario.npcs {
                if self.npc_by_id(&n.npc).is_none() {
                    return Err(err(format!("npc {:?} has no definition in npcs/", n.npc)));
                }
            }
            for s in &scenario.stations {
                if self.station_by_id(&s.station).is_none() {
                    return Err(err(format!(
                        "station {:?} has no definition in npcs/stations.ron",
                        s.station
                    )));
                }
            }
            if self.scenario(name).is_some() {
                return Err(err(format!(
                    "{SCENARIOS_DIR}/{name}.ron already defines a scenario with this name"
                )));
            }
            compiled.push((dir.clone(), lvl, scenario));
        }
        for (dir, lvl, scenario) in compiled {
            self.scenarios.insert(dir.clone(), scenario);
            self.levels.insert(dir, lvl);
        }
        Ok(self)
    }

    /// Finds a level by area ID: `test_field` is `levels/test_field`.
    pub fn level(&self, name: &str) -> Option<&Level> {
        self.levels.get(&format!("{LEVELS_DIR}/{name}"))
    }

    /// Finds a boss definition by string ID.
    pub fn boss_by_id(&self, id: &str) -> Option<&BossDef> {
        self.bosses.values().find(|b| b.id == id)
    }

    /// Finds an enemy definition by string ID.
    pub fn enemy_by_id(&self, id: &str) -> Option<&EnemyDef> {
        self.enemies.values().find(|e| e.id == id)
    }

    fn check_id(&self, file: &str, field: String, id: &str) -> Result<(), ContentError> {
        if self.registry.contains(id) {
            Ok(())
        } else {
            Err(ContentError::Unregistered {
                file: file.to_owned(),
                field,
                id: id.to_owned(),
            })
        }
    }

    /// Finds a scenario by name: `m0_flat` is `scenarios/m0_flat.ron`, and a compiled
    /// level such as `test_field` is `levels/test_field`.
    pub fn scenario(&self, name: &str) -> Option<&Scenario> {
        self.scenarios
            .get(&format!("{SCENARIOS_DIR}/{name}.ron"))
            .or_else(|| self.scenarios.get(&format!("{LEVELS_DIR}/{name}")))
    }

    /// Fails on the first content ID that has no registry entry.
    fn check_registered(&self) -> Result<(), ContentError> {
        let check = |file: &str, field: String, id: &str| {
            if self.registry.contains(id) {
                Ok(())
            } else {
                Err(ContentError::Unregistered {
                    file: file.to_owned(),
                    field,
                    id: id.to_owned(),
                })
            }
        };
        for (i, a) in self.attributes.attributes.iter().enumerate() {
            check("attributes.ron", format!("attributes[{i}].id"), &a.id)?;
        }
        for (file, items) in &self.moves {
            for (i, m) in items.iter().enumerate() {
                check(file, format!("[{i}].id (move {})", m.id), &m.id)?;
                if let Some(next) = &m.next_chain {
                    check(file, format!("{}: next_chain", m.id), next)?;
                }
            }
        }
        for (file, items) in &self.weapons {
            for (i, w) in items.iter().enumerate() {
                check(file, format!("[{i}].id (weapon {})", w.id), &w.id)?;
                for (field, id) in w.moveset.references() {
                    check(file, format!("{}: {field}", w.id), id)?;
                }
                if let Some(art) = &w.default_art {
                    check(file, format!("{}: default_art", w.id), art)?;
                }
            }
        }
        Ok(())
    }

    /// A 64-bit hash of the parsed content: `xxh3_64` over the `postcard` encoding of
    /// the whole struct. Comments and whitespace in the RON files don't change it.
    pub fn content_hash(&self) -> u64 {
        let bytes = postcard::to_allocvec(self).expect("content always serializes");
        xxhash_rust::xxh3::xxh3_64(&bytes)
    }

    /// Every move with the file it came from.
    pub fn all_moves(&self) -> impl Iterator<Item = (&str, &Move)> {
        self.moves
            .iter()
            .flat_map(|(f, v)| v.iter().map(move |m| (f.as_str(), m)))
    }

    /// Every weapon with the file it came from.
    pub fn all_weapons(&self) -> impl Iterator<Item = (&str, &Weapon)> {
        self.weapons
            .iter()
            .flat_map(|(f, v)| v.iter().map(move |w| (f.as_str(), w)))
    }

    /// Finds a move by string ID.
    pub fn move_by_id(&self, id: &str) -> Option<&Move> {
        self.all_moves().map(|(_, m)| m).find(|m| m.id == id)
    }

    /// Finds a projectile definition by string ID.
    pub fn projectile_by_id(&self, id: &str) -> Option<&ProjectileDef> {
        self.projectiles.iter().find(|p| p.id == id)
    }

    /// Finds a weapon by string ID.
    pub fn weapon_by_id(&self, id: &str) -> Option<&Weapon> {
        self.all_weapons().map(|(_, w)| w).find(|w| w.id == id)
    }
}
