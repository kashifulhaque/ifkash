# ifkash.dev design system

The site has three designs: Almanac (default), Japandi, and Ember. Each design
has its own layout (shell and home page), tokens, fonts, and graphics. All
designs share one set of token names, defined in `src/app.css`.

The **Design** dropdown in the top bar saves the choice in `localStorage`
under `ifkash-design`. An inline script in `app.html` applies it before paint
as `data-design` on `<html>` (`almanac`, `japandi`, or `ember`). Without a
saved choice, or with a saved id that no longer exists, the site uses
Almanac. Light and dark are a separate choice (`ifkash-theme`, `data-theme`).

## Folder architecture

Each design lives in its own folder. A design's files don't import from
another design's folder.

```text
src/lib/designs/
  shared.ts          Nav items, footer data, home data, DesignMeta type
  metas.ts           Imports every meta.ts. Design ids, DESIGNS list, isDesign
  registry.ts        Maps each design id to { meta, Shell, Home }
  active.ts          Store for the design that renders (waits for hydration)
  clock.ts           Store for the Bangalore time line
  relics.ts          Relic badge from the Cinderheart codex
  ThemeToggle.svelte Day and night button with the theme logic
  almanac/  meta.ts  Shell.svelte  Home.svelte  design.css
  japandi/  meta.ts  Shell.svelte  Home.svelte  design.css
  ember/    meta.ts  Shell.svelte  Home.svelte  design.css
```

`src/routes/+layout.svelte` renders `registry[design].Shell` around every page.
`src/routes/+page.svelte` renders `registry[design].Home`. Other pages render
inside the shell's slot and use the global classes and tokens.

### Design folder contract

A design folder has four files:

- `meta.ts` exports `meta: DesignMeta`: `id`, `label`, `themeColors` (the
  `<meta name="theme-color">` hex for light and dark), and `themeCopy` (the
  theme toggle labels).
- `Shell.svelte` takes the prop `fullWidth: boolean` and has a default
  `<slot />`. When `fullWidth` is true, render only the slot, with no chrome.
  Otherwise render the top bar, navigation, a content wrapper, and a footer.
  The content wrapper must have `id="main"` (the layout animates it on
  navigation, and the skip link targets it). The shell must not render
  `AiAgent`; the layout does.
- `Home.svelte` takes no props and renders the home page content. The route
  provides the `<svelte:head>` title and description.
- `design.css` holds the design's tokens and global-class overrides.

Rules for `design.css`:

- Override tokens under `[data-design="<id>"]` (light) and
  `[data-design="<id>"][data-theme="dark"]` (dark). Repeat every overridden
  token in the dark block, or the light value leaks into dark mode.
- Scope every other rule with `html[data-design="<id>"]`.
- Import the design's fonts at the top of this file, for example
  `@import "@fontsource-variable/cinzel";`. Don't import design fonts in
  `app.css`.
- Use the canonical token names. Don't add tokens that other designs need to
  know about.

Shells can reuse `ThemeToggle.svelte` (accepts `class` and a slot with `night`
and `choice`), `DesignSwitch.svelte` from `$lib/components`, the `clock`
store, `readRelicBadge()`, and the data in `shared.ts`. Shells keep
`WeatherHeadsUp` and `CommandDeck` in their own top bar.

### Add a design

1. Copy a design folder to `src/lib/designs/<id>/` and edit `meta.ts`.
2. Add the `meta` to `DESIGN_LIST` and `DESIGN_METAS` in `metas.ts`.
3. Add the Shell and Home to `registry.ts`.
4. Import `design.css` in `src/routes/+layout.svelte`, after `app.css`.
5. Add the id and colors to the `colors` table in `src/app.html`. Keep it in
   sync with `meta.ts`.
6. Run `bun run check` and `bun run build`.

### Avoid a flash of the wrong shell

The server renders the Almanac shell. If the saved design isn't Almanac,
`app.html` sets `data-shell-pending` on `<html>`, and `app.css` hides the
body. The root layout switches to the saved design's shell after mount and
removes the attribute. A 1500 ms timeout in `app.html` removes it too. Changing
the design in the dropdown remounts the shell and the current page.

## Tokens

These are the only color, type, and space tokens. The Almanac values are the
base set in `:root` and `[data-theme="dark"]`. Other designs override them.

| Role | Token | Notes |
|---|---|---|
| Page background | `--bg` | Warm paper in Almanac. |
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
| Signature | `--falu` | The `.dot` mark: one small dot per view. |
| Status | `--danger`, `--success` | Errors and confirmations. |
| States | `--hover-wash`, `--press-wash`, `--scrim` | Hover fills, pressed fills, modal backdrops. |
| Code | `--code-surface` | |
| Popover shadow | `--shadow-color` | Only inside the one allowed popover shadow. |

Charts: series 1 `--fjord`, series 2 `--moss`, series 3 `--clay`, series 4
`--ink-3`. Grid lines use `--line`, axis text uses `--ink-3` in `--font-mono`.

### Type

Each design picks its own fonts through `--font-sans`, `--font-mono`, and
`--font-display`. The base values are Almanac's:

- `--font-sans`: Hanken Grotesk. UI and body text.
- `--font-display`: Fraunces. Headlines.
- `--font-mono`: IBM Plex Mono. Labels, index numbers, metadata, code, and
  numbers in data tables. Use `font-variant-numeric: tabular-nums`.
- `--font-serif`: Newsreader. Game HUD and admin notes.

Scale: `--text-xs` 12, `--text-sm` 13, `--text-base` 16, `--text-md` 18,
`--text-lg` 22, `--text-xl` 28, `--text-2xl` 32-44, `--text-3xl` 40-76.

### Space, shape, motion

- Space: `--s-1` 4 through `--s-10` 128 (4, 8, 12, 16, 24, 32, 48, 64, 96, 128).
- Radius: `--radius-xs`, `--radius-sm`, `--radius-md`, and `--radius-pill`.
- Motion: `--ease`, `--ease-out`, `--dur-fast`, `--dur`, and `--dur-slow`.
  Respect `prefers-reduced-motion`.

## Global classes

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
| `.dot` | The signature dot. |
| `.led` | Moss status dot (`.led-hollow` for inactive). |
| `.stagger` + `style="--i: n"` | Staggered rise-in on list items. |
| `.serif` | Italic accent in the display face. |

## Almanac

Almanac is the default. It reads like a printed almanac or a field guide:
ink on warm paper, light Fraunces headlines, Hanken Grotesk body text, and
IBM Plex Mono marginalia. Eyebrows and labels are mono, uppercase, and
tracked at `0.09em`. The wordmark and emphasized phrases are Fraunces
italic. The accent (the `--fjord` slot) and the signature dot are oxblood.
Sage fills `--moss`, and ochre ink fills `--clay`.

The Almanac tokens are the base set in `src/app.css`: paper in `:root` and
lamplight in `[data-theme="dark"]`. Radii are `2`, `4`, and `6`, and motion
uses an ease-out cubic at `150`, `300`, and `600` ms. `almanac/design.css`
restyles headlines, kickers, and the grain. The top bar, footer, and
wordmark styles live in `almanac/Shell.svelte`.

The principles for Almanac:

- **Function first.** Every element earns its place. Use one primary action per
  view and plain-text navigation.
- **Lagom.** Use one accent at a time, at most two font weights per component,
  and 1px hairlines instead of shadows.
- **Craft.** Align to the 4px grid. Use tabular numerals for numbers and dates
  and a 64ch reading measure.
- **Calm motion.** Animate opacity and up to 8px of translation only.

Almanac only:

- No gradients, glows, glassmorphism, neon colors, or drop shadows on cards.
- No `box-shadow` except focus rings and a single popover shadow
  `0 12px 32px -12px var(--shadow-color)`.
- No pure `#000` or `#fff` backgrounds outside media.

## Japandi

Japandi is a placeholder. It has a token block (washi paper, sumi ink, indigo
and clay accents, charcoal in the dark theme) and a copy of the Almanac shell
and home. It is meant to be redesigned inside `src/lib/designs/japandi/`.
Fonts planned: Shippori Mincho and Zen Kaku Gothic New.

## Ember

Ember is a placeholder. It has a token block (near-black umber with a gold
accent, parchment in the light theme) and a copy of the Almanac shell and
home. It is meant to be redesigned inside `src/lib/designs/ember/`. Fonts
planned: Cinzel and Cormorant Garamond.

## Page patterns

- **Shell.** The current design's `Shell.svelte` provides the top bar, the content
  column, and the footer. Pages render only their content; don't add outer padding
  or a max-width wrapper. The patterns below describe the Almanac layout. Other
  designs can arrange the same content differently.
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

## Rules for every design

- Don't hardcode hex colors in page styles. If a color is missing, add a token
  to `app.css` and define it in every design.
- No custom cursors.
- Page templates can differ between designs. A page uses the global classes and
  tokens so that every shell can restyle it.
