//! The player's inventory: equipment slots, owned gear, item stacks, and flask charges.
//! See docs/game/06-progression-and-systems.md, "Flasks" and "Equipment slots", and
//! docs/game/04-arsenal-and-builds.md, "Consumables".
//!
//! Everything here is simulation state and part of the state hash, so it uses fixed-size
//! arrays and sorted `Vec`s only.

use lf_content::gear::ArmorSlot;
use lf_content::{ItemKind, NumId};
use serde::{Deserialize, Serialize};

use super::table::{FlaskRules, ItemTable};
use crate::actor::Actor;
use crate::combat::moves::{MoveNum, MoveTable};

/// Equipment slot numbers, as used by `Command::Equip` and `Command::Unequip`.
pub mod slot {
    pub const RIGHT_1: u8 = 0;
    pub const RIGHT_2: u8 = 1;
    pub const LEFT_1: u8 = 2;
    pub const LEFT_2: u8 = 3;
    pub const HEAD: u8 = 4;
    pub const BODY: u8 = 5;
    /// Four charm slots: `CHARM_1 + 0..4`. Only the first `charm_slots` are open.
    pub const CHARM_1: u8 = 6;
    /// Five quick item slots: `QUICK_1 + 0..5`.
    pub const QUICK_1: u8 = 10;
    pub const ARROWS: u8 = 15;
    /// Number of slot numbers in `Equipment::slots`.
    pub const COUNT: usize = 16;
    pub const QUICK_COUNT: u8 = 5;
    /// Spell attunement positions: `SPELL_1 + 0..SPELL_COUNT` (WP-055). These are slot
    /// numbers for `Command::Equip` and `Command::Unequip` only; the spells live in
    /// `World::spells` (`combat::spells`), not in `Equipment::slots`, so adding them left
    /// the state hash of every existing world alone. Append-only: new slot kinds go above
    /// `SPELL_1 + SPELL_COUNT`, not below.
    pub const SPELL_1: u8 = 32;
    pub const SPELL_COUNT: u8 = 14;
}

/// Charm slots open at the start (docs/game/06-progression-and-systems.md, "Equipment
/// slots": 2, then 3 and 4 after the first and second Great Sigils).
const START_CHARM_SLOTS: u8 = 2;

/// Which of the two flasks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlaskKind {
    /// Vigil Flask: heals HP.
    Vigil,
    /// Lucid Flask: restores FP.
    Lucid,
    /// Not a flask: the drink move is carrying this item (a throwable, oil, cure, Glimmer
    /// vial, or Lantern Ash) and its effect hasn't landed yet (WP-057).
    Item(NumId),
    /// That item's effect landed on the drink move's effect frame; `items::run` applies
    /// it on the next tick (it needs the whole world) and spends one from the stack.
    Landed(NumId),
}

/// Flask charges and potency (docs/game/06-progression-and-systems.md, "Flasks").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flasks {
    /// Charges in total; the Vigil and Lucid flasks share them. Wick Beads raise it.
    pub max_charges: u8,
    /// The Vigil Flask's share of `max_charges`; the Lucid Flask gets the rest.
    pub vigil_max: u8,
    /// Charges left in each flask.
    pub vigil: u8,
    pub lucid: u8,
    /// Potency level, +0 to the content's `max_level`. Flask Dew raises it.
    pub level: u8,
    /// The flask being drunk: set when a drink starts, read when its effect lands.
    pub drinking: Option<FlaskKind>,
}

impl Flasks {
    /// The Lucid Flask's share of the charges.
    pub fn lucid_max(&self) -> u8 {
        self.max_charges.saturating_sub(self.vigil_max)
    }

    /// Charges left in a flask; 0 for the item kinds, which have stacks instead.
    pub fn charges(&self, kind: FlaskKind) -> u8 {
        match kind {
            FlaskKind::Vigil => self.vigil,
            FlaskKind::Lucid => self.lucid,
            FlaskKind::Item(_) | FlaskKind::Landed(_) => 0,
        }
    }

    /// Refills both flasks to their split (resting at a Lantern).
    pub fn refill(&mut self) {
        self.vigil = self.vigil_max;
        self.lucid = self.lucid_max();
    }
}

/// What the player has equipped. Slot numbers are in [`slot`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equipment {
    /// The item in each slot, by slot number.
    pub slots: [Option<NumId>; slot::COUNT],
    /// The selected right-hand slot (0 or 1), left-hand slot (0 or 1), and quick slot (0
    /// to 4). The selected right-hand weapon is the one the player attacks with.
    pub right_active: u8,
    pub left_active: u8,
    pub quick_active: u8,
    /// Open charm slots.
    pub charm_slots: u8,
}

impl Default for Equipment {
    fn default() -> Self {
        Self {
            slots: [None; slot::COUNT],
            right_active: 0,
            left_active: 0,
            quick_active: 0,
            charm_slots: START_CHARM_SLOTS,
        }
    }
}

/// The group a slot number belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Group {
    Hand,
    Armor(ArmorSlot),
    Charm(u8),
    Quick,
    Arrows,
}

fn group(slot: u8) -> Option<Group> {
    match slot {
        slot::RIGHT_1..=slot::LEFT_2 => Some(Group::Hand),
        slot::HEAD => Some(Group::Armor(ArmorSlot::Head)),
        slot::BODY => Some(Group::Armor(ArmorSlot::Body)),
        6..=9 => Some(Group::Charm(slot - slot::CHARM_1)),
        10..=14 => Some(Group::Quick),
        slot::ARROWS => Some(Group::Arrows),
        _ => None,
    }
}

/// The hand slot after `active`, wrapping, whether or not it holds a weapon; `active`
/// itself if the hand holds nothing at all.
fn other_hand_slot(slots: &[Option<NumId>], active: u8) -> u8 {
    if slots.iter().all(Option::is_none) {
        return active;
    }
    ((usize::from(active) + 1) % slots.len()) as u8
}

/// The index after `active` within `slots` that holds an item, wrapping; `active` itself
/// if no other slot does.
fn next_filled(slots: &[Option<NumId>], active: u8) -> u8 {
    let n = slots.len();
    (1..=n)
        .map(|step| (usize::from(active) + step) % n)
        .find(|&i| slots[i].is_some())
        .map_or(active, |i| i as u8)
}

impl Equipment {
    /// The weapon in the selected right-hand slot.
    pub fn right_weapon(&self) -> Option<NumId> {
        self.slots[usize::from(slot::RIGHT_1 + self.right_active.min(1))]
    }

    /// The weapon in the selected left-hand slot.
    pub fn left_weapon(&self) -> Option<NumId> {
        self.slots[usize::from(slot::LEFT_1 + self.left_active.min(1))]
    }

    /// The selected quick item.
    pub fn selected_quick(&self) -> Option<NumId> {
        self.slots[usize::from(slot::QUICK_1 + self.quick_active.min(slot::QUICK_COUNT - 1))]
    }

    fn quick(&self) -> &[Option<NumId>] {
        &self.slots[usize::from(slot::QUICK_1)..usize::from(slot::QUICK_1 + slot::QUICK_COUNT)]
    }

    /// Selects the next filled quick slot (`CYCLE_ITEM`).
    pub fn cycle_quick(&mut self) {
        self.quick_active = next_filled(self.quick(), self.quick_active);
    }

    /// Selects the other right-hand slot (`CYCLE_RIGHT`). An empty slot is a valid choice:
    /// it leaves the hand unarmed, as in Elden Ring, so a lone weapon alternates with an
    /// empty hand (issue #20). A hand with nothing in either slot stays as it is.
    pub fn cycle_right(&mut self) {
        self.right_active = other_hand_slot(&self.slots[0..2], self.right_active);
    }

    /// Selects the other left-hand slot (`CYCLE_LEFT`), empty or not, like `cycle_right`.
    pub fn cycle_left(&mut self) {
        self.left_active = other_hand_slot(&self.slots[2..4], self.left_active);
    }

    /// After a slot empties: moves a selection off an empty slot onto a filled one.
    fn settle(&mut self) {
        let fix = |slots: &[Option<NumId>], active: u8| {
            if slots.get(usize::from(active)).is_some_and(Option::is_some) {
                active
            } else {
                next_filled(slots, active)
            }
        };
        self.right_active = fix(&self.slots[0..2], self.right_active);
        self.left_active = fix(&self.slots[2..4], self.left_active);
        self.quick_active = fix(self.quick(), self.quick_active);
    }
}

/// The player's inventory. Simulation state: part of the state hash.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub equipment: Equipment,
    /// Owned weapons, armor, and charms, sorted.
    pub gear: Vec<NumId>,
    /// Owned consumable stacks `(item, count)`, sorted by item.
    pub stacks: Vec<(NumId, u16)>,
    pub flasks: Flasks,
    /// Glimmer held: the currency that levels up the character. Dropped as a Remnant on
    /// death (`world_state::death`).
    pub glimmer: u32,
    /// Cycle buttons held on the previous tick, for press edges.
    pub prev_cycle: u16,
}

impl Inventory {
    /// The starting loadout: the default weapon in the first right-hand slot, the two
    /// flasks in the first two quick slots, and the starting charges and split.
    pub fn starting(items: &ItemTable) -> Self {
        let mut inv = Self::default();
        if let Some(w) = items.default_weapon {
            inv.gear.push(w);
            inv.equipment.slots[usize::from(slot::RIGHT_1)] = Some(w);
        }
        if let Some(f) = &items.flask {
            inv.equipment.slots[usize::from(slot::QUICK_1)] = Some(f.vigil);
            inv.equipment.slots[usize::from(slot::QUICK_1) + 1] = Some(f.lucid);
            inv.flasks = Flasks {
                max_charges: f.start_charges,
                vigil_max: f.start_vigil,
                vigil: f.start_vigil,
                lucid: f.start_charges - f.start_vigil,
                level: 0,
                drinking: None,
            };
        }
        inv
    }

    /// How many of `item` the player holds in a stack.
    pub fn count(&self, item: NumId) -> u16 {
        self.stacks
            .binary_search_by_key(&item, |s| s.0)
            .map_or(0, |i| self.stacks[i].1)
    }

    /// Adds up to `n` of a stackable `item`, to at most `cap`; returns how many it took.
    fn add_stack(&mut self, item: NumId, n: u16, cap: u16) -> u16 {
        let have = self.count(item);
        let take = n.min(cap.saturating_sub(have));
        if take == 0 {
            return 0;
        }
        match self.stacks.binary_search_by_key(&item, |s| s.0) {
            Ok(i) => self.stacks[i].1 += take,
            Err(i) => self.stacks.insert(i, (item, take)),
        }
        take
    }

    /// Removes up to `n` of `item`; returns how many it removed. An empty stack leaves the
    /// list but stays in any quick slot.
    pub fn remove(&mut self, item: NumId, n: u16) -> u16 {
        let Ok(i) = self.stacks.binary_search_by_key(&item, |s| s.0) else {
            return 0;
        };
        let take = n.min(self.stacks[i].1);
        self.stacks[i].1 -= take;
        if self.stacks[i].1 == 0 {
            self.stacks.remove(i);
        }
        take
    }

    /// Adds a weapon, armor piece, or charm to the owned gear. False if already owned.
    pub fn give_gear(&mut self, item: NumId) -> bool {
        match self.gear.binary_search(&item) {
            Ok(_) => false,
            Err(i) => {
                self.gear.insert(i, item);
                true
            }
        }
    }

    /// Picks up `n` of `item` and returns how many were taken (the rest stay in the
    /// world). A Wick Bead adds a flask charge (to the Vigil Flask's share, and full), up
    /// to the maximum; a Flask Dew raises the flasks' potency up to the maximum level; any
    /// other consumable goes on its stack, up to the stack limit. The flasks themselves,
    /// and unknown items, can't be picked up.
    pub fn pick_up(&mut self, item: NumId, n: u16, items: &ItemTable) -> u16 {
        // A charm or an armor piece goes to the owned gear: one of each.
        if items.gear.is_gear(item) {
            return u16::from(n > 0 && self.give_gear(item));
        }
        let (Some(def), Some(rules)) = (items.get(item), &items.flask) else {
            return 0;
        };
        match def.kind {
            ItemKind::VigilFlask | ItemKind::LucidFlask => 0,
            ItemKind::WickBead => {
                let room = rules.max_charges.saturating_sub(self.flasks.max_charges);
                let take = u8::try_from(n).unwrap_or(u8::MAX).min(room);
                self.flasks.max_charges += take;
                self.flasks.vigil_max += take;
                self.flasks.vigil += take;
                u16::from(take)
            }
            ItemKind::FlaskDew => {
                let room = rules.max_level.saturating_sub(self.flasks.level);
                let take = u8::try_from(n).unwrap_or(u8::MAX).min(room);
                self.flasks.level += take;
                u16::from(take)
            }
            _ => self.add_stack(item, n, def.stack),
        }
    }

    /// Refills both flasks (resting at a Lantern).
    pub fn refill_flasks(&mut self) {
        self.flasks.refill();
    }

    /// Sets how many charges the Vigil Flask gets; the Lucid Flask gets the rest. Both
    /// flasks refill (the split is set while resting at a Lantern). False if `vigil` is
    /// more than the charges the player has.
    pub fn split_flasks(&mut self, vigil: u8) -> bool {
        if vigil > self.flasks.max_charges {
            return false;
        }
        self.flasks.vigil_max = vigil;
        self.flasks.refill();
        true
    }

    /// Equips `item` in `slot`. False (and no change) if the slot doesn't exist or isn't
    /// open, or the item doesn't fit it or isn't owned:
    /// - hands: an owned weapon;
    /// - head and body: an owned armor piece of that slot;
    /// - charms: an owned charm, in an open charm slot;
    /// - quick slots: a flask, or a consumable with a stack, but not a Wick Bead, Flask Dew, or key;
    /// - arrows: ammunition with a stack.
    pub fn equip(&mut self, slot: u8, item: NumId, items: &ItemTable) -> bool {
        let owned_gear = self.gear.binary_search(&item).is_ok();
        let fits = match group(slot) {
            Some(Group::Hand) => items.is_weapon(item) && owned_gear,
            Some(Group::Armor(want)) => {
                owned_gear && items.gear.armor(item).is_some_and(|a| a.slot == want)
            }
            Some(Group::Charm(i)) => {
                i < self.equipment.charm_slots && owned_gear && items.gear.charm(item).is_some()
            }
            Some(Group::Quick) => items.get(item).is_some_and(|d| match d.kind {
                ItemKind::VigilFlask | ItemKind::LucidFlask => true,
                ItemKind::WickBead | ItemKind::FlaskDew | ItemKind::Key => false,
                _ => self.count(item) > 0,
            }),
            Some(Group::Arrows) => {
                items
                    .get(item)
                    .is_some_and(|d| d.kind == ItemKind::Ammunition)
                    && self.count(item) > 0
            }
            None => false,
        };
        if fits {
            self.equipment.slots[usize::from(slot)] = Some(item);
        }
        fits
    }

    /// Empties `slot`. False if the slot doesn't exist or is already empty.
    pub fn unequip(&mut self, slot: u8) -> bool {
        if group(slot).is_none() || self.equipment.slots[usize::from(slot)].is_none() {
            return false;
        }
        self.equipment.slots[usize::from(slot)] = None;
        self.equipment.settle();
        true
    }

    /// Which flask the selected quick item is, if it is one.
    pub fn selected_flask(&self, rules: &FlaskRules) -> Option<FlaskKind> {
        match self.equipment.selected_quick()? {
            n if n == rules.vigil => Some(FlaskKind::Vigil),
            n if n == rules.lucid => Some(FlaskKind::Lucid),
            _ => None,
        }
    }

    /// The selected quick item if `USE_ITEM` uses it on the drink move: an item with an
    /// effect that is in stock.
    pub fn selected_usable(&self, rules: &FlaskRules) -> Option<NumId> {
        let n = self.equipment.selected_quick()?;
        (rules.usable.binary_search(&n).is_ok() && self.count(n) > 0).then_some(n)
    }

    /// HP a Vigil Flask drink heals at the current potency.
    pub fn heal_amount(&self, rules: &FlaskRules) -> u32 {
        u32::from(rules.heal_base) + u32::from(rules.heal_per_level) * u32::from(self.flasks.level)
    }

    /// FP a Lucid Flask drink restores at the current potency.
    pub fn fp_amount(&self, rules: &FlaskRules) -> u32 {
        u32::from(rules.fp_base) + u32::from(rules.fp_per_level) * u32::from(self.flasks.level)
    }
}

/// The move a `USE_ITEM` press starts now: the flask drink, if the selected quick item
/// is a flask with a charge left, or an item that has an effect and is in stock, and the
/// current move (if any) has reached its `cancel_any_from`.
pub fn drink_move(a: &Actor, table: &MoveTable) -> Option<MoveNum> {
    let rules = table.flask.as_ref()?;
    if let Some(c) = a.moves.current {
        let from = table.get(c.num)?.data.cancel_any_from?;
        if c.frame < from {
            return None;
        }
    }
    match a.inventory.selected_flask(rules) {
        Some(kind) => (a.inventory.flasks.charges(kind) > 0).then_some(rules.drink_move),
        None => a
            .inventory
            .selected_usable(rules)
            .is_some()
            .then_some(rules.drink_move),
    }
}

/// Called when a flask drink starts: remembers which flask is being drunk. Spends nothing;
/// the charge goes when the effect lands, so an interrupted drink costs no charge.
pub fn drink_starts(a: &mut Actor, rules: &FlaskRules) {
    a.inventory.flasks.drinking = a
        .inventory
        .selected_flask(rules)
        .or_else(|| a.inventory.selected_usable(rules).map(FlaskKind::Item));
}

/// Called on the drink move's effect frame: spends one charge of the flask being drunk
/// and applies it (HP healed, or FP restored, capped at the maximum).
pub fn drink_lands(a: &mut Actor, rules: &FlaskRules) {
    let Some(kind) = a.inventory.flasks.drinking.take() else {
        return;
    };
    // An item's effect needs the world: hand it to `items::run` (see `FlaskKind::Landed`).
    if let FlaskKind::Item(n) = kind {
        if a.inventory.count(n) > 0 {
            a.inventory.flasks.drinking = Some(FlaskKind::Landed(n));
        }
        return;
    }
    if a.inventory.flasks.charges(kind) == 0 {
        return;
    }
    match kind {
        FlaskKind::Item(_) | FlaskKind::Landed(_) => {}
        FlaskKind::Vigil => {
            a.inventory.flasks.vigil -= 1;
            let heal = a.inventory.heal_amount(rules);
            a.hp = a.hp.saturating_add(heal).min(a.max_hp);
        }
        FlaskKind::Lucid => {
            a.inventory.flasks.lucid -= 1;
            let fp = a.inventory.fp_amount(rules) as f32;
            a.fp = (a.fp + fp).min(a.max_fp);
        }
    }
}
