# Agent playbook

This document tells AI coding agents (Claude Code, Cursor, Codex, or similar) how to
build *Cinderheart* from [the roadmap](10-roadmap.md). The default way to work is an
**orchestrator** session that picks ready work packages (WPs), hands each one to a
**worker** subagent, verifies the result, and records it, until it reaches its context
budget and hands off to the next session.

Humans can follow it too. It's written for agents because agents start every session
with no memory of the last one.

## The one rule

**One worker, one work package.** A worker reads the WP card, reads only the spec
sections the card lists, does the work, and reports. The orchestrator verifies it and
updates [`STATUS.md`](STATUS.md) and [`JOURNAL.md`](JOURNAL.md). If a WP turns out
too big for one worker, the orchestrator splits it (see
[split a work package](#split-a-work-package)) instead of leaving half-finished code
behind.

## Start an orchestrator session

This is the normal way to start work. Paste the following prompt into a new Claude Code
session, using Opus as the model. Replace `TARGET` with a milestone (for example `M1`),
a list of WPs, or `next` to let the orchestrator choose.

```text
You are the orchestrator for Cinderheart, the game in this repo. Work on: TARGET.
Follow docs/game/11-agent-playbook.md, section "Run a milestone as an orchestrator",
exactly, including the context budget.

Start by reading, in this order: docs/game/README.md, docs/game/11-agent-playbook.md,
docs/game/JOURNAL.md (the latest entry first), and docs/game/STATUS.md. Then read the
roadmap cards for the WPs you'll run, and nothing else until a WP needs it.

Shell rule (pass it to every worker): call /bin/ls instead of ls, and never run eza.
Process rule (pass it to every worker): give long commands a timeout, and stop every
process you started (dev servers, scripts, watchers) before you report.
```

## Run a milestone as an orchestrator

The orchestrator is an Opus session. It plans, delegates, verifies, and records; it
writes code itself only for small fixes found during verification.

### Start up

1. Read the latest [`JOURNAL.md`](JOURNAL.md) entry. Do its **Next steps** first, and
   respect anything it says is waiting on the owner.
2. Check the git state. Fetch, then start a session branch from `origin/main`. If the
   journal says an earlier session left work that isn't on `main`, stop and tell the
   owner.
3. Pick the WPs to run: every WP in the target whose **Depends on** WPs are all `done`
   in `STATUS.md` (or merged on your branch) and that isn't `in progress` for another
   session. Order them by the waves in [the roadmap](10-roadmap.md#critical-path-and-parallelism).
4. Add a journal entry with the session's name and start time, and claim the first WPs
   in `STATUS.md` (see the rules at the top of that file).

### Loop

For each ready WP:

1. **Delegate.** Start a worker subagent with the card's **Worker** model (see
   [choose a worker](#choose-a-worker)). Its brief contains: the card text, the spec
   sections the card lists, the Touch paths, the [definition of done](#definition-of-done),
   the [ground rules](#ground-rules-for-the-simulation-core) that apply, and the shell
   rule. Tell it to touch nothing outside its Touch list and to report what it changed,
   what it ran, and anything it couldn't do.
2. **Run in parallel** only when Touch lists don't overlap. Use worktree isolation for a
   worker whose Touch list overlaps a running one, or wait.
3. **Verify yourself.** Run every **Verify** command and check every **Acceptance**
   bullet. Don't trust the worker's report. Read the diff for determinism-rule
   violations.
   Then [check for leftover processes](#check-for-leftover-processes).
4. **Escalate on failure.** If a worker's result fails verification, send it the
   failure once. If it fails again, start a fresh worker one tier up (haiku to sonnet to
   opus) with both failures in its brief. If an opus worker fails, mark the WP `blocked`
   and move on.
5. **Record.** Commit with the WP ID prefix, update the WP's row in `STATUS.md` (status,
   **Done by**, date, notes), and add a line to your journal entry with the worker
   model and its reported token use. Then
   [clean up the worker's worktree](#clean-up-worktrees).
6. **Check the budget** (see [context budget and handoff](#context-budget-and-handoff))
   before starting the next WP.

### Stop

Stop when the target is done, everything left is blocked, a gate needs the owner (G1,
G2, G3, Launch), or the context budget says so. Then:

1. Finish or release every WP you claimed.
2. [Check for leftover processes](#check-for-leftover-processes), and stop every dev
   server you started unless the owner needs it for a review. If you leave one
   running, name it and its port in your message to the owner.
3. [Clean up worktrees](#clean-up-worktrees) for every worker that has finished.
4. Make sure every verified commit is on `main`, as
   [branches and commits](#branches-and-commits) describes.
5. Complete your journal entry, including **Next steps** for the next session.
6. Tell the owner: what's done, what's blocked, what needs their decision, and the
   exact prompt to start the next session.

## Check for leftover processes

Workers often leave a process running after they report: a script that stopped
responding, a dev server, or a test watcher. A background task that the app shows as
running for far longer than its WP took is usually one of these. The orchestrator
checks for them after each worker finishes, and again before it stops:

1. List processes that came from this session's commands:

   ```bash
   ps -eo pid,etime,command | grep -E "bun|node|vite|cargo|maturin|python|wrangler" | grep -v grep
   ```

2. For each process whose command points at this repo, a worktree of it, or the
   session's scratch directory, decide whether a running WP still needs it. The
   `etime` column shows how long it has run.
3. Stop the ones nothing needs: `kill PID`. Stop a dev server that the Browser pane
   started with `preview_stop` instead.
4. Note what you stopped in your journal entry if it affected a WP's result.

Don't stop processes that this session didn't start, such as the owner's own dev
servers or another session's workers.

## Clean up worktrees

Each worker worktree under `.claude/worktrees/` holds its own Cargo `target/` and
`node_modules`, about 3-6 GB. Leftover worktrees fill the owner's disk quickly, so the
orchestrator removes each one as soon as its work is done:

1. Wait until the worker's WP is verified, committed, and pushed to `main`, or released.
   Never remove the worktree of a worker that's still running.
2. Confirm that the worktree has no uncommitted work. The following command must print
   nothing:

   ```bash
   git -C WORKTREE_PATH status --porcelain
   ```

3. Remove the worktree, and keep its branch so the commits stay recoverable:

   ```bash
   git worktree remove -f -f WORKTREE_PATH
   git worktree prune
   ```

Replace `WORKTREE_PATH` with the worktree's path, for example
`.claude/worktrees/agent-a59bf26280e82dbe1`. If a worktree has uncommitted work, leave
it and name it in your message to the owner.

## Choose a worker

Every roadmap card has a **Worker** field. The orchestrator uses it as the starting
model and escalates as described in the loop. The tiers mean the following:

| Worker | Use for | Examples |
|---|---|---|
| `haiku` | Small or data-heavy WPs that follow an existing pattern: content files, movesets modeled on an existing one, simple UI, adapters | WP-015, WP-050, WP-053, WP-071 |
| `sonnet` | Most feature work: systems with clear specs, rendering, UI, levels, most bosses | WP-011, WP-022, WP-060, WP-074 |
| `opus` | Foundations that later WPs build on, or work where a subtle mistake corrupts determinism or the RL interface | WP-010, WP-020, WP-030, WP-040, WP-091 |

The orchestrator may move a WP up a tier before starting it if the card looks harder
than its size suggests, and records why in the journal. It never moves a WP down a
tier.

## Context budget and handoff

The budget is **per context window, not combined**: each session and each subagent
gets its own **500K tokens**. An orchestrator doesn't add its workers' tokens to its
own, and a worker doesn't add the tokens of subagents it starts. The owner set this
rule on 2026-11-12. Long sessions get slower, cost more, and lose track of early
details, so a session or agent hands off instead of pushing on.

- **Measure.** After each WP, read the orchestrator's own context size (in the Claude
  desktop app, the `get_usage` session tool reports it; otherwise estimate from the
  conversation length). Record it in the journal entry, with each worker's reported
  token count next to its WP.
- **350K: wind down.** At 350K of its own context, the orchestrator starts no new WPs.
  It lets running workers finish and verifies them.
- **400K: hand off.** Finish the [stop](#stop) steps now. Tell the owner the session
  is at its context budget and give the prompt for the next session, which is the
  standard orchestrator prompt with `TARGET` set to what's left.
- **500K: hard stop.** The same limit applies to each worker. A worker that nears 500K
  stops and reports what's done and what's left; the orchestrator releases or
  [splits](#split-a-work-package) the WP. A half-finished WP is released, not
  committed.
- **Size the batch.** Each verified WP costs the orchestrator roughly 15-40K of its own
  context, so an orchestrator session fits about 10-15 WPs. An L worker typically
  reports 100-250K.

## Run a single work package

To run one WP without an orchestrator, paste the following prompt into a fresh session,
replacing `WP_ID` with the card's ID, for example `WP-020`:

```text
You are implementing one work package of Cinderheart, the souls-like game in this
repo. Follow docs/game/11-agent-playbook.md exactly.

1. Read docs/game/README.md (glossary and reading order), then docs/game/STATUS.md.
2. Open docs/game/10-roadmap.md and find WP_ID. Confirm that every WP in its
   "Depends on" list is marked done in STATUS.md. If one isn't, stop and tell me.
3. Read only the spec sections the card lists under "Read". Don't read the whole
   docs folder.
4. Plan the change, then implement it. Touch only the paths the card lists under
   "Touch". If you must touch anything else, say why in your report.
5. Run every command under "Verify" and meet every "Acceptance" bullet.
6. Update STATUS.md (status, worker, session, date, branch, notes), add an entry to
   docs/game/JOURNAL.md, and, if you changed a spec decision, add an ADR under
   docs/game/decisions/.
7. Report: what was asked, what changed, what you verified, what's left.

Shell rule: call /bin/ls instead of ls, and never run eza.
Process rule: give long commands a timeout, and stop every process you started (dev
servers, scripts, watchers) before you report.
```

For a WP that you want to run with parallel subagents, add this line to the prompt:

```text
You may use subagents for independent parts of this WP, for example one per
enemy or one per data file. Give each subagent the exact file paths it owns.
```

## Branches and commits

The owner decided on 2026-11-12 that sessions commit to `main` directly, without pull
requests.

- A session works on a local branch named for its target, for example
  `game/m2-combat`, started from `origin/main`. Worker worktrees branch from it.
- Commit once per verified WP, in small buildable steps. Prefix each subject with the
  WP ID: `WP-020: Add hitbox sweep queries`. The commit message says how the WP was
  verified, what wasn't verified, and any decision the owner must make.
- After each verified WP, push it to `main`: `git fetch origin`, rebase the session
  branch onto `origin/main` if `main` moved, then `git push origin HEAD:main`. Never
  force-push `main`.
- Push only after the whole [definition of done](#definition-of-done) passes locally,
  because no pull request runs CI first. A push to `main` deploys the site. The game
  lives behind the `?engine=lf` flag until [WP-086](10-roadmap.md#wp-086), so partial
  worlds can ship without breaking the site.
- If CI fails on `main` after a push, fixing it comes before the next WP.

## Definition of done

A WP is done when all of the following are true:

- Every acceptance bullet on the card is met and was checked, not assumed.
- `cargo test --workspace` passes in `game/`.
- `cargo clippy --workspace -- -D warnings` passes in `game/`.
- `bun run check` passes at the repo root if any TypeScript or Svelte changed.
- Every golden replay in `game/tests/replays/` still produces its recorded state hash,
  or the WP deliberately changes simulation behavior and re-records the hashes with
  `cargo run -p lf-tools -- replay rehash` and says so in the commit message.
- Content data passes `cargo run -p lf-tools -- validate`.
- If the WP is visible in the browser, the agent took screenshots by using the
  [visual verification protocol](#verify-visually) and attached or described them.
- `STATUS.md` and `JOURNAL.md` are updated.

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
`game/data/` and mention the change in the commit message.

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
