# Agent playbook

This document tells an AI coding agent (Claude Code, Cursor, Codex, or similar) how to
pick up one work package (WP) from [the roadmap](10-roadmap.md), finish it in one
session, and hand it back in a state the next session can build on.

Humans can follow it too. It's written for agents because agents start every session
with no memory of the last one.

## The one rule

**One session, one work package.** A session reads the WP card, reads only the spec
sections the card lists, does the work, verifies it, updates [`STATUS.md`](STATUS.md),
and stops. If a WP turns out bigger than one session, the agent splits it (see
[Split a work package](#split-a-work-package)) instead of leaving half-finished code
behind.

## Start a session

Paste the following prompt into a fresh session, replacing `WP_ID` with the card's ID,
for example `WP-020`:

```text
You are implementing one work package of Latticefall, the souls-like game in this
repo. Follow docs/game/11-agent-playbook.md exactly.

1. Read docs/game/README.md (glossary and reading order), then docs/game/STATUS.md.
2. Open docs/game/10-roadmap.md and find WP_ID. Confirm that every WP in its
   "Depends on" list is marked done in STATUS.md. If one isn't, stop and tell me.
3. Read only the spec sections the card lists under "Read". Don't read the whole
   docs folder.
4. Plan the change, then implement it. Touch only the paths the card lists under
   "Touch". If you must touch anything else, say why in your report.
5. Run every command under "Verify" and meet every "Acceptance" bullet.
6. Update STATUS.md (status, date, branch, notes) and, if you changed a spec
   decision, add an ADR under docs/game/decisions/.
7. Report: what was asked, what changed, what you verified, what's left.

Shell rule: call /bin/ls instead of ls, and never run eza.
```

For a WP that you want to run with parallel subagents, add this line to the prompt:

```text
You may use subagents for independent parts of this WP, for example one per
enemy or one per data file. Give each subagent the exact file paths it owns.
```

## Branches, commits, and pull requests

- Branch from `main` as `game/WP_ID-SHORT_SLUG`, for example
  `game/wp-020-move-system`.
- Commit in small, buildable steps. Prefix each subject with the WP ID:
  `WP-020: Add hitbox sweep queries`.
- Open one pull request per WP. Its description holds the three-part report from the
  session prompt plus the output of the verify commands.
- Merge only when CI is green. The game lives behind the `?engine=lf` flag
  until [WP-086](10-roadmap.md#wp-086), so partial worlds can merge
  without breaking the site.

## Definition of done

A WP is done when all of the following are true:

- Every acceptance bullet on the card is met and was checked, not assumed.
- `cargo test --workspace` passes in `game/`.
- `cargo clippy --workspace -- -D warnings` passes in `game/`.
- `bun run check` passes at the repo root if any TypeScript or Svelte changed.
- Every golden replay in `game/tests/replays/` still produces its recorded state hash,
  or the WP deliberately changes simulation behavior and re-records the hashes with
  `cargo run -p lf-tools -- replay rehash` and says so in the pull request.
- Content data passes `cargo run -p lf-tools -- validate`.
- If the WP is visible in the browser, the agent took screenshots by using the
  [visual verification protocol](#verify-visually) and attached or described them.
- `STATUS.md` is updated.

## Ground rules for the simulation core

These rules protect determinism. A broken determinism rule is a bug even if every
test passes, because it silently corrupts replays and RL training data. The full
reasoning lives in [the architecture doc](01-architecture.md#determinism-contract).

- No `std::collections::HashMap` or `HashSet` iteration inside `lf-core`. Use
  `BTreeMap`, `IndexMap`, or sorted `Vec`s.
- No wall-clock time, no `Instant`, no `thread_rng`. Time is the tick counter. Randomness
  comes from the world's seeded `Rng` streams only.
- No `f32::sin`, `cos`, `powf`, `exp`, `sqrt` from `std` in gameplay code. Use the
  `lf_core::math::dmath` wrappers, which call `libm`.
- No platform-dependent code paths (`cfg(target_arch)`) inside `lf-core`.
- Gameplay numbers live in `game/data/`, not in Rust constants. A magic number in code
  needs a comment that points to the spec section it comes from.
- The renderer never writes to simulation state. It reads snapshots and events only.

## Ground rules for the browser side

- The renderer lives in `src/lib/latticefall/`. Don't import from `src/lib/game/` (the
  old game); it's deleted in [WP-086](10-roadmap.md#wp-086).
- Use `getApiBase()` from `src/lib/apiBase.ts` for every API call.
- Use the site's design tokens from `src/app.css` for all DOM UI: menus, codex pages,
  and the HUD, which is DOM drawn over the canvas (see
  [the presentation spec](07-presentation.md#hud)).
- Keep Three.js and the WASM module out of the home page bundle. Anything the home page
  needs (progress, codex unlocks) lives in a module with no Three.js or WASM import,
  the same pattern `src/lib/game/journal.ts` uses today.

## Verify visually

Agents can't judge a 3D scene from code. Every WP that changes what the player sees
ends with screenshots:

1. Start the dev server from `.claude/launch.json` (the `web` configuration, port 5180).
2. Open a dev URL that puts the camera where the change is. Until
   [WP-086](10-roadmap.md#wp-086) makes the new game the default, add `&engine=lf` to
   every URL. The dev URL parameters are defined in [WP-014](10-roadmap.md#wp-014):
   - `/game?dev=1&spawn=LOCATION_ID` spawns the player at a named spawn point.
   - `&cam=orbit` or `&cam=fixed:X,Y,Z,YAW,PITCH` controls the camera.
   - `&overlay=hitboxes,nav,ai` turns on debug overlays.
   - `&freeze=1` pauses the simulation so the screenshot is stable.
   - `&seed=N` fixes the world seed.
3. Take at least one screenshot per changed area, in both the default and
   `data-theme="dark"` page themes if DOM UI changed.
4. Describe what each screenshot shows and anything that looks wrong.

## Split a work package

If, after planning, a WP clearly needs more than one session:

1. Stop before writing code.
2. Add child cards under the parent in the roadmap, named `WP_ID.a`, `WP_ID.b`, and so
   on, each with its own Touch, Acceptance, and Verify lists.
3. Mark the parent as `split` in `STATUS.md`.
4. Implement `WP_ID.a` in the same session only if it fits.

## Change a spec decision

The specs are the source of truth, but they're not frozen. When the implementation
shows that a spec is wrong (a number feels bad, an approach doesn't work):

1. Make the smallest change that unblocks the WP.
2. Write an ADR in `docs/game/decisions/NNNN-SLUG.md` with the context, the decision,
   and the consequences. Copy [the template](decisions/0000-template.md).
3. Edit the spec section so it matches, and link the ADR from it.

Tuning numbers (damage, timings, health) don't need an ADR. Change them in
`game/data/` and mention the change in the pull request.

## Run work in parallel

The roadmap groups WPs into lanes. WPs in different lanes touch different paths and
can run in separate sessions at the same time, each in its own git worktree. The lanes
are as follows:

| Lane | Owns | Typical WPs |
|---|---|---|
| `core` | `game/crates/core/src/{world,physics,math,rng,tick}`, `game/crates/{content,tools}/`, `src/lib/latticefall/gen/` | Tick loop, physics, snapshots, content, tools |
| `combat` | `game/crates/core/src/combat/`, `game/data/{moves,weapons,spells}` | Moves, damage, status |
| `ai` | `game/crates/core/src/ai/`, `game/data/{enemies,bosses}` | Enemies, bosses |
| `world` | `game/data/{levels,kits}`, `game/crates/tools/src/bake` | Regions, dungeons |
| `render` | `src/lib/latticefall/{engine,render,input,dev,audio,agent}/`, `static/latticefall/` | Renderer, input, VFX, animation, audio |
| `ui` | `src/lib/latticefall/{ui,codex}/`, `src/routes/{game,codex}/`, `src/lib/content.ts` | HUD, menus, codex, portfolio |
| `rl` | `game/crates/py/`, `game/python/` | Gymnasium env, training |
| `infra` | `.github/workflows/`, `game/Cargo.toml`, `mise.toml`, `api/`, `src/lib/latticefall/online/` | CI, builds, Worker endpoints |

Lanes are a guide. The hard rule is that **two sessions never touch the same files at
once**. Before starting a WP, compare its Touch list with every running WP's. WPs in the
same lane can run together when their Touch lists don't overlap; most combat, enemy, and
boss WPs only add their own data files. The core's `world_state/`, `actor/`, and `items/`
modules are shared by several lanes, so only one running session may edit each. Shared
files, such as
`game/crates/core/src/lib.rs` and `game/data/registry.ron`, take append-only edits;
resolve conflicts by keeping both sides.

## Common mistakes

- Reading every spec before starting. The card lists what matters; the rest wastes
  context.
- Adding gameplay logic to the renderer "just for now." It never moves back, and the RL
  environment can't see it.
- Hand-placing hundreds of props in data files. Use the scatter rules in
  [the world spec](02-world.md#procedural-dressing) instead.
- Importing a new crate or npm package without an ADR. Each one costs bundle size and
  determinism review.
- Copying names, text, or visuals from the games that inspired this one. See
  [the IP rules](00-vision-and-scope.md#ip-rules) for the banned list.
