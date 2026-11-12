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
