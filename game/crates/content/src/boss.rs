//! Boss definitions (`bosses/<id>.ron`, one boss per file). See
//! docs/game/05-enemies-and-bosses.md, "Boss framework" and "Fairness rules".
//!
//! A boss file holds the boss's **body** (an [`EnemyDef`]: HP, poise, and the option
//! table, with each option gated on `Phase(n)` for the phase it belongs to), its extra
//! **parts** (the Choir's other members, the Colossus's legs), the phase thresholds with
//! their transition moves, the rewards, and the fight's music. The core spawns the body
//! and the parts as AI actors that skip the attack-token pools (`lf_core::ai::boss`).
//!
//! Phase 1 needs no entry. `phases[0]` starts phase 2, `phases[1]` phase 3, and so on.

use serde::{Deserialize, Serialize};

use crate::enemy::EnemyDef;

/// One boss.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossDef {
    /// Registry ID, for example `b01_hollow_sentry`. Also the flag key:
    /// `boss.<id>.defeated`.
    pub id: String,
    /// The name on the boss bar.
    pub name: String,
    /// The main actor. Its ID needs a registry entry of its own.
    pub body: EnemyDef,
    /// More actors in the fight. Empty for a single-body boss.
    #[serde(default)]
    pub parts: Vec<BossPart>,
    /// How long the core waits after the player crosses the fog gate before the fight
    /// starts (docs "Boss framework", Fog gate: 90 ticks, a skippable 3 s camera move).
    #[serde(default = "default_intro_ticks")]
    pub intro_ticks: u16,
    /// The phases after the first, in order of falling HP.
    #[serde(default)]
    pub phases: Vec<BossPhase>,
    #[serde(default)]
    pub rewards: BossRewards,
    /// An Echo Stone stands in the arena (every boss except B01 and B12's second phase).
    #[serde(default = "default_true")]
    pub echo_stone: bool,
    #[serde(default)]
    pub music: BossMusic,
}

fn default_intro_ticks() -> u16 {
    90
}

fn default_true() -> bool {
    true
}

/// An extra actor in a boss fight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossPart {
    /// Its definition. Needs its own registry entry.
    pub def: EnemyDef,
    /// Where it stands, relative to the body's spawn, in meters.
    pub offset: [f32; 3],
    /// The fight ends only when every required part and the body are dead, and the phase
    /// thresholds read their combined HP (the Choir's members). A leg that only staggers
    /// the Colossus isn't required.
    #[serde(default)]
    pub required: bool,
}

/// A phase change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossPhase {
    /// The phase starts when the fight's HP fraction (0 to 1) falls below this.
    pub below: f32,
    /// The transition move the body plays first: 45 ticks, invulnerable throughout.
    pub transition: String,
}

/// What defeating the boss grants (docs "Boss framework", Rewards). The world flag
/// `boss.<id>.defeated` is always raised.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct BossRewards {
    pub glimmer: u32,
    pub items: Vec<BossItem>,
    /// The codex entry the world flag `codex.<entry>` unlocks. Empty for none.
    pub codex: String,
}

/// A reward item and how many.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossItem {
    pub item: String,
    pub count: u16,
}

/// The fight's music, as stem names for the renderer. Phase 2 adds a stem.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct BossMusic {
    pub intro: String,
    pub loop_stem: String,
    pub phase2_stem: String,
}

impl BossDef {
    /// Every actor definition: the body, then the parts.
    pub fn actors(&self) -> impl Iterator<Item = &EnemyDef> {
        std::iter::once(&self.body).chain(self.parts.iter().map(|p| &p.def))
    }

    /// Every registry ID the definition references, with its field path.
    pub fn references(&self) -> Vec<(String, &str)> {
        let mut out = vec![("id".to_owned(), self.id.as_str())];
        for (i, p) in self.phases.iter().enumerate() {
            out.push((format!("phases[{i}].transition"), p.transition.as_str()));
        }
        for (i, r) in self.rewards.items.iter().enumerate() {
            out.push((format!("rewards.items[{i}].item"), r.item.as_str()));
        }
        out
    }

    /// Shape problems the core relies on, as `(field, message)`.
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if self.name.is_empty() {
            out.push(("name".to_owned(), "the boss bar needs a name".to_owned()));
        }
        let mut last = 1.0_f32;
        for (i, p) in self.phases.iter().enumerate() {
            if !(p.below > 0.0 && p.below < last) {
                out.push((
                    format!("phases[{i}].below"),
                    "thresholds must fall: above 0 and below the previous phase's".to_owned(),
                ));
            }
            last = p.below;
        }
        // `Condition::Phase` is a u8, and phase 1 is the first.
        if self.phases.len() > 8 {
            out.push(("phases".to_owned(), "at most 9 phases".to_owned()));
        }
        for (i, r) in self.rewards.items.iter().enumerate() {
            if r.count == 0 {
                out.push((
                    format!("rewards.items[{i}].count"),
                    "must be positive".to_owned(),
                ));
            }
        }
        for (i, d) in self.actors().enumerate() {
            let at = if i == 0 {
                "body".to_owned()
            } else {
                format!("parts[{}].def", i - 1)
            };
            for (field, message) in d.problems() {
                out.push((format!("{at}.{field}"), message));
            }
        }
        out
    }
}

/// The tick length of a phase transition move (docs "Boss framework", Phases).
pub const TRANSITION_TICKS: u16 = 45;
/// The shortest telegraph, in ticks (docs "Fairness rules").
pub const MIN_TELEGRAPH_TICKS: u16 = 12;
/// The shortest punish window after a combo, in ticks (docs "Fairness rules").
pub const MIN_PUNISH_TICKS: u16 = 20;
/// The smallest leash a boss actor may have, in meters. The arena is sealed, so a boss
/// never leashes; the core keeps its actors in Combat while the fight lasts.
pub const MIN_LEASH_M: f32 = 50.0;

/// The phase an option belongs to, or `None` if it applies in every phase. An option
/// gated on two different phases never applies; `Some(0)` stands for that.
fn option_phase(o: &crate::enemy::AiOption) -> Option<u8> {
    use crate::enemy::Condition;
    let mut phase = None;
    for c in &o.when {
        if let Condition::Phase(p) = *c {
            match phase {
                None => phase = Some(p),
                Some(q) if q != p => return Some(0),
                Some(_) => {}
            }
        }
    }
    phase
}

fn applies(o: &crate::enemy::AiOption, phase: u8) -> bool {
    option_phase(o).is_none_or(|p| p == phase)
}

/// The fairness rules that can be checked statically (docs "Fairness rules"), the phase
/// rules, and the boss scenarios' references. What this can't see: the arena's escape
/// directions and darkness, which need the level format (WP-040); a combo's delayed first
/// hit (no data says a hit is delayed); a sound or ground marker as a telegraph; and that
/// tracking stops 4 ticks before the active frames (the core enforces that at run time,
/// `lf_core::ai::boss`).
pub fn check(content: &crate::Content) -> Vec<crate::validate::Issue> {
    use crate::validate::{Issue, Severity};
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (file, boss) in &content.bosses {
        let mut push = |severity, field: String, message: String| {
            out.push(Issue {
                severity,
                file: file.clone(),
                item: boss.id.clone(),
                field,
                message,
            });
        };
        if !seen.insert(boss.id.as_str()) {
            push(Severity::Error, "id".into(), "duplicate boss ID".into());
        }
        for (field, message) in boss.problems() {
            push(Severity::Error, field, message);
        }
        let phases = boss.phases.len() as u8 + 1;
        for (ai, def) in boss.actors().enumerate() {
            let at = if ai == 0 {
                "body".to_owned()
            } else {
                format!("parts[{}].def", ai - 1)
            };
            if !def.ignores_tokens {
                push(
                    Severity::Error,
                    format!("{at}.ignores_tokens"),
                    "boss actors skip the attack-token pools: set ignores_tokens".into(),
                );
            }
            if def.leash_m.is_none_or(|m| m < MIN_LEASH_M) {
                push(
                    Severity::Error,
                    format!("{at}.leash_m"),
                    format!("a boss never leashes: set leash_m to at least {MIN_LEASH_M}"),
                );
            }
            for (i, o) in def.options.iter().enumerate() {
                if option_phase(o).is_some_and(|p| p == 0 || p > phases) {
                    push(
                        Severity::Error,
                        format!("{at}.options[{i}].when"),
                        format!("Phase must name a phase from 1 to {phases}, once"),
                    );
                }
            }
            if ai == 0 {
                // Every phase has an attack, and each later phase adds a move.
                let moves_in = |p: u8| -> std::collections::BTreeSet<&str> {
                    def.options
                        .iter()
                        .filter(|o| applies(o, p))
                        .filter_map(|o| o.move_id.as_deref())
                        .collect()
                };
                for p in 1..=phases {
                    let now = moves_in(p);
                    if now.is_empty() {
                        push(
                            Severity::Error,
                            format!("{at}.options"),
                            format!("phase {p} has no attack move"),
                        );
                    } else if p > 1 && now.is_subset(&moves_in(p - 1)) {
                        push(
                            Severity::Error,
                            format!("{at}.options"),
                            format!(
                                "phase {p} adds no new move: a phase must add behavior, not only damage"
                            ),
                        );
                    }
                }
            }
            for (i, o) in def.options.iter().enumerate() {
                if let Some(id) = &o.move_id {
                    check_attack(content, id, &format!("{at}.options[{i}].move"), &mut push);
                }
            }
        }
        for (i, p) in boss.phases.iter().enumerate() {
            let field = format!("phases[{i}].transition");
            let Some(m) = content.move_by_id(&p.transition) else {
                push(Severity::Error, field, "the move is not defined".into());
                continue;
            };
            if m.total_frames != TRANSITION_TICKS {
                push(
                    Severity::Error,
                    field.clone(),
                    format!("a transition move is {TRANSITION_TICKS} ticks long"),
                );
            }
            let covered = m
                .iframes
                .is_some_and(|w| w.from == 0 && w.to + 1 >= m.total_frames);
            if !covered {
                push(
                    Severity::Error,
                    field.clone(),
                    "a transition move is invulnerable throughout: iframes must cover every frame"
                        .into(),
                );
            }
            if !m.hitboxes.is_empty() || !m.projectiles.is_empty() {
                push(
                    Severity::Warning,
                    field,
                    "a transition move shouldn't attack".into(),
                );
            }
        }
    }
    for (file, sc) in &content.scenarios {
        for (i, b) in sc.bosses.iter().enumerate() {
            let mut issue = |field: String, message: &str| {
                out.push(Issue {
                    severity: Severity::Error,
                    file: file.clone(),
                    item: b.name.clone(),
                    field,
                    message: message.to_owned(),
                });
            };
            if content.boss_by_id(&b.boss).is_none() {
                issue(format!("bosses[{i}].boss"), "the boss is not defined");
            }
            if content.ai.is_none() {
                issue(format!("bosses[{i}]"), "bosses need ai.ron");
            }
        }
    }
    out
}

/// The attack rules for the move an option starts: telegraph, punish window after the
/// combo's last move, and a flash on grabs and unblockable hits.
fn check_attack(
    content: &crate::Content,
    id: &str,
    field: &str,
    push: &mut impl FnMut(crate::validate::Severity, String, String),
) {
    use crate::moves::HitTag;
    use crate::validate::Severity::{Error, Warning};
    let Some(head) = content.move_by_id(id) else {
        return; // The enemy checks report an undefined move.
    };
    let first = |m: &crate::moves::Move| {
        m.hitboxes
            .iter()
            .map(|h| h.frames.0)
            .chain(m.projectiles.iter().map(|p| p.frame))
            .min()
    };
    let last = |m: &crate::moves::Move| {
        m.hitboxes
            .iter()
            .map(|h| h.frames.1)
            .chain(m.projectiles.iter().map(|p| p.frame))
            .max()
    };
    let Some(start) = first(head) else {
        push(
            Warning,
            field.to_owned(),
            "the move has no hitbox or projectile".into(),
        );
        return;
    };
    if start < MIN_TELEGRAPH_TICKS {
        push(
            Error,
            field.to_owned(),
            format!(
                "telegraph is {start} ticks, under the {MIN_TELEGRAPH_TICKS} every boss attack needs"
            ),
        );
    }
    // Follow the combo to its last move; a cycle ends the walk (the move checks cover it).
    let mut end = head;
    let mut visited = vec![head];
    while let Some(next) = end
        .next_chain
        .as_deref()
        .and_then(|n| content.move_by_id(n))
    {
        if visited.iter().any(|m| m.id == next.id) {
            break;
        }
        visited.push(next);
        end = next;
    }
    if let Some(l) = last(end) {
        let punish = end.total_frames.saturating_sub(l + 1);
        if punish < MIN_PUNISH_TICKS {
            push(
                Error,
                field.to_owned(),
                format!(
                    "the combo ends with a {punish}-tick punish window, under the {MIN_PUNISH_TICKS} it needs"
                ),
            );
        }
    }
    // Grabs and unblockable hits flash a color of their own before they land.
    for m in visited {
        for (tag, flash) in [
            (HitTag::Grab, "flash_grab"),
            (HitTag::Unblockable, "flash_unblockable"),
        ] {
            let Some(hit) = m.hitboxes.iter().find(|h| h.tags.contains(&tag)) else {
                continue;
            };
            if !m.vfx.iter().any(|(f, n)| n == flash && *f < hit.frames.0) {
                push(
                    Error,
                    field.to_owned(),
                    format!("{} needs a \"{flash}\" vfx before its hit", m.id),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Content;
    use crate::moves::{FrameWindow, HitTag};
    use crate::validate::{Severity, Validate, has_errors};

    fn content() -> Content {
        Content::load_embedded().unwrap()
    }

    fn errors(c: &Content) -> Vec<String> {
        c.validate()
            .into_iter()
            .filter(|i| i.severity == Severity::Error)
            .map(|i| format!("{}: {}", i.field, i.message))
            .collect()
    }

    fn edit_move(c: &mut Content, id: &str, f: impl FnOnce(&mut crate::moves::Move)) {
        let m = c
            .moves
            .values_mut()
            .flatten()
            .find(|m| m.id == id)
            .expect("move");
        f(m);
    }

    #[test]
    fn the_test_boss_passes_every_check() {
        let c = content();
        assert!(!has_errors(&c.validate()), "{:?}", errors(&c));
        assert_eq!(c.boss_by_id("test_boss").unwrap().intro_ticks, 90);
        // The body and the part are enemies too, for the AI table.
        assert!(c.enemy_by_id("test_boss_body").is_some());
        assert!(c.enemy_by_id("test_boss_pillar").is_some());
    }

    #[test]
    fn a_short_telegraph_fails() {
        let mut c = content();
        edit_move(&mut c, "tb_swipe", |m| m.hitboxes[0].frames = (11, 14));
        assert!(errors(&c).iter().any(|e| e.contains("telegraph is 11")));
        edit_move(&mut c, "tb_swipe", |m| m.hitboxes[0].frames = (12, 15));
        assert!(!errors(&c).iter().any(|e| e.contains("telegraph")));
    }

    #[test]
    fn a_short_punish_window_fails_at_the_end_of_the_combo() {
        let mut c = content();
        // The swipe's last active frame is 21: a 41-frame move leaves 19 ticks.
        edit_move(&mut c, "tb_swipe", |m| m.total_frames = 41);
        assert!(errors(&c).iter().any(|e| e.contains("19-tick punish")));
        edit_move(&mut c, "tb_swipe", |m| m.total_frames = 42);
        assert!(!errors(&c).iter().any(|e| e.contains("punish")));
    }

    #[test]
    fn grabs_and_unblockable_hits_need_their_own_flash() {
        let mut c = content();
        edit_move(&mut c, "tb_slam", |m| {
            m.hitboxes[0].tags = vec![HitTag::Grab]
        });
        assert!(errors(&c).iter().any(|e| e.contains("flash_grab")));
        edit_move(&mut c, "tb_slam", |m| {
            m.vfx.push((10, "flash_grab".to_owned()));
        });
        assert!(!errors(&c).iter().any(|e| e.contains("flash")));
        edit_move(&mut c, "tb_slam", |m| {
            m.hitboxes[0].tags = vec![HitTag::Unblockable];
        });
        assert!(errors(&c).iter().any(|e| e.contains("flash_unblockable")));
    }

    #[test]
    fn a_transition_is_45_ticks_and_invulnerable_throughout() {
        let mut c = content();
        edit_move(&mut c, "tb_roar", |m| {
            m.iframes = Some(FrameWindow { from: 0, to: 43 });
        });
        assert!(errors(&c).iter().any(|e| e.contains("invulnerable")));
        edit_move(&mut c, "tb_roar", |m| {
            m.iframes = Some(FrameWindow { from: 1, to: 44 });
        });
        assert!(errors(&c).iter().any(|e| e.contains("invulnerable")));
        edit_move(&mut c, "tb_roar", |m| {
            m.iframes = Some(FrameWindow { from: 0, to: 49 });
            m.total_frames = 50;
        });
        assert!(errors(&c).iter().any(|e| e.contains("45 ticks")));
    }

    #[test]
    fn a_phase_must_add_a_move_and_every_phase_needs_an_attack() {
        let mut c = content();
        let boss = c.bosses.get_mut("bosses/test_boss.ron").unwrap();
        // Drop the slam: phase 2 now has only what phase 1 has.
        boss.body
            .options
            .retain(|o| o.move_id.as_deref() != Some("tb_slam"));
        assert!(
            errors(&c)
                .iter()
                .any(|e| e.contains("phase 2 adds no new move"))
        );
        // Gate the swipe on phase 2: phase 1 then has no attack.
        let boss = c.bosses.get_mut("bosses/test_boss.ron").unwrap();
        boss.body.options[0]
            .when
            .push(crate::enemy::Condition::Phase(2));
        assert!(
            errors(&c)
                .iter()
                .any(|e| e.contains("phase 1 has no attack"))
        );
        // A phase that doesn't exist.
        let boss = c.bosses.get_mut("bosses/test_boss.ron").unwrap();
        boss.body.options[0].when = vec![crate::enemy::Condition::Phase(3)];
        assert!(errors(&c).iter().any(|e| e.contains("from 1 to 2")));
    }

    #[test]
    fn bosses_skip_tokens_never_leash_and_thresholds_fall() {
        let mut c = content();
        let boss = c.bosses.get_mut("bosses/test_boss.ron").unwrap();
        boss.body.ignores_tokens = false;
        boss.body.leash_m = Some(30.0);
        boss.phases.push(BossPhase {
            below: 0.7,
            transition: "tb_roar".to_owned(),
        });
        let e = errors(&c).join("\n");
        assert!(e.contains("ignores_tokens"), "{e}");
        assert!(e.contains("leash_m"), "{e}");
        assert!(e.contains("phases[1].below"), "{e}");
    }

    #[test]
    fn an_undefined_boss_in_a_scenario_is_an_error() {
        let mut c = content();
        c.scenarios
            .get_mut("scenarios/boss_gate.ron")
            .unwrap()
            .bosses[0]
            .boss = "nobody".to_owned();
        assert!(
            errors(&c)
                .iter()
                .any(|e| e.contains("the boss is not defined"))
        );
    }
}
