//! `lf-wasm`: the wasm-bindgen facade over `lf-core` for the browser host.
//! See docs/game/01-architecture.md, "The browser host".
//!
//! The host owns the snapshot buffer, so snapshots never touch the state hash. JS reads
//! the buffer straight out of WASM memory: `snapshot_ptr()` is a byte offset into
//! `memory.buffer`, and `snapshot_len()` counts records of `SNAPSHOT_STRIDE` bytes.
//! Memory growth detaches `memory.buffer`, so re-create views after any call that can grow it.

use lf_content::Content;
use lf_core::snapshot::{ACTOR_SNAP_BYTES, ActorSnap, write_snapshot};
use lf_core::{ActionFrame, Replay, TickInput, World, run_replay as core_run_replay, tick};
use wasm_bindgen::prelude::*;

/// Bytes between consecutive snapshot records (`size_of::<ActorSnap>()`).
#[wasm_bindgen]
pub fn snapshot_stride() -> u32 {
    ACTOR_SNAP_BYTES as u32
}

/// The content hash as 16 lowercase hex digits, matching `lf-tools validate`.
#[wasm_bindgen]
pub fn content_hash() -> Result<String, JsError> {
    let content = Content::load_embedded().map_err(|e| JsError::new(&e.to_string()))?;
    Ok(format!("{:016x}", content.content_hash()))
}

/// Runs a `.lfr` replay from bytes and returns its final state hash as 16 hex digits.
/// Throws with the mismatch message if any recorded checkpoint differs.
#[wasm_bindgen]
pub fn run_replay(bytes: &[u8]) -> Result<String, JsError> {
    let replay = Replay::decode(bytes).map_err(|e| JsError::new(&e.to_string()))?;
    let content = Content::load_embedded().map_err(|e| JsError::new(&e.to_string()))?;
    let report = core_run_replay(&replay, content.content_hash())
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(format!("{:016x}", report.final_hash))
}

/// One simulation world plus its snapshot buffer.
#[wasm_bindgen]
pub struct LfWorld {
    world: World,
    snapshot: Vec<ActorSnap>,
}

#[wasm_bindgen]
impl LfWorld {
    /// Starts a fresh world. `scenario` is reserved: only the flat M0 scenario exists so far.
    #[wasm_bindgen(constructor)]
    pub fn new_world(seed: u64, scenario: &str) -> LfWorld {
        let _ = scenario;
        let world = World::new(seed);
        let mut snapshot = Vec::new();
        write_snapshot(&world, &mut snapshot);
        LfWorld { world, snapshot }
    }

    /// Advances one tick per 4-byte `ActionFrame` in `frames` (`[move_x, move_z, buttons_lo,
    /// buttons_hi]`), then rewrites the snapshot. Trailing bytes short of a full frame are
    /// ignored. Returns the number of ticks run.
    pub fn tick(&mut self, frames: &[u8]) -> u32 {
        let mut ran = 0;
        let (chunks, _) = frames.as_chunks::<4>();
        for &bytes in chunks {
            let frame = ActionFrame::from_bytes(bytes);
            tick(
                &mut self.world,
                &TickInput {
                    frame,
                    commands: Vec::new(),
                },
            );
            ran += 1;
        }
        write_snapshot(&self.world, &mut self.snapshot);
        ran
    }

    /// Byte offset of the first `ActorSnap` in WASM memory. Valid until the next `tick`.
    pub fn snapshot_ptr(&self) -> u32 {
        self.snapshot.as_ptr() as u32
    }

    /// Number of `ActorSnap` records (not bytes); the stride is `snapshot_stride()`.
    pub fn snapshot_len(&self) -> u32 {
        self.snapshot.len() as u32
    }

    /// Drains queued events as a `Uint32Array`. The encoding will be `[kind, payload...]`
    /// per event once the first `Event` variants land; the queue is always empty today.
    pub fn drain_events(&mut self) -> Vec<u32> {
        // `Event` has no variants yet, so this maps nothing; the match must grow with it.
        self.world
            .events
            .drain()
            .into_iter()
            .map(|event| match event {})
            .collect()
    }

    /// Ray query against the world. Stub until WP-010 adds physics: always `undefined`.
    /// On a hit it will return the distance along the ray.
    pub fn raycast(&self, origin: &[f32], dir: &[f32], max: f32) -> Option<f32> {
        let _ = (origin, dir, max);
        None
    }

    /// The world state hash as 16 lowercase hex digits.
    pub fn state_hash(&self) -> String {
        format!("{:016x}", self.world.state_hash())
    }

    /// Ticks simulated so far.
    pub fn tick_count(&self) -> f64 {
        self.world.tick as f64
    }
}
