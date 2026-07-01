# CLAUDE.md

Guidance for working in this repo. For the full deployment runbook, auth setup, and
first-time provisioning, see [README.md](README.md) — this file is the day-to-day
orientation.

## What this is

`ifkash.dev` — a personal site plus a small API, both on Cloudflare's free tier. **Two
deployables in one repo:**

- **Frontend** (`/src`) — SvelteKit 4 + Vite, `@sveltejs/adapter-cloudflare`, served by
  Cloudflare **Pages** at `ifkash.dev`. Effectively an SPA.
- **API** (`/api`) — a **Rust** Cloudflare **Worker** (compiled to WASM via
  `worker-build`) mounted at `ifkash.dev/api/*`.

Beyond the résumé editor the README centers on, the app has grown several
Google-authenticated mini-apps: `/fitness` (workout + meal + cardio tracker), `/tools/splitter`
(expense splitter), and `/game` (a browser FPS with a leaderboard).

## Toolchain & commands

Tool versions are pinned by [mise](https://mise.jdx.dev) (`mise.toml`): Rust, Bun, Node
LTS, Typst, Wrangler. Run `mise install` once. **Package manager is Bun**, not npm/pnpm.

```sh
# Frontend (SvelteKit → http://localhost:5173, or :5180 per .claude/launch.json)
mise run web:dev          # bun install && bun run dev --host

# API Worker (→ http://127.0.0.1:8787), local KV/D1/R2 persisted to .wrangler/state
mise run api:prep         # one time: wasm target + worker-build
mise run api:dev
mise run api:clean        # wipe local Miniflare state

# Checks / builds
bun run check             # svelte-check — the frontend typecheck (run before finishing FE work)
bun run build             # production frontend build
mise run api:build        # production Worker build (worker-build --release)
cd api && cargo check     # typecheck the Rust Worker (run before finishing API work)
```

There is **no test suite and no separate lint step** — `bun run check` (frontend) and
`cargo check` (API) are the gates. Match existing style rather than introducing tooling.

### Seeding local D1

Miniflare keeps a separate local DB. After adding a migration, apply it locally and
restart the Worker (`wrangler dev` does **not** hot-reload D1 schema or `.dev.vars`):

```sh
cd api && npx wrangler d1 migrations apply ifkash --local
```

## Architecture notes

- **Frontend → API base URL** is resolved by `src/lib/apiBase.ts`: `localhost`/`127.0.0.1`
  hits the local Worker (`:8787`); everything else hits prod. Append `?api=dev` / `?api=prod`
  to any URL to override (persisted in `localStorage`). Always call `getApiBase()` — never
  hardcode a base.
- **API routing**: every endpoint is registered in `api/src/routes/mod.rs`
  (`register_routes`), which maps URLs to handlers in `api/src/handlers/*.rs`. Adding an
  endpoint = handler fn + a line in `mod.rs` (+ update `routes/openapi.rs` if it should
  show in Swagger at `/api/docs`).
- **Data lives in Cloudflare D1** (SQLite), with **R2** for résumé PDFs and **KV** for the
  HN cache. Bindings (`IFKASH_D1`, `RESUME_BUCKET`, `IFKASH_HN`) are in `api/wrangler.toml`.

## Two separate auth models — don't mix them up

1. **Cloudflare Access (owner-only)** — protects the résumé editor and its write APIs.
   The Worker trusts the edge-injected `Cf-Access-Authenticated-User-Email` header and
   checks it equals `OWNER_EMAIL` (`handlers/access.rs`). Bypassed locally via
   `LOCAL_DEV="1"` in `api/.dev.vars`.
2. **Google ID token (per-user)** — used by `/game`, `/tools/splitter`, and `/fitness`
   (workout/meals/profile). The browser sends `Authorization: Bearer <google_id_token>`;
   the Worker verifies it against `GOOGLE_CLIENT_ID` in `handlers/google_auth.rs` and scopes
   rows to that user (`game_users`). On the frontend the token is stored in `localStorage`
   and shared across the splitter and fitness tools via `setToken`/`loadToken`/`AuthError`
   in `src/lib/splitterApi.ts` — reuse those rather than re-implementing sign-in. A `401`
   throws `AuthError` so the UI can drop the token and re-show the Google button.

## Conventions & gotchas

- **Money and bodyweights are stored as integer units, never floats** — cents for money,
  **grams** for weights — to avoid float drift. Convert at the UI boundary
  (`kgToGrams`/`gramsToKg` in `src/lib/workout.ts`). D1 also rejects bigint JsValues, so the
  Rust side binds every integer column as `f64` (see the `n()` helper in `handlers/workout.rs`).
- **D1 has no real transactions across statements** except via `d1.batch(...)`, which runs
  as one transaction. The workout upsert relies on this: it clears and re-inserts a session's
  sets/cardio in a single batch so overlapping auto-saves can't double-insert rows. Keep
  clear-then-reinsert flows inside one `batch`.
- **Auto-save pattern**: the fitness pages debounce saves and serialize in-flight flushes
  (a `flushing`/`flushPending` pair) so two saves never overlap. Follow that pattern when
  adding auto-saved fields rather than firing requests on every keystroke.
- **Shared logic lives in `src/lib`**, imported via the `$lib` alias. Pure domain math is
  kept I/O-free and separate from API wrappers — e.g. `fitnessMetrics.ts` (BMI/BMR/TDEE, MET
  kcal math) vs `workoutApi.ts` (fetch calls). Put new pure helpers alongside the former.
- **Migrations are append-only**, numbered `NNNN_name.sql` in `api/migrations/`. Add a new
  file; never edit an applied one. New tables/columns need a matching migration **and** a
  redeploy before the feature works in prod (`wrangler d1 migrations apply ifkash --remote`).
- **Résumé rendering is fully client-side** — Typst → PDF happens in the browser via
  `typst.ts` WASM (`src/lib/typst.ts`); there is no compile server. It fetches its WASM
  compiler from a CDN on first compile (needs internet) and uses fonts from `static/fonts`.
- **Secrets**: prod via `wrangler secret put`; local via `api/.dev.vars` (gitignored).
  Committed non-secret vars (`OWNER_EMAIL`, `GOOGLE_CLIENT_ID`) live in `wrangler.toml`
  `[vars]`. `PUBLIC_GOOGLE_CLIENT_ID` (web env) must match the Worker's `GOOGLE_CLIENT_ID`.

## Deploy (brief)

Push to `main`: the Worker deploys via `.github/workflows/deploy.yml` (on `api/**` changes),
the frontend via the Cloudflare Pages Git integration. Migrations are **not** applied by CI —
run `wrangler d1 migrations apply ifkash --remote` yourself. Full runbook in the README.
