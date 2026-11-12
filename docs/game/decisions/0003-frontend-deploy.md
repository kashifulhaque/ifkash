# 0003: Deploy the frontend from GitHub Actions

- **Status:** Proposed. Needs the owner's approval before anything changes.
- **Date:** 2026-11-11
- **WP:** [WP-005](../10-roadmap.md#wp-005)

## Context

The frontend deploys today through the Cloudflare Pages Git integration, which runs
`bun run build`. The game adds a WASM module built from `game/crates/wasm` with Cargo,
`wasm-bindgen`, and `wasm-opt`. The Pages build image doesn't include a Rust toolchain,
and installing one on every build is slow.

## Decision (proposed)

Build and deploy the frontend from a GitHub Actions workflow:

1. On pushes to `main` that touch `src/**`, `static/**`, `game/**`, or the frontend
   config files, install Rust (with the pinned toolchain), Bun, `wasm-bindgen-cli`,
   and `binaryen`.
2. Run `mise run game:wasm`, `bun install`, and `bun run build`.
3. Deploy with `wrangler pages deploy` to the existing Pages project, using a
   repository secret for the Cloudflare API token.
4. Disconnect the Pages Git integration only after the workflow has deployed
   successfully once.

WP-005 fills in the exact workflow file in this ADR before asking for approval.

## Consequences

- The WASM module is always built from the same commit as the site.
- Deploys depend on GitHub Actions minutes and a Cloudflare API token secret.
- Preview deployments for pull requests need a second job (`wrangler pages deploy`
  with a branch name).

## Alternatives considered

- **Commit the built WASM files** (`src/lib/latticefall/wasm/`) and keep the Git
  integration. Simple, but every core change needs a local WASM build and commit, and
  it's easy to deploy a stale module. A CI check can compare the committed module's
  hash to a fresh build to catch staleness. This is the fallback if the owner declines.
- **Install Rust in the Pages build command.** Possible, but slow and fragile.
