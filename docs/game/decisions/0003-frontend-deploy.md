# 0003: Deploy the frontend from GitHub Actions

- **Status:** Accepted. The owner approved it on 2026-11-12. Rollout is waiting on the
  owner steps in [Cut-over steps](#cut-over-steps).
- **Date:** 2026-11-12
- **WP:** [WP-005](../10-roadmap.md#wp-005)

## Context

The frontend deploys today through the Cloudflare Pages Git integration, which runs
`bun run build`. The game adds a WASM module built from `game/crates/wasm` with Cargo,
`wasm-bindgen`, and `wasm-opt`. The Pages build image doesn't include a Rust toolchain,
and installing one on every build is slow.

## Decision

Build and deploy the frontend from a GitHub Actions workflow:

1. On pushes to `main` that touch `src/**`, `static/**`, `game/**`, or the frontend
   config files, install Rust (with the pinned toolchain), Bun, `wasm-bindgen-cli`,
   and `binaryen`.
2. Run `mise run game:wasm`, `bun install --frozen-lockfile`, and `bun run build`.
3. Deploy the build output to the existing Pages project with `wrangler pages deploy`,
   using repository secrets for the Cloudflare API token and account ID.
4. On pull requests, deploy a preview to a branch alias of the same project.
5. Disconnect the Pages Git integration only after the workflow has deployed
   successfully once.

The Pages project's build settings are `bun run build` with output directory
`.svelte-kit/cloudflare` (see the README deploy runbook). That is where
`@sveltejs/adapter-cloudflare` writes `_worker.js`, `_routes.json`, and the static
assets, so that's the directory to deploy.

### Workflow

The workflow is [`.github/workflows/web-deploy.yml`](../../../.github/workflows/web-deploy.yml).
It has a build job (Rust toolchain, `wasm-bindgen-cli` matching `Cargo.lock`, binaryen 126,
`mise run game:wasm`, `bun install --frozen-lockfile`, `bun run build`), a production
deploy job for pushes to `main`, and a preview deploy job for pull requests from this
repository. Both deploy jobs deploy to the `ifkash` Pages project.

One detail needs attention before the first deploy:

- **Environment variables.** The Pages dashboard may define build-time variables such as
  `PUBLIC_GOOGLE_CLIENT_ID`. Move each one to a GitHub repository variable (or secret)
  and pass it to the **Build site** step. A variable left behind in the dashboard isn't
  available to a GitHub-hosted build.

### Secrets and variables

| Name | Kind | Purpose |
|---|---|---|
| `CLOUDFLARE_ACCOUNT_ID` | Secret | Already exists. Used by `deploy.yml`. |
| `CLOUDFLARE_PAGES_API_TOKEN` | Secret | New. A token with **Cloudflare Pages: Edit**. |
| `PUBLIC_GOOGLE_CLIENT_ID` | Variable | New. Must match the Worker's `GOOGLE_CLIENT_ID`. |

`deploy.yml` uses `CLOUDFLARE_WORKERS_API_TOKEN`, whose scopes are Workers, D1, R2, and
KV. Create a separate Pages token instead of widening that one. If you'd rather reuse a
single token, add the **Cloudflare Pages: Edit** scope to
`CLOUDFLARE_WORKERS_API_TOKEN` and change the workflow's secret name.

### Cut-over steps

> **Warning:** The built WASM module is no longer committed. The Pages Git integration
> can't build it, so from the moment this change reaches `main`, Git-integration
> deploys break. Finish steps 1 to 3 before you merge, and step 4 right after the first
> Actions deploy succeeds.

1. Create the `CLOUDFLARE_PAGES_API_TOKEN` secret, the `PUBLIC_GOOGLE_CLIENT_ID`
   variable, and a `production` environment in the GitHub repository settings.
2. Open a pull request and confirm that the preview job deploys to the `ifkash` project
   and that the preview site loads.
3. Merge. Wait for the production deploy to finish, then check `ifkash.dev` (home,
   `/fitness`, `/game`, and a hard refresh on a deep link).
4. Disconnect the Git integration with `cf pages projects disconnect-source ifkash`, or
   in the Cloudflare dashboard under the project's **Settings** >
   **Builds & deployments**. Don't disconnect before step 3 passes.
   Until you disconnect, the Git integration keeps building on each push, and those
   builds fail because the WASM module isn't committed.
5. Update the README deploy runbook and the "Deploy (brief)" section in `CLAUDE.md` to
   say that Actions deploys the frontend.

### Rollback

- **A bad deploy.** In the Cloudflare dashboard, open the project's **Deployments** and
  choose **Rollback to this deployment** on the last good one. This takes effect in
  seconds and doesn't need a Git change.
- **The workflow itself fails.** Reconnect the Git integration in **Settings** >
  **Builds & deployments**, and commit the output of `mise run game:wasm` (remove
  `src/lib/latticefall/wasm/` from `.gitignore`). The Git integration can't build the
  site without those files.
- **Longer term.** Switch to the committed-WASM fallback in the next section.

## Consequences

- The WASM module is always built from the same commit as the site.
- Deploys depend on GitHub Actions minutes and a Cloudflare API token secret.
- Preview deployments for pull requests use a second job (`wrangler pages deploy`
  with a branch name), as shown in the listing.
- Pages build settings and environment variables move out of the Cloudflare dashboard
  and into the repository.

## Alternatives considered

- **Commit the built WASM files** (`src/lib/latticefall/wasm/`) and keep the Git
  integration. Simple, but every core change needs a local WASM build and commit, and
  it's easy to deploy a stale module. A CI check can compare the committed module's
  hash to a fresh build to catch staleness. M0 used this until the owner approved this
  ADR, and it remains the rollback path.
- **Install Rust in the Pages build command.** Possible, but slow and fragile.

## Owner action needed

You approved this ADR. Before merging the branch that adds `web-deploy.yml`:

1. Create the `CLOUDFLARE_PAGES_API_TOKEN` secret (a token with **Cloudflare Pages:
   Edit**) and the `PUBLIC_GOOGLE_CLIENT_ID` variable. `CLOUDFLARE_ACCOUNT_ID` already
   exists.
2. After the first successful Actions deploy, disconnect the Pages Git integration with
   `cf pages projects disconnect-source ifkash`.
