# Status

This file tracks every work package. Agents update their row at the end of each
session, as the [agent playbook](11-agent-playbook.md#definition-of-done) requires.

Status values are `todo`, `in progress`, `blocked`, `split`, `review`, and `done`.

## Gates

| Gate | Status | Date | Notes |
|---|---|---|---|
| G1: Rust-core decision (after M1) | todo | | |
| G2: Vertical slice playtest (after M4) | todo | | |
| G3: Content complete (after M7) | todo | | |
| Launch (after M8) | todo | | |

## Work packages

| WP | Title | Lane | Status | Branch | Date | Notes |
|---|---|---|---|---|---|---|
| 001 | Scaffold the game workspace | infra | done | claude/game-m0-1730d5 | 2026-11-12 | Rust pinned to 1.99.1; game:* mise tasks set RUSTUP_TOOLCHAIN because mise's root rust=stable overrides rust-toolchain.toml. Installed toolchain 1.99.1 into ~/.rustup. |
| 002 | Deterministic math, RNG, tick skeleton | core | done | claude/game-m0-1730d5 | 2026-11-12 | nalgebra 0.35 (rapier3d 0.36.1's math crate) with libm-force; own PCG32 with splitmix64-derived streams; state hash = xxh3_64 over postcard, events excluded. clippy.toml in lf-core denies HashMap/HashSet and std transcendental f32 methods. |
| 003 | Content crate | core | done | claude/game-m0-1730d5 | 2026-11-12 | u32 numeric IDs; content hash = xxh3_64 over postcard of parsed Content (efd431704141700b). Curves stored as integer tenths. Added game/data/weapons/straight_sword.ron (wpn_wayfarer_blade) and an optional move 'charge' field. New data files need an include_str! line in lf-content (a test catches misses). |
| 004 | Inputs, replays, golden tests | core | done | claude/game-m0-1730d5 | 2026-11-12 | Replay = postcard .lfr (header, RLE frames, commands, hash every 30 ticks). Golden m0_random_3000.lfr: 3,196 bytes, final hash 81ea2bbe90c69441. Placeholder kinematic player movement (walk 1.6, run 4.5 m/s) until WP-010. lf-tools replay run/rehash/record-random and a minimal bench. |
| 005 | CI and frontend deploy proposal | infra | done | claude/game-m0-1730d5 | 2026-11-12 | game.yml passes actionlint and its steps run locally; not yet run on GitHub (branch not pushed). ADR 0003 is Proposed with a concrete web-deploy.yml; needs owner sign-off and the Pages project name. Installed actionlint to ~/go/bin. |
| 006 | Browser bridge | render | done | claude/game-m0-1730d5 | 2026-11-12 | WASM and native hashes match for the golden replay and content. lf_wasm_bg.wasm is 349,267 bytes raw, 130,806 gzipped. Uses wasm-bindgen 0.2.106 and wasm-opt 126 from the worker-build cache (no installs). WASM output is committed (ADR 0003 fallback) and rebuilds byte-identically. /game?engine=lf shows the new engine; old game moved unchanged to OldGame.svelte. The 60 fps check was not measured: the hidden Browser pane throttles rAF. |
| 007 | Python binding stub | rl | done | claude/game-m0-1730d5 | 2026-11-12 | pyo3 0.30.1 (abi3-py311), maturin 1.16.0 in the project venv game/python/.venv (created with uv; maturin and pytest installed there). Python hash equals native (81ea2bbe90c69441). Bench: step 17-20M steps/s, step_many 157-181M steps/s on the near-empty M0 sim. CI python job installs uv. |
| 008 | Portfolio slugs | ui | done | claude/game-m0-1730d5 | 2026-11-12 | Slugs on all 20 content.ts items; scripts/export-content.ts writes game/data/gen/portfolio_content.json, run by game:validate before lf-tools validate (CI rust job now installs Bun). Portfolio is validated separately and is not in the content hash. Nine of ten relic flavor lines are new wording that needs owner review. |
| 010 | Physics and character controller | core | todo | | | |
| 011 | Input mapping | render | todo | | | |
| 012 | Third-person camera | render | todo | | | |
| 013 | Character rendering and animation | render | todo | | | |
| 014 | Developer tools and dev URLs | render | todo | | | |
| 015 | Lock-on in the core | core | todo | | | |
| 020 | Move system | combat | todo | | | |
| 021 | Hitboxes and hits | combat | todo | | | |
| 022 | Stamina, dodging, guarding | combat | todo | | | |
| 023 | Damage pipeline | combat | todo | | | |
| 024 | Poise, stance, parries, criticals | combat | todo | | | |
| 025 | Status effects and buffs | combat | todo | | | |
| 026 | Inventory, items, flasks | combat | todo | | | |
| 027 | Projectiles | combat | todo | | | |
| 028 | Combat VFX | render | todo | | | |
| 029 | Death, Remnants, Lanterns | core | todo | | | |
| 030 | AI framework | ai | todo | | | |
| 031 | Navigation grid | world | todo | | | |
| 032 | Enemy batch 1 | ai | todo | | | |
| 033 | Boss framework | ai | todo | | | |
| 034 | B01 Hollow Sentry | ai | todo | | | |
| 035 | Enemy rendering | render | todo | | | |
| 036 | Enemy batch 2 | ai | todo | | | |
| 037 | Enemy batch 3 and elites | ai | todo | | | |
| 039 | RL smoke test | rl | todo | | | |
| 040 | Level format and baker | world | todo | | | |
| 041 | Terrain, kits, atmosphere | render | todo | | | |
| 042 | World mechanics | world | todo | | | |
| 043 | Waking Hollow | world | todo | | | |
| 044 | The Verge | world | todo | | | |
| 045 | HUD | ui | todo | | | |
| 046 | Menus | ui | todo | | | |
| 047 | Saving and loading | core | todo | | | |
| 048 | Audio engine | render | todo | | | |
| 049 | Landing card, Ledger, codex store | ui | todo | | | |
| 050 | Leveling and respec | combat | todo | | | |
| 051 | Movesets batch A | combat | todo | | | |
| 052 | Movesets batch B | combat | todo | | | |
| 053 | Weapons, upgrades, tempering | combat | todo | | | |
| 054 | Arts | combat | todo | | | |
| 055 | Spell system and Formulae | combat | todo | | | |
| 056 | Litanies | combat | todo | | | |
| 057 | Charms, armor, consumables | combat | todo | | | |
| 058 | Equipment rendering | render | todo | | | |
| 059 | Echo summons | ai | todo | | | |
| 060 | Mirrormere | world | todo | | | |
| 061 | Sunken Ossuary, Drowned Belfry | world | todo | | | |
| 062 | Hollowdeep, Cinderdeep Mine | world | todo | | | |
| 063 | Mossgut Cave, Glasswing Grotto | world | todo | | | |
| 064 | Castle Graymantle, part 1 | world | todo | | | |
| 065 | Castle Graymantle, part 2 | world | todo | | | |
| 066 | Rimeshelf, Whitewarden, Bound Ring | world | todo | | | |
| 067 | Lattice Spire | world | todo | | | |
| 068 | Hollowhearth, NPCs, questlines | ui | todo | | | |
| 069 | Map and fast travel | ui | todo | | | |
| 070 | B02 the Briar Stag | ai | todo | | | |
| 071 | B03 Bridgewarden Corvane | ai | todo | | | |
| 072 | B04 Mossgut Brood | ai | todo | | | |
| 073 | B05 Ossuary Keeper | ai | todo | | | |
| 074 | B06 the Drowned Choir | ai | todo | | | |
| 075 | B07 Castellan Voss | ai | todo | | | |
| 076 | B08 Forgeheart Colossus | ai | todo | | | |
| 077 | B09 the Lamplight Matriarch | ai | todo | | | |
| 078 | B10 the Bellwright | ai | todo | | | |
| 079 | B11 the Bound Exile | ai | todo | | | |
| 080 | B12 the Unspooled | ai | todo | | | |
| 081 | Performance budgets | render | todo | | | |
| 082 | Accessibility | ui | todo | | | |
| 083 | Mobile view and touch mode | ui | todo | | | |
| 084 | Async features and telemetry | infra | todo | | | |
| 085 | Codex pages | ui | todo | | | |
| 086 | Launch: replace the old game | ui | todo | | | |
| 087 | Music and audio | render | todo | | | |
| 088 | Marks, photo mode, gestures, timer | ui | todo | | | |
| 089 | Balance and fairness pass | combat | todo | | | |
| 090 | Gymnasium environment and scenarios | rl | todo | | | |
| 091 | Observation encoders | rl | todo | | | |
| 092 | Action spaces and masks | rl | todo | | | |
| 093 | Rewards and safeguards | rl | todo | | | |
| 094 | Randomization and curriculum | rl | todo | | | |
| 095 | Vector environment | rl | todo | | | |
| 096 | PPO baseline | rl | todo | | | |
| 097 | Text environment and MCP server | rl | todo | | | |
| 098 | Human demonstration pipeline | rl | todo | | | |
| 099 | Policy in the browser | render | todo | | | |
| 100 | OpenEnv and verifiers adapters | rl | todo | | | |

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
- **WASM output is committed** in `src/lib/latticefall/wasm/` while ADR 0003 is Proposed,
  because the Pages Git integration can't build Rust. Run `mise run game:wasm` and commit
  the result after any core change. CI warns when the committed output is stale.
