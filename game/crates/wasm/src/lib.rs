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
use lf_core::world_state::map;
use lf_core::{ActionFrame, Command, Replay, TickInput, World, run_replay as core_run_replay};
use wasm_bindgen::prelude::*;

#[cfg(feature = "dev")]
mod debug;
mod menu;

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
    /// The seed the world started from, kept for `save`.
    seed: u64,
    snapshot: Vec<ActorSnap>,
    /// Display names by registry number (weapons and consumables), for the HUD.
    names: std::collections::HashMap<u32, String>,
    /// Commands from `queue_command`, applied by the first tick of the next `tick` call.
    pending: Vec<Command>,
    /// The content the world was built from, kept to build the destination of a trip to
    /// another area (WP-069), and every Lantern of the realm for the map.
    content: Content,
    places: Vec<map::Place>,
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
        let names = display_names(&content);
        let places = map::places(&content);
        Ok(LfWorld {
            world,
            seed,
            snapshot,
            names,
            pending: Vec::new(),
            content,
            places,
            #[cfg(feature = "dev")]
            dev: debug::DevState::default(),
        })
    }

    /// Queues a menu-level `Command` for the next tick (the first one of the next `tick`
    /// call), by `kind`: 0 Rest, 1 LevelUp (`a` is the attribute, 0 to 6), 2 Equip (`a` is
    /// the slot, `b` the item), 3 Unequip (`a` the slot), 4 UseItem (`a` the quick slot),
    /// 5 Gesture (`a`), 6 SplitFlasks (`a` is the Vigil Flask's charges), 7 Travel (`a` is
    /// the Lantern's index), and the dialogue commands (WP-068): 40 Ask (`a` is a topic of the
    /// open NPC's menu, 254 for its quest, 255 to go back), 41 EndTalk, 42 Appearance (`a`
    /// is the Mirror's preset, `b` its dye), and the map commands (WP-069): 60 TravelFar (`a` is
    /// a place of `map_json`'s `places`), 61 MapMarker (`a` and `b` are the point in decimeters,
    /// each plus 32768). Returns false, queuing nothing, for an unknown kind or an
    /// operand that doesn't fit its type. The core decides whether the command does
    /// anything; the host records `(tick_count(), kind, a, b)` for replays.
    pub fn queue_command(&mut self, kind: u32, a: u32, b: u32) -> bool {
        let Some(c) = menu::command_from_words(kind, a, b) else {
            return false;
        };
        self.pending.push(c);
        true
    }

    /// Advances one tick per 4-byte `ActionFrame` in `frames` (`[move_x, move_z, buttons_lo,
    /// buttons_hi]`), then rewrites the snapshot. Trailing bytes short of a full frame are
    /// ignored. Returns the number of ticks run.
    pub fn tick(&mut self, frames: &[u8]) -> u32 {
        let mut ran = 0;
        let (chunks, _) = frames.as_chunks::<4>();
        for &bytes in chunks {
            let frame = ActionFrame::from_bytes(bytes);
            // A trip to another area replaces the world after the tick (WP-069).
            map::tick_world(
                &mut self.world,
                &TickInput {
                    frame,
                    commands: std::mem::take(&mut self.pending),
                },
                &self.content,
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
    /// - Kind 14, `Mechanic` (WP-042), 3 words: kind, what happened (0 door opened, 1 door
    ///   shut, 2 locked door tried, 3 one-way door tried from the wrong side, 4 keyed door
    ///   tried without its key, 5 lever pulled, 6 lift set off, 7 lift arrived, 8 ladder
    ///   grabbed, 9 ladder released, 10 ladder kicked down, 11 hollow wall revealed; WP-062:
    ///   40 mine cart set off, 41 cart arrived, 42 pushed cart broke its wall, 43 trapdoor
    ///   pushed open; WP-063: 70 mirror turned, 71 receptor lit by a beam, 72 a swing struck a
    ///   low ceiling; WP-066: 100 brazier lit, 101 sigil touched, 102 sigil sealed its doors,
    ///   103 sigil's doors opened again; WP-072: 130 egg sac destroyed, 131 weak point exposed,
    ///   132 egg sac spawned a Crawler; WP-074: 150 the Drowned Choir sang a new verse, 151
    ///   the Cantor's hymn healed a member, 152 the Lamenter reappeared behind the player;
    ///   WP-076: 170 a Colossus leg's stance broke, 171 the Colossus's lava began to creep,
    ///   172 the lava burned the player), and the index of the door, lever, lift, ladder, wall,
    ///   cart, trapdoor, mirror, receptor, low-ceiling zone, brazier, or sigil in its scenario
    ///   list (for 130 and 132, the egg sac's index among the boss's four parts; for 150-152,
    ///   the Choir member's number: 0 Cantor, 1 Bellbearer, 2 Lamenter; for 170, the leg: 0
    ///   left, 1 right; for 171 and 172, the boss).
    /// - Kind 15, `Loot` (WP-042), 6 words: kind, source (1 chest), the chest's index, the
    ///   item's registry number (0 for none), how many, and Glimmer.
    /// - Kind 16, `Message` (WP-042), 2 words: kind, the message's index (see
    ///   `message_text`).
    /// - Kind 17, `Pickup` (WP-049), 3 words: kind, the pickup's index in the scenario's
    ///   pickup list, and the item's registry number.
    /// - Kind 21, `Echo` (WP-059), 5 words: kind, what happened (0 summoned, 1 a body fell,
    ///   2 dismissed, 3 bowed to, 4 the Bell was rung and nothing came), the Echo's registry
    ///   number, the body it concerns (`pack_id`; `0xFFFFFFFF` for none), and for 4 why (1 no
    ///   Echo Stone in reach, 2 no Echo owned, 3 not enough FP, 4 an Echo is already out).
    /// - Kind 31, `Dialogue` (WP-068), 5 words: kind, what happened (0 opened, 1 closed, 2 the
    ///   topic changed), what is open (0 an NPC, 1 a station), its index in the scenario's
    ///   NPCs or stations, and the open topic (0 the menu, a topic's index plus one, 255 the
    ///   quest line).
    /// - Kind 32, `Quest` (WP-068), 3 words: kind, the giver's index in the scenario's NPCs,
    ///   the new state (1 taken up, 2 done).
    /// - Kind 33, `Appearance` (WP-068), 3 words: kind, the Mirror's preset, and its dye.
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
                Event::Pickup(e) => {
                    out.extend([17, u32::from(e.index), e.item]);
                }
                Event::Mechanic(e) => {
                    out.extend([14, u32::from(e.kind), u32::from(e.index)]);
                }
                Event::Loot(e) => {
                    out.extend([
                        15,
                        u32::from(e.source),
                        u32::from(e.index),
                        e.item,
                        u32::from(e.count),
                        e.glimmer,
                    ]);
                }
                Event::Message(e) => {
                    out.extend([16, u32::from(e.index)]);
                }
                Event::Boss(e) => {
                    out.extend([11, u32::from(e.boss), u32::from(e.kind), u32::from(e.phase)]);
                }
                Event::Dialogue(e) => {
                    out.extend([
                        31,
                        u32::from(e.what),
                        u32::from(e.target),
                        u32::from(e.index),
                        u32::from(e.topic),
                    ]);
                }
                Event::Quest(e) => {
                    out.extend([32, u32::from(e.npc), u32::from(e.state)]);
                }
                Event::Appearance(e) => {
                    out.extend([33, u32::from(e.preset), u32::from(e.dye)]);
                }
                Event::Echo(e) => {
                    out.extend([
                        21,
                        u32::from(e.kind),
                        e.echo,
                        e.actor.map_or(u32::MAX, pack_id),
                        u32::from(e.reason),
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

    /// Serializes the persistent state (see `lf_core::save`) as a versioned blob for the
    /// IndexedDB save slots.
    pub fn save(&self) -> Result<Vec<u8>, JsError> {
        let content = Content::load_embedded().map_err(|e| JsError::new(&e.to_string()))?;
        Ok(lf_core::save::save(&self.world, self.seed, &content))
    }

    /// Builds a world from a blob made by `save`: a fresh world for the saved scenario and
    /// seed with the persistent state applied. Throws if the blob isn't a save, is from an
    /// unsupported version, or names a scenario the content doesn't have.
    #[wasm_bindgen(js_name = fromSave)]
    pub fn from_save(bytes: &[u8]) -> Result<LfWorld, JsError> {
        let content = Content::load_embedded().map_err(|e| JsError::new(&e.to_string()))?;
        let data = lf_core::save::decode(bytes).map_err(|e| JsError::new(&e.to_string()))?;
        let world =
            lf_core::save::restore(&data, &content).map_err(|e| JsError::new(&e.to_string()))?;
        let mut snapshot = Vec::new();
        write_snapshot(&world, &mut snapshot);
        Ok(LfWorld {
            world,
            seed: data.seed,
            snapshot,
            names: display_names(&content),
            pending: Vec::new(),
            places: map::places(&content),
            content,
            #[cfg(feature = "dev")]
            dev: debug::DevState::default(),
        })
    }

    /// Hash of the persistent state only, as 16 lowercase hex digits. Equal before a save
    /// and after loading it. Unrelated to `state_hash`.
    pub fn persistent_hash(&self) -> Result<String, JsError> {
        let content = Content::load_embedded().map_err(|e| JsError::new(&e.to_string()))?;
        Ok(format!(
            "{:016x}",
            lf_core::save::persistent_hash(&self.world, self.seed, &content)
        ))
    }

    /// The player's HUD values as `hud_words()` floats (WP-045). Read-only. Layout:
    ///
    /// - 0 to 5: HP, max HP, FP, max FP, stamina, max stamina.
    /// - 6: Glimmer held.
    /// - 7 to 10: Vigil charges, Vigil max, Lucid charges, Lucid max.
    /// - 11 to 14: registry numbers of the selected right-hand weapon, left-hand weapon,
    ///   spell (always 0 until spells exist), and quick item (0 for none).
    /// - 15: count of the quick item (flask charges, or the stack).
    /// - 16: 1 if any enemy is in Combat awareness of the player.
    /// - 17: interaction prompt: 0 none, 1 a Lantern is in reach.
    /// - 18: the boss on the bar plus one (0 for none): a boss whose fight is running.
    /// - 19, 20: that boss's HP fraction (body and required parts) and phase.
    /// - 21 to 26: the player's buildup meters, 0 to 1, in the order of the `StatusProc`
    ///   event's status codes (Poison, Blight, Drowse, Delirium, Rupture, Rime).
    /// - 27: 1 if the player is dead.
    /// - 28: 1 if the quick item is a flask (so word 15 is charges, not a stack).
    pub fn hud_state(&self) -> Vec<f32> {
        use lf_core::ai::{Awareness, boss::Stage};
        use lf_core::combat::status::KINDS;
        let w = &self.world;
        let mut out = vec![0.0f32; HUD_WORDS];
        let Some(p) = w.actors.get(w.player) else {
            return out;
        };
        let inv = &p.inventory;
        let eq = &inv.equipment;
        out[..7].copy_from_slice(&[
            p.hp as f32,
            p.max_hp as f32,
            p.fp,
            p.max_fp,
            p.stamina,
            p.max_stamina,
            inv.glimmer as f32,
        ]);
        let f = &inv.flasks;
        out[7..11].copy_from_slice(&[
            f32::from(f.vigil),
            f32::from(f.vigil_max),
            f32::from(f.lucid),
            f32::from(f.lucid_max()),
        ]);
        let quick = eq.selected_quick();
        out[11] = eq.right_weapon().unwrap_or(0) as f32;
        out[12] = eq.left_weapon().unwrap_or(0) as f32;
        if let Some(q) = quick {
            out[14] = q as f32;
            out[15] = match w.items.flask.as_ref() {
                Some(r) if q == r.vigil => f32::from(f.vigil),
                Some(r) if q == r.lucid => f32::from(f.lucid),
                _ => f32::from(inv.count(q)),
            };
            out[28] = f32::from(u8::from(
                w.items
                    .flask
                    .as_ref()
                    .is_some_and(|r| q == r.vigil || q == r.lucid),
            ));
        }
        let in_combat = w.actors.iter().any(|(_, a)| {
            a.ai.as_ref()
                .is_some_and(|b| b.awareness == Awareness::Combat)
        });
        out[16] = f32::from(u8::from(in_combat && !p.dead));
        out[17] = f32::from(u8::from(
            lf_core::world_state::lantern::in_reach(w).is_some(),
        ));
        if let Some(b) = w
            .bosses
            .iter()
            .position(|r| matches!(r.stage, Stage::Intro { .. } | Stage::Fight))
        {
            let layout = &w.layout.bosses[b];
            let (mut hp, mut max) = (0u64, 0u64);
            for (i, spawn) in layout.members.iter().enumerate() {
                let required = i == 0 || layout.def.parts.get(i - 1).is_some_and(|q| q.required);
                let actor = w
                    .state
                    .ai_enemies
                    .get(usize::from(*spawn))
                    .and_then(|id| w.actors.get(*id));
                if let (true, Some(a)) = (required, actor) {
                    max += u64::from(a.max_hp);
                    if !(a.dead || a.hp == 0) {
                        hp += u64::from(a.hp);
                    }
                }
            }
            out[18] = (b + 1) as f32;
            out[19] = if max == 0 {
                0.0
            } else {
                hp as f32 / max as f32
            };
            out[20] = f32::from(w.bosses[b].phase);
        }
        for (i, k) in KINDS.iter().enumerate() {
            let threshold = p.status.threshold_milli(*k, &w.status.rules);
            if threshold > 0 {
                out[21 + i] = (p.status.meter_milli[i] as f32 / threshold as f32).clamp(0.0, 1.0);
            }
        }
        out[27] = f32::from(u8::from(p.dead));
        out
    }

    /// Display name of a weapon or consumable by registry number; empty if unknown.
    pub fn item_name(&self, num: u32) -> String {
        self.names.get(&num).cloned().unwrap_or_default()
    }

    /// The renderer's view of the world mechanics, as floats (WP-042). Read-only. See
    /// `lf_core::world_state::mechanics::render_state` for the layout: the ladder the
    /// player is on plus one, then per scenario list the ladders' climbable flags, the
    /// lifts' rise in meters, and the open, pulled, revealed, taken, opened, and read
    /// flags, then the pools' depths.
    pub fn mechanics_state(&self) -> Vec<f32> {
        lf_core::world_state::mechanics::render_state(&self.world)
    }

    /// What `INTERACT` would do now, for the prompt: 0 nothing, 1 rest at a Lantern,
    /// 2 take or open, 3 pull a lever, 4 open a door, 5 read a message, 6 climb a ladder,
    /// 7 kick a ladder down (WP-042). WP-062: 30 ride a mine cart, 31 push one, 32 push a
    /// trapdoor open. WP-063: 40 turn a crystal mirror. WP-066: 55 light a Warden Brazier, 56 touch a
    /// seal sigil.
    pub fn interact_prompt(&self) -> u32 {
        if lf_core::world_state::lantern::in_reach(&self.world).is_some() {
            1
        } else {
            match lf_core::world_state::mechanics::prompt(&self.world) {
                // 8 speak with someone, 9 use a station (WP-068).
                0 => lf_core::world_state::quests::prompt(&self.world),
                code => code,
            }
        }
    }

    /// The dialogue as floats (WP-068): see `lf_core::world_state::quests::render_state` for
    /// the layout. Read-only.
    pub fn dialogue_state(&self) -> Vec<f32> {
        lf_core::world_state::quests::render_state(&self.world)
    }

    /// The definition of the NPC at `index` of the scenario's NPCs as JSON (its name, role,
    /// greetings, topics, and quest); empty if none (WP-068). Facts are named by section and
    /// slug, never written out.
    pub fn npc_json(&self, index: u32) -> String {
        self.world
            .layout
            .npcs
            .get(index as usize)
            .map(|n| n.def.to_json())
            .unwrap_or_default()
    }

    /// The definition of the station at `index` of the scenario's stations as JSON; empty if
    /// none (WP-068).
    pub fn station_json(&self, index: u32) -> String {
        self.world
            .layout
            .stations
            .get(index as usize)
            .map(|s| s.def.to_json())
            .unwrap_or_default()
    }

    /// The text of the authored message at `index` of the scenario's messages; empty if
    /// none (WP-042).
    pub fn message_text(&self, index: u32) -> String {
        self.world
            .layout
            .messages
            .get(index as usize)
            .map(|m| m.text.clone())
            .unwrap_or_default()
    }

    /// Name on the boss bar for the boss at `index` in the scenario; empty if none.
    pub fn boss_name(&self, index: u32) -> String {
        self.world
            .layout
            .bosses
            .get(index as usize)
            .map(|b| b.def.name.clone())
            .unwrap_or_default()
    }
}

/// Floats in `LfWorld::hud_state`.
const HUD_WORDS: usize = 29;

/// Length of `LfWorld::hud_state`, so the host can check it matches its reader.
#[wasm_bindgen]
pub fn hud_words() -> u32 {
    HUD_WORDS as u32
}

/// Display names by registry number (weapons and consumables), for the HUD (WP-045).
fn display_names(content: &Content) -> std::collections::HashMap<u32, String> {
    let mut names = std::collections::HashMap::new();
    for (_, w) in content.all_weapons() {
        if let Some(n) = content.registry.num(&w.id) {
            names.insert(n, w.name.clone());
        }
    }
    if let Some(c) = &content.consumables {
        for i in &c.items {
            if let Some(n) = content.registry.num(&i.id) {
                names.insert(n, i.name.clone());
            }
        }
    }
    names
}
