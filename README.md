# [ifkash.dev](https://ifkash.dev) — Kashiful Haque

Personal website + a small API, both running on Cloudflare's free tier. Includes a
browser‑based **résumé editor** that compiles [Typst](https://typst.app) to PDF
client‑side and stores versions in Cloudflare D1 + R2.

> **No servers to maintain.** Everything runs on Cloudflare Pages, Workers, D1, R2
> and KV. There is no VM, container, or database host to pay for or babysit.

---

## Table of contents

- [What it is](#what-it-is)
- [Architecture](#architecture)
- [Repo structure](#repo-structure)
- [Tech stack](#tech-stack)
- [API endpoints](#api-endpoints)
- [Cloudflare resources & bindings](#cloudflare-resources--bindings)
- [Environment variables & secrets](#environment-variables--secrets)
- [Run locally](#run-locally)
- [The résumé editor](#the-résumé-editor)
- [Deployment runbook](#deployment-runbook)
- [Troubleshooting](#troubleshooting)

---

## What it is

Two deployables in one repo:

1. **Frontend** (`/src`) — a [SvelteKit](https://kit.svelte.dev) site (home, work,
   projects, blog, leetcode/tensara/HN dashboards, a résumé editor, etc.) built with
   `@sveltejs/adapter-cloudflare` and served by **Cloudflare Pages** at `ifkash.dev`.
2. **API** (`/api`) — a **Rust Cloudflare Worker** (compiled to WASM via
   `worker-build`) mounted at `ifkash.dev/api/*`. It proxies a few third‑party APIs
   (Hacker News, LeetCode, Tensara, WeatherUnion, OpenRouter) and powers the résumé
   feature with D1 + R2.

The résumé pipeline is fully serverless:

- **Rendering** happens **in the browser** via [`typst.ts`](https://github.com/Myriad-Dreamin/typst.ts) (WASM) — there is no compile server.
- **Storage**: the Typst source + version metadata live in **D1**; the rendered PDF
  lives in **R2**.
- **Auth** for the editor is handled at the edge by **Cloudflare Access**.

---

## Architecture

```
                          ┌───────────────────────────────────────────┐
   Browser  ──────────▶   │  Cloudflare (zone: ifkash.dev)              │
                          │                                             │
   ifkash.dev/*           │   Pages  ──────────────▶  SvelteKit (SPA)   │
                          │                                             │
   ifkash.dev/api/*       │   Worker (Rust/WASM, "ifkash-api")          │
                          │     ├── KV   IFKASH_HN      (HN cache)       │
                          │     ├── D1   IFKASH_D1      (résumé source + │
                          │     │                        version history)│
                          │     └── R2   RESUME_BUCKET  (rendered PDFs)  │
                          │                                             │
   /editor, write APIs ── │   Cloudflare Access (email policy) ─────────│
                          └───────────────────────────────────────────┘
                                        │
                                        ▼  (outbound from the Worker)
                   OpenRouter · WeatherUnion · Hacker News · LeetCode · Tensara
```

**Résumé read path** (`GET /api/resume`): Worker reads the latest row from D1 →
fetches the PDF bytes from R2 by key → streams them back. If D1/R2 is empty or
unavailable, it falls back to the committed static PDF at
`/assets/Kashiful_Haque.pdf` so the resume always loads.

**Résumé write path** (editor → `POST /api/resume/upload`): the browser compiles
Typst → PDF with `typst.ts`, then uploads the PDF + source; the Worker writes the
PDF to R2 (`resumes/<timestamp>.pdf`) and inserts a row in D1 with the R2 key.

---

## Repo structure

```
ifkash/
├── api/                          # Rust Cloudflare Worker (the API)
│   ├── src/
│   │   ├── lib.rs                # Worker entry (#[event(fetch)]) + CORS
│   │   ├── routes/               # thin route → handler wrappers + router
│   │   │   ├── mod.rs            # register_routes() — the URL map
│   │   │   └── openapi.rs        # OpenAPI doc (served at /api/openapi, UI at /api/docs)
│   │   └── handlers/             # actual logic
│   │       ├── resume.rs         # public GET /api/resume (D1 + R2, static fallback)
│   │       ├── resume_api.rs     # latest/history/record/upload (D1 + R2, Access)
│   │       ├── access.rs         # Cloudflare Access verification helper
│   │       ├── ai.rs             # /api/ai/rewrite (OpenRouter)
│   │       ├── hn.rs lc.rs tensara.rs home_weather.rs ...
│   │       └── ...
│   ├── migrations/               # D1 schema migrations (SQL)
│   ├── Cargo.toml
│   ├── wrangler.toml             # Worker config: route, bindings, vars
│   └── .dev.vars                 # local secrets (gitignored)
│
├── src/                          # SvelteKit frontend
│   ├── routes/                   # pages (/, /work, /projects, /editor, ...)
│   │   └── editor/               # the résumé editor (Monaco + typst.ts preview)
│   ├── lib/
│   │   ├── typst.ts              # browser Typst → PDF (typst.ts WASM)
│   │   └── apiBase.ts            # dev vs prod API base URL
│   └── app.html / app.css
│
├── static/
│   ├── assets/Kashiful_Haque.{typ,pdf}   # committed résumé (static fallback)
│   └── fonts/                            # Crimson Pro + Cardo (used by typst.ts)
│
├── .github/workflows/deploy.yml  # CI: build + deploy the Worker on push to main
├── mise.toml                     # tool versions (rust, bun, node, typst, wrangler) + tasks
├── svelte.config.js              # adapter-cloudflare
└── package.json
```

---

## Tech stack

| Layer | Tech |
|---|---|
| Tooling / versions | [mise](https://mise.jdx.dev) → Rust, Bun, Node (LTS), Typst, Wrangler |
| Frontend | [SvelteKit](https://kit.svelte.dev) 4 + Vite, `@sveltejs/adapter-cloudflare`, Monaco editor |
| Typst rendering | [`@myriaddreamin/typst.ts`](https://github.com/Myriad-Dreamin/typst.ts) (WASM, in‑browser) |
| API | Rust + [`worker`](https://crates.io/crates/worker) 0.7 → WASM via `worker-build` |
| Data | Cloudflare **D1** (SQLite), **R2** (objects), **KV** |
| Auth | Cloudflare **Access** (Zero Trust) |
| Hosting | Cloudflare **Pages** (frontend) + **Workers** (API) |

---

## API endpoints

Base URL: `https://ifkash.dev/api` (locally `http://127.0.0.1:8787/api`).
Interactive docs: [`/api/docs`](https://ifkash.dev/api/docs) (Swagger UI), spec at `/api/openapi`.

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/resume` | public | Latest résumé PDF. `?format=json\|url\|view\|download`, `?id=<n>` for a specific version |
| GET | `/api/resume/latest` | Access | Latest version incl. Typst source (for the editor) |
| GET | `/api/resume/history` | Access | List of versions |
| GET | `/api/resume/record/:id` | Access | A specific version incl. source |
| POST | `/api/resume/upload` | Access | Save a new version (multipart: `pdf_file`, `typst_source`, `version`) |
| POST | `/api/ai/rewrite` | Access | Tailor the Typst résumé to a job description (OpenRouter) |
| GET | `/api/hn` | public | Hacker News top stories (cached in KV) |
| GET | `/api/lc/profile` · POST `/api/lc/submissions` | public | LeetCode profile / submissions |
| GET | `/api/tensara/profile` | public | Tensara profile |
| GET | `/api/home_weather` | public | WeatherUnion proxy |
| GET | `/api/hello` · `/api/ping` | public | Health checks |

> "Access" = protected by Cloudflare Access; the Worker also checks the
> `Cf-Access-Authenticated-User-Email` header against `OWNER_EMAIL`. On `localhost`
> these checks are bypassed so the editor works in local dev.

---

## Cloudflare resources & bindings

Configured in [`api/wrangler.toml`](api/wrangler.toml):

| Binding | Type | Name | Purpose |
|---|---|---|---|
| `IFKASH_HN` | KV | — | Hacker News cache |
| `IFKASH_D1` | D1 | `ifkash` | Résumé source + version history |
| `RESUME_BUCKET` | R2 | `ifkash` | Rendered résumé PDFs |

- Route: `ifkash.dev/api/*` → Worker `ifkash-api`.
- `workers_dev = false` — the Worker is **only** reachable via `ifkash.dev/api/*`, so
  a `*.workers.dev` URL can't bypass Cloudflare Access.

---

## Environment variables & secrets

| Name | Where | Used by |
|---|---|---|
| `OWNER_EMAIL` | `[vars]` in `wrangler.toml` (committed) | Access email check |
| `OPENROUTER_API_KEY` | Worker secret (`wrangler secret put`) | `/api/ai/rewrite` |
| `WEATHER_UNION_API_KEY` | Worker secret | `/api/home_weather` |
| `CLOUDFLARE_WORKERS_API_TOKEN` | GitHub Actions secret | CI deploy of the Worker |
| `CLOUDFLARE_ACCOUNT_ID` | GitHub Actions secret | CI deploy of the Worker |

For **local** development, create `api/.dev.vars` (gitignored):

```ini
OPENROUTER_API_KEY="sk-or-..."
WEATHER_UNION_API_KEY="..."
# Bypass the Cloudflare Access check under `wrangler dev` (never set in prod).
LOCAL_DEV="1"
```

> `wrangler dev` hot-reloads Worker code but **not** `.dev.vars` — restart
> `mise run api:dev` after changing it.

---

## Run locally

### Prerequisites

```sh
# Install mise (one time) — manages Rust, Bun, Node, Typst, Wrangler
curl https://mise.run | sh

# Install the pinned toolchains
mise install
```

### Start the frontend (SvelteKit → http://localhost:5173)

```sh
mise run web:dev          # runs `bun install` then `bun run dev` (Vite, --host)
```

### Start the API (Worker → http://127.0.0.1:8787)

```sh
mise run api:prep         # one time: add wasm target + install worker-build
mise run api:dev          # wrangler dev with local KV/D1/R2 persisted to .wrangler/state
```

Then seed the **local** D1 database (Miniflare keeps it separate from production):

```sh
cd api
mise exec -- wrangler d1 migrations apply ifkash --local
```

Notes for local dev:
- The frontend auto‑targets `http://127.0.0.1:8787` when running on `localhost`
  (see `src/lib/apiBase.ts`). Append `?api=prod` to a URL to hit production instead.
- Cloudflare Access isn't in front of localhost. Under `wrangler dev` the Worker
  even sees the request host as `ifkash.dev` (the route host), so the auth bypass
  is driven by the `LOCAL_DEV` flag in `api/.dev.vars` rather than host detection.
- `typst.ts` fetches its WASM compiler from jsDelivr on first compile (needs
  internet); fonts are served from `/static/fonts`.

Useful commands:

```sh
bun run check             # svelte-check (frontend types)
bun run build             # production build of the frontend
mise run api:build        # production build of the Worker (worker-build --release)
mise run api:clean        # wipe local Miniflare state (KV/D1/R2)
mise run assets:fonts     # (re)download résumé fonts into ./.fonts
```

---

## The résumé editor

- Lives at **`/editor`** (history at `/editor/history`), protected by Cloudflare Access.
- Edit Typst in Monaco → **Run** compiles to PDF **in the browser** (`src/lib/typst.ts`
  via `typst.ts`) → preview in an iframe.
- **Upload** saves a new version: the PDF goes to R2, the source + metadata to D1.
- **AI Tailor** rewrites the Typst for a pasted job description via `/api/ai/rewrite`.
- Fonts: the résumé uses **Crimson Pro** + **Cardo**, committed in `static/fonts/` and
  loaded into the WASM compiler. (Source of truth for regenerating them:
  `mise run assets:fonts`.)
- The canonical résumé is also committed at `static/assets/Kashiful_Haque.{typ,pdf}`
  and is what `GET /api/resume` falls back to before the first save.

---

## Deployment runbook

### Routine deploy (the normal case)

Push to `main`:

- **Worker**: `.github/workflows/deploy.yml` runs on changes under `api/**`, builds
  with `worker-build`, and deploys via `cloudflare/wrangler-action` using the
  `CLOUDFLARE_WORKERS_API_TOKEN` / `CLOUDFLARE_ACCOUNT_ID` GitHub secrets.
- **Frontend**: deployed automatically by the **Cloudflare Pages Git integration**.

Manual Worker deploy (needs a token with *Workers Scripts: Edit* — note a plain
`wrangler login` OAuth token may lack this):

```sh
cd api
mise exec -- wrangler deploy
```

### Applying D1 migrations

Migrations live in `api/migrations/` (`migrations_dir` is set in `wrangler.toml`).

```sh
cd api
# remote (production)
mise exec -- wrangler d1 migrations apply ifkash --remote
# or run a single file directly
mise exec -- wrangler d1 execute ifkash --remote --file=migrations/0002_use_r2_pdf_key.sql
```

### First‑time / from‑scratch setup

1. **Create resources** (names must match `wrangler.toml`):
   ```sh
   cd api
   mise exec -- wrangler d1 create ifkash          # then copy database_id into wrangler.toml
   mise exec -- wrangler r2 bucket create ifkash
   mise exec -- wrangler kv namespace create IFKASH_HN
   ```
2. **Apply migrations**: `wrangler d1 migrations apply ifkash --remote`.
3. **Set Worker secrets**:
   ```sh
   mise exec -- wrangler secret put OPENROUTER_API_KEY
   mise exec -- wrangler secret put WEATHER_UNION_API_KEY
   ```
4. **Set `OWNER_EMAIL`** in `wrangler.toml` `[vars]` to the email allowed to edit.
5. **Configure Cloudflare Access** (Zero Trust → Access → Applications →
   *Self‑hosted*) on `ifkash.dev`, with one policy **Allow → Emails → <your email>**,
   covering these paths (leave **`/api/resume` public**):
   ```
   /editor
   /api/resume/latest
   /api/resume/history
   /api/resume/record
   /api/resume/upload
   /api/ai/rewrite
   ```
6. **Add GitHub Actions secrets**: `CLOUDFLARE_WORKERS_API_TOKEN` (Workers Scripts +
   D1 + R2 + KV edit) and `CLOUDFLARE_ACCOUNT_ID`.
7. **Connect Cloudflare Pages** to the repo (build: `bun run build`, output:
   `.svelte-kit/cloudflare`) and point the `ifkash.dev` domain at it.
8. Push to `main` → Worker + Pages deploy. Open `/editor`, **Run**, then **Upload**
   to write version 1.

---

## Troubleshooting

| Symptom | Likely cause / fix |
|---|---|
| `GET /api/resume` returns the static PDF / `"version":"static-fallback"` | D1 is empty (no saved versions yet) or unreachable — save once in `/editor`. |
| Editor shows 401 on load/upload in production | Cloudflare Access not configured for that path, or your email ≠ `OWNER_EMAIL`. |
| `wrangler deploy` → `Authentication error [10000]` | Local token lacks *Workers Scripts: Edit*. Deploy via CI (push to `main`) or use a scoped API token. |
| Editor preview blank / "no PDF" | `typst.ts` WASM failed to load (needs internet on first compile) or a font/glyph issue — check the browser console. |
| Local editor can't read/write résumé | Run `wrangler d1 migrations apply ifkash --local` and restart `mise run api:dev`. |
| Fonts look wrong in the rendered PDF | Re‑fetch with `mise run assets:fonts` and ensure `static/fonts/*.ttf` are present. |

---

Built by [Kashiful Haque](https://ifkash.dev) · API docs at [`/api/docs`](https://ifkash.dev/api/docs)
