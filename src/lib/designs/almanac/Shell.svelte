<script lang="ts">
  /**
   * Almanac shell: a solid paper top bar with a hairline, a content column,
   * and a footer with a coordinate line. See DESIGN.md for the shell contract.
   */
  import { onMount, tick } from "svelte";
  import { page } from "$app/stores";
  import { animate, utils } from "animejs";
  import { prefersReducedMotion } from "$lib/motion";
  import CommandDeck from "$lib/components/CommandDeck.svelte";
  import WeatherHeadsUp from "$lib/components/WeatherHeadsUp.svelte";
  import DesignSwitch from "$lib/components/DesignSwitch.svelte";
  import ThemeToggle from "../ThemeToggle.svelte";
  import { clock } from "../clock";
  import { navItems, isActive, owner, blurb, coordinates, footerGroups } from "../shared";

  /** True on routes that render without chrome (game, editor, tools). */
  export let fullWidth = false;

  $: currentPath = $page.url.pathname;

  // The nav underline slides between items instead of jumping. It is placed
  // in the browser only, so the CSS underline still marks the page without JS.
  let navList: HTMLElement | undefined;
  let indicator: HTMLElement | undefined;
  let mounted = false;
  let placed = false;
  let enhanced = false;

  function placeIndicator(instant = false) {
    if (!navList || !indicator) {
      placed = false;
      return;
    }
    const link = navList.querySelector<HTMLElement>('a[aria-current="page"]');
    if (!link) {
      indicator.style.opacity = "0";
      placed = false;
      return;
    }
    const inset = parseFloat(getComputedStyle(link).paddingLeft) || 0;
    const x = link.offsetLeft + inset;
    const width = link.offsetWidth - inset * 2;
    enhanced = true;
    indicator.style.opacity = "1";
    if (instant || !placed || prefersReducedMotion()) {
      utils.set(indicator, { translateX: x, width });
    } else {
      animate(indicator, { translateX: x, width, duration: 500, ease: "outExpo" });
    }
    placed = true;
  }

  $: if (mounted && navList) {
    currentPath;
    tick().then(() => placeIndicator());
  }
  $: if (!navList) placed = false;

  onMount(() => {
    mounted = true;
    document.fonts?.ready.then(() => placeIndicator(true));
  });

  const year = new Date().getFullYear();
</script>

<svelte:window on:resize={() => placeIndicator(true)} />

{#if fullWidth}
  <slot />
{:else}
  <a class="skip" href="#main">Skip to content</a>

  <header class="topbar">
    <div class="topbar-inner">
      <a href="/" class="wordmark" aria-label="ifkash.dev home">
        <span class="dot" aria-hidden="true"></span>
        <span>ifkash</span>
      </a>

      <nav class="nav" class:enhanced aria-label="Primary">
        <ul bind:this={navList}>
          {#each navItems as item}
            <li>
              <a
                href={item.href}
                aria-current={isActive(item.href, currentPath) ? "page" : undefined}
              >
                {item.label}
              </a>
            </li>
          {/each}
          <span class="indicator" bind:this={indicator} aria-hidden="true"></span>
        </ul>
      </nav>

      <div class="tools">
        <WeatherHeadsUp />
        <CommandDeck />
        <DesignSwitch />
        <ThemeToggle />
      </div>
    </div>
  </header>

  <main id="main" class="main">
    <slot />
  </main>

  <footer class="footer">
    <div class="footer-inner">
      <div class="footer-sign">
        <p class="footer-name">{owner}</p>
        <p class="footer-line">{blurb}</p>
      </div>

      <nav class="footer-cols" aria-label="Footer">
        {#each footerGroups as group}
          <div>
            <p class="eyebrow">{group.title}</p>
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

      <div class="footer-base">
        <span>© {year} {owner}</span>
        <span class="coords">
          {coordinates}{#if $clock}<span class="sep">·</span><time>{$clock} IST</time>{/if}
        </span>
      </div>
    </div>
  </footer>
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

  /* ── Top bar ─────────────────────────────────────────────── */

  .topbar {
    position: sticky;
    top: 0;
    z-index: 40;
    border-bottom: 1px solid var(--line);
    background: var(--bg);
  }

  .topbar-inner {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--s-6);
    max-width: calc(var(--content-max) + 2 * var(--gutter));
    height: 64px;
    margin: 0 auto;
    padding: 0 var(--gutter);
  }

  .wordmark {
    display: inline-flex;
    align-items: center;
    gap: 9px;
    color: var(--ink);
    font-family: var(--font-display);
    font-size: 17px;
    font-style: italic;
    font-weight: 400;
    letter-spacing: -0.01em;
    font-optical-sizing: none;
    font-variation-settings: "SOFT" 50, "WONK" 0, "opsz" 48, "wght" 400;
  }
  .wordmark .dot {
    width: 9px;
    height: 9px;
    transition: transform var(--dur) var(--ease-out);
  }
  .wordmark:hover {
    color: var(--ink);
  }
  .wordmark:hover .dot {
    transform: scale(1.25);
  }

  .nav ul {
    position: relative;
    display: flex;
    justify-content: center;
    gap: var(--s-1);
    list-style: none;
  }

  .nav a {
    position: relative;
    display: inline-flex;
    align-items: center;
    height: 32px;
    padding: 0 var(--s-3);
    border-radius: var(--radius-sm);
    color: var(--ink-2);
    font-size: 14px;
    letter-spacing: -0.005em;
  }
  .nav a:hover {
    color: var(--ink);
    background: var(--hover-wash);
  }
  .nav a[aria-current="page"] {
    color: var(--ink);
  }
  .nav a[aria-current="page"]::after {
    content: "";
    position: absolute;
    bottom: -16px;
    left: var(--s-3);
    right: var(--s-3);
    height: 1px;
    background: var(--ink);
  }

  .nav.enhanced a[aria-current="page"]::after {
    display: none;
  }
  .indicator {
    position: absolute;
    bottom: -16px;
    left: 0;
    width: 0;
    height: 1px;
    background: var(--ink);
    opacity: 0;
    pointer-events: none;
  }

  .tools {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  /* ── Main ────────────────────────────────────────────────── */

  .main {
    max-width: calc(var(--content-max) + 2 * var(--gutter));
    min-height: 60vh;
    margin: 0 auto;
    padding: 0 var(--gutter) var(--s-9);
  }

  /* ── Footer ──────────────────────────────────────────────── */

  .footer {
    border-top: 1px solid var(--line);
    background: var(--bg);
  }

  .footer-inner {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: var(--s-7) var(--s-8);
    max-width: calc(var(--content-max) + 2 * var(--gutter));
    margin: 0 auto;
    padding: var(--s-8) var(--gutter) var(--s-6);
  }

  .footer-name {
    margin: 0 0 var(--s-2);
    color: var(--ink);
    font-size: var(--text-lg);
    font-weight: 500;
    letter-spacing: -0.02em;
  }

  .footer-line {
    max-width: 44ch;
    margin: 0;
    color: var(--ink-2);
    font-size: 15px;
  }

  .footer-cols {
    display: grid;
    grid-template-columns: repeat(2, minmax(140px, auto));
    gap: var(--s-7);
  }

  .footer-cols ul {
    margin-top: var(--s-3);
    list-style: none;
  }
  .footer-cols li + li {
    margin-top: var(--s-2);
  }
  .footer-cols a {
    color: var(--ink);
    font-size: 15px;
    text-decoration: none;
  }
  .footer-cols a:hover {
    color: var(--fjord);
  }

  .footer-base {
    grid-column: 1 / -1;
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: var(--s-2) var(--s-5);
    padding-top: var(--s-5);
    border-top: 1px solid var(--line);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .sep {
    margin: 0 0.6em;
  }

  @media (max-width: 960px) {
    .topbar-inner {
      grid-template-columns: minmax(0, 1fr) auto;
      grid-template-rows: 56px auto;
      height: auto;
      gap: 0 var(--s-4);
    }
    .tools {
      justify-self: end;
    }
    .nav {
      min-width: 0;
      grid-column: 1 / -1;
      grid-row: 2;
      margin: 0 calc(-1 * var(--gutter));
      border-top: 1px solid var(--line);
    }
    .nav ul {
      justify-content: flex-start;
      gap: 0;
      padding: var(--s-1) calc(var(--gutter) - var(--s-3));
      overflow-x: auto;
      scrollbar-width: none;
    }
    .nav ul::-webkit-scrollbar {
      display: none;
    }
    .nav a {
      height: 40px;
      white-space: nowrap;
    }
    .nav a[aria-current="page"]::after {
      bottom: 4px;
    }
    .indicator {
      bottom: 8px;
    }
  }

  @media (max-width: 640px) {
    .footer-inner {
      grid-template-columns: 1fr;
    }
  }
</style>
