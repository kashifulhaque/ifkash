//! Light beams, mirrors, and receptors (WP-063; docs/game/02-world.md, "Mini-dungeons", D5
//! Glasswing Grotto: "Crystal mirrors reflect light beams; aim them to open sealed doors").
//!
//! A [`BeamSource`] shines along one of the four compass directions, level at its own height.
//! The beam runs until it meets, in order of distance, a mirror, a receptor, or a wall (a ray
//! against the level's colliders, which includes shut doors). Every beam direction is a compass
//! direction, and every mirror stands at 45 degrees, so a reflection is whole-number
//! arithmetic: no trigonometry, nothing to drift.
//!
//! - A **mirror** has four facings, the directions its face points: 0 north-east (+X +Z),
//!   1 south-east (+X -Z), 2 south-west, 3 north-west. `INTERACT` beside it turns it a quarter
//!   turn (the facing flag `mirror.<id>.facing`, value facing plus one, so a mirror nobody has
//!   touched needs no flag). A beam that strikes the face bounces to the compass direction
//!   `d - k n`, where `n` is the face's diagonal and `k = d . n`; a beam that strikes the back
//!   stops there.
//! - A **receptor** lit by a beam raises `receptor.<id>.lit` for good and opens the door it
//!   names. The doors stay open: a lit socket never goes dark, so a puzzle can't be undone
//!   by walking away, and the player can't be shut out of the next room by a stray turn.
//! - The first time a mirror is turned it raises `mirror.<id>.turned`, which an enemy's
//!   `wakes_on` can read (the Grotto's twist room).
//!
//! The beam is recomputed every tick from the mirrors' facings and the doors' flags, so it is
//! a pure function of the world flags and needs no state of its own. [`render_words`] sends
//! the renderer what to draw.

use lf_content::scenario::{BeamSource, Mirror, Receptor};

use super::{flags, mechanic, set_door};
use crate::math::dmath::sqrt;
use crate::world::World;

/// A mirror was turned (WASM event kind 14, Mechanic; index is the mirror's). WP-063 reserves
/// the kinds 70-79.
pub const MIRROR_TURNED: u8 = 70;
/// A receptor was lit for the first time (index is the receptor's).
pub const RECEPTOR_LIT: u8 = 71;

/// The prompt code (`interact_prompt`): turn a mirror. WP-063 reserves 40-44.
pub const PROMPT_TURN: u32 = 40;

/// The most legs one beam is drawn with: the source's own leg and seven bounces.
pub const MAX_SEGMENTS: usize = 8;

/// How far a beam goes with nothing in its way, in meters.
pub const MAX_RANGE_M: f32 = 64.0;

/// How far off the beam's line a mirror or a receptor may stand and still be struck, in meters.
/// Levels place them on the line, so this only absorbs rounding.
const STRIKE_HALF_WIDTH_M: f32 = 0.6;

/// The beam starts this far from its source's center, so a ray doesn't start inside the
/// crystal's own socket, in meters.
const START_CLEAR_M: f32 = 0.4;

/// Flag name for a mirror's facing (value is the facing plus one).
pub fn facing_flag(id: &str) -> String {
    format!("mirror.{id}.facing")
}

/// Flag name for "the player has turned this mirror".
pub fn turned_flag(id: &str) -> String {
    format!("mirror.{id}.turned")
}

/// Flag name for "a beam has lit this receptor".
pub fn lit_flag(id: &str) -> String {
    format!("receptor.{id}.lit")
}

/// A compass direction as a step in (x, z): 0 is +Z, 1 is +X, 2 is -Z, 3 is -X.
pub fn step_of(dir: u8) -> (i32, i32) {
    match dir % 4 {
        0 => (0, 1),
        1 => (1, 0),
        2 => (0, -1),
        _ => (-1, 0),
    }
}

/// The compass direction of an axis-aligned step.
fn dir_of(step: (i32, i32)) -> u8 {
    match step {
        (0, 1) => 0,
        (1, 0) => 1,
        (0, -1) => 2,
        _ => 3,
    }
}

/// The diagonal a mirror's face points along, for a facing.
pub fn normal_of(facing: u8) -> (i32, i32) {
    match facing % 4 {
        0 => (1, 1),
        1 => (1, -1),
        2 => (-1, -1),
        _ => (-1, 1),
    }
}

/// What a beam traveling `dir` does at a mirror facing `facing`: `Some(new direction)` if it
/// strikes the face, `None` if it strikes the back.
pub fn reflect(dir: u8, facing: u8) -> Option<u8> {
    let (dx, dz) = step_of(dir);
    let (nx, nz) = normal_of(facing);
    let k = dx * nx + dz * nz;
    if k < 0 {
        Some(dir_of((dx - k * nx, dz - k * nz)))
    } else {
        None
    }
}

/// Mirror `i`'s facing now.
pub fn facing(world: &World, i: usize) -> u8 {
    let m = &world.layout.mirrors[i];
    match world.state.flags.get(&facing_flag(&m.id)) {
        0 => m.facing % 4,
        v => ((v - 1) % 4) as u8,
    }
}

/// Whether receptor `i` has been lit.
pub fn is_lit(world: &World, i: usize) -> bool {
    world
        .state
        .flags
        .is_set(&lit_flag(&world.layout.receptors[i].id))
}

/// What a beam ended on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    /// Ran out of range.
    Range,
    /// Hit a wall or a shut door.
    Wall,
    /// Struck the back of a mirror.
    MirrorBack(usize),
    /// Reached a receptor.
    Receptor(usize),
}

/// One beam, as drawn: its legs as `[x0, z0, x1, z1]`, level at `y`, and what it ended on.
#[derive(Clone, Debug, PartialEq)]
pub struct Beam {
    pub y: f32,
    pub legs: Vec<[f32; 4]>,
    pub end: End,
}

#[derive(Clone, Copy)]
enum Hit {
    Mirror(usize),
    Receptor(usize),
    Wall,
}

/// How far along the beam (from `from`, going `step`) a point is, if the beam strikes it.
fn strike(from: [f32; 2], step: (i32, i32), p: [f32; 3], min_t: f32) -> Option<f32> {
    let (sx, sz) = (step.0 as f32, step.1 as f32);
    let (dx, dz) = (p[0] - from[0], p[2] - from[1]);
    let along = dx * sx + dz * sz;
    let across = (dx * sz - dz * sx).abs();
    (along > min_t && across <= STRIKE_HALF_WIDTH_M).then_some(along)
}

/// Follows source `s`'s beam through the mirrors as they stand now.
pub fn trace(world: &World, s: usize) -> Beam {
    let src: &BeamSource = &world.layout.beam_sources[s];
    let y = src.pos[1];
    let mut at = [src.pos[0], src.pos[2]];
    let mut dir = src.dir % 4;
    let mut skip: Option<usize> = None;
    let mut legs = Vec::new();
    // The first leg starts clear of the crystal; later legs start at a mirror's center.
    let mut clear = START_CLEAR_M;
    let end = loop {
        let step = step_of(dir);
        let mut best: Option<(f32, Hit)> = None;
        let mut offer = |t: f32, hit: Hit| {
            if best.is_none_or(|(bt, _)| t < bt) {
                best = Some((t, hit));
            }
        };
        for (i, m) in world.layout.mirrors.iter().enumerate() {
            if skip != Some(i)
                && let Some(t) = strike(at, step, m.pos, 0.05)
            {
                offer(t, Hit::Mirror(i));
            }
        }
        for (i, r) in world.layout.receptors.iter().enumerate() {
            if let Some(t) = strike(at, step, r.pos, 0.05) {
                offer(t, Hit::Receptor(i));
            }
        }
        let origin = [
            at[0] + step.0 as f32 * clear,
            y,
            at[1] + step.1 as f32 * clear,
        ];
        let ray = [step.0 as f32, 0.0, step.1 as f32];
        if let Some(t) = world.physics.raycast(origin, ray, MAX_RANGE_M) {
            offer(t + clear, Hit::Wall);
        }
        let (t, hit) = best.unwrap_or((MAX_RANGE_M, Hit::Wall));
        let to = match hit {
            Hit::Mirror(i) => [
                world.layout.mirrors[i].pos[0],
                world.layout.mirrors[i].pos[2],
            ],
            Hit::Receptor(i) => [
                world.layout.receptors[i].pos[0],
                world.layout.receptors[i].pos[2],
            ],
            Hit::Wall => [at[0] + step.0 as f32 * t, at[1] + step.1 as f32 * t],
        };
        legs.push([at[0], at[1], to[0], to[1]]);
        match hit {
            Hit::Mirror(i) => match reflect(dir, facing(world, i)) {
                Some(next) if legs.len() < MAX_SEGMENTS => {
                    at = to;
                    dir = next;
                    skip = Some(i);
                    clear = 0.0;
                }
                Some(_) => break End::Range,
                None => break End::MirrorBack(i),
            },
            Hit::Receptor(i) => break End::Receptor(i),
            Hit::Wall => {
                break if t >= MAX_RANGE_M {
                    End::Range
                } else {
                    End::Wall
                };
            }
        }
    };
    Beam { y, legs, end }
}

/// Every beam, in source order.
pub fn beams(world: &World) -> Vec<Beam> {
    (0..world.layout.beam_sources.len())
        .map(|s| trace(world, s))
        .collect()
}

/// Whether the area has anything this module does.
pub fn present(world: &World) -> bool {
    !(world.layout.beam_sources.is_empty()
        && world.layout.mirrors.is_empty()
        && world.layout.receptors.is_empty())
}

/// Tick system 1, after the player's `INTERACT`: lights the receptors the beams reach, and
/// opens their doors.
pub fn step(world: &mut World) {
    if world.layout.beam_sources.is_empty() {
        return;
    }
    let mut lit: Vec<usize> = Vec::new();
    for b in beams(world) {
        if let End::Receptor(i) = b.end
            && !lit.contains(&i)
        {
            lit.push(i);
        }
    }
    lit.sort_unstable();
    for i in lit {
        if is_lit(world, i) {
            continue;
        }
        let r: Receptor = world.layout.receptors[i].clone();
        world.state.flags.raise(&lit_flag(&r.id));
        mechanic(world, RECEPTOR_LIT, i);
        if let Some(id) = &r.opens
            && let Some(d) = world.layout.doors.iter().position(|d| &d.id == id)
            && !world.state.flags.is_set(&flags::door_open(id))
        {
            set_door(world, d, true);
        }
    }
}

/// The player pressed `INTERACT` beside mirror `i`: turn it a quarter turn.
pub fn turn(world: &mut World, i: usize) {
    let m: Mirror = world.layout.mirrors[i].clone();
    let next = (facing(world, i) + 1) % 4;
    world
        .state
        .flags
        .set(&facing_flag(&m.id), u32::from(next) + 1);
    world.state.flags.raise(&turned_flag(&m.id));
    mechanic(world, MIRROR_TURNED, i);
}

/// The renderer's words, after the visibility word: each mirror's facing (0 to 3), each receptor
/// (1 once lit), then, per source, the number of legs and [`MAX_SEGMENTS`] legs of
/// `x0, z0, x1, z1` (unused legs are zero). The beam's height is the source's `pos.y`.
pub fn render_words(world: &World) -> Vec<f32> {
    let l = &world.layout;
    let mut out: Vec<f32> = (0..l.mirrors.len())
        .map(|i| f32::from(facing(world, i)))
        .collect();
    out.extend((0..l.receptors.len()).map(|i| f32::from(u8::from(is_lit(world, i)))));
    for b in beams(world) {
        out.push(b.legs.len() as f32);
        for k in 0..MAX_SEGMENTS {
            match b.legs.get(k) {
                Some(leg) => out.extend_from_slice(leg),
                None => out.extend_from_slice(&[0.0; 4]),
            }
        }
    }
    out
}

/// The length of a beam's path, in meters (a test helper).
pub fn length(beam: &Beam) -> f32 {
    beam.legs
        .iter()
        .map(|l| {
            let (dx, dz) = (l[2] - l[0], l[3] - l[1]);
            sqrt(dx * dx + dz * dz)
        })
        .sum()
}
