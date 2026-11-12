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

pub mod attributes;
pub mod boss;
pub mod consumables;
pub mod enemy;
pub mod locomotion;
pub mod moves;
pub mod nav;
pub mod portfolio;
pub mod projectiles;
pub mod registry;
pub mod scenario;
pub mod status;
pub mod validate;
pub mod weapons;

pub use attributes::Attributes;
pub use boss::BossDef;
pub use consumables::{Consumables, Flask, Item, ItemKind};
pub use enemy::{AiTuning, EnemyDef};
pub use locomotion::{Death, EquipLoad, Guard, LockOn, Locomotion, Poise};
pub use moves::Move;
pub use nav::NavConfig;
pub use portfolio::{Portfolio, PortfolioContent};
pub use projectiles::{Homing, ProjectileDef};
pub use registry::{NumId, Registry};
pub use scenario::{Lantern, Scenario};
pub use status::StatusRules;
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
/// Directory (relative to the data root) holding enemy definitions, one per file.
const ENEMIES_DIR: &str = "enemies";
/// Directory (relative to the data root) holding boss definitions, one per file.
const BOSSES_DIR: &str = "bosses";
/// The shared AI tuning file (relative to the data root).
const AI_FILE: &str = "ai.ron";

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
        "moves/bosses/test_boss.ron",
        include_str!("../../../data/moves/bosses/test_boss.ron"),
    ),
    (
        "moves/bosses/b01.ron",
        include_str!("../../../data/moves/bosses/b01.ron"),
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
];
const EMBEDDED_REGISTRY: &str = include_str!("../../../data/registry.ron");
const EMBEDDED_ATTRIBUTES: &str = include_str!("../../../data/attributes.ron");
const EMBEDDED_LOCOMOTION: &str = include_str!("../../../data/locomotion.ron");
const EMBEDDED_CONSUMABLES: &str = include_str!("../../../data/consumables.ron");
const EMBEDDED_PROJECTILES: &str = include_str!("../../../data/projectiles.ron");
const EMBEDDED_STATUS: &str = include_str!("../../../data/status.ron");
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
        "scenarios/test_field.ron",
        include_str!("../../../data/scenarios/test_field.ron"),
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
        "scenarios/boss_gate.ron",
        include_str!("../../../data/scenarios/boss_gate.ron"),
    ),
    (
        "scenarios/boss_b01.ron",
        include_str!("../../../data/scenarios/boss_b01.ron"),
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
    /// `scenarios/test_field.ron`. Look one up by name with [`Content::scenario`].
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
        .with_enemies(ai.as_deref(), list(ENEMIES_DIR)?)?
        .with_bosses(list(BOSSES_DIR)?)
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
        .with_enemies(Some(EMBEDDED_AI), own(EMBEDDED_ENEMIES))?
        .with_bosses(own(EMBEDDED_BOSSES))
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
            ai: None,
            enemies: BTreeMap::new(),
            bosses: BTreeMap::new(),
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

    /// Finds a scenario by name: `test_field` is `scenarios/test_field.ron`.
    pub fn scenario(&self, name: &str) -> Option<&Scenario> {
        self.scenarios.get(&format!("{SCENARIOS_DIR}/{name}.ron"))
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
