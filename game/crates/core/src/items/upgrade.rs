//! Weapon upgrades and tempering at the Smith (docs/game/04-arsenal-and-builds.md, "Temper"
//! and "Upgrades"). The numbers live in `upgrades.ron`; the per-weapon state lives in the
//! world's flags, so a game that never visits the Smith hashes exactly as before.
//!
//! # State
//!
//! - `weapon.level.<num>`: the steps a weapon has been upgraded, by registry number. A common
//!   weapon's steps are its level (0 to 10). A unique or relic weapon's are its Starglass
//!   level (0 to 5), which counts as two common levels.
//! - `weapon.temper.<num>`: the temper's index in `upgrades.ron` plus 1, or absent for none.
//!
//! # Where it plugs in
//!
//! [`weapon_rating`] is the attack rating of a weapon in someone's hands, with its level and
//! temper applied. The combat pipeline calls it instead of `DamageTable::attack_rating`.
//! [`upgrade_weapon`] and [`temper_weapon`] are the Smith's commands (`Command::Upgrade` and
//! `Command::Temper`, applied in `world_state::lantern_menu`). Both apply only while resting at
//! a Lantern, since there is no Smith to talk to yet.

use lf_content::{NumId, Temper};

use super::slot;
use crate::actor::stats::Attrs;
use crate::combat::damage::{self, AttackRating, WeaponStats};
use crate::world::World;

/// The flag prefix for a weapon's upgrade steps.
const LEVEL_PREFIX: &str = "weapon.level.";
/// The flag prefix for a weapon's temper (index plus 1; 0 or absent is none).
const TEMPER_PREFIX: &str = "weapon.temper.";

/// Why the Smith refused a command. A refusal changes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmithError {
    /// Not resting at a Lantern, or the player is dead.
    NotResting,
    /// The equipment slot isn't a hand slot, or holds no weapon.
    NoWeapon,
    /// The content has no upgrade rules (`upgrades.ron` is missing).
    NoRules,
    /// The weapon is at its highest level.
    Maxed,
    /// The level needs the Forgeheart Ember, and the player doesn't hold it.
    NeedsEmber,
    /// Not enough Glimmer for the next level.
    NotEnoughGlimmer,
    /// Not enough of the level's material (Forgeglass, or Starglass).
    NotEnoughMaterials,
    /// Only common weapons take a temper; unique and relic weapons don't.
    NotCommon,
    /// The weapon already has a temper, and takes one only.
    AlreadyTempered,
    /// No temper has that index.
    UnknownTemper,
    /// The temper converts base damage into an element the weapon already has a different
    /// element for. A weapon has one elemental part.
    ElementClash,
    /// No Temper Whetstone to spend.
    NoWhetstone,
}

/// What the next upgrade of a weapon costs and gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpgradeCost {
    /// The effective common level the upgrade reaches (a Starglass level counts as two).
    pub level: u8,
    pub glimmer: u32,
    /// The material's registry number, and how many.
    pub item: NumId,
    pub count: u16,
    /// Whether the level needs the Forgeheart Ember held.
    pub needs_ember: bool,
}

/// `base * level^1.5`, rounded down, in integers: the square root of `base^2 * level^3`.
pub fn glimmer_cost(base: u32, level: u8) -> u32 {
    let l = u128::from(level);
    let b = u128::from(base);
    isqrt(b * b * l * l * l) as u32
}

/// The integer square root: the largest `x` with `x * x <= n`.
fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// The steps a weapon has been upgraded (its raw level, or Starglass level).
fn steps(world: &World, num: NumId) -> u8 {
    let v = world.state.flags.get(&format!("{LEVEL_PREFIX}{num}"));
    u8::try_from(v).unwrap_or(u8::MAX)
}

/// The temper's index in `upgrades.ron`, if the weapon has one.
fn temper_index(world: &World, num: NumId) -> Option<usize> {
    match world.state.flags.get(&format!("{TEMPER_PREFIX}{num}")) {
        0 => None,
        v => Some((v - 1) as usize),
    }
}

/// The level the weapon's attack rating uses: its steps for a common weapon, and two per
/// Starglass level for a unique one. Zero with no upgrade rules.
pub fn effective_level(world: &World, num: NumId) -> u8 {
    let s = steps(world, num);
    let unique = world.damage.weapon(num).is_some_and(|w| w.unique);
    match (&world.upgrades, unique) {
        (Some(r), true) => s.saturating_mul(r.starglass_common_levels),
        _ => s,
    }
}

/// The temper of weapon `num`, if it has one.
pub fn temper_of(world: &World, num: NumId) -> Option<&Temper> {
    let i = temper_index(world, num)?;
    world.upgrades.as_ref()?.tempers.get(i)
}

/// The stats a temper gives a weapon (docs/game/04-arsenal-and-builds.md, "Temper"). Each
/// scaling is multiplied by the temper's percent, then its addition is added. Part of the
/// base damage moves into the temper's element, with the weapon's own element kept.
pub fn tempered_stats(base: &WeaponStats, t: &Temper) -> Result<WeaponStats, SmithError> {
    let mut out = base.clone();
    for i in 0..4 {
        out.scaling_permille[i] = base.scaling_permille[i] * u32::from(t.scaling_percent[i]) / 100
            + u32::from(t.scaling_add_permille[i]);
    }
    if let Some(element) = t.element {
        let moved = base.base_damage * u32::from(t.element_percent) / 100;
        let have = match base.elem {
            Some((e, b)) if e == element => b,
            Some(_) => return Err(SmithError::ElementClash),
            None => 0,
        };
        out.base_damage = base.base_damage - moved;
        out.elem = Some((element, have + moved));
    }
    Ok(out)
}

/// The attack rating of weapon `num` in the hands of someone with `attrs`, with its level
/// and temper applied. `None` if the weapon isn't in the content.
pub fn weapon_rating(
    world: &World,
    num: NumId,
    attrs: &Attrs,
    two_handing: bool,
) -> Option<AttackRating> {
    let base = world.damage.weapon(num)?;
    let tempered;
    let stats = match temper_of(world, num) {
        Some(t) => {
            tempered = tempered_stats(base, t).unwrap_or_else(|_| base.clone());
            &tempered
        }
        None => base,
    };
    Some(damage::attack_rating(
        &world.damage.rules,
        world.damage.attr_max,
        stats,
        attrs,
        effective_level(world, num),
        two_handing,
    ))
}

/// The attack rating of an empty right hand in `attrs`' hands (`DamageRules::unarmed`), or
/// `None` if the content gives fists none. Issue #76.
pub fn unarmed_rating(world: &World, attrs: &Attrs) -> Option<AttackRating> {
    let stats = world.damage.unarmed()?;
    Some(damage::attack_rating(
        &world.damage.rules,
        world.damage.attr_max,
        &stats,
        attrs,
        0,
        false,
    ))
}

/// The player, if the Smith can work now: resting at a Lantern, and alive.
fn smith_player(world: &World) -> Result<&crate::actor::Actor, SmithError> {
    if world.state.resting.is_none() {
        return Err(SmithError::NotResting);
    }
    let p = world
        .actors
        .get(world.player)
        .ok_or(SmithError::NotResting)?;
    if p.dead {
        return Err(SmithError::NotResting);
    }
    Ok(p)
}

/// The weapon in hand slot `slot` of the player.
fn weapon_in_slot(world: &World, slot_index: u8) -> Result<NumId, SmithError> {
    let p = smith_player(world)?;
    if slot_index > slot::LEFT_2 {
        return Err(SmithError::NoWeapon);
    }
    let num = p.inventory.equipment.slots[usize::from(slot_index)].ok_or(SmithError::NoWeapon)?;
    if world.damage.weapon(num).is_none() {
        return Err(SmithError::NoWeapon);
    }
    Ok(num)
}

/// What the next upgrade of weapon `num` costs, or why it can't be bought.
pub fn upgrade_cost(world: &World, num: NumId) -> Result<UpgradeCost, SmithError> {
    let rules = world.upgrades.as_ref().ok_or(SmithError::NoRules)?;
    let p = smith_player(world)?;
    let stats = world.damage.weapon(num).ok_or(SmithError::NoWeapon)?;
    let next = steps(world, num).saturating_add(1);
    let (max_steps, level) = if stats.unique {
        (
            rules.starglass_max,
            next.saturating_mul(rules.starglass_common_levels),
        )
    } else {
        (world.damage.rules.max_upgrade, next)
    };
    if next > max_steps {
        return Err(SmithError::Maxed);
    }
    let needs_ember = level > rules.ember_above;
    if needs_ember {
        let held = world
            .items
            .num_of(&rules.ember_item)
            .is_some_and(|e| p.inventory.count(e) > 0);
        if !held {
            return Err(SmithError::NeedsEmber);
        }
    }
    let glimmer = glimmer_cost(rules.glimmer_base, level);
    let (item_id, count) = if stats.unique {
        (rules.starglass_item.as_str(), 1)
    } else {
        let band = rules
            .common_costs
            .iter()
            .find(|c| level <= c.through)
            .ok_or(SmithError::NoRules)?;
        (band.item.as_str(), band.count)
    };
    let item = world
        .items
        .num_of(item_id)
        .ok_or(SmithError::NotEnoughMaterials)?;
    Ok(UpgradeCost {
        level,
        glimmer,
        item,
        count,
        needs_ember,
    })
}

/// The Smith's `Upgrade` command: raises the weapon in hand slot `slot` by one step, paying
/// its Glimmer and material. Returns the new steps.
pub fn upgrade_weapon(world: &mut World, slot_index: u8) -> Result<u8, SmithError> {
    let num = weapon_in_slot(world, slot_index)?;
    let cost = upgrade_cost(world, num)?;
    let player = world.player;
    let p = world.actors.get_mut(player).ok_or(SmithError::NotResting)?;
    if p.inventory.glimmer < cost.glimmer {
        return Err(SmithError::NotEnoughGlimmer);
    }
    if p.inventory.count(cost.item) < cost.count {
        return Err(SmithError::NotEnoughMaterials);
    }
    p.inventory.glimmer -= cost.glimmer;
    p.inventory.remove(cost.item, cost.count);
    let next = steps(world, num) + 1;
    world
        .state
        .flags
        .set(&format!("{LEVEL_PREFIX}{num}"), u32::from(next));
    Ok(next)
}

/// The Smith's `Temper` command: tempers the common weapon in hand slot `slot` with the
/// temper at index `temper` in `upgrades.ron`, spending a Temper Whetstone.
pub fn temper_weapon(world: &mut World, slot_index: u8, temper: u8) -> Result<(), SmithError> {
    let num = weapon_in_slot(world, slot_index)?;
    let rules = world.upgrades.as_ref().ok_or(SmithError::NoRules)?;
    let stats = world.damage.weapon(num).ok_or(SmithError::NoWeapon)?;
    if stats.unique {
        return Err(SmithError::NotCommon);
    }
    if temper_index(world, num).is_some() {
        return Err(SmithError::AlreadyTempered);
    }
    let t = rules
        .tempers
        .get(usize::from(temper))
        .ok_or(SmithError::UnknownTemper)?;
    tempered_stats(stats, t)?;
    let whetstone = world
        .items
        .num_of(&rules.whetstone_item)
        .ok_or(SmithError::NoWhetstone)?;
    let player = world.player;
    let p = world.actors.get_mut(player).ok_or(SmithError::NotResting)?;
    if p.inventory.count(whetstone) == 0 {
        return Err(SmithError::NoWhetstone);
    }
    p.inventory.remove(whetstone, 1);
    world
        .state
        .flags
        .set(&format!("{TEMPER_PREFIX}{num}"), u32::from(temper) + 1);
    Ok(())
}

#[cfg(test)]
mod tests {
    use lf_content::Content;

    use super::*;
    use crate::input::{ActionFrame, Command, TickInput};
    use crate::tick::tick;

    fn field() -> World {
        let content = Content::load_embedded().expect("embedded content");
        World::new(9, &content, "test_field").expect("test_field exists")
    }

    /// The registry number of a content ID. Every weapon, item, and Art is registered.
    fn num(id: &str) -> NumId {
        Content::load_embedded()
            .expect("embedded content")
            .registry
            .num(id)
            .unwrap_or_else(|| panic!("{id} has a registry number"))
    }

    /// Puts `weapon` in the right-hand slot, held in the gear list.
    fn hold(w: &mut World, weapon: NumId) {
        let player = w.player;
        let items = &w.items;
        let p = w.actors.get_mut(player).unwrap();
        p.inventory.give_gear(weapon);
        assert!(p.inventory.equip(slot::RIGHT_1, weapon, items));
    }

    /// Gives the player `n` of `item`.
    fn give(w: &mut World, item: &str, n: u16) {
        let id = w.items.num_of(item).expect("item exists");
        let player = w.player;
        let items = &w.items;
        w.actors
            .get_mut(player)
            .unwrap()
            .inventory
            .pick_up(id, n, items);
    }

    fn step(w: &mut World, commands: Vec<Command>) {
        tick(
            w,
            &TickInput {
                frame: ActionFrame::default(),
                commands,
            },
        );
    }

    /// Rests at the test field's Lantern, which is 3 m behind the spawn.
    fn rest(w: &mut World) {
        let id = w.player;
        w.actors.get_mut(id).unwrap().pos = [0.0, 0.0, -2.0];
        step(w, vec![Command::Rest]);
        assert!(w.state.resting.is_some());
    }

    fn glimmer(w: &mut World, n: u32) {
        let player = w.player;
        w.actors.get_mut(player).unwrap().inventory.glimmer = n;
    }

    #[test]
    fn glimmer_cost_follows_200_times_level_to_the_1_5() {
        assert_eq!(glimmer_cost(200, 1), 200);
        assert_eq!(glimmer_cost(200, 2), 565); // 200 * 2^1.5 = 565.68
        assert_eq!(glimmer_cost(200, 4), 1600);
        assert_eq!(glimmer_cost(200, 9), 5400);
        assert_eq!(glimmer_cost(200, 10), 6324); // 200 * 10^1.5 = 6324.55
        assert_eq!(glimmer_cost(0, 10), 0);
    }

    #[test]
    fn forgeglass_bands_cost_the_spec_counts() {
        let mut w = field();
        rest(&mut w);
        let wayfarer = num("wpn_wayfarer_blade");
        hold(&mut w, wayfarer);
        let shard = num("itm_forgeglass_shard");
        let chunk = num("itm_forgeglass_chunk");
        let cost = |w: &World| upgrade_cost(w, wayfarer).unwrap();
        // Level 1 costs one shard; the shard counts step up through +5.
        let c = cost(&w);
        assert_eq!((c.level, c.item, c.count), (1, shard, 1));
        assert_eq!(c.glimmer, 200);
        give(&mut w, "itm_forgeglass_shard", 20);
        glimmer(&mut w, 100_000);
        // The level reached is 1, 2, 3, 4, 5, so the bands cost 1, 1, 2, 2, 3 shards.
        for expected in [1u16, 1, 2, 2, 3] {
            let before = upgrade_cost(&w, wayfarer).unwrap().count;
            assert_eq!(before, expected);
            upgrade_weapon(&mut w, slot::RIGHT_1).unwrap();
        }
        // Level 6 onward costs chunks, which the player doesn't have yet, and needs the
        // Ember, which they don't have either.
        let c = upgrade_cost(&w, wayfarer);
        assert_eq!(c, Err(SmithError::NeedsEmber));
        give(&mut w, "itm_forgeheart_ember", 1);
        let c = upgrade_cost(&w, wayfarer).unwrap();
        assert_eq!((c.level, c.item, c.count), (6, chunk, 1));
    }

    #[test]
    fn upgrade_spends_glimmer_and_material_and_raises_the_level() {
        let mut w = field();
        rest(&mut w);
        let wayfarer = num("wpn_wayfarer_blade");
        hold(&mut w, wayfarer);
        give(&mut w, "itm_forgeglass_shard", 3);
        glimmer(&mut w, 1000);
        let shard = num("itm_forgeglass_shard");
        assert_eq!(upgrade_weapon(&mut w, slot::RIGHT_1), Ok(1));
        let player = w.player;
        let p = w.actors.get(player).unwrap();
        assert_eq!(p.inventory.glimmer, 800);
        assert_eq!(p.inventory.count(shard), 2);
        assert_eq!(effective_level(&w, wayfarer), 1);
    }

    #[test]
    fn upgrade_damage_curve_follows_the_formula() {
        // Base 100 with Might 1.0 at Might 60 (the curve reads 1.0): AR = base(U) * (1 +
        // k(U)), base(U) = 100 * (1 + 0.06 U), k(U) = 1.0 * (1 + 0.03 U).
        let w = field();
        let num = num("wpn_wayfarer_blade");
        let attrs = Attrs {
            might: 60,
            ..Attrs::default()
        };
        // Use a 100-base weapon with Might 1.0 and no requirements, so only the Might term
        // is live: AR = 100 * (1 + 0.06 U) * (1 + (1 + 0.03 U) * 1.0).
        let base = w.damage.weapon(num).unwrap().clone();
        let ar_at = |w: &World, u: u8| {
            let mut stats = base.clone();
            stats.base_damage = 100;
            stats.scaling_permille = [1000, 0, 0, 0];
            stats.requirements = [0; 4];
            damage::attack_rating(&w.damage.rules, w.damage.attr_max, &stats, &attrs, u, false)
                .phys_milli
        };
        // U = 0: 100 * 1 * 2.0 = 200.
        assert_eq!(ar_at(&w, 0), 200_000);
        // U = 5: 100 * 1.3 * 2.15 = 279.5.
        assert_eq!(ar_at(&w, 5), 279_500);
        // U = 10: 100 * 1.6 * 2.3 = 368.
        assert_eq!(ar_at(&w, 10), 368_000);
    }

    #[test]
    fn unique_weapons_take_two_common_levels_per_starglass() {
        let mut w = field();
        rest(&mut w);
        let ferray = num("wpn_ferray");
        hold(&mut w, ferray);
        give(&mut w, "itm_starglass", 2);
        glimmer(&mut w, 100_000);
        assert_eq!(upgrade_weapon(&mut w, slot::RIGHT_1), Ok(1));
        assert_eq!(effective_level(&w, ferray), 2);
        // Starglass 2 reaches effective level 4, which needs no Ember. Starglass 3 reaches 6.
        assert_eq!(upgrade_weapon(&mut w, slot::RIGHT_1), Ok(2));
        assert_eq!(effective_level(&w, ferray), 4);
        give(&mut w, "itm_starglass", 1);
        assert_eq!(upgrade_cost(&w, ferray), Err(SmithError::NeedsEmber));
    }

    #[test]
    fn heavy_temper_scales_might_up_and_removes_finesse() {
        let w = field();
        let base = w.damage.weapon(num("wpn_wayfarer_blade")).unwrap().clone();
        let heavy = &w.upgrades.as_ref().unwrap().tempers[0];
        assert_eq!(heavy.id, "heavy");
        let t = tempered_stats(&base, heavy).unwrap();
        assert_eq!(t.scaling_permille[0], base.scaling_permille[0] * 3 / 2);
        assert_eq!(t.scaling_permille[1], 0);
        assert_eq!(t.base_damage, base.base_damage);
        assert_eq!(t.elem, None);
    }

    #[test]
    fn keen_temper_scales_finesse_up_and_might_down() {
        let w = field();
        let base = w.damage.weapon(num("wpn_wayfarer_blade")).unwrap().clone();
        let keen = &w.upgrades.as_ref().unwrap().tempers[1];
        let t = tempered_stats(&base, keen).unwrap();
        assert_eq!(t.scaling_permille[0], base.scaling_permille[0] / 2);
        assert_eq!(t.scaling_permille[1], base.scaling_permille[1] * 3 / 2);
    }

    #[test]
    fn lucid_temper_moves_40_percent_into_aether_and_adds_insight() {
        let w = field();
        let base = w.damage.weapon(num("wpn_wayfarer_blade")).unwrap().clone();
        let lucid = &w.upgrades.as_ref().unwrap().tempers[2];
        assert_eq!(lucid.id, "lucid");
        let t = tempered_stats(&base, lucid).unwrap();
        assert_eq!(t.base_damage, 105 - 42);
        assert_eq!(t.elem, Some((lf_content::moves::Element::Aether, 42)));
        assert_eq!(t.scaling_permille[2], 700);
    }

    #[test]
    fn a_temper_refuses_a_clashing_element() {
        let w = field();
        let claymore = w
            .damage
            .weapon(num("wpn_rimeforged_claymore"))
            .unwrap()
            .clone();
        // The claymore is Aether already; the Sacred temper would add Radiance.
        let sacred = &w.upgrades.as_ref().unwrap().tempers[3];
        assert_eq!(sacred.id, "sacred");
        assert_eq!(
            tempered_stats(&claymore, sacred),
            Err(SmithError::ElementClash)
        );
        // The Lucid temper adds Aether, which the claymore already has, so it stacks.
        let lucid = &w.upgrades.as_ref().unwrap().tempers[2];
        let t = tempered_stats(&claymore, lucid).unwrap();
        assert_eq!(t.elem, Some((lf_content::moves::Element::Aether, 40 + 48)));
    }

    #[test]
    fn smith_refuses_when_not_resting() {
        let mut w = field();
        let wayfarer = num("wpn_wayfarer_blade");
        hold(&mut w, wayfarer);
        give(&mut w, "itm_forgeglass_shard", 3);
        glimmer(&mut w, 1000);
        assert_eq!(
            upgrade_weapon(&mut w, slot::RIGHT_1),
            Err(SmithError::NotResting)
        );
        assert_eq!(
            temper_weapon(&mut w, slot::RIGHT_1, 0),
            Err(SmithError::NotResting)
        );
        assert_eq!(effective_level(&w, wayfarer), 0);
        // The Command does nothing outside a Lantern either.
        step(
            &mut w,
            vec![Command::Upgrade {
                slot: slot::RIGHT_1,
            }],
        );
        assert_eq!(effective_level(&w, wayfarer), 0);
    }

    #[test]
    fn smith_refuses_an_empty_or_non_hand_slot() {
        let mut w = field();
        rest(&mut w);
        assert_eq!(
            upgrade_weapon(&mut w, slot::HEAD),
            Err(SmithError::NoWeapon)
        );
        let player = w.player;
        let empty = slot::LEFT_1;
        w.actors.get_mut(player).unwrap().inventory.equipment.slots[usize::from(empty)] = None;
        assert_eq!(upgrade_weapon(&mut w, empty), Err(SmithError::NoWeapon));
    }

    #[test]
    fn smith_refuses_without_glimmer_or_material() {
        let mut w = field();
        rest(&mut w);
        let wayfarer = num("wpn_wayfarer_blade");
        hold(&mut w, wayfarer);
        glimmer(&mut w, 199);
        give(&mut w, "itm_forgeglass_shard", 1);
        assert_eq!(
            upgrade_weapon(&mut w, slot::RIGHT_1),
            Err(SmithError::NotEnoughGlimmer)
        );
        glimmer(&mut w, 1000);
        // Level 2 costs one more shard, and the player has none left after level 1.
        assert_eq!(upgrade_weapon(&mut w, slot::RIGHT_1), Ok(1));
        assert_eq!(
            upgrade_weapon(&mut w, slot::RIGHT_1),
            Err(SmithError::NotEnoughMaterials)
        );
        assert_eq!(effective_level(&w, wayfarer), 1);
    }

    #[test]
    fn smith_refuses_a_level_above_five_without_the_ember() {
        let mut w = field();
        rest(&mut w);
        let wayfarer = num("wpn_wayfarer_blade");
        hold(&mut w, wayfarer);
        give(&mut w, "itm_forgeglass_shard", 20);
        glimmer(&mut w, 100_000);
        for _ in 0..5 {
            upgrade_weapon(&mut w, slot::RIGHT_1).unwrap();
        }
        assert_eq!(
            upgrade_weapon(&mut w, slot::RIGHT_1),
            Err(SmithError::NeedsEmber)
        );
    }

    #[test]
    fn smith_refuses_a_second_temper_and_tempering_a_unique_weapon() {
        let mut w = field();
        rest(&mut w);
        let wayfarer = num("wpn_wayfarer_blade");
        let ferray = num("wpn_ferray");
        hold(&mut w, wayfarer);
        give(&mut w, "itm_temper_whetstone", 2);
        assert_eq!(temper_weapon(&mut w, slot::RIGHT_1, 0), Ok(()));
        assert_eq!(
            temper_weapon(&mut w, slot::RIGHT_1, 1),
            Err(SmithError::AlreadyTempered)
        );
        // A unique weapon can't be tempered at all.
        let player = w.player;
        let items = &w.items;
        let p = w.actors.get_mut(player).unwrap();
        p.inventory.give_gear(ferray);
        assert!(p.inventory.equip(slot::RIGHT_1, ferray, items));
        assert_eq!(
            temper_weapon(&mut w, slot::RIGHT_1, 1),
            Err(SmithError::NotCommon)
        );
    }

    #[test]
    fn smith_temper_refuses_without_a_whetstone_or_for_an_unknown_temper() {
        let mut w = field();
        rest(&mut w);
        let wayfarer = num("wpn_wayfarer_blade");
        hold(&mut w, wayfarer);
        assert_eq!(
            temper_weapon(&mut w, slot::RIGHT_1, 0),
            Err(SmithError::NoWhetstone)
        );
        give(&mut w, "itm_temper_whetstone", 1);
        assert_eq!(
            temper_weapon(&mut w, slot::RIGHT_1, 9),
            Err(SmithError::UnknownTemper)
        );
        // Refused tempers keep the whetstone.
        let whetstone = num("itm_temper_whetstone");
        let player = w.player;
        assert_eq!(w.actors.get(player).unwrap().inventory.count(whetstone), 1);
    }

    #[test]
    fn a_temper_changes_the_attack_rating_and_the_level_does_too() {
        let mut w = field();
        rest(&mut w);
        let wayfarer = num("wpn_wayfarer_blade");
        hold(&mut w, wayfarer);
        give(&mut w, "itm_temper_whetstone", 1);
        give(&mut w, "itm_forgeglass_shard", 1);
        glimmer(&mut w, 1000);
        let attrs = w.actors.get(w.player).unwrap().stats.attrs;
        let before = weapon_rating(&w, wayfarer, &attrs, false).unwrap();
        assert_eq!(temper_weapon(&mut w, slot::RIGHT_1, 2), Ok(()));
        let lucid = weapon_rating(&w, wayfarer, &attrs, false).unwrap();
        // Lucid moves 40% of base into Aether: the physical part shrinks and Aether appears.
        assert!(lucid.phys_milli < before.phys_milli);
        assert!(lucid.elem_milli[0] > 0);
        assert_eq!(lucid.element, Some(lf_content::moves::Element::Aether));
        assert_eq!(upgrade_weapon(&mut w, slot::RIGHT_1), Ok(1));
        let up = weapon_rating(&w, wayfarer, &attrs, false).unwrap();
        assert!(up.phys_milli > lucid.phys_milli);
    }

    #[test]
    fn the_smith_commands_run_through_a_tick() {
        let mut w = field();
        rest(&mut w);
        let wayfarer = num("wpn_wayfarer_blade");
        hold(&mut w, wayfarer);
        give(&mut w, "itm_forgeglass_shard", 1);
        glimmer(&mut w, 1000);
        step(
            &mut w,
            vec![Command::Upgrade {
                slot: slot::RIGHT_1,
            }],
        );
        assert_eq!(effective_level(&w, wayfarer), 1);
        give(&mut w, "itm_temper_whetstone", 1);
        step(
            &mut w,
            vec![Command::Temper {
                slot: slot::RIGHT_1,
                temper: 1,
            }],
        );
        assert_eq!(temper_index(&w, wayfarer), Some(1));
    }
}
