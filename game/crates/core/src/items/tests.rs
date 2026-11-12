use lf_content::Content;

use super::*;
use crate::actor::Actor;
use crate::input::{ActionFrame, Command, TickInput, buttons};
use crate::replay::{Replay, run_replay};
use crate::tick::tick;
use crate::world::World;

fn flat() -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(5, &content, "m0_flat").expect("m0_flat exists")
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

fn item(w: &World, id: &str) -> u32 {
    w.items.num_of(id).unwrap()
}

fn rules(w: &World) -> FlaskRules {
    w.items.flask.clone().unwrap()
}

/// Starts a drink on this tick (frame 0) and returns after it.
fn start_drink(w: &mut World) {
    tick_with(w, buttons::USE_ITEM, Vec::new());
    let c = player(w).moves.current.expect("the drink started");
    assert_eq!(c.num, rules(w).drink_move);
    assert_eq!(c.frame, 0);
}

#[test]
fn the_starting_loadout() {
    let w = flat();
    let inv = &player(&w).inventory;
    assert_eq!(inv.equipment.right_weapon(), w.items.default_weapon);
    assert_eq!(
        inv.equipment.selected_quick(),
        Some(item(&w, "itm_vigil_flask"))
    );
    // 4 charges, split 3 Vigil and 1 Lucid.
    assert_eq!(inv.flasks.max_charges, 4);
    assert_eq!((inv.flasks.vigil, inv.flasks.lucid), (3, 1));
    assert_eq!(inv.flasks.level, 0);
    // The active moveset is the equipped weapon's.
    let weapon = inv.equipment.right_weapon().unwrap();
    assert_eq!(w.moves.player_moveset.as_ref(), w.moves.moveset_of(weapon));
}

#[test]
fn drinking_heals_on_frame_16_and_spends_the_charge_then() {
    let mut w = flat();
    player_mut(&mut w).hp = 1;
    start_drink(&mut w);
    // Frames 1 to 15: nothing yet.
    for f in 1..16 {
        idle(&mut w, 1);
        assert_eq!(player(&w).moves.current.unwrap().frame, f);
        assert_eq!(player(&w).hp, 1, "no heal on frame {f}");
        assert_eq!(player(&w).inventory.flasks.vigil, 3, "no charge spent yet");
    }
    // Frame 16: +300 HP at +0, and the charge goes.
    idle(&mut w, 1);
    assert_eq!(player(&w).moves.current.unwrap().frame, 16);
    assert_eq!(player(&w).hp, 301.min(player(&w).max_hp));
    assert_eq!(player(&w).inventory.flasks.vigil, 2);
    // Recovery to frame 30 doesn't heal again.
    let hp = player(&w).hp;
    idle(&mut w, 20);
    assert!(player(&w).moves.current.is_none());
    assert_eq!(player(&w).hp, hp);
    assert_eq!(player(&w).inventory.flasks.vigil, 2);
}

#[test]
fn a_stagger_before_frame_16_means_no_heal_and_no_charge() {
    let mut w = flat();
    player_mut(&mut w).hp = 1;
    start_drink(&mut w);
    idle(&mut w, 14); // frame 14
    idle(&mut w, 1); // frame 15
    assert_eq!(player(&w).moves.current.unwrap().frame, 15);
    // A guard break (or any stagger) calls `interrupt`; the stagger blocks new moves.
    player_mut(&mut w).moves.interrupt();
    player_mut(&mut w).moves.guard.stagger = 30;
    idle(&mut w, 40);
    assert_eq!(player(&w).hp, 1);
    assert_eq!(player(&w).inventory.flasks.vigil, 3);
    assert_eq!(player(&w).inventory.flasks.lucid, 1);
}

#[test]
fn a_stagger_after_frame_16_keeps_the_heal() {
    let mut w = flat();
    player_mut(&mut w).hp = 1;
    start_drink(&mut w);
    idle(&mut w, 16);
    player_mut(&mut w).moves.interrupt();
    idle(&mut w, 5);
    assert_eq!(player(&w).hp, 301.min(player(&w).max_hp));
    assert_eq!(player(&w).inventory.flasks.vigil, 2);
}

#[test]
fn a_flask_heals_no_more_than_max_hp() {
    let mut w = flat();
    let max = player(&w).max_hp;
    player_mut(&mut w).hp = max - 5;
    start_drink(&mut w);
    idle(&mut w, 16);
    assert_eq!(player(&w).hp, max);
}

#[test]
fn cycling_to_the_lucid_flask_restores_fp() {
    let mut w = flat();
    player_mut(&mut w).fp = 0.0;
    // One press of CYCLE_ITEM held for several ticks is one cycle.
    for _ in 0..4 {
        tick_with(&mut w, buttons::CYCLE_ITEM, Vec::new());
    }
    assert_eq!(player(&w).inventory.equipment.quick_active, 1);
    idle(&mut w, 1);
    start_drink(&mut w);
    idle(&mut w, 15);
    assert_eq!(player(&w).fp, 0.0);
    idle(&mut w, 1);
    assert_eq!(player(&w).fp, 60.0f32.min(player(&w).max_fp));
    assert_eq!(player(&w).inventory.flasks.lucid, 0);
    assert_eq!(player(&w).inventory.flasks.vigil, 3);
}

#[test]
fn an_empty_flask_does_not_start() {
    let mut w = flat();
    player_mut(&mut w).inventory.flasks.vigil = 0;
    tick_with(&mut w, buttons::USE_ITEM, Vec::new());
    assert!(player(&w).moves.current.is_none());
}

#[test]
fn a_press_during_a_drink_does_not_restart_it() {
    let mut w = flat();
    start_drink(&mut w);
    idle(&mut w, 5);
    tick_with(&mut w, buttons::USE_ITEM, Vec::new());
    assert_eq!(player(&w).moves.current.unwrap().frame, 6);
    // The buffered press expires before the drink ends (no cancel window): exactly one
    // charge goes by the time everything settles.
    idle(&mut w, 60);
    assert_eq!(player(&w).inventory.flasks.vigil, 2);
}

#[test]
fn drinking_walks_at_40_percent_and_cannot_jump_or_sprint() {
    let mut w = flat();
    let walk = w.locomotion.walk_speed;
    start_drink(&mut w);
    let before = player(&w).pos;
    let full = ActionFrame {
        move_x: 0,
        move_z: 127,
        buttons: buttons::DODGE | buttons::JUMP,
    };
    for _ in 0..10 {
        tick(
            &mut w,
            &TickInput {
                frame: full,
                commands: Vec::new(),
            },
        );
        let p = player(&w);
        let speed = crate::math::dmath::sqrt(p.vel[0] * p.vel[0] + p.vel[2] * p.vel[2]);
        assert!(speed <= walk * 0.4 + 1e-4, "speed {speed}");
        assert!(!p.motor.sprinting);
        assert_eq!(p.motor.jump_startup, 0);
    }
    let moved = player(&w).pos[2] - before[2];
    assert!(moved > 0.0, "the player walks while drinking: {moved}");
    assert!(moved < walk * 0.4 * 10.0 / 30.0 + 0.05, "{moved}");
}

#[test]
fn use_item_command_selects_the_slot_and_drinks() {
    let mut w = flat();
    player_mut(&mut w).fp = 0.0;
    tick_with(&mut w, 0, vec![Command::UseItem { slot: 1 }]);
    assert_eq!(player(&w).moves.current.unwrap().num, rules(&w).drink_move);
    assert_eq!(player(&w).inventory.equipment.quick_active, 1);
    idle(&mut w, 16);
    assert!(player(&w).fp > 0.0);
    // An empty or missing slot does nothing.
    let mut w = flat();
    tick_with(&mut w, 0, vec![Command::UseItem { slot: 4 }]);
    tick_with(&mut w, 0, vec![Command::UseItem { slot: 9 }]);
    assert!(player(&w).moves.current.is_none());
}

#[test]
fn pickups() {
    let mut w = flat();
    let bead = item(&w, "itm_wick_bead");
    let dew = item(&w, "itm_flask_dew");
    let bomb = item(&w, "itm_pitch_bomb");
    let r = rules(&w);
    let items = w.items.clone();
    let inv = &mut player_mut(&mut w).inventory;
    // Six Wick Beads take 4 charges to 10; a seventh stays in the world.
    for _ in 0..6 {
        assert_eq!(inv.pick_up(bead, 1, &items), 1);
    }
    assert_eq!(inv.pick_up(bead, 1, &items), 0);
    assert_eq!(inv.flasks.max_charges, r.max_charges);
    assert_eq!((inv.flasks.vigil, inv.flasks.lucid), (9, 1));
    // Six Flask Dew take both flasks to +6.
    assert_eq!(inv.pick_up(dew, 9, &items), 6);
    assert_eq!(inv.flasks.level, r.max_level);
    assert_eq!(inv.heal_amount(&r), 600);
    assert_eq!(inv.fp_amount(&r), 120);
    // Stacks stop at their limit; the rest stays.
    assert_eq!(inv.pick_up(bomb, 7, &items), 7);
    assert_eq!(inv.pick_up(bomb, 7, &items), 3);
    assert_eq!(inv.count(bomb), 10);
    // The flasks aren't pickups.
    assert_eq!(inv.pick_up(r.vigil, 1, &items), 0);
    // Using a stack removes from it.
    assert_eq!(inv.remove(bomb, 4), 4);
    assert_eq!(inv.count(bomb), 6);
    assert_eq!(inv.remove(bomb, 40), 6);
    assert_eq!(inv.count(bomb), 0);
}

#[test]
fn the_split_and_lantern_refill() {
    let mut w = flat();
    let inv = &mut player_mut(&mut w).inventory;
    inv.flasks.vigil = 0;
    inv.flasks.lucid = 0;
    assert!(inv.split_flasks(1));
    assert_eq!((inv.flasks.vigil, inv.flasks.lucid), (1, 3));
    assert!(inv.split_flasks(4));
    assert_eq!((inv.flasks.vigil, inv.flasks.lucid), (4, 0));
    assert!(!inv.split_flasks(5), "more than the charges owned");
    assert_eq!(inv.flasks.vigil_max, 4);
    inv.flasks.vigil = 1;
    inv.refill_flasks();
    assert_eq!(inv.flasks.vigil, 4);
}

#[test]
fn equip_rules() {
    let mut w = flat();
    let bomb = item(&w, "itm_pitch_bomb");
    let arrows = item(&w, "itm_arrows");
    let bead = item(&w, "itm_wick_bead");
    let weapon = w.items.default_weapon.unwrap();
    let lucid = item(&w, "itm_lucid_flask");
    let items = w.items.clone();
    let inv = &mut player_mut(&mut w).inventory;
    // Not owned: refused.
    assert!(!inv.equip(slot::QUICK_1 + 2, bomb, &items));
    assert!(!inv.equip(slot::ARROWS, arrows, &items));
    inv.pick_up(bomb, 3, &items);
    inv.pick_up(arrows, 20, &items);
    assert!(inv.equip(slot::QUICK_1 + 2, bomb, &items));
    assert!(inv.equip(slot::ARROWS, arrows, &items));
    // A bomb isn't ammunition, and a pickup-only item can't be a quick item.
    assert!(!inv.equip(slot::ARROWS, bomb, &items));
    assert!(!inv.equip(slot::QUICK_1 + 3, bead, &items));
    // Weapons go in hands only; a flask doesn't go in a hand.
    assert!(inv.equip(slot::RIGHT_2, weapon, &items));
    assert!(inv.equip(slot::LEFT_1, weapon, &items));
    assert!(!inv.equip(slot::QUICK_1 + 3, weapon, &items));
    assert!(!inv.equip(slot::RIGHT_2, lucid, &items));
    assert!(!inv.equip(slot::HEAD, weapon, &items));
    // The charm slots beyond the first two are closed; slot numbers past the end don't exist.
    assert!(!inv.equip(slot::CHARM_1 + 2, weapon, &items));
    assert!(!inv.equip(16, weapon, &items));
    // Unequip moves the selection off the emptied slot.
    inv.equipment.quick_active = 2;
    assert!(inv.unequip(slot::QUICK_1 + 2));
    assert!(!inv.unequip(slot::QUICK_1 + 2));
    assert_ne!(inv.equipment.quick_active, 2);
}

#[test]
fn equip_commands_change_the_loadout_and_the_moveset() {
    let mut w = flat();
    let weapon = w.items.default_weapon.unwrap();
    // Weapon in the second right-hand slot, then cycle to it.
    tick_with(
        &mut w,
        0,
        vec![Command::Equip {
            slot: slot::RIGHT_2,
            item: weapon,
        }],
    );
    assert_eq!(
        player(&w).inventory.equipment.slots[usize::from(slot::RIGHT_2)],
        Some(weapon)
    );
    tick_with(&mut w, buttons::CYCLE_RIGHT, Vec::new());
    assert_eq!(player(&w).inventory.equipment.right_active, 1);
    // Unequipping the Lucid Flask leaves the selection on a filled slot.
    tick_with(
        &mut w,
        0,
        vec![Command::Unequip {
            slot: slot::QUICK_1,
        }],
    );
    assert_eq!(player(&w).inventory.equipment.quick_active, 1);
    // A hand doesn't cycle in the middle of a move.
    idle(&mut w, 2);
    tick_with(&mut w, buttons::LIGHT, Vec::new());
    assert!(player(&w).moves.current.is_some());
    tick_with(&mut w, buttons::LIGHT | buttons::CYCLE_RIGHT, Vec::new());
    assert_eq!(player(&w).inventory.equipment.right_active, 1);
}

/// The equip commands replay: a recording with commands re-simulates to the recorded
/// hashes, and the commands change the state.
#[test]
fn equip_commands_replay_to_the_same_state_hash() {
    let content = Content::load_embedded().unwrap();
    let probe = World::new(5, &content, "m0_flat").unwrap();
    let weapon = probe.items.default_weapon.unwrap();
    let frame = |buttons| ActionFrame {
        move_x: 0,
        move_z: 0,
        buttons,
    };
    let mut frames = vec![frame(0); 120];
    frames[40] = frame(buttons::USE_ITEM);
    frames[80] = frame(buttons::CYCLE_ITEM);
    frames[82] = frame(buttons::USE_ITEM);
    let commands = vec![
        (
            5,
            Command::Equip {
                slot: slot::RIGHT_2,
                item: weapon,
            },
        ),
        (6, Command::SplitFlasks { vigil: 2 }),
        (
            10,
            Command::Unequip {
                slot: slot::QUICK_1 + 1,
            },
        ),
        (
            11,
            Command::Equip {
                slot: slot::QUICK_1 + 3,
                item: probe.items.flask.as_ref().unwrap().lucid,
            },
        ),
        (20, Command::UseItem { slot: 3 }),
    ];
    let r = Replay::record(5, "m0_flat", &content, &frames, commands.clone()).unwrap();
    let decoded = Replay::decode(&r.encode()).unwrap();
    let report = run_replay(&decoded, &content).expect("the replay re-simulates");
    assert_eq!(report.final_hash, r.hashes.last().unwrap().1);

    // The commands matter: without them the run ends in a different state.
    let without = Replay::record(5, "m0_flat", &content, &frames, Vec::new()).unwrap();
    assert_ne!(without.hashes.last(), r.hashes.last());

    // And the commands did what they say.
    let mut w = World::new(5, &content, "m0_flat").unwrap();
    for (t, f) in frames.iter().enumerate() {
        let cmds = commands
            .iter()
            .filter(|(ct, _)| *ct == t as u64)
            .map(|(_, c)| c.clone())
            .collect();
        tick(
            &mut w,
            &TickInput {
                frame: *f,
                commands: cmds,
            },
        );
    }
    let inv = &player(&w).inventory;
    // `m0_flat` has no Lantern, so the split is refused (see `world_state` tests for the
    // split while resting).
    assert_eq!(inv.flasks.vigil_max, rules(&w).start_vigil);
    assert_eq!(
        inv.equipment.slots[usize::from(slot::RIGHT_2)],
        Some(weapon)
    );
    assert_eq!(inv.equipment.slots[usize::from(slot::QUICK_1) + 1], None);
    // One drink: the Lucid Flask from slot 3 at tick 20. The press at tick 40 lands inside
    // that drink and expires unused.
    assert_eq!(inv.flasks.lucid, inv.flasks.lucid_max() - 1);
}
