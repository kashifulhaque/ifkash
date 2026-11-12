//! Charms and armor: what the equipped gear does. See docs/game/04-arsenal-and-builds.md,
//! "Charms" and "Armor", and docs/game/06-progression-and-systems.md, "Equipment slots".
//!
//! The definitions come from `charms.ron` and `armor.ron` ([`GearTable`]). Nothing here
//! stores a gear bonus: every query reads the equipped slots (`Equipment`), which are
//! already part of the state hash, so a loadout always gives the same numbers.
//!
//! # Two kinds of effect
//!
//! - **Stored stats** are written into the player when the loadout changes
//!   ([`refresh`], called by `items::run` after an equip or unequip of armor or a charm):
//!   defense and absorption, poise, the three resistances, and max HP. [`refresh`] works
//!   from the attributes, not from the old value, so it can be called again at any time
//!   (a level-up does, WP-050) and gives the same result.
//! - **Queries** are read where the effect happens, by the systems that own that rule:
//!   stamina regeneration (`systems::status_regen`), the parry window (`combat::poise`),
//!   the block cost (`combat::resolve`), damage dealt ([`outgoing`], `combat::resolve` and
//!   `combat::projectile`), equip load ([`load_tenths`]), what enemies see
//!   ([`scale_sight`], `ai`), and the procs ([`on_status_proc`], `combat::status`).
//!   FP regeneration is [`tick`]. The two spell charms (Scholar's Seal's slot,
//!   Shrike-Lite's startup) are queries for the spell work (WP-055).
//!
//! # The armor stub
//!
//! Until the player wears armor, `stub_defense`, `stub_poise` and `stub_armor_weight_tenths`
//! in `locomotion.ron` stand for the starting coat. With any piece equipped the pieces
//! replace the stub entirely, and with none equipped again the stub is back.

use lf_content::gear::{ArmorSlot, GearEffect, Resistance};
use lf_content::moves::{Element, StatusKind};
use lf_content::{Content, NumId};
use serde::{Deserialize, Serialize};

use super::inventory::{Equipment, Inventory, slot};
use crate::actor::Actor;
use crate::actor::stats::Defense;
use crate::combat::damage::{Damage, Hit, TYPE_COUNT, element_type};
use crate::combat::dodge::{LoadTier, load_tier};
use crate::combat::status::{self, StatusState, StatusTable};
use crate::world::World;

/// A charm from content, with its registry number.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharmDef {
    pub num: NumId,
    pub id: String,
    pub name: String,
    pub relic: bool,
    pub effects: Vec<GearEffect>,
}

/// An armor piece from content, with its registry number.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArmorDef {
    pub num: NumId,
    pub id: String,
    pub name: String,
    pub set: String,
    pub slot: ArmorSlot,
    pub weight_tenths: u16,
    pub defense_physical: u32,
    pub defense_elemental: u32,
    /// Percent absorbed per damage type, by `DamageType as usize`.
    pub absorb: [u8; TYPE_COUNT],
    pub poise: u16,
    pub effects: Vec<GearEffect>,
}

/// The sums over the armor the player wears.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArmorTotals {
    /// At least one piece is equipped (otherwise the stub applies).
    pub worn: bool,
    pub defense_physical: u32,
    pub defense_elemental: u32,
    /// Percent absorbed per damage type, capped at 100 here (the damage rules cap lower).
    pub absorb: [u8; TYPE_COUNT],
    pub poise: u32,
    pub weight_tenths: u32,
}

/// Every charm and armor piece, sorted by registry number. Built from content, so it
/// isn't hashed: the content hash covers it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GearTable {
    charms: Vec<CharmDef>,
    armor: Vec<ArmorDef>,
}

impl GearTable {
    pub fn from_content(content: &Content) -> Result<Self, String> {
        let num = |id: &str| {
            content
                .registry
                .num(id)
                .ok_or_else(|| format!("\"{id}\" is missing from registry.ron"))
        };
        let mut charms = Vec::new();
        for c in &content.charms {
            charms.push(CharmDef {
                num: num(&c.id)?,
                id: c.id.clone(),
                name: c.name.clone(),
                relic: c.relic,
                effects: c.effects.clone(),
            });
        }
        charms.sort_by_key(|c| c.num);
        let mut armor = Vec::new();
        for p in &content.armor {
            let mut absorb = [0u8; TYPE_COUNT];
            for &(target, pct) in &p.absorption {
                for &t in target.types() {
                    absorb[t as usize] = absorb[t as usize].saturating_add(pct).min(100);
                }
            }
            armor.push(ArmorDef {
                num: num(&p.id)?,
                id: p.id.clone(),
                name: p.name.clone(),
                set: p.set.clone(),
                slot: p.slot,
                weight_tenths: p.weight_tenths,
                defense_physical: p.defense.physical,
                defense_elemental: p.defense.elemental,
                absorb,
                poise: p.poise,
                effects: p.effects.clone(),
            });
        }
        armor.sort_by_key(|a| a.num);
        Ok(Self { charms, armor })
    }

    pub fn charm(&self, n: NumId) -> Option<&CharmDef> {
        self.charms
            .binary_search_by_key(&n, |c| c.num)
            .ok()
            .map(|i| &self.charms[i])
    }

    pub fn armor(&self, n: NumId) -> Option<&ArmorDef> {
        self.armor
            .binary_search_by_key(&n, |a| a.num)
            .ok()
            .map(|i| &self.armor[i])
    }

    /// Every charm, sorted by number.
    pub fn charms(&self) -> &[CharmDef] {
        &self.charms
    }

    /// Every armor piece, sorted by number.
    pub fn armor_pieces(&self) -> &[ArmorDef] {
        &self.armor
    }

    /// Whether `n` is a charm or an armor piece.
    pub fn is_gear(&self, n: NumId) -> bool {
        self.charm(n).is_some() || self.armor(n).is_some()
    }

    /// The effects of `n`, if it is a charm or an armor piece.
    fn effects_of(&self, n: NumId) -> &[GearEffect] {
        if let Some(c) = self.charm(n) {
            &c.effects
        } else if let Some(a) = self.armor(n) {
            &a.effects
        } else {
            &[]
        }
    }

    /// The effects of everything worn: both armor slots and the open charm slots.
    fn worn_effects<'a>(&'a self, eq: &'a Equipment) -> impl Iterator<Item = &'a GearEffect> {
        let charms = usize::from(eq.charm_slots.min(4));
        let armor = [slot::HEAD, slot::BODY].map(usize::from);
        let first = usize::from(slot::CHARM_1);
        armor
            .into_iter()
            .chain(first..first + charms)
            .filter_map(|i| eq.slots[i])
            .flat_map(|n| self.effects_of(n).iter())
    }

    /// Max HP, in percent more.
    pub fn max_hp_percent(&self, eq: &Equipment) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::MaxHpPercent(p) => u32::from(*p),
            _ => 0,
        })
    }

    /// Max equip load, in percent more.
    pub fn equip_load_percent(&self, eq: &Equipment) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::EquipLoadPercent(p) => u32::from(*p),
            _ => 0,
        })
    }

    /// Stamina regeneration, in percent more.
    pub fn stamina_regen_percent(&self, eq: &Equipment) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::StaminaRegenPercent(p) => u32::from(*p),
            _ => 0,
        })
    }

    /// The parry window, in ticks more.
    pub fn parry_window_bonus(&self, eq: &Equipment) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::ParryWindowTicks(t) => u32::from(*t),
            _ => 0,
        })
    }

    /// Formula damage, in percent more.
    pub fn formula_damage_percent(&self, eq: &Equipment) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::FormulaDamagePercent(p) => u32::from(*p),
            _ => 0,
        })
    }

    /// Spell slots, more. For the spell work (WP-055) to add to the Focus-derived slots.
    pub fn spell_slot_bonus(&self, eq: &Equipment) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::SpellSlots(n) => u32::from(*n),
            _ => 0,
        })
    }

    /// Spell startup, in percent less (at most 100). For the spell work (WP-055): a
    /// startup of `n` frames becomes `n * (100 - reduce) / 100`.
    pub fn spell_startup_reduce_percent(&self, eq: &Equipment) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::SpellStartupReducePercent(p) => u32::from(*p),
            _ => 0,
        })
        .min(100)
    }

    /// The weight of equipped weapons, in percent less (at most 100).
    pub fn weapon_weight_reduce_percent(&self, eq: &Equipment) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::WeaponWeightReducePercent(p) => u32::from(*p),
            _ => 0,
        })
        .min(100)
    }

    /// The stamina a block costs, in percent less (at most 100).
    pub fn guard_stamina_reduce_percent(&self, eq: &Equipment) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::GuardStaminaReducePercent(p) => u32::from(*p),
            _ => 0,
        })
        .min(100)
    }

    /// Damage of `element`, in percent more.
    pub fn element_damage_percent(&self, eq: &Equipment, element: Element) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::ElementDamagePercent {
                element: x,
                percent,
            } if *x == element => u32::from(*percent),
            _ => 0,
        })
    }

    /// Damage in percent more at `hp` of `max_hp` (the low-HP charms).
    pub fn low_hp_damage_percent(&self, eq: &Equipment, hp: u32, max_hp: u32) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::LowHpDamage {
                below_percent,
                percent,
            } if u64::from(hp) * 100 < u64::from(max_hp) * u64::from(*below_percent) => {
                u32::from(*percent)
            }
            _ => 0,
        })
    }

    /// FP restored on tick `tick` (the FP charms).
    pub fn fp_regen_at(&self, eq: &Equipment, tick: u64) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::FpRegen {
                every_ticks,
                amount,
            } if tick.is_multiple_of(u64::from(*every_ticks)) => u32::from(*amount),
            _ => 0,
        })
    }

    /// A resistance, more.
    pub fn resist(&self, eq: &Equipment, kind: Resistance) -> u32 {
        self.sum(eq, |e| match e {
            GearEffect::Resist { kind: k, amount } if *k == kind => u32::from(*amount),
            _ => 0,
        })
    }

    /// What enemies see of the wearer, in percent of their sight range: the product of
    /// every sight effect that applies (`dark` says whether the wearer stands in darkness).
    pub fn sight_percent(&self, eq: &Equipment, dark: bool) -> u32 {
        self.worn_effects(eq).fold(100u32, |acc, e| match e {
            GearEffect::SightPercent { percent, dark_only } if dark || !dark_only => {
                acc * u32::from(*percent) / 100
            }
            _ => acc,
        })
    }

    fn sum(&self, eq: &Equipment, f: impl Fn(&GearEffect) -> u32) -> u32 {
        self.worn_effects(eq).map(f).sum()
    }

    /// The sums over the armor in the head and body slots.
    pub fn armor_totals(&self, eq: &Equipment) -> ArmorTotals {
        let mut t = ArmorTotals::default();
        for s in [slot::HEAD, slot::BODY] {
            let Some(a) = eq.slots[usize::from(s)].and_then(|n| self.armor(n)) else {
                continue;
            };
            t.worn = true;
            t.defense_physical += a.defense_physical;
            t.defense_elemental += a.defense_elemental;
            t.poise += u32::from(a.poise);
            t.weight_tenths += u32::from(a.weight_tenths);
            for (sum, add) in t.absorb.iter_mut().zip(a.absorb) {
                *sum = sum.saturating_add(add).min(100);
            }
        }
        t
    }
}

// --- Equip load -------------------------------------------------------------------

/// The equip load of `inv` in tenths of a unit: the four hand slots' weapons (lighter
/// with smol-llama) plus the armor, or the stub while no armor is worn.
pub fn load_tenths(world: &World, inv: &Inventory) -> u32 {
    let eq = &inv.equipment;
    let reduce = world.items.gear.weapon_weight_reduce_percent(eq);
    let weapons: u32 = eq.slots[usize::from(slot::RIGHT_1)..=usize::from(slot::LEFT_2)]
        .iter()
        .flatten()
        .map(|&n| u32::from(world.items.weight_tenths(n)) * (100 - reduce) / 100)
        .sum();
    let armor = world.items.gear.armor_totals(eq);
    let armor_weight = if armor.worn {
        armor.weight_tenths
    } else {
        u32::from(world.locomotion.stub_armor_weight_tenths)
    };
    weapons + armor_weight
}

/// The heaviest load the wearer of `inv` carries without penalty, in tenths: the
/// Endurance curve, raised by Ox Charm.
pub fn max_load_tenths(world: &World, endurance: u16, inv: &Inventory) -> u32 {
    let base = world.attributes.max_equip_load_tenths(endurance).max(0) as u64;
    let pct = u64::from(world.items.gear.equip_load_percent(&inv.equipment));
    (base * (100 + pct) / 100) as u32
}

/// The equip load tier of `inv` in `attrs`' hands.
pub fn tier_of(world: &World, endurance: u16, inv: &Inventory) -> LoadTier {
    load_tier(
        load_tenths(world, inv),
        max_load_tenths(world, endurance, inv),
        &world.locomotion.equip_load,
    )
}

/// Flat physical defense of `inv`'s armor (the stub's while none is worn): the number
/// the equipment menu shows.
pub fn defense_physical(world: &World, inv: &Inventory) -> u32 {
    let armor = world.items.gear.armor_totals(&inv.equipment);
    if armor.worn {
        armor.defense_physical
    } else {
        world.locomotion.stub_defense
    }
}

/// Whether any armor or charm is worn.
pub fn any_worn(eq: &Equipment) -> bool {
    eq.slots[usize::from(slot::HEAD)..=usize::from(slot::BODY)]
        .iter()
        .any(Option::is_some)
        || eq.slots[usize::from(slot::CHARM_1)..usize::from(slot::CHARM_1) + 4]
            .iter()
            .any(Option::is_some)
}

/// The slots whose contents [`refresh`] depends on: head, body, and the four charms.
pub fn gear_slots(eq: &Equipment) -> [Option<NumId>; 6] {
    let at = |i: u8| eq.slots[usize::from(i)];
    [
        at(slot::HEAD),
        at(slot::BODY),
        at(slot::CHARM_1),
        at(slot::CHARM_1 + 1),
        at(slot::CHARM_1 + 2),
        at(slot::CHARM_1 + 3),
    ]
}

// --- Stored stats -----------------------------------------------------------------

/// Writes the stats that armor and charms set into the player: defense and absorption,
/// poise, resistances, max HP, and the equip load tier. Works from the attributes, so
/// it can run any time and gives the same answer (a level-up should call it).
pub fn refresh(world: &mut World) {
    let id = world.player;
    let Some(p) = world.actors.get(id) else {
        return;
    };
    let eq = p.inventory.equipment;
    let attrs = p.stats.attrs;
    let armor = world.items.gear.armor_totals(&eq);
    let stub = world.locomotion;
    let gear = &world.items.gear;
    let resist_extra = |k| gear.resist(&eq, k);
    let hp_pct = u64::from(gear.max_hp_percent(&eq));
    let base_status = StatusState::for_player(&attrs, &world.attributes);
    let base_hp = attrs.derived(&world.attributes).max_hp;
    let Some(p) = world.actors.get_mut(id) else {
        return;
    };
    // Defense and absorption; weakness is left as it is.
    for t in 0..TYPE_COUNT {
        let physical = t < 4;
        (p.stats.defense.flat[t], p.stats.defense.absorb_percent[t]) = if armor.worn {
            (
                if physical {
                    armor.defense_physical
                } else {
                    armor.defense_elemental
                },
                armor.absorb[t],
            )
        } else {
            (stub.stub_defense, 0)
        };
    }
    p.poise.poise = if armor.worn {
        u16::try_from(armor.poise).unwrap_or(u16::MAX)
    } else {
        stub.poise.stub_poise
    };
    // Resistances: the attribute curves plus the gear.
    let extra = [
        (StatusKind::Poison, resist_extra(Resistance::Immunity)),
        (StatusKind::Blight, resist_extra(Resistance::Immunity)),
        (StatusKind::Rupture, resist_extra(Resistance::Robustness)),
        (StatusKind::Rime, resist_extra(Resistance::Robustness)),
        (StatusKind::Drowse, resist_extra(Resistance::Composure)),
        (StatusKind::Delirium, resist_extra(Resistance::Composure)),
    ];
    p.status.resist = base_status.resist;
    for (kind, more) in extra {
        let i = status::index(kind);
        p.status.resist[i] =
            p.status.resist[i].saturating_add(u16::try_from(more).unwrap_or(u16::MAX));
    }
    // Max HP.
    let max_hp = (u64::from(base_hp) * (100 + hp_pct) / 100) as u32;
    p.max_hp = max_hp;
    p.hp = p.hp.min(max_hp);
    // Shrike-Lite shortens spell startup (the cast speed rule reads it).
    world.moves.cast_reduce_percent = world.items.gear.spell_startup_reduce_percent(&eq);
    sync_load_tier(world);
}

/// Sets the player's moveset load tier from what the player wears and wields.
pub fn sync_load_tier(world: &mut World) {
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    let tier = tier_of(world, p.stats.attrs.endurance, &p.inventory);
    if let Some(set) = world.moves.player_moveset.as_mut() {
        set.load_tier = tier;
    }
}

/// Per-tick gear effects: FP regeneration (Lantern Moth). Called from `items::run`.
pub fn tick(world: &mut World) {
    let Some(p) = world.actors.get_mut(world.player) else {
        return;
    };
    if p.dead {
        return;
    }
    let fp = world
        .items
        .gear
        .fp_regen_at(&p.inventory.equipment, world.tick);
    if fp > 0 {
        p.fp = (p.fp + fp as f32).min(p.max_fp);
    }
}

// --- Damage -----------------------------------------------------------------------

/// A weapon buff that adds an element (an oil) turns a hit with no element of its own
/// into one that also deals that element, at the buff's percent of the physical attack
/// rating.
pub fn weapon_element(attacker: &Actor, table: &StatusTable, mut hit: Hit) -> Hit {
    if hit.element.is_some() {
        return hit;
    }
    let Some((element, pct)) = attacker
        .status
        .weapon_buff
        .and_then(|b| table.buff(b.num))
        .and_then(|d| d.element)
    else {
        return hit;
    };
    hit.element = Some(element_type(element));
    hit.ar.elem_milli[element_index(element)] = hit.ar.phys_milli * u64::from(pct) / 100;
    hit
}

/// The index of `e` in the Aether, Fire, Storm, Radiance arrays.
pub fn element_index(e: Element) -> usize {
    match e {
        Element::Aether => 0,
        Element::Fire => 1,
        Element::Storm => 2,
        Element::Radiance => 3,
    }
}

/// What `attacker`'s gear does to the damage `dmg` of `hit`: Ember Locket's Fire, the
/// Formula bonus (`formula` says the hit is a Formula), and Last Stand's. Only the
/// player has gear, so every other attacker's damage comes back unchanged.
pub fn outgoing(
    attacker: &Actor,
    gear: &GearTable,
    hit: &Hit,
    dmg: Damage,
    formula: bool,
) -> Damage {
    if !attacker.is_player {
        return dmg;
    }
    let eq = &attacker.inventory.equipment;
    let low = u128::from(gear.low_hp_damage_percent(eq, attacker.hp, attacker.max_hp));
    let mut elem_pct = 0u128;
    if let Some(e) = hit.element {
        let element = match e {
            lf_content::scenario::DamageType::Aether => Some(Element::Aether),
            lf_content::scenario::DamageType::Fire => Some(Element::Fire),
            lf_content::scenario::DamageType::Storm => Some(Element::Storm),
            lf_content::scenario::DamageType::Radiance => Some(Element::Radiance),
            _ => None,
        };
        if let Some(x) = element {
            elem_pct += u128::from(gear.element_damage_percent(eq, x));
        }
        if formula {
            elem_pct += u128::from(gear.formula_damage_percent(eq));
        }
    }
    if low == 0 && elem_pct == 0 {
        return dmg;
    }
    // Each bonus is a multiplier of its own: Last Stand on top of the element's.
    let phys = u128::from(dmg.phys_milli) * (100 + low) / 100;
    let elem = u128::from(dmg.elem_milli) * (100 + elem_pct) / 100 * (100 + low) / 100;
    let (phys, elem) = (phys as u64, elem as u64);
    Damage {
        total: (((phys + elem + 500) / 1000) as u32).max(1),
        phys_milli: phys,
        elem_milli: elem,
    }
}

// --- Procs and perception ---------------------------------------------------------

/// A status proc happened on `target`: a wearer of a [`GearEffect::ProcBuff`] charm for
/// that status, within its radius, gets the buff (Thornheart). Called from
/// `combat::status` when a meter fills.
pub fn on_status_proc(world: &mut World, target: crate::arena::Id, kind: StatusKind) {
    let player = world.player;
    let Some(at) = world.actors.get(target).map(|a| a.pos) else {
        return;
    };
    let Some(p) = world.actors.get(player).filter(|p| !p.dead) else {
        return;
    };
    let mut buffs: Vec<NumId> = Vec::new();
    for e in world.items.gear.worn_effects(&p.inventory.equipment) {
        if let GearEffect::ProcBuff {
            status,
            buff,
            radius_m,
        } = e
            && *status == kind
        {
            let d = [p.pos[0] - at[0], p.pos[1] - at[1], p.pos[2] - at[2]];
            let dist = crate::math::dmath::sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
            if dist <= *radius_m
                && let Some(n) = world.status.buff_num(buff)
            {
                buffs.push(n);
            }
        }
    }
    if let Some(p) = world.actors.get_mut(player) {
        for n in buffs {
            status::apply_buff(p, &world.status, n);
        }
    }
}

/// `perception` as it applies to `target`: the sight range scaled by what the target
/// wears (Boo, and endark in darkness). Darkness isn't built yet, so `dark` is false:
/// endark's effect waits for it (`ai.ron`, `dark_sight_m`).
pub fn scale_sight(
    gear: &GearTable,
    target: &Actor,
    perception: &lf_content::enemy::Perception,
) -> lf_content::enemy::Perception {
    let mut p = *perception;
    if target.is_player {
        let pct = gear.sight_percent(&target.inventory.equipment, false);
        p.sight_m = p.sight_m * pct as f32 / 100.0;
    }
    p
}

/// The player's defenses after a [`refresh`], as a fresh [`Defense`] (for tests and the
/// menu): the flat defense and absorption `refresh` writes.
pub fn defense_of(world: &World, inv: &Inventory) -> Defense {
    let armor = world.items.gear.armor_totals(&inv.equipment);
    let mut d = Defense::flat(world.locomotion.stub_defense);
    if armor.worn {
        for t in 0..TYPE_COUNT {
            d.flat[t] = if t < 4 {
                armor.defense_physical
            } else {
                armor.defense_elemental
            };
            d.absorb_percent[t] = armor.absorb[t];
        }
    }
    d
}
