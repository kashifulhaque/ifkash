//! Darkness and glowing zones (WP-062; docs/game/02-world.md, "Hollowdeep" and "Traversal and
//! world mechanics").
//!
//! A dark area (Hollowdeep) has a [`Darkness`] rule and a list of glowing zones. Outside the
//! zones, and farther than the rule's `lantern_light_m` from every Lantern, the player sees
//! only `visibility_m` (6 m in the spec). Two things push it back: a carried light, which is
//! the world flag the rule names (`torch_flag`, the codex flag the *smoltorch* relic pickup
//! raises), and a Lantern Flare ([`flare`]), which lights the dark for a number of ticks.
//!
//! "Visibility" is what the host draws: [`render_word`] is the number of meters the player
//! sees now (0 for no limit), and the renderer fogs the world to it. Darkness never
//! blocks movement, and the critical path stays inside glowing zones: the bake checks it
//! (`lf-tools bake`, the `dark-critical-path` warning), and this module's [`lit_at`] is the
//! rule the check uses. Enemies don't read it yet (the *endark* cloak, which halves the range
//! enemies notice a player in darkness, is the armor WP's).

use crate::world::World;

/// Whether the ground at (x, z) is lit: inside a glowing zone or within a Lantern's light.
/// An area with no darkness rule is lit everywhere.
pub fn lit_at(world: &World, x: f32, z: f32) -> bool {
    let Some(rule) = &world.layout.darkness else {
        return true;
    };
    lit_by(
        &world.layout.glows,
        &world.layout.lanterns,
        rule.lantern_light_m,
        x,
        z,
    )
}

/// The glowing-zone and Lantern test on bare lists, so the bake can use it without a world.
pub fn lit_by(
    glows: &[lf_content::scenario::Zone],
    lanterns: &[lf_content::scenario::Lantern],
    lantern_light_m: f32,
    x: f32,
    z: f32,
) -> bool {
    glows.iter().any(|g| {
        (x - g.center[0]).abs() <= g.half_extents[0] && (z - g.center[1]).abs() <= g.half_extents[1]
    }) || lanterns.iter().any(|l| {
        let (dx, dz) = (x - l.pos[0], z - l.pos[2]);
        dx * dx + dz * dz <= lantern_light_m * lantern_light_m
    })
}

/// Whether the player carries a light now: the relic's flag is raised, or a Lantern Flare is
/// still burning.
pub fn carrying_light(world: &World) -> bool {
    let Some(rule) = &world.layout.darkness else {
        return false;
    };
    world.state.flags.is_set(&rule.torch_flag) || world.tick < world.mech.flare_until
}

/// How far the player sees right now, in meters. `None` means no limit: the area isn't dark, or
/// the player stands in light.
pub fn visibility_m(world: &World) -> Option<f32> {
    let rule = world.layout.darkness.as_ref()?;
    let p = world.actors.get(world.player)?;
    if lit_at(world, p.pos[0], p.pos[2]) {
        return None;
    }
    Some(if carrying_light(world) {
        rule.torch_visibility_m
    } else {
        rule.visibility_m
    })
}

/// Lights the dark for `ticks` ticks (Lantern Flare: 600, `art_lantern_flare`). Relighting
/// while it burns keeps the later end. The Art's effect calls this.
pub fn flare(world: &mut World, ticks: u32) {
    let until = world.tick + u64::from(ticks);
    if until > world.mech.flare_until {
        world.mech.flare_until = until;
    }
}

/// The renderer's word: the player's visibility in meters, or 0 for no limit. Only areas with a
/// darkness rule send it.
pub fn render_word(world: &World) -> f32 {
    visibility_m(world).unwrap_or(0.0)
}
