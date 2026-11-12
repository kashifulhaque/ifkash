//! NPCs, stations, and their dialogue (`npcs/*.ron`, WP-068). See
//! docs/game/06-progression-and-systems.md, "NPCs and questlines", and
//! docs/game/08-portfolio-integration.md.
//!
//! One file per NPC (`npcs/archivist.ron` holds one [`NpcDef`]) plus `npcs/stations.ron`
//! (a list of [`StationDef`]). The simulation owns the dialogue: which NPC is being talked
//! to, which topic is open, and the quest flags. The browser draws the lines and the
//! facts. A fact is never written here: a topic names portfolio entries by [`FactRef`]
//! (section and slug), and the browser pulls their text from `src/lib/content.ts`.
//!
//! **No portfolio fact is locked behind a quest.** Every topic is open from the first
//! conversation; a quest only changes an NPC's greeting, adds a reward, and unlocks
//! codex flavor. `lf-core`'s dialogue test walks every topic and checks that every
//! portfolio entry is reachable this way.

use serde::{Deserialize, Serialize};

/// The section of [`FactRef::slug`] that names every item of its section, so adding an
/// entry to `content.ts` reaches the hub without editing this data.
pub const ALL_SLUGS: &str = "*";

/// A portfolio entry, by section and slug.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactRef {
    /// A `content.ts` section: `work`, `education`, `projects`, `resume`, `blog`, `contact`.
    pub section: String,
    /// An item's slug, or `*` for every item of the section.
    pub slug: String,
}

/// The services an NPC's topic can hand the player to. The simulation only records which
/// one; the Lantern menu and the shop do the work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Service {
    /// The Archivist's respec (the Lantern menu's Respec, which costs a Rewrite Quill).
    Respec,
    /// The Smith's weapon upgrades (the Lantern menu's Upgrade and Temper).
    Upgrade,
    /// The Peddler's wares.
    Shop,
}

/// One line of an **Ask about...** menu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topic {
    /// Unique within the NPC, lowercase with `_`.
    pub id: String,
    /// The menu entry, for example "Where the Wayfarer studied".
    pub label: String,
    /// The NPC's reply: one to three sentences in the fantasy voice. Facts are never here.
    pub line: String,
    /// The portfolio entries the reply shows in the plain facts panel.
    #[serde(default)]
    pub facts: Vec<FactRef>,
    /// A service the reply points to.
    #[serde(default)]
    pub service: Option<Service>,
}

/// A line for each state of a questline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateLines {
    pub before: String,
    pub during: String,
    pub after: String,
}

impl StateLines {
    /// The line for quest state 0 (before), 1 (during), or 2 and up (after).
    pub fn for_state(&self, state: u8) -> &str {
        match state {
            0 => &self.before,
            1 => &self.during,
            _ => &self.after,
        }
    }
}

/// What a quest needs the player to have brought.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Need {
    /// A world flag is set (a pickup placed in another area raises `pickup.<id>.taken`).
    Flag(String),
    /// The player holds at least one of an item (a registry ID).
    Item(String),
}

/// What turning a quest in gives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reward {
    /// World flags to raise.
    #[serde(default)]
    pub flags: Vec<String>,
    /// Codex entries to unlock, as flag suffixes such as `projects.opencode-theme`.
    #[serde(default)]
    pub codex: Vec<String>,
    /// Items (registry IDs) put in the inventory. An ID the item table doesn't know is
    /// only a flag (`quest.<id>.reward.<item>`), like a relic.
    #[serde(default)]
    pub items: Vec<String>,
}

/// A questline: before, during, and after.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestDef {
    /// Unique among quests; the state flag is `quest.<id>.state`.
    pub id: String,
    /// The quest's name in the journal.
    pub name: String,
    /// The **Ask about...** entry that takes the quest up and turns it in.
    pub label: String,
    /// The NPC's reply when the player asks before the quest starts (the offer), while it
    /// runs and is not finished (the reminder), and after it is done.
    pub replies: StateLines,
    /// What to bring. See `at_least`.
    pub needs: Vec<Need>,
    /// How many of `needs` have to be met; 0 means all of them. The Cartographer's pages
    /// use it: a work entry fewer than the five slots leaves one page unwritten.
    #[serde(default)]
    pub at_least: u8,
    /// The NPC's line when the player turns the quest in.
    pub done_line: String,
    pub reward: Reward,
}

/// An NPC.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpcDef {
    /// Registry ID, for example `npc_archivist`.
    pub id: String,
    /// The name above the dialogue.
    pub name: String,
    /// What the NPC does, one short phrase.
    pub role: String,
    /// The greeting: one line per quest state.
    pub greeting: StateLines,
    /// What the NPC says when the player ends the conversation.
    pub farewell: String,
    /// The **Ask about...** menu, in order.
    pub topics: Vec<Topic>,
    /// The questline; the Peddler has none.
    #[serde(default)]
    pub quest: Option<QuestDef>,
}

/// The kinds of station in Hollowhearth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StationKind {
    /// The Archive shelves: the blog's Letters.
    Archive,
    /// The Ledger Stand: the résumé. Opening it grants the Reader Mark.
    Ledger,
    /// The Mirror: the appearance preset and the dye.
    Mirror,
    /// The keepsake table: a project with no relic mapping, shown by its facts alone.
    Keepsakes,
    /// The Hearth: the owner's name and tagline.
    Hearth,
}

impl StationKind {
    /// The kind's number in the WASM words.
    pub fn code(self) -> u8 {
        match self {
            Self::Archive => 0,
            Self::Ledger => 1,
            Self::Mirror => 2,
            Self::Keepsakes => 3,
            Self::Hearth => 4,
        }
    }
}

/// A station's definition (`npcs/stations.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StationDef {
    /// Registry ID, for example `stn_archive`.
    pub id: String,
    pub kind: StationKind,
    /// The name above the panel.
    pub name: String,
    /// The line in the fantasy voice.
    pub line: String,
    /// The entries the panel shows in the plain facts panel.
    #[serde(default)]
    pub facts: Vec<FactRef>,
    /// For the keepsake table: show only the projects that have no relic mapping in
    /// `portfolio.ron`.
    #[serde(default)]
    pub unmapped_only: bool,
    /// For the Mirror: the appearance presets by name.
    #[serde(default)]
    pub presets: Vec<String>,
    /// For the Mirror: the dyes, the first of which should need no flag ("None").
    #[serde(default)]
    pub dyes: Vec<Dye>,
}

/// A dye the Mirror can apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dye {
    pub name: String,
    /// The world flag that says the player owns it; `None` for a dye everyone has.
    #[serde(default)]
    pub unlock_flag: Option<String>,
}

/// The file `npcs/stations.ron` holds a `Vec<StationDef>`.
pub const STATIONS_FILE: &str = "npcs/stations.ron";

impl StationDef {
    /// The definition as JSON, for the browser's dialogue (WP-068).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl NpcDef {
    /// The definition as JSON, for the browser's dialogue (WP-068).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// The registry IDs this NPC refers to, with the field they are in.
    pub fn references(&self) -> Vec<(String, &str)> {
        let mut out = vec![("id".to_owned(), self.id.as_str())];
        if let Some(q) = &self.quest {
            for (i, n) in q.needs.iter().enumerate() {
                if let Need::Item(id) = n {
                    out.push((format!("quest.needs[{i}]"), id.as_str()));
                }
            }
            for (i, id) in q.reward.items.iter().enumerate() {
                out.push((format!("quest.reward.items[{i}]"), id.as_str()));
            }
        }
        out
    }

    /// Whether `needs` is met by `have` satisfied entries.
    pub fn quest_met(&self, have: usize) -> bool {
        self.quest.as_ref().is_some_and(|q| q.met(have))
    }
}

impl QuestDef {
    /// How many needs have to be met.
    pub fn required(&self) -> usize {
        if self.at_least == 0 {
            self.needs.len()
        } else {
            usize::from(self.at_least).min(self.needs.len())
        }
    }

    /// Whether `have` satisfied needs finish the quest.
    pub fn met(&self, have: usize) -> bool {
        have >= self.required()
    }
}

// ---------------------------------------------------------------------------------------
// Validation

use std::collections::BTreeSet;

use crate::Content;
use crate::portfolio::PortfolioContent;
use crate::validate::{Issue, Severity};

/// The `content.ts` sections a [`FactRef`] may name.
pub const SECTIONS: [&str; 6] = ["work", "education", "projects", "resume", "blog", "contact"];

/// The topic number that takes up or turns in the NPC's quest. It is not an index into
/// `topics`: the quest has its own line in the menu, after the topics.
pub const QUEST_TOPIC: u8 = 254;

/// The topic number that goes back to the NPC's menu.
pub const BACK_TOPIC: u8 = 255;

/// A line longer than this is surely a mistake (docs "Dialogue rules": one to three
/// sentences).
const MAX_LINE_CHARS: usize = 400;

fn issue(file: &str, item: &str, field: impl Into<String>, message: impl Into<String>) -> Issue {
    Issue {
        severity: Severity::Error,
        file: file.to_owned(),
        item: item.to_owned(),
        field: field.into(),
        message: message.into(),
    }
}

/// Checks the NPCs and stations: unique topic IDs, real sections, and sensible lengths.
pub fn check(content: &Content) -> Vec<Issue> {
    let mut out = Vec::new();
    let mut quest_ids = BTreeSet::new();
    for (file, n) in &content.npcs {
        let mut at = |field: String, message: &str| out.push(issue(file, &n.id, field, message));
        if n.topics.is_empty() {
            at("topics".into(), "an NPC needs an Ask about menu");
        }
        if n.topics.len() >= usize::from(QUEST_TOPIC) {
            at("topics".into(), "too many topics");
        }
        let mut ids = BTreeSet::new();
        for (i, t) in n.topics.iter().enumerate() {
            if !ids.insert(t.id.as_str()) {
                at(format!("topics[{i}].id"), "duplicate topic ID");
            }
            if t.label.is_empty() || t.line.is_empty() {
                at(format!("topics[{i}]"), "needs a label and a line");
            }
            if t.line.chars().count() > MAX_LINE_CHARS {
                at(format!("topics[{i}].line"), "line is too long");
            }
            for (j, f) in t.facts.iter().enumerate() {
                if !SECTIONS.contains(&f.section.as_str()) {
                    at(
                        format!("topics[{i}].facts[{j}].section"),
                        "not a content.ts section",
                    );
                }
            }
        }
        if let Some(q) = &n.quest {
            if !quest_ids.insert(q.id.as_str()) {
                at("quest.id".into(), "duplicate quest ID");
            }
            if q.needs.is_empty() {
                at("quest.needs".into(), "a quest needs something");
            }
            if usize::from(q.at_least) > q.needs.len() {
                at("quest.at_least".into(), "more than the needs");
            }
            for (i, need) in q.needs.iter().enumerate() {
                if matches!(need, Need::Flag(f) if f.is_empty()) {
                    at(format!("quest.needs[{i}]"), "empty flag name");
                }
            }
        }
    }
    for s in &content.stations {
        for (j, f) in s.facts.iter().enumerate() {
            if !SECTIONS.contains(&f.section.as_str()) {
                out.push(issue(
                    STATIONS_FILE,
                    &s.id,
                    format!("facts[{j}].section"),
                    "not a content.ts section",
                ));
            }
        }
        if s.kind == StationKind::Mirror && (s.presets.is_empty() || s.dyes.is_empty()) {
            out.push(issue(
                STATIONS_FILE,
                &s.id,
                "presets",
                "the Mirror needs presets and dyes",
            ));
        }
    }
    out
}

/// Every `(section, slug)` of the portfolio export that the hub's NPCs and stations show,
/// with `*` expanded. A keepsake table's `unmapped_only` is ignored here: the entries it
/// might hide are shown by an NPC's topic.
pub fn shown_facts(content: &Content, portfolio: &PortfolioContent) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    let mut add = |f: &FactRef| {
        for s in portfolio.sections.iter().filter(|s| s.id == f.section) {
            for item in &s.items {
                let Some(slug) = &item.slug else { continue };
                if f.slug == ALL_SLUGS || &f.slug == slug {
                    out.insert((s.id.clone(), slug.clone()));
                }
            }
        }
    };
    for n in content.npcs.values() {
        n.topics.iter().flat_map(|t| &t.facts).for_each(&mut add);
    }
    for s in content.stations.iter().filter(|s| !s.unmapped_only) {
        s.facts.iter().for_each(&mut add);
    }
    out
}

/// Checks the NPCs' and stations' fact references against the exported portfolio: a slug
/// that does not exist is an error, and so is a portfolio entry the hub never shows
/// (every portfolio entry is reachable from the hub).
pub fn check_facts(content: &Content, portfolio: &PortfolioContent) -> Vec<Issue> {
    let mut out = Vec::new();
    let exists = |f: &FactRef| {
        f.slug == ALL_SLUGS
            || portfolio.sections.iter().any(|s| {
                s.id == f.section && s.items.iter().any(|i| i.slug.as_ref() == Some(&f.slug))
            })
    };
    for (file, n) in &content.npcs {
        for (i, t) in n.topics.iter().enumerate() {
            for (j, f) in t.facts.iter().enumerate() {
                if !exists(f) {
                    out.push(issue(
                        file,
                        &n.id,
                        format!("topics[{i}].facts[{j}]"),
                        format!("{}/{} is not in content.ts", f.section, f.slug),
                    ));
                }
            }
        }
    }
    for s in &content.stations {
        for (j, f) in s.facts.iter().enumerate() {
            if !exists(f) {
                out.push(issue(
                    STATIONS_FILE,
                    &s.id,
                    format!("facts[{j}]"),
                    format!("{}/{} is not in content.ts", f.section, f.slug),
                ));
            }
        }
    }
    let shown = shown_facts(content, portfolio);
    for s in &portfolio.sections {
        for item in &s.items {
            if let Some(slug) = &item.slug
                && !shown.contains(&(s.id.clone(), slug.clone()))
            {
                out.push(issue(
                    "npcs",
                    "hub",
                    format!("{}/{}", s.id, slug),
                    "no NPC topic or station shows this portfolio entry",
                ));
            }
        }
    }
    out
}
