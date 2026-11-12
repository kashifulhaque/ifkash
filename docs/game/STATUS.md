# Status

This file tracks every work package. Agents update their row at the end of each
session, as the [agent playbook](11-agent-playbook.md#definition-of-done) requires.

Status values are `todo`, `in progress`, `blocked`, `split`, `review`, and `done`.

The orchestrator owns this file. It follows these rules, from
[the orchestrator section of the playbook](11-agent-playbook.md#run-a-milestone-as-an-orchestrator):

- **Claim before work.** Before handing out a WP, set its status to `in progress` and
  fill in **Session** (the orchestrator's session name) and **Branch**. A row that is
  `in progress` with another session's name belongs to that session; don't touch it.
- **Worker** is the planned model from the roadmap card. **Done by** is the model that
  actually finished it, including any escalation, for example `haiku, then sonnet`.
- **Release on handoff.** A session that hands off before finishing a WP sets it back to
  `todo` (or `blocked`) and explains why in **Notes** and in [`JOURNAL.md`](JOURNAL.md).

## Gates

| Gate | Status | Date | Notes |
|---|---|---|---|
| G1: Rust-core decision (after M1) | done | 2026-11-12 | Owner confirmed the Rust core (ADR 0001) on 2026-11-12; M2 may start. WASM 366,194 B gzipped (936,707 raw; budget 1.5 MB). Native bench: test_field 49,019 ticks/s, m0_flat 482,910. WASM under Node 25: test_field 30,131 ticks/s, m0_flat 563,253 (30 Hz needed). Golden replays match on native, WASM, and Python (Python last run after WP-015). |
| G2: Vertical slice playtest (after M4) | todo | | |
| G3: Content complete (after M7) | todo | | |
| Launch (after M8) | todo | | |

## Work packages

| WP | Title | Lane | Worker | Status | Done by | Session | Branch | Date | Notes |
|---|---|---|---|---|---|---|---|---|---|
| 001 | Scaffold the game workspace | infra | sonnet | done | sonnet, Opus-verified | latticefall-m0 | claude/game-m0-1730d5 | 2026-11-12 | Rust pinned to 1.99.1; game:* mise tasks set RUSTUP_TOOLCHAIN because mise's root rust=stable overrides rust-toolchain.toml. Installed toolchain 1.99.1 into ~/.rustup. |
| 002 | Deterministic math, RNG, tick skeleton | core | sonnet | done | sonnet, Opus-verified | latticefall-m0 | claude/game-m0-1730d5 | 2026-11-12 | nalgebra 0.35 (rapier3d 0.36.1's math crate) with libm-force; own PCG32 with splitmix64-derived streams; state hash = xxh3_64 over postcard, events excluded. clippy.toml in lf-core denies HashMap/HashSet and std transcendental f32 methods. |
| 003 | Content crate | core | sonnet | done | sonnet, Opus-verified | latticefall-m0 | claude/game-m0-1730d5 | 2026-11-12 | u32 numeric IDs; content hash = xxh3_64 over postcard of parsed Content (efd431704141700b). Curves stored as integer tenths. Added game/data/weapons/straight_sword.ron (wpn_wayfarer_blade) and an optional move 'charge' field. New data files need an include_str! line in lf-content (a test catches misses). |
| 004 | Inputs, replays, golden tests | core | sonnet | done | sonnet, Opus-verified | latticefall-m0 | claude/game-m0-1730d5 | 2026-11-12 | Replay = postcard .lfr (header, RLE frames, commands, hash every 30 ticks). Golden m0_random_3000.lfr: 3,196 bytes, final hash 81ea2bbe90c69441. Placeholder kinematic player movement (walk 1.6, run 4.5 m/s) until WP-010. lf-tools replay run/rehash/record-random and a minimal bench. |
| 005 | CI and frontend deploy proposal | infra | sonnet | done | sonnet, Opus-verified | latticefall-m0 | claude/game-m0-1730d5 | 2026-11-12 | game.yml passes actionlint and its steps run locally; not yet run on GitHub (branch not pushed). ADR 0003 accepted 2026-11-12; web-deploy.yml added; deploys to the `ifkash` Pages project. Installed actionlint to ~/go/bin. |
| 006 | Browser bridge | render | sonnet | done | sonnet, Opus-verified | latticefall-m0 | claude/game-m0-1730d5 | 2026-11-12 | WASM and native hashes match for the golden replay and content. lf_wasm_bg.wasm is 349,267 bytes raw, 130,806 gzipped. Uses wasm-bindgen 0.2.106 and wasm-opt 126 from the worker-build cache (no installs). WASM output rebuilds byte-identically; it was committed until ADR 0003 was accepted and is gitignored again. /game?engine=lf shows the new engine; old game moved unchanged to OldGame.svelte. The 60 fps check was not measured: the hidden Browser pane throttles rAF. |
| 007 | Python binding stub | rl | sonnet | done | sonnet, Opus-verified | latticefall-m0 | claude/game-m0-1730d5 | 2026-11-12 | pyo3 0.30.1 (abi3-py311), maturin 1.16.0 in the project venv game/python/.venv (created with uv; maturin and pytest installed there). Python hash equals native (81ea2bbe90c69441). Bench: step 17-20M steps/s, step_many 157-181M steps/s on the near-empty M0 sim. CI python job installs uv. |
| 008 | Portfolio slugs | ui | sonnet | done | sonnet, Opus-verified | latticefall-m0 | claude/game-m0-1730d5 | 2026-11-12 | Slugs on all 20 content.ts items; scripts/export-content.ts writes game/data/gen/portfolio_content.json, run by game:validate before lf-tools validate (CI rust job now installs Bun). Portfolio is validated separately and is not in the content hash. Nine of ten relic flavor lines are new wording that needs owner review. |
| 010 | Physics and character controller | core | opus | done | opus, Opus-verified | latticefall-m1 | game/m1-movement | 2026-11-12 | Rapier 0.36.1 (enhanced-determinism, no SIMD). test_field scenario; controller numbers in game/data/locomotion.ron. 10 m fall = 8% max HP (46 of 570, stub Vitality 10). Golden m1_movement.lfr 45b288f9165bc8b6; m0_random_3000 re-recorded 81ea2bbe90c69441 -> 5ca304e1b5598ad9. Content hash 008ee1f4e5f21bed. Native = WASM = Python. Bench test_field 51,242 ticks/s. WASM 356,445 B gzipped. Actors aren't Rapier bodies yet (no actor-vs-actor collision); PhysicsState is static and not in the state hash. Python bench runs a player that dies off the m0_flat floor, so its steps/s are optimistic. |
| 011 | Input mapping | render | sonnet | done | sonnet, Opus-verified | latticefall-m1 | game/m1-movement | 2026-11-12 | Real-browser check by the orchestrator: a replay recorded in /game?engine=lf&dev=1 (969 ticks, 210 non-idle frames) re-simulated in Python matches the live browser hashes at ticks 922 and 968, and lf-tools replay run passes (final 0e6b8cb8fd3bf7a7). Input was synthetic KeyboardEvents; pointer lock and gamepad weren't exercised. Arrow keys now cycle slots and the keyboard always moves at full tilt (no Shift-to-run). New lf-wasm export encode_replay. |
| 012 | Third-person camera | render | sonnet | done | sonnet, Opus-verified | latticefall-m1b | game/m1-camera | 2026-11-12 | CameraRig in render/camera/ took over WP-011's orbit code in scene.ts. Distance 4.5 m (3-8), pivot 1.6 m, pitch -35 to +60 degrees (spec); FOV 60 and resting pitch 20 are the worker's picks. Collision: 9-ray sphere cast plus a clearance probe through the existing read-only `raycast` export; pulls in instantly, eases out 0.3 s. Orchestrator checks: `__lfCamera.sweep()` of the scripted ledge/ridge/drop orbit (`&cam=orbit-script`) gives min clearance 0.25 m over 4,321 frames; real lock-on on test_field (`&scenario=test_field`, Q) reports locked with player and target on screen. Dev extras: `&fakelock=X,Y,Z`, `&camt=`, and a dev level proxy mesh (no real level renderer yet). Not verified: SWITCH_L/R from the mouse wheel in the browser (core unit tests cover switching). `cam=fixed:` is WP-014's. |
| 013 | Character rendering and animation | render | sonnet | done | sonnet, Opus-verified | latticefall-m1 | game/m1-movement | 2026-11-12 | Wayfarer = Quaternius Adventurer.glb; 13 UAL clips (CC0, itch.io, trimmed source 6.7 MB committed). ADR 0005 adds @gltf-transform/cli. static/latticefall is 844 KB. Orchestrator screenshots of run, sprint, jump, and land via ?anim=STATE&animcam=side|three&animt=T look correct. In review because foot sliding at run speed wasn't seen in live motion (worker measured 5-12% slip). KTX2 not done: no textured assets yet, toktx not installed (no Homebrew formula), transcoder not shipped. Snapshot lacks grounded flag and velocity; renderer infers them. |
| 014 | Developer tools and dev URLs | render | sonnet | done | sonnet, Opus-verified | latticefall-m1c | game/m1-devtools | 2026-11-12 | One parser in `src/lib/latticefall/dev/params.ts` (dev, spawn, cam incl. `fixed:`, overlay, freeze, seed, scenario); legacy `cam`, `camt`, `fakelock`, `scenario` still work without `dev=1`, the rest need it. Dev tools lazy-load only with `dev=1`. Dev commands (teleport to a named spawn, god mode, set-level) are in `lf-wasm` `debug.rs` behind a `dev` cargo feature: build with `LF_WASM_FEATURES=dev bash game/scripts/build-wasm.sh`; the default build has none and is byte-identical in size. No lf-core change; goldens unchanged. Set-level only stores a value (core has no level yet). Lanterns don't exist (WP-029), so teleport uses a TS spawn table. Nav, AI, hitbox and hurtbox overlays toggle but draw nothing yet. A session that used a dev command can't be reproduced from its replay. Orchestrator checks: `bun run check`, cargo test, clippy, validate, golden.mjs all green. Browser overlays were checked by the worker, not re-checked by the orchestrator. |
| 015 | Lock-on in the core | core | haiku | done | sonnet (moved up a tier), Opus-verified | latticefall-m1b | game/m1-camera | 2026-11-12 | Player `lock_target: Option<Id>` (hashed). Numbers in locomotion.ron `lock_on`: 20 m range, 60-degree cone read as the full apex angle (30 degrees each side), break past 30 m, retarget within 8 m, LOS eye height 1.5 m. Acquisition uses the player's facing, not the camera's: the browser doesn't send camera yaw to the core yet. test_field has three still 100 HP dummies. Snapshot flags constants in snapshot.rs; dead and locked target (1<<4) are written. Goldens re-recorded: m0_random_3000 5ca304e1b5598ad9 -> ad69e604b3114afd, m1_movement 45b288f9165bc8b6 -> e87fd375aa351e2b; content hash e7f680315647d63d. Native = WASM = Python. WASM 366,194 B gzipped. |
| 020 | Move system | combat | opus | done | opus, Opus-verified | latticefall-m2 | game/m2-combat | 2026-11-12 | `combat/moves.rs`: `Actor.moves: MoveState` (hashed) with chains, cancel windows, 8-tick buffer (`input_buffer_ticks` in locomotion.ron), root motion, lock-target tracking before the first hitbox frame, flags HYPERARMOR/IFRAMES/CHARGING, stamina stub `try_spend` and FP costs (player FP from `stub_focus` 10). Frame 0 runs on the start tick. DODGE tap (released within `sprint_delay_ticks`) rolls toward the stick or backsteps; held DODGE sprints. Added `ss_roll` (i-frames 2-14) and `ss_backstep_dodge`; `Moveset` gains `dodge_roll`/`dodge_backstep`. Charged heavy: HEAVY held at frame 12 becomes `ss_heavy_charged`. Snapshot fills `move_id`, `move_frame`, INVULNERABLE. Jump/plunge attacks, guard counter, and move events deferred (WP-021/022). Goldens re-recorded: m0_random_3000 ad69e604b3114afd -> eb459616dc13af59, m1_movement e87fd375aa351e2b -> 6c2a537dc205c8b9; content hash 0d2d2d1fdcb0c7e1. Native = WASM = Python. WASM 374,499 B gzipped. Orchestrator regenerated `gen/Moveset.ts`. |
| 021 | Hitboxes and hits | combat | opus | done | opus, Opus-verified | latticefall-m2 | game/m2-combat (worker worktree) | 2026-11-12 | `combat/{hitbox,hurtbox,resolve}.rs`, `Event::Hit` (attacker, target, move_num, hitbox, motion_value scaled by charge, charge_frac, point, dir). Swept capsule tests in substeps (at most half the summed radii, cap 64); one hit per target per move; candidates sorted attacker, hitbox, target and decided from unchanged state by `resolve::hit_outcome` (i-frames -> Ignored, else Connect). The test_field dummies are the training dummies (controller capsule hurtbox). No damage applied yet; WP-023 wires WP-022's `guard::apply_incoming_hit` into `hit_outcome`. No line-of-sight check for hitboxes, no broad phase. lf-wasm encodes Hit events (kind 1, 13 u32). Actor-space x right maps to `(-cos yaw, 0, sin yaw)`. Goldens after rebase onto WP-022: m0_random_3000 e06b6633c0b970e8 -> fc4bd00bfaf37224, m1_movement 9641cae4114bb7ad -> c3f2abfccc4b8cf3; new m2_three_hit_chain c1ff4ac41a102bb4 (exactly three hits). Content hash 7e3f1b8a8e40b936. Native = WASM. WASM 388,016 B gzipped. |
| 022 | Stamina, dodging, guarding | combat | sonnet | done | sonnet, Opus-verified | latticefall-m2 | game/m2-combat | 2026-11-12 | New `combat/{stamina,dodge,guard}.rs`. Roll tiers in `moves/common.ron` (light/medium/heavy rolls, backstep; `ss_roll`/`ss_backstep_dodge` retired, registry ids 2012-2013 kept). Load tier from stub load (weapon 4.0 + stub armor 20.0 of 50.8 = Medium); overloaded blocks dodge and sprint. Regen 40% while guarding, 80% at Heavy. Guard raises after 3 held ticks, 4-tick lowering lock; counter window T+1..T+12 after a block. `guard::apply_incoming_hit(...) -> HitOutcome { Ignored, Connected, Guarded, GuardBroken }` is the hook for WP-021; it never touches HP. Guard break: stamina pays its share of the hit, the rest lands unblocked; 30-tick stagger. Blocks work from any side (spec silent on facing). Guard numbers: 50% phys (spec), 0% elemental and guard_boost 40 (tuning). Goldens re-recorded (new hashed guard field): m0_random_3000 eb459616dc13af59 -> e06b6633c0b970e8, m1_movement 6c2a537dc205c8b9 -> 9641cae4114bb7ad; content hash 7e3f1b8a8e40b936. Native = WASM = Python. WASM 380,020 B gzipped. |
| 023 | Damage pipeline | combat | sonnet | done | sonnet, Opus-verified | latticefall-m2 | game/m2-combat | 2026-11-12 | `combat/damage.rs` (integer AR, requirements and unwieldy 0.6, two-hand 1.5, defense, absorption cap 0.7, weakness, counter 1.15, buff cap 1.5) and `actor/stats.rs`; 21 hand-computed cases. Also wired WP-021 and WP-022 together: `resolve.rs` decides landings from unchanged state, applies them in sorted order through `guard::apply_incoming_hit`, deals HP, sets `dead` after the last landing. Counter hit = target's move has hitboxes and is at or before its last active frame. Hit event + WASM record 13 -> 17 words (damage, result, flags counter/killed, hp). Dummies: flat defense 500 (tuning); a light does 29. Not done: unwieldy +20% recovery not read by moves.rs; TWO_HAND button unwired; player defense flat stub 40 (no spec formula); no weapon elemental AR; non-player attackers deal 1. Goldens: m0_random_3000 -> c2a498ac3ad5a261, m1_movement -> be0833f7569e73fa, m2_three_hit_chain -> 47c5ef999c6c186d; content hash 8ce1b7d5c5085252. Native = WASM = Python. |
| 024 | Poise, stance, parries, criticals | combat | sonnet | in progress |  | latticefall-m2 | game/m2-combat | 2026-11-12 | |
| 025 | Status effects and buffs | combat | sonnet | todo |  |  | | | |
| 026 | Inventory, items, flasks | combat | sonnet | done | sonnet, Opus-verified | latticefall-m2 | game/m2-combat | 2026-11-12 | `items/` (inventory, 16 equipment slots, quick slots, `ItemTable`); commands `Equip`, `Unequip`, `UseItem`, `SplitFlasks`; `flask_drink` move (30 frames, heals and spends the charge on frame 16; a stagger before then: no heal, no charge lost; 40% walk, no sprint or jump). Start split 3 Vigil / 1 Lucid (tuning). Equipped right weapon sets the moveset. consumables.ron has flask rules and 21 items (added `itm_glimmer_vial_large`, `itm_arrows_fire`, `itm_arrows_storm`); registry 3010, 4000-4021. Not done: non-flask item effects (WP-057), world pickups, HUD snapshot record (ActorSnap untouched; WP-045), FlaskDrunk/ItemPickedUp events. `SplitFlasks` not gated on a Lantern: WP-029 must gate it and call `refill_flasks()` on rest. Goldens: m0_random_3000 -> aca8a8a694ae262b, m1_movement -> bd2f62d7deb6f767, m2_three_hit_chain -> e01931055715e740; content hash 248501ccf6df8ff1. Native = WASM = Python. WASM 408,444 B gzipped. |
| 027 | Projectiles | combat | sonnet | in progress |  | latticefall-m2 | game/m2-combat (worker worktree) | 2026-11-12 | |
| 028 | Combat VFX | render | sonnet | in progress |  | latticefall-m2 | game/m2-combat (worker worktree) | 2026-11-12 | |
| 029 | Death, Remnants, Lanterns | core | sonnet | in progress |  | latticefall-m2 | game/m2-combat (worker worktree) | 2026-11-12 | |
| 030 | AI framework | ai | opus | todo |  |  | | | |
| 031 | Navigation grid | world | sonnet | todo |  |  | | | |
| 032 | Enemy batch 1 | ai | sonnet | todo |  |  | | | |
| 033 | Boss framework | ai | sonnet | todo |  |  | | | |
| 034 | B01 Hollow Sentry | ai | haiku | todo |  |  | | | |
| 035 | Enemy rendering | render | sonnet | todo |  |  | | | |
| 036 | Enemy batch 2 | ai | sonnet | todo |  |  | | | |
| 037 | Enemy batch 3 and elites | ai | haiku | todo |  |  | | | |
| 039 | RL smoke test | rl | opus | todo |  |  | | | |
| 040 | Level format and baker | world | opus | todo |  |  | | | |
| 041 | Terrain, kits, atmosphere | render | sonnet | todo |  |  | | | |
| 042 | World mechanics | world | sonnet | todo |  |  | | | |
| 043 | Waking Hollow | world | sonnet | todo |  |  | | | |
| 044 | The Verge | world | sonnet | todo |  |  | | | |
| 045 | HUD | ui | sonnet | todo |  |  | | | |
| 046 | Menus | ui | sonnet | todo |  |  | | | |
| 047 | Saving and loading | core | sonnet | todo |  |  | | | |
| 048 | Audio engine | render | sonnet | todo |  |  | | | |
| 049 | Landing card, Ledger, codex store | ui | sonnet | todo |  |  | | | |
| 050 | Leveling and respec | combat | haiku | todo |  |  | | | |
| 051 | Movesets batch A | combat | haiku | todo |  |  | | | |
| 052 | Movesets batch B | combat | sonnet | todo |  |  | | | |
| 053 | Weapons, upgrades, tempering | combat | haiku | todo |  |  | | | |
| 054 | Arts | combat | haiku | todo |  |  | | | |
| 055 | Spell system and Formulae | combat | sonnet | todo |  |  | | | |
| 056 | Litanies | combat | haiku | todo |  |  | | | |
| 057 | Charms, armor, consumables | combat | sonnet | todo |  |  | | | |
| 058 | Equipment rendering | render | sonnet | todo |  |  | | | |
| 059 | Echo summons | ai | sonnet | todo |  |  | | | |
| 060 | Mirrormere | world | sonnet | todo |  |  | | | |
| 061 | Sunken Ossuary, Drowned Belfry | world | sonnet | todo |  |  | | | |
| 062 | Hollowdeep, Cinderdeep Mine | world | sonnet | todo |  |  | | | |
| 063 | Mossgut Cave, Glasswing Grotto | world | sonnet | todo |  |  | | | |
| 064 | Castle Graymantle, part 1 | world | sonnet | todo |  |  | | | |
| 065 | Castle Graymantle, part 2 | world | sonnet | todo |  |  | | | |
| 066 | Rimeshelf, Whitewarden, Bound Ring | world | sonnet | todo |  |  | | | |
| 067 | Lattice Spire | world | sonnet | todo |  |  | | | |
| 068 | Hollowhearth, NPCs, questlines | ui | sonnet | todo |  |  | | | |
| 069 | Map and fast travel | ui | sonnet | todo |  |  | | | |
| 070 | B02 the Briar Stag | ai | sonnet | todo |  |  | | | |
| 071 | B03 Bridgewarden Corvane | ai | haiku | todo |  |  | | | |
| 072 | B04 Mossgut Brood | ai | sonnet | todo |  |  | | | |
| 073 | B05 Ossuary Keeper | ai | sonnet | todo |  |  | | | |
| 074 | B06 the Drowned Choir | ai | sonnet | todo |  |  | | | |
| 075 | B07 Castellan Voss | ai | sonnet | todo |  |  | | | |
| 076 | B08 Forgeheart Colossus | ai | sonnet | todo |  |  | | | |
| 077 | B09 the Lamplight Matriarch | ai | sonnet | todo |  |  | | | |
| 078 | B10 the Bellwright | ai | sonnet | todo |  |  | | | |
| 079 | B11 the Bound Exile | ai | sonnet | todo |  |  | | | |
| 080 | B12 the Unspooled | ai | opus | todo |  |  | | | |
| 081 | Performance budgets | render | sonnet | todo |  |  | | | |
| 082 | Accessibility | ui | sonnet | todo |  |  | | | |
| 083 | Mobile view and touch mode | ui | sonnet | todo |  |  | | | |
| 084 | Async features and telemetry | infra | sonnet | todo |  |  | | | |
| 085 | Codex pages | ui | haiku | todo |  |  | | | |
| 086 | Launch: replace the old game | ui | sonnet | todo |  |  | | | |
| 087 | Music and audio | render | haiku | todo |  |  | | | |
| 088 | Marks, photo mode, gestures, timer | ui | haiku | todo |  |  | | | |
| 089 | Balance and fairness pass | combat | opus | todo |  |  | | | |
| 090 | Gymnasium environment and scenarios | rl | sonnet | todo |  |  | | | |
| 091 | Observation encoders | rl | opus | todo |  |  | | | |
| 092 | Action spaces and masks | rl | sonnet | todo |  |  | | | |
| 093 | Rewards and safeguards | rl | sonnet | todo |  |  | | | |
| 094 | Randomization and curriculum | rl | haiku | todo |  |  | | | |
| 095 | Vector environment | rl | opus | todo |  |  | | | |
| 096 | PPO baseline | rl | sonnet | todo |  |  | | | |
| 097 | Text environment and MCP server | rl | sonnet | todo |  |  | | | |
| 098 | Human demonstration pipeline | rl | sonnet | todo |  |  | | | |
| 099 | Policy in the browser | render | sonnet | todo |  |  | | | |
| 100 | OpenEnv and verifiers adapters | rl | haiku | todo |  |  | | | |

## Notes

- **M0 branching (2026-11-12):** M0 (WP-001 to WP-008) was built on one branch,
  `claude/game-m0-1730d5`, with one commit series per WP, instead of the playbook's
  branch per WP. The owner reviews the branch as a whole.
- **M0 exit gate (2026-11-12): passed locally.** The golden replay
  `m0_random_3000.lfr` gives final hash `81ea2bbe90c69441` natively (`game:test`), in
  WASM under Node (`game:wasm`), and from Python (`game:py`). Content hash
  `efd431704141700b` matches across all three. The game CI workflow hasn't run on GitHub
  yet, because the branch isn't pushed.
- **M0 measurements (2026-11-12):** `lf_wasm_bg.wasm` is 349,267 bytes raw and 130,806
  bytes gzipped (budget 1.5 MB gzipped). Native bench: about 50 to 100 million ticks per
  second. Python: about 18 to 20 million steps per second with `step` and 157 to 181
  million with `step_many`. These are numbers for a near-empty simulation; they say
  nothing about the G1 budgets yet.
- **Toolchain installed for M0:** Rust 1.99.1 (into `~/.rustup`), a uv venv at
  `game/python/.venv` with maturin 1.16.0 and pytest 9.2.0, and actionlint into
  `~/go/bin`. `game:wasm` uses `wasm-bindgen` 0.2.106 and `wasm-opt` 126 from the
  existing worker-build cache (`~/Library/Caches/worker-build/`) when they aren't on
  `PATH`. CI installs binaryen 126 from its GitHub release.
- **ADR 0003 accepted (2026-11-12).** `.github/workflows/web-deploy.yml` builds the WASM
  module and deploys the site, so `src/lib/latticefall/wasm/` is gitignored again. **Until
  the owner adds the `CLOUDFLARE_PAGES_API_TOKEN` secret and disconnects the Pages Git
  integration (`cf pages projects disconnect-source ifkash`), merging this branch breaks
  Pages deploys**, because the Git integration can't build the WASM
  module.
- **Roadmap fix (2026-11-12):** WP-004 now also depends on WP-003, because the replay
  header stores the content hash. M0 already ran WP-004 after WP-003.
