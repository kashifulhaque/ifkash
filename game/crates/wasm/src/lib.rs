//! `lf-wasm`: the wasm-bindgen facade over `lf-core` for the browser host.
//! See docs/game/01-architecture.md, "The browser host".
//!
//! The host owns the snapshot buffer, so snapshots never touch the state hash. JS reads
//! the buffer straight out of WASM memory: `snapshot_ptr()` is a byte offset into
//! `memory.buffer`, and `snapshot_len()` counts records of `SNAPSHOT_STRIDE` bytes.
//! Memory growth detaches `memory.buffer`, so re-create views after any call that can grow it.

use lf_content::Content;
use lf_core::events::Event;
use lf_core::snapshot::{ACTOR_SNAP_BYTES, ActorSnap, pack_id, write_snapshot};
use lf_core::{ActionFrame, Replay, TickInput, World, run_replay as core_run_replay, tick};
use wasm_bindgen::prelude::*;

#[cfg(feature = "dev")]
mod debug;

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
    let report = core_run_replay(&replay, &content).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(format!("{:016x}", report.final_hash))
}

/// Encodes a `.lfr` replay from a browser recording: the seed and scenario the world was
/// started with, and one 4-byte `ActionFrame` per tick run (`[move_x, move_z, buttons_lo,
/// buttons_hi]`; trailing bytes short of a frame are ignored). The checkpoint hashes come
/// from re-simulating the frames here, so the file matches what `lf-tools replay run` expects.
/// The browser records no commands yet, so the command list is empty.
#[wasm_bindgen]
pub fn encode_replay(seed: u64, scenario: &str, frames: &[u8]) -> Result<Vec<u8>, JsError> {
    let content = Content::load_embedded().map_err(|e| JsError::new(&e.to_string()))?;
    let (chunks, _) = frames.as_chunks::<4>();
    let frames: Vec<ActionFrame> = chunks.iter().map(|&b| ActionFrame::from_bytes(b)).collect();
    let replay = Replay::record(seed, scenario, &content, &frames, Vec::new())
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(replay.encode())
}

/// One simulation world plus its snapshot buffer.
#[wasm_bindgen]
pub struct LfWorld {
    world: World,
    snapshot: Vec<ActorSnap>,
    /// Dev-tool state (god mode, set level). Present only in `dev` builds.
    #[cfg(feature = "dev")]
    dev: debug::DevState,
}

#[wasm_bindgen]
impl LfWorld {
    /// Starts a fresh world from the embedded content. `scenario` names a file in
    /// `game/data/scenarios/` without the extension, for example `m0_flat` or
    /// `test_field`. Throws if the scenario doesn't exist.
    #[wasm_bindgen(constructor)]
    pub fn new_world(seed: u64, scenario: &str) -> Result<LfWorld, JsError> {
        let content = Content::load_embedded().map_err(|e| JsError::new(&e.to_string()))?;
        let world =
            World::new(seed, &content, scenario).map_err(|e| JsError::new(&e.to_string()))?;
        let mut snapshot = Vec::new();
        write_snapshot(&world, &mut snapshot);
        Ok(LfWorld {
            world,
            snapshot,
            #[cfg(feature = "dev")]
            dev: debug::DevState::default(),
        })
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
            #[cfg(feature = "dev")]
            self.dev_after_tick();
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

    /// Drains queued events as a `Uint32Array`: `[kind, payload...]` per event, with
    /// `f32` values as their bit patterns.
    ///
    /// - Kind 1, `Hit`, 17 words: kind, attacker and target (`pack_id`), move number,
    ///   hitbox index, motion value, charge fraction, contact point (3), direction (3),
    ///   then (WP-023) damage taken, result (0 connected, 1 guarded, 2 guard broken),
    ///   flags (bit 0 counter hit, bit 1 killed the target), and the target's HP after.
    pub fn drain_events(&mut self) -> Vec<u32> {
        let mut out = Vec::new();
        for event in self.world.events.drain() {
            match event {
                Event::Hit(h) => {
                    out.extend([
                        1,
                        pack_id(h.attacker),
                        pack_id(h.target),
                        h.move_num,
                        u32::from(h.hitbox),
                        h.motion_value.to_bits(),
                        h.charge_frac.to_bits(),
                    ]);
                    out.extend(h.point.map(f32::to_bits));
                    out.extend(h.dir.map(f32::to_bits));
                    out.extend([
                        h.damage,
                        h.result.code(),
                        u32::from(h.counter) | u32::from(h.killed) << 1,
                        h.hp,
                    ]);
                }
            }
        }
        out
    }

    /// Ray query against the level. `origin` and `dir` are 3 floats each. Returns the
    /// distance to the first hit within `max`, in units of `dir`'s length, or `undefined`
    /// on a miss or if either array isn't 3 long. Read-only: it never changes the world.
    pub fn raycast(&self, origin: &[f32], dir: &[f32], max: f32) -> Option<f32> {
        let origin: [f32; 3] = origin.try_into().ok()?;
        let dir: [f32; 3] = dir.try_into().ok()?;
        self.world.physics.raycast(origin, dir, max)
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
