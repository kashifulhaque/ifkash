//! `lf-wasm`: the wasm-bindgen facade over `lf-core` for the browser host.
//! See docs/game/01-architecture.md, "The browser host".
//!
//! The host owns the snapshot buffer, so snapshots never touch the state hash. JS reads
//! the buffer straight out of WASM memory: `snapshot_ptr()` is a byte offset into
//! `memory.buffer`, and `snapshot_len()` counts records of `SNAPSHOT_STRIDE` bytes.
//! Memory growth detaches `memory.buffer`, so re-create views after any call that can grow it.

use lf_content::Content;
use lf_core::events::{Event, LifeEvent};
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
    ///   A hit from a projectile has hitbox index 255.
    /// - Kind 2, `Life`, 6 words: kind, code, value, and a position (3). Codes: 1 died,
    ///   2 respawned, 3 rested (value is the Lantern's index in the scenario), 4 Remnant
    ///   dropped, 5 Remnant recovered, 6 Remnant lost (value is its Glimmer for 4 to 6,
    ///   and its position is the Remnant's). Otherwise the position is the player's feet.
    /// - Kind 3, `Stagger` (WP-024), 4 words: kind, target, heavy (0 light, 1 heavy), ticks.
    /// - Kind 4, `StanceBreak` (WP-024), 3 words: kind, target, kneel ticks.
    /// - Kind 5, `Parry` (WP-024), 4 words: kind, attacker, target, the parried move number.
    /// - Kind 6, `Critical` (WP-024), 5 words: kind, attacker, target, crit kind (0 riposte,
    ///   1 backstab), the attacker's crit move number. The damage follows as a `Hit`.
    /// - Kind 7, `StatusProc` (WP-025), 4 words: kind, target, status (0 Poison, 1 Blight,
    ///   2 Drowse, 3 Delirium, 4 Rupture, 5 Rime: the bit of that status in the snapshot's
    ///   `status_mask`), HP the burst took (0 for the statuses that have none).
    /// - Kind 8, `Projectile` (WP-027), 10 words: kind, what happened (0 spawned, 1 struck
    ///   an actor, 2 hit the level, 3 expired), projectile handle and definition number
    ///   (`proj_arrow` is 5000), shooter and target (`pack_id`; target is `0xFFFFFFFF`
    ///   unless it struck an actor), position (3), and 1 if the projectile is gone after
    ///   this event.
    /// - Kind 9, `Awareness` (WP-030), 4 words: kind, the AI actor (`pack_id`), and the
    ///   awareness state it left and entered (0 Idle, 1 Suspicious, 2 Aware, 3 Combat,
    ///   4 Return).
    /// - Kind 10 and 12 are reserved for other work packages.
    /// - Kind 11, `Boss` (WP-033), 4 words: kind, the boss's index in the scenario, what
    ///   happened (0 the gate sealed, 1 the fight started, 2 a phase started, 3 defeated,
    ///   4 reset), and the phase (the new one for 2, 1 for 1, otherwise 0).
    /// - Kind 13, `Revive` (WP-032), 3 words: kind, the enemy that got back up
    ///   (`pack_id`), and the HP it has.
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
                Event::Life(e) => {
                    let (code, value, pos) = match e {
                        LifeEvent::Died { pos } => (1, 0, pos),
                        LifeEvent::Respawned { pos } => (2, 0, pos),
                        LifeEvent::Rested { lantern, pos } => (3, u32::from(lantern), pos),
                        LifeEvent::RemnantDropped { glimmer, pos } => (4, glimmer, pos),
                        LifeEvent::RemnantRecovered { glimmer, pos } => (5, glimmer, pos),
                        LifeEvent::RemnantLost { glimmer, pos } => (6, glimmer, pos),
                    };
                    out.extend([2, code, value]);
                    out.extend(pos.map(f32::to_bits));
                }
                Event::Stagger(s) => {
                    out.extend([3, pack_id(s.target), u32::from(s.heavy), u32::from(s.ticks)]);
                }
                Event::StanceBreak(s) => {
                    out.extend([4, pack_id(s.target), u32::from(s.ticks)]);
                }
                Event::Parry(p) => {
                    out.extend([5, pack_id(p.attacker), pack_id(p.target), p.move_num]);
                }
                Event::Critical(c) => {
                    out.extend([
                        6,
                        pack_id(c.attacker),
                        pack_id(c.target),
                        c.kind.code(),
                        c.move_num,
                    ]);
                }
                Event::StatusProc(p) => {
                    out.extend([7, pack_id(p.target), p.code(), p.damage]);
                }
                Event::Projectile(e) => {
                    out.extend([
                        8,
                        e.kind.code(),
                        pack_id(e.projectile),
                        e.def,
                        pack_id(e.shooter),
                        e.target.map_or(u32::MAX, pack_id),
                    ]);
                    out.extend(e.pos.map(f32::to_bits));
                    out.push(u32::from(e.ended));
                }
                Event::Awareness(e) => {
                    out.extend([9, pack_id(e.actor), u32::from(e.from), u32::from(e.to)]);
                }
                Event::Revive(e) => {
                    out.extend([13, pack_id(e.actor), e.hp]);
                }
                Event::Boss(e) => {
                    out.extend([11, u32::from(e.boss), u32::from(e.kind), u32::from(e.phase)]);
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
