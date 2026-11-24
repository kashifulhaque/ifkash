<script lang="ts">
  /**
   * Almanac shell: a printed masthead with a live dateline, a "Contents"
   * column for navigation, an editorial content column, a right gutter for
   * marginalia, and a colophon footer. Navigation turns the page with a short
   * clip-path wipe.
   */
  import { onMount, tick } from "svelte";
  import { page } from "$app/stores";
  import { prefersReducedMotion } from "$lib/motion";
  import CommandDeck from "$lib/components/CommandDeck.svelte";
  import WeatherHeadsUp from "$lib/components/WeatherHeadsUp.svelte";
  import DesignSwitch from "$lib/components/DesignSwitch.svelte";
  import ThemeToggle from "../ThemeToggle.svelte";
  import { clock } from "../clock";
  import { navItems, isActive, owner, blurb, coordinates, footerGroups, resumeUrl } from "../shared";
  import { dateline, roman, type Dateline } from "./astro";
  import MoonGlyph from "./MoonGlyph.svelte";

  /** True on routes that render without chrome (game, editor, tools). */
  export let fullWidth = false;

  $: currentPath = $page.url.pathname;
  $: activeIndex = navItems.findIndex((n) => isActive(n.href, currentPath));
  $: folio = activeIndex >= 0 ? String(activeIndex + 1).padStart(2, "0") : "";

  const year = new Date().getFullYear();
  const volume = roman(year - 2020);

  // The dateline reads the clock, so it is computed in the browser only.
  let line: Dateline | null = null;
  let contentsOpen = false;
  let leaf: HTMLElement | undefined;
  let fold: HTMLElement | undefined;
  let lastPath = "";

  function refreshLine() {
    line = dateline(new Date());
  }

  onMount(() => {
    refreshLine();
    lastPath = currentPath;
    // The date, and so the sky, changes only at midnight; check every few minutes.
    const id = setInterval(refreshLine, 5 * 60_000);
    return () => clearInterval(id);
  });

  // Turn the page when the path changes after the first render.
  $: if (lastPath && currentPath !== lastPath) {
    lastPath = currentPath;
    contentsOpen = false;
    tick().then(turnPage);
  }

  function turnPage() {
    if (!leaf || prefersReducedMotion() || typeof leaf.animate !== "function") return;
    const timing = { duration: 620, easing: "cubic-bezier(0.33, 1, 0.68, 1)" };
    leaf.animate(
      [
        { clipPath: "inset(-12px -280px -12px 100%)", transform: "skewX(-1.4deg)" },
        { clipPath: "inset(-12px -280px -12px -12px)", transform: "skewX(0deg)" },
      ],
      timing,
    );
    fold?.animate(
      [
        { left: "100%", opacity: 0.9 },
        { left: "0%", opacity: 0 },
      ],
      timing,
    );
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && contentsOpen) contentsOpen = false;
  }
</script>

<svelte:window on:keydown={onKeydown} />

{#if fullWidth}
  <slot />
{:else}
  <a class="skip" href="#main">Skip to content</a>

  <div class="sheet">
    <header class="masthead">
      <div class="strip">
        <span class="strip-l">Vol. {volume} <span class="sep">·</span> No. {line ? String(line.doy).padStart(3, "0") : "—"}</span>
        <div class="tools">
          <WeatherHeadsUp />
          <CommandDeck />
          <DesignSwitch />
          <ThemeToggle />
        </div>
      </div>

      <div class="plate">
        <svg class="plate-orn" viewBox="0 0 120 12" aria-hidden="true" focusable="false">
          <path d="M0 6H46M74 6H120" fill="none" stroke="currentColor" stroke-width="0.8" />
          <path d="M60 1L65 6L60 11L55 6Z" fill="none" stroke="currentColor" stroke-width="0.8" />
          <circle cx="60" cy="6" r="1.2" fill="var(--falu)" />
          <circle cx="50" cy="6" r="1" fill="currentColor" />
          <circle cx="70" cy="6" r="1" fill="currentColor" />
        </svg>
        <a href="/" class="wordmark" aria-label="ifkash.dev home">ifkash<span class="tld">.dev</span></a>
        <p class="tagline">An almanac of work, projects &amp; small experiments</p>
      </div>

      <p class="dateline" aria-label="Today in Bengaluru">
        {#if line}
          <span class="dl-item"><span class="dl-k">{line.weekday}</span> {line.date}</span>
          <span class="dl-item"><span class="dl-k">Day</span> {line.doy}</span>
          <span class="dl-item"><span class="dl-k">Week</span> {line.week}</span>
          <span class="dl-item moon-item">
            <MoonGlyph fraction={line.moon.fraction} size={20} />
            {line.moon.name}, {Math.round(line.moon.lit * 100)}%<sup><a href="#fn-sky" aria-label="Footnote 1">1</a></sup>
          </span>
          <span class="dl-item"><span class="dl-k">Sun</span> ↑ {line.rise} ↓ {line.set}<sup><a href="#fn-sky" aria-label="Footnote 1">1</a></sup></span>
          {#if $clock}<span class="dl-item"><span class="dl-k">IST</span> <time>{$clock}</time></span>{/if}
        {:else}
          <span class="dl-item">Bengaluru</span>
        {/if}
      </p>
    </header>

    <div class="body">
      <aside class="cts" class:open={contentsOpen}>
        <button
          class="contents-toggle"
          type="button"
          aria-expanded={contentsOpen}
          aria-controls="contents-nav"
          on:click={() => (contentsOpen = !contentsOpen)}
        >
          <span>Contents</span>
          <span class="chev" aria-hidden="true">{contentsOpen ? "−" : "+"}</span>
        </button>

        <nav id="contents-nav" class="contents-nav" aria-label="Primary">
          <p class="contents-title">Contents</p>
          <ol>
            {#each navItems as item, i}
              <li>
                <a
                  href={item.href}
                  aria-current={isActive(item.href, currentPath) ? "page" : undefined}
                >
                  <span class="c-name">{item.label}</span>
                  <span class="c-leader" aria-hidden="true"></span>
                  <span class="c-folio">{String(i + 1).padStart(2, "0")}</span>
                </a>
              </li>
            {/each}
          </ol>
          <ul class="contents-aside">
            <li><a href="/" aria-current={currentPath === "/" ? "page" : undefined}>Title page</a></li>
            <li><a href={resumeUrl}>Résumé</a></li>
            <li><a href="/game">Cinderheart</a></li>
          </ul>
        </nav>
      </aside>

      <main id="main" class="main">
        <div class="leaf" bind:this={leaf} style:--folio={folio ? `"${folio}"` : undefined}>
          <slot />
        </div>
        <span class="fold" bind:this={fold} aria-hidden="true"></span>
      </main>
    </div>

    <footer class="colophon">
      <svg class="engraving" viewBox="0 0 240 40" aria-hidden="true" focusable="false">
        <g fill="none" stroke="currentColor" stroke-width="0.7">
          <path d="M0 20H92M148 20H240" />
          <path d="M0 24H80M160 24H240" opacity="0.5" />
          <circle cx="120" cy="20" r="11" />
          <circle cx="120" cy="20" r="6" />
          <path d="M120 3V10M120 30V37M103 20H110M130 20H137M108 8L113 13M127 27L132 32M132 8L127 13M113 27L108 32" />
          <path d="M92 20C98 12 104 12 108 20M148 20C142 28 136 28 132 20" opacity="0.7" />
        </g>
        <circle cx="120" cy="20" r="2.2" fill="var(--falu)" />
      </svg>

      <div class="col-grid">
        <div class="col-main">
          <p class="col-title">Colophon</p>
          <p class="col-name">{owner}</p>
          <p class="col-line">{blurb}</p>
          <p class="col-line small">
            Set in Fraunces, Hanken Grotesk and IBM Plex Mono, on warm paper.
            Engraved by hand, in code.
          </p>
        </div>

        <nav class="col-links" aria-label="Footer">
          {#each footerGroups as group}
            <div>
              <p class="col-title">{group.title}</p>
              <ul>
                {#each group.links as link}
                  <li>
                    {#if link.external}
                      <a href={link.href} target="_blank" rel="noopener noreferrer">{link.label}</a>
                    {:else}
                      <a href={link.href}>{link.label}</a>
                    {/if}
                  </li>
                {/each}
              </ul>
            </div>
          {/each}
        </nav>
      </div>

      <ol class="footnotes">
        <li id="fn-sky">
          Moon phase and the times of sunrise and sunset are computed in your
          browser from today's date and Bengaluru's coordinates. The phase is
          good to about a day, the times to a few minutes.
        </li>
      </ol>

      <p class="col-base">
        <span>© {year} {owner}</span>
        <span class="coords">
          {coordinates}{#if $clock}<span class="sep">·</span><time>{$clock} IST</time>{/if}
        </span>
      </p>
    </footer>
  </div>
{/if}

<style>
  .skip {
    position: absolute;
    top: -100px;
    left: var(--gutter);
    z-index: 100;
    padding: var(--s-2) var(--s-3);
    border-radius: var(--radius-sm);
    background: var(--ink);
    color: var(--ink-inverse);
    font-size: var(--text-sm);
  }
  .skip:focus {
    top: var(--s-2);
  }

  .sheet {
    max-width: 1280px;
    margin: 0 auto;
    padding: 0 var(--gutter);
  }

  /* ── Masthead ────────────────────────────────────────────── */

  .masthead {
    padding-top: var(--s-3);
  }

  .strip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
    min-height: 40px;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.09em;
    text-transform: uppercase;
  }
  .strip-l {
    white-space: nowrap;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    letter-spacing: normal;
    text-transform: none;
  }

  .plate {
    margin-top: var(--s-3);
    padding: var(--s-5) 0 var(--s-4);
    border-top: 1px solid var(--ink);
    text-align: center;
  }
  .plate-orn {
    width: 120px;
    height: 12px;
    color: var(--ink-2);
  }

  .wordmark {
    display: inline-block;
    margin-top: var(--s-2);
    color: var(--ink);
    font-family: var(--font-display);
    font-size: clamp(2.8rem, 2rem + 4.2vw, 5.4rem);
    font-style: italic;
    font-weight: 300;
    line-height: 1;
    letter-spacing: -0.03em;
    font-optical-sizing: none;
    font-variation-settings: "SOFT" 50, "WONK" 0, "opsz" 144, "wght" 300;
  }
  .wordmark:hover {
    color: var(--ink);
  }
  .tld {
    font-size: 0.45em;
    letter-spacing: 0;
    color: var(--ink-3);
  }
  .tagline {
    margin: var(--s-3) 0 0;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  .dateline {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--s-1) var(--s-5);
    margin: 0;
    padding: var(--s-3) 0;
    border-top: 1px solid var(--ink);
    border-bottom: 3px double var(--ink);
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: 11.5px;
    font-variant-numeric: tabular-nums lining-nums;
    letter-spacing: 0.04em;
    min-height: 3.2em;
    align-items: center;
  }
  .dl-item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
  }
  .dl-k {
    color: var(--ink-3);
    letter-spacing: 0.09em;
    text-transform: uppercase;
    font-size: 10.5px;
  }
  .dateline sup {
    font-size: 0.75em;
    line-height: 0;
  }
  .dateline sup a {
    color: var(--falu);
    text-decoration: none;
  }
  .moon-item {
    color: var(--ink);
  }
  .sep {
    margin: 0 0.5em;
  }

  /* ── Body: contents, text, gutter ───────────────────────── */

  .body {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    padding-bottom: var(--s-8);
  }

  .cts {
    position: relative;
    z-index: 5;
  }

  .contents-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    min-height: 44px;
    margin-top: var(--s-4);
    padding: 0 var(--s-3);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    cursor: pointer;
  }
  .contents-toggle:hover {
    background: var(--hover-wash);
  }
  .chev {
    font-size: 16px;
    line-height: 1;
  }

  /* The list unfolds like a folded leaf: it hinges at the top edge. */
  .contents-nav {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    padding: var(--s-4);
    border: 1px solid var(--line-strong);
    border-top: 0;
    border-radius: 0 0 var(--radius-md) var(--radius-md);
    background: var(--surface);
    transform-origin: top center;
    transform: perspective(700px) rotateX(-90deg);
    opacity: 0;
    visibility: hidden;
    transition:
      transform 420ms var(--ease),
      opacity 240ms var(--ease),
      visibility 0s linear 420ms;
  }
  .cts.open .contents-nav {
    transform: perspective(700px) rotateX(0deg);
    opacity: 1;
    visibility: visible;
    transition-delay: 0s;
  }

  .contents-title {
    display: none;
    margin: 0 0 var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }
  .cts ol,
  .cts ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .cts ol a {
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
    min-height: 40px;
    padding: 6px 0;
    color: var(--ink);
    font-family: var(--font-display);
    font-size: 1.15rem;
    font-weight: 350;
    font-variation-settings: "SOFT" 50, "WONK" 0, "opsz" 36, "wght" 350;
    text-decoration: none;
  }
  .c-leader {
    flex: 1;
    min-width: 12px;
    align-self: flex-end;
    margin-bottom: 0.45em;
    border-bottom: 1px dotted var(--line-strong);
  }
  .c-folio {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.06em;
  }
  .cts ol a:hover .c-name {
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 4px;
  }
  .cts ol a[aria-current="page"] .c-name {
    font-style: italic;
  }
  .cts ol a[aria-current="page"] .c-folio {
    color: var(--falu);
  }
  .cts ol a[aria-current="page"] .c-folio::before {
    content: "▸ ";
  }

  .contents-aside {
    margin-top: var(--s-4) !important;
    padding-top: var(--s-3) !important;
    border-top: 1px solid var(--line);
  }
  .contents-aside a {
    display: block;
    padding: 6px 0;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    text-decoration: none;
  }
  .contents-aside a:hover,
  .contents-aside a[aria-current="page"] {
    color: var(--ink);
  }

  .main {
    position: relative;
    min-width: 0;
    min-height: 60vh;
  }
  .leaf {
    min-height: 60vh;
  }
  .fold {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    width: 1px;
    background: var(--ink);
    opacity: 0;
    pointer-events: none;
  }

  @media (min-width: 900px) {
    .body {
      grid-template-columns: 200px minmax(0, 1fr);
      column-gap: var(--s-6);
    }
    .cts {
      position: sticky;
      top: var(--s-4);
      align-self: start;
      max-height: calc(100vh - 2 * var(--s-4));
      overflow-y: auto;
      padding: var(--s-6) 0 0;
    }
    .contents-toggle {
      display: none;
    }
    .contents-nav {
      position: static;
      padding: 0;
      border: 0;
      border-radius: 0;
      background: none;
      transform: none;
      opacity: 1;
      visibility: visible;
      transition: none;
    }
    .contents-title {
      display: block;
      padding-bottom: var(--s-2);
      border-bottom: 3px double var(--ink);
    }
    .cts ol a {
      font-size: 1.05rem;
    }
    .main {
      border-left: 1px solid var(--line);
      padding-left: var(--s-6);
    }
  }

  @media (min-width: 1200px) {
    .body {
      grid-template-columns: 200px minmax(0, 1fr) 190px;
    }
    .main {
      grid-column: 2;
      margin-right: 0;
    }
  }

  /* ── Colophon ────────────────────────────────────────────── */

  .colophon {
    padding: var(--s-7) 0 var(--s-6);
    border-top: 3px double var(--ink);
    color: var(--ink-2);
  }
  .engraving {
    display: block;
    width: min(240px, 70%);
    height: auto;
    margin: 0 auto var(--s-6);
    color: var(--ink-2);
  }
  .col-grid {
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
    gap: var(--s-6) var(--s-8);
  }
  .col-title {
    margin: 0 0 var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }
  .col-name {
    margin: 0 0 var(--s-2);
    color: var(--ink);
    font-family: var(--font-display);
    font-size: var(--text-xl);
    font-style: italic;
    font-weight: 300;
    font-variation-settings: "SOFT" 50, "WONK" 0, "opsz" 48, "wght" 300;
  }
  .col-line {
    max-width: 52ch;
    margin: 0 0 var(--s-2);
    font-size: 15px;
  }
  .col-line.small {
    color: var(--ink-3);
    font-size: 13.5px;
  }
  .col-links {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--s-5);
  }
  .col-links ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .col-links li + li {
    margin-top: var(--s-2);
  }
  .col-links a {
    color: var(--ink);
    font-size: 15px;
    text-decoration: none;
  }
  .col-links a:hover {
    color: var(--fjord);
    text-decoration: underline;
    text-underline-offset: 4px;
  }

  .footnotes {
    max-width: 70ch;
    margin: var(--s-6) 0 0;
    padding: var(--s-4) 0 0 1.4em;
    border-top: 1px solid var(--line);
    color: var(--ink-3);
    font-size: 12.5px;
    line-height: 1.55;
  }
  .footnotes li:target {
    color: var(--ink);
  }

  .col-base {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: var(--s-2) var(--s-5);
    margin: var(--s-5) 0 0;
    padding-top: var(--s-4);
    border-top: 1px solid var(--line);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  @media (max-width: 720px) {
    .col-grid {
      grid-template-columns: 1fr;
    }
    .dateline {
      justify-content: flex-start;
      gap: var(--s-1) var(--s-4);
    }
    .strip-l {
      display: none;
    }
    .strip {
      justify-content: flex-end;
    }
  }
</style>
