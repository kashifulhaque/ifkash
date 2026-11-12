# Portfolio integration

This document defines how the portfolio lives inside the game and around it: the
landing card, the mapping from portfolio entries to game objects, the codex pages, and
the rules that keep every fact reachable without playing. Read it before any WP in the
`ui` lane, and before any WP that touches `src/lib/content.ts`.

## Rules

1. **`src/lib/content.ts` is the single source of truth** for portfolio facts. The game
   never copies a fact into its own data; it references entries by slug and pulls the
   text at build time.
2. **Nothing is locked.** Every fact is reachable as plain HTML on the site, and inside
   the game from the first minute through the Wayfarer's Ledger and the codex.
3. **Facts are plain; flavor is fantasy.** The fantasy voice is for item descriptions
   and NPC lines. Role, dates, outcomes, and links appear in a plain facts panel.
4. **The game adapts to the content, not the other way round.** Adding a project,
   job, or contact link to `content.ts` must never break the game. Unmapped entries fall
   back to a generic object (see [Fallbacks](#fallbacks)).

## Landing card

Visiting `/game` first renders a DOM card, with no Three.js or WASM loaded. The card
shows:

- The owner's name and tagline (from `content.ts`).
- Buttons: **Play** (loads the game), **Résumé** (opens the PDF), **Contact**, and
  **Read the plain site**.
- A line that states the download size, for example "Downloads about 25 MB."
- A **Remember my choice** checkbox. If a visitor chooses the plain site and ticks it,
  later visits to `/game` show the card with **Read the plain site** focused.

The card renders within the site's normal layout and tokens, and meets the
three-second rule in [the success criteria](00-vision-and-scope.md#success-criteria).

## Content mapping

The following table maps each section of `content.ts` to its place in the game:

| Section | Game object | Where | Spec |
|---|---|---|---|
| Name and tagline | The Hearth's inscription; the ending's credits | Hollowhearth; after B12 | [World](02-world.md#hollowhearth-hub) |
| `work` | Chronicle Pages | One per work entry, in Chronicle Page slots | [Chronicle Pages](#chronicle-pages) |
| `education` | The Archivist; the Scholar's Seal Charm | Hollowhearth | [Systems](06-progression-and-systems.md#npcs-and-questlines) |
| `projects` | Relics | Placed in the world | [Relics](#relics) |
| `resume` | The Wayfarer's Ledger key item; the Ledger Stand | Inventory from the start; Hollowhearth | [Résumé](#résumé) |
| `blog` | Letters on the Archive shelves | Hollowhearth | [Letters](#letters) |
| `contact` | The Courier's **How to reach the Wayfarer** menu; the Courier's Letter after B12 | Hollowhearth; the ending | [Contact](#contact) |

### Stable slugs

`content.ts` items have no stable IDs today; they're identified by title. [WP-008](10-roadmap.md#wp-008)
adds an optional `slug` field to `LootItem` (for example `banana-cpp`) and fills it for
every existing item. `game/data/portfolio.ron` references items only by section and
slug. The content validator fails if a referenced slug doesn't exist.

## Chronicle Pages

Work entries become Chronicle Pages. The world has five page slots, in this order:
Ninefold Chapel (Verge), Stillwater Village (Mirrormere), the Castellan's desk (Castle),
the Frozen Foundry (Rimeshelf), and the top of the Spire.

The following rules decide which entry goes where:

1. The current role (the entry with `badge: 'current'`, or the newest) always goes to
   the Spire.
2. The remaining entries fill the other slots from oldest to newest in slot order.
3. Unused slots spawn nothing. If there are more than five entries, the oldest entries
   share the Ninefold Chapel slot as one bound bundle.

With today's four entries, the result is as follows: the internship at Ninefold Chapel,
the second role at Stillwater Village, the third at the Castellan's desk, and the
current role at the top of the Spire.

Each page shows the entry's title, subtitle, body, and links in the plain facts panel,
under one line of in-world framing written in `portfolio.ron`.

## Relics

Each project becomes a relic: a real, useful item whose description is the project.
The following table maps today's projects:

| Project slug | Relic | Item type | Why it fits |
|---|---|---|---|
| `politellama` | PoliteLlama | Echo summon | A courteous llama spirit that bows before each attack, and fights harder if you bow to it first |
| `banana-cpp` | banana.cpp | Curved sword | Fast, light, no dependencies: a blade honed for speed |
| `smol-llama` | smol-llama | Charm | Small but trained hard: equipped weapons weigh 25% less |
| `vicharak-micro-llama` | Shrike-Lite Circuit | Charm | Bare-metal speed: spells start 12% faster |
| `smoltorch` | smoltorch | Key tool (light source) | A small flame that lights the dark in about 500 lines |
| `nopokedb` | NoPokeDB | Medium shield | A vault that keeps what it holds: immune to Pierce chip damage |
| `boo` | Boo | Head armor (mask) | A friendly ghost's mask: enemies notice you later |
| `ferray` | ferray | Spear | Iron and rays: a Storm-infused spear |
| `endark` | endark | Body armor (cloak) | Dark-only, zero dependencies: hides you in darkness |
| `opencode-theme` | Opencode | Dye | Recolors armor and the HUD accent |

### Relic descriptions

A relic's in-game description has three parts:

1. **One fantasy sentence** about the item, written in `portfolio.ron`.
2. **One sentence that bridges** to the project, also in `portfolio.ron`.
3. **The facts panel**, generated from `content.ts`: title, body, badge, and links.

The following example shows the banana.cpp relic:

> **banana.cpp** (curved sword, relic)
>
> *A crescent blade forged without a single borrowed part, so light it seems to
> answer before it's asked.*
>
> *Its smith built every piece by hand: the edge, the grip, the way it turns.*
>
> ---
> **banana.cpp** · active
> Pure C++ LLM inference engine. SmolLM2, Llama 3.2, Qwen. Modular architecture with
> GQA, RoPE, SwiGLU.
> [Code](https://github.com/kashifulhaque/banana.cpp)

### Fallbacks

A project with no entry in `portfolio.ron` becomes a **Wayfarer's Keepsake**: a generic
key item placed on the Archive's keepsake table in Hollowhearth, showing only the facts
panel. The validator warns, but doesn't fail, so content updates never block a deploy.

## Résumé

- The Wayfarer's Ledger is in the inventory from the first tick. Its **Read** action
  opens the résumé (`resumePath` from `content.ts`) in a new tab.
- The Ledger Stand in Hollowhearth does the same.
- Opening it grants the **Reader** Mark.

## Letters

The blog is hosted externally. The Archive shelves hold one **Letter** that links to the
blog, from the `blog` section of `content.ts`. A later WP can add one Letter per post by
fetching the blog's feed through the Worker and caching it in KV, using the same
pattern as the existing HN cache. Until then, there's one Letter.

## Contact

- The Courier's **How to reach the Wayfarer** menu lists every `contact` entry with its
  links, from the first conversation.
- After B12, the ending hands the player the Courier's Letter, which shows the same
  links.
- The landing card's **Contact** button opens the same list as a DOM panel.

## Codex

The codex is both an in-game menu and a set of plain pages.

### Routes

| Route | Content |
|---|---|
| `/codex` | Index: portfolio sections first, then the bestiary, arsenal, places, and people |
| `/codex/work/SLUG` | A Chronicle Page with its facts panel |
| `/codex/projects/SLUG` | A relic with its item card and facts panel |
| `/codex/bosses/ID` | A boss (title only until defeated) |
| `/codex/places/ID` | A region or dungeon (title only until discovered) |

- Codex pages are prerendered (`export const prerender = true`) so crawlers and screen
  readers get full HTML.
- Portfolio pages (`work`, `projects`, education, contact, résumé) always show their
  full content. Game-lore pages show only titles until the visitor discovers them in
  the game.
- Each portfolio codex page shows the item's in-game card (icon, name, stats) and,
  if the visitor found it, a **Visit in the world** link that opens
  `/game?lantern=LANTERN_ID`. That public parameter only starts the player at a Lantern
  their save has already lit; otherwise the game ignores it.
- Codex unlocks come from `src/lib/latticefall/codex/`, which has no Three.js or WASM
  import.

## Home page

The home page reads the old game's progress from `src/lib/game/progress.ts` today.
[WP-086](10-roadmap.md#wp-086) replaces that with the codex store: the home page shows
"N of M relics found" and links to `/game` or `/codex`. Don't delete the old game until
that WP.

## Tone

- Fantasy lines: one or two short sentences, quiet and specific, no "epic" or
  "legendary," and never a pun that hides the fact.
- Facts: exactly as written in `content.ts`. The game never rewrites them.
- ML winks are allowed in flavor text at a rate of about one in five items (see
  [the vision](00-vision-and-scope.md#world-premise)), and never in facts panels.

## SEO and metadata

- `/game` and `/codex` pages have their own titles and descriptions that describe the
  owner and the portfolio first, the game second. For example: "Kashiful Haque: ML
  engineer. Portfolio, and a small action RPG."
- No metadata mentions other games or their publishers.
- Social cards use a still from photo mode with the site name.
