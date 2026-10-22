# ifkash.dev design system

The site has two designs. Both read the tokens in `src/app.css`. Edit the
tokens to restyle a design; don't hardcode colors in pages.

The **Design** control in the top bar saves the choice in `localStorage`
under `ifkash-design`. An inline script in `app.html` applies it before
paint as `data-design` on `<html>` (`nordisk` or `broadsheet`). Light and
dark are a separate choice (`ifkash-theme`, `data-theme`).

## Nordisk

Nordisk is the default. It is a Scandinavian design language: functional,
light, warm, and restrained.

## Principles

- **Function first.** Every element earns its place. Use one primary action per
  view and plain-text navigation.
- **Daylight.** The default canvas is warm birch paper, never pure white. Dark
  mode ("polar night") is charcoal-blue, never pure black.
- **Lagom.** Use one accent at a time, at most two font weights per component
  (400 and 500; 600 only for `<strong>`), and 1px hairlines instead of shadows.
- **Craft.** Align to the 4px grid. Use tabular numerals for numbers and dates,
  negative tracking on display type, and a 64ch reading measure.
- **Calm motion.** Animate opacity and up to 8px of translation only, with
  140-480ms durations. Respect `prefers-reduced-motion`.

## Tokens

These are the only color, type, and space tokens. The pre-redesign names
(`--foreground`, `--blueprint`, `--panel`, and so on) no longer exist.

| Role | Token | Notes |
|---|---|---|
| Page background | `--bg` | Linen. |
| Raised surface | `--surface` | Cards, inputs, popovers. |
| Sunken surface | `--surface-2` | Wells, code, table headers, chart tracks. |
| Primary text | `--ink` | Headings, values, active items. |
| Secondary text | `--ink-2` | Body copy, descriptions. |
| Tertiary text | `--ink-3` | Labels, metadata, index numbers. AA at 12px+. |
| Text on ink fill | `--ink-inverse` | Inside `.btn-primary` and ink badges. |
| Hairline | `--line` | Borders and rules. Decorative only, never text. |
| Strong hairline | `--line-strong` | Input borders, hovered borders. |
| Accent | `--fjord`, `--fjord-strong`, `--fjord-wash` | Links on hover, focus, selected state, chart primary series. |
| Secondary hues | `--moss` (+ `--moss-wash`), `--clay` | Moss means "done", "live", or "current". Clay is a highlight for one chart series or warning. |
| Signature | `--falu` | Falu red. Only the `.dot` mark: one small dot per view. |
| Status | `--danger`, `--success` | Errors and confirmations. |
| States | `--hover-wash`, `--press-wash`, `--scrim` | Hover fills, pressed fills, modal backdrops. |
| Code | `--code-surface` | |
| Popover shadow | `--shadow-color` | Only inside the one allowed popover shadow. |

Charts: series 1 `--fjord`, series 2 `--moss`, series 3 `--clay`, series 4
`--ink-3`. Grid lines use `--line`, axis text uses `--ink-3` in `--font-mono`.

### Type

- `--font-sans`: Schibsted Grotesk (Norwegian, by Bakken & Bæck for Schibsted).
  All UI and headings. Weights 400 and 500.
- `--font-serif`: Newsreader italic. Use it sparingly for one emphasized word
  or phrase in a heading (`<em class="serif">`), or for pull quotes.
- `--font-mono`: JetBrains Mono. Labels, index numbers, metadata, code, and
  numbers in data tables. Use `font-variant-numeric: tabular-nums`.

Scale: `--text-xs` 12, `--text-sm` 13, `--text-base` 16, `--text-md` 18,
`--text-lg` 22, `--text-xl` 28, `--text-2xl` 32-44, `--text-3xl` 40-76.
Display type uses `letter-spacing: -0.04em` and `line-height: 1-1.05`. Labels
are sentence case in mono at 12px. Avoid uppercase except for 2-3 letter codes.

### Space, shape, motion

- Space: `--s-1` 4 through `--s-10` 128 (4, 8, 12, 16, 24, 32, 48, 64, 96, 128).
  Separate sections by `--s-9` on desktop and `--s-8` on mobile.
- Radius: `--radius-xs` 2 (tags, code), `--radius-sm` 4 (buttons, inputs, cards),
  `--radius-md` 6 (large panels), `--radius-pill` (dots, pills).
- No `box-shadow` except focus rings and a single popover shadow
  `0 12px 32px -12px var(--shadow-color)`.
- Motion: `--ease`, `--ease-out`, `--dur-fast` 140ms, `--dur` 220ms,
  `--dur-slow` 480ms.

## Global classes in `app.css`

| Class | Use |
|---|---|
| `.page-head` | Header block of a page: eyebrow, title, lead. Has bottom hairline. |
| `.eyebrow` | Mono 12px label, e.g. `<p class="eyebrow">02 / Projects</p>`. |
| `.page-title`, `.section-title` | Large display heading for a page. |
| `.lead`, `.section-subtitle` | Intro paragraph under a page title. |
| `.breadcrumb` | Mono breadcrumb on detail pages. |
| `.rows` | Hairline-ruled list (`<ol class="rows">`). |
| `.tag` | Mono pill with hairline. |
| `.card` | Flat surface with a hairline. |
| `.btn`, `.btn-primary`, `.btn--ghost`, `.btn-sm`, `.btn-row` | Buttons. Primary is an ink fill; default is a hairline outline. |
| `.link-arrow` | Text link with a trailing arrow. |
| `.field` | Text input, select, or textarea. |
| `.dot` | The Falu-red signature dot. |
| `.led` | Moss status dot (`.led-hollow` for inactive). |
| `.stagger` + `style="--i: n"` | Staggered rise-in on list items. |
| `.serif` | Newsreader italic accent. |

## Page patterns

- **Shell.** `+layout.svelte` provides a 64px top bar (wordmark, text nav, design
  control, theme toggle), the content column (`max-width: var(--content-max)`, side padding
  `var(--gutter)`), and the footer. Pages render only their content; don't add
  outer padding or a max-width wrapper.
- **Index pages** (work, projects, rl, tools, education, blog, roadmap): a
  `.page-head` with an eyebrow numbered like the home index (`01 / Work`), a
  `.page-title`, and a `.lead`. Then a ruled list with a 3-column row: mono index
  number, title and description, mono metadata on the right. On hover, the
  title turns `--fjord` and the arrow nudges 3px. No cards with fills.
- **Detail pages** (project write-ups, RL experiments): `.breadcrumb`, title,
  lead, link buttons, then a two-column layout on desktop: a sticky mono
  "facts" column (stack, status, links, dates) of 240px and a 64ch prose
  column. Section headings are `h2` at `--text-xl` with an optional mono
  section number. Tables use hairlines only, mono tabular numerals, and a
  `--surface-2` header row.
- **App pages** (fitness, splitter, PDF annotator, editor, admin): functional
  and dense but calm. Panels are `.card`-style with hairlines, toolbars use
  `.btn-sm`, and stat values use mono tabular numerals at `--text-xl`.
- **Empty and loading states:** one line of `--ink-3` text, optionally with a
  `.led-hollow`. No spinners with color; use a 1px indeterminate line if needed.

## Broadsheet

Broadsheet is the editorial alternative: newsprint, square corners, and
Newsreader headlines. UI chrome (buttons, navigation, forms) stays in the
sans. Kickers are uppercase and widely tracked. The masthead and the footer
use a double rule in `--ink`. The accent (the `--fjord` slot) is crimson.
The signature dot is a small square.

Day edition and night edition override the same color tokens under
`[data-design="broadsheet"]` and
`[data-design="broadsheet"][data-theme="dark"]`. Broadsheet also sets
`--font-display` to the serif stack and every `--radius-*` token to `0`.
Rules at the end of `app.css` restyle headlines and leads. The top bar and
the footer rules live in `+layout.svelte`, because scoped styles there
would otherwise win. Page templates stay the same in both designs.

## Don'ts

- No gradients, glows, glassmorphism, neon colors, or drop shadows on cards.
- No hand-drawn or "sketch" effects (the old `--r-frame-*`, `--squig-*`, and
  `sketch-boil-*` names are gone; remove any reference to them).
- No custom cursors.
- No pure `#000` or `#fff` backgrounds outside media.
- Don't hardcode hex colors in page styles. If a color is missing, add a token
  to `app.css`.
