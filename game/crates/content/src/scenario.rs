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
    /// Water volumes (WP-036): where a `water_only` enemy can go. Real levels place them in
    /// the level format (WP-040).
    #[serde(default)]
    pub water: Vec<Zone>,
    /// Reed patches (WP-036): where a reed-hiding enemy is hidden. Real levels place them in
    /// the level format (WP-040).
    #[serde(default)]
    pub reeds: Vec<Zone>,
    /// Pickups (WP-049): items lying in the level that the player takes by walking up to
    /// them. Real levels place them in the level format (WP-040).
    #[serde(default)]
    pub pickups: Vec<Pickup>,
    /// Ladders (WP-042).
    #[serde(default)]
    pub ladders: Vec<Ladder>,
    /// Lifts (WP-042).
    #[serde(default)]
    pub lifts: Vec<Lift>,
    /// Doors: locked, one-way, and keyed (WP-042).
    #[serde(default)]
    pub doors: Vec<Door>,
    /// Levers (WP-042).
    #[serde(default)]
    pub levers: Vec<Lever>,
    /// Hollow walls (WP-042).
    #[serde(default)]
    pub hollow_walls: Vec<HollowWall>,
    /// Treasure chests (WP-042).
    #[serde(default)]
    pub chests: Vec<Chest>,
    /// Authored ground messages (WP-042).
    #[serde(default)]
    pub messages: Vec<Message>,
    /// Pools of water the player wades through (WP-042).
    #[serde(default)]
    pub pools: Vec<Pool>,
    /// Deep snow (WP-042).
    #[serde(default)]
    pub snow: Vec<Zone>,
    /// Glimmer the player starts with. Zero in real levels; test scenarios set it so a
    /// death has something to drop before enemies drop Glimmer (WP-023, WP-030).
    #[serde(default)]
    pub start_glimmer: u32,
    /// The weapon ID the player starts the scenario wielding, instead of the default
    /// weapon. The player owns it. Used by the duel scenarios (WP-024).
    #[serde(default)]
    pub player_weapon: Option<String>,
}

/// A pickup: an item lying in the level. Walking within `radius_m` of it takes it, once:
/// the world raises the flag `pickup.<id>.taken`, and `codex`, if set, raises
/// `codex.<codex>` (docs/game/08-portfolio-integration.md, "Codex").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pickup {
    /// Unique ID within the scenario, lowercase with `_` or `-`; the world flag names it.
    pub id: String,
    /// Registry ID of the item (for example a relic or a Chronicle Page).
    pub item: String,
    /// Where it lies (feet level), in meters.
    pub pos: [f32; 3],
    /// How close the player's feet must be horizontally, in meters. Defaults to 1.2.
    #[serde(default = "default_pickup_radius")]
    pub radius_m: f32,
    /// The codex entry taking it unlocks, as a flag suffix such as `projects.banana-cpp`.
    #[serde(default)]
    pub codex: Option<String>,
    /// How many of the item (WP-042). Defaults to 1.
    #[serde(default = "default_pickup_count")]
    pub count: u16,
    /// Taken with `INTERACT` instead of by walking up to it (WP-042). Defaults to false.
    #[serde(default)]
    pub interact: bool,
}

fn default_pickup_count() -> u16 {
    1
}

fn default_pickup_radius() -> f32 {
    1.2
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

/// A rectangle on the ground plane, for water and reeds (WP-036).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    /// Label for tools and error messages.
    pub name: String,
    /// Center, as (x, z) in meters.
    pub center: [f32; 2],
    /// Half the size along x and z, in meters.
    pub half_extents: [f32; 2],
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

// ---------------------------------------------------------------------------------------
// World mechanics (WP-042; docs/game/02-world.md, "Traversal and world mechanics")

/// A ladder. The climber faces `yaw` (toward the wall) the whole way up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ladder {
    /// Unique among the scenario's ladders; world flags refer to it.
    pub id: String,
    /// The foot of the ladder: where the player stands to grab it, in meters (feet).
    pub pos: [f32; 3],
    /// The climber's facing, in radians from +Z toward +X.
    pub yaw: f32,
    /// How far up the rails go, in meters.
    pub height_m: f32,
    /// Starts raised and drops when the player kicks it from the top (the flag
    /// `ladder.<id>.down`). Until then it can't be climbed.
    #[serde(default)]
    pub kick_down: bool,
}

/// A lift: a kinematic platform that moves between a bottom stop and a top stop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lift {
    /// Unique among the scenario's lifts.
    pub id: String,
    /// The platform's center, as (x, z) in meters.
    pub pos: [f32; 2],
    /// Height of the platform's top surface at stop 0, in meters. The lift starts here.
    pub bottom_y: f32,
    /// Height of the platform's top surface at stop 1, in meters.
    pub top_y: f32,
    /// Half the platform's size along x and z, in meters.
    pub half_extents: [f32; 2],
}

/// How a door opens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DoorKind {
    /// Shut until a lever opens it.
    Locked,
    /// Opens with `INTERACT` from the side it faces; from the other side it stays shut.
    OneWay,
    /// Opens with `INTERACT` while the player holds this key item (one is used up).
    Keyed(String),
}

/// A door: a slab that sinks into the floor when it opens (the flag `door.<id>.open`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Door {
    /// Unique among the scenario's doors.
    pub id: String,
    /// The middle of the door's base, in meters. The slab is `height_m` tall above it.
    pub pos: [f32; 3],
    /// The side the door faces, in radians from +Z toward +X: a one-way door opens from
    /// there. The slab is perpendicular to that direction.
    pub yaw: f32,
    pub half_width_m: f32,
    pub height_m: f32,
    pub kind: DoorKind,
}

/// What a lever works.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LeverTarget {
    /// Toggles the door with this ID open and shut.
    Door(String),
    /// A lever standing at `stop` (0 bottom, 1 top) of the lift with this ID: it calls the
    /// lift to that stop, or, if the lift is already waiting there, sends it to the
    /// other stop. A lift needs a lever at each stop, so it never strands the player.
    Lift { id: String, stop: u8 },
    /// Toggles the pool with this ID between its normal and raised depth.
    Water(String),
}

/// A lever. Pulling it flips the flag `lever.<id>.pulled` and works its target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lever {
    /// Unique among the scenario's levers.
    pub id: String,
    /// Where the player stands to pull it, in meters (feet).
    pub pos: [f32; 3],
    /// The direction the lever faces, in radians from +Z toward +X (for the renderer).
    pub yaw: f32,
    pub target: LeverTarget,
}

/// A wall that looks solid until a hit reveals the passage (the flag `wall.<id>.revealed`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HollowWall {
    /// Unique among the scenario's hollow walls.
    pub id: String,
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
}

/// What a pickup or chest gives.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Loot {
    /// An item or weapon ID from the registry, if any.
    pub item: Option<String>,
    /// How many of the item. 0 counts as 1.
    pub count: u16,
    /// Glimmer, on top of the item.
    pub glimmer: u32,
}

/// A treasure chest: `INTERACT` opens it (the flag `chest.<id>.opened`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chest {
    /// Unique among the scenario's chests.
    pub id: String,
    pub pos: [f32; 3],
    pub yaw: f32,
    pub loot: Loot,
}

/// An authored ground message: `INTERACT` reads it (the flag `message.<id>.read`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    /// Unique among the scenario's messages.
    pub id: String,
    pub pos: [f32; 3],
    pub text: String,
}

/// A pool of water on the ground plane. Shallow water slows the player; deep water is
/// a wall; blight water builds Blight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pool {
    /// Unique among the scenario's pools.
    pub name: String,
    pub center: [f32; 2],
    pub half_extents: [f32; 2],
    /// Water depth, in meters.
    pub depth_m: f32,
    /// The depth when a lever raises the water (the flag `pool.<name>.raised`).
    #[serde(default)]
    pub raised_depth_m: Option<f32>,
    /// Builds Blight while the player wades in it.
    #[serde(default)]
    pub blight: bool,
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
        let unique = |dup: bool, kind: &str, id: &str| {
            if dup {
                Err(format!("{kind} {id:?} is listed twice"))
            } else {
                Ok(())
            }
        };
        let positive = |kind: &str, id: &str, what: &str, v: f32| {
            if v.is_finite() && v > 0.0 {
                Ok(())
            } else {
                Err(format!("{kind} {id:?}: {what} must be positive"))
            }
        };
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
        for (i, p) in self.pickups.iter().enumerate() {
            if p.id.is_empty() || p.radius_m <= 0.0 {
                return Err("a pickup needs an id and a positive radius_m".to_owned());
            }
            if self.pickups[..i].iter().any(|o| o.id == p.id) {
                return Err(format!("pickup {:?} is listed twice", p.id));
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
        for (i, l) in self.ladders.iter().enumerate() {
            unique(
                self.ladders[..i].iter().any(|o| o.id == l.id),
                "ladder",
                &l.id,
            )?;
            positive("ladder", &l.id, "height_m", l.height_m)?;
        }
        for (i, l) in self.lifts.iter().enumerate() {
            unique(self.lifts[..i].iter().any(|o| o.id == l.id), "lift", &l.id)?;
            positive(
                "lift",
                &l.id,
                "half_extents",
                l.half_extents[0].min(l.half_extents[1]),
            )?;
            if l.top_y <= l.bottom_y {
                return Err(format!("lift {:?}: top_y must be above bottom_y", l.id));
            }
        }
        for (i, d) in self.doors.iter().enumerate() {
            unique(self.doors[..i].iter().any(|o| o.id == d.id), "door", &d.id)?;
            positive("door", &d.id, "half_width_m", d.half_width_m)?;
            positive("door", &d.id, "height_m", d.height_m)?;
            // The slab is an axis-aligned box, so the door faces along x or z.
            let turns = d.yaw / core::f32::consts::FRAC_PI_2;
            if (turns - turns.round()).abs() > 0.01 {
                return Err(format!(
                    "door {:?}: yaw must be a multiple of 90 degrees",
                    d.id
                ));
            }
        }
        for (i, l) in self.levers.iter().enumerate() {
            unique(
                self.levers[..i].iter().any(|o| o.id == l.id),
                "lever",
                &l.id,
            )?;
            let ok = match &l.target {
                LeverTarget::Door(id) => self.doors.iter().any(|d| &d.id == id),
                LeverTarget::Lift { id, stop } => {
                    *stop <= 1 && self.lifts.iter().any(|d| &d.id == id)
                }
                LeverTarget::Water(id) => self
                    .pools
                    .iter()
                    .any(|p| &p.name == id && p.raised_depth_m.is_some()),
            };
            if !ok {
                return Err(format!(
                    "lever {:?}: its target doesn't exist (a lift stop is 0 or 1, and a water \
                     target needs a pool with raised_depth_m)",
                    l.id
                ));
            }
        }
        for (i, w) in self.hollow_walls.iter().enumerate() {
            unique(
                self.hollow_walls[..i].iter().any(|o| o.id == w.id),
                "hollow wall",
                &w.id,
            )?;
            if w.half_extents.iter().any(|&e| e <= 0.0) {
                return Err(format!(
                    "hollow wall {:?}: half_extents must be positive",
                    w.id
                ));
            }
        }
        for (i, c) in self.chests.iter().enumerate() {
            unique(
                self.chests[..i].iter().any(|o| o.id == c.id),
                "chest",
                &c.id,
            )?;
        }
        for (i, m) in self.messages.iter().enumerate() {
            unique(
                self.messages[..i].iter().any(|o| o.id == m.id),
                "message",
                &m.id,
            )?;
        }
        for (i, p) in self.pools.iter().enumerate() {
            unique(
                self.pools[..i].iter().any(|o| o.name == p.name),
                "pool",
                &p.name,
            )?;
            positive("pool", &p.name, "depth_m", p.depth_m)?;
            if p.raised_depth_m.is_some_and(|r| r <= 0.0) {
                return Err(format!(
                    "pool {:?}: raised_depth_m must be positive",
                    p.name
                ));
            }
        }
        Ok(())
    }
}
