//! `BossArena`: one boss fight stepped with the default `MultiDiscrete` actions (WP-039).
//!
//! This is the Rust half of `Latticefall/BossArena-v0` (docs/game/09-rl-environment.md,
//! "Scenarios"). It owns the world, decodes the five action heads with
//! `lf_core::obs::ActionDecoder`, runs `frame_skip` ticks per step, and reports the
//! observation vectors from `lf_core::obs` plus the numbers the Python side turns into
//! rewards. The Gymnasium wrapper, the reward weights, and truncation live in Python
//! (`latticefall/envs.py`).
//!
//! # Episode start and end
//!
//! `reset` builds the scenario's world and then walks the player forward through the fog
//! gate (stick straight ahead while the fight is dormant, released during the intro) until
//! the fight starts. The episode starts there, inside the arena with the fight on: the
//! walk-in ticks aren't part of it. Walking in, rather than spawning past the gate, keeps
//! the boss framework's own gate and intro logic in charge.
//!
//! A step stops early on the tick the player dies or the boss falls. After that the
//! episode is over: further steps do nothing, so the world never reaches the respawn and
//! the boss reset that follows a death.

use std::sync::Arc;

use lf_content::Content;
use lf_core::ai::boss::Stage;
use lf_core::events::{BossEvent, Event, LifeEvent};
use lf_core::input::{ActionFrame, TickInput};
use lf_core::obs::{self, ActionDecoder};
use lf_core::tick::tick;
use lf_core::world::World;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use super::{load_content, value_err};

/// The longest walk from the scenario spawn to the start of the fight, in ticks. Only
/// stops a scenario whose gate the player can't reach.
const WALK_IN_LIMIT: u64 = 1800;

/// What one step (or a reset) reports: the observation, the mask, and the reward inputs.
type Report<'py> = (
    Bound<'py, PyBytes>, // proprio, f32 little endian
    Bound<'py, PyBytes>, // target, f32 little endian
    (i32, i32),          // move ids
    Bound<'py, PyBytes>, // action mask, one byte per head value
    f32,                 // player HP fraction
    f32,                 // boss HP fraction
    u8,                  // boss phase
    u64,                 // ticks since the fight started
    bool,                // the player died
    bool,                // the boss is defeated
    u32,                 // stance breaks dealt to the boss this step
    u32,                 // phase transitions started this step
);

/// One boss fight.
///
/// `BossArena(scenario="boss_b01", frame_skip=2, data_dir=None)` loads content (see the
/// module docs of `latticefall._latticefall`) and checks the scenario has a boss. Call
/// `reset(seed)` before stepping.
#[pyclass(module = "latticefall._latticefall")]
pub struct BossArena {
    world: World,
    content: Arc<Content>,
    scenario: String,
    frame_skip: u32,
    decoder: ActionDecoder,
    /// The tick the fight started on.
    fight_start: u64,
    /// The boss HP fraction after the last tick the player was alive (a death resets the
    /// boss in the same tick).
    boss_hp: f32,
    dead: bool,
    defeated: bool,
}

fn boss_hp_frac(w: &World) -> f32 {
    obs::boss_body(w, 0)
        .and_then(|id| w.actors.get(id))
        .map_or(0.0, |b| b.hp as f32 / b.max_hp.max(1) as f32)
}

fn player_hp_frac(w: &World) -> f32 {
    w.actors
        .get(w.player)
        .map_or(0.0, |p| p.hp as f32 / p.max_hp.max(1) as f32)
}

fn f32_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn run(w: &mut World, frame: ActionFrame) {
    tick(
        w,
        &TickInput {
            frame,
            commands: Vec::new(),
        },
    );
}

impl BossArena {
    fn walk_in(&mut self) -> PyResult<()> {
        let w = &mut self.world;
        while w.bosses[0].stage != Stage::Fight {
            if w.tick >= WALK_IN_LIMIT {
                return Err(value_err(format!(
                    "scenario {:?}: the fight didn't start within {WALK_IN_LIMIT} ticks of walking forward",
                    self.scenario
                )));
            }
            let move_z = if w.bosses[0].stage == Stage::Dormant {
                127
            } else {
                0
            };
            run(
                w,
                ActionFrame {
                    move_x: 0,
                    move_z,
                    buttons: 0,
                },
            );
            w.events.drain();
        }
        self.fight_start = w.tick;
        Ok(())
    }

    fn report<'py>(&self, py: Python<'py>, stance: u32, phases: u32) -> Report<'py> {
        let w = &self.world;
        let mask: Vec<u8> = obs::action_mask(w).iter().map(|&b| u8::from(b)).collect();
        let [own, theirs] = obs::move_ids(w);
        (
            PyBytes::new(py, &f32_bytes(&obs::proprio(w))),
            PyBytes::new(py, &f32_bytes(&obs::target(w))),
            (own, theirs),
            PyBytes::new(py, &mask),
            player_hp_frac(w),
            self.boss_hp,
            w.bosses[0].phase,
            w.tick - self.fight_start,
            self.dead,
            self.defeated,
            stance,
            phases,
        )
    }
}

#[pymethods]
impl BossArena {
    #[new]
    #[pyo3(signature = (scenario="boss_b01", frame_skip=2, data_dir=None))]
    fn new(scenario: &str, frame_skip: u32, data_dir: Option<&str>) -> PyResult<Self> {
        if frame_skip == 0 {
            return Err(value_err("frame_skip must be at least 1"));
        }
        let content = load_content(data_dir)?;
        let world = World::new(0, &content, scenario).map_err(value_err)?;
        if world.bosses.is_empty() {
            return Err(value_err(format!("scenario {scenario:?} has no boss")));
        }
        Ok(Self {
            world,
            content,
            scenario: scenario.to_owned(),
            frame_skip,
            decoder: ActionDecoder::default(),
            fight_start: 0,
            boss_hp: 1.0,
            dead: false,
            defeated: false,
        })
    }

    /// Starts a fresh fight from `seed`: builds the world and walks the player through
    /// the fog gate until the fight starts. Returns the same tuple as `step`.
    fn reset<'py>(&mut self, py: Python<'py>, seed: u64) -> PyResult<Report<'py>> {
        self.world = World::new(seed, &self.content, &self.scenario).map_err(value_err)?;
        self.decoder.reset();
        self.dead = false;
        self.defeated = false;
        py.detach(|| self.walk_in())?;
        self.boss_hp = boss_hp_frac(&self.world);
        Ok(self.report(py, 0, 0))
    }

    /// Runs one step: `frame_skip` ticks of the action `heads` (`move_dir`, `move_speed`,
    /// `action`, `lock`, `quick`), stopping early if the player dies or the boss falls.
    /// Returns `(proprio, target, (own_move, target_move), mask, player_hp_frac,
    /// boss_hp_frac, boss_phase, fight_ticks, player_died, boss_defeated, stance_breaks,
    /// phase_starts)`, where `proprio` and `target` are little-endian float32 bytes and
    /// `mask` is one byte per head value.
    fn step<'py>(&mut self, py: Python<'py>, heads: [u8; 5]) -> Report<'py> {
        let (stance, phases) = py.detach(|| {
            let mut stance = 0u32;
            let mut phases = 0u32;
            if self.dead || self.defeated {
                return (stance, phases);
            }
            let boss = obs::boss_body(&self.world, 0);
            let frames: Vec<ActionFrame> = (0..self.frame_skip)
                .map(|i| self.decoder.frame(&self.world, heads, i))
                .collect();
            for frame in frames {
                run(&mut self.world, frame);
                for e in self.world.events.drain() {
                    match e {
                        Event::StanceBreak(s) if Some(s.target) == boss => stance += 1,
                        Event::Boss(b) if b.boss == 0 && b.kind == BossEvent::PHASE_STARTED => {
                            phases += 1;
                        }
                        Event::Life(LifeEvent::Died { .. }) => self.dead = true,
                        _ => {}
                    }
                }
                let w = &self.world;
                self.dead |= w.actors.get(w.player).is_none_or(|p| p.dead || p.hp == 0);
                self.defeated = w.bosses[0].stage == Stage::Defeated;
                if self.dead || self.defeated {
                    if self.defeated {
                        self.boss_hp = 0.0;
                    }
                    break;
                }
                self.boss_hp = boss_hp_frac(w);
            }
            self.decoder.finish_step(heads);
            (stance, phases)
        });
        self.report(py, stance, phases)
    }

    /// The size of each action head.
    #[staticmethod]
    fn heads() -> [u8; 5] {
        obs::HEADS
    }

    /// The lengths of the `proprio` and `target` vectors.
    #[staticmethod]
    fn obs_lengths() -> (usize, usize) {
        (obs::PROPRIO_LEN, obs::TARGET_LEN)
    }

    /// The 64-bit hash of the current state.
    fn state_hash(&self) -> u64 {
        self.world.state_hash()
    }

    /// Ticks since the world was built (walk-in included).
    #[getter]
    fn tick(&self) -> u64 {
        self.world.tick
    }

    #[getter]
    fn frame_skip(&self) -> u32 {
        self.frame_skip
    }

    fn __repr__(&self) -> String {
        format!(
            "BossArena(scenario={:?}, tick={}, frame_skip={})",
            self.scenario, self.world.tick, self.frame_skip
        )
    }
}
