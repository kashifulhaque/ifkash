//! Read-only stat previews for the equipment menu: what an equip or unequip would do to
//! attack rating, defense, equip load, and roll tier, without changing the world. See
//! docs/game/06-progression-and-systems.md, "Equipment slots" (the menu shows each change's
//! effect before the player confirms it).
//!
//! A preview applies the change to a copy of the player's inventory, using the same
//! [`Inventory::equip`] and [`Inventory::unequip`] the commands use, so what it says can
//! fit is exactly what the command would accept.
//!
//! Equip load counts the weapons in all four hand slots plus the armor stub
//! (`stub_armor_weight_tenths`) until armor exists. The roll tier the simulation uses is
//! still the per-moveset stub in `combat::moves`, so it doesn't follow equipment changes
//! yet; the preview shows the tier the spec's rule gives.

use super::{Inventory, slot};
use crate::combat::dodge::{LoadTier, load_tier};
use crate::world::World;

/// The numbers the equipment menu shows for one loadout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadoutStats {
    /// Physical attack rating of the selected right-hand weapon, in thousandths,
    /// one-handed. 0 with no weapon in that hand.
    pub ar_phys_milli: u64,
    /// The weapon's requirements aren't met: scaling is lost and recovery is longer.
    pub unwieldy: bool,
    /// Flat defense against physical damage.
    pub defense: u32,
    /// Equip load and its maximum, in tenths of a unit.
    pub load_tenths: u32,
    pub max_load_tenths: u32,
    pub tier: LoadTier,
}

/// The stats of the player's inventory `inv`. `None` if the player is gone or the
/// content has no locomotion tuning to read the load tiers from.
pub fn loadout_stats(world: &World, inv: &Inventory) -> Option<LoadoutStats> {
    let p = world.actors.get(world.player)?;
    let eq = &inv.equipment;
    let (ar, unwieldy) = match eq.right_weapon() {
        Some(w) => {
            let ar =
                world
                    .damage
                    .attack_rating(w, &p.stats.attrs, p.stats.weapon_upgrade, false)?;
            (ar.phys_milli, ar.unwieldy)
        }
        None => (0, false),
    };
    let hands = &eq.slots[usize::from(slot::RIGHT_1)..=usize::from(slot::LEFT_2)];
    let weapons: u32 = hands
        .iter()
        .flatten()
        .map(|&n| u32::from(world.items.weight_tenths(n)))
        .sum();
    let load = weapons + u32::from(world.locomotion.stub_armor_weight_tenths);
    let max = world
        .attributes
        .max_equip_load_tenths(p.stats.attrs.endurance)
        .max(0) as u32;
    Some(LoadoutStats {
        ar_phys_milli: ar,
        unwieldy,
        // Index 0 is `DamageType::Neutral`, the stub's single physical defense.
        defense: p.stats.defense.flat[0],
        load_tenths: load,
        max_load_tenths: max,
        tier: load_tier(load, max, &world.locomotion.equip_load),
    })
}

/// The change a preview applies: put `item` in `slot`, or empty the slot when `item` is 0.
fn applied(world: &World, slot: u8, item: u32) -> Option<(bool, Inventory)> {
    let mut inv = world.actors.get(world.player)?.inventory.clone();
    let ok = if item == 0 {
        inv.unequip(slot)
    } else {
        inv.equip(slot, item, &world.items)
    };
    Some((ok, inv))
}

/// Whether `Command::Equip { slot, item }` (or `Unequip` for item 0) would change anything.
pub fn can_equip(world: &World, slot: u8, item: u32) -> bool {
    applied(world, slot, item).is_some_and(|(ok, _)| ok)
}

/// The stats after the change, and whether the change is allowed. When it isn't, the stats
/// are the current ones.
pub fn preview_equip(world: &World, slot: u8, item: u32) -> Option<(bool, LoadoutStats)> {
    let (ok, inv) = applied(world, slot, item)?;
    Some((ok, loadout_stats(world, &inv)?))
}

#[cfg(test)]
mod tests {
    use lf_content::Content;

    use super::*;

    fn field() -> World {
        let content = Content::load_embedded().expect("embedded content");
        World::new(9, &content, "test_field").expect("test_field exists")
    }

    fn current(w: &World) -> LoadoutStats {
        loadout_stats(w, &w.actors.get(w.player).unwrap().inventory).unwrap()
    }

    #[test]
    fn the_starting_loadout_has_a_weapon_and_a_load() {
        let w = field();
        let s = current(&w);
        assert!(s.ar_phys_milli > 0);
        let blade = w.items.default_weapon.unwrap();
        assert_eq!(
            s.load_tenths,
            u32::from(w.items.weight_tenths(blade))
                + u32::from(w.locomotion.stub_armor_weight_tenths)
        );
        assert!(s.max_load_tenths > s.load_tenths);
    }

    #[test]
    fn a_preview_changes_nothing_and_matches_the_command() {
        let mut w = field();
        let dagger = (1..10_000u32)
            .find(|&n| w.items.is_weapon(n) && Some(n) != w.items.default_weapon)
            .expect("a second weapon");
        let id = w.player;
        let before = w.state_hash();
        // Not owned: the preview refuses, as the command would.
        assert!(!can_equip(&w, slot::LEFT_1, dagger));
        let (ok, same) = preview_equip(&w, slot::LEFT_1, dagger).unwrap();
        assert!(!ok);
        assert_eq!(same, current(&w));
        // Owned: the preview shows the heavier load, and the world is untouched.
        w.actors.get_mut(id).unwrap().inventory.give_gear(dagger);
        let before_owned = w.state_hash();
        let (ok, after) = preview_equip(&w, slot::LEFT_1, dagger).unwrap();
        assert!(ok);
        assert_eq!(
            after.load_tenths,
            current(&w).load_tenths + u32::from(w.items.weight_tenths(dagger))
        );
        assert_eq!(w.state_hash(), before_owned);
        assert_ne!(before, before_owned, "giving gear is a state change");
        // Emptying the right hand drops the attack rating to 0 and the load by the weapon.
        let (ok, bare) = preview_equip(&w, slot::RIGHT_1, 0).unwrap();
        assert!(ok);
        assert_eq!(bare.ar_phys_milli, 0);
        assert!(bare.load_tenths < current(&w).load_tenths);
        // An empty slot can't be emptied.
        assert!(!can_equip(&w, slot::HEAD, 0));
    }
}
