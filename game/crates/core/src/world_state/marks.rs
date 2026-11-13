//! Marks, gestures, and the in-game timer (WP-088). See docs/game/06-progression-and-systems.md,
//! "Achievements", "Gestures and photo mode", and "Timer and new game plus".
//!
//! # Marks
//!
//! A Mark is earned in the simulation, so the RL wrapper and a replay see it. An earned Mark is
//! the world flag `mark.<slug>` (so it saves, travels between areas, hashes, and replays with
//! the rest of the world) plus one [`Event::Mark`] (WASM event kind 57) on the tick it is
//! earned. The flags are an ordered map, so the earned set is deterministic. [`run`] is the
//! last thing tick system 10 (`systems::world_logic`) does and works from two sources:
//!
//! - **Flags**, for Marks that are a fact about the world: a Lantern lit, a boss defeated, every
//!   Chronicle Page or relic found. They are checked in [`sweep`], when something that could
//!   change them happened this tick and every [`SWEEP_EVERY`] ticks (which also catches a save
//!   from before Marks existed).
//! - **Events** since the last tick (`EventQueue::since`), for Marks that need the moment
//!   itself: the tick a boss fell (Speedrunner), B07's counts (Generalist), whether the player
//!   was hit during the fight ([`Track`], Unbroken), a kill (Pacifist Path), a bow (Polite).
//!
//! | Mark | Earned when |
//! |---|---|
//! | First Light | any Lantern is lit (the player rested at one) |
//! | Woken | B01 is defeated |
//! | Reader | the player opens the Ledger Stand (`stn_ledger`) |
//! | Correspondent | the player talks to the Courier (`npc_courier`) |
//! | Chronicler | every placed Chronicle Page is taken ([`PAGES`]) |
//! | Collector | every relic is found ([`RELICS`]) |
//! | One per Great Sigil | B06, B07, B09, B10 is defeated |
//! | Ever After | B12 is defeated |
//! | Thorough | all 12 bosses are defeated |
//! | Pacifist Path | the player stands at the Spire Gate with no non-boss kill after the Verge |
//! | Generalist | B07 falls with no action category over 35% of the phase 1 counts |
//! | Speedrunner | B12 falls under 45 minutes of in-game time |
//! | Polite | the player bows to the PoliteLlama Echo in time |
//! | Unbroken | a Great Sigil boss falls and the player lost no HP since its fight began |
//!
//! # The timer
//!
//! In-game time is the tick counter (30 ticks a second), which only advances while the
//! simulation runs: a paused game, a menu, and photo mode add nothing. A Lantern trip keeps the
//! counter (`world_state::map`). A save doesn't keep the world's tick, so [`run`] writes it to
//! the flag [`PLAY_TICKS`] each time the player rests (the save point) and `save::restore` reads
//! it back: loading a save puts the timer back where it was at the Lantern.
//!
//! # Gestures
//!
//! See [`gesture`]. A gesture is a `Command::Gesture`, so it goes through the same command list
//! as the rest of the menus and replays.

pub mod gesture;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::ai::boss::Stage;
use crate::arena::Id;
use crate::events::{BossEvent, EchoEvent, Event, LifeEvent, MarkEvent};
use crate::input::TickInput;
use crate::world::World;

use super::flags::{self, Flags};
use super::quests::{self, Talk};

/// How a Mark reads in the Marks list: its slug (also the ID the cloud sync stores), its name,
/// and how to earn it, in the spec's words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mark {
    pub slug: &'static str,
    pub name: &'static str,
    pub how: &'static str,
}

pub const FIRST_LIGHT: usize = 0;
pub const WOKEN: usize = 1;
pub const READER: usize = 2;
pub const CORRESPONDENT: usize = 3;
pub const CHRONICLER: usize = 4;
pub const COLLECTOR: usize = 5;
pub const SIGIL_STILLWATER: usize = 6;
pub const SIGIL_GRAYMANTLE: usize = 7;
pub const SIGIL_DEEP_LAMP: usize = 8;
pub const SIGIL_RIMESHELF: usize = 9;
pub const EVER_AFTER: usize = 10;
pub const THOROUGH: usize = 11;
pub const PACIFIST_PATH: usize = 12;
pub const GENERALIST: usize = 13;
pub const SPEEDRUNNER: usize = 14;
pub const POLITE: usize = 15;
pub const UNBROKEN: usize = 16;
/// How many Marks there are. A `u32` mask holds them all (`earned_mask`).
pub const MARK_COUNT: usize = 17;

/// Every Mark, in the order of the constants above. The order is part of the WASM export
/// (`marks_state`'s mask and the event's index): only ever append.
pub const MARKS: [Mark; MARK_COUNT] = [
    Mark {
        slug: "first_light",
        name: "First Light",
        how: "Rest at a Lantern.",
    },
    Mark {
        slug: "woken",
        name: "Woken",
        how: "Defeat the Hollow Sentry.",
    },
    Mark {
        slug: "reader",
        name: "Reader",
        how: "Open the Wayfarer's Ledger.",
    },
    Mark {
        slug: "correspondent",
        name: "Correspondent",
        how: "Talk to the Courier.",
    },
    Mark {
        slug: "chronicler",
        name: "Chronicler",
        how: "Collect every Chronicle Page.",
    },
    Mark {
        slug: "collector",
        name: "Collector",
        how: "Find every relic.",
    },
    Mark {
        slug: "sigil_stillwater",
        name: "Sigil of Stillwater",
        how: "Defeat the Drowned Choir.",
    },
    Mark {
        slug: "sigil_graymantle",
        name: "Sigil of Graymantle",
        how: "Defeat Castellan Voss.",
    },
    Mark {
        slug: "sigil_deep_lamp",
        name: "Sigil of the Deep Lamp",
        how: "Defeat the Lamplight Matriarch.",
    },
    Mark {
        slug: "sigil_rimeshelf",
        name: "Sigil of Rimeshelf",
        how: "Defeat the Bellwright.",
    },
    Mark {
        slug: "ever_after",
        name: "Ever After",
        how: "Defeat the Unspooled.",
    },
    Mark {
        slug: "thorough",
        name: "Thorough",
        how: "Defeat all 12 bosses.",
    },
    Mark {
        slug: "pacifist_path",
        name: "Pacifist Path",
        how: "Reach the Spire Gate having killed no ordinary enemy after the Verge.",
    },
    Mark {
        slug: "generalist",
        name: "Generalist",
        how: "Defeat the Overfit Warden without being overfit: no action over 35% in phase 1.",
    },
    Mark {
        slug: "speedrunner",
        name: "Speedrunner",
        how: "Defeat the Unspooled in under 45 minutes of in-game time.",
    },
    Mark {
        slug: "polite",
        name: "Polite",
        how: "Bow to the PoliteLlama Echo.",
    },
    Mark {
        slug: "unbroken",
        name: "Unbroken",
        how: "Defeat a Great Sigil boss without taking damage.",
    },
];

/// The flag that says Mark `i` is earned.
pub fn flag(i: usize) -> String {
    format!("mark.{}", MARKS[i].slug)
}

/// True if Mark `i` is earned in `flags`.
pub fn is_earned(flags: &Flags, i: usize) -> bool {
    flags.is_set(&flag(i))
}

/// The earned Marks as a bit mask: bit `i` is Mark `i`.
pub fn earned_mask(flags: &Flags) -> u32 {
    (0..MARK_COUNT)
        .filter(|&i| is_earned(flags, i))
        .fold(0, |m, i| m | 1 << i)
}

/// The bosses, in order (docs "Boss roster"): B01 to B12.
pub const BOSS_IDS: [&str; 12] = [
    "b01_hollow_sentry",
    "b02_briar_stag",
    "b03_bridgewarden_corvane",
    "b04_mossgut_brood",
    "b05_ossuary_keeper",
    "b06_drowned_choir",
    "b07_castellan_voss",
    "b08_forgeheart_colossus",
    "b09_lamplight_matriarch",
    "b10_the_bellwright",
    "b11_bound_exile",
    "b12_the_unspooled",
];
const B01: &str = BOSS_IDS[0];
const B07: &str = BOSS_IDS[6];
const B12: &str = BOSS_IDS[11];
/// The Great Sigil bosses and the Mark each one gives (docs "Boss roster": B06, B07, B09, B10).
const SIGILS: [(&str, usize); 4] = [
    (BOSS_IDS[5], SIGIL_STILLWATER),
    (BOSS_IDS[6], SIGIL_GRAYMANTLE),
    (BOSS_IDS[8], SIGIL_DEEP_LAMP),
    (BOSS_IDS[9], SIGIL_RIMESHELF),
];

/// The Chronicle Pages that can be found (the pickups `page_chronicle_N`). Slot 4 isn't placed
/// in any level yet (`entities.ron` of Rimeshelf), so it isn't asked for; add it here when a
/// level places it.
const PAGES: [&str; 4] = [
    "page_chronicle_1",
    "page_chronicle_2",
    "page_chronicle_3",
    "page_chronicle_5",
];
/// The codex entries of the relics: seven are pickups in the levels, and the eighth,
/// `projects.opencode-theme`, is the Cartographer's reward for the Chronicle quest.
const RELICS: [&str; 8] = [
    "projects.banana-cpp",
    "projects.vicharak-micro-llama",
    "projects.nopokedb",
    "projects.boo",
    "projects.smoltorch",
    "projects.ferray",
    "projects.endark",
    "projects.opencode-theme",
];

/// Speedrunner: B12 must fall before this tick, 45 minutes at 30 ticks a second (docs
/// "Achievements").
pub const SPEEDRUN_TICKS: u64 = 45 * 60 * 30;
/// Generalist: no action category may be over this percent of the phase 1 counts (docs
/// "Achievements").
const GENERALIST_PERCENT: u32 = 35;
/// The areas that make up the Verge, where kills don't break Pacifist Path (docs "Regions": the
/// Verge, its tutorial cave, its mini-dungeon, and the hub, where nothing fights).
const VERGE_AREAS: [&str; 4] = ["waking_hollow", "verge", "mossgut_cave", "hollowhearth"];
/// The door the Spire Gate is (`levels/spire/entities.ron`).
const SPIRE_GATE: &str = "spire_gate";
/// How close the player stands to the gate to have reached it (tuning: the gate's width and a
/// step; the spec only says "reach").
const GATE_REACH_M: f32 = 6.0;
/// Ordinary enemies the player killed after the Verge (a count, saved with the flags).
pub const KILLS_AFTER_VERGE: &str = "stat.kills_after_verge";
/// The in-game tick at the player's last rest, saved with the flags (see the module docs).
pub const PLAY_TICKS: &str = "stat.play_ticks";
/// Flags are checked this often when nothing happened that could change them (tuning).
const SWEEP_EVERY: u64 = 32;

/// Bookkeeping that isn't a flag. Part of `WorldState`, and hashed only while it isn't at rest.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Track {
    /// The gesture the player is performing (see [`gesture`]).
    pub gesture: Option<gesture::Run>,
    /// The Great Sigil boss fight under watch for Unbroken: the boss's index plus one, or 0.
    pub watch: u16,
    /// The player's HP at the last tick of that fight.
    pub watch_hp: u32,
    /// The player lost HP during it.
    pub watch_hit: bool,
    /// How many events of the queue have been read (`EventQueue::pushed`). Bookkeeping about an
    /// unhashed queue, so not hashed or saved.
    #[serde(skip)]
    pub seen: u64,
}

impl Track {
    /// At rest: no gesture and no fight under watch. Such a state is left out of the hash.
    pub fn is_inert(&self) -> bool {
        self.gesture.is_none() && self.watch == 0 && self.watch_hp == 0 && !self.watch_hit
    }
}

/// What the events of a tick said, boiled down to what the Marks need.
enum Note {
    /// Boss `.0` (an index into the layout) was defeated.
    Defeated(usize),
    /// The player killed this actor.
    Killed(Id),
    /// An Echo with this registry number was bowed to.
    Bowed(u32),
    /// The player rested at a Lantern.
    Rested,
    /// Something happened that can complete a flag Mark: a pickup, a quest.
    Changed,
}

/// Tick system 10's last step: gestures, then the Marks.
pub fn run(world: &mut World, input: &TickInput) {
    gesture::run(world, input);
    // Damage in a watched fight counts before the fight's end is read below.
    watch_damage(world);
    let notes = read_events(world);
    let mut sweep_due = world.tick.is_multiple_of(SWEEP_EVERY);
    for note in notes {
        match note {
            Note::Defeated(b) => {
                sweep_due = true;
                defeated(world, b);
            }
            Note::Killed(target) => killed(world, target),
            Note::Bowed(echo) => bowed(world, echo),
            Note::Rested => {
                sweep_due = true;
                // The counter goes up at the end of this tick, so a load that starts from the
                // flag continues at the tick after the rest.
                let ticks = u32::try_from(world.tick + 1).unwrap_or(u32::MAX);
                world.state.flags.set(PLAY_TICKS, ticks);
            }
            Note::Changed => sweep_due = true,
        }
    }
    refresh_watch(world);
    talked(world);
    at_spire_gate(world);
    if sweep_due {
        sweep(world);
    }
}

/// Raises Mark `i`'s flag and emits the event, once.
pub fn earn(world: &mut World, i: usize) {
    let f = flag(i);
    if world.state.flags.is_set(&f) {
        return;
    }
    world.state.flags.raise(&f);
    let tick = world.tick;
    world.events.push(Event::Mark(MarkEvent {
        mark: i as u8,
        tick,
    }));
}

fn boss_id(world: &World, b: usize) -> Option<&str> {
    world.layout.bosses.get(b).map(|l| l.def.id.as_str())
}

/// Reads the events pushed since the last call, boiled down to [`Note`]s so the Marks can
/// change the world while they act on them.
fn read_events(world: &mut World) -> Vec<Note> {
    let mut notes = Vec::new();
    for e in world.events.since(world.state.marks.seen) {
        match e {
            Event::Boss(b) if b.kind == BossEvent::DEFEATED => {
                notes.push(Note::Defeated(usize::from(b.boss)));
            }
            Event::Hit(h) if h.killed && h.attacker == world.player => {
                notes.push(Note::Killed(h.target));
            }
            Event::Echo(e) if e.kind == EchoEvent::BOWED => notes.push(Note::Bowed(e.echo)),
            Event::Life(LifeEvent::Rested { .. }) => notes.push(Note::Rested),
            Event::Pickup(_) | Event::Quest(_) => notes.push(Note::Changed),
            _ => {}
        }
    }
    world.state.marks.seen = world.events.pushed();
    notes
}

/// A boss fell: the Marks that need the moment of the fall.
fn defeated(world: &mut World, b: usize) {
    let Some(id) = boss_id(world, b).map(str::to_owned) else {
        return;
    };
    if id == B12 && world.tick < SPEEDRUN_TICKS {
        earn(world, SPEEDRUNNER);
    }
    // B07 counts the player's actions in phase 1 and only phase 1; the counts are cleared on
    // the tick after the fall, so this tick still has them.
    if id == B07 && not_overfit(&world.b07.counts) {
        earn(world, GENERALIST);
    }
    let sigil = SIGILS.iter().any(|(s, _)| *s == id);
    let t = &world.state.marks;
    if sigil && t.watch == b as u16 + 1 && !t.watch_hit {
        earn(world, UNBROKEN);
    }
}

/// True if no category is over [`GENERALIST_PERCENT`] of the counted actions. With nothing
/// counted there is nothing to be overfit to, so that holds too.
fn not_overfit(counts: &[u16]) -> bool {
    let total: u32 = counts.iter().map(|&c| u32::from(c)).sum();
    counts
        .iter()
        .all(|&c| u32::from(c) * 100 <= GENERALIST_PERCENT * total)
}

/// The player killed `target`: an ordinary enemy (not a boss or one of its parts, and not an
/// Echo) outside the Verge breaks Pacifist Path.
fn killed(world: &mut World, target: Id) {
    let Some(actor) = world.actors.get(target) else {
        return;
    };
    let Some(ai) = actor.ai.as_ref() else {
        return;
    };
    let spawn = ai.spawn;
    let ally = actor.is_ally();
    let part_of_boss = world
        .layout
        .bosses
        .iter()
        .any(|b| b.members.contains(&spawn));
    let in_verge = VERGE_AREAS.contains(&world.physics.scenario.as_str());
    if ally || part_of_boss || in_verge {
        return;
    }
    let n = world.state.flags.get(KILLS_AFTER_VERGE);
    world
        .state
        .flags
        .set(KILLS_AFTER_VERGE, n.saturating_add(1));
}

/// The player bowed to Echo `echo` in time. Only the PoliteLlama has a courtesy to earn.
fn bowed(world: &mut World, echo: u32) {
    if world
        .echo_table
        .get(echo)
        .is_some_and(|e| e.def.id == "echo_politellama")
    {
        earn(world, POLITE);
    }
}

/// Reader and Correspondent: the conversation open now is with the Ledger Stand or the Courier.
fn talked(world: &mut World) {
    let Some(t) = quests::current(world) else {
        return;
    };
    let (mark, id, want) = match t {
        Talk::Npc(i) => (
            CORRESPONDENT,
            world.layout.npcs.get(i).map(|n| n.def.id.as_str()),
            "npc_courier",
        ),
        Talk::Station(i) => (
            READER,
            world.layout.stations.get(i).map(|s| s.def.id.as_str()),
            "stn_ledger",
        ),
    };
    if id == Some(want) {
        earn(world, mark);
    }
}

/// Pacifist Path: the player has come to the Spire Gate with no kill after the Verge.
fn at_spire_gate(world: &mut World) {
    if is_earned(&world.state.flags, PACIFIST_PATH) || world.state.flags.get(KILLS_AFTER_VERGE) > 0
    {
        return;
    }
    let Some(door) = world.layout.doors.iter().find(|d| d.id == SPIRE_GATE) else {
        return;
    };
    let Some(p) = world.actors.get(world.player).filter(|p| !p.dead) else {
        return;
    };
    let (dx, dz) = (door.pos[0] - p.pos[0], door.pos[2] - p.pos[2]);
    if dx * dx + dz * dz <= GATE_REACH_M * GATE_REACH_M && (door.pos[1] - p.pos[1]).abs() <= 3.0 {
        earn(world, PACIFIST_PATH);
    }
}

/// Unbroken, step one: HP lost since last tick in the fight under watch.
fn watch_damage(world: &mut World) {
    if world.state.marks.watch == 0 {
        return;
    }
    let hp = world.actors.get(world.player).map_or(0, |p| p.hp);
    let t = &mut world.state.marks;
    if hp < t.watch_hp {
        t.watch_hit = true;
    }
    t.watch_hp = hp;
}

/// Unbroken, step two: puts the Great Sigil fight that is on (sealed in or fighting) under
/// watch, and drops the watch when none is.
fn refresh_watch(world: &mut World) {
    let on = world.bosses.iter().enumerate().find_map(|(b, r)| {
        let fighting = matches!(r.stage, Stage::Intro { .. } | Stage::Fight);
        let sigil = boss_id(world, b).is_some_and(|id| SIGILS.iter().any(|(s, _)| *s == id));
        (fighting && sigil).then_some(b)
    });
    let hp = world.actors.get(world.player).map_or(0, |p| p.hp);
    let t = &mut world.state.marks;
    match on {
        Some(b) if t.watch != b as u16 + 1 => {
            t.watch = b as u16 + 1;
            t.watch_hp = hp;
            t.watch_hit = false;
        }
        Some(_) => {}
        None => {
            t.watch = 0;
            t.watch_hp = 0;
            t.watch_hit = false;
        }
    }
}

/// The Marks that are facts about the flags.
fn sweep(world: &mut World) {
    let f = &world.state.flags;
    let beaten = |id: &str| f.is_set(&flags::boss_defeated(id));
    let mut want = [false; MARK_COUNT];
    want[FIRST_LIGHT] = f
        .iter()
        .any(|(k, v)| v > 0 && k.starts_with("lantern.") && k.ends_with(".lit"));
    want[WOKEN] = beaten(B01);
    want[CHRONICLER] = PAGES
        .iter()
        .all(|p| f.is_set(&crate::pickups::taken_flag(p)));
    want[COLLECTOR] = RELICS.iter().all(|r| f.is_set(&flags::codex(r)));
    for (boss, mark) in SIGILS {
        want[mark] = beaten(boss);
    }
    want[EVER_AFTER] = beaten(B12);
    want[THOROUGH] = BOSS_IDS.iter().all(|b| beaten(b));
    for (i, w) in want.into_iter().enumerate() {
        if w {
            earn(world, i);
        }
    }
}

/// The player's in-game time in ticks (30 a second). The tick counter, never the wall clock.
pub fn play_ticks(world: &World) -> u64 {
    world.tick
}
