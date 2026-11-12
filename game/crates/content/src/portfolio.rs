//! The portfolio mapping: `portfolio.ron` (game-side fields) checked against
//! `gen/portfolio_content.json` (exported from `src/lib/content.ts`).
//!
//! This is deliberately NOT part of [`crate::Content`], so it is not in the content
//! hash: editing portfolio text must never invalidate a replay. Load and validate it
//! separately with [`Portfolio::load_dir`] and [`Portfolio::validate`].
//!
//! The game item IDs in `portfolio.ron` (relics, Chronicle Pages) are not checked
//! against `registry.ron` yet. Later WPs add them as stubs.

use std::collections::BTreeSet;
use std::path::Path;

use serde::Deserialize;

use crate::validate::{Issue, Severity};

/// Sections whose items must each have a mapping; unmapped ones fall back to a
/// generic object.
const MAPPED_SECTIONS: [&str; 2] = ["work", "projects"];

/// A project that becomes a relic.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Relic {
    /// Slug of the item in the `projects` section.
    pub project: String,
    /// Game item ID (not yet in the registry).
    pub item: String,
    pub kind: String,
    /// One fantasy sentence about the item.
    pub flavor: String,
    /// One sentence bridging to the project.
    pub bridge: String,
}

/// A Chronicle Page slot and the work entry it holds, if any.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Page {
    pub slot: String,
    /// Game item ID (not yet in the registry).
    pub item: String,
    /// Slug of the item in the `work` section; `None` for an empty slot.
    pub work: Option<String>,
    pub framing: String,
}

/// The parsed `portfolio.ron`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Portfolio {
    pub relics: Vec<Relic>,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ContentItem {
    pub slug: Option<String>,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ContentSection {
    pub id: String,
    pub items: Vec<ContentItem>,
}

/// The parsed `gen/portfolio_content.json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PortfolioContent {
    pub sections: Vec<ContentSection>,
}

const RON_FILE: &str = "portfolio.ron";
const JSON_FILE: &str = "gen/portfolio_content.json";

impl Portfolio {
    /// Parses `portfolio.ron` text.
    pub fn from_ron(text: &str) -> Result<Self, String> {
        ron::from_str(text).map_err(|e| format!("{RON_FILE}: parse error: {e}"))
    }

    /// Loads `portfolio.ron` and `gen/portfolio_content.json` from a data directory.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<(Self, PortfolioContent), String> {
        let dir = dir.as_ref();
        let read = |name: &str| {
            let path = dir.join(name);
            std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))
        };
        let portfolio = Self::from_ron(&read(RON_FILE)?)?;
        let content = PortfolioContent::from_json(&read(JSON_FILE)?)?;
        Ok((portfolio, content))
    }

    /// Checks every mapping against the exported content.
    pub fn validate(&self, content: &PortfolioContent) -> Vec<Issue> {
        let mut out = Vec::new();
        let mut issue = |severity, item: &str, field: &str, message: String| {
            out.push(Issue {
                severity,
                file: RON_FILE.to_owned(),
                item: item.to_owned(),
                field: field.to_owned(),
                message,
            });
        };

        // Items with no slug can never be referenced.
        for section in &content.sections {
            for item in section.items.iter().filter(|i| i.slug.is_none()) {
                issue(
                    Severity::Error,
                    &item.title,
                    &format!("{JSON_FILE}: {}", section.id),
                    format!(
                        "item \"{}\" in section \"{}\" has no slug; add `slug` in src/lib/content.ts",
                        item.title, section.id
                    ),
                );
            }
        }

        let has = |section: &str, slug: &str| {
            content
                .section(section)
                .is_some_and(|s| s.items.iter().any(|i| i.slug.as_deref() == Some(slug)))
        };
        let mut mapped: BTreeSet<(&str, &str)> = BTreeSet::new();
        for (i, r) in self.relics.iter().enumerate() {
            mapped.insert(("projects", &r.project));
            if !has("projects", &r.project) {
                issue(
                    Severity::Error,
                    &r.item,
                    &format!("relics[{i}].project"),
                    format!(
                        "slug \"{}\" does not exist in content section \"projects\"",
                        r.project
                    ),
                );
            }
        }
        for (i, p) in self.pages.iter().enumerate() {
            if let Some(work) = &p.work {
                mapped.insert(("work", work));
                if !has("work", work) {
                    issue(
                        Severity::Error,
                        &p.item,
                        &format!("pages[{i}].work"),
                        format!("slug \"{work}\" does not exist in content section \"work\""),
                    );
                }
            }
        }

        for section_id in MAPPED_SECTIONS {
            let Some(section) = content.section(section_id) else {
                continue;
            };
            for item in &section.items {
                if let Some(slug) = &item.slug
                    && !mapped.contains(&(section_id, slug.as_str()))
                {
                    issue(
                        Severity::Warning,
                        slug,
                        &format!("{JSON_FILE}: {section_id}"),
                        format!(
                            "\"{slug}\" ({}) in section \"{section_id}\" is unmapped, uses the fallback",
                            item.title
                        ),
                    );
                }
            }
        }
        out
    }
}

impl PortfolioContent {
    /// Parses the exported JSON text.
    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| format!("{JSON_FILE}: parse error: {e}"))
    }

    fn section(&self, id: &str) -> Option<&ContentSection> {
        self.sections.iter().find(|s| s.id == id)
    }
}
