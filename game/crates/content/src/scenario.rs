//! Scenarios: a static level (a heightfield plus box colliders) and a player spawn.
//! Loaded from `scenarios/<name>.ron`. See docs/game/01-architecture.md, "Physics".

use serde::{Deserialize, Serialize};

use crate::moves::StatusKind;

/// A level the simulation can start in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    /// Player feet position at spawn, in meters.
    pub spawn: [f32; 3],
    /// Player facing at spawn, in radians from +Z toward +X.
    pub spawn_yaw: f32,
    /// An actor whose feet drop below this height dies (falling out of the world).
    pub kill_y: f32,
    #[serde(default)]
    pub heightfield: Option<Heightfield>,
    #[serde(default)]
    pub boxes: Vec<BoxCollider>,
    /// Standing dummy actors: they have HP, never move or act, and can be locked on to.
    #[serde(default)]
    pub dummies: Vec<Dummy>,
    /// AI enemies (WP-030), from `enemies/<enemy>.ron`. Real levels place them in the
    /// level format (WP-040).
    #[serde(default)]
    pub enemies: Vec<EnemySpawn>,
    /// Boss fights (WP-033), from `bosses/<boss>.ron`. Real levels place them in the level
    /// format (WP-040).
    #[serde(default)]
    pub bosses: Vec<BossSpawn>,
    /// Lanterns (WP-029). Real levels place them in the level format (WP-040).
    #[serde(default)]
    pub lanterns: Vec<Lantern>,
    /// Glimmer the player starts with. Zero in real levels; test scenarios set it so a
    /// death has something to drop before enemies drop Glimmer (WP-023, WP-030).
    #[serde(default)]
    pub start_glimmer: u32,
    /// The weapon ID the player starts the scenario wielding, instead of the default
    /// weapon. The player owns it. Used by the duel scenarios (WP-024).
    #[serde(default)]
    pub player_weapon: Option<String>,
}

/// A boss fight: where the boss stands and the fog gate that starts it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossSpawn {
    /// Label for tools and error messages. Unique among the scenario's bosses and enemies.
    pub name: String,
    /// The boss definition's ID.
    pub boss: String,
    /// The body's feet position, in meters. Parts stand at their offsets from it.
    pub pos: [f32; 3],
    /// Facing, in radians from +Z toward +X.
    pub yaw: f32,
    pub gate: FogGate,
}

/// A fog gate: a vertical plane the player walks through into the arena. Once they are
/// through, the fight starts and the gate seals until the boss falls or the player dies.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FogGate {
    /// The middle of the gate's threshold, in meters.
    pub pos: [f32; 3],
    /// The direction into the arena, in radians from +Z toward +X. The gate's plane is
    /// perpendicular to it.
    pub yaw: f32,
    /// Half the gate's width, in meters.
    pub half_width_m: f32,
    /// How far past the plane, into the arena, still counts as walking through.
    pub depth_m: f32,
}

/// A Lantern: where the player rests and respawns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lantern {
    /// Unique ID within the scenario; saves and world flags refer to the Lantern by it.
    pub id: String,
    /// Where the player stands to rest, and where they respawn, in meters (feet).
    pub pos: [f32; 3],
    /// The player's facing on respawn, in radians from +Z toward +X.
    pub yaw: f32,
}

/// A training dummy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dummy {
    /// Label for tools and error messages.
    pub name: String,
    /// Feet position, in meters.
    pub pos: [f32; 3],
    /// Facing, in radians from +Z toward +X.
    pub yaw: f32,
    /// Max HP; the dummy starts at full HP.
    pub hp: u32,
    /// What the dummy shrugs off. Defaults to none.
    #[serde(default)]
    pub defense: DefenseData,
    /// Poise (docs/game/03-combat.md, "Poise and stance"). 0 means the dummy never
    /// staggers. Defaults to 100, so a padded dummy shrugs off a light chain.
    #[serde(default = "default_dummy_poise")]
    pub poise: u16,
    /// Stance meter. 0 (the default) means no stance meter, as for regular enemies.
    #[serde(default)]
    pub stance: u16,
    /// Unaware of the player until hit: it can be backstabbed. Defaults to true.
    #[serde(default = "default_true")]
    pub unaware: bool,
    /// A scripted attack loop. The hook WP-030's AI replaces.
    #[serde(default)]
    pub attack: Option<DummyAttack>,
    /// What the dummy resists: the meter a status needs to fill (WP-025). Defaults to 100.
    #[serde(default)]
    pub resist: ResistData,
    /// A boss: its resistances grow with every proc this fight, and it must list one to
    /// three immunities. Defaults to false.
    #[serde(default)]
    pub boss: bool,
    /// Statuses the dummy can't build up at all.
    #[serde(default)]
    pub immune: Vec<StatusKind>,
    /// Delirium staggers it (enemies that can go mad). Defaults to false.
    #[serde(default)]
    pub can_go_mad: bool,
}

/// Where an AI enemy spawns, and returns to when it leashes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemySpawn {
    /// Label for tools and error messages.
    pub name: String,
    /// The enemy definition's ID.
    pub enemy: String,
    /// Feet position, in meters.
    pub pos: [f32; 3],
    /// Facing, in radians from +Z toward +X.
    pub yaw: f32,
    /// Waypoints it walks in a loop while Idle. Empty: it stands at its spawn.
    #[serde(default)]
    pub patrol: Vec<[f32; 3]>,
}

/// An enemy's status resistances (docs/game/05-enemies-and-bosses.md, `resist`). In the
/// player's case they come from Vitality, Focus, and Endurance instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResistData {
    /// Poison and Blight.
    pub immunity: u16,
    /// Rupture and Rime.
    pub robustness: u16,
    /// Drowse and Delirium.
    pub composure: u16,
}

impl Default for ResistData {
    fn default() -> Self {
        Self {
            immunity: 100,
            robustness: 100,
            composure: 100,
        }
    }
}

fn default_dummy_poise() -> u16 {
    100
}

fn default_true() -> bool {
    true
}

/// A dummy that swings on a timer: it starts `move_id` on tick `first_tick`, and again
/// every `every_ticks` ticks after, whenever it is free to act. It doesn't aim or react:
/// its facing is the scenario yaw.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DummyAttack {
    pub move_id: String,
    pub first_tick: u32,
    pub every_ticks: u32,
}

/// One of the eight damage types (docs/game/03-combat.md, "Damage types").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DamageType {
    Neutral,
    Slash,
    Blunt,
    Pierce,
    Aether,
    Fire,
    Storm,
    Radiance,
}

/// An actor's defenses in data (docs/game/03-combat.md, "Damage dealt").
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct DefenseData {
    /// Flat defense against every damage type.
    pub flat: u32,
    /// Percent of a type's damage absorbed (capped by `damage.absorption_cap_percent`).
    pub absorption: Vec<(DamageType, u8)>,
    /// Damage multiplier in percent for a type; 125 is the spec's 1.25. Unlisted is 100.
    pub weakness: Vec<(DamageType, u16)>,
}

/// A terrain heightfield centered on `center`. Vertex `(row, col)` sits at
/// `x = center.x - size_x / 2 + col * size_x / (cols - 1)` and
/// `z = center.z - size_z / 2 + row * size_z / (rows - 1)`, at height
/// `center.y + heights[row * cols + col]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Heightfield {
    pub center: [f32; 3],
    /// Number of vertex rows, along Z. At least 2.
    pub rows: u32,
    /// Number of vertex columns, along X. At least 2.
    pub cols: u32,
    /// Total extent along X, in meters.
    pub size_x: f32,
    /// Total extent along Z, in meters.
    pub size_z: f32,
    /// Row-major heights in meters: `rows * cols` values.
    pub heights: Vec<f32>,
}

/// An axis-aligned box collider (a kit piece).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoxCollider {
    /// Label for tools and error messages.
    pub name: String,
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
}

impl Scenario {
    /// Checks shape invariants the physics builder relies on.
    pub fn check(&self) -> Result<(), String> {
        if let Some(h) = &self.heightfield {
            if h.rows < 2 || h.cols < 2 {
                return Err(format!(
                    "heightfield needs at least 2x2 vertices, got {}x{}",
                    h.rows, h.cols
                ));
            }
            let want = h.rows as usize * h.cols as usize;
            if h.heights.len() != want {
                return Err(format!(
                    "heightfield has {} heights, expected rows * cols = {want}",
                    h.heights.len()
                ));
            }
            if h.size_x <= 0.0 || h.size_z <= 0.0 {
                return Err("heightfield size_x and size_z must be positive".to_owned());
            }
        }
        for b in &self.boxes {
            if b.half_extents.iter().any(|&e| e <= 0.0) {
                return Err(format!("box {:?}: half_extents must be positive", b.name));
            }
        }
        for d in &self.dummies {
            if d.hp == 0 {
                return Err(format!("dummy {:?}: hp must be positive", d.name));
            }
            if d.boss && !(1..=3).contains(&d.immune.len()) {
                return Err(format!(
                    "dummy {:?}: a boss is immune to at least one status and at most three",
                    d.name
                ));
            }
            if d.attack.as_ref().is_some_and(|a| a.every_ticks == 0) {
                return Err(format!("dummy {:?}: every_ticks must be positive", d.name));
            }
        }
        for (i, e) in self.enemies.iter().enumerate() {
            if self.enemies[..i].iter().any(|o| o.name == e.name) {
                return Err(format!("enemy spawn {:?} is listed twice", e.name));
            }
        }
        for (i, b) in self.bosses.iter().enumerate() {
            if self.bosses[..i].iter().any(|o| o.name == b.name)
                || self.enemies.iter().any(|o| o.name == b.name)
            {
                return Err(format!("boss spawn {:?} is listed twice", b.name));
            }
            if b.gate.half_width_m <= 0.0 || b.gate.depth_m <= 0.0 {
                return Err(format!(
                    "boss {:?}: the gate's half_width_m and depth_m must be positive",
                    b.name
                ));
            }
        }
        for (i, l) in self.lanterns.iter().enumerate() {
            if l.id.is_empty() {
                return Err("a lantern has an empty id".to_owned());
            }
            if self.lanterns[..i].iter().any(|o| o.id == l.id) {
                return Err(format!("lantern {:?} is listed twice", l.id));
            }
        }
        Ok(())
    }
}
