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
//! - Pickups are [`Inventory::pick_up`]; world pickups (WP-0xx) call it.

pub mod inventory;
pub mod preview;
pub mod table;

pub use inventory::{Equipment, FlaskKind, Flasks, Inventory, slot};
pub use table::{FlaskRules, ItemDef, ItemTable};

use crate::combat::moves::Action;
use crate::input::{Command, TickInput, buttons};
use crate::world::World;

/// The cycle buttons whose press edges this module reads.
const CYCLE_BUTTONS: u16 = buttons::CYCLE_ITEM | buttons::CYCLE_RIGHT | buttons::CYCLE_LEFT;

/// Applies the tick's item commands and cycle buttons to the player.
pub fn run(world: &mut World, input: &TickInput) {
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
    for c in &input.commands {
        match *c {
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
    if actor.inventory.equipment.right_weapon() != weapon_before {
        world.sync_player_moveset();
    }
}

#[cfg(test)]
mod tests;
