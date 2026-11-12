//! Tests for Hollowhearth, its dialogue, and the questlines (WP-068).
//!
//! The acceptance test is `every_portfolio_entry_is_reachable_without_a_quest`: it opens each
//! NPC and station in a fresh hub, asks about every topic through `Command::Ask`, and checks
//! that the topics shown reach every entry of the exported portfolio while every quest is
//! still in state 0.

use std::collections::BTreeSet;

use lf_content::npc::{ALL_SLUGS, BACK_TOPIC, QUEST_TOPIC, StationKind};
use lf_content::{Content, PortfolioContent};

use super::super::hollow_tests::{Bot, Leg};
use super::*;
use crate::math::dmath::{cos, sin};
use crate::tick::tick;

const SEED: u64 = 68;

/// The portfolio export the dialogue is checked against.
const PORTFOLIO: &str = include_str!("../../../../../data/gen/portfolio_content.json");

fn hub() -> World {
    let content = Content::load_embedded().expect("embedded content");
    World::new(SEED, &content, "hollowhearth").expect("the hub exists")
}

fn idle(w: &mut World, n: u32) {
    for _ in 0..n {
        tick(w, &TickInput::default());
    }
}

fn command(w: &mut World, c: Command) {
    tick(
        w,
        &TickInput {
            frame: crate::input::ActionFrame::default(),
            commands: vec![c],
        },
    );
}

/// Presses and releases `INTERACT`.
fn interact(w: &mut World) {
    let press = TickInput {
        frame: crate::input::ActionFrame {
            move_x: 0,
            move_z: 0,
            buttons: buttons::INTERACT,
        },
        commands: vec![],
    };
    tick(w, &press);
    tick(w, &TickInput::default());
}

fn place(w: &mut World, pos: [f32; 3]) {
    let id = w.player;
    w.actors.get_mut(id).expect("player").pos = pos;
    idle(w, 4);
}

/// Where a player stands to speak to NPC `i`: in front of its counter.
fn stand_at_npc(w: &World, i: usize) -> [f32; 3] {
    let n = &w.layout.npcs[i];
    [
        n.pos[0] + sin(n.yaw) * 2.4,
        0.0,
        n.pos[2] + cos(n.yaw) * 2.4,
    ]
}

fn npc_index(w: &World, id: &str) -> usize {
    w.layout
        .npcs
        .iter()
        .position(|n| n.def.id == id)
        .unwrap_or_else(|| panic!("no NPC {id}"))
}

fn talk_to(w: &mut World, id: &str) -> usize {
    let i = npc_index(w, id);
    let at = stand_at_npc(w, i);
    place(w, at);
    interact(w);
    assert_eq!(current(w), Some(Talk::Npc(i)), "{id} did not open");
    i
}

fn events(w: &mut World) -> Vec<Event> {
    w.events.drain()
}

#[test]
fn the_hub_has_five_npcs_five_stations_and_is_a_safe_zone() {
    let w = hub();
    let ids: Vec<_> = w.layout.npcs.iter().map(|n| n.def.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "npc_archivist",
            "npc_cartographer",
            "npc_smith",
            "npc_courier",
            "npc_peddler"
        ]
    );
    assert_eq!(w.layout.stations.len(), 5);
    assert!(w.layout.safe_zone);
    assert_eq!(w.layout.lanterns.len(), 1);
    assert_eq!(w.layout.lanterns[0].id, "lantern_l00");
    // Four NPCs have a questline; the Peddler has none.
    let quests = w
        .layout
        .npcs
        .iter()
        .filter(|n| n.def.quest.is_some())
        .count();
    assert_eq!(quests, 4);
}

#[test]
fn a_bot_walks_to_every_npc_and_station_and_speaks() {
    // From the spawn, through the middle of the hollow, to the spot in front of each NPC's
    // counter and each station, then Interact: this also proves the layout is walkable.
    let probe = hub();
    let mut stops: Vec<(f32, f32, Talk)> = Vec::new();
    for i in 0..probe.layout.npcs.len() {
        let at = stand_at_npc(&probe, i);
        stops.push((at[0], at[2], Talk::Npc(i)));
    }
    for (i, s) in probe.layout.stations.iter().enumerate() {
        stops.push((s.pos[0], s.pos[2], Talk::Station(i)));
    }
    for (x, z, want) in stops {
        let mut w = hub();
        let mut bot = Bot::with_legs(vec![
            Leg::Goto(0.0, -12.0),
            Leg::Goto(x, z),
            Leg::Wait(3),
            Leg::Interact,
            Leg::Wait(3),
        ]);
        let mut ticks = 0;
        while !bot.done() && ticks < 6_000 {
            let input = bot.act(&w);
            tick(&mut w, &input);
            ticks += 1;
        }
        assert!(
            bot.done(),
            "the bot never reached {want:?} (leg {})",
            bot.leg
        );
        assert_eq!(current(&w), Some(want), "wrong conversation at ({x}, {z})");
    }
}

#[test]
fn every_portfolio_entry_is_reachable_without_a_quest() {
    let portfolio = PortfolioContent::from_json(PORTFOLIO).expect("portfolio export");
    let mut w = hub();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let note = |facts: &[lf_content::npc::FactRef], seen: &mut BTreeSet<_>| {
        for f in facts {
            for s in portfolio.sections.iter().filter(|s| s.id == f.section) {
                for item in &s.items {
                    let Some(slug) = &item.slug else { continue };
                    if f.slug == ALL_SLUGS || &f.slug == slug {
                        seen.insert((s.id.clone(), slug.clone()));
                    }
                }
            }
        }
    };

    // Every NPC: open the conversation, ask about every topic, go back to the menu.
    for i in 0..w.layout.npcs.len() {
        let id = w.layout.npcs[i].def.id.clone();
        let at = stand_at_npc(&w, i);
        place(&mut w, at);
        interact(&mut w);
        assert_eq!(
            current(&w),
            Some(Talk::Npc(i)),
            "{id}: Interact did not open"
        );
        assert_eq!(topic(&w), 0, "{id}: the menu is open first");
        for t in 0..w.layout.npcs[i].def.topics.len() {
            command(&mut w, Command::Ask { topic: t as u8 });
            assert_eq!(
                usize::from(topic(&w)),
                t + 1,
                "{id}: topic {t} did not open"
            );
            let facts = w.layout.npcs[i].def.topics[t].facts.clone();
            note(&facts, &mut seen);
            command(&mut w, Command::Ask { topic: BACK_TOPIC });
            assert_eq!(topic(&w), 0, "{id}: back to the menu");
        }
        command(&mut w, Command::EndTalk);
        assert_eq!(current(&w), None, "{id}: EndTalk closes");
        idle(&mut w, 2);
    }

    // Every station: open it and read what it shows.
    for i in 0..w.layout.stations.len() {
        let at = w.layout.stations[i].pos;
        place(&mut w, at);
        interact(&mut w);
        assert_eq!(current(&w), Some(Talk::Station(i)));
        let def = w.layout.stations[i].def.clone();
        if !def.unmapped_only {
            note(&def.facts, &mut seen);
        }
        command(&mut w, Command::EndTalk);
        idle(&mut w, 2);
    }

    // Nothing above completed, or even took up, a quest.
    for i in 0..w.layout.npcs.len() {
        assert_eq!(quest_state(&w, i), 0, "{}", w.layout.npcs[i].def.id);
    }
    assert!(
        w.state
            .flags
            .iter()
            .all(|(k, _)| !k.starts_with("quest.") && !k.starts_with("pickup.")),
        "walking the tree changed quest or pickup flags"
    );

    let mut want = BTreeSet::new();
    for s in &portfolio.sections {
        for item in &s.items {
            if let Some(slug) = &item.slug {
                want.insert((s.id.clone(), slug.clone()));
            }
        }
    }
    let missing: Vec<_> = want.difference(&seen).collect();
    assert!(
        missing.is_empty(),
        "not reachable from the hub: {missing:?}"
    );
    // And the data checker agrees.
    let content = Content::load_embedded().expect("embedded content");
    assert!(lf_content::npc::check_facts(&content, &portfolio).is_empty());
}

#[test]
fn a_new_portfolio_entry_is_reachable_without_editing_the_dialogue() {
    // Each section the hub reaches names it with `*`, so an entry added to content.ts shows
    // up on its own. The résumé, blog, work, education, projects, and contact all do.
    let w = hub();
    let mut wild: BTreeSet<String> = BTreeSet::new();
    for n in &w.layout.npcs {
        for t in &n.def.topics {
            for f in t.facts.iter().filter(|f| f.slug == ALL_SLUGS) {
                wild.insert(f.section.clone());
            }
        }
    }
    for s in &w.layout.stations {
        for f in s.def.facts.iter().filter(|f| f.slug == ALL_SLUGS) {
            wild.insert(f.section.clone());
        }
    }
    for section in ["work", "education", "projects", "resume", "blog", "contact"] {
        assert!(wild.contains(section), "{section} has no wildcard topic");
    }
}

#[test]
fn walking_away_ends_the_conversation_and_so_does_death() {
    let mut w = hub();
    talk_to(&mut w, "npc_courier");
    tick(
        &mut w,
        &TickInput {
            frame: crate::input::ActionFrame {
                move_x: 0,
                move_z: 100,
                buttons: 0,
            },
            commands: vec![],
        },
    );
    assert_eq!(current(&w), None, "moving ends the talk");
    talk_to(&mut w, "npc_peddler");
    place(&mut w, [0.0, 0.0, 0.0]);
    idle(&mut w, 1);
    assert_eq!(current(&w), None, "leaving reach ends the talk");
}

#[test]
fn the_prompt_offers_talk_and_use() {
    let mut w = hub();
    assert_eq!(prompt(&w), 0, "nothing in reach at the spawn");
    let i = npc_index(&w, "npc_archivist");
    let at = stand_at_npc(&w, i);
    place(&mut w, at);
    assert_eq!(prompt(&w), 8);
    let s = w
        .layout
        .stations
        .iter()
        .position(|s| s.def.kind == StationKind::Ledger)
        .expect("a Ledger Stand");
    let at = w.layout.stations[s].pos;
    place(&mut w, at);
    assert_eq!(prompt(&w), 9);
    interact(&mut w);
    assert_eq!(prompt(&w), 0, "no prompt while talking");
}

#[test]
fn the_archivist_quest_runs_before_during_after() {
    let mut w = hub();
    let i = talk_to(&mut w, "npc_archivist");
    events(&mut w);
    assert_eq!(quest_state(&w, i), 0);
    // Before: asking takes the quest up and shows the offer.
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(quest_state(&w, i), 1);
    assert_eq!(reply(&w), Reply::Offer as u8);
    // During: nothing brought, so the reminder.
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(quest_state(&w, i), 1);
    assert_eq!(reply(&w), Reply::Reminder as u8);
    assert_eq!(quest_have(&w, i), 0);
    // Two of three folios is still not enough.
    w.state.flags.raise("pickup.folio_ninefold.taken");
    w.state.flags.raise("pickup.folio_library.taken");
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(quest_state(&w, i), 1);
    assert_eq!(quest_have(&w, i), 2);
    // The third finishes it, and the Scholar's Seal is the reward.
    w.state.flags.raise("pickup.folio_whitewarden.taken");
    let seal = w.layout.npcs[i].items["chm_scholars_seal"];
    assert!(
        !w.actors
            .get(w.player)
            .unwrap()
            .inventory
            .gear
            .contains(&seal)
    );
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(quest_state(&w, i), 2);
    assert_eq!(reply(&w), Reply::Done as u8);
    assert!(
        w.actors
            .get(w.player)
            .unwrap()
            .inventory
            .gear
            .contains(&seal)
    );
    assert!(w.state.flags.is_set("reward.scholars_seal"));
    // After: the line changes, and the reward is not paid twice.
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(reply(&w), Reply::After as u8);
    let quest_events: Vec<_> = events(&mut w)
        .into_iter()
        .filter_map(|e| match e {
            Event::Quest(q) => Some((q.npc, q.state)),
            _ => None,
        })
        .collect();
    assert_eq!(quest_events, vec![(i as u16, 1), (i as u16, 2)]);
}

#[test]
fn the_cartographer_binds_four_of_five_pages() {
    let mut w = hub();
    let i = talk_to(&mut w, "npc_cartographer");
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(quest_state(&w, i), 1);
    for n in 1..=3 {
        w.state
            .flags
            .raise(&format!("pickup.page_chronicle_{n}.taken"));
    }
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(quest_state(&w, i), 1, "three pages are not enough");
    // Today's four work entries leave the fourth slot empty: four pages finish it.
    w.state.flags.raise("pickup.page_chronicle_5.taken");
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(quest_state(&w, i), 2);
    assert!(w.state.flags.is_set("dye.opencode"));
    assert!(w.state.flags.is_set("codex.projects.opencode-theme"));
    // A relic the item table doesn't know is only a flag.
    assert!(
        w.state
            .flags
            .is_set("quest.chronicle_pages.reward.relic_opencode")
    );
}

#[test]
fn the_smith_takes_the_ember_at_once_when_it_is_already_held() {
    let mut w = hub();
    let i = talk_to(&mut w, "npc_smith");
    let ember = w.layout.npcs[i].items["itm_forgeheart_ember"];
    let player = w.player;
    let took = {
        let items = &w.items;
        w.actors
            .get_mut(player)
            .unwrap()
            .inventory
            .pick_up(ember, 1, items)
    };
    assert_eq!(took, 1);
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(quest_state(&w, i), 2, "taken up and turned in together");
    assert!(w.state.flags.is_set("smith.advanced_upgrades"));
    // The Smith inspects the Ember; the player keeps it for the upgrades.
    assert_eq!(w.actors.get(player).unwrap().inventory.count(ember), 1);
}

#[test]
fn the_courier_pays_the_politellama_echo() {
    let mut w = hub();
    let i = talk_to(&mut w, "npc_courier");
    for f in [
        "pickup.satchel_verge.taken",
        "pickup.satchel_mirrormere.taken",
        "pickup.satchel_rimeshelf.taken",
    ] {
        w.state.flags.raise(f);
    }
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(quest_state(&w, i), 2);
    assert!(w.state.flags.is_set("echo.politellama.unlocked"));
}

#[test]
fn the_peddler_has_no_quest_line() {
    let mut w = hub();
    let i = talk_to(&mut w, "npc_peddler");
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    assert_eq!(quest_state(&w, i), 0);
    assert_eq!(topic(&w), 0, "the quest line does nothing for the Peddler");
    command(&mut w, Command::Ask { topic: 200 });
    assert_eq!(topic(&w), 0, "an unknown topic does nothing");
}

#[test]
fn the_mirror_applies_a_preset_and_only_an_owned_dye() {
    let mut w = hub();
    let s = w
        .layout
        .stations
        .iter()
        .position(|s| s.def.kind == StationKind::Mirror)
        .expect("a Mirror");
    let at = w.layout.stations[s].pos;
    place(&mut w, at);
    interact(&mut w);
    assert_eq!(current(&w), Some(Talk::Station(s)));
    assert_eq!(dye_mask(&w), 0b01, "only the plain dye to begin with");
    command(&mut w, Command::Appearance { preset: 2, dye: 0 });
    assert_eq!(mirror_choice(&w), (2, 0));
    // The Opencode dye is the Cartographer's reward: locked until the flag is up.
    command(&mut w, Command::Appearance { preset: 1, dye: 1 });
    assert_eq!(mirror_choice(&w), (2, 0));
    w.state.flags.raise("dye.opencode");
    assert_eq!(dye_mask(&w), 0b11);
    command(&mut w, Command::Appearance { preset: 1, dye: 1 });
    assert_eq!(mirror_choice(&w), (1, 1));
    // A preset out of range changes nothing.
    command(&mut w, Command::Appearance { preset: 9, dye: 0 });
    assert_eq!(mirror_choice(&w), (1, 1));
    let changes = events(&mut w)
        .into_iter()
        .filter(|e| matches!(e, Event::Appearance(_)))
        .count();
    assert_eq!(changes, 2, "only the two applied changes raise an event");
}

#[test]
fn a_safe_zone_clears_combat_buttons_and_caps_the_stick_at_a_walk() {
    let mut w = hub();
    idle(&mut w, 4);
    let swing = TickInput {
        frame: crate::input::ActionFrame {
            move_x: 0,
            move_z: 127,
            buttons: buttons::LIGHT | buttons::HEAVY | buttons::DODGE,
        },
        commands: vec![],
    };
    let walk_speed = w.locomotion.walk_speed;
    let mut fastest = 0.0f32;
    for _ in 0..60 {
        tick(&mut w, &swing);
        let p = w.actors.get(w.player).unwrap();
        assert!(p.moves.current.is_none(), "a swing started in a safe zone");
        fastest = fastest.max(p.vel[0].abs().max(p.vel[2].abs()));
    }
    assert!(
        fastest <= walk_speed * 1.05,
        "moved at {fastest} m/s; a walk is {walk_speed}"
    );
    assert!(fastest > 0.5, "the player did not move at all");
    // The same input in the Verge swings.
    let content = Content::load_embedded().expect("embedded content");
    let mut v = World::new(SEED, &content, "verge").expect("verge");
    idle(&mut v, 4);
    tick(&mut v, &swing);
    tick(&mut v, &swing);
    assert!(
        v.actors.get(v.player).unwrap().moves.current.is_some(),
        "the Verge is not a safe zone"
    );
}

#[test]
fn dialogue_and_quests_are_deterministic_and_saved_in_the_flags() {
    let run = || {
        let mut w = hub();
        let i = talk_to(&mut w, "npc_archivist");
        command(&mut w, Command::Ask { topic: 1 });
        command(&mut w, Command::Ask { topic: QUEST_TOPIC });
        (w.state_hash(), quest_state(&w, i), topic(&w))
    };
    let (a, state, open) = run();
    let (b, _, _) = run();
    assert_eq!(a, b);
    assert_eq!(state, 1);
    assert_eq!(open, QUEST_TOPIC);
}

#[test]
fn the_render_state_reports_the_open_conversation() {
    let mut w = hub();
    let words = render_state(&w);
    assert_eq!(words.len(), STATE_FIXED_WORDS + 5);
    assert_eq!(words[0], 0.0);
    let i = talk_to(&mut w, "npc_archivist");
    command(&mut w, Command::Ask { topic: QUEST_TOPIC });
    let words = render_state(&w);
    assert_eq!(words[0], 1.0);
    assert_eq!(words[1], 0.0);
    assert_eq!(words[2], i as f32);
    assert_eq!(words[3], f32::from(QUEST_TOPIC));
    assert_eq!(words[4], Reply::Offer as u8 as f32);
    assert_eq!(words[5], 1.0);
    assert_eq!(words[7], 3.0);
    assert_eq!(words[STATE_FIXED_WORDS + i], 1.0);
}
