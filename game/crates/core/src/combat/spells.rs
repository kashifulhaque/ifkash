//! Spells: catalysts, spell power, attunement, slots, cast speed, and the Formulae. See
//! docs/game/03-combat.md, "Spells", and docs/game/04-arsenal-and-builds.md, "Spells".
//!
//! # How a spell is built
//!
//! A spell is data (`data/spells/*.ron`, `lf_content::spells`) that carries its own cast
//! moves, projectiles, and buffs. The content loader merges those into the move,
//! projectile, and buff tables, so casting a spell is starting a move like any other: it
//! costs FP (`Move::fp`), launches projectiles on its frames (`combat::projectile`), or
//! lands hitboxes (`combat::resolve`). This module adds the parts that are particular to
//! spells:
//!
//! - **Catalysts.** A weapon of class `Catalyst` has a school (`catalyst_school`): a stylus
//!   casts Formulae, a reliquary Litanies. With a catalyst in the right hand and a spell of
//!   its school selected, `LIGHT` casts it and `HEAVY` casts its charged form (or the same
//!   cast, if it has none). With the catalyst in the left hand, `GUARD` casts
//!   ([`Action::Cast`]) and the catalyst can't guard. [`overlay`] writes the cast moves
//!   into the player's moveset; `World::sync_player_moveset` calls it.
//! - **Spell power.** The catalyst's attack rating (base damage and scaling from its
//!   data) replaces the weapon's AR in the damage formula for a spell's moves and
//!   projectiles ([`power`]). It is dealt as the element of the hitbox or projectile: the
//!   physical part is 0.
//! - **Attunement and slots.** The loadout is [`SpellState`], kept on the world (not in
//!   the inventory) and hashed only once a spell is attuned, so worlds that never attune
//!   hash as they did before spells existed. It is edited with the ordinary equipment
//!   commands: `Command::Equip { slot: slot::SPELL_1 + i, item: <spell> }` attunes a
//!   known spell (one in the player's gear) to position `i`, and `Command::Unequip` frees
//!   it ([`attune`], [`unattune`]). The slots a spell takes (1 or 2) add up to at most the
//!   Focus-derived `spell_slots` plus the gear's bonus (`GearTable::spell_slot_bonus`), and the spell's attribute requirements must be met.
//!   `CYCLE_SPELL` selects the next attuned spell ([`cycle`]).
//! - **Cast speed.** A cast starts on a later frame: Finesse skips up to a quarter of the
//!   frames before the first one that does anything (`MoveTable::cast_skip`).
//! - **Timed effects and channels.** [`run`] (tick system 6, after the move step) fires a
//!   spell's frame-timed heals, buffs, and cures, and drives a channeled spell: while the
//!   cast button holds the move, every few ticks it pays FP and launches a projectile.

use lf_content::moves::{ProjectileSpawn, StatusKind};
use lf_content::spells::{SpellEffect, SpellSchool};
use lf_content::{Content, NumId};
use serde::{Deserialize, Serialize};

use super::damage::AttackRating;
use super::moves::{MoveNum, Moveset, flags};
use super::{projectile, status};
use crate::actor::Actor;
use crate::arena::Id;
use crate::items::slot;
use crate::world::World;

/// Most spells the player can have attuned at once: the positions `slot::SPELL_1` and up.
/// Focus 99 gives 13 slots, and a spell takes at least one.
pub const MAX_ATTUNED: usize = slot::SPELL_COUNT as usize;

/// What a timed effect does, with its references resolved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    /// Heals the caster by `max_hp_percent` of its max HP, and each Echo within `radius_m`
    /// of the caster by the same share of its own (Wide Mend, WP-056).
    Heal {
        max_hp_percent: u8,
        radius_m: f32,
    },
    Buff(NumId),
    /// Cures these statuses; an empty list cures all of them.
    Cure(Vec<StatusKind>),
}

/// An effect on one frame of a cast move.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timed {
    pub move_num: MoveNum,
    pub frame: u16,
    pub effect: Effect,
}

/// A channeled spell's rules, for its cast move.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelRules {
    pub move_num: MoveNum,
    pub interval_ticks: u16,
    pub fp: u16,
    pub spawn: ProjectileSpawn,
}

/// One spell with its references resolved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spell {
    pub num: NumId,
    pub id: String,
    pub school: SpellSchool,
    pub slots: u8,
    /// Might, Finesse, Insight, Devotion.
    pub requirements: [u16; 4],
    /// The move `LIGHT` (or a left-hand `GUARD`) starts.
    pub cast: MoveNum,
    /// The move `HEAVY` starts, if the spell has a charged form.
    pub charged: Option<MoveNum>,
}

/// Every spell and catalyst in the content. Built from content, so it isn't hashed: the
/// content hash covers it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpellTable {
    /// Sorted by registry number.
    spells: Vec<Spell>,
    /// Sorted by move number.
    effects: Vec<Timed>,
    channels: Vec<ChannelRules>,
    /// Each catalyst weapon's school, sorted by weapon number.
    catalysts: Vec<(NumId, SpellSchool)>,
    /// Every spell move, sorted, with the index of its spell.
    by_move: Vec<(MoveNum, usize)>,
}

impl SpellTable {
    pub fn from_content(content: &Content) -> Result<Self, String> {
        let num = |id: &str| {
            content
                .registry
                .num(id)
                .ok_or_else(|| format!("\"{id}\" is missing from registry.ron"))
        };
        let mut t = Self::default();
        for sp in &content.spells {
            let cast = num(&sp.cast.id)?;
            let charged = sp.charged.as_ref().map(|m| num(&m.id)).transpose()?;
            t.spells.push(Spell {
                num: num(&sp.id)?,
                id: sp.id.clone(),
                school: sp.school,
                slots: sp.slots,
                requirements: [
                    sp.requirements.might,
                    sp.requirements.finesse,
                    sp.requirements.insight,
                    sp.requirements.devotion,
                ],
                cast,
                charged,
            });
            for (m, list) in [(Some(cast), &sp.effects), (charged, &sp.charged_effects)] {
                let Some(move_num) = m else { continue };
                for e in list {
                    let effect = match &e.effect {
                        SpellEffect::Heal {
                            max_hp_percent,
                            radius_m,
                        } => Effect::Heal {
                            max_hp_percent: *max_hp_percent,
                            radius_m: *radius_m,
                        },
                        SpellEffect::Buff { buff } => Effect::Buff(num(buff)?),
                        SpellEffect::Cure { statuses } => Effect::Cure(statuses.clone()),
                    };
                    t.effects.push(Timed {
                        move_num,
                        frame: e.frame,
                        effect,
                    });
                }
            }
            if let Some(c) = &sp.channel {
                t.channels.push(ChannelRules {
                    move_num: cast,
                    interval_ticks: c.interval_ticks,
                    fp: c.fp,
                    spawn: ProjectileSpawn {
                        frame: 0,
                        projectile: c.projectile.clone(),
                        offset: c.offset,
                        yaw_deg: c.yaw_deg,
                        pitch_deg: c.pitch_deg,
                    },
                });
            }
        }
        t.spells.sort_by_key(|s| s.num);
        t.effects.sort_by_key(|e| (e.move_num, e.frame));
        t.channels.sort_by_key(|c| c.move_num);
        for (i, s) in t.spells.iter().enumerate() {
            t.by_move.push((s.cast, i));
            if let Some(c) = s.charged {
                t.by_move.push((c, i));
            }
        }
        t.by_move.sort_unstable();
        for (_, w) in content.all_weapons() {
            if let Some(school) = w.catalyst_school {
                t.catalysts.push((num(&w.id)?, school));
            }
        }
        t.catalysts.sort_unstable_by_key(|c| c.0);
        Ok(t)
    }

    pub fn get(&self, num: NumId) -> Option<&Spell> {
        self.spells
            .binary_search_by_key(&num, |s| s.num)
            .ok()
            .map(|i| &self.spells[i])
    }

    /// Finds a spell's number by string ID. A linear scan, for tools and tests.
    pub fn num_of(&self, id: &str) -> Option<NumId> {
        self.spells.iter().find(|s| s.id == id).map(|s| s.num)
    }

    /// The spell whose cast or charged move is `m`.
    pub fn of_move(&self, m: MoveNum) -> Option<&Spell> {
        self.by_move
            .binary_search_by_key(&m, |e| e.0)
            .ok()
            .map(|i| &self.spells[self.by_move[i].1])
    }

    /// The school the catalyst weapon `w` casts; `None` for any other weapon.
    pub fn catalyst(&self, w: NumId) -> Option<SpellSchool> {
        self.catalysts
            .binary_search_by_key(&w, |c| c.0)
            .ok()
            .map(|i| self.catalysts[i].1)
    }

    fn effects_of(&self, m: MoveNum) -> &[Timed] {
        let lo = self.effects.partition_point(|e| e.move_num < m);
        let hi = self.effects.partition_point(|e| e.move_num <= m);
        &self.effects[lo..hi]
    }

    fn channel_of(&self, m: MoveNum) -> Option<&ChannelRules> {
        self.channels
            .binary_search_by_key(&m, |c| c.move_num)
            .ok()
            .map(|i| &self.channels[i])
    }
}

/// The player's attuned spells and the selected one. Simulation state: part of the state
/// hash, but only once something is attuned (see `World::state_hash`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellState {
    /// The spell at each attunement position (`slot::SPELL_1 + i`).
    pub attuned: [Option<NumId>; MAX_ATTUNED],
    /// The selected position, which `CYCLE_SPELL` moves.
    pub selected: u8,
}

impl SpellState {
    /// Nothing attuned and nothing selected: the state of a world that never used spells.
    pub fn is_inert(&self) -> bool {
        self.selected == 0 && self.attuned.iter().all(Option::is_none)
    }

    /// The selected spell's number.
    pub fn selected_spell(&self) -> Option<NumId> {
        self.attuned
            .get(usize::from(self.selected))
            .copied()
            .flatten()
    }

    /// Slots the attuned spells take, leaving out position `skip`.
    fn used(&self, table: &SpellTable, skip: usize) -> u16 {
        self.attuned
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != skip)
            .filter_map(|(_, s)| s.and_then(|n| table.get(n)))
            .map(|s| u16::from(s.slots))
            .sum()
    }

    /// After a position empties: moves the selection onto a filled position.
    fn settle(&mut self) {
        if self.selected_spell().is_none()
            && let Some(i) = self.attuned.iter().position(Option::is_some)
        {
            self.selected = i as u8;
        }
    }
}

/// True if `attrs` meet every attribute requirement of `s`.
fn meets(s: &Spell, a: &Actor) -> bool {
    let at = &a.stats.attrs;
    [at.might, at.finesse, at.insight, at.devotion]
        .iter()
        .zip(&s.requirements)
        .all(|(have, need)| have >= need)
}

/// Attunes the known spell `spell` to position `index` (`slot::SPELL_1 + index`),
/// replacing the spell there. False, changing nothing, if the position doesn't exist, the
/// player doesn't know the spell (it isn't in their gear), its requirements aren't met, it
/// is attuned elsewhere, or the spells wouldn't fit in the Focus-derived slots.
pub fn attune(world: &mut World, index: u8, spell: NumId) -> bool {
    let i = usize::from(index);
    let Some(s) = world.spells_table.get(spell) else {
        return false;
    };
    let Some(p) = world.actors.get(world.player) else {
        return false;
    };
    let known = p.inventory.gear.binary_search(&spell).is_ok();
    let derived = p.stats.attrs.derived(&world.attributes);
    let st = &world.spells;
    // Gear adds slots on top of Focus's (Scholar's Seal, WP-057).
    let bonus = world.items.gear.spell_slot_bonus(&p.inventory.equipment);
    let capacity = u32::from(derived.spell_slots) + bonus;
    let fits = u32::from(st.used(&world.spells_table, i)) + u32::from(s.slots) <= capacity;
    let elsewhere = st
        .attuned
        .iter()
        .enumerate()
        .any(|(j, a)| j != i && *a == Some(spell));
    if i >= MAX_ATTUNED || !known || !meets(s, p) || elsewhere || !fits {
        return false;
    }
    world.spells.attuned[i] = Some(spell);
    world.spells.settle();
    world.sync_player_moveset();
    true
}

/// Frees position `index`. False if it is out of range or already empty.
pub fn unattune(world: &mut World, index: u8) -> bool {
    let i = usize::from(index);
    if i >= MAX_ATTUNED || world.spells.attuned[i].is_none() {
        return false;
    }
    world.spells.attuned[i] = None;
    world.spells.settle();
    world.sync_player_moveset();
    true
}

/// Selects the next attuned spell, wrapping (`CYCLE_SPELL`).
pub fn cycle(world: &mut World) {
    let st = &mut world.spells;
    let n = MAX_ATTUNED;
    let next = (1..=n)
        .map(|step| (usize::from(st.selected) + step) % n)
        .find(|&i| st.attuned[i].is_some());
    if let Some(i) = next
        && i as u8 != st.selected
    {
        st.selected = i as u8;
        world.sync_player_moveset();
    }
}

/// Writes the selected spell's cast moves into the player's moveset `set`. `LIGHT` and
/// `HEAVY` cast it from a right-hand catalyst of its school; `left_cast` is set for a
/// left-hand one. With no matching catalyst or spell the moveset is the weapon's own.
pub fn overlay(world: &World, set: &mut Moveset) {
    set.left_cast = None;
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    let Some(spell) = world
        .spells
        .selected_spell()
        .and_then(|n| world.spells_table.get(n))
    else {
        return;
    };
    let eq = &p.inventory.equipment;
    let school = |w: Option<NumId>| w.and_then(|w| world.spells_table.catalyst(w));
    if school(eq.right_weapon()) == Some(spell.school) {
        let heavy = spell.charged.unwrap_or(spell.cast);
        set.light_chain = vec![spell.cast];
        set.heavy = heavy;
        set.charged_heavy = heavy;
    } else if school(eq.left_weapon()) == Some(spell.school) {
        set.left_cast = Some(spell.cast);
    }
}

/// The attack rating of a spell cast by `a`, when `move_num` is a spell move: the
/// catalyst's spell power, in all four elements and no physical damage. `None` for any
/// other move, which keeps the weapon's attack rating.
pub fn power(world: &World, a: &Actor, move_num: MoveNum) -> Option<AttackRating> {
    world.spells_table.of_move(move_num)?;
    let eq = &a.inventory.equipment;
    let catalyst = [eq.right_weapon(), eq.left_weapon()]
        .into_iter()
        .flatten()
        .find(|&w| world.spells_table.catalyst(w).is_some())?;
    let ar = world.damage.attack_rating(
        catalyst,
        &a.stats.attrs,
        a.stats.weapon_upgrade,
        // A catalyst is held in one hand (two-handing raises Might only).
        false,
    )?;
    Some(AttackRating {
        phys_milli: 0,
        elem_milli: [ar.phys_milli; 4],
        unwieldy: ar.unwieldy,
    })
}

/// What a tick's spell step does, decided from the unchanged world.
enum Pending {
    /// A heal: the share of max HP, and the radius that also reaches Echoes.
    Heal(u8, f32),
    Buff(NumId),
    Cure(Vec<StatusKind>),
    /// A channel interval: pay `fp` and launch the spawn.
    Channel(u16, ProjectileSpawn),
}

/// Heals `id` by `percent` of its max HP, up to its max.
fn heal_share(world: &mut World, id: Id, percent: u8) {
    if let Some(a) = world.actors.get_mut(id) {
        let heal = (u64::from(a.max_hp) * u64::from(percent) / 100) as u32;
        a.hp = a.hp.saturating_add(heal).min(a.max_hp);
    }
}

/// Tick system 6, before projectiles launch: fires the player's spell effects for the frame
/// executed this tick, and drives a channel.
pub fn run(world: &mut World) {
    if world.spells_table.spells.is_empty() {
        return;
    }
    let id = world.player;
    let Some(a) = world.actors.get(id).filter(|a| !a.dead) else {
        return;
    };
    let Some(c) = a.moves.current else {
        return;
    };
    let table = &world.spells_table;
    let max_hold = world
        .moves
        .get(c.num)
        .and_then(|d| d.data.charge)
        .map_or(0, |ch| ch.max_frames);
    let mut pending = Vec::new();
    if a.moves.has(flags::CHARGING) {
        if let Some(ch) = table.channel_of(c.num)
            && c.charge_ticks > 0
            && (c.charge_ticks - 1) % ch.interval_ticks == 0
        {
            pending.push(Pending::Channel(ch.fp, ch.spawn.clone()));
        }
    } else {
        for e in table
            .effects_of(c.num)
            .iter()
            .filter(|e| e.frame == c.frame)
        {
            pending.push(match &e.effect {
                Effect::Heal {
                    max_hp_percent,
                    radius_m,
                } => Pending::Heal(*max_hp_percent, *radius_m),
                Effect::Buff(b) => Pending::Buff(*b),
                Effect::Cure(k) => Pending::Cure(k.clone()),
            });
        }
    }
    for p in pending {
        match p {
            Pending::Heal(percent, radius) => {
                heal_share(world, id, percent);
                // Wide Mend: the living Echoes within the radius heal by the same share.
                let Some(center) = world.actors.get(id).map(|a| a.pos) else {
                    continue;
                };
                let r2 = radius * radius;
                let echoes: Vec<Id> = world
                    .actors
                    .iter()
                    .filter(|(aid, a)| {
                        *aid != id && a.is_ally() && !a.dead && {
                            let (dx, dz) = (a.pos[0] - center[0], a.pos[2] - center[2]);
                            dx * dx + dz * dz <= r2
                        }
                    })
                    .map(|(aid, _)| aid)
                    .collect();
                if radius > 0.0 {
                    for aid in echoes {
                        heal_share(world, aid, percent);
                    }
                }
            }
            Pending::Buff(b) => {
                if let Some(a) = world.actors.get_mut(id) {
                    status::apply_buff(a, &world.status, b);
                }
            }
            Pending::Cure(kinds) => {
                if let Some(a) = world.actors.get_mut(id) {
                    if kinds.is_empty() {
                        status::cure_all(a);
                    }
                    for k in kinds {
                        status::cure(a, k);
                    }
                }
            }
            Pending::Channel(fp, spawn) => {
                let cost = f32::from(fp);
                let Some(a) = world.actors.get_mut(id) else {
                    continue;
                };
                if a.fp >= cost {
                    a.fp -= cost;
                    projectile::spawn(world, id, c.num, &spawn);
                } else if let Some(m) = a.moves.current.as_mut() {
                    // Out of FP: the channel ends, and the move plays out its recovery.
                    m.charge_ticks = max_hold;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
