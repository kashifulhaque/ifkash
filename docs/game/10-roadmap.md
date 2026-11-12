# Roadmap

This document breaks *Latticefall* into milestones and work packages (WPs). Each WP is
sized for one AI coding session and has everything an agent needs: what to read, what
to touch, what to build, how to prove it works, and what it depends on.

To run a WP, follow [the agent playbook](11-agent-playbook.md). Track progress in
[`STATUS.md`](STATUS.md).

## How to read a card

Each WP card has the following fields:

- **Lane:** The area of the codebase it owns. Two sessions never own the same lane's
  shared files at once. See [lanes](11-agent-playbook.md#run-work-in-parallel).
- **Depends on:** WPs that must be done first.
- **Size:** S (under a third of a session), M (about half a session), or L (a full
  session). A WP bigger than L must be split.
- **Read:** The spec sections to read. Read nothing else unless stuck.
- **Touch:** The paths it may change.
- **Do:** What to build.
- **Acceptance:** Checkable conditions. All must hold.
- **Verify:** Commands to run. [WP-001](#wp-001) defines the `mise` tasks.

The `mise` tasks used in **Verify** fields are as follows:

| Task | Runs |
|---|---|
| `game:test` | `cargo test --workspace` in `game/`, including golden replays |
| `game:lint` | `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` |
| `game:validate` | `lf-tools validate` on `game/data/` |
| `game:wasm` | Release build of `lf-wasm` into `src/lib/latticefall/wasm/`, plus the WASM determinism test in Node |
| `game:bench` | `lf-tools bench` |
| `game:py` | `maturin develop --release` and `pytest` in `game/python/` |
| (root) `bun run check` | Frontend typecheck |

### Stub items

A WP sometimes needs to place or reward an item that a later WP implements, for example a
region WP placing a relic weapon before the weapon WPs exist. In that case, add the
item's ID to `registry.ron` with a minimal definition marked `stub: true`. The validator
warns on stubs but doesn't fail. The WP that implements the item removes the flag.

## Milestones

| Milestone | Goal | WPs | Exit gate |
|---|---|---|---|
| [M0](#m0-foundations) | Workspace, determinism, content, replays, CI, bridges | 001-008 | Golden replay passes natively, in WASM, and from Python |
| [M1](#m1-movement-and-camera) | A character runs, jumps, and locks on in the browser | 010-015 | **G1:** Rust-core decision gate |
| [M2](#m2-combat-core) | Full combat rules against a training dummy | 020-029 | Every formula in the combat spec has a passing test |
| [M3](#m3-enemies-and-ai) | Enemies, AI, the boss framework, B01, and an RL smoke test | 030-039 | PPO learns something against B01 |
| [M4](#m4-vertical-slice) | Waking Hollow and the Verge, playable for 20-30 minutes | 040-049 | **G2:** owner playtest of the slice |
| [M5](#m5-build-variety) | All weapons, Arts, spells, Charms, armor, and Echoes | 050-059 | All nine builds are equippable |
| [M6](#m6-world-build-out) | Every region, dungeon, the hub, NPCs, and the map | 060-069 | Every Lantern is reachable |
| [M7](#m7-bosses) | Bosses B02-B12 | 070-080 | **G3:** content complete |
| [M8](#m8-polish-and-launch) | Performance, accessibility, async features, codex, launch | 081-089 | **Launch:** the game replaces the old `/game` |
| [M9](#m9-rl-environment) | The full RL environment and first experiments | 090-100 | `pip install` to a trained PPO policy in a day |

### Critical path and parallelism

The critical path is `001 → 002 → 004 → 010 → 020 → 021 → 030 → 033 → 034 → 043 →
044`, which ends at the vertical slice. WP-040 and WP-042 also feed WP-043, so start
them as soon as their own dependencies are done. After M4, most WPs fan out across lanes.
The following table shows waves of WPs that can run in parallel once their dependencies
are done:

| Wave | Can run in parallel |
|---|---|
| 1 | 001 |
| 2 | 002, 003, 005 |
| 3 | 004, 008 |
| 4 | 006, 007, 010 |
| 5 | 011, 012, 013, 015 |
| 6 | 014, 020 |
| 7 | 021, 022, 026 |
| 8 | 023, 024, 027, 028, 029 |
| 9 | 025, 030, 031 |
| 10 | 032, 033, 040 |
| 11 | 034, 035, 041, 042, 045, 047, 048 |
| 12 | 036, 037, 039, 043, 046, 049 |
| 13 | 044, 050, 051, 055, 057, 059 |
| 14 | M5 remainder, M6 regions, M7 bosses (each boss is its own parallel WP) |
| 15 | M8, M9 |

With two or three sessions in parallel, M0-M4 takes roughly 4-6 weeks of evenings and
weekends, and M5-M8 another 6-10 weeks. M9 can start any time after M3.

---

## M0: Foundations

<a id="wp-001"></a>
### WP-001: Scaffold the game workspace

- **Lane:** infra · **Depends on:** none · **Size:** M
- **Read:** [Architecture: repository layout](01-architecture.md#repository-layout),
  [build, test, and deploy](01-architecture.md#build-test-and-deploy).
- **Touch:** `game/**`, `mise.toml`, `.gitignore`, `docs/game/STATUS.md`.
- **Do:**
  - Create the Cargo workspace with crates `lf-core`, `lf-content`, `lf-wasm`, `lf-py`,
    and `lf-tools`, each compiling with a placeholder.
  - Add `game/rust-toolchain.toml` pinning a stable Rust version, and the
    `wasm32-unknown-unknown` target.
  - Add the `mise` tasks listed in [How to read a card](#how-to-read-a-card). `game:wasm`
    and `game:py` can be stubs that print "not yet implemented" until WP-006 and WP-007.
  - Gitignore `game/target/`, `src/lib/latticefall/wasm/`, and Python build outputs.
- **Acceptance:**
  - `mise run game:test` and `mise run game:lint` pass on a clean clone.
  - `api/` is untouched and still builds.
- **Verify:** `mise run game:test`, `mise run game:lint`, `cd api && cargo check`.

<a id="wp-002"></a>
### WP-002: Add deterministic math, random streams, and the tick skeleton

- **Lane:** core · **Depends on:** 001 · **Size:** M
- **Read:** [The simulation core](01-architecture.md#the-simulation-core),
  [determinism contract](01-architecture.md#determinism-contract).
- **Touch:** `game/crates/core/src/{lib.rs,world.rs,tick.rs,math/,rng.rs}`.
- **Do:**
  - `math::dmath`: `sin`, `cos`, `atan2`, `sqrt`, `exp`, `powf`, and `lerp` wrappers over
    `libm`, plus vector helpers on `glam` or `nalgebra` types (match Rapier's choice).
  - `RngStreams` with independent PCG32 streams `ai`, `combat`, `loot`, `world`, seeded
    from one `u64`.
  - `World` with `tick`, `rng`, an empty actor arena, and an event queue. Derive
    `Clone`, `Serialize`, `Deserialize`.
  - `tick(&mut World, &TickInput)` running the eleven systems in order as empty
    functions, each in its own module.
  - `World::state_hash()` over a canonical `postcard` serialization (excluding events).
  - A clippy lint configuration in `lf-core` that denies `std::collections::HashMap`
    and `HashSet` (use `clippy::disallowed_types`).
- **Acceptance:**
  - A unit test runs 10,000 ticks from seed 42 twice and gets the same hash.
  - Cloning a world mid-run and ticking both copies gives identical hashes.
  - Using `HashMap` in `lf-core` fails `game:lint`.
- **Verify:** `mise run game:test`, `mise run game:lint`.

<a id="wp-003"></a>
### WP-003: Build the content crate

- **Lane:** core · **Depends on:** 001 · **Size:** L
- **Read:** [Content pipeline](01-architecture.md#content-pipeline),
  [move data schema](03-combat.md#move-data-schema), [attributes](03-combat.md#attributes).
- **Touch:** `game/crates/content/**`, `game/crates/tools/src/{validate.rs,gen_ts.rs}`,
  `game/data/{registry.ron,attributes.ron,moves/straight_sword.ron}`,
  `src/lib/latticefall/gen/`.
- **Do:**
  - Typed structs for attributes, moves, and a minimal weapon, deserialized from RON.
  - A loader that reads a directory, and an `include_str!` variant for WASM.
  - `registry.ron`: string ID to stable numeric ID, append-only; the loader fails on a
    missing or reused ID.
  - A content hash over the parsed data.
  - Validation framework: a `Validate` trait with errors and warnings, and checks for
    missing references, out-of-range numbers, and frame windows outside
    `total_frames`.
  - `lf-tools validate` and `lf-tools gen-ts` (`ts-rs` types plus a UI JSON digest).
  - Real data for `attributes.ron` (from the combat spec tables) and the straight-sword
    moveset.
- **Acceptance:**
  - A deliberately broken fixture (a hitbox at frame 40 in a 27-frame move) fails
    validation with a message naming the file, move, and field.
  - `gen-ts` output compiles under `bun run check`.
  - Derived stats at attribute 10 and 40 match the combat spec's table in a unit test.
- **Verify:** `mise run game:test`, `mise run game:validate`, `bun run check`.

<a id="wp-004"></a>
### WP-004: Add inputs, replays, and golden replay tests

- **Lane:** core · **Depends on:** 002 · **Size:** M
- **Read:** [Inputs](01-architecture.md#inputs),
  [state hash and golden replays](01-architecture.md#state-hash-and-golden-replays),
  [persistence](01-architecture.md#persistence).
- **Touch:** `game/crates/core/src/input.rs`, `game/crates/core/src/replay.rs`,
  `game/crates/tools/src/replay.rs`, `game/tests/replays/`.
- **Do:**
  - `ActionFrame`, the `buttons` bit constants, `Command`, and `TickInput`.
  - The replay format: header (format version, content hash, seed, scenario ID),
    run-length-encoded action frames, and commands, with expected hashes every 30 ticks.
  - `lf-tools replay run FILE` and `lf-tools replay rehash [FILE]`.
  - A test harness that runs every replay in `game/tests/replays/` and compares hashes.
  - One placeholder golden replay (random inputs for 3,000 ticks).
- **Acceptance:**
  - Round-trip encode and decode is lossless (property test).
  - Changing one input in the golden replay makes the test fail and names the first
    mismatching tick.
- **Verify:** `mise run game:test`.

<a id="wp-005"></a>
### WP-005: Add CI and propose the frontend deploy change

- **Lane:** infra · **Depends on:** 001 · **Size:** M
- **Read:** [Build, test, and deploy](01-architecture.md#build-test-and-deploy),
  [ADR 0003](decisions/0003-frontend-deploy.md).
- **Touch:** `.github/workflows/game.yml`, `docs/game/decisions/0003-frontend-deploy.md`.
- **Do:**
  - A workflow that runs on changes to `game/**`: test, lint, validate, WASM build and
    WASM determinism test (once WP-006 exists), and a bench smoke test.
  - Cache Cargo registries and `game/target`.
  - **Don't** change the frontend deploy. Update ADR 0003 with the exact workflow that
    would replace the Pages Git integration, then stop and ask the owner to approve it.
- **Acceptance:**
  - The workflow passes on a pull request.
  - ADR 0003 has the status "Proposed" and a concrete plan.
- **Verify:** The workflow run on the pull request.

<a id="wp-006"></a>
### WP-006: Bridge the core to the browser

- **Lane:** render · **Depends on:** 002, 004 · **Size:** L
- **Read:** [The browser host](01-architecture.md#the-browser-host),
  [bundle and loading](01-architecture.md#bundle-and-loading).
- **Touch:** `game/crates/wasm/**`, `src/lib/latticefall/engine/`,
  `src/routes/game/`, `vite.config.ts` (only if WASM loading needs it).
- **Do:**
  - `lf-wasm` exports: `new_world(seed, scenario)`, `tick(frames)`, `snapshot_ptr()`,
    `snapshot_len()`, `drain_events()`, `raycast(origin, dir, max)`, and
    `state_hash()`.
  - `mise run game:wasm` builds with `wasm-bindgen` and `wasm-opt -O3`, and runs the
    golden replays under WASM in Node, comparing hashes with native.
  - A TypeScript engine: WASM loader, fixed-step accumulator loop, snapshot views
    recreated on memory growth, and interpolation.
  - A Three.js scene with a ground plane and a capsule per actor, at `/game?engine=lf`.
    The old game stays the default at `/game` until WP-086.
- **Acceptance:**
  - WASM and native produce identical hashes for every golden replay.
  - The capsule renders and moves smoothly at 60 frames per second with the core at 30
    ticks per second.
  - The home page bundle doesn't include the WASM module or the new engine (check the
    build output).
- **Verify:** `mise run game:wasm`, `bun run check`, `bun run build`, screenshot of
  `/game?engine=lf&dev=1`.

<a id="wp-007"></a>
### WP-007: Add the Python binding stub

- **Lane:** rl · **Depends on:** 002, 004 · **Size:** M
- **Read:** [The Python host](01-architecture.md#the-python-host),
  [Python API](09-rl-environment.md#python-api).
- **Touch:** `game/crates/py/**`, `game/python/**`.
- **Do:**
  - `lf-py` with a `Sim` class: `reset(seed, scenario)`, `step(action_frame_bytes)`,
    `clone()`, `state_hash()`.
  - `game/python/pyproject.toml` with maturin, and a `latticefall` package that imports
    the extension.
  - A pytest that replays the golden replay through Python and checks the hash.
  - A tiny benchmark printing steps per second.
- **Acceptance:**
  - `mise run game:py` passes.
  - The Python hash equals the native hash. CI also compares it with WASM once WP-006 is
    merged.
- **Verify:** `mise run game:py`.

<a id="wp-008"></a>
### WP-008: Add stable portfolio slugs

- **Lane:** ui · **Depends on:** 003 · **Size:** S
- **Read:** [Portfolio integration: rules and stable slugs](08-portfolio-integration.md#rules).
- **Touch:** `src/lib/content.ts`, `scripts/export-content.ts`,
  `game/data/portfolio.ron`, `game/data/gen/portfolio_content.json`,
  `game/crates/content/src/portfolio.rs`.
- **Do:**
  - Add an optional `slug` to `LootItem` and set it on every item.
  - A Bun script that exports `content.ts` to JSON for the Rust validator.
  - `portfolio.ron` with the relic and Chronicle Page mappings from the portfolio spec.
  - Validation: every referenced slug exists; unmapped projects produce a warning.
- **Acceptance:**
  - Removing a slug from `content.ts` makes `game:validate` fail with the slug's name.
  - Pages that use `content.ts` still render the same.
- **Verify:** `bun run check`, `mise run game:validate`.

## M1: Movement and camera

<a id="wp-010"></a>
### WP-010: Add physics and the character controller

- **Lane:** core · **Depends on:** 002, 003, 004 · **Size:** L
- **Read:** [Physics](01-architecture.md#physics),
  [traversal and world mechanics](02-world.md#traversal-and-world-mechanics) (jump and fall
  rows only), [combat verbs](03-combat.md#combat-verbs) (sprint and jump).
- **Touch:** `game/crates/core/src/{physics/,actor/}`, `game/data/scenarios/test_field.ron`,
  `game/tests/replays/`.
- **Do:**
  - Rapier with `enhanced-determinism` and no SIMD features; a `PhysicsState` that
    clones and serializes.
  - A test scenario: a heightfield with a slope, steps, a 1 m ledge, and a 10 m drop.
  - Player actor with a kinematic capsule and Rapier's `KinematicCharacterController`:
    walk, run, sprint (stamina stub), jump, gravity, slope limits, step-up, snap to
    ground, and fall damage.
  - Movement from `ActionFrame.move_x` and `move_z`.
  - A golden replay that walks, sprints, jumps the ledge, and falls the drop.
- **Acceptance:**
  - Hashes match natively, and in WASM and Python if WP-006 and WP-007 are merged.
    Otherwise the WP that merges second adds those checks.
  - The 10 m fall applies the damage the world spec defines.
  - `game:bench` reports over 20,000 ticks per second for one player on the test field.
- **Verify:** `mise run game:test`, `mise run game:wasm`, `mise run game:py`,
  `mise run game:bench`.

<a id="wp-011"></a>
### WP-011: Map keyboard, mouse, and gamepad input

- **Lane:** render · **Depends on:** 006 · **Size:** M
- **Read:** [Controls](07-presentation.md#controls), [ActionFrame](01-architecture.md#actionframe).
- **Touch:** `src/lib/latticefall/input/`.
- **Do:**
  - Keyboard via `KeyboardEvent.code`, mouse via pointer lock, gamepad via
    `navigator.getGamepads()` polled per frame.
  - Camera-relative stick input converted to world-space `move_x` and `move_z`.
  - A binding table with the defaults, saved to `localStorage`, plus toggle-instead-of-
    hold and one-button-attack options.
  - Browser replay recording: every `TickInput` appended to an in-memory replay, with a
    dev-only **Download replay** button.
- **Acceptance:**
  - A replay downloaded from the browser reproduces the same final hash in
    `lf-tools replay run`.
- **Verify:** `bun run check`; record a replay in the browser and run it natively.

<a id="wp-012"></a>
### WP-012: Build the third-person camera

- **Lane:** render · **Depends on:** 006, 010 · **Size:** M
- **Read:** [Camera](07-presentation.md#camera).
- **Touch:** `src/lib/latticefall/render/camera/`.
- **Do:** Orbit camera, pitch clamps, collision through the `raycast` export, smoothing,
  lock-on framing (reads the lock target from the snapshot), and field-of-view and
  distance settings.
- **Acceptance:** The camera never clips into the test field's walls in a scripted orbit
  around the ledge; lock-on keeps both actors on screen.
- **Verify:** `bun run check`, screenshots from `/game?engine=lf&dev=1&cam=orbit`.

<a id="wp-013"></a>
### WP-013: Render and animate characters

- **Lane:** render · **Depends on:** 006 · **Size:** L
- **Read:** [Characters and animation](07-presentation.md#characters-and-animation),
  [assets](07-presentation.md#assets).
- **Touch:** `src/lib/latticefall/render/actors/`, `game/assets-src/`,
  `static/latticefall/`, `game/crates/tools/src/assets.rs`.
- **Do:**
  - `lf-tools assets` pipeline with `gltf-transform` (meshopt, KTX2).
  - Load a Quaternius humanoid and the Universal Animation Library clips; record both
    licenses in `game/assets-src/`.
  - The move-to-clip mapping, scrubbing by `move_frame`, locomotion blending by speed,
    and the generic-pose fallback for missing clips.
- **Acceptance:** The player model idles, walks, runs, sprints, jumps, and lands with no
  foot sliding visible at run speed in screenshots.
- **Verify:** `bun run check`, screenshots.

<a id="wp-014"></a>
### WP-014: Add developer tools and dev URLs

- **Lane:** render · **Depends on:** 006, 012 · **Size:** M
- **Read:** [Verify visually](11-agent-playbook.md#verify-visually).
- **Touch:** `src/lib/latticefall/dev/`, `game/crates/wasm/src/debug.rs`.
- **Do:**
  - URL parameters: `dev`, `spawn`, `cam`, `overlay`, `freeze`, `seed`, `scenario`.
  - Overlay panel: core tick time, frame time, draw calls, triangles, actor list,
    state hash.
  - Debug draw for colliders, hitboxes, hurtboxes, the nav grid, and AI state (each
    behind an overlay flag; empty until those systems exist).
  - Teleport to a Lantern, god mode, and set-level commands (dev-only `Command`s, compiled
    out of release builds).
- **Acceptance:** Every URL parameter works on the test field; overlays don't run when
  `dev` is absent.
- **Verify:** `bun run check`, screenshots with each overlay.

<a id="wp-015"></a>
### WP-015: Add lock-on and targeting to the core

- **Lane:** core · **Depends on:** 010 · **Size:** S
- **Read:** [Lock-on](03-combat.md#lock-on).
- **Touch:** `game/crates/core/src/actor/targeting.rs`.
- **Do:** Target acquisition, breaking, auto-retarget, switching left and right, and
  strafing locomotion while locked. Add three dummy actors to the test field.
- **Acceptance:** Unit tests cover acquisition by angle and distance, break at 30 m,
  and auto-retarget within 8 m.
- **Verify:** `mise run game:test`.

### Gate G1: Rust-core decision

After M1, the owner reviews the following and either confirms ADR 0001 or writes a
superseding ADR:

- WASM size (gzipped) and native and WASM ticks per second from `game:bench`.
- How much session time went to toolchain issues versus features.
- Whether golden replays have stayed green across native, WASM, and Python.

## M2: Combat core

<a id="wp-020"></a>
### WP-020: Build the move system

- **Lane:** combat · **Depends on:** 003, 010 · **Size:** L
- **Read:** [Combat verbs](03-combat.md#combat-verbs), [input buffer](03-combat.md#input-buffer),
  [move data schema](03-combat.md#move-data-schema).
- **Touch:** `game/crates/core/src/combat/moves.rs`, `game/data/moves/straight_sword.ron`.
- **Do:** Per-actor move state, frame advance, chains, cancel windows, the 8-tick input
  buffer, root motion curves, tracking toward the lock target, hyperarmor and i-frame
  windows as flags, and stamina and FP costs (stamina stub until WP-022).
- **Acceptance:** Unit tests: a light chain of three; a dodge buffered during recovery
  fires on `cancel_dodge_from`; root motion moves 0.9 m.
- **Verify:** `mise run game:test`.

<a id="wp-021"></a>
### WP-021: Resolve hitboxes and hits

- **Lane:** combat · **Depends on:** 020 · **Size:** L
- **Read:** [Physics](01-architecture.md#physics), [tick schedule](01-architecture.md#tick-schedule),
  [move data schema](03-combat.md#move-data-schema).
- **Touch:** `game/crates/core/src/combat/{hitbox.rs,hurtbox.rs,resolve.rs}`,
  `game/crates/core/src/events.rs`.
- **Do:** Hitbox shapes in actor space, hurtbox capsules per actor, swept tests between
  the previous and current frame to prevent tunneling, one hit per target per move,
  deterministic ordering, a training dummy actor, and `Hit` events.
- **Acceptance:** A golden replay where the player hits the dummy with a three-hit chain
  produces exactly three hits; a fast attack against a thin target doesn't tunnel.
- **Verify:** `mise run game:test`, `mise run game:wasm`.

<a id="wp-022"></a>
### WP-022: Add stamina, dodging, and guarding

- **Lane:** combat · **Depends on:** 020 · **Size:** M
- **Read:** [Combat verbs](03-combat.md#combat-verbs),
  [equip load and roll tiers](03-combat.md#equip-load-and-roll-tiers),
  [regeneration](03-combat.md#regeneration),
  [guard and guard counter](03-combat.md#guard-and-guard-counter).
- **Touch:** `game/crates/core/src/combat/{stamina.rs,dodge.rs,guard.rs}`,
  `game/data/moves/common.ron`.
- **Do:** Stamina costs and regeneration, roll and backstep per load tier, i-frames that
  skip hit resolution, sprint drain, guard absorption, guard break, and guard counters.
- **Acceptance:** Tests: a hit during roll frames 2-14 does nothing; a hit on frame 15
  connects; guard break triggers at zero stamina.
- **Verify:** `mise run game:test`.

<a id="wp-023"></a>
### WP-023: Implement the damage pipeline

- **Lane:** combat · **Depends on:** 021 · **Size:** M
- **Read:** [Attributes](03-combat.md#attributes), [damage](03-combat.md#damage).
- **Touch:** `game/crates/core/src/combat/damage.rs`, `game/crates/core/src/actor/stats.rs`,
  `game/data/attributes.ron`.
- **Do:** Derived stats, AR with upgrades, requirements and the unwieldy penalty,
  two-handing, defense, absorption, weakness, and counter-hit multipliers.
- **Acceptance:** Table-driven tests reproduce hand-computed values for at least ten
  cases, including an unmet requirement and two-handing.
- **Verify:** `mise run game:test`.

<a id="wp-024"></a>
### WP-024: Add poise, stance, parries, and critical hits

- **Lane:** combat · **Depends on:** 021, 022 · **Size:** L
- **Read:** [Poise and stance](03-combat.md#poise-and-stance),
  [parry and critical hits](03-combat.md#parry-and-critical-hits).
- **Touch:** `game/crates/core/src/combat/{poise.rs,crit.rs}`, `game/data/moves/crits.ron`.
- **Do:** Poise accumulation and reset, light and heavy staggers, hyperarmor, stance
  meters and stance breaks, parry windows and the parried state, paired critical moves,
  and riposte and backstab triggers.
- **Acceptance:** Golden replays: parry the dummy then riposte; break its stance with
  charged heavies; backstab it while it faces away.
- **Verify:** `mise run game:test`, `mise run game:wasm`.

<a id="wp-025"></a>
### WP-025: Add status effects and buffs

- **Lane:** combat · **Depends on:** 023 · **Size:** M
- **Read:** [Status effects](03-combat.md#status-effects), [buffs](03-combat.md#buffs).
- **Touch:** `game/crates/core/src/combat/status.rs`, `game/data/status.ron`.
- **Do:** Six buildup meters, decay, procs, the 60-tick lockout, boss tolerance growth,
  every status effect, and the weapon-buff and body-buff slots.
- **Acceptance:** Tests for each status: proc timing at a given buildup per hit, and the
  boss tolerance after two procs.
- **Verify:** `mise run game:test`.

<a id="wp-026"></a>
### WP-026: Add inventory, items, and flasks

- **Lane:** combat · **Depends on:** 020 · **Size:** M
- **Read:** [Equipment slots](06-progression-and-systems.md#equipment-slots),
  [flasks](06-progression-and-systems.md#flasks), [consumables](04-arsenal-and-builds.md#consumables).
- **Touch:** `game/crates/core/src/items/`, `game/data/consumables.ron`.
- **Do:** Inventory, equipment slots, quick slots and cycling, `Command`s for equipping,
  flask drinking as a move, flask charges and splits, and pickups.
- **Acceptance:** Tests: drinking heals on frame 16 and is interrupted by a stagger
  before then (no heal); equip commands replay correctly.
- **Verify:** `mise run game:test`.

<a id="wp-027"></a>
### WP-027: Add projectiles

- **Lane:** combat · **Depends on:** 021 · **Size:** M
- **Read:** [Physics](01-architecture.md#physics), [spells](03-combat.md#spells).
- **Touch:** `game/crates/core/src/combat/projectile.rs`.
- **Do:** Shape-cast projectiles with speed, gravity, homing, lifetime, pierce count,
  and collision with level geometry; spawned from move data.
- **Acceptance:** A golden replay with an arrow and a homing projectile.
- **Verify:** `mise run game:test`.

<a id="wp-028"></a>
### WP-028: Hook up combat VFX and hit feedback

- **Lane:** render · **Depends on:** 021, 013 · **Size:** M
- **Read:** [VFX](07-presentation.md#vfx).
- **Touch:** `src/lib/latticefall/render/vfx/`.
- **Do:** Weapon trails, hit sparks by damage type, hitstop, camera shake (off with
  reduced motion), and status auras, all driven by events and snapshots.
- **Acceptance:** Screenshots at hit frames show sparks and trails; reduced motion turns
  off shake.
- **Verify:** `bun run check`, screenshots with `&freeze=1`.

<a id="wp-029"></a>
### WP-029: Add death, Remnants, and Lanterns

- **Lane:** core · **Depends on:** 026 · **Size:** M
- **Read:** [Glimmer and death](06-progression-and-systems.md#glimmer-and-death),
  [Lanterns](06-progression-and-systems.md#lanterns).
- **Touch:** `game/crates/core/src/world_state/{lantern.rs,death.rs,flags.rs}`.
- **Do:** Death, respawn at the last rested Lantern, Remnant drop at the last safe
  position and recovery, Lantern resting (refill, respawn enemies, set respawn), and
  world flags.
- **Acceptance:** A golden replay: die, respawn, recover the Remnant; die twice and lose
  it.
- **Verify:** `mise run game:test`.

## M3: Enemies and AI

<a id="wp-030"></a>
### WP-030: Build the AI framework

- **Lane:** ai · **Depends on:** 020, 021 · **Size:** L
- **Read:** [AI framework](05-enemies-and-bosses.md#ai-framework).
- **Touch:** `game/crates/core/src/ai/{mod.rs,perception.rs,options.rs,behaviors.rs,tokens.rs}`,
  `game/crates/content/src/enemy.rs`.
- **Do:** Awareness states, sight and hearing, option selection with conditions and
  cooldowns on the `ai` random stream, built-in behaviors, attack tokens, leashing, and
  return-and-heal. Think at 10 Hz staggered by index.
- **Acceptance:** Tests: an enemy behind a wall doesn't see the player; a sprint within
  10 m makes it suspicious; three enemies never attack at once.
- **Verify:** `mise run game:test`.

<a id="wp-031"></a>
### WP-031: Add the navigation grid and pathfinding

- **Lane:** world · **Depends on:** 010 · **Size:** M
- **Read:** [Navigation](01-architecture.md#navigation).
- **Touch:** `game/crates/core/src/nav/`, `game/crates/tools/src/bake/nav.rs`.
- **Do:** Grid baking from colliders (walkable, floor height, cost, layers), A* with an
  expansion budget, path smoothing, and per-tick request limits.
- **Acceptance:** Paths around the test field's obstacles; a request that exceeds the
  budget fails cleanly; baking is deterministic (same hash twice).
- **Verify:** `mise run game:test`.

<a id="wp-032"></a>
### WP-032: Add enemy batch 1

- **Lane:** ai · **Depends on:** 030, 031, 024, 025 · **Size:** L
- **Read:** [Enemy roster](05-enemies-and-bosses.md#enemy-roster) (rows for Husk, Thorn
  Hound, Gray Knight, Gray Archer, Whitewarden Skeleton).
- **Touch:** `game/data/enemies/`, `game/data/moves/enemies/`,
  `game/crates/core/src/ai/special/` (revive logic).
- **Do:** Data and movesets for the five enemies and the Husk variants, and the
  skeleton's revive rule.
- **Acceptance:** Each enemy has a golden replay of a short fight; each attack has a
  telegraph of at least 12 ticks (validator check on enemy moves).
- **Verify:** `mise run game:test`, `mise run game:validate`.

<a id="wp-033"></a>
### WP-033: Build the boss framework

- **Lane:** ai · **Depends on:** 030 · **Size:** M
- **Read:** [Boss framework](05-enemies-and-bosses.md#boss-framework),
  [fairness rules](05-enemies-and-bosses.md#fairness-rules).
- **Touch:** `game/crates/core/src/ai/boss.rs`, `game/crates/content/src/boss.rs`.
- **Do:** Fog gates, intro wait, phases with transition moves, multi-part bosses
  (Choir members, Colossus legs), resets on death, rewards, and validator checks for the
  fairness rules that can be checked statically.
- **Acceptance:** A dummy two-phase boss passes a golden replay including a phase
  change and a reset after the player's death.
- **Verify:** `mise run game:test`, `mise run game:validate`.

<a id="wp-034"></a>
### WP-034: Build B01, the Hollow Sentry

- **Lane:** ai · **Depends on:** 033, 032 · **Size:** M
- **Read:** [B01](05-enemies-and-bosses.md#b01-hollow-sentry).
- **Touch:** `game/data/bosses/b01_hollow_sentry.ron`, `game/data/moves/bosses/b01.ron`,
  `game/data/scenarios/boss_b01.ron`.
- **Do:** Data, moveset, arena scenario, and a scripted test bot that wins by rolling and
  punishing.
- **Acceptance:** The scripted bot beats B01 in a golden replay; a do-nothing bot dies.
- **Verify:** `mise run game:test`.

<a id="wp-035"></a>
### WP-035: Render enemies

- **Lane:** render · **Depends on:** 013, 032 · **Size:** M
- **Read:** [Assets](07-presentation.md#assets), [procedural models](07-presentation.md#procedural-models).
- **Touch:** `src/lib/latticefall/render/actors/`, `src/lib/latticefall/render/procedural/`,
  `game/assets-src/`, `static/latticefall/`.
- **Do:** Models and animation mapping for batch 1 and B01, and the procedural model
  framework (primitives, bone hierarchy, procedural animation by move frame).
- **Acceptance:** Screenshots of each enemy idle and mid-attack.
- **Verify:** `bun run check`, screenshots.

<a id="wp-036"></a>
### WP-036: Add enemy batch 2

- **Lane:** ai · **Depends on:** 032, 025, 027 · **Size:** L
- **Read:** [Enemy roster](05-enemies-and-bosses.md#enemy-roster) (Acolyte, Drowned,
  Reed Stalker, Marsh Crawler, Cave Moth, Lamp Mite, Shard Golem, Glasswing Stalker).
- **Touch:** `game/data/enemies/`, `game/data/moves/enemies/`, `game/crates/core/src/ai/special/`.
- **Do:** Data and movesets, plus special rules: water-only movement, reed hiding,
  flying (hover height and dives), swarm explosion, Formula reflection, and shimmer
  invisibility (a render flag).
- **Acceptance:** A golden replay per enemy; validator telegraph checks pass.
- **Verify:** `mise run game:test`, `mise run game:validate`.

<a id="wp-037"></a>
### WP-037: Add enemy batch 3 and the elites

- **Lane:** ai · **Depends on:** 032, 025, 027 · **Size:** L
- **Read:** [Enemy roster](05-enemies-and-bosses.md#enemy-roster) (Lantern Thief, Rime
  Wolf, Mimic Chest, and the five elites).
- **Touch:** `game/data/enemies/`, `game/data/moves/enemies/`.
- **Do:** Data and movesets; pack circling for wolves; the mimic's disguise state.
- **Acceptance:** A golden replay per enemy and elite.
- **Verify:** `mise run game:test`, `mise run game:validate`.

<a id="wp-039"></a>
### WP-039: Run an RL smoke test

- **Lane:** rl · **Depends on:** 007, 034 · **Size:** M
- **Read:** [RL environment: scenarios, observations, actions, rewards](09-rl-environment.md#scenarios).
- **Touch:** `game/crates/py/`, `game/crates/core/src/obs/`, `game/python/latticefall/`,
  `game/python/examples/ppo_smoke.py`.
- **Do:** A minimal `BossArena-v0` for B01 with `proprio` and `target` observations,
  the `MultiDiscrete` action space, and the default rewards; run PPO for 30 minutes on
  a laptop CPU.
- **Acceptance:** Average return improves clearly over a random policy; steps per second
  are logged; any problem with the observations or actions is written up as an ADR or a
  spec change. This WP exists to catch RL design mistakes before content scales up.
- **Verify:** `mise run game:py`, `python game/python/examples/ppo_smoke.py --minutes 30`.

## M4: Vertical slice

<a id="wp-040"></a>
### WP-040: Build the level format and baker

- **Lane:** world · **Depends on:** 010, 031 · **Size:** L
- **Read:** [Levels](01-architecture.md#levels), [build an area](02-world.md#build-an-area),
  [procedural dressing](02-world.md#procedural-dressing).
- **Touch:** `game/crates/content/src/level.rs`, `game/crates/tools/src/bake/`,
  `game/data/levels/test_field/`, `game/data/kits/`.
- **Do:** Area, terrain, layout, entities, and dressing formats; sculpt operations;
  kit catalogs with collider shapes; `lf-tools bake` producing colliders, the nav grid,
  and a render manifest; and bake warnings (unreachable Lanterns, items off the nav grid,
  spawns inside colliders).
- **Acceptance:** The test field is rebuilt from data; a deliberately unreachable
  Lantern produces a warning.
- **Verify:** `mise run game:test`, `cargo run -p lf-tools -- bake test_field`.

<a id="wp-041"></a>
### WP-041: Render terrain, kits, and atmosphere

- **Lane:** render · **Depends on:** 040 · **Size:** L
- **Read:** [Art direction](07-presentation.md#art-direction),
  [performance budgets](01-architecture.md#performance-budgets),
  [quality presets](07-presentation.md#quality-presets).
- **Touch:** `src/lib/latticefall/render/{terrain,kits,sky,lighting}/`.
- **Do:** Heightfield mesh with vertex colors, kit instancing with `InstancedMesh` and
  `BatchedMesh`, scatter rendering, fog, sky, per-region lighting, quality presets, and
  region asset streaming from the manifest.
- **Acceptance:** The test field renders within the draw-call and triangle budgets on
  the medium preset.
- **Verify:** `bun run check`, screenshots, the dev overlay's counters.

<a id="wp-042"></a>
### WP-042: Add world mechanics

- **Lane:** world · **Depends on:** 040, 029 · **Size:** L
- **Read:** [Traversal and world mechanics](02-world.md#traversal-and-world-mechanics).
- **Touch:** `game/crates/core/src/world_state/`, `src/lib/latticefall/render/props/`.
- **Do:** Ladders, lifts, doors, levers, hollow walls, fog gates, pickups, chests,
  authored messages, water and snow modifiers, and their rendering.
- **Acceptance:** A golden replay that rides a lift, climbs a ladder, opens a one-way
  door, and reveals a hollow wall.
- **Verify:** `mise run game:test`, screenshots.

<a id="wp-043"></a>
### WP-043: Build Waking Hollow

- **Lane:** world · **Depends on:** 040, 042, 034 · **Size:** L
- **Read:** [Hollowhearth and the Verge](02-world.md#the-verge),
  [dungeon layout template](02-world.md#dungeon-layout-template),
  [B01](05-enemies-and-bosses.md#b01-hollow-sentry).
- **Touch:** `game/data/levels/waking_hollow/`.
- **Do:** The tutorial cave from L01 to B01, teaching movement, attacks, dodging,
  guarding, items, and Lanterns through authored messages and safe rooms.
- **Acceptance:** A golden replay of the critical path; a first-time tester (the owner)
  reaches B01 without reading controls outside the game.
- **Verify:** `mise run game:test`, screenshots at every Lantern and the arena.

<a id="wp-044"></a>
### WP-044: Build the Verge

- **Lane:** world · **Depends on:** 043, 032, 036, 037, 041 · **Size:** L (split into
  greybox and art passes if needed)
- **Read:** [The Verge](02-world.md#the-verge), [design rules](02-world.md#design-rules),
  [build an area](02-world.md#build-an-area).
- **Touch:** `game/data/levels/verge/`.
- **Do:** Terrain, the aqueduct loop, Ninefold Chapel, the windmill, the Briar Thicket
  (the B02 arena), Lanterns L02-L05, enemy placement, the chapel's hollow wall, and the
  Verge's relics (banana.cpp, the Shrike-Lite Circuit), Chronicle Page slot 1, items,
  and materials.
- **Acceptance:** Every walk between Lanterns is under 45 seconds without content; the
  Spire silhouette is visible from L02; a golden replay walks the main route.
- **Verify:** `mise run game:test`, screenshots from each Lantern and the landmark view.

<a id="wp-045"></a>
### WP-045: Build the HUD

- **Lane:** ui · **Depends on:** 026, 033 · **Size:** M
- **Read:** [HUD](07-presentation.md#hud).
- **Touch:** `src/lib/latticefall/ui/hud/`.
- **Do:** Bars, status meters, quick slots, Glimmer, the boss bar, prompts,
  notifications, and HUD scale, using site tokens in both themes.
- **Acceptance:** Screenshots in light and dark themes at 75% and 150% HUD scale.
- **Verify:** `bun run check`, screenshots.

<a id="wp-046"></a>
### WP-046: Build the menus

- **Lane:** ui · **Depends on:** 045, 023 · **Size:** L
- **Read:** [Menus](07-presentation.md#menus), [Lanterns](06-progression-and-systems.md#lanterns),
  [accessibility](07-presentation.md#accessibility).
- **Touch:** `src/lib/latticefall/ui/menus/`.
- **Do:** Pause, equipment (with stat previews), inventory, the Lantern menu (level up,
  attune, flasks, travel), and settings (controls, audio, graphics, difficulty, and basic
  accessibility). All state changes go through `Command`s.
- **Acceptance:** Every menu works with keyboard only, gamepad only, and a screen reader
  announces item names.
- **Verify:** `bun run check`, screenshots.

<a id="wp-047"></a>
### WP-047: Add saving and loading

- **Lane:** core · **Depends on:** 029 · **Size:** M
- **Read:** [Persistence](01-architecture.md#persistence).
- **Touch:** `game/crates/core/src/save.rs`, `src/lib/latticefall/engine/save.ts`.
- **Do:** A versioned `postcard` save blob, three IndexedDB slots, autosave on rest and
  on area change, and migration stubs for future versions.
- **Acceptance:** Save, reload the page, load: the world hash of the persistent state
  matches.
- **Verify:** `mise run game:test`, `bun run check`.

<a id="wp-048"></a>
### WP-048: Build the audio engine

- **Lane:** render · **Depends on:** 028 · **Size:** M
- **Read:** [Audio](07-presentation.md#audio).
- **Touch:** `src/lib/latticefall/audio/`, `static/latticefall/audio/`.
- **Do:** Buses, positional sounds from events, footsteps by surface, a procedural
  ambient bed per region, and boss music stem switching (placeholders).
- **Acceptance:** Volume settings work per bus; muting persists.
- **Verify:** `bun run check`.

<a id="wp-049"></a>
### WP-049: Add the landing card, the Ledger, and the codex store

- **Lane:** ui · **Depends on:** 008, 045 · **Size:** M
- **Read:** [Portfolio integration](08-portfolio-integration.md).
- **Touch:** `src/routes/game/`, `src/lib/latticefall/codex/`, `src/lib/latticefall/ui/portfolio/`.
- **Do:** The landing card; the Wayfarer's Ledger; the facts panel component; the codex
  unlock store (no Three.js or WASM import); and relic and Chronicle Page pickups that
  unlock codex entries, tested with a stub relic on the test field.
- **Acceptance:** The landing card shows name, role, résumé, and contact within 3
  seconds with the network throttled to "Fast 4G"; picking up the test relic unlocks
  its codex entry.
- **Verify:** `bun run check`, `bun run build`, screenshots.

### Gate G2: Vertical slice

The owner plays Waking Hollow and the Verge for 20-30 minutes and decides:

- Does combat feel good? If not, which numbers change?
- Is the scope still right? If the slice took far longer than planned, cut regions or
  bosses now (an ADR records the cut).
- Is the RL smoke test result (WP-039) encouraging?

## M5: Build variety

<a id="wp-050"></a>
### WP-050: Add leveling and respec

- **Lane:** combat · **Depends on:** 023, 046 · **Size:** S
- **Read:** [Leveling](06-progression-and-systems.md#leveling), [respec](04-arsenal-and-builds.md#respec).
- **Touch:** `game/crates/core/src/actor/level.rs`.
- **Do:** Level-up commands with the cost curve, and respec.
- **Acceptance:** Tests match the leveling table.
- **Verify:** `mise run game:test`.

<a id="wp-051"></a>
### WP-051: Add movesets, batch A

- **Lane:** combat · **Depends on:** 024 · **Size:** L
- **Read:** [Weapon classes](04-arsenal-and-builds.md#weapon-classes), [combat verbs](03-combat.md#combat-verbs).
- **Touch:** `game/data/moves/{dagger,greatsword,curved_sword,thrusting_sword,axe,hammer}.ron`.
- **Do:** Full movesets (light chain, heavy, charged, running, rolling, backstep, jump,
  plunge, guard, and critical moves) for six classes.
- **Acceptance:** Validation passes; one golden replay per class against the dummy.
- **Verify:** `mise run game:test`, `mise run game:validate`.

<a id="wp-052"></a>
### WP-052: Add movesets, batch B

- **Lane:** combat · **Depends on:** 051, 027 · **Size:** L
- **Read:** As WP-051.
- **Touch:** `game/data/moves/{colossal,spear,halberd,fist,bow,shield,catalyst}.ron`,
  `game/crates/core/src/combat/{bow.rs,two_hand.rs}`.
- **Do:** The remaining classes, bow aiming (locked-on and free aim), shield bashes,
  spear attacks while guarding, and two-handing.
- **Acceptance:** As WP-051.
- **Verify:** `mise run game:test`, `mise run game:validate`.

<a id="wp-053"></a>
### WP-053: Add all weapons, upgrades, and tempering

- **Lane:** combat · **Depends on:** 052 · **Size:** M
- **Read:** [Weapon list](04-arsenal-and-builds.md#weapon-list), [temper](04-arsenal-and-builds.md#temper),
  [upgrades](04-arsenal-and-builds.md#upgrades).
- **Touch:** `game/data/weapons/`, `game/crates/core/src/items/upgrade.rs`.
- **Do:** All 36 weapons, Forgeglass and Starglass upgrades, tempers, and Smith commands.
- **Acceptance:** Validator cross-checks every weapon's class, moveset, and Art.
- **Verify:** `mise run game:test`, `mise run game:validate`.

<a id="wp-054"></a>
### WP-054: Add Arts

- **Lane:** combat · **Depends on:** 053, 025 · **Size:** M
- **Read:** [Arts](04-arsenal-and-builds.md#arts).
- **Touch:** `game/data/moves/arts.ron`, `game/data/weapons/`.
- **Do:** All 14 Arts, Art Scrolls, and class compatibility.
- **Acceptance:** One golden replay per Art.
- **Verify:** `mise run game:test`.

<a id="wp-055"></a>
### WP-055: Add the spell system and Formulae

- **Lane:** combat · **Depends on:** 027, 046 · **Size:** L
- **Read:** [Spells](03-combat.md#spells), [Formulae](04-arsenal-and-builds.md#formulae-insight-stylus).
- **Touch:** `game/crates/core/src/combat/spells.rs`, `game/data/spells/formulae.ron`.
- **Do:** Catalysts, spell power, attunement, slots, cast speed, and all nine Formulae.
- **Acceptance:** A golden replay per Formula.
- **Verify:** `mise run game:test`.

<a id="wp-056"></a>
### WP-056: Add Litanies

- **Lane:** combat · **Depends on:** 055, 025 · **Size:** M
- **Read:** [Litanies](04-arsenal-and-builds.md#litanies-devotion-reliquary).
- **Touch:** `game/data/spells/litanies.ron`.
- **Do:** All nine Litanies, including heals, cures, buffs, and the channeled breath.
- **Acceptance:** A golden replay per Litany.
- **Verify:** `mise run game:test`.

<a id="wp-057"></a>
### WP-057: Add Charms, armor, and the remaining consumables

- **Lane:** combat · **Depends on:** 022, 023, 027 · **Size:** M
- **Read:** [Charms](04-arsenal-and-builds.md#charms), [armor](04-arsenal-and-builds.md#armor),
  [consumables](04-arsenal-and-builds.md#consumables).
- **Touch:** `game/data/{charms.ron,armor.ron,consumables.ron}`, `game/crates/core/src/items/`.
- **Do:** All Charms with their effects, armor with defense, absorption, poise, and
  weight, the relic armor effects, throwables, oils, arrows, and cures.
- **Acceptance:** Tests for each Charm's effect.
- **Verify:** `mise run game:test`.

<a id="wp-058"></a>
### WP-058: Render equipment

- **Lane:** render · **Depends on:** 053, 057 · **Size:** L
- **Read:** [Characters and animation](07-presentation.md#characters-and-animation).
- **Touch:** `src/lib/latticefall/render/equipment/`, `game/assets-src/`, `static/latticefall/`.
- **Do:** Weapon meshes per class, armor tints and attachments, catalyst and spell VFX.
- **Acceptance:** A dev "armory" scene that shows every weapon and armor set, with
  screenshots.
- **Verify:** `bun run check`, screenshots.

<a id="wp-059"></a>
### WP-059: Add Echo summons

- **Lane:** ai · **Depends on:** 030 · **Size:** M
- **Read:** [Echo summons](05-enemies-and-bosses.md#echo-summons).
- **Touch:** `game/crates/core/src/ai/echo.rs`, `game/data/echoes/`.
- **Do:** Echo Stones, the Echo Bell, ally AI (following, engaging the player's target,
  reverse attack tokens), all four Echoes, and the bow gesture as a `Command` (the
  PoliteLlama reacts to it; WP-088 adds the other gestures).
- **Acceptance:** A golden replay where an Echo helps defeat a test boss with an Echo
  Stone (B01 has none).
- **Verify:** `mise run game:test`.

## M6: World build-out

Each region WP follows [build an area](02-world.md#build-an-area): greybox, play,
then art and dressing. A region WP that runs long splits into `.a` (greybox) and `.b`
(art and dressing).

Each region WP also builds every boss arena in its region (geometry, fog gate, and Echo
Stone; for example, WP-060 builds the Drowned Cathedral's nave), and places the region's
relics, Chronicle Page slot, materials, and key items from
[materials and key items](04-arsenal-and-builds.md#materials-and-key-items), using
[stub items](#stub-items) for anything not implemented yet.

<a id="wp-060"></a>
### WP-060: Build Mirrormere

- **Lane:** world · **Depends on:** 044, 036 · **Size:** L
- **Read:** [Mirrormere](02-world.md#mirrormere).
- **Touch:** `game/data/levels/mirrormere/`, `src/lib/latticefall/render/water/`.
- **Do:** The lake, village, marsh, causeway, and the Drowned Cathedral, including the B06 nave; water levels tied
  to the Belfry sluice; Blight water; the islet with NoPokeDB; Lanterns L06-L09.
- **Acceptance:** As WP-044.
- **Verify:** `mise run game:test`, screenshots.

<a id="wp-061"></a>
### WP-061: Build the Sunken Ossuary and the Drowned Belfry

- **Lane:** world · **Depends on:** 060, 037 · **Size:** L
- **Read:** [Mini-dungeons](02-world.md#mini-dungeons) (D2, D3), [dungeon layout template](02-world.md#dungeon-layout-template).
- **Touch:** `game/data/levels/{sunken_ossuary,drowned_belfry}/`.
- **Do:** Both dungeons, the water-level lever, the bell stun, and the bell-shaft lift.
- **Acceptance:** A golden replay of each dungeon's critical path.
- **Verify:** `mise run game:test`, screenshots.

<a id="wp-062"></a>
### WP-062: Build Hollowdeep and Cinderdeep Mine

- **Lane:** world · **Depends on:** 044, 036 · **Size:** L
- **Read:** [Hollowdeep](02-world.md#hollowdeep), [mini-dungeons](02-world.md#mini-dungeons) (D4).
- **Touch:** `game/data/levels/{hollowdeep,cinderdeep_mine}/`, `src/lib/latticefall/render/lighting/`.
- **Do:** The cavern region, the darkness and light system, mine carts, the Root Stair
  trapdoor, and Lanterns L10-L13.
- **Acceptance:** The critical path stays inside glowing zones (bake check); golden
  replay.
- **Verify:** `mise run game:test`, screenshots.

<a id="wp-063"></a>
### WP-063: Build Mossgut Cave and Glasswing Grotto

- **Lane:** world · **Depends on:** 044, 062 · **Size:** L
- **Read:** [Mini-dungeons](02-world.md#mini-dungeons) (D1, D5).
- **Touch:** `game/data/levels/{mossgut_cave,glasswing_grotto}/`.
- **Do:** Both caves, low ceilings, the lift to Hollowdeep, and the light-beam mirrors.
- **Acceptance:** Golden replays of both critical paths.
- **Verify:** `mise run game:test`, screenshots.

<a id="wp-064"></a>
### WP-064: Build Castle Graymantle, part 1

- **Lane:** world · **Depends on:** 044, 032, 036 · **Size:** L
- **Read:** [Castle Graymantle](02-world.md#castle-graymantle).
- **Touch:** `game/data/levels/castle/`.
- **Do:** The bridge, the cliff path, the Barbican, and the Lower Ward, with the
  gatehouse shortcut, the armory and its mimic, and Lanterns L14-L15.
- **Acceptance:** Golden replay from L14 to the Lower Ward stair.
- **Verify:** `mise run game:test`, screenshots.

<a id="wp-065"></a>
### WP-065: Build Castle Graymantle, part 2

- **Lane:** world · **Depends on:** 064, 037 · **Size:** L
- **Read:** [Castle Graymantle](02-world.md#castle-graymantle).
- **Touch:** `game/data/levels/castle/`.
- **Do:** The ramparts, the keep, the library, the bell tower, the hall, the second
  shortcut, and Lanterns L16-L17.
- **Acceptance:** Golden replay from L15 to the hall's fog gate.
- **Verify:** `mise run game:test`, screenshots.

<a id="wp-066"></a>
### WP-066: Build Rimeshelf, Whitewarden Catacombs, and the Bound Ring

- **Lane:** world · **Depends on:** 065, 037 · **Size:** L
- **Read:** [Rimeshelf](02-world.md#rimeshelf), [mini-dungeons](02-world.md#mini-dungeons) (D6, D7).
- **Touch:** `game/data/levels/{rimeshelf,whitewarden,bound_ring}/`.
- **Do:** Deep snow, the foundry, the chimney drop into the castle, the braziers, the
  standing-stone ring, and Lanterns L18-L21.
- **Acceptance:** Golden replays of the region path and both dungeons.
- **Verify:** `mise run game:test`, screenshots.

<a id="wp-067"></a>
### WP-067: Build the Lattice Spire

- **Lane:** world · **Depends on:** 044, 036, 037 · **Size:** L
- **Read:** [The Lattice Spire](02-world.md#the-lattice-spire).
- **Touch:** `game/data/levels/spire/`.
- **Do:** The gate check for two Great Sigils, the outer spiral with fallback ladders,
  the three gauntlet floors, and Lanterns L22-L23.
- **Acceptance:** Golden replay from L22 to the final fog gate.
- **Verify:** `mise run game:test`, screenshots.

<a id="wp-068"></a>
### WP-068: Build Hollowhearth, NPCs, and questlines

- **Lane:** ui · **Depends on:** 046, 049, 053, 059 · **Size:** L
- **Read:** [Hollowhearth](02-world.md#hollowhearth-hub),
  [NPCs and questlines](06-progression-and-systems.md#npcs-and-questlines),
  [portfolio integration](08-portfolio-integration.md).
- **Touch:** `game/data/levels/hollowhearth/`, `game/data/npcs/`,
  `game/crates/core/src/world_state/quests.rs`, `src/lib/latticefall/ui/dialogue/`.
- **Do:** The hub, the dialogue system with **Ask about** menus, the five NPCs, their
  quest flags, the Archive shelves, the Ledger Stand, the Mirror, and the keepsake table.
- **Acceptance:** Every portfolio entry is reachable from the hub without completing any
  quest (checked by a test that walks the dialogue tree).
- **Verify:** `mise run game:test`, `bun run check`, screenshots.

<a id="wp-069"></a>
### WP-069: Build the map and fast travel

- **Lane:** ui · **Depends on:** 060, 062, 064 · **Size:** M
- **Read:** [Map](06-progression-and-systems.md#map).
- **Touch:** `src/lib/latticefall/ui/map/`, `game/crates/core/src/world_state/map.rs`.
- **Do:** Visited-cell reveal, markers, the list view, and Lantern travel.
- **Acceptance:** The list view is navigable by screen reader; travel replays correctly.
- **Verify:** `mise run game:test`, `bun run check`.

## M7: Bosses

Each boss WP is independent once its arena exists, so boss WPs can run in parallel
sessions even though they share the `ai` lane: each one touches only its own files
(`game/data/bosses/bNN_*.ron`, `game/data/moves/bosses/bNN.ron`,
`game/crates/core/src/ai/bosses/bNN.rs`, `game/data/scenarios/boss_bNN.ron`, and
`src/lib/latticefall/render/bosses/bNN/`).

Every boss WP has the same acceptance criteria:

- Data, moveset, special mechanics, and model with animation.
- The fairness rules pass the validator, and the owner confirms them in play.
- A scripted test bot wins in a golden replay; a do-nothing bot loses.
- A `BossArena-v0` scenario file exists for it.
- Screenshots of the intro, each phase, and the defeat.
- **Verify:** `mise run game:test`, `mise run game:validate`, `bun run check`.

| WP | Boss | Depends on | Size | Read |
|---|---|---|---|---|
| <a id="wp-070"></a>WP-070 | B02 the Briar Stag | 044, 033 | M | [B02](05-enemies-and-bosses.md#b02-the-briar-stag) |
| <a id="wp-071"></a>WP-071 | B03 Bridgewarden Corvane | 064, 033 | M | [B03](05-enemies-and-bosses.md#b03-bridgewarden-corvane) |
| <a id="wp-072"></a>WP-072 | B04 Mossgut Brood | 063, 036 | M | [B04](05-enemies-and-bosses.md#b04-mossgut-brood) |
| <a id="wp-073"></a>WP-073 | B05 Ossuary Keeper | 061, 032 | L | [B05](05-enemies-and-bosses.md#b05-ossuary-keeper) |
| <a id="wp-074"></a>WP-074 | B06 the Drowned Choir | 060, 033 | L | [B06](05-enemies-and-bosses.md#b06-the-drowned-choir) |
| <a id="wp-075"></a>WP-075 | B07 Castellan Voss | 065, 033 | L | [B07](05-enemies-and-bosses.md#b07-castellan-voss-the-overfit-warden) |
| <a id="wp-076"></a>WP-076 | B08 Forgeheart Colossus | 062, 033 | L | [B08](05-enemies-and-bosses.md#b08-forgeheart-colossus) |
| <a id="wp-077"></a>WP-077 | B09 the Lamplight Matriarch | 062, 036 | L | [B09](05-enemies-and-bosses.md#b09-the-lamplight-matriarch) |
| <a id="wp-078"></a>WP-078 | B10 the Bellwright | 066, 048 | L | [B10](05-enemies-and-bosses.md#b10-the-bellwright) |
| <a id="wp-079"></a>WP-079 | B11 the Bound Exile | 066, 027 | M | [B11](05-enemies-and-bosses.md#b11-the-bound-exile) |
| <a id="wp-080"></a>WP-080 | B12 the Unspooled | 067, 074, 075, 077, 078 | L | [B12](05-enemies-and-bosses.md#b12-the-unspooled) |

### Gate G3: Content complete

Every region, dungeon, boss, item, and relic is in the game, and a full playthrough is
possible. The owner plays it end to end before M8 starts.

## M8: Polish and launch

<a id="wp-081"></a>
### WP-081: Meet the performance budgets

- **Lane:** render · **Depends on:** G3 · **Size:** L
- **Read:** [Performance budgets](01-architecture.md#performance-budgets), [bundle and loading](01-architecture.md#bundle-and-loading).
- **Touch:** `src/lib/latticefall/render/`, `static/latticefall/`.
- **Do:** LODs, culling, streaming and unloading by region, texture budgets, and a
  benchmark route that flies a fixed camera path through every region.
- **Acceptance:** Every region meets the frame-time, draw-call, and triangle budgets on
  the medium preset; total download under 60 MB.
- **Verify:** The benchmark route's report.

<a id="wp-082"></a>
### WP-082: Complete accessibility

- **Lane:** ui · **Depends on:** G3 · **Size:** M
- **Read:** [Accessibility](07-presentation.md#accessibility).
- **Touch:** `src/lib/latticefall/ui/`, `src/lib/latticefall/render/`.
- **Do:** Every option in the accessibility list, including color-blind shapes and
  subtitles for important sounds.
- **Acceptance:** A checklist in the pull request against the Game Accessibility
  Guidelines' basic tier.
- **Verify:** `bun run check`, screenshots per mode.

<a id="wp-083"></a>
### WP-083: Add the mobile view and touch mode

- **Lane:** ui · **Depends on:** G3 · **Size:** M
- **Read:** [Mobile](07-presentation.md#mobile).
- **Touch:** `src/routes/game/`, `src/lib/latticefall/input/touch/`.
- **Do:** The phone codex view, the trailer clip, and touch mode.
- **Acceptance:** Screenshots at 375 px width; touch mode holds 30 frames per second on
  the low preset on a mid-range phone.
- **Verify:** `bun run check`, screenshots with the mobile viewport.

<a id="wp-084"></a>
### WP-084: Add asynchronous features and telemetry

- **Lane:** infra · **Depends on:** G3 · **Size:** L
- **Read:** [Asynchronous features](06-progression-and-systems.md#asynchronous-features),
  [telemetry](06-progression-and-systems.md#telemetry).
- **Touch:** `api/src/handlers/latticefall.rs`, `api/src/routes/mod.rs`,
  `api/src/routes/openapi.rs`, `api/migrations/NNNN_latticefall.sql`,
  `src/lib/latticefall/online/`.
- **Do:** Messages, Afterimages, Wanderers, cloud saves (`PUT /api/latticefall/save`),
  Mark sync, the speedrun leaderboard, and telemetry endpoints, with rate limits, the
  90-day cleanup, and the client side.
- **Acceptance:** Works against the local Worker; signed-out reads work; writes need a
  token.
- **Verify:** `cd api && cargo check`, local Worker test with `mise run api:dev`.

<a id="wp-085"></a>
### WP-085: Build the codex pages

- **Lane:** ui · **Depends on:** 068 · **Size:** M
- **Read:** [Codex](08-portfolio-integration.md#codex), [SEO and metadata](08-portfolio-integration.md#seo-and-metadata).
- **Touch:** `src/routes/codex/`.
- **Do:** Prerendered codex routes, locked and unlocked states, and metadata.
- **Acceptance:** `bun run build` prerenders every portfolio codex page; pages work with
  JavaScript disabled.
- **Verify:** `bun run build`, screenshots.

<a id="wp-086"></a>
### WP-086: Launch: replace the old game

- **Lane:** ui · **Depends on:** 081, 082, 083, 085 · **Size:** M
- **Read:** [Home page](08-portfolio-integration.md#home-page).
- **Touch:** `src/routes/game/`, `src/routes/+page.svelte`, `src/lib/game/` (delete),
  `CLAUDE.md`, `README.md`.
- **Do:** Make Latticefall the default `/game`; move the home page to the codex store;
  delete `src/lib/game/` and its assets that nothing else uses; update `CLAUDE.md` and the
  README to describe the new game.
- **Acceptance:** No import of `$lib/game` remains; the home page shows relic progress.
- **Verify:** `bun run check`, `bun run build`, screenshots of the home page and `/game`.

<a id="wp-087"></a>
### WP-087: Finish music and audio

- **Lane:** render · **Depends on:** G3 · **Size:** M
- **Read:** [Audio](07-presentation.md#audio).
- **Touch:** `src/lib/latticefall/audio/`, `static/latticefall/audio/`, `game/assets-src/audio/`.
- **Do:** Region loops, boss stems (including the Bellwright's beat), ambience, and
  final effects, with licenses recorded.
- **Acceptance:** Every region and boss has music; credits list every audio source.
- **Verify:** `bun run check`, `lf-tools credits`.

<a id="wp-088"></a>
### WP-088: Add Marks, photo mode, gestures, and the timer

- **Lane:** ui · **Depends on:** G3 · **Size:** M
- **Read:** [Achievements](06-progression-and-systems.md#achievements),
  [gestures and photo mode](06-progression-and-systems.md#gestures-and-photo-mode),
  [timer and new game plus](06-progression-and-systems.md#timer-and-new-game-plus).
- **Touch:** `src/lib/latticefall/ui/`, `game/crates/core/src/world_state/marks.rs`.
- **Do:** All Marks, photo mode, six gestures, and the in-game timer.
- **Acceptance:** Each Mark has a test or a golden replay that earns it.
- **Verify:** `mise run game:test`, `bun run check`.

<a id="wp-089"></a>
### WP-089: Balance and fairness pass

- **Lane:** combat · **Depends on:** G3 · **Size:** L
- **Read:** [Tuning targets](03-combat.md#tuning-targets), [builds](04-arsenal-and-builds.md#builds),
  [fairness rules](05-enemies-and-bosses.md#fairness-rules),
  [expected level curve](06-progression-and-systems.md#expected-level-curve).
- **Touch:** `game/data/**` (numbers only).
- **Do:** A headless balance report (`lf-tools balance`) that runs scripted bots for each
  of the nine builds against each required boss at the expected level, and reports time
  to kill, hits taken, and win rate; then tune data until every build is within the
  tuning targets.
- **Acceptance:** The report shows every build within 30% of the target time to kill for
  every required boss.
- **Verify:** `cargo run -p lf-tools -- balance`.

**Stretch:** Reweave (new game plus). Write its card when M8 starts if there's time.

## M9: RL environment

<a id="wp-090"></a>
### WP-090: Build the full Gymnasium environment and scenarios

- **Lane:** rl · **Depends on:** 039 · **Size:** L
- **Read:** [Scenarios](09-rl-environment.md#scenarios), [Python API](09-rl-environment.md#python-api).
- **Touch:** `game/crates/py/`, `game/data/scenarios/`, `game/python/latticefall/`.
- **Do:** All six scenarios (adding a boss-arena option for each boss as boss WPs land),
  reset options, the info dictionary, and state control
  methods.
- **Acceptance:** `gymnasium.utils.env_checker.check_env` passes for every scenario.
- **Verify:** `mise run game:py`.

<a id="wp-091"></a>
### WP-091: Add the observation encoders

- **Lane:** rl · **Depends on:** 090 · **Size:** L
- **Read:** [Observations](09-rl-environment.md#observations).
- **Touch:** `game/crates/core/src/obs/`.
- **Do:** Every observation key, the `privileged` option, and the ray-cast pixel camera.
- **Acceptance:** Encoders are deterministic (hash tests) and identical in WASM; the
  vector observation adds under 20% to step time.
- **Verify:** `mise run game:test`, `mise run game:wasm`, `mise run game:bench`.

<a id="wp-092"></a>
### WP-092: Add action spaces and masks

- **Lane:** rl · **Depends on:** 090 · **Size:** M
- **Read:** [Actions](09-rl-environment.md#actions).
- **Touch:** `game/crates/core/src/obs/actions.rs`, `game/python/latticefall/spaces.py`.
- **Do:** The `MultiDiscrete`, flat, and continuous variants; masks; and
  `reaction_delay`.
- **Acceptance:** Tests show that masked actions match what the core would accept.
- **Verify:** `mise run game:py`.

<a id="wp-093"></a>
### WP-093: Add rewards and safeguards

- **Lane:** rl · **Depends on:** 090 · **Size:** M
- **Read:** [Rewards](09-rl-environment.md#rewards).
- **Touch:** `game/crates/core/src/obs/rewards.rs`, `game/python/latticefall/rewards.py`.
- **Do:** Every component, weights, and the reachability gate and stall penalty.
- **Acceptance:** A cheese test: a scripted bot standing on an unreachable ledge earns no
  damage reward.
- **Verify:** `mise run game:py`.

<a id="wp-094"></a>
### WP-094: Add randomization and curriculum configs

- **Lane:** rl · **Depends on:** 090 · **Size:** M
- **Read:** [Curriculum and randomization](09-rl-environment.md#curriculum-and-randomization).
- **Touch:** `game/crates/core/src/scenario/randomize.rs`, `game/python/latticefall/`.
- **Do:** Every randomization option and the boss league sampler.
- **Acceptance:** Randomization is reproducible from the seed.
- **Verify:** `mise run game:py`.

<a id="wp-095"></a>
### WP-095: Build the vector environment

- **Lane:** rl · **Depends on:** 091, 092 · **Size:** L
- **Read:** [Vector environment](09-rl-environment.md#vector-environment),
  [benchmarks and evaluation](09-rl-environment.md#benchmarks-and-evaluation).
- **Touch:** `game/crates/py/src/vec.rs`, `game/python/latticefall/vector.py`.
- **Do:** Rayon stepping with the GIL released, zero-copy buffers, autoreset, the
  PufferLib wrapper, and the CI throughput gate.
- **Acceptance:** 5,000 or more steps per second per core for `BossArena-v0` with vector
  observations, scaling to at least 6x on 8 cores.
- **Verify:** `mise run game:bench`, `mise run game:py`.

<a id="wp-096"></a>
### WP-096: Train the PPO baseline

- **Lane:** rl · **Depends on:** 093, 095 · **Size:** L
- **Read:** [Experiment plan](09-rl-environment.md#experiment-plan) (E1, E2).
- **Touch:** `game/python/examples/ppo_boss.py`, `game/python/RESULTS.md`.
- **Do:** A single-file PPO (with an LSTM option), and runs E1 and E2 with results.
- **Acceptance:** Results table with win rates and confidence intervals on the held-out
  seeds.
- **Verify:** The training logs and `RESULTS.md`.

<a id="wp-097"></a>
### WP-097: Build the text environment and MCP server

- **Lane:** rl · **Depends on:** 091 · **Size:** L
- **Read:** [Text environment](09-rl-environment.md#text-environment).
- **Touch:** `game/crates/core/src/macros/`, `game/python/latticefall/{text.py,mcp_server.py}`,
  `game/data/**` (move descriptions for `doing`).
- **Do:** Macro controllers, interrupts, JSON observations, candidate scoring, the MCP
  server, and an example LLM agent loop.
- **Acceptance:** A scripted "LLM" that follows candidates clears a mini-dungeon; an MCP
  client can play through the server.
- **Verify:** `mise run game:py`, the example agent run.

<a id="wp-098"></a>
### WP-098: Build the human demonstration pipeline

- **Lane:** rl · **Depends on:** 084, 091 · **Size:** L
- **Read:** [Human demonstrations](09-rl-environment.md#human-demonstrations).
- **Touch:** `src/lib/latticefall/ui/settings/research/`, `api/src/handlers/latticefall.rs`,
  `api/src/routes/mod.rs`, `game/crates/tools/src/replay_export.rs`.
- **Do:** The consent UI, upload to R2, deletion, and the Minari exporter.
- **Acceptance:** A browser recording exports to a Minari dataset that loads with
  `minari.load_dataset`.
- **Verify:** `cd api && cargo check`, `mise run game:py`.

<a id="wp-099"></a>
### WP-099: Run a policy in the browser

- **Lane:** render · **Depends on:** 096 · **Size:** M
- **Read:** [Policies in the browser](09-rl-environment.md#policies-in-the-browser).
- **Touch:** `src/routes/rl/latticefall/`, `src/lib/latticefall/agent/`.
- **Do:** ONNX export, browser inference, and the `/rl/latticefall` page.
- **Acceptance:** The browser agent's win rate over 20 seeds matches the Python
  evaluation within noise.
- **Verify:** `bun run check`, `bun run build`, screenshots.

<a id="wp-100"></a>
### WP-100: Add OpenEnv and verifiers adapters

- **Lane:** rl · **Depends on:** 097 · **Size:** M
- **Read:** [Text environment: interfaces](09-rl-environment.md#interfaces).
- **Touch:** `game/python/latticefall/adapters/`.
- **Do:** Both adapters, checked against the libraries' current documentation and
  examples at the time of the WP.
- **Acceptance:** Each adapter runs one episode through its library's own example
  client.
- **Verify:** `mise run game:py`.
