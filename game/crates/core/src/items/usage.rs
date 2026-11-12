//! Using the consumables that aren't flasks (WP-057): throwables, oils, cures, Glimmer
//! vials, and Lantern Ash, plus the arrows a bow fires. See
//! docs/game/04-arsenal-and-builds.md, "Consumables".
//!
//! `USE_ITEM` (or `Command::UseItem`) with one of these selected starts the same drink
//! move a flask does. On the move's effect frame `combat::moves::step` marks the item as
//! landed ([`FlaskKind::Landed`](super::FlaskKind::Landed)); [`apply_landed`] applies the
//! effect at the start of the next tick, which is when one is spent from the stack. A
//! stagger before the effect frame uses nothing up.

use lf_content::ItemEffect;
use lf_content::moves::ProjectileSpawn;

use super::inventory::{FlaskKind, slot};
use crate::arena::Id;
use crate::combat::projectile;
use crate::combat::status;
use crate::world::World;

/// The projectile a bow fires with no arrow in the arrow slot, and the one the arrow
/// slot's ammunition replaces. Bow aiming and firing are WP-052's; until then the stub
/// bow's `proj_arrow` is the base arrow.
const BASE_ARROW: &str = "proj_arrow";

/// Where a thrown item leaves the thrower's hand: x right, y up, z forward, from the
/// feet. Tuning: chest height, a little ahead.
const THROW_OFFSET: [f32; 3] = [0.0, 1.3, 0.4];
/// How steeply a thrown item is lobbed, in degrees above the horizontal. Tuning.
const THROW_PITCH_DEG: f32 = 15.0;

/// Applies the item whose effect landed last tick, if any. Called at the start of
/// `items::run`.
pub fn apply_landed(world: &mut World) {
    let player = world.player;
    let Some(FlaskKind::Landed(item)) = world
        .actors
        .get(player)
        .and_then(|p| p.inventory.flasks.drinking)
    else {
        return;
    };
    if let Some(p) = world.actors.get_mut(player) {
        p.inventory.flasks.drinking = None;
    }
    let Some(effect) = world.items.get(item).map(|d| d.effect.clone()) else {
        return;
    };
    let used = match effect {
        ItemEffect::Throw {
            projectile,
            phys,
            elem,
        } => throw(world, &projectile, phys, elem),
        ItemEffect::Oil { buff } => {
            let Some(num) = world.status.buff_num(&buff) else {
                return;
            };
            world
                .actors
                .get_mut(player)
                .is_some_and(|p| status::apply_buff(p, &world.status, num))
        }
        ItemEffect::Cure { kinds } => {
            if let Some(p) = world.actors.get_mut(player) {
                for k in kinds {
                    status::cure(p, k);
                }
            }
            true
        }
        ItemEffect::Glimmer { amount } => {
            if let Some(p) = world.actors.get_mut(player) {
                p.inventory.glimmer = p.inventory.glimmer.saturating_add(amount);
            }
            true
        }
        ItemEffect::LanternAsh => return_to_lantern(world),
        ItemEffect::None | ItemEffect::Ammo { .. } => false,
    };
    if used && let Some(p) = world.actors.get_mut(player) {
        p.inventory.remove(item, 1);
    }
}

/// Launches `projectile` from the player with `phys` physical and `elem` elemental
/// attack rating (the item's own damage, not the player's weapon's), in whole points.
fn throw(world: &mut World, projectile: &str, phys: u32, elem: u32) -> bool {
    let Some(move_num) = world.moves.flask.as_ref().map(|f| f.drink_move) else {
        return false;
    };
    let element = world
        .projectile_defs
        .num_of(projectile)
        .and_then(|n| world.projectile_defs.get(n))
        .and_then(|d| d.element);
    let spawn = ProjectileSpawn {
        frame: 0,
        projectile: projectile.to_owned(),
        offset: THROW_OFFSET,
        yaw_deg: 0.0,
        pitch_deg: THROW_PITCH_DEG,
    };
    let player = world.player;
    let Some(id) = projectile::spawn(world, player, move_num, &spawn) else {
        return false;
    };
    if let Some(p) = world.projectiles.get_mut(id) {
        p.ar_phys_milli = u64::from(phys) * 1000;
        p.ar_elem_milli = [0; 4];
        if let Some(e) = element {
            p.ar_elem_milli[super::gear::element_index(e)] = u64::from(elem) * 1000;
        }
        // A thrown item isn't boosted by the thrower's damage buffs.
        p.buff_percent = 100;
    }
    true
}

/// Moves the player to the last Lantern rested at (or the spawn), as a respawn does but
/// without respawning the enemies. False if the player is gone.
fn return_to_lantern(world: &mut World) -> bool {
    let (pos, yaw) = crate::world_state::death::respawn_point(world);
    let Some(p) = world.actors.get_mut(world.player) else {
        return false;
    };
    p.pos = pos;
    p.yaw = yaw;
    p.vel = [0.0; 3];
    p.lock_target = None;
    p.motor.grounded = true;
    p.motor.ground_y = pos[1];
    p.motor.fall_peak_y = pos[1];
    true
}

/// A bow's shot at the moment it spawns: if `shooter` is the player, the shot is the base
/// arrow, and the arrow slot holds ammunition in stock, spends one and returns the
/// projectile ID to fire instead and the percent of the shooter's physical attack rating
/// to add as the projectile's element. `None` leaves the shot as it is (no arrows
/// equipped, or none left: the stub bow keeps firing the base arrow).
pub fn take_ammo(world: &mut World, shooter: Id, projectile: &str) -> Option<(String, u8)> {
    // WP-052: the bow's heavy and fully drawn arrows are base arrows too: they spend ammunition
    // and take its swap (and so its fixed stats) like the light arrow does.
    let base =
        projectile == BASE_ARROW || matches!(projectile, "proj_arrow_heavy" | "proj_arrow_charged");
    if shooter != world.player || !base {
        return None;
    }
    let item = world.actors.get(shooter)?.inventory.equipment.slots[usize::from(slot::ARROWS)]?;
    let ItemEffect::Ammo {
        projectile: swap,
        elem_percent,
    } = world.items.get(item)?.effect.clone()
    else {
        return None;
    };
    let inv = &mut world.actors.get_mut(shooter)?.inventory;
    if inv.remove(item, 1) == 0 {
        return None;
    }
    Some((swap, elem_percent))
}
