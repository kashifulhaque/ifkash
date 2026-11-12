# Journal

This file is the handoff log between sessions. Every orchestrator (or single-WP)
session adds one entry and keeps it current while it works. The newest entry goes at
the top. Never edit an earlier session's entry; correct it in your own.

[`STATUS.md`](STATUS.md) says *what* state each WP is in. This journal says *who* did
it, *how*, at what context cost, and *what the next session must do first*.

## Entry template

Copy this block to the top of the entries and fill it in as you go:

```markdown
### YYYY-MM-DD: SESSION_NAME (TARGET)

- **Orchestrator:** model and effort; session ID if the app shows one
- **Branch and PR:** branch name; pull request link
- **Started from:** `main` at COMMIT, or the previous session's branch
- **WPs:**
  - WP-NNN: done by WORKER (escalated from ...?); commit SHA; worker tokens
  - WP-NNN: blocked: reason
- **Context:** orchestrator NNK + workers NNK = NNK total (budget 500K)
- **Not verified:** anything the session couldn't check
- **Waiting on the owner:** decisions, credentials, playtests, merges
- **Next steps:** what the next session does first, in order
```

## Entries

### 2026-11-12: latticefall-m1 (M1)

- **Orchestrator:** Opus 5.5, low effort; session `local_e9f21284-d98a-499d-989a-a9b9aee9498c`
- **Branch and PR:** `game/m1-movement`; PR linked below once opened
- **Started from:** `origin/main` at `daffff9` (M0 and the playbook are merged)
- **WPs:**
  - WP-010: done by opus; commit `99b31ed`; 235K worker tokens
  - WP-011: done by sonnet in its own worktree, merged in `543fdb4`, with a follow-up
    fix for WP-010's new `Replay::record` signature; 126K worker tokens
  - WP-013: done by sonnet in its own worktree (the owner approved the animation in
    motion; fine-tuning comes later), merged in `9a34f77` (resolved a
    `scene.ts` conflict with WP-011's camera); 215K worker tokens. The owner approved
    downloading the Quaternius Universal Animation Library, `@gltf-transform/cli`, and
    toktx.
  - WP-012, WP-014, WP-015: not started (budget)
- **Context:** orchestrator 163K + workers 576K = 739K total (budget 500K). The three
  workers alone passed the hard stop, so this session stopped after verifying them.
  The playbook's "6-10 WPs per session" estimate doesn't hold when worker tokens count:
  two L workers and one M worker used 576K. Either raise the budget or count only the
  orchestrator's context; that's the owner's call.
- **Not verified:** pointer lock and gamepad input
  (WP-011 was checked with synthetic key events); x86 determinism (macOS arm64 only);
  the `game` CI workflow on GitHub for this branch.
- **Leftover process:** a WP-013 worker script stopped responding and ran for about
  1.5 hours after the worker reported. The orchestrator stopped it. The playbook now has
  a process rule for workers and a
  [leftover-process check](11-agent-playbook.md#check-for-leftover-processes).
- **Waiting on the owner:** decide the context-budget rule above; merge the PR.
- **Next steps:**
  1. Branch from `game/m1-movement` if the PR isn't merged, otherwise from `origin/main`.
  2. Run WP-015 (haiku; it touches `actor/targeting.rs` and `test_field.ron`) and
     WP-012 (sonnet; render/camera) in parallel. Agree on the snapshot's lock-target
     field first: WP-012 reads what WP-015 writes. WP-012 must take over the orbit yaw
     and pitch that WP-011 put in `engine/scene.ts`.
  3. Then WP-014. Then stop at gate G1 with the numbers: WASM 356,445 bytes gzipped;
     native bench 51,242 ticks/s on `test_field`; golden replays green on native, WASM,
     and Python.

### 2026-11-12: latticefall-m0 (M0)

- **Orchestrator:** Opus 5.5, medium effort; session `local_9d89a627-8c84-46ee-a494-c285e2afed9c`
- **Branch and PR:** `claude/game-m0-1730d5`;
  [kashifulhaque/ifkash#13](https://github.com/kashifulhaque/ifkash/pull/13)
- **Started from:** `main`, with the plan copied in from the planning session as the
  first commit
- **WPs:** WP-001 to WP-008 are all done. Every worker was Sonnet 5.5 at medium effort,
  and the orchestrator re-ran every verify command. Commits and details are in
  `STATUS.md`. ADR 0004 (portfolio text isn't part of the content hash) was added.
- **Context:** not recorded. This entry predates the context budget.
- **Not verified:** 60 fps in a visible browser; a build from a fresh clone.
- **Waiting on the owner:** none for M0 itself; see the planning session's entry.

### 2026-11-12: planning and deploy follow-up (plan, ADR 0003, PR #13 CI)

- **Orchestrator:** Opus 5.5 (the session that wrote the plan)
- **Branch and PR:** follow-up commits on `claude/game-m0-1730d5`, PR #13
- **What happened:**
  - Wrote the plan in `docs/game/`, with research from five Haiku agents and a
    consistency review by a separate agent.
  - The owner approved ADR 0003. `web-deploy.yml` deploys the site to the `ifkash`
    Pages project from GitHub Actions. The `CLOUDFLARE_PAGES_API_TOKEN` secret exists.
  - Fixed two preview-deploy failures on PR #13: the deploy jobs now get the full
    SvelteKit output and install `node_modules`. The preview deployed and serves
    `/`, `/game`, `/game?engine=lf`, `/work`, and `/projects`.
  - Added the orchestrator playbook, a worker model on every WP card, this journal, and
    the context budget.
- **Context:** about 470K. This session ran past the 400K handoff point while writing
  these rules, and handed off at the end.
- **Not verified:** that the game renders in a real browser on the preview, and the
  Google sign-in pages on the preview.
- **Owner actions, done:**
  - PR #13 was squash-merged to `main` as `0834919`, and the first production deploy
    from GitHub Actions succeeded. `ifkash.dev` serves `/`, `/game`,
    `/game?engine=lf`, and `/work`.
  - The Pages Git integration's automatic builds are off: production and preview
    deployments and PR comments are disabled. The Git connection itself was kept,
    because `cf pages projects disconnect-source` warns that it permanently deletes
    the resource. GitHub Actions is now the only deploy path.
  - The plan update (this playbook, worker models, and this journal) missed the squash
    merge and went to `main` in a follow-up pull request.
- **Still waiting on the owner:** a review of the relic text in
  `game/data/portfolio.ron`. It doesn't block M1.
- **Next steps:**
  1. Branch M1 from `main`. The owner's main checkout may be behind `origin/main`;
     fetch first and branch from `origin/main`.
  2. Run M1: WP-010 (opus), then WP-011, WP-012, WP-013, and WP-015 in parallel where
     Touch lists allow, then WP-014.
  3. Stop at gate G1 and give the owner the G1 numbers from the roadmap.
