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
pub mod moves;
pub mod registry;
pub mod validate;
pub mod weapons;

pub use attributes::Attributes;
pub use moves::Move;
pub use registry::{NumId, Registry};
pub use validate::{Issue, Severity, Validate};
pub use weapons::Weapon;

use registry::RegistryFile;

/// Directory (relative to the data root) holding move files.
const MOVES_DIR: &str = "moves";
/// Directory (relative to the data root) holding weapon files.
const WEAPONS_DIR: &str = "weapons";

/// Embedded copies of every data file, as `(path relative to the data root, text)`.
/// A new data file needs a line here; the `embedded_matches_data_dir` test fails if one
/// is missing.
const EMBEDDED_MOVES: &[(&str, &str)] = &[(
    "moves/straight_sword.ron",
    include_str!("../../../data/moves/straight_sword.ron"),
)];
const EMBEDDED_WEAPONS: &[(&str, &str)] = &[(
    "weapons/straight_sword.ron",
    include_str!("../../../data/weapons/straight_sword.ron"),
)];
const EMBEDDED_REGISTRY: &str = include_str!("../../../data/registry.ron");
const EMBEDDED_ATTRIBUTES: &str = include_str!("../../../data/attributes.ron");

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
                }
            }
            names.sort();
            names
                .into_iter()
                .map(|name| Ok((format!("{sub}/{name}"), read(&sub_dir.join(&name))?)))
                .collect()
        };
        Self::from_sources(
            &read(&dir.join("registry.ron"))?,
            &read(&dir.join("attributes.ron"))?,
            list(MOVES_DIR)?,
            list(WEAPONS_DIR)?,
        )
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
        )
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
        };
        content.check_registered()?;
        Ok(content)
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

    /// Finds a weapon by string ID.
    pub fn weapon_by_id(&self, id: &str) -> Option<&Weapon> {
        self.all_weapons().map(|(_, w)| w).find(|w| w.id == id)
    }
}
