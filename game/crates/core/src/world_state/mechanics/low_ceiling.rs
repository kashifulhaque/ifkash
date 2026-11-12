//! Low ceilings (WP-063; docs/game/02-world.md, "Mini-dungeons", D1 Mossgut Cave: "Low
//! ceilings block overhead attacks; fight with thrusts and horizontal swings").
//!
//! A [`LowCeiling`](lf_content::scenario::LowCeiling) zone is a rectangle on the ground plane
//! with a height. While an actor stands in it, any active hitbox of theirs that reaches
//! higher than `height_m` above their feet strikes the rock instead: the hit resolver
//! (`combat::resolve`) skips it, so it can't land on anyone, and the player's swing raises
//! `CEILING_STRUCK`. "Reaches higher" is geometry, not a move tag: the top of the hitbox's
//! capsule (its higher end plus its radius). With the cave's 2.0 m ceiling that stops the
//! overhead chops (a sword's heavy reaches 2.2 m, an axe's light 2.18 m, a plunge 2.4 m),
//! the rising Arts, and the jump heavies, and leaves every thrust, every horizontal swing,
//! and nearly every roll and backstep attack (under 1.6 m; the axe's reach 2.08 m) alone.
//! It applies to enemies and Echoes the same as to the player.
//!
//! The roof slabs in the cave's `layout.ron` sit at the same height, so a jump bumps its head
//! on them and the camera can't rise through them: the rule and the rock agree.

use crate::combat::hitbox::Capsule;
use crate::world::World;

use super::mechanic;

/// A swing of the player's struck the ceiling (WASM event kind 14, Mechanic; index is the
/// zone's). WP-063 reserves the kinds 70-79.
pub const CEILING_STRUCK: u8 = 72;

/// The zone an actor standing at `pos` is under, if any: the first listed whose rectangle holds
/// (x, z).
pub fn zone_at(world: &World, pos: [f32; 3]) -> Option<usize> {
    world.layout.low_ceilings.iter().position(|z| {
        (pos[0] - z.center[0]).abs() <= z.half_extents[0]
            && (pos[2] - z.center[1]).abs() <= z.half_extents[1]
    })
}

/// The zone whose ceiling this hitbox strikes, if it does: the attacker stands under a low
/// ceiling and the capsule's top is higher than it above the attacker's feet.
pub fn struck_by(world: &World, feet: [f32; 3], hitbox: &Capsule) -> Option<usize> {
    if world.layout.low_ceilings.is_empty() {
        return None;
    }
    let zone = zone_at(world, feet)?;
    let top = hitbox.a[1].max(hitbox.b[1]) + hitbox.radius - feet[1];
    (top > world.layout.low_ceilings[zone].height_m).then_some(zone)
}

/// Tells the host that a swing struck the ceiling of zone `zone`.
pub fn note(world: &mut World, zone: usize) {
    mechanic(world, CEILING_STRUCK, zone);
}
