//! `lf-py`: the PyO3 module over `lf-core`.
//!
//! This is the stub from WP-007: one `Sim` plus replay helpers. `VecSim`, observations,
//! and rewards come with the RL work packages. Python imports this module as
//! `latticefall._latticefall`; the `latticefall` package re-exports the public names.
//!
//! # Where content comes from
//!
//! Unlike the WASM build, the Python build reads content from disk so researchers can
//! edit it without recompiling. [`load_content`] resolves the data directory in this order:
//!
//! 1. The `data_dir` argument, if given. A failure to load it is an error.
//! 2. The `LATTICEFALL_DATA` environment variable, if set. A failure is an error.
//! 3. `game/data` in the source tree this module was built from, if that folder exists.
//! 4. The content embedded in `lf-content` at build time.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use lf_content::Content;
use lf_core::input::{ActionFrame, TickInput};
use lf_core::replay::{self, Replay};
use lf_core::tick::tick;
use lf_core::world::World;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::pybacked::PyBackedBytes;
use pyo3::types::PyBytes;

/// The scenario a new `Sim` starts in, and the default for `reset`.
const DEFAULT_SCENARIO: &str = "m0_flat";
/// Environment variable that names a content directory.
const DATA_ENV: &str = "LATTICEFALL_DATA";

fn value_err(e: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(e.to_string())
}

fn load_from(dir: &Path) -> PyResult<Content> {
    Content::load_dir(dir)
        .map_err(|e| value_err(format!("cannot load content from {}: {e}", dir.display())))
}

/// Loads content following the resolution order in the module docs.
fn load_content(data_dir: Option<&str>) -> PyResult<Arc<Content>> {
    if let Some(dir) = data_dir {
        return load_from(Path::new(dir)).map(Arc::new);
    }
    if let Some(dir) = std::env::var_os(DATA_ENV) {
        return load_from(&PathBuf::from(dir)).map(Arc::new);
    }
    let source_tree = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    if source_tree.join("registry.ron").is_file() {
        return load_from(&source_tree).map(Arc::new);
    }
    embedded().map(Arc::new)
}

fn embedded() -> PyResult<Content> {
    Content::load_embedded().map_err(|e| value_err(format!("cannot load embedded content: {e}")))
}

fn step_world(world: &mut World, frame: ActionFrame) {
    let input = TickInput {
        frame,
        commands: Vec::new(),
    };
    tick(world, &input);
}

/// One simulation world.
///
/// `Sim(data_dir=None)` loads content (see the module docs for the lookup order) and
/// starts a world with seed 0. Call `reset` to choose the seed.
#[pyclass(module = "latticefall._latticefall")]
struct Sim {
    world: World,
    content: Arc<Content>,
    content_hash: u64,
}

#[pymethods]
impl Sim {
    #[new]
    #[pyo3(signature = (data_dir=None))]
    fn new(data_dir: Option<&str>) -> PyResult<Self> {
        let content = load_content(data_dir)?;
        let content_hash = content.content_hash();
        Ok(Self {
            world: World::new(0, &content, DEFAULT_SCENARIO).map_err(value_err)?,
            content,
            content_hash,
        })
    }

    /// Starts a fresh world from `seed` in `scenario` (a file in the content's
    /// `scenarios/` folder, without the extension). Raises `ValueError` if it doesn't exist.
    #[pyo3(signature = (seed, scenario=DEFAULT_SCENARIO))]
    fn reset(&mut self, seed: u64, scenario: &str) -> PyResult<()> {
        self.world = World::new(seed, &self.content, scenario).map_err(value_err)?;
        Ok(())
    }

    /// Advances one tick with one action frame: exactly 4 bytes
    /// (`move_x: i8`, `move_z: i8`, `buttons: u16` little endian).
    fn step(&mut self, py: Python<'_>, action_frame: PyBackedBytes) -> PyResult<()> {
        if action_frame.len() != 4 {
            return Err(value_err(format!(
                "action frame must be exactly 4 bytes, got {}",
                action_frame.len()
            )));
        }
        let frame = ActionFrame::from_bytes([
            action_frame[0],
            action_frame[1],
            action_frame[2],
            action_frame[3],
        ]);
        let world = &mut self.world;
        py.detach(|| step_world(world, frame));
        Ok(())
    }

    /// Advances one tick per 4-byte frame in `frames`, releasing the GIL for the whole
    /// batch. Returns the tick number afterwards.
    fn step_many(&mut self, py: Python<'_>, frames: PyBackedBytes) -> PyResult<u64> {
        if !frames.len().is_multiple_of(4) {
            return Err(value_err(format!(
                "frames length must be a multiple of 4, got {}",
                frames.len()
            )));
        }
        let world = &mut self.world;
        py.detach(|| {
            let (chunks, _) = frames.as_chunks::<4>();
            for &chunk in chunks {
                step_world(world, ActionFrame::from_bytes(chunk));
            }
        });
        Ok(self.world.tick)
    }

    /// Returns an independent copy of this world.
    fn clone(&self) -> Self {
        Self {
            world: self.world.clone(),
            content: Arc::clone(&self.content),
            content_hash: self.content_hash,
        }
    }

    /// The 64-bit hash of the current state.
    fn state_hash(&self) -> u64 {
        self.world.state_hash()
    }

    /// Ticks run since the last reset.
    #[getter]
    fn tick(&self) -> u64 {
        self.world.tick
    }

    /// Hash of the content this Sim loaded.
    #[getter]
    fn content_hash(&self) -> u64 {
        self.content_hash
    }

    fn __repr__(&self) -> String {
        format!(
            "Sim(tick={}, state_hash={:#018x})",
            self.world.tick,
            self.state_hash()
        )
    }
}

/// Hash of the content found by the default lookup (no `data_dir`).
#[pyfunction]
fn content_hash() -> PyResult<u64> {
    Ok(load_content(None)?.content_hash())
}

/// Hash of the content embedded in the extension at build time.
#[pyfunction]
fn embedded_content_hash() -> PyResult<u64> {
    Ok(embedded()?.content_hash())
}

fn decode(data: &[u8]) -> PyResult<Replay> {
    Replay::decode(data).map_err(value_err)
}

/// Runs a `.lfr` replay and returns the final state hash. Raises `ValueError` on a decode
/// failure, a content hash mismatch, or a state hash mismatch (the message names the tick).
#[pyfunction]
fn run_replay(py: Python<'_>, data: PyBackedBytes) -> PyResult<u64> {
    let r = decode(&data)?;
    let content = load_content(None)?;
    py.detach(|| replay::run_replay(&r, &content))
        .map(|report| report.final_hash)
        .map_err(value_err)
}

/// The final expected state hash recorded in a replay.
#[pyfunction]
fn replay_final_expected(data: PyBackedBytes) -> PyResult<u64> {
    decode(&data)?
        .hashes
        .last()
        .map(|&(_, hash)| hash)
        .ok_or_else(|| value_err("replay has no recorded hashes"))
}

/// The replay's frames, expanded to 4 bytes per tick.
#[pyfunction]
fn replay_frames<'py>(py: Python<'py>, data: PyBackedBytes) -> PyResult<Bound<'py, PyBytes>> {
    let frames = replay::expand(&decode(&data)?.frames);
    let bytes: Vec<u8> = frames.iter().flat_map(|f| f.to_bytes()).collect();
    Ok(PyBytes::new(py, &bytes))
}

/// The seed recorded in a replay.
#[pyfunction]
fn replay_seed(data: PyBackedBytes) -> PyResult<u64> {
    Ok(decode(&data)?.header.seed)
}

/// The scenario name recorded in a replay.
#[pyfunction]
fn replay_scenario(data: PyBackedBytes) -> PyResult<String> {
    Ok(decode(&data)?.header.scenario)
}

#[pymodule]
mod _latticefall {
    #[pymodule_export]
    use super::{
        Sim, content_hash, embedded_content_hash, replay_final_expected, replay_frames,
        replay_scenario, replay_seed, run_replay,
    };
}
