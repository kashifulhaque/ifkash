//! Dialogue and questlines (WP-068). See docs/game/06-progression-and-systems.md, "NPCs and
//! questlines", and docs/game/02-world.md, "Hollowhearth (hub)".
//!
//! # What lives here
//!
//! The simulation owns the conversation and the quests; the browser's DOM dialogue reads
//! the state and sends [`Command`]s.
//!
//! - `INTERACT` near an NPC or a station opens a conversation ([`run`]). Walking, any other
//!   button, leaving reach, dying, or `Command::EndTalk` closes it.
//! - `Command::Ask` picks a topic from the NPC's **Ask about...** menu, or the quest line.
//!   `Command::Appearance` sets the Mirror's preset and dye.
//! - A quest has three states: 0 before, 1 during, 2 after (flag `quest.<id>.state`).
//!   Asking about the quest takes it up; asking again with everything brought turns it in
//!   and pays the reward. **No portfolio fact is locked behind a quest**: every topic is
//!   open from the first conversation, and a quest only changes greetings, rewards, and
//!   flavor.
//! - A level marked `safe_zone` (Hollowhearth) clears the combat buttons and caps the stick
//!   at a walk ([`safe_zone_input`]).
//!
//! # State
//!
//! All of it is world flags, so it saves, hashes, and replays with the rest of the world,
//! and a world that never talks hashes exactly as before:
//!
//! | Flag | Value |
//! |---|---|
//! | `dialogue.npc` | the NPC's index plus one while talking to one |
//! | `dialogue.station` | the station's index plus one while at one |
//! | `dialogue.topic` | 0 for the menu, a topic's index plus one, or 255 for the quest line |
//! | `dialogue.reply` | which quest reply is showing (see [`Reply`]) |
//! | `quest.<id>.state` | 1 during, 2 after (0 is unset) |
//! | `mirror.preset`, `mirror.dye` | the Mirror's choice (0 is unset) |

use std::collections::BTreeMap;

use lf_content::npc::{BACK_TOPIC, Need, QUEST_TOPIC, StationKind};
use lf_content::scenario::Scenario;
use lf_content::{Content, NpcDef, StationDef};
use serde::{Deserialize, Serialize};

use crate::events::{AppearanceEvent, DialogueEvent, Event, QuestEvent};
use crate::input::{Command, TickInput, buttons};
use crate::math::dmath::sqrt;
use crate::world::World;

use super::{Layout, flags};

/// Flag: the open NPC's index plus one.
pub const DIALOGUE_NPC: &str = "dialogue.npc";
/// Flag: the open station's index plus one.
pub const DIALOGUE_STATION: &str = "dialogue.station";
/// Flag: the open topic (0 menu, index plus one, or 255 for the quest line).
pub const DIALOGUE_TOPIC: &str = "dialogue.topic";
/// Flag: which quest reply is showing.
pub const DIALOGUE_REPLY: &str = "dialogue.reply";
/// Flag: the Mirror's preset.
pub const MIRROR_PRESET: &str = "mirror.preset";
/// Flag: the Mirror's dye.
pub const MIRROR_DYE: &str = "mirror.dye";

/// How the stick is capped in a safe zone: this many units below `walk_below`, so a full
/// push is still a walk.
const WALK_MARGIN: f32 = 1.0;

/// The buttons a safe zone clears: weapons are sheathed and nobody dodges, guards, or
/// casts. (Sprinting is DODGE held, so this also keeps the walk a walk.)
const SAFE_BLOCKED: u16 = buttons::LIGHT
    | buttons::HEAVY
    | buttons::GUARD
    | buttons::DODGE
    | buttons::ART
    | buttons::TWO_HAND;

/// An NPC placed in the level, with its definition and the registry numbers of the items
/// its quest refers to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NpcSpot {
    pub def: NpcDef,
    pub pos: [f32; 3],
    pub yaw: f32,
    pub radius_m: f32,
    /// Registry numbers of the items the quest needs or gives, by string ID.
    pub items: BTreeMap<String, u32>,
}

/// A station placed in the level, with its definition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StationSpot {
    pub def: StationDef,
    pub pos: [f32; 3],
    pub yaw: f32,
    pub radius_m: f32,
}

/// What the player is talking to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Talk {
    /// The NPC at this index of `Layout::npcs`.
    Npc(usize),
    /// The station at this index of `Layout::stations`.
    Station(usize),
}

/// Which line of a quest the NPC is saying (the value of [`DIALOGUE_REPLY`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reply {
    None = 0,
    /// The offer, when the quest is taken up.
    Offer = 1,
    /// The reminder, when the player asks with something still missing.
    Reminder = 2,
    /// The thanks, when the quest is turned in.
    Done = 3,
    /// The line after the quest.
    After = 4,
}

/// The flag of a quest's state.
pub fn quest_flag(quest_id: &str) -> String {
    format!("quest.{quest_id}.state")
}

/// Fills the layout's NPCs, stations, and safe-zone flag from the scenario. Fails on an
/// NPC, station, or item the content doesn't know.
pub fn extend_layout(
    layout: &mut Layout,
    level: &Scenario,
    content: &Content,
) -> Result<(), String> {
    layout.safe_zone = level.safe_zone;
    for n in &level.npcs {
        let def = content
            .npc_by_id(&n.npc)
            .ok_or_else(|| format!("npc \"{}\" has no definition", n.npc))?;
        let mut items = BTreeMap::new();
        if let Some(q) = &def.quest {
            let wanted = q
                .needs
                .iter()
                .filter_map(|n| match n {
                    Need::Item(id) => Some(id),
                    Need::Flag(_) => None,
                })
                .chain(&q.reward.items);
            for id in wanted {
                let num = content
                    .registry
                    .num(id)
                    .ok_or_else(|| format!("npc \"{}\": item \"{id}\" is not registered", n.npc))?;
                items.insert(id.clone(), num);
            }
        }
        layout.npcs.push(NpcSpot {
            def: def.clone(),
            pos: n.pos,
            yaw: n.yaw,
            radius_m: n.radius_m,
            items,
        });
    }
    for s in &level.stations {
        let def = content
            .station_by_id(&s.station)
            .ok_or_else(|| format!("station \"{}\" has no definition", s.station))?;
        layout.stations.push(StationSpot {
            def: def.clone(),
            pos: s.pos,
            yaw: s.yaw,
            radius_m: s.radius_m,
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------
// Reading the state

/// The conversation that is open, if any.
pub fn current(world: &World) -> Option<Talk> {
    let f = &world.state.flags;
    let npc = f.get(DIALOGUE_NPC);
    if npc > 0 && (npc as usize) <= world.layout.npcs.len() {
        return Some(Talk::Npc(npc as usize - 1));
    }
    let st = f.get(DIALOGUE_STATION);
    if st > 0 && (st as usize) <= world.layout.stations.len() {
        return Some(Talk::Station(st as usize - 1));
    }
    None
}

/// The open topic: 0 for the menu, a topic's index plus one, or 255 for the quest line.
pub fn topic(world: &World) -> u8 {
    u8::try_from(world.state.flags.get(DIALOGUE_TOPIC)).unwrap_or(0)
}

/// The quest reply showing now.
pub fn reply(world: &World) -> u8 {
    u8::try_from(world.state.flags.get(DIALOGUE_REPLY)).unwrap_or(0)
}

/// An NPC's quest state: 0 before, 1 during, 2 after (0 for an NPC with no quest).
pub fn quest_state(world: &World, npc: usize) -> u8 {
    let Some(q) = world
        .layout
        .npcs
        .get(npc)
        .and_then(|n| n.def.quest.as_ref())
    else {
        return 0;
    };
    u8::try_from(world.state.flags.get(&quest_flag(&q.id)))
        .unwrap_or(2)
        .min(2)
}

/// How many of an NPC's quest needs are met now.
pub fn quest_have(world: &World, npc: usize) -> usize {
    let Some(n) = world.layout.npcs.get(npc) else {
        return 0;
    };
    let Some(q) = &n.def.quest else { return 0 };
    let held = |id: &String| {
        let Some(num) = n.items.get(id) else {
            return false;
        };
        world
            .actors
            .get(world.player)
            .is_some_and(|p| p.inventory.count(*num) > 0)
    };
    q.needs
        .iter()
        .filter(|need| match need {
            Need::Flag(f) => world.state.flags.is_set(f),
            Need::Item(id) => held(id),
        })
        .count()
}

/// The Mirror's dyes the player owns, as a bit mask over the first Mirror's `dyes`.
pub fn dye_mask(world: &World) -> u32 {
    let Some(m) = world
        .layout
        .stations
        .iter()
        .find(|s| s.def.kind == StationKind::Mirror)
    else {
        return 0;
    };
    m.def
        .dyes
        .iter()
        .enumerate()
        .filter(|(_, d)| {
            d.unlock_flag
                .as_ref()
                .is_none_or(|f| world.state.flags.is_set(f))
        })
        .fold(0, |mask, (i, _)| mask | 1 << i)
}

fn horizontal_d2(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (dx, dz) = (a[0] - b[0], a[2] - b[2]);
    dx * dx + dz * dz
}

fn in_reach(world: &World, t: Talk) -> bool {
    let Some(p) = world.actors.get(world.player) else {
        return false;
    };
    let h = world.locomotion.mechanics.interact_height_m;
    let (at, r) = match t {
        Talk::Npc(i) => (world.layout.npcs[i].pos, world.layout.npcs[i].radius_m),
        Talk::Station(i) => (
            world.layout.stations[i].pos,
            world.layout.stations[i].radius_m,
        ),
    };
    horizontal_d2(p.pos, at) <= r * r && (at[1] - p.pos[1]).abs() <= h
}

/// The NPC or station `INTERACT` would talk to now: the nearest in reach (an NPC wins a
/// tie, then the lower index). The player must be free to act: alive, standing, not
/// mid-move, and not at a Lantern.
pub fn nearest(world: &World) -> Option<Talk> {
    let p = world.actors.get(world.player)?;
    if p.dead || !p.motor.grounded || p.moves.current.is_some() || world.state.resting.is_some() {
        return None;
    }
    let h = world.locomotion.mechanics.interact_height_m;
    let mut best: Option<(f32, Talk)> = None;
    let mut offer = |d2: f32, r: f32, y: f32, t: Talk| {
        if d2 <= r * r && (y - p.pos[1]).abs() <= h && best.is_none_or(|(b, _)| d2 < b) {
            best = Some((d2, t));
        }
    };
    for (i, n) in world.layout.npcs.iter().enumerate() {
        offer(
            horizontal_d2(p.pos, n.pos),
            n.radius_m,
            n.pos[1],
            Talk::Npc(i),
        );
    }
    for (i, s) in world.layout.stations.iter().enumerate() {
        offer(
            horizontal_d2(p.pos, s.pos),
            s.radius_m,
            s.pos[1],
            Talk::Station(i),
        );
    }
    best.map(|(_, t)| t)
}

/// What the HUD should offer for `INTERACT`: 0 nothing, 8 speak with someone, 9 use a
/// station. A conversation that is open offers nothing.
pub fn prompt(world: &World) -> u32 {
    if world.layout.npcs.is_empty() && world.layout.stations.is_empty() {
        return 0;
    }
    if current(world).is_some() {
        return 0;
    }
    match nearest(world) {
        None => 0,
        Some(Talk::Npc(_)) => 8,
        Some(Talk::Station(_)) => 9,
    }
}

// ---------------------------------------------------------------------------------------
// Safe zone

/// The input a safe zone lets through: the combat buttons cleared, and the stick capped to
/// a walk. `None` outside a safe zone, or when nothing needs to change.
pub fn safe_zone_input(world: &World, input: &TickInput) -> Option<TickInput> {
    if !world.layout.safe_zone {
        return None;
    }
    let f = input.frame;
    let cap = (world.locomotion.walk_below - WALK_MARGIN).max(1.0);
    let (x, z) = (f32::from(f.move_x), f32::from(f.move_z));
    let mag = sqrt(x * x + z * z);
    let capped = mag > cap;
    if !capped && f.buttons & SAFE_BLOCKED == 0 {
        return None;
    }
    let mut out = input.clone();
    out.frame.buttons &= !SAFE_BLOCKED;
    if capped {
        let k = cap / mag;
        out.frame.move_x = (x * k) as i8;
        out.frame.move_z = (z * k) as i8;
    }
    Some(out)
}

// ---------------------------------------------------------------------------------------
// Tick system 1

/// Opens, runs, and ends conversations. Called from tick system 1 (`apply_commands`).
pub fn run(world: &mut World, input: &TickInput) {
    if world.layout.npcs.is_empty() && world.layout.stations.is_empty() {
        return;
    }
    let alive = world
        .actors
        .get(world.player)
        .is_some_and(|p| !p.dead && p.hp > 0);
    if let Some(t) = current(world) {
        let f = input.frame;
        let walked = f.move_x != 0 || f.move_z != 0 || f.buttons & !buttons::INTERACT != 0;
        if !alive || walked || !in_reach(world, t) {
            close(world);
            return;
        }
        for c in &input.commands {
            match *c {
                Command::EndTalk => {
                    close(world);
                    return;
                }
                Command::Ask { topic } => {
                    if let Talk::Npc(i) = t {
                        ask(world, i, topic);
                    }
                }
                Command::Appearance { preset, dye } => {
                    if let Talk::Station(i) = t {
                        appearance(world, i, preset, dye);
                    }
                }
                _ => {}
            }
        }
    } else if alive {
        let prev = world
            .actors
            .get(world.player)
            .map_or(0, |p| p.motor.prev_buttons);
        let pressed = input.frame.buttons & !prev;
        if pressed & buttons::INTERACT != 0
            && let Some(t) = nearest(world)
        {
            open(world, t);
        }
    }
}

fn target(t: Talk) -> (u8, u16) {
    match t {
        Talk::Npc(i) => (0, i as u16),
        Talk::Station(i) => (1, i as u16),
    }
}

fn dialogue_event(world: &mut World, what: u8, t: Talk) {
    let (kind, index) = target(t);
    let topic = topic(world);
    world.events.push(Event::Dialogue(DialogueEvent {
        what,
        target: kind,
        index,
        topic,
    }));
}

/// Starts talking to `t`.
pub fn open(world: &mut World, t: Talk) {
    let f = &mut world.state.flags;
    f.clear(DIALOGUE_TOPIC);
    f.clear(DIALOGUE_REPLY);
    match t {
        Talk::Npc(i) => f.set(DIALOGUE_NPC, i as u32 + 1),
        Talk::Station(i) => f.set(DIALOGUE_STATION, i as u32 + 1),
    }
    dialogue_event(world, DialogueEvent::OPENED, t);
}

/// Ends the conversation, if one is open.
pub fn close(world: &mut World) {
    let Some(t) = current(world) else { return };
    dialogue_event(world, DialogueEvent::CLOSED, t);
    let f = &mut world.state.flags;
    f.clear(DIALOGUE_NPC);
    f.clear(DIALOGUE_STATION);
    f.clear(DIALOGUE_TOPIC);
    f.clear(DIALOGUE_REPLY);
}

fn set_topic(world: &mut World, npc: usize, topic: u8, reply: Reply) {
    world.state.flags.set(DIALOGUE_TOPIC, u32::from(topic));
    world.state.flags.set(DIALOGUE_REPLY, reply as u32);
    dialogue_event(world, DialogueEvent::TOPIC, Talk::Npc(npc));
}

/// Asks NPC `npc` about `topic` (see `Command::Ask`). Does nothing for a topic that isn't
/// in the menu.
pub fn ask(world: &mut World, npc: usize, topic: u8) {
    let Some(n) = world.layout.npcs.get(npc) else {
        return;
    };
    let has_quest = n.def.quest.is_some();
    let topics = n.def.topics.len();
    if topic == BACK_TOPIC {
        set_topic(world, npc, 0, Reply::None);
    } else if topic == QUEST_TOPIC {
        if has_quest {
            ask_quest(world, npc);
        }
    } else if usize::from(topic) < topics {
        set_topic(world, npc, topic + 1, Reply::None);
    }
}

fn ask_quest(world: &mut World, npc: usize) {
    let state = quest_state(world, npc);
    let (id, required) = {
        let q = world.layout.npcs[npc]
            .def
            .quest
            .as_ref()
            .expect("has a quest");
        (q.id.clone(), q.required())
    };
    let met = quest_have(world, npc) >= required;
    match state {
        0 => {
            world.state.flags.set(&quest_flag(&id), 1);
            world.events.push(Event::Quest(QuestEvent {
                npc: npc as u16,
                state: 1,
            }));
            if met {
                // The player already holds everything: it is turned in at once.
                complete(world, npc, &id);
            } else {
                set_topic(world, npc, QUEST_TOPIC, Reply::Offer);
            }
        }
        1 if met => complete(world, npc, &id),
        1 => set_topic(world, npc, QUEST_TOPIC, Reply::Reminder),
        _ => set_topic(world, npc, QUEST_TOPIC, Reply::After),
    }
}

fn complete(world: &mut World, npc: usize, id: &str) {
    world.state.flags.set(&quest_flag(id), 2);
    let spot = world.layout.npcs[npc].clone();
    let reward = &spot.def.quest.as_ref().expect("has a quest").reward;
    for f in &reward.flags {
        world.state.flags.raise(f);
    }
    for entry in &reward.codex {
        world.state.flags.raise(&flags::codex(entry));
    }
    for item in &reward.items {
        // An item the table doesn't know (a relic) is only a flag.
        let got = spot.items.get(item).is_some_and(|&num| {
            let player = world.player;
            if world.items.is_weapon(num) {
                world
                    .actors
                    .get_mut(player)
                    .is_some_and(|p| p.inventory.give_gear(num))
            } else if world.items.gear.is_gear(num) || world.items.get(num).is_some() {
                let items = &world.items;
                world
                    .actors
                    .get_mut(player)
                    .is_some_and(|p| p.inventory.pick_up(num, 1, items) > 0)
            } else {
                false
            }
        });
        if !got {
            world
                .state
                .flags
                .raise(&format!("quest.{id}.reward.{item}"));
        }
    }
    world.events.push(Event::Quest(QuestEvent {
        npc: npc as u16,
        state: 2,
    }));
    set_topic(world, npc, QUEST_TOPIC, Reply::Done);
}

fn appearance(world: &mut World, station: usize, preset: u8, dye: u8) {
    let s = &world.layout.stations[station].def;
    if s.kind != StationKind::Mirror
        || usize::from(preset) >= s.presets.len()
        || usize::from(dye) >= s.dyes.len()
    {
        return;
    }
    let owned = s.dyes[usize::from(dye)]
        .unlock_flag
        .as_ref()
        .is_none_or(|f| world.state.flags.is_set(f));
    if !owned {
        return;
    }
    world.state.flags.set(MIRROR_PRESET, u32::from(preset));
    world.state.flags.set(MIRROR_DYE, u32::from(dye));
    world
        .events
        .push(Event::Appearance(AppearanceEvent { preset, dye }));
}

/// The Mirror's current preset and dye indexes.
pub fn mirror_choice(world: &World) -> (u8, u8) {
    let f = &world.state.flags;
    (
        u8::try_from(f.get(MIRROR_PRESET)).unwrap_or(0),
        u8::try_from(f.get(MIRROR_DYE)).unwrap_or(0),
    )
}

/// Floats in [`render_state`] before the per-NPC quest states.
pub const STATE_FIXED_WORDS: usize = 12;

/// The dialogue as floats, for the host: 0 open (0 or 1), 1 what is open (0 NPC, 1
/// station), 2 its index, 3 the open topic (0 menu, a topic's index plus one, 255 the quest
/// line), 4 the quest reply showing (see [`Reply`]), 5 the NPC's quest state, 6 and 7 how
/// many needs are met and how many are required, 8 how many needs the quest has, 9 the
/// Mirror's preset, 10 its dye, 11 a mask of the dyes the player owns; then one word per NPC
/// (in the scenario's order) with its quest state.
pub fn render_state(world: &World) -> Vec<f32> {
    let mut out = vec![0.0f32; STATE_FIXED_WORDS];
    if let Some(t) = current(world) {
        let (kind, index) = target(t);
        out[0] = 1.0;
        out[1] = f32::from(kind);
        out[2] = f32::from(index);
        out[3] = f32::from(topic(world));
        out[4] = f32::from(reply(world));
        if let Talk::Npc(i) = t
            && let Some(q) = &world.layout.npcs[i].def.quest
        {
            out[5] = f32::from(quest_state(world, i));
            out[6] = quest_have(world, i) as f32;
            out[7] = q.required() as f32;
            out[8] = q.needs.len() as f32;
        }
    }
    let (preset, dye) = mirror_choice(world);
    out[9] = f32::from(preset);
    out[10] = f32::from(dye);
    out[11] = dye_mask(world) as f32;
    out.extend((0..world.layout.npcs.len()).map(|i| f32::from(quest_state(world, i))));
    out
}

#[cfg(test)]
mod tests;
