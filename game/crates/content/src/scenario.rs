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
    /// Spell IDs the player knows from the start, so a test scenario can attune them with
    /// `Command::Equip` (WP-055). Real levels teach spells with pickups.
    #[serde(default)]
    pub player_spells: Vec<String>,
    /// Item IDs (a key item or a consumable, one of each) the player holds from the start
    /// (WP-067). Real levels place items instead; the Spire lists two Great Sigils until the
    /// bosses that grant them exist (see `levels/spire/area.ron`).
    #[serde(default)]
    pub player_items: Vec<String>,
    /// Echo Stones (WP-059): where the Echo Bell summons an Echo. A boss arena has one
    /// unless its definition says otherwise. Real levels place them in the level format.
    #[serde(default)]
    pub echo_stones: Vec<EchoStone>,
    /// Echoes (registry IDs) the player owns from the start of the scenario, on top of the
    /// ones their world flags unlock. Used by test scenarios (WP-059).
    #[serde(default)]
    pub echoes_unlocked: Vec<String>,
    /// NPCs (WP-068): people the player talks to. Real levels place them in the level
    /// format.
    #[serde(default)]
    pub npcs: Vec<NpcSpawn>,
    /// Stations (WP-068): the Archive shelves, the Ledger Stand, the Mirror, and the like.
    #[serde(default)]
    pub stations: Vec<StationSpawn>,
    /// A safe place (WP-068, Hollowhearth): no combat, weapons sheathed, walk only. The
    /// core clears the combat buttons and caps the stick at a walk.
    #[serde(default)]
    pub safe_zone: bool,
    /// The darkness rule (WP-062, Hollowdeep): outside the glowing zones the player sees
    /// only a few meters. `None` in every other area.
    #[serde(default)]
    pub darkness: Option<Darkness>,
    /// Glowing zones (WP-062): rectangles on the ground plane that are lit. A dark area's
    /// critical path stays inside them.
    #[serde(default)]
    pub glows: Vec<Zone>,
    /// Mine carts (WP-062): ride one along its rail, or push one into a wall.
    #[serde(default)]
    pub carts: Vec<Cart>,
    /// Trapdoors (WP-062): a hatch the player pushes open from below.
    #[serde(default)]
    pub trapdoors: Vec<Trapdoor>,
    /// Low-ceiling zones (WP-063, Mossgut Cave): a swing that rises above the ceiling hits
    /// rock instead of the target.
    #[serde(default)]
    pub low_ceilings: Vec<LowCeiling>,
    /// Light-beam emitters, mirrors, and receptors (WP-063, Glasswing Grotto).
    #[serde(default)]
    pub beam_sources: Vec<BeamSource>,
    #[serde(default)]
    pub mirrors: Vec<Mirror>,
    #[serde(default)]
    pub receptors: Vec<Receptor>,
    /// Warden Braziers (WP-066, the Whitewarden Catacombs): lit with `INTERACT`; a lit brazier
    /// keeps a revive-prone enemy down within its radius.
    #[serde(default)]
    pub braziers: Vec<Brazier>,
    /// Seal sigils (WP-066, the Bound Ring): touching one shuts the doors it names.
    #[serde(default)]
    pub sigils: Vec<Sigil>,
}

/// Where an NPC stands. The NPC's lines and quest are in `npcs/<name>.ron`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpcSpawn {
    /// The NPC's registry ID (`npc_archivist`).
    pub npc: String,
    /// Where they stand (feet level), in meters.
    pub pos: [f32; 3],
    /// The direction they face, in radians from +Z toward +X.
    pub yaw: f32,
    /// How close the player's feet must be horizontally to speak, in meters. Defaults to 3.0
    /// (the NPC stands behind a counter).
    #[serde(default = "default_talk_radius")]
    pub radius_m: f32,
}

fn default_talk_radius() -> f32 {
    3.0
}

/// Where a station stands. The station's content is in `npcs/stations.ron`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StationSpawn {
    /// The station's registry ID (`stn_archive`).
    pub station: String,
    pub pos: [f32; 3],
    pub yaw: f32,
    /// How close the player's feet must be horizontally to use it, in meters.
    #[serde(default = "default_talk_radius")]
    pub radius_m: f32,
}

/// An Echo Stone: stand within the Echo rules' `stone_ring_m` of it and ring the Echo Bell
/// to summon the selected Echo beside it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EchoStone {
    /// Unique among the scenario's Echo Stones.
    pub id: String,
    /// Where it stands (feet level), in meters. The Echo appears around this point.
    pub pos: [f32; 3],
    /// The direction the Echo faces when it appears, in radians from +Z toward +X.
    pub yaw: f32,
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
    /// Opens with `INTERACT` while the player holds at least `need` different items from
    /// `items`, and keeps them (WP-067: the Spire Gate takes any two of the four Great
    /// Sigils). Unlike [`DoorKind::Keyed`], nothing is used up.
    Held { items: Vec<String>, need: u8 },
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
    /// Raises the world flag with this name, once and for good (WP-061). Pulling it again
    /// leaves the flag set. A pool's `lowered_by` and an enemy's `wakes_on` name such flags:
    /// the Sunken Ossuary's drain levers and the Drowned Belfry's sluice
    /// (`sluice.mirrormere.open`) work this way.
    Flag(String),
    /// A bell (WP-061): ringing it stuns every living enemy within `radius_m` for
    /// `stun_ticks` (they kneel, open to a riposte) and, if `opens` names a door, opens that
    /// door for good. Ringing is the same `INTERACT` as pulling a lever.
    Bell {
        radius_m: f32,
        stun_ticks: u8,
        #[serde(default)]
        opens: Option<String>,
    },
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
    /// A hit doesn't reveal it: only a mine cart pushed into it breaks it (WP-062). A
    /// sturdy wall must be the target of a `Push` cart.
    #[serde(default)]
    pub sturdy: bool,
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
    /// The ground level under the pool, in meters (WP-060). The water's surface is
    /// `floor_y_m + depth`; the core only reads the depth, so this is for the renderer.
    /// Defaults to 0, a pool lying on flat ground (the surface is then `depth` above it).
    #[serde(default)]
    pub floor_y_m: f32,
    /// A world flag that lowers the water (WP-060): while it is set, the pool's depth is
    /// `lowered_depth_m`. The Drowned Belfry's sluice sets `sluice.mirrormere.open`. Needs
    /// `lowered_depth_m`.
    #[serde(default)]
    pub lowered_by: Option<String>,
    /// The depth while `lowered_by` is set, in meters.
    #[serde(default)]
    pub lowered_depth_m: Option<f32>,
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
    /// A world flag that wakes it (WP-061). Until the flag is set the enemy stands where it
    /// is, unaware and not thinking, so it can be backstabbed asleep; once set it fights as
    /// usual. The Sunken Ossuary's Drowned wake when a lever drains the water.
    #[serde(default)]
    pub wakes_on: Option<String>,
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

// ---------------------------------------------------------------------------------------
// Darkness, mine carts, and trapdoors (WP-062; docs/game/02-world.md, "Traversal and world
// mechanics" and "Mini-dungeons", D4)

/// How dark an area is, and what lights it. Outside every glowing zone, and farther than
/// `lantern_light_m` from a Lantern, the player sees `visibility_m` meters
/// (docs/game/02-world.md: 6 m). A carried light (the *smoltorch* relic, which raises
/// `torch_flag`) or a Lantern Flare raises it to `torch_visibility_m`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Darkness {
    /// Visibility in the dark with no light, in meters.
    pub visibility_m: f32,
    /// Visibility in the dark with a carried light, in meters.
    pub torch_visibility_m: f32,
    /// A Lantern lights the ground this far around it, in meters.
    pub lantern_light_m: f32,
    /// The world flag that means the player carries a light: the codex flag the *smoltorch*
    /// relic pickup raises (`codex.projects.smoltorch`).
    pub torch_flag: String,
}

/// What a mine cart does.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CartKind {
    /// The player rides it from one end of its rail to the other: `INTERACT` at the end
    /// the cart waits at, and again at the far end to ride back. Never strands the player.
    Ride,
    /// The player pushes it (`INTERACT` beside it) and it rolls to the end of its rail,
    /// where it breaks the hollow wall with this ID. A pushed cart stays at the end.
    Push(String),
}

/// A mine cart on a rail (D4, Cinderdeep Mine). The cart waits at the first rail point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cart {
    /// Unique among the scenario's carts.
    pub id: String,
    /// The rail as (x, y, z) points in meters (feet level), at least two. The cart rolls
    /// along the straight segments between them.
    pub rail: Vec<[f32; 3]>,
    pub kind: CartKind,
    /// How fast it rolls, in meters per second.
    pub speed_mps: f32,
}

/// A trapdoor: a hatch overhead that the player pushes open from below (the flag
/// `trapdoor.<id>.open`). One way: it can't be opened from above. Opening it lowers the
/// ladder it names, so the Root Stair's ladder climbs to the Spire's underside.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trapdoor {
    /// Unique among the scenario's trapdoors.
    pub id: String,
    /// The middle of the hatch's underside, in meters.
    pub pos: [f32; 3],
    /// Half the hatch's size along x and z, in meters. It slides -x when it opens.
    pub half_extents: [f32; 2],
    /// Where the player stands to push it open (feet), in meters.
    pub stand: [f32; 3],
    /// The kick-down ladder that comes down when the hatch opens, if any.
    #[serde(default)]
    pub ladder: Option<String>,
}

// ---------------------------------------------------------------------------------------
// Low ceilings and light beams (WP-063; docs/game/02-world.md, "Mini-dungeons", D1 and D5)

/// A low-ceiling zone (D1, Mossgut Cave). While an actor stands inside the rectangle, a hitbox
/// of theirs that reaches higher than `height_m` above their feet strikes the rock instead of
/// what is in front of them, so the heavy, jumping, and plunging swings (all of which rise
/// over 2 m) fail and the thrusts and the horizontal swings (all of which stay under 1.6 m)
/// work. The cave's roof slabs in `layout.ron` sit at the same height.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LowCeiling {
    /// Label for tools and error messages.
    pub name: String,
    /// Center, as (x, z) in meters.
    pub center: [f32; 2],
    /// Half the size along x and z, in meters.
    pub half_extents: [f32; 2],
    /// The ceiling's height above the floor, in meters.
    pub height_m: f32,
}

/// A beam emitter (D5, Glasswing Grotto): a crystal that sends a light beam along one of the
/// four compass directions, always on. The beam runs level at `pos`'s height until it meets a
/// mirror, a receptor, or a wall.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeamSource {
    /// Unique among the scenario's emitters.
    pub id: String,
    /// The crystal's center, in meters. The beam's height is `pos.y`.
    pub pos: [f32; 3],
    /// The way it shines: 0 toward +Z, 1 toward +X, 2 toward -Z, 3 toward -X.
    pub dir: u8,
}

/// A crystal mirror (D5): the player turns it a quarter turn with `INTERACT`. It reflects a
/// beam that strikes its face and swallows one that strikes its back.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mirror {
    /// Unique among the scenario's mirrors.
    pub id: String,
    /// The crystal's center, in meters. The player turns it from beside it.
    pub pos: [f32; 3],
    /// Which way its face points at the start: 0 north-east (+X +Z), 1 south-east (+X -Z),
    /// 2 south-west, 3 north-west. A beam traveling against the face's direction bounces off.
    pub facing: u8,
}

/// A beam receptor (D5): a crystal socket. When a beam reaches it, the flag
/// `receptor.<id>.lit` is raised for good and the door it names opens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receptor {
    /// Unique among the scenario's receptors.
    pub id: String,
    /// The socket's center, in meters.
    pub pos: [f32; 3],
    /// The door that opens when it is lit, if any.
    #[serde(default)]
    pub opens: Option<String>,
}

/// A Warden Brazier (D6, Whitewarden Catacombs): the player lights it with `INTERACT` (the flag
/// `brazier.<id>.lit`, for good). A Whitewarden Skeleton that dies within its revive rule's
/// `LitLantern` radius of a lit brazier stays down, and one already down when the brazier is lit
/// stays down from then on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brazier {
    /// Unique among the scenario's braziers.
    pub id: String,
    /// Where the bowl stands (feet level), in meters. The player lights it from beside it.
    pub pos: [f32; 3],
}

/// A seal sigil (D7, the Bound Ring): the player touches it with `INTERACT`, which raises
/// `sigil.<id>.touched`. If it is `armed`, that also raises `sigil.<id>.sealed` and shuts every
/// door in `doors`: they stand open until then, and open again when the world raises
/// `sigil.<id>.released` (M7's B11 raises it when the prisoner falls). An unarmed sigil is cold: it
/// can be touched and nothing shuts. B11 has no definition yet, so the Ring's sigil is unarmed;
/// arm it in the same change that places the boss.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sigil {
    /// Unique among the scenario's sigils.
    pub id: String,
    /// Where the player stands to touch it (feet level), in meters.
    pub pos: [f32; 3],
    /// The doors the seal shuts. Each must exist.
    pub doors: Vec<String>,
    /// Whether touching it seals anything yet.
    #[serde(default)]
    pub armed: bool,
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
        for (i, s) in self.echo_stones.iter().enumerate() {
            unique(
                self.echo_stones[..i].iter().any(|o| o.id == s.id),
                "echo stone",
                &s.id,
            )?;
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
            if let DoorKind::Held { items, need } = &d.kind
                && (*need == 0 || usize::from(*need) > items.len())
            {
                return Err(format!(
                    "door {:?}: need must be between 1 and the number of items",
                    d.id
                ));
            }
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
                LeverTarget::Flag(flag) => !flag.is_empty(),
                LeverTarget::Bell {
                    radius_m,
                    stun_ticks,
                    opens,
                } => {
                    *radius_m > 0.0
                        && *stun_ticks > 0
                        && opens
                            .as_ref()
                            .is_none_or(|id| self.doors.iter().any(|d| &d.id == id))
                }
            };
            if !ok {
                return Err(format!(
                    "lever {:?}: its target doesn't exist (a lift stop is 0 or 1, and a water \
                     target needs a pool with raised_depth_m, a flag lever a flag name, and a bell \
                     a positive radius and stun and a door that exists)",
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
        for (i, n) in self.npcs.iter().enumerate() {
            unique(self.npcs[..i].iter().any(|o| o.npc == n.npc), "npc", &n.npc)?;
            positive("npc", &n.npc, "radius_m", n.radius_m)?;
        }
        for (i, n) in self.stations.iter().enumerate() {
            unique(
                self.stations[..i].iter().any(|o| o.station == n.station),
                "station",
                &n.station,
            )?;
            positive("station", &n.station, "radius_m", n.radius_m)?;
        }
        if let Some(d) = &self.darkness {
            let ok = d.visibility_m > 0.0
                && d.torch_visibility_m >= d.visibility_m
                && d.lantern_light_m > 0.0
                && !d.torch_flag.is_empty();
            if !ok {
                return Err(
                    "darkness: visibility_m and lantern_light_m must be positive, \
                     torch_visibility_m at least visibility_m, and torch_flag a flag"
                        .to_owned(),
                );
            }
        }
        for (i, g) in self.glows.iter().enumerate() {
            unique(
                self.glows[..i].iter().any(|o| o.name == g.name),
                "glow",
                &g.name,
            )?;
            positive(
                "glow",
                &g.name,
                "half_extents",
                g.half_extents[0].min(g.half_extents[1]),
            )?;
        }
        for (i, c) in self.carts.iter().enumerate() {
            unique(self.carts[..i].iter().any(|o| o.id == c.id), "cart", &c.id)?;
            positive("cart", &c.id, "speed_mps", c.speed_mps)?;
            if c.rail.len() < 2 || c.rail.windows(2).any(|s| s[0] == s[1]) {
                return Err(format!(
                    "cart {:?}: the rail needs two or more distinct points",
                    c.id
                ));
            }
            if let CartKind::Push(wall) = &c.kind
                && !self.hollow_walls.iter().any(|w| &w.id == wall)
            {
                return Err(format!(
                    "cart {:?}: it breaks hollow wall {wall:?}, which doesn't exist",
                    c.id
                ));
            }
        }
        if let Some(w) = self.hollow_walls.iter().find(|w| {
            w.sturdy
                && !self
                    .carts
                    .iter()
                    .any(|c| matches!(&c.kind, CartKind::Push(id) if id == &w.id))
        }) {
            return Err(format!(
                "hollow wall {:?} is sturdy, so a Push cart has to break it",
                w.id
            ));
        }
        for (i, t) in self.trapdoors.iter().enumerate() {
            unique(
                self.trapdoors[..i].iter().any(|o| o.id == t.id),
                "trapdoor",
                &t.id,
            )?;
            positive(
                "trapdoor",
                &t.id,
                "half_extents",
                t.half_extents[0].min(t.half_extents[1]),
            )?;
            if let Some(l) = &t.ladder
                && !self.ladders.iter().any(|d| &d.id == l && d.kick_down)
            {
                return Err(format!(
                    "trapdoor {:?}: its ladder {l:?} must be a kick_down ladder",
                    t.id
                ));
            }
        }
        for (i, z) in self.low_ceilings.iter().enumerate() {
            unique(
                self.low_ceilings[..i].iter().any(|o| o.name == z.name),
                "low ceiling",
                &z.name,
            )?;
            positive(
                "low ceiling",
                &z.name,
                "half_extents",
                z.half_extents[0].min(z.half_extents[1]),
            )?;
            positive("low ceiling", &z.name, "height_m", z.height_m)?;
        }
        for (i, s) in self.beam_sources.iter().enumerate() {
            unique(
                self.beam_sources[..i].iter().any(|o| o.id == s.id),
                "beam source",
                &s.id,
            )?;
            if s.dir > 3 {
                return Err(format!("beam source {:?}: dir must be 0 to 3", s.id));
            }
        }
        for (i, m) in self.mirrors.iter().enumerate() {
            unique(
                self.mirrors[..i].iter().any(|o| o.id == m.id),
                "mirror",
                &m.id,
            )?;
            if m.facing > 3 {
                return Err(format!("mirror {:?}: facing must be 0 to 3", m.id));
            }
        }
        for (i, r) in self.receptors.iter().enumerate() {
            unique(
                self.receptors[..i].iter().any(|o| o.id == r.id),
                "receptor",
                &r.id,
            )?;
            if let Some(d) = &r.opens
                && !self.doors.iter().any(|o| &o.id == d)
            {
                return Err(format!(
                    "receptor {:?}: it opens door {d:?}, which doesn't exist",
                    r.id
                ));
            }
        }
        for (i, b) in self.braziers.iter().enumerate() {
            unique(
                self.braziers[..i].iter().any(|o| o.id == b.id),
                "brazier",
                &b.id,
            )?;
        }
        for (i, g) in self.sigils.iter().enumerate() {
            unique(
                self.sigils[..i].iter().any(|o| o.id == g.id),
                "sigil",
                &g.id,
            )?;
            if let Some(d) = g
                .doors
                .iter()
                .find(|d| !self.doors.iter().any(|o| &o.id == *d))
            {
                return Err(format!(
                    "sigil {:?}: it seals door {d:?}, which doesn't exist",
                    g.id
                ));
            }
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
            if p.lowered_by.is_some() != p.lowered_depth_m.is_some()
                || p.lowered_by.as_deref() == Some("")
                || p.lowered_depth_m.is_some_and(|d| d <= 0.0)
            {
                return Err(format!(
                    "pool {:?}: lowered_by (a flag) and a positive lowered_depth_m go together",
                    p.name
                ));
            }
        }
        Ok(())
    }
}
