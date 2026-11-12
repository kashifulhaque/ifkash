//! Tests for the spell rules: attunement and slots, spell power, cast speed, casting input,
//! body buffs, and the generic effects and channels a Litany uses. The golden replays of the
//! nine Formulae are asserted in `game/crates/tools/tests/golden_spells.rs`.

use lf_content::moves::{Move, StatusKind};
use lf_content::spells::{Channel, SpellEffect, SpellSchool, TimedEffect};
use lf_content::status::{BuffDef, BuffSlot};
use lf_content::{Content, NumId, SpellDef};

use super::*;
use crate::combat::moves::MoveTable;
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::items::slot;
use crate::tick::tick;
use crate::world::World;

/// The spell range with the player's attributes set directly, so each test says what it
/// needs. Insight 30 and Focus 10 (2 slots) unless a test changes them.
fn range() -> World {
    let content = Content::load_embedded().expect("embedded content");
    from(&content)
}

fn from(content: &Content) -> World {
    let mut w = World::new(7, content, "spell_range").expect("spell_range exists");
    let id = w.player;
    let a = w.actors.get_mut(id).unwrap();
    a.stats.attrs.insight = 30;
    a.stats.attrs.devotion = 30;
    w
}

fn player(w: &World) -> &Actor {
    w.actors.get(w.player).unwrap()
}

fn player_mut(w: &mut World) -> &mut Actor {
    let id = w.player;
    w.actors.get_mut(id).unwrap()
}

fn spell(w: &World, id: &str) -> NumId {
    w.spells_table
        .num_of(id)
        .unwrap_or_else(|| panic!("no spell {id}"))
}

fn reg(id: &str) -> NumId {
    Content::load_embedded().unwrap().registry.num(id).unwrap()
}

fn try_attune(w: &mut World, index: u8, id: &str) -> bool {
    let n = spell(w, id);
    attune(w, index, n)
}

fn step(w: &mut World, buttons: u16, commands: Vec<Command>) {
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

fn idle(w: &mut World, n: u32) {
    for _ in 0..n {
        step(w, 0, Vec::new());
    }
}

fn equip(w: &mut World, index: u8, id: &str) {
    let item = spell(w, id);
    step(
        w,
        0,
        vec![Command::Equip {
            slot: slot::SPELL_1 + index,
            item,
        }],
    );
}

fn current_move(w: &World) -> Option<String> {
    player(w)
        .moves
        .current
        .and_then(|c| w.moves.get(c.num))
        .map(|d| d.data.id.clone())
}

#[test]
fn attuning_takes_a_known_spell_through_the_equip_command() {
    let mut w = range();
    assert!(w.spells.is_inert());
    equip(&mut w, 0, "spl_aether_dart");
    assert_eq!(w.spells.attuned[0], Some(spell(&w, "spl_aether_dart")));
    assert_eq!(w.spells.selected_spell(), w.spells.attuned[0]);
    assert!(!w.spells.is_inert());
    // The equipment slots are untouched.
    assert!(player(&w).inventory.equipment.slots[slot::COUNT - 1].is_none());
}

#[test]
fn an_unknown_spell_does_not_attune() {
    let content = Content::load_embedded().unwrap();
    let mut w = World::new(7, &content, "m0_flat").unwrap();
    player_mut(&mut w).stats.attrs.insight = 30;
    let dart = spell(&w, "spl_aether_dart");
    // m0_flat teaches nothing: the spell isn't in the gear.
    assert!(!attune(&mut w, 0, dart));
    player_mut(&mut w).inventory.give_gear(dart);
    assert!(attune(&mut w, 0, dart));
}

#[test]
fn requirements_gate_attunement() {
    let mut w = range();
    player_mut(&mut w).stats.attrs.insight = 9;
    let dart = spell(&w, "spl_aether_dart");
    assert!(!attune(&mut w, 0, dart), "Aether Dart needs Insight 10");
    player_mut(&mut w).stats.attrs.insight = 10;
    assert!(attune(&mut w, 0, dart));
    // Glasswing Swarm needs Insight 18 and Finesse 12 (the stub player has 12).
    let swarm = spell(&w, "spl_glasswing_swarm");
    player_mut(&mut w).stats.attrs.insight = 17;
    assert!(!attune(&mut w, 1, swarm));
    player_mut(&mut w).stats.attrs.insight = 18;
    player_mut(&mut w).stats.attrs.finesse = 11;
    assert!(!attune(&mut w, 1, swarm));
    player_mut(&mut w).stats.attrs.finesse = 12;
    assert!(attune(&mut w, 1, swarm));
}

#[test]
fn slots_come_from_focus_and_a_spell_takes_one_or_two() {
    let mut w = range();
    // Focus 10: 2 slots. A two-slot spell fills them.
    let lance = spell(&w, "spl_unbinding_lance");
    let dart = spell(&w, "spl_aether_dart");
    assert!(attune(&mut w, 0, lance));
    assert!(!attune(&mut w, 1, dart), "no slot left");
    assert!(unattune(&mut w, 0));
    assert!(attune(&mut w, 0, dart));
    assert!(try_attune(&mut w, 1, "spl_aether_bolt"));
    assert!(
        !try_attune(&mut w, 2, "spl_veil"),
        "two one-slot spells fill 2 slots"
    );
    // Focus 18 gives 3 slots (2 at 10, +1 every 8 above): one more fits.
    player_mut(&mut w).stats.attrs.focus = 18;
    assert!(try_attune(&mut w, 2, "spl_veil"));
    assert!(!try_attune(&mut w, 3, "spl_mirror_ward"));
}

#[test]
fn replacing_a_spell_counts_the_slots_it_frees() {
    let mut w = range();
    let lance = spell(&w, "spl_unbinding_lance");
    assert!(try_attune(&mut w, 0, "spl_aether_dart"));
    // Position 0 holds a one-slot spell and one slot is free: a two-slot spell fits there.
    assert!(attune(&mut w, 0, lance));
    assert_eq!(w.spells.attuned[0], Some(lance));
}

#[test]
fn a_spell_attunes_to_one_position_only() {
    let mut w = range();
    let dart = spell(&w, "spl_aether_dart");
    assert!(attune(&mut w, 0, dart));
    assert!(!attune(&mut w, 1, dart));
    assert!(
        !attune(&mut w, slot::SPELL_COUNT, dart),
        "past the last position"
    );
}

#[test]
fn unequipping_frees_the_position_and_moves_the_selection() {
    let mut w = range();
    equip(&mut w, 0, "spl_aether_dart");
    equip(&mut w, 1, "spl_aether_bolt");
    assert_eq!(w.spells.selected, 0);
    step(
        &mut w,
        0,
        vec![Command::Unequip {
            slot: slot::SPELL_1,
        }],
    );
    assert_eq!(w.spells.attuned[0], None);
    assert_eq!(
        w.spells.selected, 1,
        "the selection moved to the spell that is left"
    );
    assert!(!unattune(&mut w, 5), "an empty position");
}

#[test]
fn cycle_spell_selects_the_next_attuned_spell_and_points_the_moveset_at_it() {
    let mut w = range();
    equip(&mut w, 0, "spl_aether_dart");
    equip(&mut w, 1, "spl_aether_bolt");
    let set = |w: &World| w.moves.player_moveset.as_ref().unwrap().light_chain.clone();
    let cast = |w: &World, id: &str| vec![w.moves.num_of(id).unwrap()];
    assert_eq!(set(&w), cast(&w, "spm_aether_dart"));
    step(&mut w, buttons::CYCLE_SPELL, Vec::new());
    assert_eq!(w.spells.selected, 1);
    assert_eq!(set(&w), cast(&w, "spm_aether_bolt"));
    // Holding the button doesn't cycle again; a new press wraps around.
    step(&mut w, buttons::CYCLE_SPELL, Vec::new());
    assert_eq!(w.spells.selected, 1);
    step(&mut w, 0, Vec::new());
    step(&mut w, buttons::CYCLE_SPELL, Vec::new());
    assert_eq!(w.spells.selected, 0);
    assert_eq!(set(&w), cast(&w, "spm_aether_dart"));
}

#[test]
fn cycling_with_nothing_attuned_changes_nothing() {
    let mut w = range();
    step(&mut w, buttons::CYCLE_SPELL, Vec::new());
    assert!(w.spells.is_inert());
}

#[test]
fn light_casts_the_spell_and_heavy_its_charged_form() {
    let mut w = range();
    equip(&mut w, 0, "spl_unbinding_lance");
    idle(&mut w, 5);
    step(&mut w, buttons::LIGHT, Vec::new());
    assert_eq!(current_move(&w).as_deref(), Some("spm_unbinding_lance"));
    idle(&mut w, 80);
    assert!(current_move(&w).is_none());
    step(&mut w, buttons::HEAVY, Vec::new());
    assert_eq!(
        current_move(&w).as_deref(),
        Some("spm_unbinding_lance_charged")
    );
}

#[test]
fn heavy_casts_the_plain_spell_when_it_has_no_charged_form() {
    let mut w = range();
    equip(&mut w, 0, "spl_aether_dart");
    idle(&mut w, 5);
    step(&mut w, buttons::HEAVY, Vec::new());
    assert_eq!(current_move(&w).as_deref(), Some("spm_aether_dart"));
}

#[test]
fn a_cast_costs_its_fp_and_needs_it_in_full() {
    let mut w = range();
    equip(&mut w, 0, "spl_aether_dart");
    idle(&mut w, 5);
    player_mut(&mut w).fp = 4.0;
    step(&mut w, buttons::LIGHT, Vec::new());
    assert!(current_move(&w).is_none(), "5 FP needed, 4 held");
    assert_eq!(player(&w).fp, 4.0);
    idle(&mut w, 12);
    player_mut(&mut w).fp = 20.0;
    step(&mut w, buttons::LIGHT, Vec::new());
    assert_eq!(current_move(&w).as_deref(), Some("spm_aether_dart"));
    assert_eq!(player(&w).fp, 15.0);
}

#[test]
fn a_spell_of_the_other_school_is_not_cast_by_this_catalyst() {
    let mut w = range();
    equip(&mut w, 0, "spl_aether_dart");
    // A stylus casts Formulae. Swap in the reliquary (a Litany catalyst) and the dart is
    // not cast: LIGHT is the catalyst's own swing.
    let id = w.moves.num_of("ss_light_1").unwrap();
    let rel = reg("wpn_pilgrims_reliquary");
    let p = player_mut(&mut w);
    p.inventory.give_gear(rel);
    p.inventory.equipment.slots[usize::from(slot::RIGHT_1)] = Some(rel);
    w.sync_player_moveset();
    idle(&mut w, 5);
    step(&mut w, buttons::LIGHT, Vec::new());
    assert_eq!(player(&w).moves.current.map(|c| c.num), Some(id));
}

#[test]
fn a_left_hand_catalyst_casts_on_guard_and_cannot_guard() {
    let mut w = range();
    equip(&mut w, 0, "spl_aether_dart");
    let sword = reg("wpn_wayfarer_blade");
    let stylus = reg("wpn_scholars_stylus");
    let p = player_mut(&mut w);
    p.inventory.give_gear(sword);
    p.inventory.equipment.slots[usize::from(slot::RIGHT_1)] = Some(sword);
    p.inventory.equipment.slots[usize::from(slot::LEFT_1)] = Some(stylus);
    w.sync_player_moveset();
    // LIGHT swings the sword; the stylus in the left hand holds the cast on GUARD.
    assert!(w.moves.player_moveset.as_ref().unwrap().left_cast.is_some());
    idle(&mut w, 5);
    step(&mut w, buttons::GUARD, Vec::new());
    assert_eq!(current_move(&w).as_deref(), Some("spm_aether_dart"));
    assert!(!player(&w).moves.guard.active, "a catalyst can't guard");
    idle(&mut w, 40);
    step(&mut w, buttons::LIGHT, Vec::new());
    assert_eq!(current_move(&w).as_deref(), Some("ss_light_1"));
}

#[test]
fn spell_power_is_the_catalysts_attack_rating_as_every_element() {
    let w = range();
    let stylus = reg("wpn_scholars_stylus");
    let a = player(&w);
    let want = w
        .damage
        .attack_rating(stylus, &a.stats.attrs, a.stats.weapon_upgrade, false)
        .unwrap();
    let dart = w.moves.num_of("spm_aether_dart").unwrap();
    let got = power(&w, a, dart).expect("a spell move has spell power");
    assert_eq!(got.phys_milli, 0);
    assert_eq!(got.elem_milli, [want.phys_milli; 4]);
    // Insight scales it: 100 percent of the curve at Insight 30 beats Insight 10.
    let mut weak = player(&w).clone();
    weak.stats.attrs.insight = 10;
    let low = power(&w, &weak, dart).unwrap();
    assert!(got.elem_milli[0] > low.elem_milli[0]);
    // A move that isn't a spell has none.
    let slash = w.moves.num_of("ss_light_1").unwrap();
    assert!(power(&w, a, slash).is_none());
}

#[test]
fn finesse_shortens_cast_startup_by_up_to_a_quarter_at_60() {
    let w = range();
    let t: &MoveTable = &w.moves;
    let lance = t
        .get(t.num_of("spm_unbinding_lance_charged").unwrap())
        .unwrap();
    assert_eq!(lance.cast_startup, 42, "the first frame that does anything");
    // 25% at 60 Finesse; a share of that below it; none at 0.
    assert_eq!(t.cast_skip(lance, 60), 10);
    assert_eq!(t.cast_skip(lance, 99), 10, "no more than 25 percent");
    assert_eq!(t.cast_skip(lance, 30), 5);
    assert_eq!(t.cast_skip(lance, 0), 0);
    // A move that isn't a spell skips nothing.
    let slash = t.get(t.num_of("ss_light_1").unwrap()).unwrap();
    assert_eq!(t.cast_skip(slash, 60), 0);
}

#[test]
fn a_cast_starts_on_the_frame_finesse_skips_to() {
    let mut w = range();
    equip(&mut w, 0, "spl_unbinding_lance");
    player_mut(&mut w).stats.attrs.finesse = 60;
    idle(&mut w, 5);
    step(&mut w, buttons::HEAVY, Vec::new());
    // The charged lance's first frame that does anything is 42; a quarter is 10.
    assert_eq!(player(&w).moves.current.unwrap().frame, 10);
    // The lance leaves on frame 42, 32 ticks later instead of 42.
    for _ in 0..31 {
        idle(&mut w, 1);
        assert_eq!(w.projectiles.iter().count(), 0);
    }
    idle(&mut w, 1);
    assert_eq!(w.projectiles.iter().count(), 1);
}

#[test]
fn mirror_ward_takes_40_percent_off_three_hits_then_ends() {
    let mut w = range();
    let buff = reg("spb_mirror_ward");
    let id = w.player;
    let a = w.actors.get_mut(id).unwrap();
    assert!(status::apply_buff(a, &w.status, buff));
    assert_eq!(a.status.body_buff.unwrap().hits, 3);
    for expect in [60, 60, 60, 100] {
        let a = w.actors.get_mut(id).unwrap();
        assert_eq!(status::incoming_damage(a, &w.status, 100), expect);
    }
    assert!(
        player(&w).status.body_buff.is_none(),
        "the ward ended with the third hit"
    );
}

#[test]
fn a_hit_that_deals_nothing_does_not_use_up_the_ward() {
    let mut w = range();
    let buff = reg("spb_mirror_ward");
    let id = w.player;
    let a = w.actors.get_mut(id).unwrap();
    status::apply_buff(a, &w.status, buff);
    assert_eq!(status::incoming_damage(a, &w.status, 0), 0);
    assert_eq!(a.status.body_buff.unwrap().hits, 3);
}

#[test]
fn veil_halves_the_range_enemies_notice_the_player_at() {
    let mut w = range();
    let id = w.player;
    assert_eq!(status::notice_percent(player(&w), &w.status), 100);
    let veil = reg("spb_veil");
    let a = w.actors.get_mut(id).unwrap();
    status::apply_buff(a, &w.status, veil);
    assert_eq!(status::notice_percent(player(&w), &w.status), 50);
    // A second body buff replaces it.
    let ward = reg("spb_mirror_ward");
    let a = w.actors.get_mut(id).unwrap();
    status::apply_buff(a, &w.status, ward);
    assert_eq!(status::notice_percent(player(&w), &w.status), 100);
}

#[test]
fn an_attuned_world_hashes_differently_and_the_same_way_twice() {
    let mut a = range();
    let mut b = range();
    let before = a.state_hash();
    assert_eq!(before, b.state_hash());
    equip(&mut a, 0, "spl_aether_dart");
    equip(&mut b, 0, "spl_aether_dart");
    idle(&mut a, 1);
    idle(&mut b, 1);
    assert_eq!(a.state_hash(), b.state_hash());
    let mut c = range();
    idle(&mut c, 2);
    assert_ne!(a.state_hash(), c.state_hash());
}

// ---------------------------------------------------------------------------------------
// What a Litany needs, built in the test from data alone: no Rust beyond this file knows
// about any of these spells.

fn move_of(id: &str, total: u16, fp: u16) -> Move {
    let mut m = Content::load_embedded()
        .unwrap()
        .move_by_id("spm_aether_dart")
        .unwrap()
        .clone();
    m.id = id.to_owned();
    m.total_frames = total;
    m.fp = fp;
    m.projectiles.clear();
    m.cancel_attack_from = Some(total - 2);
    m.cancel_any_from = Some(total - 2);
    m.cancel_dodge_from = Some(total - 4);
    m
}

fn litany(id: &str, cast: Move) -> SpellDef {
    let dart = Content::load_embedded()
        .unwrap()
        .spell_by_id("spl_aether_dart")
        .unwrap()
        .clone();
    SpellDef {
        id: id.to_owned(),
        name: id.to_owned(),
        school: SpellSchool::Litany,
        cast,
        projectiles: Vec::new(),
        buffs: Vec::new(),
        effects: Vec::new(),
        requirements: lf_content::weapons::Requirements {
            might: 0,
            finesse: 0,
            insight: 0,
            devotion: 10,
        },
        ..dart
    }
}

/// The embedded content plus `spells`, each registered with the given numbers, and the
/// reliquary equipped in a world that knows them.
fn litany_world(spells: Vec<(SpellDef, NumId)>) -> World {
    let mut content = Content::load_embedded().unwrap();
    let mut next = 15500;
    for (mut s, num) in spells {
        content.registry.ids.insert(s.id.clone(), num);
        for m in std::iter::once(&s.cast).chain(s.charged.as_ref()) {
            next += 1;
            content.registry.ids.insert(m.id.clone(), next);
        }
        for p in &s.projectiles {
            next += 1;
            content.registry.ids.insert(p.id.clone(), next);
        }
        for b in &s.buffs {
            next += 1;
            content.registry.ids.insert(b.id.clone(), next);
        }
        content.moves.insert(format!("spells/test_{}.ron", s.id), {
            let mut v = vec![s.cast.clone()];
            v.extend(s.charged.clone());
            v
        });
        content.projectiles.extend(s.projectiles.iter().cloned());
        content
            .status
            .as_mut()
            .unwrap()
            .buffs
            .extend(s.buffs.iter().cloned());
        content.spells.push(std::mem::replace(
            &mut s,
            litany("unused", move_of("unused", 10, 0)),
        ));
    }
    let mut w = from(&content);
    let reliquary = content.registry.num("wpn_pilgrims_reliquary").unwrap();
    let p = player_mut(&mut w);
    p.inventory.give_gear(reliquary);
    p.inventory.equipment.slots[usize::from(slot::RIGHT_1)] = Some(reliquary);
    p.stats.attrs.focus = 40;
    w.sync_player_moveset();
    w
}

fn learn(w: &mut World, id: &str, index: u8) {
    let n = spell(w, id);
    player_mut(w).inventory.give_gear(n);
    assert!(attune(w, index, n), "{id} attunes");
}

#[test]
fn a_heal_spell_restores_a_share_of_max_hp_on_its_frame() {
    let mut s = litany("spl_test_mend", move_of("spm_test_mend", 30, 18));
    s.effects.push(TimedEffect {
        frame: 10,
        effect: SpellEffect::Heal {
            max_hp_percent: 35,
            radius_m: 0.0,
        },
    });
    let mut w = litany_world(vec![(s, 15520)]);
    learn(&mut w, "spl_test_mend", 0);
    player_mut(&mut w).hp = 10;
    let max = player(&w).max_hp;
    idle(&mut w, 3);
    step(&mut w, buttons::LIGHT, Vec::new());
    for _ in 0..9 {
        idle(&mut w, 1);
        assert_eq!(player(&w).hp, 10, "not before frame 10");
    }
    idle(&mut w, 1);
    assert_eq!(player(&w).hp, 10 + max * 35 / 100);
    idle(&mut w, 25);
    assert_eq!(player(&w).hp, 10 + max * 35 / 100, "healed once");
}

#[test]
fn a_cure_spell_clears_named_statuses_or_all_of_them() {
    let mut s = litany("spl_test_purge", move_of("spm_test_purge", 30, 12));
    s.effects.push(TimedEffect {
        frame: 8,
        effect: SpellEffect::Cure { statuses: vec![] },
    });
    let mut one = litany("spl_test_clot", move_of("spm_test_clot", 30, 12));
    one.effects.push(TimedEffect {
        frame: 8,
        effect: SpellEffect::Cure {
            statuses: vec![StatusKind::Poison],
        },
    });
    let mut w = litany_world(vec![(s, 15521), (one, 15522)]);
    learn(&mut w, "spl_test_clot", 0);
    learn(&mut w, "spl_test_purge", 1);
    let id = w.player;
    let hurt = |w: &mut World| {
        let a = w.actors.get_mut(id).unwrap();
        a.status.meter_milli = [5000; status::COUNT];
        a.status.active[status::index(StatusKind::Poison)] = 100;
        a.status.active[status::index(StatusKind::Blight)] = 100;
    };
    hurt(&mut w);
    idle(&mut w, 3);
    step(&mut w, buttons::LIGHT, Vec::new()); // the first spell cures Poison only
    idle(&mut w, 12);
    let a = player(&w);
    assert_eq!(a.status.meter_milli[status::index(StatusKind::Poison)], 0);
    assert_eq!(a.status.active[status::index(StatusKind::Poison)], 0);
    assert!(
        a.status.active[status::index(StatusKind::Blight)] > 0,
        "Blight is not cured"
    );
    idle(&mut w, 30);
    step(&mut w, buttons::CYCLE_SPELL, Vec::new());
    idle(&mut w, 1);
    step(&mut w, buttons::LIGHT, Vec::new()); // the second cures everything
    idle(&mut w, 12);
    let a = player(&w);
    assert_eq!(a.status.meter_milli, [0; status::COUNT]);
    assert_eq!(a.status.active[status::index(StatusKind::Blight)], 0);
}

#[test]
fn a_buff_spell_applies_its_own_buff_in_its_slot() {
    let mut s = litany("spl_test_edge", move_of("spm_test_edge", 30, 14));
    s.buffs.push(BuffDef {
        id: "spb_test_edge".to_owned(),
        slot: BuffSlot::Weapon,
        damage_percent: 120,
        status: vec![],
        duration_ticks: 600,
        vfx: "edge_test".to_owned(),
        element: None,
        absorb_percent: 0,
        absorb_hits: 0,
        notice_percent: 100,
    });
    s.effects.push(TimedEffect {
        frame: 8,
        effect: SpellEffect::Buff {
            buff: "spb_test_edge".to_owned(),
        },
    });
    let mut w = litany_world(vec![(s, 15523)]);
    learn(&mut w, "spl_test_edge", 0);
    idle(&mut w, 3);
    step(&mut w, buttons::LIGHT, Vec::new());
    idle(&mut w, 10);
    assert!(player(&w).status.weapon_buff.is_some());
    assert_eq!(player(&w).stats.buff_percent, 120);
}

#[test]
fn a_channeled_spell_launches_a_projectile_and_pays_each_interval_while_held() {
    let mut s = litany("spl_test_breath", {
        let mut m = move_of("spm_test_breath", 40, 0);
        m.charge = Some(lf_content::moves::Charge {
            charge_from: 4,
            max_frames: 120,
            motion_value_max: 1.0,
        });
        m
    });
    let mut flame = Content::load_embedded()
        .unwrap()
        .projectile_by_id("spp_glasswing_shard")
        .unwrap()
        .clone();
    flame.id = "spp_test_flame".to_owned();
    s.projectiles.push(flame);
    s.channel = Some(Channel {
        interval_ticks: 10,
        fp: 2,
        projectile: "spp_test_flame".to_owned(),
        offset: [0.0, 1.2, 0.8],
        yaw_deg: 0.0,
        pitch_deg: 0.0,
    });
    let mut w = litany_world(vec![(s, 15524)]);
    learn(&mut w, "spl_test_breath", 0);
    player_mut(&mut w).fp = 50.0;
    idle(&mut w, 3);
    // Hold LIGHT for 45 ticks: it holds the move at frame 4 and breathes every 10 ticks.
    for _ in 0..45 {
        step(&mut w, buttons::LIGHT, Vec::new());
    }
    let spent = 50.0 - player(&w).fp;
    assert!(
        (4.0..=10.0).contains(&spent) && spent % 2.0 == 0.0,
        "paid 2 FP per interval, got {spent}"
    );
    let launched = w.events.drain().iter().filter(|e| {
        matches!(e, crate::events::Event::Projectile(p) if p.kind == crate::events::ProjectileEventKind::Spawned)
    }).count();
    assert!(launched >= 2, "launched {launched} flames");
    // Letting go ends the channel: no more FP is paid.
    let fp = player(&w).fp;
    idle(&mut w, 60);
    assert_eq!(player(&w).fp, fp);
}

#[test]
fn a_channel_ends_when_fp_runs_out() {
    let mut s = litany("spl_test_stream", {
        let mut m = move_of("spm_test_stream", 40, 0);
        m.charge = Some(lf_content::moves::Charge {
            charge_from: 4,
            max_frames: 300,
            motion_value_max: 1.0,
        });
        m
    });
    let mut flame = Content::load_embedded()
        .unwrap()
        .projectile_by_id("spp_glasswing_shard")
        .unwrap()
        .clone();
    flame.id = "spp_test_stream".to_owned();
    s.projectiles.push(flame);
    s.channel = Some(Channel {
        interval_ticks: 5,
        fp: 3,
        projectile: "spp_test_stream".to_owned(),
        offset: [0.0, 1.2, 0.8],
        yaw_deg: 0.0,
        pitch_deg: 0.0,
    });
    let mut w = litany_world(vec![(s, 15525)]);
    learn(&mut w, "spl_test_stream", 0);
    player_mut(&mut w).fp = 7.0;
    idle(&mut w, 3);
    for _ in 0..120 {
        step(&mut w, buttons::LIGHT, Vec::new());
    }
    // 7 FP pays two intervals of 3; the third can't, and the move plays out.
    assert_eq!(player(&w).fp, 1.0);
    assert!(
        current_move(&w).is_none(),
        "still held with no FP: the move should have ended"
    );
}

#[test]
fn a_spell_file_validates_the_spell_it_defines() {
    let mut content = Content::load_embedded().unwrap();
    assert!(content.spells.iter().all(|s| s.problems().is_empty()));
    let mut bad = content.spell_by_id("spl_aether_dart").unwrap().clone();
    bad.slots = 3;
    bad.effects.push(TimedEffect {
        frame: 999,
        effect: SpellEffect::Buff {
            buff: "nope".to_owned(),
        },
    });
    bad.channel = Some(Channel {
        interval_ticks: 0,
        fp: 1,
        projectile: "nope".to_owned(),
        offset: [0.0; 3],
        yaw_deg: 0.0,
        pitch_deg: 0.0,
    });
    let fields: Vec<String> = bad.problems().into_iter().map(|p| p.0).collect();
    for want in [
        "slots",
        "effects[0].frame",
        "effects[0].effect",
        "channel",
        "channel.interval_ticks",
        "channel.projectile",
    ] {
        assert!(
            fields.iter().any(|f| f == want),
            "missing {want} in {fields:?}"
        );
    }
    content.spells.clear();
}
