//! Enemy definitions (`enemies/<id>.ron`, one enemy per file) and the shared AI tuning
//! (`ai.ron`). See docs/game/05-enemies-and-bosses.md, "AI framework".
//!
//! An enemy lists **options**: attack moves and built-in behaviors, each with conditions,
//! a weight, and a cooldown. The core's AI think step filters them by condition and
//! samples one by weight from the `ai` random stream (`lf_core::ai::options`).

use serde::{Deserialize, Serialize};

use crate::moves::StatusKind;
use crate::scenario::{DefenseData, ResistData};
use crate::weapons::GuardStats;

/// One enemy type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyDef {
    /// Registry ID, for example `gray_knight`.
    pub id: String,
    /// The renderer's model name.
    #[serde(default)]
    pub model: String,
    pub hp: u32,
    /// Poise (docs/game/03-combat.md, "Poise and stance"). 0 means it never staggers.
    #[serde(default)]
    pub poise: u16,
    /// Stance meter. 0 means none, as for regular enemies.
    #[serde(default)]
    pub stance: u16,
    #[serde(default)]
    pub defense: DefenseData,
    #[serde(default)]
    pub resist: ResistData,
    /// Statuses it can't build up at all.
    #[serde(default)]
    pub immune: Vec<StatusKind>,
    /// Delirium staggers it.
    #[serde(default)]
    pub can_go_mad: bool,
    /// Glimmer dropped on death. Not paid out yet (enemy drops arrive later).
    #[serde(default)]
    pub glimmer: u32,
    /// Sight and hearing. `None` uses the defaults in `ai.ron`.
    #[serde(default)]
    pub perception: Option<Perception>,
    /// The moveset file under `moves/`, for tools. The options name the moves.
    #[serde(default)]
    pub moveset: Option<String>,
    /// Leash distance from the spawn, in meters. `None` uses `ai.ron`'s `leash_m`.
    #[serde(default)]
    pub leash_m: Option<f32>,
    /// Ignores attack tokens (bosses and their adds; docs "Attack tokens").
    #[serde(default)]
    pub ignores_tokens: bool,
    /// Attack rating: the physical attack rating of every hit it lands, in the damage
    /// formula's `AR` (docs/game/03-combat.md, "Damage dealt"). 0 deals the minimum of 1.
    #[serde(default)]
    pub attack: u32,
    /// Stamina a raised guard spends on blocks. 0 means none: a guard breaks on the first
    /// block. Only an enemy with `guard` needs any.
    #[serde(default)]
    pub stamina: u16,
    /// Block stats while its `Guard` behavior holds the guard up. `None`: it can't block.
    #[serde(default)]
    pub guard: Option<GuardStats>,
    /// Walk and run speed as a percent of the shared speeds in `ai.ron`.
    #[serde(default = "full_speed")]
    pub speed_percent: u16,
    /// It carries a lit lantern and is never unaware, so it can't be backstabbed (the
    /// Miner husk).
    #[serde(default)]
    pub backstab_immune: bool,
    /// It gets back up after dying, unless something stops it (the Whitewarden Skeleton).
    #[serde(default)]
    pub revive: Option<Revive>,
    /// What it picks from in Combat.
    pub options: Vec<AiOption>,
    /// Item drops. Not rolled yet (enemy drops arrive later).
    #[serde(default)]
    pub drops: Vec<EnemyDrop>,
}

fn full_speed() -> u16 {
    100
}

/// A revive rule (docs "Enemy roster", Whitewarden Skeleton): `after_ticks` after dying
/// the enemy stands up again with `hp_percent` of its HP, unless a [`ReviveStop`] applies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revive {
    /// Ticks lying dead before it stands up.
    pub after_ticks: u16,
    /// HP it gets back up with, as a percent of its maximum.
    pub hp_percent: u8,
    /// Any one of these on its death keeps it down for good.
    pub stopped_by: Vec<ReviveStop>,
}

/// What keeps a reviver dead.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ReviveStop {
    /// The killing blow dealt Radiance damage.
    Radiance,
    /// The killing blow was a critical hit (a riposte or a backstab).
    Critical,
    /// It died within this many meters of a lit Lantern (the scenario's light sources
    /// stand in for braziers until those exist).
    LitLantern(f32),
}

/// Sight cone and hearing (docs "Perception").
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Perception {
    /// Full cone angle, in degrees.
    pub sight_deg: f32,
    /// Sight range, in meters.
    pub sight_m: f32,
    /// Scales every noise radius it hears.
    pub hearing_mult: f32,
}

/// One entry of an enemy's option table: exactly one of `move_id` and `behavior`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiOption {
    /// An attack move to start (written `move:` in the data).
    #[serde(rename = "move", default)]
    pub move_id: Option<String>,
    /// A built-in behavior to run.
    #[serde(default)]
    pub behavior: Option<Behavior>,
    /// Every condition must hold for the option to be picked.
    #[serde(default)]
    pub when: Vec<Condition>,
    /// Relative chance among the options whose conditions hold. At least 1.
    pub weight: u16,
    /// Ticks after the option is picked before it can be picked again.
    #[serde(default)]
    pub cooldown: u16,
}

/// The built-in behaviors (docs "Decision making"). `dur` is in ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Behavior {
    /// Close in on the target at run speed.
    Approach { dur: u16 },
    /// Circle the target at walk speed, facing it.
    Strafe { dur: u16 },
    /// Back away from the target at walk speed, facing it.
    Retreat { dur: u16 },
    /// Hold position facing the target. The guard isn't raised yet (see
    /// `lf_core::ai::behaviors`).
    Guard { dur: u16 },
    /// Stand facing the target.
    Wait { dur: u16 },
    /// Run away from the target.
    Flee { dur: u16 },
    /// Walk the spawn's patrol waypoints. An Idle behavior; not an option.
    Patrol,
}

impl Behavior {
    /// The behavior's length in ticks; `None` for `Patrol`, which has none.
    pub fn dur(self) -> Option<u16> {
        match self {
            Self::Approach { dur }
            | Self::Strafe { dur }
            | Self::Retreat { dur }
            | Self::Guard { dur }
            | Self::Wait { dur }
            | Self::Flee { dur } => Some(dur),
            Self::Patrol => None,
        }
    }
}

/// An option condition (docs "Decision making"). Distances are horizontal, in meters;
/// angles in degrees; HP fractions from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    /// The target is closer than this.
    DistBelow(f32),
    /// The target is farther than this.
    DistAbove(f32),
    /// The target is at least the first and less than the second away.
    DistBetween(f32, f32),
    /// The target is within this many degrees of the enemy's facing (either side).
    FacingWithin(f32),
    /// The enemy is within this many degrees of the target's facing: the target faces it.
    TargetFacingWithin(f32),
    /// The enemy's HP fraction is below this.
    HpBelow(f32),
    /// The enemy's HP fraction is at least this.
    HpAbove(f32),
    /// The target's HP fraction is below this.
    TargetHpBelow(f32),
    /// The target is in an attack move.
    TargetAttacking,
    /// The target is drinking a flask.
    TargetHealing,
    /// The target's guard is up.
    TargetGuarding,
    /// The target is casting. Always false until spells exist.
    TargetCasting,
    /// The boss phase equals this (0 for regular enemies; WP-033 sets it).
    Phase(u8),
    /// The enemy lost HP within this many ticks (an archer repositions after a hit).
    HurtWithin(u16),
}

/// An item drop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyDrop {
    pub item: String,
    /// 0 to 1.
    pub chance: f32,
}

/// Shared AI tuning (`ai.ron`). Sources are in the file's comments.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiTuning {
    /// An AI actor thinks every this many ticks, staggered by arena index.
    pub think_every_ticks: u8,
    /// Perception for an enemy that lists none.
    pub perception: Perception,
    /// Sight range in darkness without a player light. Unused until darkness exists.
    pub dark_sight_m: f32,
    /// Eye height above the feet, the start of the sight ray.
    pub eye_height_m: f32,
    /// Chest height above the target's feet, the end of the sight ray.
    pub chest_height_m: f32,
    pub noise: NoiseRadii,
    /// How long a Suspicious actor investigates a noise before giving up.
    pub suspicious_ticks: u16,
    /// The Aware notice lasts a random count of ticks in this range (inclusive).
    pub notice_ticks: (u16, u16),
    /// Combat ends when the actor is farther than this from its spawn.
    pub leash_m: f32,
    /// Combat ends when the actor hasn't seen its target for this long.
    pub lost_sight_ticks: u16,
    /// Melee attack tokens per target.
    pub melee_tokens: u8,
    /// Ranged attack tokens per target.
    pub ranged_tokens: u8,
    pub walk_speed: f32,
    pub run_speed: f32,
    /// How fast an AI actor turns outside a move.
    pub turn_deg_per_tick: f32,
    /// Close enough to a goal (noise, waypoint, spawn) to count as there.
    pub arrive_m: f32,
    /// Approach stops this close to the target.
    pub approach_stop_m: f32,
    /// When no option fits, the actor strafes (no token) or approaches for this long.
    pub fallback_ticks: u16,
    /// How far ahead `desired_move` checks for a wall in the way.
    pub probe_m: f32,
    /// A nav path is replanned once its goal has moved this far, in meters (WP-032).
    pub repath_m: f32,
    /// After a path request fails, an actor steers straight for this many ticks before it
    /// asks again (WP-032).
    pub path_retry_ticks: u16,
}

/// Noise radius of each player action, in meters (docs "Perception", Hearing).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoiseRadii {
    pub walk_m: f32,
    pub run_m: f32,
    pub sprint_m: f32,
    pub roll_m: f32,
    pub attack_m: f32,
    pub landing_m: f32,
}

impl EnemyDef {
    /// Every registry ID the definition references, with its field path.
    pub fn references(&self) -> Vec<(String, &str)> {
        let mut out = vec![("id".to_owned(), self.id.as_str())];
        for (i, o) in self.options.iter().enumerate() {
            if let Some(m) = &o.move_id {
                out.push((format!("options[{i}].move"), m.as_str()));
            }
        }
        for (i, d) in self.drops.iter().enumerate() {
            out.push((format!("drops[{i}].item"), d.item.as_str()));
        }
        out
    }

    /// Shape problems the core relies on, as `(field, message)`.
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if self.hp == 0 {
            out.push(("hp".to_owned(), "must be positive".to_owned()));
        }
        if self.options.is_empty() {
            out.push((
                "options".to_owned(),
                "an enemy needs at least one option".to_owned(),
            ));
        }
        for (i, o) in self.options.iter().enumerate() {
            let f = |s: &str| format!("options[{i}].{s}");
            match (&o.move_id, &o.behavior) {
                (Some(_), Some(_)) | (None, None) => out.push((
                    format!("options[{i}]"),
                    "needs exactly one of move and behavior".to_owned(),
                )),
                (None, Some(Behavior::Patrol)) => out.push((
                    f("behavior"),
                    "Patrol is an Idle behavior, not a Combat option".to_owned(),
                )),
                (None, Some(b)) if b.dur() == Some(0) => {
                    out.push((f("behavior"), "dur must be positive".to_owned()));
                }
                _ => {}
            }
            if o.weight == 0 {
                out.push((f("weight"), "must be positive".to_owned()));
            }
            for (j, c) in o.when.iter().enumerate() {
                if let Condition::DistBetween(lo, hi) = c
                    && lo >= hi
                {
                    out.push((
                        f(&format!("when[{j}]")),
                        "DistBetween needs its first value below its second".to_owned(),
                    ));
                }
            }
        }
        if let Some(p) = &self.perception {
            out.extend(p.problems("perception"));
        }
        if self.speed_percent == 0 {
            out.push(("speed_percent".to_owned(), "must be positive".to_owned()));
        }
        if let Some(r) = &self.revive {
            if r.after_ticks == 0 {
                out.push((
                    "revive.after_ticks".to_owned(),
                    "must be positive".to_owned(),
                ));
            }
            if r.hp_percent == 0 || r.hp_percent > 100 {
                out.push((
                    "revive.hp_percent".to_owned(),
                    "must be 1 to 100".to_owned(),
                ));
            }
            if r.stopped_by
                .iter()
                .any(|s| matches!(s, ReviveStop::LitLantern(m) if *m <= 0.0))
            {
                out.push((
                    "revive.stopped_by".to_owned(),
                    "LitLantern needs a positive radius".to_owned(),
                ));
            }
        }
        if self.guard.is_some() && self.stamina == 0 {
            out.push((
                "stamina".to_owned(),
                "an enemy that guards needs stamina to block with".to_owned(),
            ));
        }
        if self.leash_m.is_some_and(|m| m <= 0.0) {
            out.push(("leash_m".to_owned(), "must be positive".to_owned()));
        }
        for (i, d) in self.drops.iter().enumerate() {
            if !(0.0..=1.0).contains(&d.chance) {
                out.push((format!("drops[{i}].chance"), "must be 0 to 1".to_owned()));
            }
        }
        out
    }
}

impl Perception {
    fn problems(&self, at: &str) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if !(self.sight_deg > 0.0 && self.sight_deg <= 360.0) {
            out.push((
                format!("{at}.sight_deg"),
                "must be above 0 and at most 360".to_owned(),
            ));
        }
        if self.sight_m < 0.0 || self.hearing_mult < 0.0 {
            out.push((
                at.to_owned(),
                "sight_m and hearing_mult can't be negative".to_owned(),
            ));
        }
        out
    }
}

impl AiTuning {
    /// Shape problems the core relies on, as `(field, message)`.
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut out = self.perception.problems("perception");
        if self.think_every_ticks == 0 {
            out.push((
                "think_every_ticks".to_owned(),
                "must be positive".to_owned(),
            ));
        }
        if self.notice_ticks.0 > self.notice_ticks.1 {
            out.push((
                "notice_ticks".to_owned(),
                "the range is reversed".to_owned(),
            ));
        }
        if self.melee_tokens == 0 || self.ranged_tokens == 0 {
            out.push((
                "melee_tokens".to_owned(),
                "token pools need at least one token".to_owned(),
            ));
        }
        out
    }
}
