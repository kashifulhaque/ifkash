//! Items: the player's inventory, equipment slots, quick slots and cycling, flasks, and
//! pickups. See docs/game/06-progression-and-systems.md, "Flasks" and "Equipment slots",
//! and docs/game/04-arsenal-and-builds.md, "Consumables".
//!
//! # Where things happen
//!
//! - [`run`] is the body of tick system 1 (`systems::apply_commands`): it applies the
//!   tick's `Command`s (`Equip`, `Unequip`, `UseItem`, `SplitFlasks`, only while resting at a Lantern) and the cycle
//!   buttons (`CYCLE_ITEM`, `CYCLE_RIGHT`, `CYCLE_LEFT`), then points the player's active
//!   moveset at the equipped right-hand weapon.
//! - Drinking a flask is a move (`flask_drink` in `moves/common.ron`) started by
//!   `USE_ITEM` or `Command::UseItem`. `combat::moves::step` starts it (see
//!   [`inventory::drink_move`]) and fires its effect on the content's `effect_frame`
//!   ([`inventory::drink_lands`]), which is when the charge is spent. A stagger or guard
//!   break before that frame ends the move with no heal and no charge spent.
//! - The other consumables (throwables, oils, cures, Glimmer vials, Lantern Ash) use the
//!   same drink move; [`usage::apply_landed`] applies the effect on the tick after it
//!   lands. Arrows are [`usage::take_ammo`].
//! - Charms and armor are [`gear`]: [`gear::refresh`] writes the stored stats when the
//!   loadout changes, and the combat systems query the rest.
//! - Pickups are [`Inventory::pick_up`]; world pickups (WP-0xx) call it.

pub mod gear;
pub mod inventory;
pub mod preview;
pub mod table;
pub mod upgrade;
pub mod usage;

pub use inventory::{Equipment, FlaskKind, Flasks, Inventory, slot};
pub use table::{FlaskRules, ItemDef, ItemTable};

use lf_content::NumId;

use crate::combat::moves::Action;
use crate::input::{Command, TickInput, buttons};
use crate::world::World;

/// The cycle buttons whose press edges this module reads.
const CYCLE_BUTTONS: u16 = buttons::CYCLE_ITEM | buttons::CYCLE_RIGHT | buttons::CYCLE_LEFT;

/// Applies the tick's item commands and cycle buttons to the player.
pub fn run(world: &mut World, input: &TickInput) {
    // The item whose effect landed last tick takes effect now (WP-057).
    usage::apply_landed(world);
    let id = world.player;
    // The Lantern menu (Split flasks) is open only while resting at a Lantern.
    let resting = world.state.resting.is_some();
    let Some(actor) = world.actors.get_mut(id) else {
        return;
    };
    if actor.dead {
        return;
    }
    let weapon_before = actor.inventory.equipment.right_weapon();
    let gear_before = gear::gear_slots(&actor.inventory.equipment);
    let mut spell_cmds: Vec<(bool, u8, NumId)> = Vec::new();
    for c in &input.commands {
        match *c {
            // Spell positions attune a spell instead (`combat::spells`); that needs the
            // whole world, so it runs after this loop.
            Command::Equip { slot, item } if slot >= slot::SPELL_1 => {
                spell_cmds.push((true, slot - slot::SPELL_1, item));
            }
            Command::Unequip { slot } if slot >= slot::SPELL_1 => {
                spell_cmds.push((false, slot - slot::SPELL_1, 0));
            }
            Command::Equip { slot, item } => {
                actor.inventory.equip(slot, item, &world.items);
            }
            Command::Unequip { slot } => {
                actor.inventory.unequip(slot);
            }
            Command::UseItem { slot } => {
                // `slot` is a quick slot index, 0 to 4. An empty slot does nothing.
                let eq = &mut actor.inventory.equipment;
                let at = usize::from(slot::QUICK_1) + usize::from(slot);
                if slot < slot::QUICK_COUNT && eq.slots[at].is_some() {
                    eq.quick_active = slot;
                    actor.moves.press(Action::UseItem, [0, 0]);
                }
            }
            Command::SplitFlasks { vigil } if resting => {
                actor.inventory.split_flasks(vigil);
            }
            _ => {}
        }
    }

    let pressed = input.frame.buttons & CYCLE_BUTTONS & !actor.inventory.prev_cycle;
    actor.inventory.prev_cycle = input.frame.buttons & CYCLE_BUTTONS;
    let eq = &mut actor.inventory.equipment;
    if pressed & buttons::CYCLE_ITEM != 0 {
        eq.cycle_quick();
    }
    // Hands don't swap in the middle of a move.
    if actor.moves.current.is_none() {
        if pressed & buttons::CYCLE_RIGHT != 0 {
            eq.cycle_right();
        }
        if pressed & buttons::CYCLE_LEFT != 0 {
            eq.cycle_left();
        }
    }
    // Only a change of weapon swaps the moveset, so tuning applied to the active moveset
    // (the equip load tier, in tests) isn't overwritten every tick.
    let weapon_changed = actor.inventory.equipment.right_weapon() != weapon_before;
    let gear_changed = gear::gear_slots(&actor.inventory.equipment) != gear_before;
    let worn = gear::any_worn(&actor.inventory.equipment);
    if weapon_changed {
        world.sync_player_moveset();
    }
    // A new loadout rewrites the stored stats; swapping weapons while wearing gear
    // refreshes the load tier the new moveset reset (WP-057).
    if gear_changed {
        gear::refresh(world);
    } else if weapon_changed && worn {
        gear::sync_load_tier(world);
    }
    gear::tick(world);
    // `CYCLE_SPELL` selects the next attuned spell. The moves system keeps last tick's
    // buttons until it runs later this tick, so the press edge is against those.
    let spell_pressed = input.frame.buttons & buttons::CYCLE_SPELL != 0
        && world
            .actors
            .get(world.player)
            .is_some_and(|p| p.moves.prev_buttons & buttons::CYCLE_SPELL == 0);
    for (attune, index, item) in spell_cmds {
        if attune {
            crate::combat::spells::attune(world, index, item);
        } else {
            crate::combat::spells::unattune(world, index);
        }
    }
    if spell_pressed {
        crate::combat::spells::cycle(world);
    }
}

#[cfg(test)]
mod gear_tests;
#[cfg(test)]
mod tests;
