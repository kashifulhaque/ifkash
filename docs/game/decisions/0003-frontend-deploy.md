# 0003: Deploy the frontend from GitHub Actions

- **Status:** Proposed. Needs the owner's approval before anything changes.
- **Date:** 2026-11-12
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

### Proposed workflow

You would save the following as `.github/workflows/web-deploy.yml`. It doesn't exist in
the repository yet.

```yaml
name: Deploy frontend to Cloudflare Pages

on:
  push:
    branches:
      - main
    paths:
      - 'src/**'
      - 'static/**'
      - 'game/**'
      - 'package.json'
      - 'bun.lock'
      - 'svelte.config.js'
      - 'vite.config.ts'
      - 'tsconfig.json'
      - 'mise.toml'
      - '.github/workflows/web-deploy.yml'
  pull_request:
    paths:
      - 'src/**'
      - 'static/**'
      - 'game/**'
      - 'package.json'
      - 'bun.lock'
      - 'svelte.config.js'
      - 'vite.config.ts'
      - 'tsconfig.json'
      - 'mise.toml'
      - '.github/workflows/web-deploy.yml'
  workflow_dispatch:

# Serialize production deploys. Cancel superseded pull request previews only.
concurrency:
  group: web-deploy-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

permissions:
  contents: read
  pull-requests: write # For the preview URL comment.
  deployments: write # For GitHub deployment status from wrangler-action.

env:
  GAME_RUST_VERSION: '1.99.1' # Keep in sync with game/rust-toolchain.toml.
  # TODO(owner): Replace with the real Pages project name from the Cloudflare dashboard.
  PAGES_PROJECT: REPLACE_WITH_PAGES_PROJECT_NAME

jobs:
  build:
    name: Build
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v7

      - name: Install mise tools
        uses: jdx/mise-action@v5
        with:
          install_args: rust bun node

      - name: Install pinned Rust toolchain
        run: >
          rustup toolchain install "$GAME_RUST_VERSION" --profile minimal
          -c clippy -c rustfmt -t wasm32-unknown-unknown

      - name: Cache Cargo registry and build output
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: game -> target

      # The wasm-bindgen CLI version must match the crate version in Cargo.lock.
      - name: Read wasm-bindgen version from Cargo.lock
        id: wbg
        run: |
          version="$(grep -A1 -x 'name = "wasm-bindgen"' game/Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p' | head -n1)"
          echo "version=${version}" >> "$GITHUB_OUTPUT"

      - name: Install wasm-bindgen-cli
        if: steps.wbg.outputs.version != ''
        uses: taiki-e/install-action@v2
        with:
          tool: wasm-bindgen-cli@${{ steps.wbg.outputs.version }}

      - name: Install binaryen
        run: |
          sudo apt-get update
          sudo apt-get install -y --no-install-recommends binaryen

      - name: Build WASM module
        run: mise run game:wasm

      - name: Install frontend dependencies
        run: bun install --frozen-lockfile

      - name: Build site
        run: bun run build
        env:
          PUBLIC_GOOGLE_CLIENT_ID: ${{ vars.PUBLIC_GOOGLE_CLIENT_ID }}

      # adapter-cloudflare writes the Pages bundle here.
      - name: Upload build output
        uses: actions/upload-artifact@v7
        with:
          name: pages-bundle
          path: .svelte-kit/cloudflare
          if-no-files-found: error

  deploy-production:
    name: Deploy production
    needs: build
    if: github.event_name != 'pull_request'
    runs-on: ubuntu-latest
    environment: production
    steps:
      - name: Download build output
        uses: actions/download-artifact@v7
        with:
          name: pages-bundle
          path: dist

      - name: Deploy to Cloudflare Pages
        uses: cloudflare/wrangler-action@v3
        with:
          apiToken: ${{ secrets.CLOUDFLARE_PAGES_API_TOKEN }}
          accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
          wranglerVersion: '4.50.0'
          command: pages deploy dist --project-name=${{ env.PAGES_PROJECT }} --branch=main

  deploy-preview:
    name: Deploy preview
    needs: build
    # Pull requests from forks don't get repository secrets, so skip them.
    if: github.event_name == 'pull_request' && github.event.pull_request.head.repo.full_name == github.repository
    runs-on: ubuntu-latest
    steps:
      - name: Download build output
        uses: actions/download-artifact@v7
        with:
          name: pages-bundle
          path: dist

      - name: Deploy preview to Cloudflare Pages
        id: deploy
        uses: cloudflare/wrangler-action@v3
        with:
          apiToken: ${{ secrets.CLOUDFLARE_PAGES_API_TOKEN }}
          accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
          wranglerVersion: '4.50.0'
          command: pages deploy dist --project-name=${{ env.PAGES_PROJECT }} --branch=pr-${{ github.event.number }}

      - name: Comment preview URL
        env:
          GH_TOKEN: ${{ github.token }}
          PREVIEW_URL: ${{ steps.deploy.outputs.deployment-url }}
        run: gh pr comment "${{ github.event.number }}" --repo "${{ github.repository }}" --body "Preview deployed to ${PREVIEW_URL}"
```

Two details in the listing need attention before you approve it:

- **Project name.** The repository doesn't record the Pages project name. The
  `PAGES_PROJECT` value is a placeholder. Copy the real name from the Cloudflare
  dashboard under **Workers & Pages**.
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

1. Create the `CLOUDFLARE_PAGES_API_TOKEN` secret, the `PUBLIC_GOOGLE_CLIENT_ID`
   variable, and a `production` environment in the GitHub repository settings.
2. Replace the `PAGES_PROJECT` placeholder, then add `web-deploy.yml` in a pull request.
   Confirm the preview job deploys and the preview site loads.
3. Merge. Wait for the production deploy to finish, then check `ifkash.dev` (home,
   `/fitness`, `/game`, and a hard refresh on a deep link).
4. Disconnect the Git integration in the Cloudflare dashboard under the project's
   **Settings** > **Builds & deployments**. Don't disconnect before step 3 passes.
   Until you disconnect, both pipelines deploy on each push, which is harmless but
   wasteful.
5. Update the README deploy runbook and the "Deploy (brief)" section in `CLAUDE.md` to
   say that Actions deploys the frontend.

### Rollback

- **A bad deploy.** In the Cloudflare dashboard, open the project's **Deployments** and
  choose **Rollback to this deployment** on the last good one. This takes effect in
  seconds and doesn't need a Git change.
- **The workflow itself fails.** Reconnect the Git integration in **Settings** >
  **Builds & deployments**. The Git integration still builds the site, but without the
  WASM module, so keep the game behind the `?engine=lf` flag (see the
  [agent playbook](../11-agent-playbook.md#branches-commits-and-pull-requests)) until
  the workflow works again.
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
  hash to a fresh build to catch staleness. This is the fallback if the owner declines.
- **Install Rust in the Pages build command.** Possible, but slow and fragile.

## Owner action needed

Approve or decline this proposal. If you approve, provide the Pages project name and
create the `CLOUDFLARE_PAGES_API_TOKEN` secret. No agent changes the frontend deploy
until you do.
