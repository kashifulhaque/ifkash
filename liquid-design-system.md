# Liquid AI — Look & Feel Specification

A portable design-system spec reverse-engineered from the live tokens and computed styles on
[liquid.ai](https://www.liquid.ai/) (captured 2026-08-08). Every value below was read off the running
page — CSS custom properties, `getComputedStyle()` on real elements, and stylesheet rule extraction —
at **six viewport widths: 375, 640, 768, 1024, 1280, 1440**.

Fonts have been re-specified as **100% open / free** (see §7). The one proprietary face in the original
(Iowan Old Style, an Apple system font) has an empirically-chosen OFL replacement.

Pair this with [`liquid-theme.css`](liquid-theme.css) — a drop-in stylesheet implementing everything here,
mobile behaviour included.

---

## 1. The idea in one paragraph

A near-black editorial canvas. Structure comes from **hairline rules**, not from cards, shadows, or
rounded boxes — content sits inside a ruled grid like a broadsheet newspaper. Type carries the
personality: a **high-contrast old-style serif** for every headline (italic for emphasis), a neutral
grotesque for body copy, and a **technical uppercase face at 11px** for labels, metadata, and tickers.
Colour is almost entirely absent — a single **soft periwinkle-violet** does all the accent work, and it
glows rather than fills. The result reads as *research lab*, not *SaaS landing page*.

Four rules to keep the feel intact:

1. **Rules over boxes.** Separate things with 1px `rgba(255,255,255,0.13)` lines. No shadows, no card radii.
2. **Serif headlines, always,** at weight 400 with hard negative tracking. Italic is the emphasis mechanism.
3. **One accent, used sparingly.** `#ac8dff` on maybe two elements per viewport, plus a soft glow.
4. **The layout changes character at 1024px,** not gradually. See §5.

---

## 2. Colour

### Core palette

| Token | Value | Use |
|---|---|---|
| `--background` | `#0a0a0a` | Page canvas (neutral-950) |
| `--card` | `#09090b` | Slightly-cooler recessed surface (zinc-950) |
| `--popover` | `#18181b` | Dropdowns, menus (zinc-900) |
| `--muted-surface` | `#27272a` | Filled panels, image placeholders (zinc-800) |
| `--foreground` | `#fafafa` | Primary text, headlines |
| `--muted-foreground` | `#a1a1aa` | Body copy, labels, metadata (zinc-400) |

> **Body copy is `#a1a1aa`, not white.** This is the single biggest driver of the calm, low-contrast
> feel. White is reserved for headlines and links.

### Lines & edges

| Token | Value | Use |
|---|---|---|
| `--border` | `rgba(255,255,255,0.13)` | The default hairline — grid rules, rails, section dividers |
| `--border-strong` | `rgba(255,255,255,0.15)` | Hover / emphasis edges |
| `--input` | `rgba(255,255,255,0.08)` | Field borders, subtle inner edges, nav hover fill |
| `--rule` | `#27272a` | **Opaque** divider — used inside the mobile menu instead of `--border` |

That last one is a real distinction: translucent `--border` everywhere on the page, opaque `--rule`
inside the solid-background mobile menu (where a translucent line over a solid panel would read wrong).

### Accent

| Token | Value | Use |
|---|---|---|
| `--accent` | `#ac8dff` | The brand violet — primary button fill, italic headline emphasis, category labels |
| `--accent-2` | `#d4c0ff` | Lighter tint — focus rings, hover lift |
| `--accent-wash` | `#2a1d4a` | Deep muted violet for large tinted areas |
| `--accent-orchid` | `#cd82f0` | Pink-leaning secondary accent, gradient artwork only |
| `--destructive` | `#ef4444` | Errors |

### Gradients & glows

```css
--gradient-purple-base:   linear-gradient(90deg, #5b1ccc 0%, #7c3aed 50%, #9866f6 100%);
--gradient-purple-lifted: linear-gradient(90deg, #ac8dff 0%, #c0a6ff 50%, #d4c0ff 100%);
--gradient-accent:        var(--gradient-purple-lifted);  /* the on-dark default */

--accent-glow:      0 0 20px 2px #7c3aed40;                 /* box-shadow  */
--accent-text-glow: 0 0 22px #7c3aed33, 0 0 44px #7c3aed1c; /* text-shadow */
```

The **lifted** gradient is the on-dark default (readable against `#0a0a0a`); the **base** gradient is for
light or inverted surfaces. Full-bleed gradient bands — a soft violet→lilac wash occupying a whole
section — are the site's one moment of saturated colour, used to break up long runs of black.

---

## 3. Typography

Three families, three jobs. This split is non-negotiable if you want the look. **All three are SIL Open
Font License** and free to self-host or serve from Google Fonts.

```css
--font-serif: Newsreader, Georgia, "Times New Roman", ui-serif, serif;
--font-sans:  Inter, ui-sans-serif, system-ui, -apple-system, sans-serif;
--font-label: "Chakra Petch", ui-monospace, "SF Mono", Menlo, monospace;
```

- **Serif — Newsreader.** Every heading, plus hero lead-ins and mobile menu items. Always weight **400** —
  never bold a headline. See §7 for why this font and how it was chosen.
- **Sans — Inter.** Body copy, links, buttons, navigation. Weights 400/500/600/700.
- **Label — Chakra Petch.** Uppercase micro-type only: eyebrows, tickers, metadata, footer headings,
  stat labels, dates, categories. Never for sentences.

### Scale, and how it responds

Only **h1 and h2 scale**. Body, labels, and h3 are fixed at every width — worth internalising, because
it means you do not need a fluid type system to reproduce this.

| Role | Family | < 768 | 768–1023 | ≥ 1024 | Tracking | Weight | Colour |
|---|---|---|---|---|---|---|---|
| Display (h1) | serif | `48px / 1.0` | `60px / 1.0` | `60px / 1.0` | `-0.05em` | 400 | `--foreground` |
| Section head (h2) | serif | `28px / 1.2` | `28px / 1.2` | `32px / 1.2` | `-0.023em` | 400 | `--foreground` |
| Card head (h3) | serif | `24px / 1.2` | `24px / 1.2` | `24px / 1.2` | `-0.025em` | 400 | `--foreground` |
| Menu item (mobile) | serif | `20px / 28px` | — | — | normal | 400 | `--foreground` |
| Lead paragraph | serif | `20px / 26px` | same | same | normal | 400 | `--foreground` |
| Body | sans | `16px / 1.6` | same | same | normal | 400 | `--muted-foreground` |
| Small | sans | `13px / 1.6` | same | same | normal | 400 | `--muted-foreground` |
| Caption | sans | `12px / 1.43` | same | same | normal | 400 | `--muted-foreground` |
| **Label / eyebrow** | label | `11px / 18px` | same | same | `0.1em` | **500** | `--muted-foreground` |

Notes on the numbers:

- **h1 tracking is proportional, not fixed** — measured `-3px` at 60px and `-2.4px` at 48px, i.e. `-0.05em`
  at both. Express it in `em` and it survives any resize.
- **h1 line-height is exactly 1.0** at every width. Dense, editorial, slightly uncomfortable — deliberately.
- **The 11px label never changes size.** It's 11px at 375 and at 1440. Same tracking, same weight.
- The h1 step happens at **768** (md); the h2 step at **1024** (lg). They are not the same breakpoint.

### The headline emphasis pattern

The signature move: one word inside the h1 set in **serif italic, accent-coloured, with a soft glow**.

```html
<h1 class="display">
  <span class="block"><em class="headline-italic accent-em">Device</em><em class="headline-italic">-native</em></span>
  <span class="block">foundation models.</span>
</h1>
```

The italic carries its own optical correction — this is lifted verbatim from the site:

```css
.headline-italic { font-style: italic; font-size: 1.05em; letter-spacing: -0.03em; }
.accent-em       { color: var(--accent); text-shadow: var(--accent-text-glow); }
```

**The italic is scaled up 5% and tracked looser than the roman** (`-0.03em` against the headline's
`-0.05em`). Italics optically shrink and their sloped forms need more room; without this correction the
emphasis reads smaller and cramped. It's the kind of detail that separates a copy from the real thing.

Also note that in `Device-native`, *both* words are italic — the italic spans a whole phrase, while the
colour and glow pick out just one word inside it. And headlines break onto explicit lines via
`display:block` spans, not by letting the browser wrap.

---

## 4. Space & radius

### Radius

```css
--radius-sm:   4px;    /* header/nav controls, the mobile menu's full-width CTA */
--radius-md:   6px;    /* icon buttons — hamburger, close */
--radius-lg:  12px;    /* rare — large media containers */
--radius-pill: 9999px; /* all body CTAs */
```

**Radius discipline matters.** Content containers are **square** — 0px radius everywhere. Adding rounded
cards is the fastest way to lose the look.

### The spacing scale steps once, at `lg` (1024px)

Every spacing token has a compact value below 1024 and a roomy value at/above it. There is no gradual
interpolation — it is a single hard step. `--spacing-page-x` steps a second time at `xl` (1280).

| Token | < 1024 | ≥ 1024 (lg) | ≥ 1280 (xl) |
|---|---|---|---|
| `--spacing-page-x` | `0` | `50px` | `128px` |
| `--spacing-frame-gutter` | `16px` | `32px` | `32px` |
| **effective page padding** | **`16px`** | **`82px`** | **`160px`** |
| `--spacing-content-sm` | `16px` | `24px` | `24px` |
| `--spacing-content-lg` | `32px` | `48px` | `48px` |
| `--spacing-content-xl` | `32px` | `64px` | `64px` |
| `--spacing-section` | `96px` | `128px` | `128px` |
| `--section-gap-sm` | `64px` | `80px` | `80px` |
| `--section-gap-md` | `80px` | `96px` | `96px` |
| `--section-gap-lg` | `96px` | `128px` | `128px` |
| `--spacing-hero-top` | `130px` | `150px` | `150px` |
| `--page-max-w` | `1440px` | `1440px` | `1440px` |

Measured section padding follows from this: **`64px` top / `96px` bottom below lg**, **`80px` / `128px` at lg+**.
Note the asymmetry — bottom padding is always larger than top.

### Breakpoints

Tailwind v4 defaults, plus two customs:

| Name | Width | What changes |
|---|---|---|
| `sm` | 640px | minor |
| `md` | 768px | **h1 48→60px**; hero CTAs stop stacking; stat grid 2→3 cols |
| `lg` | 1024px | **the big one** — whole spacing scale steps up; ruled grids become grids; 12-col grid activates; h2 28→32px |
| `xl` | 1280px | **hamburger → full horizontal nav**; page-x 50→128px; header bottom border removed |
| `2xl` | 1536px | minor |
| custom | 480px, 896px | isolated tweaks |
| custom | `max-width: 767px` | marquee slows down |

---

## 5. Layout

### The ruled grid — and how it collapses

The core layout device, and the single most important responsive behaviour on the site:

```html
<div class="ruled-grid"> <!-- flex column below lg, 3-col grid at lg+ -->
  <div>…</div><div>…</div><div>…</div>
</div>
```

- **Below 1024:** a **flex column** with **horizontal** hairline dividers between items (`divide-y`).
  Each cell keeps its `32px` padding. Vertical rules do not exist here.
- **At 1024+:** becomes a **3-column grid** and the horizontal dividers are switched **off**
  (`lg:divide-y-0`). The vertical separation is then supplied by the rail overlay below, not by borders.

This flip — horizontal dividers on mobile, vertical rails on desktop — is what lets the same content read
as a ruled grid at both ends. Getting it wrong (e.g. keeping vertical borders and letting them collapse
into a single column) is the most common way this design fails on a phone.

A second, simpler variant exists for looser content: `grid-cols-1 gap-12` → `lg:grid-cols-3 lg:gap-16`,
i.e. pure whitespace separation with **no dividers at all**, gaps stepping **48px → 64px**.

### Vertical rails

The vertical lines are **not borders**. They are absolutely-positioned 1px divs
(`position:absolute; width:1px; background:var(--border)`) spanning a section's full height. Measured
left offsets:

| Viewport | Rail positions | Count |
|---|---|---|
| 375 | `16, 358` | 2 — content edges only |
| 1024 | `0, 50, 358, 666, 973` | 5 — frame + 3 interior |
| 1280 | `0, 128, 469, 811, 1151` | 5 |
| 1440 | `0, 128, 523, 917, 1311` | 5 |

So: on mobile just the two frame edges; from `lg` a full four-column rail grid. Because they're
positioned elements rather than borders, they run the whole height of a section independent of the
content inside it — which is exactly what makes it look like printed grid paper rather than a stack of
bordered divs.

### Page grid

A **12-column** grid with `32px` column gap and `64px` row gap activates at `lg`. Below that everything
is a single column. Max width `1440px`, centred.

### Article / list rows

| | ≥ lg | < lg |
|---|---|---|
| Structure | CSS grid | flex column |
| Template | `160px minmax(0,1fr) auto` — or `160px 160px minmax(0,1fr) auto` with a category | — |
| Date | fixed 160px lead column | `display:none`; a duplicate date renders inline above the title |
| Category | own 160px column | inline, next to the date |
| Title | serif, fills remaining space | serif, on its own line below |
| Divider | full-bleed 1px `--border` | same |
| Row height | ~90px | 113–137px |

Rendered desktop: `08.05 · NEWS · Headline… · →`. Rendered mobile:

```
08.05  NEWS
MacPaw Partners with Liquid AI to
Bring On-Device AI to Millions…
```

**The category label is accent-coloured** (`#ac8dff`, 11px uppercase) while the date stays
`--muted-foreground`. One of the few places the accent appears in body content.

---

## 6. Components

### Header / nav

Sticky, `top: 0`, `z-index: 50`, height **69px**. It is **frosted, not transparent** — the effect comes
from a pseudo-element sitting behind the content:

```css
.site-header { position: sticky; top: 0; z-index: 50; isolation: isolate;
               border-bottom: 1px solid var(--border); }
.site-header::before {
  content: ""; position: absolute; inset: 0; z-index: -10;
  background: rgb(10 10 10 / 0.8);
  backdrop-filter: blur(20px);
}
@media (min-width: 1280px) { .site-header { border-bottom: 0; } }
```

Over the black hero this reads as fully transparent; over a gradient band it reads as smoked glass.
At `xl` the bottom border is **removed** — the frame rails take over that job.

**Navigation switches at `xl` (1280px), not at `lg`.** Tablets and small laptops get the hamburger.

- **≥1280:** horizontal links, 14px Inter, `#fafafa`, 36px tall, `0 16px`, `4px` radius (radius only visible
  on hover fill). Solid white pill CTA at the right.
- **<1280:** a 36×36 icon button, `6px` radius, `aria-label="Open menu"` / `"Close menu"`.

### Mobile menu — the biggest departure from desktop

This is where the design does something genuinely different rather than just reflowing:

```css
.mobile-menu {
  position: fixed;
  inset: 106px 0 0;            /* below ticker (38px) + header (69px) */
  background: var(--background); /* solid — no blur, no scrim */
  z-index: 40;
  overflow-y: auto;
}
```

| Element | Spec |
|---|---|
| Top-level item | **serif 20px / 28px, weight 400, normal tracking** — not the 14px Inter of desktop |
| Row | full-bleed, 60px tall, `16px` padding, hover → underline |
| Divider | 1px **`--rule`** (`#27272a`, opaque) — not `--border` |
| Expandable | 20×20 chevron in `--muted-foreground`, rotates on open |
| Submenu item | Inter 16px, `8px 16px` padding, ~37px tall |
| Footer CTA | **full-width**, `#fafafa` bg, `#0a0a0a` text, `4px` radius, 43px tall |

Promoting nav items to 20px serif is a deliberate inversion: on desktop the serif is reserved for
content and the nav is quiet sans; on mobile, with the menu occupying the whole screen, the nav *becomes*
the content and gets the editorial treatment. Copy this — it's most of why the mobile menu feels designed
rather than generated.

### Ticker / marquee

Full-width scrolling strip above the header. **38px** tall, `8px 0` padding, 11px Chakra Petch uppercase
in `--muted-foreground`, items separated by a small `▪` glyph.

```css
--mask-marquee: linear-gradient(90deg, transparent, black 64px, black calc(100% - 64px), transparent);

[data-marquee-track] { animation: marquee var(--marquee-duration, 40s) linear infinite; }
@media (max-width: 767px) {
  [data-marquee-track] { animation-duration: var(--marquee-duration-mobile, 160s); }
}
@media (any-hover: hover) { .group\/marquee:hover [data-marquee-track] { animation-play-state: paused; } }
@media (prefers-reduced-motion: reduce) { [data-marquee-track] { animation: none !important; } }
```

**The marquee slows dramatically on small screens** — measured 160s under 768px against 120s above it
(the CSS default var is 40s; instances override it). Same distance, far less speed: on a narrow screen a
fast marquee is unreadable and nauseating. Note also the hover-pause is gated behind `any-hover: hover`
so it doesn't misfire on touch, and the whole thing is disabled under `prefers-reduced-motion`.

### Buttons

| Variant | Fill | Text | Radius | Padding / height |
|---|---|---|---|---|
| **Primary** | `#ac8dff` | `#0a0a0a` | pill | `10px 16px` / 41px |
| **Ghost** | transparent + gradient ring | gradient text | pill | `10px 16px` / 41px |
| **Header CTA** | `#fafafa` | `#0a0a0a` | `4px` | `0 16px` / 36px |
| **Outline** | `#09090b` + 1px `--border` | `#fafafa` | `4px` | `0 16px` / 36px |
| **Mobile menu CTA** | `#fafafa` | `#0a0a0a` | `4px` | full-width / 43px |

All are **weight 400, 14px, sentence case, no letter-spacing.** No uppercase CTAs anywhere — the
uppercase treatment belongs exclusively to 11px labels. Body CTAs **never go full-width**; they keep
their intrinsic width and simply stack below 768px.

The ghost button is more interesting than it looks — both its text *and* its border are gradient-filled:

```css
/* gradient-filled label */
.btn-gradient-text {
  background-image: var(--btn-text, var(--gradient-accent));
  background-clip: text;
  -webkit-text-fill-color: transparent;
  color: transparent;
}
/* 1px gradient border ring, via mask-composite */
.btn-gradient-ring { position: relative; }
.btn-gradient-ring::after {
  content: ""; position: absolute; inset: 0; padding: 1px;
  border-radius: inherit; pointer-events: none;
  background: var(--btn-ring, var(--gradient-accent));
  mask: linear-gradient(#fff 0 0) content-box exclude, linear-gradient(#fff 0 0);
}
```

That mask trick paints the gradient only in the 1px padding band — a gradient border without a wrapper
element. It's why the "Connect with us" button reads violet on mobile screenshots while its computed
`color` is `#fafafa`.

### Transitions

```css
--ease-out:   cubic-bezier(0, 0, 0.2, 1);
--ease-inout: cubic-bezier(0.4, 0, 0.2, 1);
```

- Links & nav: `0.15s var(--ease-inout)` on `color`, `background-color`
- Buttons: `0.3s var(--ease-out)` on `color`, `background-color`

Fast, colour-only, no transforms or scale. Motion is restrained to the marquee and gradient artwork.

### Footer

| | Desktop | Mobile |
|---|---|---|
| Columns | 4–5 link columns | single column, `64px` row gap |
| Column heading | 11px uppercase label, `--muted-foreground` | same |
| Links | Inter 16px, `#fafafa` | same |
| Bottom bar | copyright left, legal links right | stacked, copyright centred |

Also carries a brand mark, a one-line mission statement in body grey, an `EST. 2023` label, a row of
monochrome social glyphs, and an underline-only email subscribe field (transparent background, no radius,
no box).

### Accessibility notes from the audit

Two things worth fixing rather than faithfully copying:

- **Ticker items measure 22px tall** — under the 44px touch-target guideline. They're links. Either pad
  them out or make them non-interactive.
- **Body CTAs are 41px** — just under 44px. Bumping padding to `12px 16px` gets you to 45px with no
  visual cost.

The mobile menu rows (60px) and header controls (36px, but with generous surrounding space) are fine.

---

## 7. Fonts — the open replacement

The original uses **Iowan Old Style**, an Apple system font. It cannot be webfont-served, so on Windows
and Android the original site already falls back to something else. For a portable system it has to be
replaced outright.

### How the replacement was chosen

I rendered 16 free serif candidates against Iowan Old Style in the browser and measured each with canvas
`TextMetrics` at 100px — x-height/cap-height ratio (perceived size) and the advance width of the full
headline string (whether line breaks hold).

| Font | x-height / cap | Δ vs Iowan | Line width | Δ width |
|---|---|---|---|---|
| **Iowan Old Style** *(target)* | 0.687 | — | 1496.8 | — |
| Libre Caslon Text | 0.688 | +0.001 | 1603.9 | +7.2% |
| **Petrona** | 0.691 | +0.004 | 1456.8 | −2.7% |
| **Spectral** | 0.682 | −0.005 | 1487.3 | −0.6% |
| **Vollkorn** | 0.677 | −0.010 | 1476.8 | −1.3% |
| Source Serif 4 | 0.675 | −0.012 | 1358.1 | −9.3% |
| **Gelasio** | 0.700 | +0.013 | 1487.1 | −0.6% |
| Literata | 0.703 | +0.016 | 1555.7 | +3.9% |
| Alegreya | 0.706 | +0.019 | 1382.0 | −7.7% |
| Lora | 0.714 | +0.027 | 1566.7 | +4.7% |
| PT Serif | 0.714 | +0.027 | 1503.7 | +0.5% |
| **Newsreader** | 0.716 | +0.029 | 1502.0 | **+0.3%** |
| Crimson Pro | 0.733 | +0.046 | 1352.5 | −9.6% |
| Noto Serif | 0.751 | +0.064 | 1599.1 | +6.8% |
| EB Garamond | 0.620 | −0.067 | 1285.9 | −14.1% |
| Faustina | 0.762 | +0.075 | 1397.4 | −6.6% |
| Bitter | 0.762 | +0.075 | 1562.1 | +4.4% |

### Verdict: **Newsreader** (SIL OFL, Production Type)

```css
--font-serif: Newsreader, Georgia, "Times New Roman", ui-serif, serif;
```

- **Width match is near-exact (+0.3%)** — headlines break in the same places, so the hero layout holds
  without retuning.
- Visually the closest of the set: same Venetian old-style skeleton, same moderate stroke contrast, same
  sturdy bracketed serifs. Its true italic is calligraphic and slightly more sloped than Iowan's, which
  actually *helps* the accent-word treatment in §3.
- It is what the original site itself designates as its non-Apple fallback — so this isn't a guess about
  what the designers would accept, it's what they already chose.
- Drawn specifically for screen reading, with a real italic (not an oblique) and a wide weight range.

**One adjustment.** Newsreader's x-height is `0.512em` against Iowan's `0.479em` — it reads **6.9% larger
at the same pixel size**. Two options:

- *Keep the scale as-is* (recommended). 60px stays 60px; the headline reads slightly more present. This is
  what the theme CSS does.
- *Match Iowan optically* — set the display size to **56px** where the original used 60px.

Newsreader ships an optical-size axis (`opsz 6..72`). In testing, manually setting
`font-variation-settings: 'opsz'` produced no measurable change, so rely on the default
`font-optical-sizing: auto` rather than driving the axis by hand.

### Alternates, if you want a different flavour

- **Spectral** — the best pure-metric match (−0.6% width, x-ratio within 0.005). Slightly higher contrast
  and a touch more literary. Use if you want the headlines more refined.
- **Vollkorn** — warmer, sturdier, a little quirkier. Use if the black canvas feels too austere.
- **Gelasio** — metric-compatible with Georgia, so it's the safest choice if you must degrade gracefully
  to a system font.

**Avoid** for this design: EB Garamond and Crimson Pro (x-height far too small — they look weak and lost
at 60px on black), Source Serif 4 (−9.3% width shifts every line break), Bitter and Faustina (x-height far
too large — the tight 1.0 line-height stops working).

### The complete open stack

| Role | Font | Licence | Why |
|---|---|---|---|
| Serif / display | **Newsreader** | SIL OFL 1.1 | Replaces Iowan Old Style; see above |
| Sans / body | **Inter** | SIL OFL 1.1 | Unchanged from the original |
| Label / micro | **Chakra Petch** | SIL OFL 1.1 | Unchanged; the 11px uppercase voice |
| Mono | **Geist Mono** | SIL OFL 1.1 | Declared in the original's tokens; unused on the homepage |

All four are on Google Fonts and all four may be self-hosted. Single import:

```html
<link href="https://fonts.googleapis.com/css2?family=Newsreader:ital,opsz,wght@0,6..72,400;1,6..72,400&family=Inter:wght@400;500;600;700&family=Chakra+Petch:wght@400;500;600&display=swap" rel="stylesheet">
```

Self-hosting is worth it here: three families plus an italic is a lot of network round-trips, and the
serif is above the fold in the hero.

---

## 8. Applying this to another site

**Minimum viable transplant** (gets ~80% of the feel):

1. Page background `#0a0a0a`, body text `#a1a1aa`, headings `#fafafa`.
2. Load Newsreader + Inter + Chakra Petch. Every heading → Newsreader 400, `-0.05em` tracking at display size.
3. Replace card borders and shadows with `1px solid rgba(255,255,255,0.13)` hairlines; container radius → 0.
4. Recolour CTAs to the `#ac8dff` pill / gradient-ring pill pair.
5. Eyebrows, metadata, table headers → 11px Chakra Petch uppercase, `0.1em` tracking.

**Then the four things that make it work on mobile:**

6. **Collapse ruled grids to horizontal dividers**, never to orphaned vertical borders (§5).
7. **Promote mobile nav items to 20px serif** on a solid full-screen panel with opaque `--rule` dividers (§6).
8. **Step the whole spacing scale at 1024px** — `16px` page padding below, `82px` above (§4).
9. **Slow any marquee to ~4× on small screens**, pause on `any-hover: hover` only, kill under reduced motion (§6).

**Things that will break the look:**

- Bold headlines (the serif is designed for 400 — bolding reads as a mistake)
- Rounded cards with drop shadows
- More than one accent hue on screen
- Body copy at full white
- Uppercase buttons, or letter-spaced body text
- Full-width stacked mobile CTAs (the real ones keep their intrinsic width)
- Scaling the 11px label down on mobile — it's fixed at every breakpoint
- A plain-transparent sticky header — the real one is `blur(20px)` over 80% background

**Licensing.** Newsreader, Inter, Chakra Petch, and Geist Mono are all SIL OFL and free for commercial
use, self-hosting, and modification. The design tokens, layout patterns, and type scale described here are
generic craft; the Liquid AI **name, logo, wordmark, and copy** are not, and shouldn't be carried across.

---

## 9. Provenance

Colour, spacing, radius, and gradient tokens were read directly from the site's `:root` custom properties
at each breakpoint. Typography, component dimensions, transitions, and responsive behaviour were read from
`getComputedStyle()` on live elements — h1/h2/h3, nav links, all button variants, ticker items, mobile menu
rows, article rows, footer headings — at 375, 640, 768, 1024, 1280, and 1440px. Component CSS
(`.btn-gradient-ring`, `.btn-gradient-text`, `.headline-italic`, marquee rules) was extracted from the live
stylesheet rather than inferred. Font metrics in §7 were measured in-browser with canvas `TextMetrics` at
100px against locally-installed Iowan Old Style.

Not covered: the `2xl` (1536px) breakpoint and the 480px/896px custom breakpoints were identified but their
specific effects were not isolated; interior pages (blog, model pages) were not sampled — this describes the
homepage system.
