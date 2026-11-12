//! Tests for WP-057: every Charm's effect, the armor sets, the relic armor, and the
//! consumables that aren't flasks (throwables, oils, cures, Glimmer vials, Lantern Ash,
//! arrows).

use std::sync::OnceLock;

use lf_content::Content;
use lf_content::moves::StatusKind;
use lf_content::scenario::DamageType;

use super::gear::{self, load_tenths, max_load_tenths, tier_of};
use super::*;
use crate::actor::Actor;
use crate::actor::stats::Defense;
use crate::arena::Id;
use crate::combat::damage::{self, AttackRating, Damage, Hit};
use crate::combat::resolve::{self, Landing};
use crate::combat::{poise, status};
use crate::events::{Event, ProjectileEventKind};
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::tick::tick;
use crate::world::World;

fn content() -> &'static Content {
    static C: OnceLock<Content> = OnceLock::new();
    C.get_or_init(|| Content::load_embedded().expect("embedded content"))
}

/// A registry number by string ID.
fn reg(id: &str) -> u32 {
    content()
        .registry
        .num(id)
        .unwrap_or_else(|| panic!("{id} is registered"))
}

fn field() -> World {
    World::new(5, content(), "test_field").expect("test_field exists")
}

fn player(w: &World) -> &Actor {
    w.actors.get(w.player).unwrap()
}

fn player_mut(w: &mut World) -> &mut Actor {
    let id = w.player;
    w.actors.get_mut(id).unwrap()
}

fn tick_with(w: &mut World, buttons: u16, commands: Vec<Command>) {
    tick(
        w,
        &TickInput {
            frame: ActionFrame {
                move_x: 0,
                move_z: 0,
                buttons,
            },
            commands,
        },
    );
}

fn idle(w: &mut World, ticks: u32) {
    for _ in 0..ticks {
        tick_with(w, 0, Vec::new());
    }
}

/// Gives the player `id` and equips it in `slot`, through the command.
fn wear(w: &mut World, slot: u8, id: &str) {
    let n = reg(id);
    player_mut(w).inventory.give_gear(n);
    tick_with(w, 0, vec![Command::Equip { slot, item: n }]);
    assert_eq!(
        player(w).inventory.equipment.slots[usize::from(slot)],
        Some(n),
        "{id} is equipped"
    );
}

fn take_off(w: &mut World, slot: u8) {
    tick_with(w, 0, vec![Command::Unequip { slot }]);
    assert_eq!(player(w).inventory.equipment.slots[usize::from(slot)], None);
}

fn near(got: f32, want: f32) {
    assert!((got - want).abs() < 1e-3, "{got} != {want}");
}

fn resist(w: &World, kind: StatusKind) -> u16 {
    player(w).status.resist[status::index(kind)]
}

/// A hit of `ar_milli` thousandths against no defense: `raw = ar * 1.0`, so a physical
/// hit of 100 AR deals 100.
fn hit(phys_milli: u64, element: Option<DamageType>) -> Hit {
    let mut ar = AttackRating {
        phys_milli,
        ..AttackRating::default()
    };
    if let Some(e) = element {
        ar.elem_milli[match e {
            DamageType::Aether => 0,
            DamageType::Fire => 1,
            DamageType::Storm => 2,
            _ => 3,
        }] = 100_000;
    }
    Hit {
        ar,
        phys: DamageType::Neutral,
        element,
        motion_milli: 1000,
        counter: false,
        buff_percent: 100,
    }
}

fn dealt(w: &World, h: &Hit, formula: bool) -> Damage {
    let base = damage::damage(&w.damage.rules, h, &Defense::default());
    gear::outgoing(player(w), &w.items.gear, h, base, formula)
}

// --- the data --------------------------------------------------------------------

#[test]
fn every_charm_and_armor_piece_in_the_spec_exists() {
    let w = field();
    let g = &w.items.gear;
    assert_eq!(g.charms().len(), 14, "the Charms table has 14 rows");
    for id in [
        "chm_vital",
        "chm_ox",
        "chm_quickening",
        "chm_parriers_eye",
        "chm_thornheart",
        "chm_ember_locket",
        "chm_scholars_seal",
        "chm_shrike_lite",
        "chm_smol_llama",
        "chm_bulwark",
        "chm_last_stand",
        "chm_lantern_moth",
        "chm_rimeward",
        "chm_clearblood",
    ] {
        assert!(g.charm(reg(id)).is_some(), "{id}");
    }
    // The three relic charms are marked.
    let relics: Vec<_> = g
        .charms()
        .iter()
        .filter(|c| c.relic)
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(
        relics,
        ["chm_scholars_seal", "chm_shrike_lite", "chm_smol_llama"]
    );
    assert_eq!(
        g.armor_pieces().len(),
        16,
        "seven sets and two relic pieces"
    );
}

#[test]
fn each_armor_set_adds_up_to_the_spec_table() {
    let w = field();
    let g = &w.items.gear;
    // (head, body, weight in tenths, poise)
    let sets = [
        ("arm_wayfarer_head", "arm_wayfarer_body", 70, 8),
        ("arm_chapel_head", "arm_chapel_body", 50, 4),
        ("arm_reedwalker_head", "arm_reedwalker_body", 70, 8),
        ("arm_miner_head", "arm_miner_body", 120, 18),
        ("arm_exile_head", "arm_exile_body", 50, 4),
        ("arm_graymantle_head", "arm_graymantle_body", 250, 45),
        ("arm_rimeforged_head", "arm_rimeforged_body", 220, 40),
    ];
    for (head, body, weight, poise) in sets {
        let (h, b) = (g.armor(reg(head)).unwrap(), g.armor(reg(body)).unwrap());
        assert_eq!(h.slot, lf_content::gear::ArmorSlot::Head, "{head}");
        assert_eq!(b.slot, lf_content::gear::ArmorSlot::Body, "{body}");
        assert_eq!(h.weight_tenths + b.weight_tenths, weight, "{head} weight");
        assert_eq!(h.poise + b.poise, poise, "{head} poise");
        assert_eq!(h.set, b.set);
    }
}

#[test]
fn armor_goes_in_its_own_slot_and_charms_in_charm_slots() {
    let mut w = field();
    let (head, body, vital) = (
        reg("arm_wayfarer_head"),
        reg("arm_wayfarer_body"),
        reg("chm_vital"),
    );
    let items = w.items.clone();
    let inv = &mut player_mut(&mut w).inventory;
    for n in [head, body, vital] {
        assert!(inv.give_gear(n));
    }
    assert!(!inv.equip(slot::BODY, head, &items), "a hood isn't a coat");
    assert!(!inv.equip(slot::HEAD, body, &items));
    assert!(
        !inv.equip(slot::CHARM_1, head, &items),
        "armor isn't a charm"
    );
    assert!(!inv.equip(slot::HEAD, vital, &items), "a charm isn't armor");
    assert!(inv.equip(slot::HEAD, head, &items));
    assert!(inv.equip(slot::BODY, body, &items));
    assert!(inv.equip(slot::CHARM_1 + 1, vital, &items));
    // Only two charm slots are open at the start.
    assert!(!inv.equip(slot::CHARM_1 + 2, vital, &items));
    // Pickups: one of each.
    assert_eq!(inv.pick_up(reg("chm_ox"), 1, &items), 1);
    assert_eq!(inv.pick_up(reg("chm_ox"), 1, &items), 0);
}

// --- the charms ------------------------------------------------------------------

#[test]
fn vital_charm_adds_8_percent_max_hp() {
    let mut w = field();
    let base = player(&w).max_hp;
    wear(&mut w, slot::CHARM_1, "chm_vital");
    let boosted = (u64::from(base) * 108 / 100) as u32;
    assert_eq!(player(&w).max_hp, boosted);
    assert!(boosted > base);
    // Full HP above the old maximum is clipped when the charm comes off.
    player_mut(&mut w).hp = boosted;
    take_off(&mut w, slot::CHARM_1);
    assert_eq!(player(&w).max_hp, base);
    assert_eq!(player(&w).hp, base);
}

#[test]
fn ox_charm_adds_15_percent_max_equip_load() {
    let mut w = field();
    let endurance = player(&w).stats.attrs.endurance;
    let base = max_load_tenths(&w, endurance, &player(&w).inventory);
    wear(&mut w, slot::CHARM_1, "chm_ox");
    let boosted = max_load_tenths(&w, endurance, &player(&w).inventory);
    assert_eq!(boosted, (u64::from(base) * 115 / 100) as u32);
    // The equipment menu's preview sees it too.
    let stats = preview::loadout_stats(&w, &player(&w).inventory).unwrap();
    assert_eq!(stats.max_load_tenths, boosted);
}

#[test]
fn quickening_charm_adds_12_percent_stamina_regeneration() {
    let gain = |charm: bool| {
        let mut w = field();
        if charm {
            wear(&mut w, slot::CHARM_1, "chm_quickening");
        }
        let a = player_mut(&mut w);
        a.stamina = 20.0;
        a.motor.stamina_idle = 15; // the regeneration delay has passed
        tick_with(&mut w, 0, Vec::new());
        player(&w).stamina - 20.0
    };
    near(gain(false), 1.5);
    near(gain(true), 1.5 * 1.12);
}

#[test]
fn parriers_eye_lengthens_the_parry_window_by_2_ticks() {
    let mut w = field();
    // The Parry Dagger parries; the Wayfarer's Blade has no window to lengthen.
    let dagger = reg("wpn_parry_dagger");
    player_mut(&mut w).inventory.give_gear(dagger);
    let blade_window = poise::parry_window_of(&w, w.player);
    tick_with(
        &mut w,
        0,
        vec![Command::Equip {
            slot: slot::RIGHT_1,
            item: dagger,
        }],
    );
    let base = poise::parry_window_of(&w, w.player);
    assert!(base > 0 && blade_window == 0, "{base} and {blade_window}");
    wear(&mut w, slot::CHARM_1, "chm_parriers_eye");
    assert_eq!(poise::parry_window_of(&w, w.player), base + 2);
    // A weapon with no parry window gains none.
    w.moves
        .player_moveset
        .as_mut()
        .unwrap()
        .guard
        .parry_window_ticks = 0;
    assert_eq!(poise::parry_window_of(&w, w.player), 0);
}

/// An enemy-like actor that can fill a Rupture meter, at `pos`.
fn rupturable(w: &mut World, pos: [f32; 3]) -> Id {
    let id = w
        .actors
        .iter()
        .map(|(id, _)| id)
        .find(|&id| id != w.player)
        .expect("the test field has dummies");
    let a = w.actors.get_mut(id).unwrap();
    a.pos = pos;
    a.status.resist[status::index(StatusKind::Rupture)] = 10;
    id
}

#[test]
fn thornheart_buffs_damage_12_percent_for_600_ticks_after_a_rupture_proc_near_you() {
    let mut w = field();
    wear(&mut w, slot::CHARM_1, "chm_thornheart");
    assert_eq!(player(&w).stats.buff_percent, 100);
    // A proc 20 m away does nothing.
    let foe = rupturable(&mut w, [20.0, 0.0, 20.0]);
    assert!(status::add_buildup(
        &mut w,
        foe,
        StatusKind::Rupture,
        1_000_000
    ));
    assert_eq!(player(&w).status.body_buff, None);
    // A proc within 8 m grants the buff.
    let f = w.actors.get_mut(foe).unwrap();
    f.pos = [3.0, 0.0, 3.0];
    f.status.lockout = [0; 6]; // the first proc's lockout
    assert!(status::add_buildup(
        &mut w,
        foe,
        StatusKind::Rupture,
        1_000_000_000
    ));
    let buff = player(&w).status.body_buff.expect("Thornheart's buff");
    assert_eq!(Some(buff.num), w.status.buff_num("buf_thornheart"));
    assert_eq!(buff.ticks, 600);
    assert_eq!(player(&w).stats.buff_percent, 112);
    // It ends after its 600 ticks.
    idle(&mut w, 601);
    assert_eq!(player(&w).status.body_buff, None);
    assert_eq!(player(&w).stats.buff_percent, 100);
    // Without the charm, the same proc does nothing for the player.
    let mut bare = field();
    let foe = rupturable(&mut bare, [3.0, 0.0, 3.0]);
    assert!(status::add_buildup(
        &mut bare,
        foe,
        StatusKind::Rupture,
        1_000_000_000
    ));
    assert_eq!(player(&bare).status.body_buff, None);
}

#[test]
fn ember_locket_adds_12_percent_fire_damage() {
    let mut w = field();
    let fire = hit(0, Some(DamageType::Fire));
    let storm = hit(0, Some(DamageType::Storm));
    assert_eq!(dealt(&w, &fire, false).total, 100);
    wear(&mut w, slot::CHARM_1, "chm_ember_locket");
    assert_eq!(dealt(&w, &fire, false).total, 112);
    assert_eq!(dealt(&w, &storm, false).total, 100, "only Fire");
    assert_eq!(
        dealt(&w, &hit(100_000, None), false).total,
        100,
        "not physical"
    );
}

#[test]
fn scholars_seal_adds_10_percent_formula_damage_and_a_spell_slot() {
    let mut w = field();
    let bolt = hit(0, Some(DamageType::Aether));
    assert_eq!(
        w.items
            .gear
            .spell_slot_bonus(&player(&w).inventory.equipment),
        0
    );
    wear(&mut w, slot::CHARM_1, "chm_scholars_seal");
    assert_eq!(dealt(&w, &bolt, true).total, 110);
    assert_eq!(dealt(&w, &bolt, false).total, 100, "not a Formula");
    assert_eq!(
        w.items
            .gear
            .spell_slot_bonus(&player(&w).inventory.equipment),
        1
    );
}

#[test]
fn shrike_lite_cuts_spell_startup_12_percent() {
    let mut w = field();
    let eq = |w: &World| player(w).inventory.equipment;
    assert_eq!(w.items.gear.spell_startup_reduce_percent(&eq(&w)), 0);
    wear(&mut w, slot::CHARM_1, "chm_shrike_lite");
    let cut = w.items.gear.spell_startup_reduce_percent(&eq(&w));
    assert_eq!(cut, 12);
    // A 50-frame startup becomes 44.
    assert_eq!(50 * (100 - cut) / 100, 44);
}

#[test]
fn smol_llama_makes_equipped_weapons_weigh_25_percent_less() {
    let mut w = field();
    let blade = w.items.default_weapon.unwrap();
    let weight = u32::from(w.items.weight_tenths(blade));
    let before = load_tenths(&w, &player(&w).inventory);
    wear(&mut w, slot::CHARM_1, "chm_smol_llama");
    let after = load_tenths(&w, &player(&w).inventory);
    assert_eq!(before - after, weight - weight * 75 / 100);
    assert!(after < before);
}

/// The stamina one blocked hit of 20 poise damage costs the player.
fn block_cost(w: &mut World) -> f32 {
    let light = w.moves.num_of("ss_light_1").unwrap();
    let id = w.player;
    let a = player_mut(w);
    a.moves.guard.active = true;
    a.stamina = 100.0;
    let l = Landing {
        attacker: id,
        target: id,
        move_num: light,
        hitbox: 0,
        motion_value: 1.0,
        charge_frac: 0.0,
        point: [0.0; 3],
        dir: [0.0, 0.0, 1.0],
        damage: Damage {
            total: 10,
            phys_milli: 10_000,
            elem_milli: 0,
        },
        counter: false,
        poise_damage: 20,
        unblockable: false,
        status: None,
    };
    assert!(resolve::apply_landing(w, &l));
    100.0 - player(w).stamina
}

#[test]
fn bulwark_charm_cuts_the_guard_stamina_cost_15_percent() {
    let mut w = field();
    let plain = block_cost(&mut w);
    assert!(plain > 0.0);
    let mut w = field();
    wear(&mut w, slot::CHARM_1, "chm_bulwark");
    let with = block_cost(&mut w);
    let ratio = with / plain;
    assert!((0.84..=0.86).contains(&ratio), "{with} / {plain} = {ratio}");
}

#[test]
fn last_stand_charm_adds_15_percent_damage_under_20_percent_hp() {
    let mut w = field();
    wear(&mut w, slot::CHARM_1, "chm_last_stand");
    let h = hit(100_000, None);
    let max = player(&w).max_hp;
    player_mut(&mut w).hp = max;
    assert_eq!(dealt(&w, &h, false).total, 100, "healthy");
    // Exactly 20% isn't under 20%.
    player_mut(&mut w).hp = max / 5;
    assert_eq!(
        u64::from(max / 5) * 100,
        u64::from(max) * 20,
        "max HP divides by 5"
    );
    assert_eq!(dealt(&w, &h, false).total, 100);
    player_mut(&mut w).hp = max / 5 - 1;
    assert_eq!(dealt(&w, &h, false).total, 115);
}

#[test]
fn lantern_moth_charm_restores_1_fp_every_2_seconds() {
    let mut w = field();
    wear(&mut w, slot::CHARM_1, "chm_lantern_moth");
    player_mut(&mut w).fp = 0.0;
    let start = w.tick;
    idle(&mut w, 120);
    // One restore for each multiple of 60 ticks in the span.
    let multiples = (start..start + 120).filter(|t| t % 60 == 0).count();
    assert_eq!(multiples, 2);
    near(player(&w).fp, multiples as f32);
    // And none without the charm.
    let mut bare = field();
    player_mut(&mut bare).fp = 0.0;
    idle(&mut bare, 120);
    near(player(&bare).fp, 0.0);
}

#[test]
fn rimeward_charm_adds_60_robustness() {
    let mut w = field();
    let (rupture, rime, poison) = (
        resist(&w, StatusKind::Rupture),
        resist(&w, StatusKind::Rime),
        resist(&w, StatusKind::Poison),
    );
    wear(&mut w, slot::CHARM_1, "chm_rimeward");
    assert_eq!(resist(&w, StatusKind::Rupture), rupture + 60);
    assert_eq!(resist(&w, StatusKind::Rime), rime + 60);
    assert_eq!(
        resist(&w, StatusKind::Poison),
        poison,
        "Immunity is untouched"
    );
    take_off(&mut w, slot::CHARM_1);
    assert_eq!(resist(&w, StatusKind::Rupture), rupture);
}

#[test]
fn clearblood_charm_adds_60_immunity() {
    let mut w = field();
    let (poison, blight, rupture) = (
        resist(&w, StatusKind::Poison),
        resist(&w, StatusKind::Blight),
        resist(&w, StatusKind::Rupture),
    );
    wear(&mut w, slot::CHARM_1, "chm_clearblood");
    assert_eq!(resist(&w, StatusKind::Poison), poison + 60);
    assert_eq!(resist(&w, StatusKind::Blight), blight + 60);
    assert_eq!(
        resist(&w, StatusKind::Rupture),
        rupture,
        "Robustness is untouched"
    );
    // Two charms stack: Clearblood and Rimeward in the two open slots.
    wear(&mut w, slot::CHARM_1 + 1, "chm_rimeward");
    assert_eq!(resist(&w, StatusKind::Poison), poison + 60);
    assert_eq!(resist(&w, StatusKind::Rupture), rupture + 60);
}

// --- armor -----------------------------------------------------------------------

fn wear_set(w: &mut World, head: &str, body: &str) {
    wear(w, slot::HEAD, head);
    wear(w, slot::BODY, body);
}

#[test]
fn wearing_the_starting_coat_keeps_the_stub_defense_and_sets_its_poise_and_weight() {
    let mut w = field();
    let stub_defense = player(&w).stats.defense;
    let stub_load = load_tenths(&w, &player(&w).inventory);
    wear_set(&mut w, "arm_wayfarer_head", "arm_wayfarer_body");
    let d = player(&w).stats.defense;
    // 40 flat against everything, as the stub; the coat adds 5% absorption.
    assert_eq!(d.flat, stub_defense.flat);
    assert_eq!(d.absorb_percent, [5; 8]);
    assert_eq!(player(&w).poise.poise, 8);
    // 7.0 of armor replaces the stub's 20.0.
    let blade = u32::from(w.items.weight_tenths(w.items.default_weapon.unwrap()));
    assert_eq!(load_tenths(&w, &player(&w).inventory), blade + 70);
    assert!(load_tenths(&w, &player(&w).inventory) < stub_load);
    // Taking it off brings the stub back.
    take_off(&mut w, slot::HEAD);
    take_off(&mut w, slot::BODY);
    assert_eq!(player(&w).stats.defense, stub_defense);
    assert_eq!(player(&w).poise.poise, w.locomotion.poise.stub_poise);
    assert_eq!(load_tenths(&w, &player(&w).inventory), stub_load);
}

#[test]
fn graymantle_plate_is_heavy_defended_and_absorbs_30_percent_physical() {
    let mut w = field();
    let plain = player(&w).stats.defense;
    wear_set(&mut w, "arm_graymantle_head", "arm_graymantle_body");
    let d = player(&w).stats.defense;
    assert_eq!(d.flat[DamageType::Slash as usize], 120);
    assert!(d.flat[DamageType::Slash as usize] > plain.flat[DamageType::Slash as usize]);
    for t in [
        DamageType::Neutral,
        DamageType::Slash,
        DamageType::Blunt,
        DamageType::Pierce,
    ] {
        assert_eq!(d.absorb_percent[t as usize], 30, "{t:?}");
    }
    assert_eq!(d.absorb_percent[DamageType::Fire as usize], 0);
    assert_eq!(player(&w).poise.poise, 45);
    // The same hit hurts less: defense and absorption both count.
    let h = hit(100_000, None);
    let lighter = damage::damage(&w.damage.rules, &h, &d).total;
    let heavier = damage::damage(&w.damage.rules, &h, &plain).total;
    assert!(lighter < heavier, "{lighter} < {heavier}");
    // The roll tier follows the new load.
    let tier = tier_of(&w, player(&w).stats.attrs.endurance, &player(&w).inventory);
    assert_eq!(w.moves.player_moveset.as_ref().unwrap().load_tier, tier);
}

#[test]
fn a_heavier_loadout_changes_the_tier_and_the_moveset_knows() {
    let mut w = field();
    let light = w.moves.player_moveset.as_ref().unwrap().load_tier;
    wear_set(&mut w, "arm_graymantle_head", "arm_graymantle_body");
    // Squeeze the load limit until the plate is a heavy load (a level-up or respec calls
    // `refresh` again; it works from the attributes).
    player_mut(&mut w).stats.attrs.endurance = 1;
    gear::refresh(&mut w);
    let tier = tier_of(&w, 1, &player(&w).inventory);
    assert_eq!(tier, crate::combat::dodge::LoadTier::Heavy);
    assert_ne!(tier, light);
    assert_eq!(w.moves.player_moveset.as_ref().unwrap().load_tier, tier);
    // Ox Charm raises the limit and can lift the tier back.
    let before = tier;
    wear(&mut w, slot::CHARM_1, "chm_ox");
    let after = w.moves.player_moveset.as_ref().unwrap().load_tier;
    assert_eq!(after, tier_of(&w, 1, &player(&w).inventory));
    assert!(after as u8 <= before as u8);
}

#[test]
fn the_elemental_sets_absorb_their_elements() {
    let cases: [(&str, &str, DamageType, u8); 3] = [
        (
            "arm_chapel_head",
            "arm_chapel_body",
            DamageType::Radiance,
            20,
        ),
        ("arm_miner_head", "arm_miner_body", DamageType::Fire, 15),
        ("arm_exile_head", "arm_exile_body", DamageType::Aether, 25),
    ];
    for (head, body, t, pct) in cases {
        let mut w = field();
        wear_set(&mut w, head, body);
        let d = player(&w).stats.defense;
        assert_eq!(d.absorb_percent[t as usize], pct, "{head}");
        let others: u32 = d.absorb_percent.iter().map(|&p| u32::from(p)).sum();
        assert_eq!(others, u32::from(pct), "{head} absorbs nothing else");
    }
}

#[test]
fn reedwalker_garb_adds_40_immunity_and_rimeforged_mail_80_robustness() {
    let mut w = field();
    let (poison, rupture) = (
        resist(&w, StatusKind::Poison),
        resist(&w, StatusKind::Rupture),
    );
    wear_set(&mut w, "arm_reedwalker_head", "arm_reedwalker_body");
    assert_eq!(resist(&w, StatusKind::Poison), poison + 40);
    assert_eq!(resist(&w, StatusKind::Blight), poison + 40);
    assert_eq!(resist(&w, StatusKind::Rupture), rupture);
    wear_set(&mut w, "arm_rimeforged_head", "arm_rimeforged_body");
    assert_eq!(resist(&w, StatusKind::Poison), poison, "the garb is off");
    assert_eq!(resist(&w, StatusKind::Rupture), rupture + 80);
    assert_eq!(resist(&w, StatusKind::Rime), rupture + 80);
}

#[test]
fn boo_shows_you_at_80_percent_range_and_endark_at_50_in_the_dark() {
    let mut w = field();
    let eq = |w: &World| player(w).inventory.equipment;
    assert_eq!(w.items.gear.sight_percent(&eq(&w), false), 100);
    wear(&mut w, slot::HEAD, "arm_boo");
    assert_eq!(w.items.gear.sight_percent(&eq(&w), false), 80);
    assert_eq!(w.items.gear.sight_percent(&eq(&w), true), 80);
    wear(&mut w, slot::BODY, "arm_endark");
    // Endark only counts in darkness, which isn't built yet: the lit value is Boo's.
    assert_eq!(w.items.gear.sight_percent(&eq(&w), false), 80);
    assert_eq!(w.items.gear.sight_percent(&eq(&w), true), 40, "80% of 50%");
    // The AI's perception range is scaled the same way.
    let per = lf_content::enemy::Perception {
        sight_deg: 120.0,
        sight_m: 20.0,
        hearing_mult: 1.0,
    };
    let seen = gear::scale_sight(&w.items.gear, player(&w), &per);
    near(seen.sight_m, 16.0);
    // Only the player is scaled.
    let other = Actor::default();
    near(gear::scale_sight(&w.items.gear, &other, &per).sight_m, 20.0);
}

// --- consumables -----------------------------------------------------------------

/// Gives the player `n` of `id` in quick slot 3 and uses it, then lets the effect land.
fn use_item(w: &mut World, id: &str, n: u16) {
    let item = reg(id);
    let items = w.items.clone();
    let inv = &mut player_mut(w).inventory;
    assert_eq!(inv.pick_up(item, n, &items), n, "{id} goes on its stack");
    assert!(inv.equip(slot::QUICK_1 + 2, item, &items));
    tick_with(w, 0, vec![Command::UseItem { slot: 2 }]);
    let drink = w.items.flask.as_ref().unwrap().drink_move;
    assert_eq!(player(w).moves.current.map(|c| c.num), Some(drink));
    // Frame 16 lands it and the world applies it on the next tick; the move ends on
    // frame 30, so the next use can start.
    idle(w, 40);
}

fn count(w: &World, id: &str) -> u16 {
    player(w).inventory.count(reg(id))
}

#[test]
fn glimmer_vials_grant_200_and_1000() {
    let mut w = field();
    let before = player(&w).inventory.glimmer;
    use_item(&mut w, "itm_glimmer_vial", 2);
    assert_eq!(player(&w).inventory.glimmer, before + 200);
    assert_eq!(count(&w, "itm_glimmer_vial"), 1);
    // Another one after the first move ends.
    use_item(&mut w, "itm_glimmer_vial_large", 1);
    assert_eq!(player(&w).inventory.glimmer, before + 1200);
    assert_eq!(count(&w, "itm_glimmer_vial_large"), 0);
}

#[test]
fn a_cure_clears_only_what_it_cures() {
    let cases: [(&str, &[StatusKind]); 5] = [
        ("itm_clotweed", &[StatusKind::Rupture]),
        ("itm_antidote_root", &[StatusKind::Poison]),
        ("itm_clearwater", &[StatusKind::Blight]),
        ("itm_hearth_ember", &[StatusKind::Rime]),
        (
            "itm_waking_salts",
            &[StatusKind::Drowse, StatusKind::Delirium],
        ),
    ];
    for (item, cures) in cases {
        let mut w = field();
        let a = player_mut(&mut w);
        for k in status::KINDS {
            a.status.meter_milli[status::index(k)] = 5000;
            a.status.active[status::index(k)] = 100;
        }
        use_item(&mut w, item, 1);
        for k in status::KINDS {
            let i = status::index(k);
            let cleared = player(&w).status.meter_milli[i] == 0 && player(&w).status.active[i] == 0;
            assert_eq!(cleared, cures.contains(&k), "{item} and {k:?}");
        }
        assert_eq!(count(&w, item), 0, "{item} was used up");
    }
}

#[test]
fn lantern_ash_returns_you_to_the_last_lantern() {
    let mut w = field();
    // Rest at the lantern 3 m behind the spawn, then wander off.
    crate::world_state::lantern::rest(&mut w, 0);
    let (home, _) = crate::world_state::death::respawn_point(&w);
    assert_eq!(home, [0.0, 0.0, -3.0]);
    player_mut(&mut w).pos = [5.0, 0.0, 12.0];
    use_item(&mut w, "itm_lantern_ash", 1);
    assert_eq!(player(&w).pos, home);
    assert_eq!(count(&w, "itm_lantern_ash"), 0);
}

#[test]
fn oils_put_their_buff_on_the_weapon() {
    let mut w = field();
    use_item(&mut w, "itm_pitch_oil", 2);
    let buff = player(&w).status.weapon_buff.expect("the oil's buff");
    assert_eq!(Some(buff.num), w.status.buff_num("buf_pitch_oil"));
    assert!(
        (850..900).contains(&buff.ticks),
        "900 ticks, started a while ago"
    );
    assert_eq!(count(&w, "itm_pitch_oil"), 1);
    // Another oil replaces it (one weapon buff at a time).
    use_item(&mut w, "itm_spark_oil", 1);
    assert_eq!(
        player(&w).status.weapon_buff.map(|b| b.num),
        w.status.buff_num("buf_spark_oil")
    );
    use_item(&mut w, "itm_bloodbloom_oil", 1);
    assert_eq!(
        player(&w).status.weapon_buff.map(|b| b.num),
        w.status.buff_num("buf_bloodbloom_oil")
    );
}

#[test]
fn an_elemental_oil_adds_20_percent_of_ar_as_its_element() {
    let mut w = field();
    let melee = hit(100_000, None);
    for (oil, element, idx) in [
        ("itm_pitch_oil", DamageType::Fire, 1),
        ("itm_spark_oil", DamageType::Storm, 2),
    ] {
        use_item(&mut w, oil, 1);
        let oiled = gear::weapon_element(player(&w), &w.status, melee);
        assert_eq!(oiled.element, Some(element), "{oil}");
        assert_eq!(oiled.ar.elem_milli[idx], 20_000, "{oil}: 20% of 100 AR");
        // It's a real extra: 100 physical plus 20 elemental.
        let base = damage::damage(&w.damage.rules, &melee, &Defense::default());
        let more = damage::damage(&w.damage.rules, &oiled, &Defense::default());
        assert_eq!((base.total, more.total), (100, 120), "{oil}");
    }
    // A hit that has an element of its own keeps it.
    let fire = hit(100_000, Some(DamageType::Fire));
    assert_eq!(gear::weapon_element(player(&w), &w.status, fire), fire);
    // Without a buff the hit is unchanged.
    let bare = field();
    assert_eq!(
        gear::weapon_element(player(&bare), &bare.status, melee),
        melee
    );
    // Ember Locket stacks with Pitch Oil's Fire.
    wear(&mut w, slot::CHARM_1, "chm_ember_locket");
    use_item(&mut w, "itm_pitch_oil", 1);
    let oiled = gear::weapon_element(player(&w), &w.status, melee);
    let total = dealt(&w, &oiled, false);
    assert_eq!(total.elem_milli, 22_400, "20 Fire x 1.12");
}

/// Moves every actor but the player far from the throwing lane.
fn clear_the_lane(w: &mut World) {
    let ids: Vec<Id> = w
        .actors
        .iter()
        .map(|(id, _)| id)
        .filter(|&i| i != w.player)
        .collect();
    for (k, id) in ids.into_iter().enumerate() {
        w.actors.get_mut(id).unwrap().pos = [-25.0, 0.0, -25.0 + 3.0 * k as f32];
    }
}

/// Where `item`'s projectile first comes down when thrown straight ahead on a clear field.
fn throw_distance(item: &str) -> [f32; 3] {
    let mut w = field();
    clear_the_lane(&mut w);
    use_item_quiet(&mut w, item);
    let mut end = None;
    for _ in 0..120 {
        tick_with(&mut w, 0, Vec::new());
        for e in w.events.drain() {
            if let Event::Projectile(p) = e
                && p.ended
            {
                end = Some(p.pos);
            }
        }
        if end.is_some() {
            break;
        }
    }
    end.expect("the throw came down")
}

/// Uses an item and stops as soon as the projectile is out (it flies on the caller's ticks).
fn use_item_quiet(w: &mut World, id: &str) {
    let item = reg(id);
    let items = w.items.clone();
    let inv = &mut player_mut(w).inventory;
    inv.pick_up(item, 3, &items);
    inv.equip(slot::QUICK_1 + 2, item, &items);
    tick_with(w, 0, vec![Command::UseItem { slot: 2 }]);
    while w.projectiles.iter().next().is_none() {
        tick_with(w, 0, Vec::new());
        assert!(w.tick < 100, "the item was thrown");
    }
}

fn hp_of(w: &World, id: Id) -> u32 {
    w.actors.get(id).unwrap().hp
}

#[test]
fn a_thrown_item_is_spent_when_it_leaves_the_hand() {
    let mut w = field();
    clear_the_lane(&mut w);
    use_item_quiet(&mut w, "itm_throwing_knife");
    assert_eq!(count(&w, "itm_throwing_knife"), 2, "one of three thrown");
    let p = w.projectiles.iter().next().unwrap().1;
    assert_eq!(p.def, reg("proj_throwing_knife"));
    // Its damage is the item's, not the weapon's: 40 Pierce, no element.
    assert_eq!((p.ar_phys_milli, p.ar_elem_milli), (40_000, [0; 4]));
    assert_eq!(p.buff_percent, 100);
}

#[test]
fn the_pitch_bomb_burns_everyone_within_2_m_of_where_it_lands() {
    let at = throw_distance("itm_pitch_bomb");
    let mut w = field();
    clear_the_lane(&mut w);
    let ids: Vec<Id> = w
        .actors
        .iter()
        .map(|(id, _)| id)
        .filter(|&i| i != w.player)
        .collect();
    let (a, b, far) = (ids[0], ids[1], ids[2]);
    // Two dummies 1.2 m either side of the landing point (too far for a direct hit) and
    // one 6 m away.
    w.actors.get_mut(a).unwrap().pos = [at[0] + 1.2, 0.0, at[2]];
    w.actors.get_mut(b).unwrap().pos = [at[0] - 1.2, 0.0, at[2]];
    w.actors.get_mut(far).unwrap().pos = [at[0] + 6.0, 0.0, at[2]];
    let hp = [hp_of(&w, a), hp_of(&w, b), hp_of(&w, far)];
    use_item_quiet(&mut w, "itm_pitch_bomb");
    idle(&mut w, 120);
    assert!(hp_of(&w, a) < hp[0], "the first dummy burned");
    assert!(hp_of(&w, b) < hp[1], "the second dummy burned");
    assert_eq!(hp_of(&w, far), hp[2], "the far dummy didn't");
    assert_eq!(count(&w, "itm_pitch_bomb"), 2);
}

/// Throws `item` at a dummy standing where it comes down. Returns the dummy and the
/// highest the `kind` meter got (it decays afterward).
fn throw_at_a_dummy(w: &mut World, item: &str, kind: StatusKind) -> (Id, u32) {
    let at = throw_distance(item);
    clear_the_lane(w);
    let id = w
        .actors
        .iter()
        .map(|(id, _)| id)
        .find(|&i| i != w.player)
        .unwrap();
    let d = w.actors.get_mut(id).unwrap();
    d.pos = at;
    d.status.resist = [200; 6];
    use_item_quiet(w, item);
    let mut peak = 0;
    for _ in 0..120 {
        tick_with(w, 0, Vec::new());
        peak = peak.max(meter(w, id, kind));
    }
    (id, peak)
}

fn meter(w: &World, id: Id, kind: StatusKind) -> u32 {
    w.actors.get(id).unwrap().status.meter_milli[status::index(kind)]
}

#[test]
fn the_frostjar_builds_80_rime_the_blight_pot_120_blight_and_the_knife_40_poison() {
    for (item, kind, amount) in [
        ("itm_frostjar", StatusKind::Rime, 80),
        ("itm_blight_pot", StatusKind::Blight, 120),
        ("itm_throwing_knife", StatusKind::Poison, 40),
    ] {
        let mut w = field();
        let (_, peak) = throw_at_a_dummy(&mut w, item, kind);
        assert_eq!(peak, amount * 1000, "{item}");
    }
}

#[test]
fn the_frostjar_hits_for_60_aether_and_the_knife_for_40_pierce() {
    // Against flat 500 defense: 60^2 / 560 = 6.4 and 40^2 / 540 = 3.0; the pot and bomb
    // numbers follow the same rule. Compare with a hit from the damage pipeline.
    for (item, raw, kind) in [
        ("itm_frostjar", 60u64, StatusKind::Rime),
        ("itm_throwing_knife", 40, StatusKind::Poison),
    ] {
        let mut w = field();
        let hp0 = 100;
        let (dummy, _) = throw_at_a_dummy(&mut w, item, kind);
        let taken = hp0 - hp_of(&w, dummy);
        let want = (raw * raw * 1000 / (raw + 500)) as u32 / 1000;
        assert!(
            taken >= want.saturating_sub(1) && taken <= want + 1,
            "{item}: took {taken}, expected about {want}"
        );
    }
}

#[test]
fn arrows_in_the_arrow_slot_replace_the_bows_arrow_and_are_spent() {
    let spawned = |w: &mut World| -> Vec<(u32, [u64; 4])> {
        let mut out = Vec::new();
        for _ in 0..60 {
            tick_with(w, buttons::LIGHT, Vec::new());
            for e in w.events.drain() {
                if let Event::Projectile(p) = e
                    && p.kind == ProjectileEventKind::Spawned
                {
                    let proj = w.projectiles.get(p.projectile).unwrap();
                    out.push((p.def, proj.ar_elem_milli));
                }
            }
            if !out.is_empty() {
                break;
            }
        }
        out
    };
    let bow = reg("wpn_stub_bow");
    let arm = |w: &mut World, arrows: Option<(&str, u16)>| {
        let items = w.items.clone();
        let inv = &mut player_mut(w).inventory;
        inv.give_gear(bow);
        assert!(inv.equip(slot::RIGHT_1, bow, &items));
        if let Some((id, n)) = arrows {
            inv.pick_up(reg(id), n, &items);
            assert!(inv.equip(slot::ARROWS, reg(id), &items));
        }
        w.sync_player_moveset();
    };
    // No arrows equipped: the stub bow fires its own arrow, as before.
    let mut w = field();
    arm(&mut w, None);
    let shots = spawned(&mut w);
    assert_eq!(shots.len(), 1);
    assert_eq!(shots[0], (reg("proj_arrow"), [0; 4]));
    // Fire arrows: the shot is a Fire Arrow with Fire at 20% of the bow's AR, and one
    // arrow is gone.
    let mut w = field();
    arm(&mut w, Some(("itm_arrows_fire", 5)));
    let shots = spawned(&mut w);
    assert_eq!(shots.len(), 1);
    assert_eq!(shots[0].0, reg("proj_arrow_fire"));
    assert!(shots[0].1[1] > 0, "Fire AR: {:?}", shots[0].1);
    assert_eq!(count(&w, "itm_arrows_fire"), 4);
    // Storm arrows add Storm; iron arrows add nothing but are spent too.
    let mut w = field();
    arm(&mut w, Some(("itm_arrows_storm", 5)));
    let shots = spawned(&mut w);
    assert_eq!(shots[0].0, reg("proj_arrow_storm"));
    assert!(shots[0].1[2] > 0);
    let mut w = field();
    arm(&mut w, Some(("itm_arrows", 5)));
    let shots = spawned(&mut w);
    assert_eq!(shots[0], (reg("proj_arrow"), [0; 4]));
    assert_eq!(count(&w, "itm_arrows"), 4);
}

#[test]
fn an_empty_stack_of_arrows_falls_back_to_the_base_arrow() {
    let mut w = field();
    let bow = reg("wpn_stub_bow");
    let items = w.items.clone();
    let inv = &mut player_mut(&mut w).inventory;
    inv.give_gear(bow);
    inv.equip(slot::RIGHT_1, bow, &items);
    inv.pick_up(reg("itm_arrows_fire"), 1, &items);
    inv.equip(slot::ARROWS, reg("itm_arrows_fire"), &items);
    inv.remove(reg("itm_arrows_fire"), 1);
    let me = w.player;
    assert_eq!(
        usage::take_ammo(&mut w, me, "proj_arrow"),
        None,
        "no arrows left"
    );
    // An enemy's arrow is never swapped.
    let other = w
        .actors
        .iter()
        .map(|(id, _)| id)
        .find(|&i| i != w.player)
        .unwrap();
    assert_eq!(usage::take_ammo(&mut w, other, "proj_arrow"), None);
}

#[test]
fn an_item_with_no_stock_does_not_start_the_drink() {
    let mut w = field();
    let item = reg("itm_clotweed");
    let items = w.items.clone();
    let inv = &mut player_mut(&mut w).inventory;
    inv.pick_up(item, 1, &items);
    inv.equip(slot::QUICK_1 + 2, item, &items);
    inv.remove(item, 1);
    tick_with(&mut w, 0, vec![Command::UseItem { slot: 2 }]);
    assert_eq!(player(&w).moves.current, None);
}

#[test]
fn an_interrupted_use_spends_nothing() {
    let mut w = field();
    let item = reg("itm_glimmer_vial");
    let items = w.items.clone();
    let before = player(&w).inventory.glimmer;
    let inv = &mut player_mut(&mut w).inventory;
    inv.pick_up(item, 1, &items);
    inv.equip(slot::QUICK_1 + 2, item, &items);
    tick_with(&mut w, 0, vec![Command::UseItem { slot: 2 }]);
    idle(&mut w, 5);
    // A stagger before frame 16 ends the move.
    player_mut(&mut w).moves.interrupt();
    idle(&mut w, 40);
    assert_eq!(player(&w).inventory.glimmer, before);
    assert_eq!(count(&w, "itm_glimmer_vial"), 1);
}

#[test]
fn equipping_gear_replays_to_the_same_state_hash() {
    let run = || {
        let mut w = field();
        wear(&mut w, slot::CHARM_1, "chm_vital");
        wear_set(&mut w, "arm_graymantle_head", "arm_graymantle_body");
        use_item(&mut w, "itm_glimmer_vial", 1);
        w.state_hash()
    };
    assert_eq!(run(), run());
}
