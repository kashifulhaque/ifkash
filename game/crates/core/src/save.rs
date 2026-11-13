//! Saving and loading. See docs/game/01-architecture.md, "Persistence".
//!
//! A save is a versioned `postcard` blob of the *persistent* parts of a world: the
//! character's attributes, the inventory, the world flags, the last Lantern, the death
//! count and the Remnant. The transient simulation (positions, enemy and boss state,
//! projectiles, the RNG, the tick count) is not saved. Loading builds a fresh world for the
//! saved scenario and seed, applies the persistent state, and puts the player at their
//! respawn point.
//!
//! # Format
//!
//! ```text
//! "LFSV"  u16 version (little endian)  postcard(body)
//! ```
//!
//! The body's layout depends on the version. [`decode`] reads the header and calls the
//! reader for that version, so adding a version means adding a body type, a reader that
//! upgrades it to the current [`SaveData`], and a match arm. Version 1 is the only one.
//!
//! # The persistent hash
//!
//! [`persistent_hash`] is the xxh3 of the postcard bytes of [`SaveData`] as captured from
//! a world. It leaves out everything that isn't saved, so a world that was saved and
//! loaded has the same persistent hash as the one that was saved. It is unrelated to
//! [`World::state_hash`].

use lf_content::Content;
use serde::{Deserialize, Serialize};

use crate::actor::stats::Attrs;
use crate::items::Inventory;
use crate::world::{World, WorldError};
use crate::world_state::Remnant;
use crate::world_state::flags::Flags;

/// The four bytes every save starts with.
pub const MAGIC: [u8; 4] = *b"LFSV";
/// The version [`encode`] writes.
pub const CURRENT_VERSION: u16 = 1;
/// Bytes before the body: the magic and the version.
const HEADER_LEN: usize = 6;

/// Everything a save keeps. Equal `SaveData` means equal persistent state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveData {
    /// Hash of the content the save was made with. Informational: registry numbers are
    /// append-only, so a save from older content still loads.
    pub content_hash: u64,
    /// The seed the world was started from.
    pub seed: u64,
    /// The scenario (`scenarios/<name>.ron`) the world was running.
    pub scenario: String,
    pub attrs: Attrs,
    pub inventory: Inventory,
    pub flags: Flags,
    /// ID of the Lantern the player last rested at.
    pub respawn_lantern: Option<String>,
    pub keep_glimmer_on_death: bool,
    pub deaths: u32,
    pub remnant: Option<Remnant>,
}

/// Why a save couldn't be read or applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveError {
    /// The blob doesn't start with [`MAGIC`] or is too short for a header.
    NotASave,
    /// The save is from a version this build doesn't know (older than 1, or newer than
    /// [`CURRENT_VERSION`]).
    UnsupportedVersion(u16),
    /// The body doesn't decode.
    Corrupt(String),
    /// The save names a scenario the content doesn't have, or the world can't be built.
    World(WorldError),
}

impl core::fmt::Display for SaveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotASave => write!(f, "not a Latticefall save"),
            Self::UnsupportedVersion(v) => write!(
                f,
                "save version {v} isn't supported (this build reads up to {CURRENT_VERSION})"
            ),
            Self::Corrupt(m) => write!(f, "the save is damaged: {m}"),
            Self::World(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<WorldError> for SaveError {
    fn from(e: WorldError) -> Self {
        Self::World(e)
    }
}

/// The persistent state of `world`. `seed` is what the world was started from: the world
/// doesn't keep it, so the host passes it.
pub fn capture(world: &World, seed: u64, content: &Content) -> SaveData {
    let player = world.actors.get(world.player);
    SaveData {
        content_hash: content.content_hash(),
        seed,
        scenario: world.physics.scenario.clone(),
        attrs: player.map(|p| p.stats.attrs).unwrap_or_default(),
        inventory: player.map(|p| p.inventory.clone()).unwrap_or_default(),
        flags: world.state.flags.clone(),
        respawn_lantern: world.state.respawn_lantern.clone(),
        keep_glimmer_on_death: world.state.keep_glimmer_on_death,
        deaths: world.state.death.deaths,
        remnant: world.state.death.remnant.clone(),
    }
}

/// Serializes `data` with the current version header.
pub fn encode(data: &SaveData) -> Vec<u8> {
    let mut out = Vec::with_capacity(256);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&CURRENT_VERSION.to_le_bytes());
    out.extend_from_slice(&postcard::to_allocvec(data).expect("save serialization cannot fail"));
    out
}

/// Reads a save of any supported version and returns it as the current [`SaveData`].
pub fn decode(bytes: &[u8]) -> Result<SaveData, SaveError> {
    if bytes.len() < HEADER_LEN || bytes[..4] != MAGIC {
        return Err(SaveError::NotASave);
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let body = &bytes[HEADER_LEN..];
    match version {
        1 => read_v1(body),
        v => Err(SaveError::UnsupportedVersion(v)),
    }
}

/// Reads a version 1 body, which is the current layout. A future version 2 keeps this
/// function, changes `SaveData`, and turns the old body into the new one here, so old
/// saves keep loading.
fn read_v1(body: &[u8]) -> Result<SaveData, SaveError> {
    postcard::from_bytes(body).map_err(|e| SaveError::Corrupt(e.to_string()))
}

/// Builds a world from `data` and applies the persistent state to it. The player starts
/// at the respawn point with full HP, stamina, and FP.
pub fn restore(data: &SaveData, content: &Content) -> Result<World, SaveError> {
    let mut world = World::new(data.seed, content, &data.scenario)?;
    let player = world.player;
    if let Some(p) = world.actors.get_mut(player) {
        p.stats.attrs = data.attrs;
        let d = data.attrs.derived(&content.attributes);
        p.max_hp = d.max_hp;
        p.max_stamina = d.max_stamina;
        p.max_fp = d.max_fp;
        p.hp = p.max_hp;
        p.stamina = p.max_stamina;
        p.fp = p.max_fp;
        p.inventory = data.inventory.clone();
    }
    world.state.flags = data.flags.clone();
    // The in-game timer picks up at the last rest (WP-088; see `world_state::marks`).
    world.tick = u64::from(world.state.flags.get(crate::world_state::marks::PLAY_TICKS));
    world.state.respawn_lantern = data.respawn_lantern.clone();
    world.state.keep_glimmer_on_death = data.keep_glimmer_on_death;
    world.state.death.deaths = data.deaths;
    world.state.death.remnant = data.remnant.clone();
    let (pos, yaw) = crate::world_state::death::respawn_point(&world);
    world.state.death.safe_pos = pos;
    if let Some(p) = world.actors.get_mut(player) {
        p.pos = pos;
        p.yaw = yaw;
        p.motor.ground_y = pos[1];
        p.motor.fall_peak_y = pos[1];
    }
    world.sync_player_moveset();
    Ok(world)
}

/// [`decode`] then [`restore`].
pub fn load(bytes: &[u8], content: &Content) -> Result<World, SaveError> {
    restore(&decode(bytes)?, content)
}

/// [`capture`] then [`encode`].
pub fn save(world: &World, seed: u64, content: &Content) -> Vec<u8> {
    encode(&capture(world, seed, content))
}

/// 64-bit xxh3 of the persistent state of `world` (see the module docs).
pub fn persistent_hash(world: &World, seed: u64, content: &Content) -> u64 {
    let bytes = postcard::to_allocvec(&capture(world, seed, content))
        .expect("save serialization cannot fail");
    xxhash_rust::xxh3::xxh3_64(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::TickInput;
    use crate::tick::tick;

    const SEED: u64 = 9;

    fn content() -> Content {
        Content::load_embedded().expect("embedded content")
    }

    /// A world with some persistent state worth saving.
    fn played(content: &Content) -> World {
        let mut w = World::new(SEED, content, "test_field").unwrap();
        for _ in 0..30 {
            tick(&mut w, &TickInput::default());
        }
        w.state.flags.raise("boss.test");
        w.state.flags.set("lantern.test", 3);
        w.state.death.deaths = 4;
        w.state.death.remnant = Some(Remnant {
            pos: [1.0, 0.0, 2.0],
            glimmer: 77,
        });
        let id = w.player;
        let p = w.actors.get_mut(id).unwrap();
        p.inventory.glimmer = 1234;
        p.stats.attrs.might += 2;
        w
    }

    #[test]
    fn round_trip_keeps_the_persistent_hash() {
        let c = content();
        let w = played(&c);
        let before = persistent_hash(&w, SEED, &c);
        let bytes = save(&w, SEED, &c);
        let loaded = load(&bytes, &c).unwrap();
        assert_eq!(persistent_hash(&loaded, SEED, &c), before);
        assert_eq!(capture(&loaded, SEED, &c), capture(&w, SEED, &c));
        assert!(loaded.state.flags.is_set("boss.test"));
        assert_eq!(loaded.state.death.deaths, 4);
    }

    #[test]
    fn the_persistent_hash_ignores_the_transient_simulation() {
        let c = content();
        let mut w = played(&c);
        let before = persistent_hash(&w, SEED, &c);
        let id = w.player;
        w.actors.get_mut(id).unwrap().pos = [3.0, 0.0, 3.0];
        for _ in 0..10 {
            tick(&mut w, &TickInput::default());
        }
        assert_eq!(persistent_hash(&w, SEED, &c), before);
    }

    #[test]
    fn the_hash_changes_with_persistent_state() {
        let c = content();
        let mut w = played(&c);
        let before = persistent_hash(&w, SEED, &c);
        w.state.flags.raise("another");
        assert_ne!(persistent_hash(&w, SEED, &c), before);
    }

    #[test]
    fn a_loaded_world_keeps_running() {
        let c = content();
        let w = played(&c);
        let mut loaded = load(&save(&w, SEED, &c), &c).unwrap();
        let hash = persistent_hash(&loaded, SEED, &c);
        for _ in 0..30 {
            tick(&mut loaded, &TickInput::default());
        }
        assert_eq!(persistent_hash(&loaded, SEED, &c), hash);
    }

    #[test]
    fn bad_blobs_are_rejected() {
        let c = content();
        assert_eq!(decode(b"").unwrap_err(), SaveError::NotASave);
        assert_eq!(decode(b"NOPE\x01\x00x").unwrap_err(), SaveError::NotASave);
        let good = save(&played(&c), SEED, &c);
        let mut newer = good.clone();
        newer[4..6].copy_from_slice(&(CURRENT_VERSION + 1).to_le_bytes());
        assert_eq!(
            decode(&newer).unwrap_err(),
            SaveError::UnsupportedVersion(CURRENT_VERSION + 1)
        );
        let mut zero = good.clone();
        zero[4..6].copy_from_slice(&0u16.to_le_bytes());
        assert_eq!(decode(&zero).unwrap_err(), SaveError::UnsupportedVersion(0));
        assert!(matches!(
            decode(&good[..HEADER_LEN + 3]).unwrap_err(),
            SaveError::Corrupt(_)
        ));
    }

    #[test]
    fn an_unknown_scenario_is_an_error() {
        let c = content();
        let mut data = capture(&played(&c), SEED, &c);
        data.scenario = "nowhere".into();
        assert!(matches!(
            restore(&data, &c).unwrap_err(),
            SaveError::World(WorldError::UnknownScenario(_))
        ));
    }
}
