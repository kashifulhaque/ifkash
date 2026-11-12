//! Water levels tied to a world flag (WP-060; docs/game/02-world.md, Mirrormere's "Loop").
//!
//! A [`Pool`](lf_content::scenario::Pool) has a normal depth, an optional depth a lever
//! raises it to (`raised_depth_m`, flag `pool.<name>.raised`), and an optional depth a world
//! flag lowers it to (`lowered_by` and `lowered_depth_m`). The flag is not tied to a lever
//! in the same level: Mirrormere's lake is lowered by the sluice lever inside the Drowned
//! Belfry (WP-061), which is a different area, so the Belfry only has to raise the flag
//! [`sluice_flag`]`("mirrormere")` (`sluice.mirrormere.open`). World flags outlive the
//! area, so the lake is lower when the player comes back.
//!
//! Lowering the lake by 1.5 m takes the deep water over the Cathedral island's ford
//! (1.6 m deep) down to 0.1 m, which wades, so the ford becomes a walkable path; the open
//! lake (4 m, lowered to 2.5 m) stays a wall. The depth is what the rest of the core reads:
//! [`super::before_locomotion`] (shallow slows, deep blocks) and `render_state` (the
//! renderer eases the water surface to the new level).

use super::super::flags;
use crate::world::World;

/// The flag that lowers a region's water: `sluice.<region>.open`. Raise it with
/// [`open_sluice`]; level data names it in `lowered_by`.
pub fn sluice_flag(region: &str) -> String {
    format!("sluice.{region}.open")
}

/// Opens a region's sluice: every pool that names its flag in `lowered_by` drops to its
/// `lowered_depth_m` from the next tick. This is what the Drowned Belfry's lever does.
pub fn open_sluice(world: &mut World, region: &str) {
    world.state.flags.raise(&sluice_flag(region));
}

/// Whether a region's sluice is open.
pub fn sluice_open(world: &World, region: &str) -> bool {
    world.state.flags.is_set(&sluice_flag(region))
}

/// The depth of pool `i` right now, in meters: the lever-raised depth if its lever is
/// pulled, else the lowered depth if its sluice flag is set, else the normal depth.
pub fn depth(world: &World, i: usize) -> f32 {
    let p = &world.layout.pools[i];
    if let Some(r) = p.raised_depth_m
        && world.state.flags.is_set(&flags::pool_raised(&p.name))
    {
        return r;
    }
    if let (Some(flag), Some(d)) = (&p.lowered_by, p.lowered_depth_m)
        && world.state.flags.is_set(flag)
    {
        return d;
    }
    p.depth_m
}
