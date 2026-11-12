# Architecture

This document defines how *Cinderheart* is built: the simulation core, the browser
renderer, the Python bindings, the data pipeline, and the rules that keep all of them
deterministic. Read it before any WP in the `core`, `render`, `rl`, or `infra` lanes.

## Decision summary

| Decision | Choice | ADR |
|---|---|---|
| Simulation language | Rust, one crate with no web or Python dependencies | [0001](decisions/0001-rust-core.md) |
| Physics | `rapier3d` with `enhanced-determinism`, kinematic character controllers only | [0001](decisions/0001-rust-core.md) |
| Browser binding | `wasm-bindgen`, release build optimized with `wasm-opt` | [0001](decisions/0001-rust-core.md) |
| Python binding | PyO3 and maturin, batched stepping with the GIL released | [0001](decisions/0001-rust-core.md) |
| Renderer | Three.js (already in the repo at r182), WebGL 2 renderer | [0002](decisions/0002-renderer.md) |
| Simulation rate | Fixed 30 ticks per second; rendering interpolates | [0001](decisions/0001-rust-core.md) |
| Combat timing | Frame data in content files; skeletons are visual only | [0001](decisions/0001-rust-core.md) |
| Content format | RON files in `game/data/`, parsed only by Rust | [0001](decisions/0001-rust-core.md) |
| Rust-to-TypeScript types | `ts-rs`, generated into `src/lib/latticefall/gen/` | [0001](decisions/0001-rust-core.md) |
| Navigation | Height-aware grid per area with A*; no navmesh | [0001](decisions/0001-rust-core.md) |
| Frontend deploy | GitHub Actions builds WASM and deploys Pages with Wrangler | [0003](decisions/0003-frontend-deploy.md) |

### Why a Rust core

The RL requirement decides this. An RL environment needs thousands of steps per second
per core, many parallel instances, and bit-for-bit reproducibility. A TypeScript
simulation could run in Bun, but Python would reach it over sockets, and per-step
inter-process overhead caps throughput well below that target. Bevy, Godot, Babylon.js,
and PlayCanvas either lack a deterministic headless path or add an engine whose API
churn makes agent-written code go stale.

A plain Rust crate gives one simulation that compiles to WASM for the browser and to a
native Python extension for training. The repo already has a Rust toolchain (the API
Worker), and Rapier is native Rust with a documented cross-platform determinism mode.

**Decision gate.** At the end of [milestone M1](10-roadmap.md#m1-movement-and-camera),
the owner reviews the WASM bundle size, the native and WASM steps per second, and how
much time the toolchain cost. If the cost is far higher than the benefit, fall back to a
TypeScript core run in Bun for RL, and accept lower throughput. Write that up as an ADR
that supersedes ADR 0001.

## System overview

The following diagram shows the three hosts of the one simulation core:

```text
                         game/data/*.ron  (content: moves, weapons, enemies, levels)
                                  |
                                  v
                    +---------------------------+
                    |  lf-core  (pure Rust)     |
                    |  World, tick(), rapier3d, |
                    |  combat, AI, nav, rng     |
                    +---------------------------+
                      |            |           |
          +-----------+     +------+----+      +-----------+
          v                 v           v                  v
    lf-wasm             lf-py        lf-tools           tests
  (wasm-bindgen)      (PyO3)       (CLI: validate,    (golden replays,
          |              |          bake, replay,      unit tests)
          v              v          bench)
  src/lib/latticefall  game/python/latticefall
  Three.js renderer,   Gymnasium envs, vector env,
  input, HUD, audio    text env, dataset tools
          |
          v
  src/routes/game  (SvelteKit page)
```

The renderer reads snapshots and events. It never writes simulation state. Every
player, whether a human at a keyboard or a policy network, produces the same
[`ActionFrame`](#actionframe) each tick.

## Repository layout

The game adds the following paths to the repo. Paths not listed stay as they are.

```text
game/
  Cargo.toml                 # Cargo workspace; separate from api/
  rust-toolchain.toml        # pins the Rust version for determinism
  crates/
    core/                    # crate lf-core: the whole simulation
      src/
        lib.rs
        tick.rs              # system schedule
        world.rs             # World struct, snapshot, clone, hash
        math/                # dmath (libm wrappers), fixed helpers
        rng.rs               # PCG streams
        physics/             # rapier setup, character controller glue
        actor/               # Actor struct, stats, equipment
        combat/              # moves, hitboxes, damage, poise, status
        ai/                  # perception, behaviors, bosses
        nav/                 # nav grid, A*
        items/               # inventory, consumables, pickups
        world_state/         # flags, lanterns, doors, quests, regions
        input.rs             # ActionFrame, Command
        snapshot.rs          # render snapshot layout
        events.rs            # event ring buffer
        obs/                 # RL observation encoders (feature "obs")
        macros/              # macro-actions for the text environment
        scenario/            # scenario loading and randomization
        save.rs              # save blobs
        replay.rs            # replay format
    content/                 # crate lf-content: data types, loader, validation
    wasm/                    # crate lf-wasm: wasm-bindgen facade
    py/                      # crate lf-py: PyO3 module, built by maturin
    tools/                   # crate lf-tools: CLI
  data/
    registry.ron             # every content ID, for stable numeric IDs
    attributes.ron
    moves/                   # one file per moveset
    weapons/
    spells/
    charms.ron
    armor.ron
    consumables.ron
    status.ron
    enemies/
    bosses/
    npcs/
    portfolio.ron            # maps game items to src/lib/content.ts entries
    kits/                    # modular kit catalogs with collider shapes
    levels/                  # one directory per area
    scenarios/               # test, boss-arena, and RL scenarios
    echoes/                  # Echo summons
  assets-src/                # source .glb files with license files, not served
  tests/
    replays/                 # golden replays: seed, inputs, expected hashes
  python/
    latticefall/             # Python package (Gymnasium registration, wrappers)
    pyproject.toml
    examples/                # PPO baseline, LLM harness
src/lib/latticefall/
  engine/                    # WASM loader, fixed-step loop, interpolation
  render/                    # scene, materials, actors, VFX, camera
  input/                     # keyboard, mouse, gamepad, touch -> ActionFrame
  ui/                        # Svelte HUD and menus
  audio/
  dev/                       # dev overlay and dev URL parameters
  online/                    # async features and telemetry client
  agent/                     # in-browser policy inference
  codex/                     # codex unlock store; no Three.js or WASM import
  gen/                       # ts-rs output (generated, committed)
  wasm/                      # wasm-bindgen output (generated, gitignored)
src/routes/game/             # the game page
src/routes/codex/            # plain-HTML codex pages
static/latticefall/          # compressed .glb, textures, audio
```

## The simulation core

### World and actors

`World` holds the entire game state. It derives `Clone`, `serde::Serialize`, and
`serde::Deserialize`. Cloning a `World` is how the RL environment saves and restores
state, and how a search-based agent branches.

Actors (the player, enemies, bosses, and Echoes) share one `Actor` struct stored in a
generational arena. Optional fields hold role-specific state, for example
`ai: Option<AiState>` and `player: Option<PlayerState>`. This is deliberately not an ECS.
A single struct is easier for agents to read and modify, iterates in a deterministic
order, and is fast enough for the 40 or so active actors the game ever has at once.

The following code sample shows the shape of the core types:

```rust
pub struct World {
    pub tick: u64,
    pub rng: RngStreams,          // separate PCG streams: ai, loot, combat, world
    pub content: Arc<Content>,    // immutable, shared across cloned worlds
    pub physics: PhysicsState,    // rapier sets, query pipeline
    pub actors: Arena<Actor>,
    pub projectiles: Arena<Projectile>,
    pub pickups: Arena<Pickup>,
    pub area: AreaState,          // loaded area, active cells, doors, levers
    pub flags: WorldFlags,        // bosses killed, shortcuts, quests, codex
    pub events: EventQueue,       // drained by the host every tick
}

pub fn tick(world: &mut World, input: &TickInput);
```

### Tick schedule

`tick()` runs the following systems in this exact order. Changing the order is a
determinism-breaking change and needs re-recorded golden replays.

1. **Apply commands.** Equip, level up, rest, travel, and menu actions from
   `TickInput.commands`.
2. **Player intent.** Convert the `ActionFrame` into movement and action requests,
   using the input buffer described in [the combat spec](03-combat.md#input-buffer).
3. **AI think.** Perception and decision for every awake AI actor, in arena index order.
   Thinking runs at 10 Hz (every third tick), staggered by actor index.
4. **Move state machines.** Advance every actor's current move by one frame; start
   queued moves; apply cancels.
5. **Locomotion.** Character controller movement through Rapier, gravity, and root
   motion from move data.
6. **Projectiles.** Move projectiles and sweep them against hurtboxes and level geometry.
7. **Hit resolution.** Test active hitboxes against hurtboxes, then apply damage, poise,
   status buildup, guard, parry, and i-frames, sorted by attacker index then hitbox index.
8. **Status and regeneration.** Status timers, stamina and FP regeneration, buffs.
9. **Deaths and drops.** Death handling, Glimmer and loot drops, boss phase changes.
10. **World logic.** Triggers, doors, lifts, levers, fog gates, region and cell
    activation, and Lantern interactions.
11. **Snapshot.** Write the render snapshot and increment `tick`.

### Activity cells

The world is divided into 32-meter cells. Actors in cells more than 2 cells away from
the player are dormant: they don't think, move, or regenerate. Activation depends only
on the player's cell, so it's deterministic. This keeps the per-tick cost flat no matter
how much of the world exists.

### Physics

- The level is static colliders: one heightfield per outdoor area and box, capsule, and
  convex-hull colliders for kit pieces. Trimesh colliders are allowed only for terrain
  features a heightfield can't express.
- Actors are kinematic bodies moved by Rapier's `KinematicCharacterController`, with
  `snap_to_ground` on and `autostep` set for stairs. Gravity is applied by the core.
- Rapier never resolves combat. Hitboxes are tested with simple shape tests in
  `combat/hitbox.rs`, using the Rapier query pipeline only for broad-phase culling and
  line-of-sight rays.
- Projectiles are shape casts, not rigid bodies.
- There are no dynamic rigid bodies in gameplay. Breakable props and ragdolls are
  render-only effects.

### Navigation

Each area has a navigation grid baked by `lf-tools bake`: 0.5-meter cells, each storing
a walkable flag, floor height, and a cost. Multi-level spaces (castle floors, cave
ledges) use stacked layers linked by ladder and stair edges. Enemies path with A* on the
grid, limited to 512 node expansions per request and at most 4 requests per tick. Bosses
don't path; their arenas are open, and they steer directly.

## Determinism contract

Every rule in this section exists because breaking it makes the same seed and inputs
produce different results on different machines. That silently corrupts replays, golden
tests, and RL datasets.

- **Floating point.** Gameplay uses `f32`. Transcendental functions (`sin`, `cos`,
  `atan2`, `exp`, `powf`, `sqrt`) go through `lf_core::math::dmath`, which wraps `libm`.
  Rapier is compiled with `enhanced-determinism`, which is incompatible with its `simd`
  features; don't enable them.
- **No fused multiply-add by accident.** Don't call `mul_add`. Build with the default
  target features; don't add `-C target-cpu=native` to the core.
- **Iteration order.** Never iterate a `HashMap` or `HashSet` in the core. Use arenas,
  `Vec`, `BTreeMap`, or `IndexMap`.
- **Randomness.** All randomness comes from `World.rng`, which holds independent PCG32
  streams seeded from the episode seed: `ai`, `combat`, `loot`, and `world`. Adding a
  random call in one stream must not shift another.
- **Time.** The only clock is `World.tick`. No `Instant`, no `SystemTime`.
- **Threads.** The core is single-threaded. Parallelism happens across worlds (many envs
  on a rayon pool), never inside one.
- **Versions.** `rust-toolchain.toml` pins Rust, and `Cargo.lock` pins Rapier. A Rapier
  upgrade is a determinism-breaking change.
- **Content.** The content hash (a hash of the simulation data in `game/data/` after
  parsing; portfolio files are excluded, see [ADR 0004](decisions/0004-portfolio-outside-content-hash.md)) is part
  of every replay header. A replay only replays against the same content hash.

### State hash and golden replays

`World::state_hash()` returns a 64-bit hash of a canonical serialization of the world,
excluding the event queue. `game/tests/replays/` holds golden replays: a seed, a content
hash, an input log, and the expected hash every 30 ticks. CI runs every golden replay
natively and under WASM (with `wasm-bindgen-test` in Node), and fails on any mismatch.

## Inputs

### ActionFrame

One `ActionFrame` per tick is the entire per-tick input of one player. It's 4 bytes, so
replays are tiny.

| Field | Type | Meaning |
|---|---|---|
| `move_x`, `move_z` | `i8` each | Movement in world space, -127 to 127. Magnitude above 90 is a run; below 40 is a walk. |
| `buttons` | `u16` bitmask | See the following list |

The following list defines the `buttons` bits:

- `LIGHT`, `HEAVY`: Right-hand attacks. Holding `HEAVY` charges.
- `GUARD`: Hold to guard with the left hand, or to parry if the left hand holds a
  parry tool and `HEAVY` is not held.
- `DODGE`: Tap to roll toward the move direction, or backstep with no direction; hold
  to sprint.
- `ART`: Use the equipped weapon's Art.
- `JUMP`
- `USE_ITEM`: Use the selected quick item.
- `INTERACT`: Talk, open, pick up, rest.
- `LOCK`: Toggle lock-on.
- `SWITCH_L`, `SWITCH_R`: Switch the lock-on target.
- `CYCLE_ITEM`, `CYCLE_SPELL`, `CYCLE_RIGHT`, `CYCLE_LEFT`: Cycle quick slots.
- `TWO_HAND`: Toggle two-handing the right weapon.

Spells use the attack buttons; see [spells](03-combat.md#spells). Gestures are
`Command` values, not buttons.

The browser converts camera-relative stick input to world space before it builds the
frame. RL wrappers convert their actions to world space relative to the agent's facing
or lock-on target. Lock-on target selection happens in the core, so the camera follows
the core's choice rather than making its own.

### Commands

Menu actions that change state (equip, level up, upgrade, rest, travel, buy, and so on)
are `Command` values in `TickInput.commands`. The browser's menus emit commands; they
never mutate state directly. Commands are recorded in replays alongside action frames.

## The browser host

### Loop

The page runs a fixed-step loop as described in "Fix Your Timestep" by Glenn Fiedler: an
accumulator advances the core in 33.3-millisecond ticks, with at most 4 ticks per frame,
and the renderer interpolates between the previous and current snapshot by
`alpha = accumulator / dt`. Pausing stops ticking. The core runs on the main thread;
move it to a Web Worker only if profiling shows a need.

### Snapshot layout

After each tick, the core writes a flat, `#[repr(C)]` array of `ActorSnap` records into
WASM memory. The page reads it without copying through a `DataView`, because the record mixes integer
and float fields (all 4-byte aligned). The view
is recreated whenever `memory.buffer` changes, because WASM memory growth detaches old
views.

| Field | Type | Notes |
|---|---|---|
| `id` | `u32` | Arena index and generation |
| `kind` | `u16` | Content ID of the actor type, for model lookup |
| `flags` | `u16` | Dead, staggered, guarding, invulnerable, locked target, two-handing |
| `pos` | `f32 x 3` | Meters |
| `yaw` | `f32` | Radians |
| `move_id` | `u16` | Current move, for animation lookup |
| `move_frame` | `u16` | Frame within the move |
| `hp_frac`, `poise_frac` | `f32` each | For health bars |
| `status_mask` | `u32` | Active status effects, for VFX |

Player-only state (stamina, FP, flasks, equipment, Glimmer) comes from a separate
`PlayerSnap` record. Low-frequency data (inventory lists, item text, codex entries)
comes through `serde-wasm-bindgen` calls, never per frame.

### Events

The core pushes events (hits, deaths, parries, item pickups, boss phase changes, sounds,
UI prompts, and codex unlocks) into a ring buffer. The page drains it every frame and
triggers VFX, audio, camera shake, and UI. Events never feed back into the simulation.

### Animation

Character models are glTF with a shared humanoid skeleton. The renderer plays the
animation clip mapped to the actor's `move_id` and scrubs it to
`move_frame / total_frames`. Locomotion clips blend by speed. Animation is purely
visual; if a clip and the move's frame data disagree, the frame data wins and the clip
is retimed.

### Bundle and loading

- The `/game` route lazy-loads the WASM module and Three.js after the landing card is
  on screen. The home page never imports either.
- Assets stream per region. Each region's assets are listed in a generated manifest.
- Budgets: WASM under 1.5 MB gzipped, initial JavaScript under 400 KB gzipped, first
  region under 8 MB, and the whole game under 60 MB.

## The Python host

`lf-py` is a PyO3 extension module built by maturin. It exposes the following classes:

- `Sim`: one world with `reset(seed, scenario)`, `step(action_frame)`, `clone()`,
  `state_hash()`, `save()`, and `load()`.
- `VecSim`: N worlds stepped in parallel on a rayon pool, with the GIL released, writing
  observations, rewards, and flags into caller-owned NumPy buffers.

The Python package `latticefall` wraps these in Gymnasium environments, wrappers, and
the text interface. The full design is in [the RL environment spec](09-rl-environment.md).

## Content pipeline

1. Designers and agents edit RON files in `game/data/`.
2. `lf-content` parses them into typed structs and validates them: every referenced ID
   exists, numbers are in range, frame windows don't overlap illegally, and every
   portfolio item maps to a real entry in `src/lib/content.ts`.
3. `registry.ron` assigns each string ID a stable numeric ID. IDs are append-only, so
   old replays and saved games keep working.
4. The core embeds content at build time with `include_str!` for the WASM build, and
   loads it from disk for the Python build so researchers can edit scenarios without
   recompiling.
5. `lf-tools gen-ts` writes TypeScript types (`ts-rs`) and a content JSON digest for the
   UI (names, descriptions, icons) into `src/lib/latticefall/gen/`.

### Levels

Levels are data, not Blender scenes. This lets an agent with no art tools build them.
Each area directory in `game/data/levels/` holds:

- `area.ron`: ID, seed, bounds (the activity cells follow from them), kill height,
  ambient settings, music, named region polygons, and connections.
- `terrain.ron`: a heightfield over the bounds: a vertex spacing, seeded noise, then
  sculpt operations applied in order (`SetRect`, `Ramp`, `RaiseCircle`, `FlattenPath`,
  `CarveRiver`, and `CliffLine`). Indoor areas omit it.
- `layout.ron`: modular kit pieces placed on a 2-meter grid with a quarter-turn
  rotation, plus freely placed, scalable landmarks.
- `entities.ron`: the player spawn and a list of entities: Lanterns, enemies, bosses
  with their fog gates, dummies, water, and reeds. Doors, levers, lifts, items, and
  messages join this list as new entity kinds. An Echo Stone is
  `EchoStone(id:, pos:, yaw:)`: a boss arena's region WP places one inside its fog gate
  (WP-060 added it).
- `dressing.ron`: scatter rules (Poisson-disk sampled props by tag and density, masked
  by region, terrain slope, and paths) and a few hand-placed props. The renderer
  applies these; the core ignores them unless a rule marks props as colliders.

The format is documented in `game/crates/content/src/level.rs`. The content loader
compiles every level into the scenario the core runs, so the core finds `test_field` the
same way it finds a file in `scenarios/`. Kit colliders become axis-aligned boxes, which
is why pieces turn only in quarter turns; a capsule becomes its bounding box until the
core supports more shapes.

`lf-tools bake AREA` runs the same compiler, then writes the collision set and the
navigation grid to `game/target/levels/AREA/` for tools, and the render manifest to
`static/latticefall/levels/AREA.json` for the renderer. It warns about unreachable
Lanterns, spawns off the navigation grid, spawns inside colliders, and spawns outside the
bounds; `--deny-warnings` turns warnings into a failure. The kit catalogs in
`game/data/kits/` give each modular piece its glTF path and its simplified collider
shapes, so the core never loads a glTF file.

## Persistence

- **Local saves.** A save is a versioned `postcard` blob of the persistent parts of the
  world: character, inventory, flags, and the last Lantern. The browser keeps three
  slots in IndexedDB.
- **Cloud saves.** Optional, for signed-in players. The existing Google ID token flow
  (`handlers/google_auth.rs`, `game_users`) authorizes `PUT /api/latticefall/save`,
  which stores the blob in D1.
- **Codex unlocks.** Stored separately in `localStorage` under `latticefall-codex` by
  `src/lib/latticefall/codex/`, which has no Three.js or WASM import, so the home page
  and the codex pages can read it.
- **Replays.** A replay is a header (format version, content hash, seed, scenario,
  start state) plus run-length-encoded action frames and commands. One hour of play is
  about 50 KB compressed.

## Build, test, and deploy

The following commands are the gates for every WP. [WP-001](10-roadmap.md#wp-001)
adds them as `mise` tasks.

```sh
cd game && cargo test --workspace            # unit tests and golden replays
cd game && cargo clippy --workspace -- -D warnings
cd game && cargo run -p lf-tools -- validate  # content validation
cd game && cargo run -p lf-tools -- bench     # steps per second, native
mise run game:wasm                            # builds lf-wasm into src/lib/latticefall/wasm
bun run check                                 # frontend typecheck
cd game/python && maturin develop && pytest   # Python bindings
```

Cloudflare Pages' Git integration can't build Rust without extra setup. Per
[ADR 0003](decisions/0003-frontend-deploy.md) (accepted), `.github/workflows/web-deploy.yml`
builds the WASM module and the site, then deploys with `wrangler pages deploy`, replacing
the Git integration for the frontend.

## Performance budgets

| Budget | Target | Checked by |
|---|---|---|
| Core tick, browser, busiest scene | Under 2 ms | Dev overlay timer |
| Core tick, native, boss arena | Under 0.15 ms (over 6,000 ticks per second) | `lf-tools bench` |
| Frame time, 2021 integrated-graphics laptop, 1080p | Under 16.7 ms | Manual check per region |
| Draw calls per frame | Under 250 | Dev overlay |
| Triangles per frame | Under 600,000 | Dev overlay |
| Active actors | Under 40 | Activity cells |
