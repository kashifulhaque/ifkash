# Research notes

This document records what the planning research found, so later sessions can see why
the specs say what they say. Five research passes ran in parallel on 2026-11-11: world
design, combat systems, browser technology, RL environments, and portfolio UX. Claims
marked **unverified** came from the researchers' background knowledge or from sources
they couldn't open; check them before relying on them.

## World design

**Findings that shaped [the world spec](02-world.md):**

- Mini-dungeons in the genre reuse one or two mechanics each (lever gates, sealed doors,
  hollow walls). A mechanic learned once becomes a clue later.
- The genre's director has described its guidance light as minimal guidance, leaving
  discovery to players who go off it. Guidance should be a light, not a quest marker.
- Bosses were placed early in that game's open-world design and became the map's
  skeleton. The plan does the same: the boss roster and the gates came first.
- Small worlds feel large through loops (shortcuts back to hubs), vertical stacking,
  revisiting with new abilities, and one secret per region.
- Reviews of small souls-likes criticize filler: empty spaces and quantity over quality.
  Every room must earn its place.
- Phases should add behavior, not only damage. Telegraphs combine gesture, color, and
  sound. Delays belong on later hits of a combo.

**Recommended targets** for a 1.5-3 hour game: 4-5 regions, 3 major dungeons, 6-8
mini-dungeons, 12-15 bosses with 4-6 required, 20-30 checkpoints, and 2-3 hard gates.
The plan uses 5 regions, 2 legacy dungeons, 7 mini-dungeons, 12 bosses (4 required),
24 Lanterns, and 3 hard gates.

**Unverified:** checkpoint and dungeon counts in the source game (sources conflict),
heavy-attack readability windows (0.6-1.2 s), aggro and leash distances.

**Sources:**

- [PlayStation Blog interview with the director](https://blog.playstation.com/2022/01/28/an-interview-with-fromsoftwares-hidetaka-miyazki/)
- [Game Developer: designing for difficulty and readability in ARPGs](https://www.gamedeveloper.com/game-platforms/designing-for-difficulty-readability-in-arpgs)
- [Superjump: getting lost by design in Hollow Knight](https://www.superjumpmagazine.com/getting-lost-by-design-in-hollow-knight/)
- [80.lv: Tunic's in-game manual](https://80.lv/articles/tunic-s-developer-on-creating-the-in-game-manual-full-of-mysteries)

## Combat systems

**Findings that shaped [the combat spec](03-combat.md) and [the arsenal](04-arsenal-and-builds.md):**

- Roll invincibility in the genre's reference game is about 13-14 frames at 30 fps for
  light and medium loads, slightly fewer at heavy load, with load thresholds at 30%
  and 70% of the maximum. The plan uses 13 ticks at 30 ticks per second.
- A parry costs a small fixed stamina amount; early or late parries partly block.
- Status effects map to resistances in pairs (poison-type to one stat, bleed and
  frost-type to another, sleep and madness-type to a third). The plan keeps that
  three-resistance structure with original names.
- Scaling letters name ranges, not fixed multipliers. The plan stores numeric scaling
  and derives the letter for display.
- Soft caps in the reference game differ by stat and sources conflict; the plan uses
  its own piecewise curve with breakpoints at 20 and 40.
- The reference game's level cost formula was verified against published values, but
  the plan uses its own, simpler curve.
- For RL, keep the action set fixed across weapon classes.

**Unverified:** exact parry windows, status buildup numbers, the two-handing multiplier,
guard stamina formulas. All plan numbers are original starting values to tune.

**Sources:**

- [Invincibility frames (wiki.gg)](https://eldenring.wiki.gg/wiki/Invincibility_Frames)
- [Status effects (wiki.gg)](https://eldenring.wiki.gg/wiki/Status_Effects)
- [Equip load (Fextralife)](https://eldenring.wiki.fextralife.com/Equip+Load)
- [Level (wiki.gg)](https://eldenring.wiki.gg/wiki/Level)

## Browser technology

**Findings that shaped [the architecture](01-architecture.md):**

- A Rust core with Rapier, WASM for the browser, and PyO3 for Python is the only
  compared option that gives both high RL throughput and one shared deterministic core.
  Fallback: a TypeScript core in Bun, with lower RL throughput.
- Rapier is locally deterministic by default. Cross-platform determinism needs the
  `enhanced-determinism` feature (incompatible with `simd8`), IEEE 754 targets (WASM
  qualifies), and trigonometry from deterministic implementations rather than the
  standard library's float methods. The `parallel` feature is compatible.
- Rapier's `KinematicCharacterController` handles translation with slopes, step-up,
  and ground snapping; gravity is the caller's job.
- WASM memory growth detaches `ArrayBuffer` views, so snapshot views must be recreated.
- "Fix Your Timestep" by Glenn Fiedler: fixed `dt`, an accumulator, interpolation by
  `alpha`, and a frame-time clamp to avoid a spiral of death.
- Verified CC0 asset sources: the Quaternius Universal Animation Library and KayKit's
  Skeletons pack. Kenney and other Quaternius and KayKit packs are CC0 per their
  listings but each pack page should be checked. Poly Pizza is per-model. Mixamo terms
  couldn't be verified, so the plan doesn't use it.
- `gltf-transform` supports meshopt, Draco, KTX2, instancing, and pruning.
- Three.js has moved past r182 (r185-r187 seen). Read the migration guide before
  upgrading.
- `ts-rs` generates TypeScript from Rust types; 64-bit integers map to `bigint` by
  default, so avoid them in shared types.

**Unverified:** WASM bundle size and native-versus-WASM speed for this design (measure at
gate G1), Bevy's current status, `recast-navigation-js`, Cloudflare Pages header support
for cross-origin isolation.

**Sources:**

- [Rapier determinism](https://rapier.rs/docs/user_guides/rust/determinism)
- [Rapier character controller](https://rapier.rs/docs/user_guides/rust/character_controller)
- [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/)
- [Quaternius Universal Animation Library](https://quaternius.com/packs/universalanimationlibrary.html)
- [KayKit Skeletons](https://kaylousberg.itch.io/kaykit-skeletons)
- [gltf-transform CLI](https://gltf-transform.dev/cli)
- [ts-rs](https://github.com/Aleph-Alpha/ts-rs)
- [Three.js migration guide](https://github.com/mrdoob/three.js/wiki/Migration-Guide)

## RL environments

**Findings that shaped [the RL spec](09-rl-environment.md):**

- The only souls-like RL prior art found, SoulsGym, wraps a commercial game's memory,
  supports one boss, uses ground-truth state vectors, and reports that boss fights
  violate the Markov property. Hence recurrent policies and the `privileged` option.
- Throughput is decided by the simulation, not the learner. Craftax's JAX rewrite was
  roughly 250x faster than its Python original; EnvPool's C++ thread pool reaches very
  high step rates. The plan copies the EnvPool and PufferLib pattern: batched native
  stepping into preallocated buffers.
- Gymnasium's `terminated` versus `truncated` distinction matters for value
  bootstrapping.
- LLM game benchmarks (BALROG) show a "knowing-doing gap": models know rules but act
  poorly, especially over long horizons. Voyager's skill library of reusable macros was
  the key pattern for an LLM agent. Hence macro-actions and turn-based stepping.
- Multi-turn RL for LLM agents: research reports that naive dense per-turn rewards can
  hurt. Hence terminal or calibrated rewards in `Text-v0`.
- Liquid AI published open-weight d1 decision models (`d1-3B`, `d1-omni-600M`) that
  answer in one forward pass. License terms, training data, and input format for
  action choice are unverified. Hence the candidate-scoring interface.
- OpenEnv (Meta PyTorch and Hugging Face) exposes `reset`, `step`, and `state` over HTTP
  and is marked experimental. Prime Intellect's verifiers use a `load_environment()`
  entry point.
- Minari stores offline RL datasets in HDF5 per episode and is the default export
  format.

**Unverified:** Liquid d1 license and capabilities, OpenEnv's wire protocol, PufferLib's
current throughput figures, VideoGameBench results.

**Sources:**

- [SoulsGym](https://github.com/amacati/SoulsGym)
- [Craftax](https://arxiv.org/abs/2402.16801)
- [EnvPool](https://pypi.org/project/envpool/)
- [BALROG](https://arxiv.org/abs/2411.13543)
- [Voyager](https://arxiv.org/abs/2305.16291)
- [Gymnasium Env API](https://gymnasium.farama.org/main/api/env/)
- [Terminated and truncated](https://farama.org/Gymnasium-Terminated-Truncated-Step-API)
- [Minari dataset standards](https://minari.farama.org/main/content/dataset_standards)
- [OpenEnv](https://github.com/meta-pytorch/openenv)
- [Verifiers](https://github.com/PrimeIntellect-ai/verifiers)
- [Liquid AI: Open d1](https://www.liquid.ai/blog/d1-open)
- [Multi-turn RL with iterative reward calibration](https://arxiv.org/html/2604.02869)

## Portfolio UX and extra features

**Findings that shaped [the portfolio spec](08-portfolio-integration.md) and
[the systems spec](06-progression-and-systems.md):**

- Successful game portfolios (Bruno Simon's driving portfolio, Robby Leonardi's
  platformer résumé) work because content is laid out as a world. The common failure is
  locked content and long loads. Hence the landing card, the three-second rule, and
  the plain-site escape hatch.
- Canvas text is invisible to search engines and screen readers. Hence DOM HUD and menus,
  and prerendered codex pages.
- Assist modes in Celeste and Another Crab's Treasure are granular and never lock
  content. Hence Wayfarer mode and individual difficulty settings.
- Xbox accessibility guidelines ask that players can pause; for a single-player game,
  pausing costs little. Hence pause everywhere.
- Asynchronous messages, death replays, and ghosts give the genre's sense of a shared
  world without netcode, and fit a serverless backend. Template-only messages avoid
  moderation.
- Cloudflare Web Analytics is cookieless; game telemetry should be anonymous, batched,
  and opt-in, with a retention window.
- Mechanics are generally not protected; names, art, music, text, and distinctive UI
  can be. Hence [the IP rules](00-vision-and-scope.md#ip-rules). This isn't legal
  advice; the owner runs a trademark search before launch.

**Ideas from this pass the plan adopted** (things not in the original request):

- Template-only player messages, death Afterimages, and Wanderer ghosts.
- A codex with stable URLs that doubles as plain portfolio pages.
- Speedrun timer, Marks (achievements), photo mode with a site watermark, gestures.
- Wayfarer mode, boss skip after repeated deaths, one-button-attack mode.
- Recording human play as tiny input logs for RL datasets.
- A trained policy playing in the browser on `/rl/latticefall`.
- The Overfit Warden: a boss whose second phase counters the player's habits, which
  also makes a small non-stationary-opponent RL study.

**Sources:**

- [Celeste assist mode](https://gameaccessibilityguidelines.com/celeste-assist-mode)
- [Game Accessibility Guidelines: basic tier](https://gameaccessibilityguidelines.com/basic/)
- [Xbox Accessibility Guideline 117](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/117)
- [Bruno Simon's portfolio case study](https://medium.com/@bruno_simon/bruno-simon-portfolio-case-study-960402cc259b)
- [Cloudflare Web Analytics](https://developers.cloudflare.com/analytics/web-analytics)

## Open questions

- How large is the WASM module with Rapier and the full core, gzipped? (Gate G1.)
- What's the real native step rate for a boss arena with vector observations? (WP-039,
  WP-095.)
- Do the CC0 packs cover enough enemy and armor variety, or do more creatures need
  procedural models? (WP-035, WP-058.)
- Can the existing D1 free tier absorb async-feature writes at expected traffic?
  (WP-084.)
