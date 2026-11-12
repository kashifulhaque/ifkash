//! Levels: areas built from data, and the kit catalogs they place
//! (docs/game/01-architecture.md, "Levels"; docs/game/02-world.md, "Build an area").
//!
//! # Format
//!
//! Each area is a directory `levels/<id>/` with up to five RON files:
//!
//! - `area.ron` ([`Area`], required): the area's ID, seed, bounds, kill height, ambient
//!   settings, music, named region polygons, and connections to other areas.
//! - `terrain.ron` ([`Terrain`], optional): a heightfield covering the area's bounds,
//!   built from a base height, seeded noise, and [`Sculpt`] operations applied in order.
//!   Without it the area has no terrain (an indoor area built from kit pieces).
//! - `layout.ron` ([`Layout`], optional): kit pieces snapped to the 2-meter grid, plus
//!   freely placed and scaled landmarks.
//! - `entities.ron` ([`Entities`], required): the player spawn and a list of [`Entity`]
//!   values: Lanterns, dummies, enemies, bosses with their fog gates, water, reeds, and the
//!   world mechanics (ladders, lifts, doors, levers, hollow walls, pickups, chests,
//!   messages, pools, and snow).
//! - `dressing.ron` ([`Dressing`], optional): scatter rules and a few hand-placed props.
//!
//! Kit catalogs live in `kits/<name>.ron` ([`KitCatalog`]). Each piece has a model path
//! for the renderer and simplified collider shapes for the core, so the core never
//! loads a glTF file. Piece IDs are unique across all catalogs and registered in
//! `registry.ron`.
//!
//! RON enums are written as `Name(field: value, ...)`: the loader turns on RON's
//! `unwrap_variant_newtypes` extension, so an entity reads `Lantern(id: "x", ...)`
//! rather than `Lantern((id: "x", ...))`. It also turns on `implicit_some`, so an
//! optional field can be written without `Some(...)`.
//!
//! # Compiling
//!
//! [`Level::compile`] turns a level into the [`Scenario`] the core runs: the terrain
//! becomes the heightfield, every kit collider in the layout (and in scatter rules marked
//! `collider: true`) becomes a box, and the entities fill the spawn lists. The content
//! loader compiles every level, so `Content::scenario("<id>")` finds a level the same
//! way it finds a file in `scenarios/`. `lf-tools bake` runs the same code, then bakes
//! the navigation grid, writes a render manifest, and reports warnings.
//!
//! Everything here is deterministic on every platform: plain `f32` arithmetic, `floor`,
//! and `libm`, with no square roots (distances are compared squared) and an integer RNG
//! seeded by the area's seed.
//!
//! # Extending
//!
//! A new entity kind is a new [`Entity`] variant, a line in [`Entity::kind`],
//! [`Entity::label`], and [`Entity::anchor`] (so the bake checks it), and whatever
//! [`Level::compile`] has to hand the core (a list on [`Scenario`]). WP-042 added the world
//! mechanics this way: ladders, lifts, doors, levers, hollow walls, pickups, chests,
//! messages, pools, and snow. A new sculpt operation is a new [`Sculpt`] variant and a match arm in
//! [`Sculpt::apply`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Scenario;
use crate::scenario::{
    BossSpawn, BoxCollider, Chest, Door, Dummy, EnemySpawn, Heightfield, HollowWall, Ladder,
    Lantern, Lever, LeverTarget, Lift, Message, NpcSpawn, Pickup, Pool, StationSpawn, Zone,
};

/// The files an area directory can hold, in load order.
pub const LEVEL_FILES: [&str; 5] = [
    "area.ron",
    "terrain.ron",
    "layout.ron",
    "entities.ron",
    "dressing.ron",
];

/// Kit pieces in `layout.ron` snap to a grid of this many meters (docs/game/01-architecture.md,
/// "Levels").
pub const GRID_M: f32 = 2.0;

/// The most hand-placed decorative props an area should have (docs/game/02-world.md,
/// "Procedural dressing"); the bake warns past it.
pub const MAX_HAND_PLACED_PROPS: usize = 50;

/// The most instances one scatter rule may produce, so a typo in a density can't
/// produce a manifest of millions of props.
pub const MAX_SCATTER_PER_RULE: usize = 20_000;

/// Darts thrown per wanted instance when Poisson-disk sampling a scatter rule.
const DARTS_PER_INSTANCE: usize = 8;

/// Slack when testing whether a terrain vertex is inside a sculpt rectangle, in meters.
const VERTEX_EPS: f32 = 1.0e-4;

/// Parses level or kit RON with the `unwrap_variant_newtypes` and `implicit_some`
/// extensions.
pub fn parse_ron<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, String> {
    use ron::extensions::Extensions;
    ron::Options::default()
        .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES | Extensions::IMPLICIT_SOME)
        .from_str(text)
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------------------
// area.ron

/// `area.ron`: what the area is and how it connects to the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Area {
    /// The area's ID. Must match its directory name and have a `registry.ron` entry.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Seeds the terrain noise and the scatter rules.
    pub seed: u64,
    /// The area's extent on the ground plane. The terrain covers exactly this rectangle.
    pub bounds: Bounds,
    /// An actor whose feet drop below this height dies.
    pub kill_y: f32,
    /// Fog, sky, and light for the renderer. The core ignores it.
    #[serde(default)]
    pub ambient: Ambient,
    /// The area's music track ID, if any. The core ignores it.
    #[serde(default)]
    pub music: Option<String>,
    /// Named polygons, for scatter masks (`region_mask`) and region lighting.
    #[serde(default)]
    pub regions: Vec<Region>,
    /// Exits into other areas.
    #[serde(default)]
    pub connections: Vec<Connection>,
    /// Glimmer the player starts with. Zero in real areas; test areas set it.
    #[serde(default)]
    pub start_glimmer: u32,
    /// A weapon ID the player starts wielding instead of the default weapon.
    #[serde(default)]
    pub player_weapon: Option<String>,
    /// Spell IDs the player knows from the start (test areas).
    #[serde(default)]
    pub player_spells: Vec<String>,
    /// A safe place (WP-068): no combat, weapons sheathed, walk only (Hollowhearth).
    #[serde(default)]
    pub safe_zone: bool,
}

/// An axis-aligned rectangle on the ground plane, as (x, z) corners in meters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl Bounds {
    pub fn contains(&self, x: f32, z: f32) -> bool {
        x >= self.min[0] && x <= self.max[0] && z >= self.min[1] && z <= self.max[1]
    }

    fn area(&self) -> f32 {
        (self.max[0] - self.min[0]) * (self.max[1] - self.min[1])
    }
}

/// Render-only atmosphere. Colors are linear RGB in 0..1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Ambient {
    /// A sky preset name the renderer knows.
    pub sky: String,
    pub fog_color: [f32; 3],
    /// Fog starts at this distance from the camera, in meters.
    pub fog_near_m: f32,
    /// Fog is opaque at this distance, in meters.
    pub fog_far_m: f32,
    /// Direction the sunlight travels (pointing away from the sun).
    pub sun_dir: [f32; 3],
    pub sun_color: [f32; 3],
    pub sun_intensity: f32,
    pub ambient_color: [f32; 3],
    pub ambient_intensity: f32,
}

impl Default for Ambient {
    fn default() -> Self {
        Self {
            sky: "day".to_owned(),
            fog_color: [0.75, 0.8, 0.85],
            fog_near_m: 40.0,
            fog_far_m: 160.0,
            sun_dir: [-0.4, -1.0, -0.3],
            sun_color: [1.0, 0.96, 0.9],
            sun_intensity: 1.0,
            ambient_color: [0.6, 0.65, 0.75],
            ambient_intensity: 0.5,
        }
    }
}

/// A named polygon on the ground plane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub name: String,
    /// Corners as (x, z) in meters, in order. At least three.
    pub points: Vec<[f32; 2]>,
}

impl Region {
    /// Even-odd point-in-polygon test.
    pub fn contains(&self, x: f32, z: f32) -> bool {
        let n = self.points.len();
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let [xi, zi] = self.points[i];
            let [xj, zj] = self.points[j];
            if (zi > z) != (zj > z) && x < (xj - xi) * (z - zi) / (zj - zi) + xi {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    /// The polygon's area in square meters (shoelace formula).
    pub fn area(&self) -> f32 {
        let n = self.points.len();
        let mut twice = 0.0;
        for i in 0..n {
            let [x0, z0] = self.points[i];
            let [x1, z1] = self.points[(i + 1) % n];
            twice += x0 * z1 - x1 * z0;
        }
        (twice * 0.5).abs()
    }

    fn bounds(&self) -> Bounds {
        let mut b = Bounds {
            min: [f32::INFINITY; 2],
            max: [f32::NEG_INFINITY; 2],
        };
        for &[x, z] in &self.points {
            b.min = [b.min[0].min(x), b.min[1].min(z)];
            b.max = [b.max[0].max(x), b.max[1].max(z)];
        }
        b
    }
}

/// An exit into another area: walking into the sphere loads `to`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    /// The other area's ID.
    pub to: String,
    /// Where the exit is, in meters (feet height).
    pub pos: [f32; 3],
    pub radius_m: f32,
}

// ---------------------------------------------------------------------------------------
// terrain.ron

/// `terrain.ron`: a heightfield over the area's bounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terrain {
    /// Distance between heightfield vertices, in meters. The bounds' width and depth must
    /// be whole multiples of it.
    pub spacing_m: f32,
    /// Starting height of every vertex, in meters.
    #[serde(default)]
    pub base_y: f32,
    /// Seeded value noise added to the base height before sculpting.
    #[serde(default)]
    pub noise: Option<Noise>,
    /// Sculpt operations, applied in order.
    #[serde(default)]
    pub ops: Vec<Sculpt>,
}

/// Fractal value noise. Each octave halves the amplitude and the wavelength.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Noise {
    /// Peak height of the first octave, in meters.
    pub amplitude_m: f32,
    /// Feature size of the first octave, in meters.
    pub wavelength_m: f32,
    /// 1 to 8.
    pub octaves: u8,
}

/// An axis on the ground plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    X,
    Z,
}

/// A terrain edit. Positions are (x, z) in meters; heights are absolute, in meters.
/// "Within" a rectangle includes its edges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Sculpt {
    /// Sets every vertex within the rectangle to `height`.
    SetRect {
        min: [f32; 2],
        max: [f32; 2],
        height: f32,
    },
    /// Within the rectangle, a straight slope from `from` at the rectangle's low edge
    /// along `axis` to `to` at its high edge.
    Ramp {
        min: [f32; 2],
        max: [f32; 2],
        axis: Axis,
        from: f32,
        to: f32,
    },
    /// Adds a smooth dome (or a pit, if `height` is negative) of `radius` meters.
    RaiseCircle {
        center: [f32; 2],
        radius: f32,
        height: f32,
    },
    /// Sets every vertex within `width / 2` of the polyline to `height`. Scatter rules
    /// can avoid these paths.
    FlattenPath {
        points: Vec<[f32; 2]>,
        width: f32,
        height: f32,
    },
    /// Lowers a channel `width` meters wide along the polyline, `depth` meters at its
    /// center and smoothly less toward its banks.
    CarveRiver {
        points: Vec<[f32; 2]>,
        width: f32,
        depth: f32,
    },
    /// Raises the vertices on the left of the segment `from -> to` (looking from `from`
    /// toward `to`), within `width` meters of it and between its ends, by `height`: a
    /// sheer step one vertex wide.
    CliffLine {
        from: [f32; 2],
        to: [f32; 2],
        width: f32,
        height: f32,
    },
}

impl Sculpt {
    fn name(&self) -> &'static str {
        match self {
            Self::SetRect { .. } => "SetRect",
            Self::Ramp { .. } => "Ramp",
            Self::RaiseCircle { .. } => "RaiseCircle",
            Self::FlattenPath { .. } => "FlattenPath",
            Self::CarveRiver { .. } => "CarveRiver",
            Self::CliffLine { .. } => "CliffLine",
        }
    }

    fn check(&self) -> Result<(), String> {
        let positive = |what: &str, v: f32| {
            if v.is_finite() && v > 0.0 {
                Ok(())
            } else {
                Err(format!("{what} must be positive"))
            }
        };
        let rect = |min: [f32; 2], max: [f32; 2]| {
            if min[0] <= max[0] && min[1] <= max[1] {
                Ok(())
            } else {
                Err("min must not exceed max".to_owned())
            }
        };
        match self {
            Self::SetRect { min, max, .. } => rect(*min, *max),
            Self::Ramp { min, max, axis, .. } => {
                rect(*min, *max)?;
                let k = axis_index(*axis);
                if max[k] > min[k] {
                    Ok(())
                } else {
                    Err("the rectangle has no length along the ramp's axis".to_owned())
                }
            }
            Self::RaiseCircle { radius, .. } => positive("radius", *radius),
            Self::FlattenPath { points, width, .. } | Self::CarveRiver { points, width, .. } => {
                positive("width", *width)?;
                if points.is_empty() {
                    Err("a path needs at least one point".to_owned())
                } else {
                    Ok(())
                }
            }
            Self::CliffLine {
                from, to, width, ..
            } => {
                positive("width", *width)?;
                if from == to {
                    Err("from and to must differ".to_owned())
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Applies the operation to the vertex at (x, z) whose current height is `h`.
    fn apply(&self, x: f32, z: f32, h: f32) -> f32 {
        let in_rect = |min: [f32; 2], max: [f32; 2]| {
            x >= min[0] - VERTEX_EPS
                && x <= max[0] + VERTEX_EPS
                && z >= min[1] - VERTEX_EPS
                && z <= max[1] + VERTEX_EPS
        };
        match self {
            Self::SetRect { min, max, height } => {
                if in_rect(*min, *max) {
                    *height
                } else {
                    h
                }
            }
            Self::Ramp {
                min,
                max,
                axis,
                from,
                to,
            } => {
                if !in_rect(*min, *max) {
                    return h;
                }
                let k = axis_index(*axis);
                let c = [x, z][k].clamp(min[k], max[k]);
                // Multiply before dividing so whole-meter ramps land on exact heights.
                from + (to - from) * (c - min[k]) / (max[k] - min[k])
            }
            Self::RaiseCircle {
                center,
                radius,
                height,
            } => {
                let d2 = sq(x - center[0]) + sq(z - center[1]);
                let r2 = radius * radius;
                if d2 >= r2 {
                    h
                } else {
                    let t = 1.0 - d2 / r2;
                    h + height * t * t
                }
            }
            Self::FlattenPath {
                points,
                width,
                height,
            } => {
                if polyline_dist2(points, x, z) <= sq(width * 0.5) {
                    *height
                } else {
                    h
                }
            }
            Self::CarveRiver {
                points,
                width,
                depth,
            } => {
                let d2 = polyline_dist2(points, x, z);
                let r2 = sq(width * 0.5);
                if d2 >= r2 {
                    h
                } else {
                    h - depth * (1.0 - d2 / r2)
                }
            }
            Self::CliffLine {
                from,
                to,
                width,
                height,
            } => {
                let (dx, dz) = (to[0] - from[0], to[1] - from[1]);
                let (px, pz) = (x - from[0], z - from[1]);
                let len2 = dx * dx + dz * dz;
                let along = px * dx + pz * dz;
                // Left of the direction of travel, with yaw measured from +Z toward +X,
                // is the +X side when facing +Z: cross(d, p) > 0.
                let cross = dz * px - dx * pz;
                if (0.0..=len2).contains(&along)
                    && cross > 0.0
                    && cross * cross <= sq(*width) * len2
                {
                    h + height
                } else {
                    h
                }
            }
        }
    }
}

fn axis_index(axis: Axis) -> usize {
    match axis {
        Axis::X => 0,
        Axis::Z => 1,
    }
}

fn sq(v: f32) -> f32 {
    v * v
}

/// Squared distance from (x, z) to a polyline (one point is a dot).
fn polyline_dist2(points: &[[f32; 2]], x: f32, z: f32) -> f32 {
    if points.len() == 1 {
        return sq(x - points[0][0]) + sq(z - points[0][1]);
    }
    points
        .windows(2)
        .map(|w| segment_dist2(w[0], w[1], x, z))
        .fold(f32::INFINITY, f32::min)
}

fn segment_dist2(a: [f32; 2], b: [f32; 2], x: f32, z: f32) -> f32 {
    let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
    let len2 = dx * dx + dz * dz;
    let t = if len2 > 0.0 {
        (((x - a[0]) * dx + (z - a[1]) * dz) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    sq(x - (a[0] + t * dx)) + sq(z - (a[1] + t * dz))
}

impl Terrain {
    /// Builds the heightfield over `bounds`.
    pub fn heightfield(&self, bounds: &Bounds, seed: u64) -> Result<Heightfield, String> {
        let s = self.spacing_m;
        if !(s.is_finite() && s > 0.0) {
            return Err("spacing_m must be positive".to_owned());
        }
        let size_x = bounds.max[0] - bounds.min[0];
        let size_z = bounds.max[1] - bounds.min[1];
        let steps = |size: f32, what: &str| -> Result<u32, String> {
            let n = size / s;
            if n < 1.0 || n.fract() != 0.0 || n > 4096.0 {
                return Err(format!(
                    "the bounds' {what} ({size} m) must be a whole multiple of spacing_m ({s} m), at most 4096 steps"
                ));
            }
            Ok(n as u32)
        };
        let cols = steps(size_x, "width along x")? + 1;
        let rows = steps(size_z, "depth along z")? + 1;
        for (i, op) in self.ops.iter().enumerate() {
            op.check()
                .map_err(|e| format!("ops[{i}] ({}): {e}", op.name()))?;
        }
        if let Some(n) = &self.noise
            && (!(1..=8).contains(&n.octaves)
                || !(n.wavelength_m.is_finite() && n.wavelength_m > 0.0)
                || !n.amplitude_m.is_finite())
        {
            return Err(
                "noise needs 1 to 8 octaves, a positive wavelength, and a finite amplitude"
                    .to_owned(),
            );
        }
        let mut heights = Vec::with_capacity(rows as usize * cols as usize);
        for row in 0..rows {
            let z = bounds.min[1] + row as f32 * s;
            for col in 0..cols {
                let x = bounds.min[0] + col as f32 * s;
                let mut h = self.base_y;
                if let Some(n) = &self.noise
                    && n.amplitude_m != 0.0
                {
                    h += fractal_noise(n, seed, x, z);
                }
                for op in &self.ops {
                    h = op.apply(x, z, h);
                }
                heights.push(h);
            }
        }
        Ok(Heightfield {
            center: [
                (bounds.min[0] + bounds.max[0]) * 0.5,
                0.0,
                (bounds.min[1] + bounds.max[1]) * 0.5,
            ],
            rows,
            cols,
            size_x,
            size_z,
            heights,
        })
    }
}

fn fractal_noise(n: &Noise, seed: u64, x: f32, z: f32) -> f32 {
    let mut sum = 0.0;
    let mut amp = n.amplitude_m;
    let mut wave = n.wavelength_m;
    for octave in 0..n.octaves {
        sum += amp
            * value_noise(
                seed ^ (u64::from(octave) + 1).wrapping_mul(GOLDEN),
                x / wave,
                z / wave,
            );
        amp *= 0.5;
        wave *= 0.5;
    }
    sum
}

/// Smoothly interpolated lattice noise in -1..1, with lattice points at whole coordinates.
fn value_noise(seed: u64, x: f32, z: f32) -> f32 {
    let (fx, fz) = (x.floor(), z.floor());
    let (ix, iz) = (fx as i32, fz as i32);
    let (tx, tz) = (smooth(x - fx), smooth(z - fz));
    let at = |dx: i32, dz: i32| {
        let key = seed
            ^ u64::from(ix.wrapping_add(dx) as u32).wrapping_mul(0x9E37_79B9)
            ^ (u64::from(iz.wrapping_add(dz) as u32) << 32);
        unit(splitmix(key)) * 2.0 - 1.0
    };
    let a = at(0, 0) + (at(1, 0) - at(0, 0)) * tx;
    let b = at(0, 1) + (at(1, 1) - at(0, 1)) * tx;
    a + (b - a) * tz
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(GOLDEN);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The top 24 bits of `v` as a float in 0..1 (exact).
fn unit(v: u64) -> f32 {
    (v >> 40) as f32 / 16_777_216.0
}

/// A SplitMix64 stream for scatter sampling.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(GOLDEN);
        splitmix(self.0)
    }

    fn unit(&mut self) -> f32 {
        unit(self.next())
    }
}

/// Terrain height at (x, z), bilinear between the four surrounding vertices. Off the
/// heightfield it clamps to the nearest edge.
pub fn sample_height(h: &Heightfield, x: f32, z: f32) -> f32 {
    sample(h, x, z).0
}

/// Height and gradient (dh/dx, dh/dz) at (x, z).
fn sample(h: &Heightfield, x: f32, z: f32) -> (f32, f32, f32) {
    let sx = h.size_x / (h.cols - 1) as f32;
    let sz = h.size_z / (h.rows - 1) as f32;
    let gx = ((x - (h.center[0] - h.size_x * 0.5)) / sx).clamp(0.0, (h.cols - 1) as f32);
    let gz = ((z - (h.center[2] - h.size_z * 0.5)) / sz).clamp(0.0, (h.rows - 1) as f32);
    let c0 = (gx.floor() as u32).min(h.cols - 2);
    let r0 = (gz.floor() as u32).min(h.rows - 2);
    let (tx, tz) = (gx - c0 as f32, gz - r0 as f32);
    let at = |r: u32, c: u32| h.heights[(r * h.cols + c) as usize];
    let (h00, h01) = (at(r0, c0), at(r0, c0 + 1));
    let (h10, h11) = (at(r0 + 1, c0), at(r0 + 1, c0 + 1));
    let a = h00 + (h01 - h00) * tx;
    let b = h10 + (h11 - h10) * tx;
    let height = h.center[1] + a + (b - a) * tz;
    let dx = ((h01 - h00) + ((h11 - h10) - (h01 - h00)) * tz) / sx;
    let dz = (b - a) / sz;
    (height, dx, dz)
}

// ---------------------------------------------------------------------------------------
// layout.ron

/// `layout.ron`: kit pieces on the grid and free landmarks.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Layout {
    pub pieces: Vec<Piece>,
    pub landmarks: Vec<Landmark>,
}

/// A kit piece on the 2-meter grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Piece {
    /// The kit piece ID.
    pub kit: String,
    /// Grid cell as (x, z): the piece's origin is at (x * 2, y, z * 2) meters.
    pub cell: [i32; 2],
    /// Height of the piece's origin, in meters.
    #[serde(default)]
    pub y: f32,
    /// Quarter turns from +Z toward +X, 0 to 3.
    #[serde(default)]
    pub rot: u8,
    /// Label for colliders and tools. Defaults to `<kit>@<x>,<z>`.
    #[serde(default)]
    pub name: Option<String>,
}

/// A freely placed, scalable kit piece: landmarks, and greybox blocks of any size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Landmark {
    /// Label for colliders and tools. Unique in the layout.
    pub name: String,
    pub kit: String,
    /// The piece's origin, in meters.
    pub pos: [f32; 3],
    /// Quarter turns from +Z toward +X, 0 to 3. Colliders stay axis-aligned, which is
    /// why rotation is limited to quarter turns.
    #[serde(default)]
    pub rot: u8,
    /// Scale along the piece's own x, y, and z. Defaults to 1.
    #[serde(default = "unit_scale")]
    pub scale: [f32; 3],
}

fn unit_scale() -> [f32; 3] {
    [1.0; 3]
}

/// Where a placed piece came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Piece,
    Landmark,
    Prop,
    Scatter,
}

/// A kit piece placed in the world, after grid snapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    pub name: String,
    /// The kit piece ID.
    pub piece: String,
    pub pos: [f32; 3],
    /// Yaw in radians from +Z toward +X.
    pub yaw: f32,
    pub scale: [f32; 3],
    pub source: Source,
}

// ---------------------------------------------------------------------------------------
// entities.ron

/// `entities.ron`: where the player starts, and everything else that isn't scenery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entities {
    pub spawn: Spawn,
    #[serde(default)]
    pub entities: Vec<Entity>,
}

/// The player's start: feet position in meters and facing in radians from +Z toward +X.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spawn {
    pub pos: [f32; 3],
    pub yaw: f32,
}

/// One thing in the level. The variants reuse the scenario types, so their fields are
/// the same as in `scenarios/*.ron`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Entity {
    /// A Lantern: rest and respawn point.
    Lantern(Lantern),
    /// A training dummy.
    Dummy(Dummy),
    /// An AI enemy spawn.
    Enemy(EnemySpawn),
    /// A boss and the fog gate into its arena.
    Boss(BossSpawn),
    /// A water volume (where water-only enemies go).
    Water(Zone),
    /// A reed patch (where reed-hiding enemies are hidden).
    Reeds(Zone),
    /// An item lying in the level that unlocks a codex entry (WP-049).
    Pickup(Pickup),
    /// A ladder (WP-042).
    Ladder(Ladder),
    /// A lift: a platform between two stops, called by a lever at each (WP-042).
    Lift(Lift),
    /// A locked, one-way, or keyed door (WP-042).
    Door(Door),
    /// A lever that works a door, a lift, or a pool's water level (WP-042).
    Lever(Lever),
    /// A wall one hit reveals a passage in (WP-042).
    HollowWall(HollowWall),
    /// A treasure chest (WP-042).
    Chest(Chest),
    /// An authored ground message (WP-042).
    Message(Message),
    /// A pool of water: shallow slows, deep blocks, blight builds Blight (WP-042).
    Pool(Pool),
    /// Deep snow: slows and costs more stamina to sprint through (WP-042).
    Snow(Zone),
    /// A person to talk to (WP-068).
    Npc(NpcSpawn),
    /// A station: the Archive shelves, the Ledger Stand, the Mirror (WP-068).
    Station(StationSpawn),
}

impl Entity {
    /// The variant name, for messages.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Lantern(_) => "Lantern",
            Self::Dummy(_) => "Dummy",
            Self::Enemy(_) => "Enemy",
            Self::Boss(_) => "Boss",
            Self::Water(_) => "Water",
            Self::Reeds(_) => "Reeds",
            Self::Pickup(_) => "Pickup",
            Self::Ladder(_) => "Ladder",
            Self::Lift(_) => "Lift",
            Self::Door(_) => "Door",
            Self::Lever(_) => "Lever",
            Self::HollowWall(_) => "HollowWall",
            Self::Chest(_) => "Chest",
            Self::Message(_) => "Message",
            Self::Pool(_) => "Pool",
            Self::Snow(_) => "Snow",
            Self::Npc(_) => "Npc",
            Self::Station(_) => "Station",
        }
    }

    /// The entity's ID or name.
    pub fn label(&self) -> &str {
        match self {
            Self::Lantern(l) => &l.id,
            Self::Dummy(d) => &d.name,
            Self::Enemy(e) => &e.name,
            Self::Boss(b) => &b.name,
            Self::Pickup(p) => &p.id,
            Self::Water(z) | Self::Reeds(z) | Self::Snow(z) => &z.name,
            Self::Ladder(l) => &l.id,
            Self::Lift(l) => &l.id,
            Self::Door(d) => &d.id,
            Self::Lever(l) => &l.id,
            Self::HollowWall(w) => &w.id,
            Self::Chest(c) => &c.id,
            Self::Message(m) => &m.id,
            Self::Pool(p) => &p.name,
            Self::Npc(n) => &n.npc,
            Self::Station(s) => &s.station,
        }
    }

    /// The point an actor stands on or the player walks to, if the entity has one: the
    /// bake checks that it is on the navigation grid and not inside a collider. Zones
    /// have none.
    pub fn anchor(&self) -> Option<[f32; 3]> {
        match self {
            Self::Lantern(l) => Some(l.pos),
            Self::Dummy(d) => Some(d.pos),
            Self::Enemy(e) => Some(e.pos),
            Self::Boss(b) => Some(b.pos),
            Self::Pickup(p) => Some(p.pos),
            Self::Ladder(l) => Some(l.pos),
            Self::Lift(l) => Some([l.pos[0], l.bottom_y, l.pos[1]]),
            Self::Lever(l) => Some(l.pos),
            Self::Chest(c) => Some(c.pos),
            Self::Message(m) => Some(m.pos),
            Self::Npc(n) => Some(n.pos),
            Self::Station(s) => Some(s.pos),
            Self::Water(_)
            | Self::Reeds(_)
            | Self::Snow(_)
            | Self::Pool(_)
            | Self::Door(_)
            | Self::HollowWall(_) => None,
        }
    }

    /// Whether the player must be able to walk to the anchor from the spawn.
    pub fn must_be_reachable(&self) -> bool {
        matches!(self, Self::Lantern(_))
    }
}

// ---------------------------------------------------------------------------------------
// dressing.ron

/// `dressing.ron`: render-side decoration. The core only sees scatter rules marked
/// `collider: true`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Dressing {
    pub rules: Vec<ScatterRule>,
    /// Hand-placed decorative props. Keep these under [`MAX_HAND_PLACED_PROPS`]; use
    /// rules instead. Props never collide.
    pub props: Vec<Prop>,
}

/// Poisson-disk scatter of kit pieces with a tag (docs/game/02-world.md, "Procedural
/// dressing"). Each rule draws from its own stream of the area's seed, keyed by its
/// index in the list, so reordering rules changes where props land.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScatterRule {
    /// Kit catalog tag; each instance picks one piece with the tag.
    pub tag: String,
    /// Wanted instances per square meter of the mask.
    pub density: f32,
    /// No two instances of this rule are closer than this, in meters.
    pub min_spacing: f32,
    /// Steepest ground an instance stands on, in degrees. Defaults to 90 (any).
    #[serde(default = "any_slope")]
    pub max_slope_deg: f32,
    #[serde(default)]
    pub avoid: Vec<Avoid>,
    /// A region name from `area.ron`. Without one the rule covers the whole area.
    #[serde(default)]
    pub region_mask: Option<String>,
    /// Uniform scale range (min, max).
    #[serde(default = "unit_range")]
    pub scale: (f32, f32),
    /// Whether instances get the piece's colliders in the core. Collider instances face a
    /// quarter turn so their boxes stay axis-aligned.
    #[serde(default)]
    pub collider: bool,
}

fn any_slope() -> f32 {
    90.0
}

fn unit_range() -> (f32, f32) {
    (1.0, 1.0)
}

/// Places a scatter rule keeps clear of.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Avoid {
    /// A corridor `width` meters wide centered on each `FlattenPath` in the terrain.
    Path { width: f32 },
    /// Within `radius` meters of a Lantern.
    Lantern { radius: f32 },
    /// Within `radius` meters of the player spawn, an enemy, a dummy, or a boss.
    Spawn { radius: f32 },
    /// Within `margin` meters of a layout collider's footprint.
    Layout { margin: f32 },
}

/// A hand-placed decorative prop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prop {
    pub kit: String,
    pub pos: [f32; 3],
    #[serde(default)]
    pub yaw: f32,
    #[serde(default = "one")]
    pub scale: f32,
}

fn one() -> f32 {
    1.0
}

/// One scatter instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScatterInstance {
    /// Index of the rule in `dressing.ron`.
    pub rule: usize,
    pub piece: String,
    pub pos: [f32; 3],
    pub yaw: f32,
    pub scale: f32,
}

// ---------------------------------------------------------------------------------------
// kits/*.ron

/// A kit catalog file: `kits/<name>.ron`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KitCatalog {
    pub pieces: Vec<KitPiece>,
}

/// One modular piece.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KitPiece {
    /// Unique across all catalogs, registered in `registry.ron`.
    pub id: String,
    /// The glTF file, relative to `static/latticefall/`. `None` for greybox pieces, which
    /// the renderer draws from their colliders.
    #[serde(default)]
    pub model: Option<String>,
    /// Scatter tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Simplified collision, in the piece's own frame (origin at the piece's origin).
    #[serde(default)]
    pub colliders: Vec<KitCollider>,
}

/// A collider shape in a kit piece's frame.
///
/// The core's static level is a heightfield plus axis-aligned boxes, so every shape is
/// baked to a box: a capsule becomes its bounding box. Rotated boxes, real capsules,
/// and convex hulls need core support first (docs/game/01-architecture.md, "Physics").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum KitCollider {
    Box {
        center: [f32; 3],
        half_extents: [f32; 3],
    },
    /// An upright capsule: a segment `half_height` above and below `center`, swept by
    /// `radius`.
    Capsule {
        center: [f32; 3],
        half_height: f32,
        radius: f32,
    },
}

impl KitCollider {
    /// The shape's bounding box as (center, half extents).
    pub fn aabb(&self) -> ([f32; 3], [f32; 3]) {
        match *self {
            Self::Box {
                center,
                half_extents,
            } => (center, half_extents),
            Self::Capsule {
                center,
                half_height,
                radius,
            } => (center, [radius, half_height + radius, radius]),
        }
    }

    fn check(&self) -> Result<(), String> {
        let (_, e) = self.aabb();
        if e.iter().all(|v| v.is_finite() && *v > 0.0) {
            Ok(())
        } else {
            Err("collider sizes must be positive".to_owned())
        }
    }
}

/// Every kit piece by ID, across catalogs.
#[derive(Debug, Default)]
pub struct Kits<'a> {
    pieces: BTreeMap<&'a str, &'a KitPiece>,
}

impl<'a> Kits<'a> {
    /// Indexes catalogs as `(file, catalog)`. Fails on a duplicate piece ID or a bad
    /// collider.
    pub fn new(
        catalogs: impl IntoIterator<Item = (&'a str, &'a KitCatalog)>,
    ) -> Result<Self, String> {
        let mut pieces = BTreeMap::new();
        for (file, cat) in catalogs {
            for p in &cat.pieces {
                if pieces.insert(p.id.as_str(), p).is_some() {
                    return Err(format!("{file}: kit piece {:?} is defined twice", p.id));
                }
                for (i, c) in p.colliders.iter().enumerate() {
                    c.check()
                        .map_err(|e| format!("{file}: {}: colliders[{i}]: {e}", p.id))?;
                }
            }
        }
        Ok(Self { pieces })
    }

    pub fn get(&self, id: &str) -> Option<&'a KitPiece> {
        self.pieces.get(id).copied()
    }

    /// Pieces with a tag, in ID order.
    pub fn tagged(&self, tag: &str) -> Vec<&'a KitPiece> {
        self.pieces
            .values()
            .copied()
            .filter(|p| p.tags.iter().any(|t| t == tag))
            .collect()
    }
}

/// Rotates (x, z) by `rot` quarter turns from +Z toward +X.
fn quarter_turn(rot: u8, x: f32, z: f32) -> (f32, f32) {
    match rot % 4 {
        0 => (x, z),
        1 => (z, -x),
        2 => (-x, -z),
        _ => (-z, x),
    }
}

/// The world boxes for a piece placed at `pos`, turned `rot` quarter turns and scaled.
fn piece_boxes(
    piece: &KitPiece,
    name: &str,
    pos: [f32; 3],
    rot: u8,
    scale: [f32; 3],
) -> Vec<BoxCollider> {
    let many = piece.colliders.len() > 1;
    piece
        .colliders
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let (c, e) = c.aabb();
            let (cx, cz) = quarter_turn(rot, c[0] * scale[0], c[2] * scale[2]);
            let (ex, ez) = if rot % 2 == 1 {
                (e[2] * scale[2], e[0] * scale[0])
            } else {
                (e[0] * scale[0], e[2] * scale[2])
            };
            BoxCollider {
                name: if many {
                    format!("{name}#{i}")
                } else {
                    name.to_owned()
                },
                center: [pos[0] + cx, pos[1] + c[1] * scale[1], pos[2] + cz],
                half_extents: [ex, e[1] * scale[1], ez],
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Level

/// A parsed area directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Level {
    pub area: Area,
    pub terrain: Option<Terrain>,
    pub layout: Layout,
    pub entities: Entities,
    pub dressing: Dressing,
}

impl Level {
    /// Parses an area directory's files. `file` returns a file's text by name (one of
    /// [`LEVEL_FILES`]), or `None` if it is missing. Errors are `(file, message)`.
    pub fn parse(file: impl Fn(&str) -> Option<String>) -> Result<Self, (String, String)> {
        fn one<T: serde::de::DeserializeOwned>(
            file: &impl Fn(&str) -> Option<String>,
            name: &str,
        ) -> Result<Option<T>, (String, String)> {
            file(name)
                .map(|t| parse_ron(&t).map_err(|e| (name.to_owned(), e)))
                .transpose()
        }
        let missing = |name: &str| (name.to_owned(), "the file is missing".to_owned());
        Ok(Self {
            area: one(&file, "area.ron")?.ok_or_else(|| missing("area.ron"))?,
            terrain: one(&file, "terrain.ron")?,
            layout: one(&file, "layout.ron")?.unwrap_or_default(),
            entities: one(&file, "entities.ron")?.ok_or_else(|| missing("entities.ron"))?,
            dressing: one(&file, "dressing.ron")?.unwrap_or_default(),
        })
    }

    /// The terrain heightfield, if the area has terrain.
    pub fn heightfield(&self) -> Result<Option<Heightfield>, String> {
        self.terrain
            .as_ref()
            .map(|t| {
                t.heightfield(&self.area.bounds, self.area.seed)
                    .map_err(|e| format!("terrain.ron: {e}"))
            })
            .transpose()
    }

    /// Every kit piece from `layout.ron`, snapped and resolved, pieces first.
    pub fn placements(&self) -> Vec<Placement> {
        let pieces = self.layout.pieces.iter().map(|p| Placement {
            name: p
                .name
                .clone()
                .unwrap_or_else(|| format!("{}@{},{}", p.kit, p.cell[0], p.cell[1])),
            piece: p.kit.clone(),
            pos: [p.cell[0] as f32 * GRID_M, p.y, p.cell[1] as f32 * GRID_M],
            yaw: quarter_yaw(p.rot),
            scale: [1.0; 3],
            source: Source::Piece,
        });
        let landmarks = self.layout.landmarks.iter().map(|l| Placement {
            name: l.name.clone(),
            piece: l.kit.clone(),
            pos: l.pos,
            yaw: quarter_yaw(l.rot),
            scale: l.scale,
            source: Source::Landmark,
        });
        pieces.chain(landmarks).collect()
    }

    /// The colliders of the layout, in placement order.
    pub fn layout_boxes(&self, kits: &Kits<'_>) -> Result<Vec<BoxCollider>, String> {
        let rots = self
            .layout
            .pieces
            .iter()
            .map(|p| p.rot)
            .chain(self.layout.landmarks.iter().map(|l| l.rot));
        let mut boxes = Vec::new();
        for (p, rot) in self.placements().iter().zip(rots) {
            let piece = kits.get(&p.piece).ok_or_else(|| {
                format!("layout.ron: {}: unknown kit piece {:?}", p.name, p.piece)
            })?;
            boxes.extend(piece_boxes(piece, &p.name, p.pos, rot, p.scale));
        }
        Ok(boxes)
    }

    /// Checks references and ranges that parsing can't.
    pub fn check(&self, dir_name: &str, kits: &Kits<'_>) -> Result<(), String> {
        let a = &self.area;
        if a.id != dir_name {
            return Err(format!(
                "area.ron: id {:?} must match the directory name {dir_name:?}",
                a.id
            ));
        }
        if !(a.bounds.min[0] < a.bounds.max[0] && a.bounds.min[1] < a.bounds.max[1]) {
            return Err("area.ron: bounds min must be below max".to_owned());
        }
        for (i, r) in a.regions.iter().enumerate() {
            if r.points.len() < 3 {
                return Err(format!("area.ron: region {:?} needs three points", r.name));
            }
            if a.regions[..i].iter().any(|o| o.name == r.name) {
                return Err(format!("area.ron: region {:?} is listed twice", r.name));
            }
        }
        let mut names: Vec<String> = Vec::new();
        for p in &self.layout.pieces {
            if p.rot > 3 {
                return Err(format!("layout.ron: {}: rot must be 0 to 3", p.kit));
            }
        }
        for l in &self.layout.landmarks {
            if l.rot > 3 {
                return Err(format!("layout.ron: {}: rot must be 0 to 3", l.name));
            }
            if !l.scale.iter().all(|s| s.is_finite() && *s > 0.0) {
                return Err(format!("layout.ron: {}: scale must be positive", l.name));
            }
        }
        for p in self.placements() {
            if names.contains(&p.name) {
                return Err(format!("layout.ron: the name {:?} is used twice", p.name));
            }
            names.push(p.name);
        }
        // A lift must never strand the player: both stops have a call lever
        // (docs/game/02-world.md, "Traversal and world mechanics").
        for e in &self.entities.entities {
            let Entity::Lift(l) = e else { continue };
            for stop in 0..=1u8 {
                let called = self.entities.entities.iter().any(|o| {
                    matches!(o, Entity::Lever(v) if matches!(&v.target,
                        LeverTarget::Lift { id, stop: s } if id == &l.id && *s == stop))
                });
                if !called {
                    return Err(format!(
                        "entities.ron: lift {:?} has no lever that calls it to stop {stop}",
                        l.id
                    ));
                }
            }
        }
        for (i, p) in self.dressing.props.iter().enumerate() {
            if kits.get(&p.kit).is_none() {
                return Err(format!(
                    "dressing.ron: props[{i}]: unknown kit piece {:?}",
                    p.kit
                ));
            }
        }
        for (i, r) in self.dressing.rules.iter().enumerate() {
            let at = format!("dressing.ron: rules[{i}] ({})", r.tag);
            if kits.tagged(&r.tag).is_empty() {
                return Err(format!("{at}: no kit piece has the tag"));
            }
            if !(r.density.is_finite() && r.density >= 0.0) {
                return Err(format!("{at}: density must not be negative"));
            }
            if !(r.min_spacing.is_finite() && r.min_spacing >= 0.05) {
                return Err(format!("{at}: min_spacing must be at least 0.05 m"));
            }
            if !(r.scale.0 > 0.0 && r.scale.0 <= r.scale.1) {
                return Err(format!("{at}: scale must be a positive (min, max) range"));
            }
            if let Some(m) = &r.region_mask
                && !a.regions.iter().any(|g| &g.name == m)
            {
                return Err(format!("{at}: unknown region {m:?}"));
            }
        }
        Ok(())
    }

    /// Runs the scatter rules (only the collider rules if `colliders_only`). `hf` is the
    /// level's heightfield.
    pub fn scatter(
        &self,
        kits: &Kits<'_>,
        hf: Option<&Heightfield>,
        colliders_only: bool,
    ) -> Result<Vec<ScatterInstance>, String> {
        let layout_boxes = self.layout_boxes(kits)?;
        let paths: Vec<&[[f32; 2]]> = self
            .terrain
            .iter()
            .flat_map(|t| &t.ops)
            .filter_map(|op| match op {
                Sculpt::FlattenPath { points, .. } => Some(points.as_slice()),
                _ => None,
            })
            .collect();
        let lanterns: Vec<[f32; 3]> = self
            .entities
            .entities
            .iter()
            .filter_map(|e| match e {
                Entity::Lantern(l) => Some(l.pos),
                _ => None,
            })
            .collect();
        let spawns: Vec<[f32; 3]> = std::iter::once(self.entities.spawn.pos)
            .chain(self.entities.entities.iter().filter_map(|e| match e {
                Entity::Lantern(_) => None,
                other => other.anchor(),
            }))
            .collect();
        let mut out = Vec::new();
        for (i, rule) in self.dressing.rules.iter().enumerate() {
            if colliders_only && !rule.collider {
                continue;
            }
            let variants = kits.tagged(&rule.tag);
            let region = rule
                .region_mask
                .as_ref()
                .and_then(|m| self.area.regions.iter().find(|r| &r.name == m));
            let (bounds, mask_area) = match region {
                Some(r) => (r.bounds(), r.area()),
                None => (self.area.bounds, self.area.bounds.area()),
            };
            let wanted = (rule.density * mask_area).floor();
            if wanted > MAX_SCATTER_PER_RULE as f32 {
                return Err(format!(
                    "dressing.ron: rules[{i}] ({}) wants {wanted} instances; the limit is {MAX_SCATTER_PER_RULE}",
                    rule.tag
                ));
            }
            let wanted = wanted as usize;
            let max_grad2 = if rule.max_slope_deg >= 90.0 {
                f32::INFINITY
            } else {
                sq(libm::tanf(
                    rule.max_slope_deg.max(0.0) * (core::f32::consts::PI / 180.0),
                ))
            };
            let blocked = |x: f32, z: f32| {
                rule.avoid.iter().any(|a| match *a {
                    Avoid::Path { width } => paths
                        .iter()
                        .any(|pts| polyline_dist2(pts, x, z) < sq(width * 0.5)),
                    Avoid::Lantern { radius } => lanterns
                        .iter()
                        .any(|p| sq(p[0] - x) + sq(p[2] - z) < sq(radius)),
                    Avoid::Spawn { radius } => spawns
                        .iter()
                        .any(|p| sq(p[0] - x) + sq(p[2] - z) < sq(radius)),
                    Avoid::Layout { margin } => layout_boxes.iter().any(|b| {
                        (x - b.center[0]).abs() < b.half_extents[0] + margin
                            && (z - b.center[2]).abs() < b.half_extents[2] + margin
                    }),
                })
            };
            let mut rng = Rng(self.area.seed ^ (i as u64 + 1).wrapping_mul(0xD1B5_4A32_D192_ED03));
            let cell = rule.min_spacing;
            let mut grid: BTreeMap<(i32, i32), Vec<[f32; 2]>> = BTreeMap::new();
            let mut placed = 0;
            for _ in 0..wanted * DARTS_PER_INSTANCE {
                if placed == wanted {
                    break;
                }
                let x = bounds.min[0] + rng.unit() * (bounds.max[0] - bounds.min[0]);
                let z = bounds.min[1] + rng.unit() * (bounds.max[1] - bounds.min[1]);
                // Draw every number for the dart up front so a rejection doesn't shift
                // the stream for the darts after it differently per reason.
                let pick = rng.next();
                let s_t = rng.unit();
                let yaw_t = rng.unit();
                if !self.area.bounds.contains(x, z) || region.is_some_and(|r| !r.contains(x, z)) {
                    continue;
                }
                let (y, gx, gz) = match hf {
                    Some(h) => sample(h, x, z),
                    None => (0.0, 0.0, 0.0),
                };
                if gx * gx + gz * gz > max_grad2 || blocked(x, z) {
                    continue;
                }
                let key = ((x / cell).floor() as i32, (z / cell).floor() as i32);
                let crowded = (-1..=1).any(|dx| {
                    (-1..=1).any(|dz| {
                        grid.get(&(key.0 + dx, key.1 + dz)).is_some_and(|v| {
                            v.iter()
                                .any(|p| sq(p[0] - x) + sq(p[1] - z) < sq(rule.min_spacing))
                        })
                    })
                });
                if crowded {
                    continue;
                }
                grid.entry(key).or_default().push([x, z]);
                placed += 1;
                let yaw = if rule.collider {
                    quarter_yaw((yaw_t * 4.0) as u8 % 4)
                } else {
                    yaw_t * core::f32::consts::TAU
                };
                out.push(ScatterInstance {
                    rule: i,
                    piece: variants[(pick % variants.len() as u64) as usize].id.clone(),
                    pos: [x, y, z],
                    yaw,
                    scale: rule.scale.0 + (rule.scale.1 - rule.scale.0) * s_t,
                });
            }
        }
        Ok(out)
    }

    /// Builds the scenario the core runs.
    pub fn compile(&self, kits: &Kits<'_>) -> Result<Scenario, String> {
        let heightfield = self.heightfield()?;
        let mut boxes = self.layout_boxes(kits)?;
        for (n, s) in self
            .scatter(kits, heightfield.as_ref(), true)?
            .iter()
            .enumerate()
        {
            let rule = &self.dressing.rules[s.rule];
            let piece = kits
                .get(&s.piece)
                .ok_or_else(|| format!("dressing.ron: unknown kit piece {:?}", s.piece))?;
            let rot = yaw_quarter(s.yaw);
            boxes.extend(piece_boxes(
                piece,
                &format!("scatter:{}#{n}", rule.tag),
                s.pos,
                rot,
                [s.scale; 3],
            ));
        }
        let mut sc = Scenario {
            spawn: self.entities.spawn.pos,
            spawn_yaw: self.entities.spawn.yaw,
            kill_y: self.area.kill_y,
            heightfield,
            boxes,
            dummies: Vec::new(),
            enemies: Vec::new(),
            bosses: Vec::new(),
            lanterns: Vec::new(),
            water: Vec::new(),
            reeds: Vec::new(),
            pickups: Vec::new(),
            ladders: Vec::new(),
            lifts: Vec::new(),
            doors: Vec::new(),
            levers: Vec::new(),
            hollow_walls: Vec::new(),
            chests: Vec::new(),
            messages: Vec::new(),
            pools: Vec::new(),
            snow: Vec::new(),
            start_glimmer: self.area.start_glimmer,
            player_weapon: self.area.player_weapon.clone(),
            player_spells: self.area.player_spells.clone(),
            echo_stones: Vec::new(),
            echoes_unlocked: Vec::new(),
            npcs: Vec::new(),
            stations: Vec::new(),
            safe_zone: self.area.safe_zone,
        };
        for e in &self.entities.entities {
            match e {
                Entity::Lantern(l) => sc.lanterns.push(l.clone()),
                Entity::Dummy(d) => sc.dummies.push(d.clone()),
                Entity::Enemy(s) => sc.enemies.push(s.clone()),
                Entity::Boss(b) => sc.bosses.push(b.clone()),
                Entity::Water(z) => sc.water.push(z.clone()),
                Entity::Reeds(z) => sc.reeds.push(z.clone()),
                Entity::Pickup(p) => sc.pickups.push(p.clone()),
                Entity::Ladder(l) => sc.ladders.push(l.clone()),
                Entity::Lift(l) => sc.lifts.push(l.clone()),
                Entity::Door(d) => sc.doors.push(d.clone()),
                Entity::Lever(l) => sc.levers.push(l.clone()),
                Entity::HollowWall(w) => sc.hollow_walls.push(w.clone()),
                Entity::Chest(c) => sc.chests.push(c.clone()),
                Entity::Message(m) => sc.messages.push(m.clone()),
                Entity::Pool(p) => sc.pools.push(p.clone()),
                Entity::Snow(z) => sc.snow.push(z.clone()),
                Entity::Npc(n) => sc.npcs.push(n.clone()),
                Entity::Station(s) => sc.stations.push(s.clone()),
            }
        }
        sc.check().map_err(|e| format!("entities.ron: {e}"))?;
        Ok(sc)
    }
}

fn quarter_yaw(rot: u8) -> f32 {
    f32::from(rot % 4) * core::f32::consts::FRAC_PI_2
}

fn yaw_quarter(yaw: f32) -> u8 {
    ((yaw / core::f32::consts::FRAC_PI_2) + 0.5).floor() as i32 as u8 % 4
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_level() -> Level {
        Level {
            area: Area {
                id: "t".into(),
                name: "T".into(),
                seed: 7,
                bounds: Bounds {
                    min: [-10.0, -10.0],
                    max: [10.0, 10.0],
                },
                kill_y: -20.0,
                ambient: Ambient::default(),
                music: None,
                regions: vec![Region {
                    name: "east".into(),
                    points: vec![[0.0, -10.0], [10.0, -10.0], [10.0, 10.0], [0.0, 10.0]],
                }],
                connections: vec![],
                start_glimmer: 0,
                player_weapon: None,
                player_spells: Vec::new(),
                safe_zone: false,
            },
            terrain: Some(Terrain {
                spacing_m: 1.0,
                base_y: 0.0,
                noise: None,
                ops: vec![],
            }),
            layout: Layout::default(),
            entities: Entities {
                spawn: Spawn {
                    pos: [0.0; 3],
                    yaw: 0.0,
                },
                entities: vec![],
            },
            dressing: Dressing::default(),
        }
    }

    fn catalog() -> KitCatalog {
        parse_ron(
            r#"(pieces: [
                (id: "cube", colliders: [Box(center: (0.0, 0.5, 0.0), half_extents: (0.5, 0.5, 0.5))]),
                (id: "wall", colliders: [Box(center: (1.0, 1.0, 0.0), half_extents: (1.0, 1.0, 0.25))]),
                (id: "tree", tags: ["tree"], colliders: [Capsule(center: (0.0, 2.0, 0.0), half_height: 1.5, radius: 0.3)]),
                (id: "grass", tags: ["grass"]),
            ])"#,
        )
        .unwrap()
    }

    #[test]
    fn sculpt_ops_shape_the_heightfield() {
        let mut level = flat_level();
        level.terrain.as_mut().unwrap().ops = vec![
            Sculpt::Ramp {
                min: [-2.0, 0.0],
                max: [2.0, 4.0],
                axis: Axis::Z,
                from: 0.0,
                to: 2.0,
            },
            Sculpt::SetRect {
                min: [5.0, 5.0],
                max: [6.0, 6.0],
                height: 3.0,
            },
            Sculpt::RaiseCircle {
                center: [-6.0, -6.0],
                radius: 2.0,
                height: 1.0,
            },
            Sculpt::CarveRiver {
                points: vec![[-10.0, -2.0], [10.0, -2.0]],
                width: 2.0,
                depth: 1.0,
            },
        ];
        let h = level.heightfield().unwrap().unwrap();
        assert_eq!((h.rows, h.cols), (21, 21));
        let at = |x: i32, z: i32| h.heights[((z + 10) * 21 + (x + 10)) as usize];
        assert_eq!(at(0, 0), 0.0);
        assert_eq!(at(0, 2), 1.0);
        assert_eq!(at(0, 4), 2.0);
        assert_eq!(at(3, 4), 0.0);
        assert_eq!(at(5, 5), 3.0);
        assert_eq!(at(-6, -6), 1.0);
        assert!(at(-5, -6) > 0.0 && at(-5, -6) < 1.0);
        assert_eq!(at(-8, -6), 0.0);
        assert_eq!(at(3, -2), -1.0);
        assert_eq!(at(3, -3), 0.0);
        assert_eq!(sample_height(&h, 0.0, 1.0), 0.5);
    }

    #[test]
    fn cliff_line_raises_the_left_side() {
        let mut level = flat_level();
        level.terrain.as_mut().unwrap().ops = vec![Sculpt::CliffLine {
            from: [0.0, -5.0],
            to: [0.0, 5.0],
            width: 3.0,
            height: 4.0,
        }];
        let h = level.heightfield().unwrap().unwrap();
        let at = |x: i32, z: i32| h.heights[((z + 10) * 21 + (x + 10)) as usize];
        // Walking toward +Z, the left is +X.
        assert_eq!(at(2, 0), 4.0);
        assert_eq!(at(-2, 0), 0.0);
        assert_eq!(at(4, 0), 0.0);
        assert_eq!(at(2, 7), 0.0);
    }

    #[test]
    fn noise_is_seeded_and_bounded() {
        let mut level = flat_level();
        level.terrain.as_mut().unwrap().noise = Some(Noise {
            amplitude_m: 2.0,
            wavelength_m: 8.0,
            octaves: 3,
        });
        let a = level.heightfield().unwrap().unwrap();
        assert_eq!(a, level.heightfield().unwrap().unwrap());
        assert!(a.heights.iter().all(|h| h.abs() <= 3.5));
        assert!(a.heights.iter().any(|h| *h != 0.0));
        level.area.seed = 8;
        assert_ne!(a, level.heightfield().unwrap().unwrap());
    }

    #[test]
    fn layout_pieces_snap_turn_and_scale() {
        let cat = catalog();
        let kits = Kits::new([("kits/t.ron", &cat)]).unwrap();
        let mut level = flat_level();
        level.layout.pieces = vec![Piece {
            kit: "wall".into(),
            cell: [1, 2],
            y: 0.5,
            rot: 1,
            name: None,
        }];
        level.layout.landmarks = vec![Landmark {
            name: "big".into(),
            kit: "cube".into(),
            pos: [0.0, 0.0, -4.0],
            rot: 0,
            scale: [4.0, 2.0, 6.0],
        }];
        level.check("t", &kits).unwrap();
        let boxes = level.layout_boxes(&kits).unwrap();
        assert_eq!(boxes[0].name, "wall@1,2");
        // Origin (2, 0.5, 4); the wall's center (1, 1, 0) turned a quarter is (0, 1, -1).
        assert_eq!(boxes[0].center, [2.0, 1.5, 3.0]);
        assert_eq!(boxes[0].half_extents, [0.25, 1.0, 1.0]);
        assert_eq!(boxes[1].center, [0.0, 1.0, -4.0]);
        assert_eq!(boxes[1].half_extents, [2.0, 1.0, 3.0]);
    }

    #[test]
    fn scatter_is_deterministic_spaced_and_masked() {
        let cat = catalog();
        let kits = Kits::new([("kits/t.ron", &cat)]).unwrap();
        let mut level = flat_level();
        level.dressing.rules = vec![ScatterRule {
            tag: "grass".into(),
            density: 0.5,
            min_spacing: 1.0,
            max_slope_deg: 30.0,
            avoid: vec![Avoid::Spawn { radius: 3.0 }],
            region_mask: Some("east".into()),
            scale: (0.8, 1.2),
            collider: false,
        }];
        level.check("t", &kits).unwrap();
        let hf = level.heightfield().unwrap();
        let a = level.scatter(&kits, hf.as_ref(), false).unwrap();
        assert_eq!(a, level.scatter(&kits, hf.as_ref(), false).unwrap());
        assert!(a.len() > 30, "only {} instances", a.len());
        for (i, p) in a.iter().enumerate() {
            assert!(p.pos[0] >= 0.0, "outside the mask: {p:?}");
            assert!(sq(p.pos[0]) + sq(p.pos[2]) >= 9.0, "too close to the spawn");
            assert!((0.8..=1.2).contains(&p.scale));
            for q in &a[..i] {
                assert!(sq(p.pos[0] - q.pos[0]) + sq(p.pos[2] - q.pos[2]) >= 1.0);
            }
        }
        // Non-collider rules never reach the core.
        assert!(level.compile(&kits).unwrap().boxes.is_empty());
    }

    #[test]
    fn collider_scatter_adds_boxes_to_the_scenario() {
        let cat = catalog();
        let kits = Kits::new([("kits/t.ron", &cat)]).unwrap();
        let mut level = flat_level();
        level.dressing.rules = vec![ScatterRule {
            tag: "tree".into(),
            density: 0.02,
            min_spacing: 3.0,
            max_slope_deg: 90.0,
            avoid: vec![],
            region_mask: None,
            scale: (1.0, 1.0),
            collider: true,
        }];
        let sc = level.compile(&kits).unwrap();
        assert_eq!(sc.boxes.len(), 8);
        assert!(sc.boxes[0].name.starts_with("scatter:tree#"));
        assert_eq!(sc.boxes[0].half_extents, [0.3, 1.8, 0.3]);
    }

    #[test]
    fn entities_fill_the_scenario_lists() {
        let level_entities: Entities = parse_ron(
            r#"(
                spawn: (pos: (1.0, 0.0, 2.0), yaw: 0.5),
                entities: [
                    Lantern(id: "l1", pos: (0.0, 0.0, 3.0), yaw: 0.0),
                    Water(name: "pond", center: (4.0, 4.0), half_extents: (2.0, 2.0)),
                    Enemy(name: "e1", enemy: "husk", pos: (5.0, 0.0, 5.0), yaw: 0.0),
                ],
            )"#,
        )
        .unwrap();
        let mut level = flat_level();
        level.entities = level_entities;
        let cat = catalog();
        let kits = Kits::new([("kits/t.ron", &cat)]).unwrap();
        let sc = level.compile(&kits).unwrap();
        assert_eq!(sc.spawn, [1.0, 0.0, 2.0]);
        assert_eq!(sc.lanterns[0].id, "l1");
        assert_eq!(sc.water[0].name, "pond");
        assert_eq!(sc.enemies[0].enemy, "husk");
    }

    #[test]
    fn the_spec_dressing_sample_parses() {
        // docs/game/02-world.md, "Procedural dressing".
        let d: Dressing = parse_ron(
            r#"(
              rules: [
                (
                  tag: "ash_grass",           // kit catalog tag; picks a random variant
                  density: 0.6,               // per square meter
                  min_spacing: 0.8,           // Poisson-disk radius in meters
                  max_slope_deg: 30,
                  avoid: [Path(width: 2.0), Lantern(radius: 4.0), Spawn(radius: 2.0)],
                  region_mask: "meadow",      // named polygon in area.ron
                  scale: (0.8, 1.3),
                  collider: false,
                ),
              ],
            )"#,
        )
        .unwrap();
        assert_eq!(d.rules[0].max_slope_deg, 30.0);
        assert_eq!(d.rules[0].region_mask.as_deref(), Some("meadow"));
        assert_eq!(d.rules[0].avoid[0], Avoid::Path { width: 2.0 });
    }

    #[test]
    fn bad_references_fail() {
        let cat = catalog();
        let kits = Kits::new([("kits/t.ron", &cat)]).unwrap();
        let mut level = flat_level();
        level.layout.landmarks = vec![Landmark {
            name: "x".into(),
            kit: "nope".into(),
            pos: [0.0; 3],
            rot: 0,
            scale: [1.0; 3],
        }];
        assert!(
            level
                .compile(&kits)
                .unwrap_err()
                .contains("unknown kit piece")
        );
        let mut level = flat_level();
        level.terrain.as_mut().unwrap().spacing_m = 3.0;
        assert!(level.heightfield().unwrap_err().contains("whole multiple"));
        assert!(flat_level().check("other", &kits).is_err());
        assert!(Kits::new([("a", &cat), ("b", &cat)]).is_err());
    }
}
