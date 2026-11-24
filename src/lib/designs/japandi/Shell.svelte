<script lang="ts">
  /**
   * Japandi shell. There is no top bar. On wide screens a slim vertical rail
   * on the right edge holds the wordmark, the navigation, and the tools, and
   * the content sits in an off-centre column. On narrow screens the same
   * rail becomes a bottom sheet that slides up like a shoji panel.
   */
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { afterNavigate } from "$app/navigation";
  import CommandDeck from "$lib/components/CommandDeck.svelte";
  import WeatherHeadsUp from "$lib/components/WeatherHeadsUp.svelte";
  import DesignSwitch from "$lib/components/DesignSwitch.svelte";
  import ThemeToggle from "../ThemeToggle.svelte";
  import { clock } from "../clock";
  import { navItems, isActive, owner, coordinates, footerGroups } from "../shared";
  import Seal from "./Seal.svelte";

  /** True on routes that render without chrome (game, editor, tools). */
  export let fullWidth = false;

  // A short Japanese caption for each nav label, keyed by href.
  const captions: Record<string, string> = {
    "/work": "仕事",
    "/projects": "作品",
    "/rl": "学習",
    "/radar": "探",
    "/roadmap": "道",
    "/tools": "道具",
    "/education": "学歴",
    "/blog": "記",
  };

  $: currentPath = $page.url.pathname;

  let sheetOpen = false;
  let mainEl: HTMLElement | undefined;
  let menuButton: HTMLButtonElement | undefined;

  // Close the sheet when the page changes.
  $: if (currentPath) sheetOpen = false;

  // Replay the ink-bleed fade on each client-side navigation.
  afterNavigate(({ from }) => {
    if (!from || !mainEl) return;
    mainEl.classList.remove("bleed");
    void mainEl.offsetWidth;
    mainEl.classList.add("bleed");
  });

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && sheetOpen) {
      sheetOpen = false;
      menuButton?.focus();
    }
  }

  let mounted = false;
  onMount(() => {
    mounted = true;
  });

  const year = new Date().getFullYear();
</script>

<svelte:window on:keydown={onKeydown} />

{#if fullWidth}
  <slot />
{:else}
  <!-- Filter for the seal's rough edge. Used by every <Seal />. -->
  <svg class="defs" width="0" height="0" aria-hidden="true" focusable="false">
    <filter id="japandi-hanko" x="-10%" y="-10%" width="120%" height="120%">
      <feTurbulence type="fractalNoise" baseFrequency="0.06" numOctaves="2" seed="5" result="warp" />
      <feDisplacementMap in="SourceGraphic" in2="warp" scale="2.4" xChannelSelector="R" yChannelSelector="G" result="rough" />
      <feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="1" seed="9" result="grain" />
      <feColorMatrix in="grain" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 -2.2 2.0" result="speckle" />
      <feComposite in="rough" in2="speckle" operator="in" />
    </filter>
  </svg>

  <a class="skip" href="#main">Skip to content</a>

  <div class="frame">
    <aside class="rail" class:open={sheetOpen} id="rail" aria-label="Site">
      <button
        type="button"
        class="sheet-close"
        on:click={() => {
          sheetOpen = false;
          menuButton?.focus();
        }}
      >
        <span lang="ja" aria-hidden="true">閉</span> Close menu
      </button>

      <a href="/" class="seal-link" aria-label="ifkash.dev home">
        <Seal size={44} />
      </a>

      <a href="/" class="wordmark" aria-hidden="true" tabindex="-1">ifkash.dev</a>

      <nav class="nav" aria-label="Primary">
        <ul>
          {#each navItems as item}
            <li>
              <a
                href={item.href}
                aria-current={isActive(item.href, currentPath) ? "page" : undefined}
              >
                <span class="jp" lang="ja" aria-hidden="true">{captions[item.href] ?? ""}</span>
                <span class="en">{item.label}</span>
              </a>
            </li>
          {/each}
        </ul>
      </nav>

      <div class="tools">
        <WeatherHeadsUp />
        <CommandDeck />
        <DesignSwitch />
        <ThemeToggle />
      </div>
    </aside>

    <button
      type="button"
      class="scrim"
      class:open={sheetOpen}
      tabindex="-1"
      aria-hidden="true"
      on:click={() => (sheetOpen = false)}
    ></button>

    <div class="dock">
      <button
        type="button"
        class="dock-menu"
        bind:this={menuButton}
        aria-expanded={sheetOpen}
        aria-controls="rail"
        on:click={() => (sheetOpen = !sheetOpen)}
      >
        <span lang="ja" aria-hidden="true">目次</span>
        Menu
      </button>
      <a href="/" class="dock-mark" aria-label="ifkash.dev home">
        <Seal size={30} />
        <span>ifkash.dev</span>
      </a>
    </div>

    <div class="column">
      <main id="main" class="main bleed" bind:this={mainEl}>
        <slot />
      </main>

      <footer class="footer">
        <svg class="hills" viewBox="0 0 1200 40" preserveAspectRatio="none" aria-hidden="true">
          <path
            d="M0 40 L0 30 C120 22 200 12 300 20 C380 26 430 30 520 22 C640 10 700 6 800 18 C900 30 980 28 1080 16 C1130 10 1170 14 1200 18 L1200 40 Z"
          />
        </svg>
        <div class="beam" aria-hidden="true"></div>

        <div class="footer-inner">
          <p class="haiku" lang="en">
            Soft light on the page —<br />
            one small house on the network.<br />
            The build passes. Rest.
          </p>

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

          <p class="footer-base">
            <span>© {year} {owner}</span>
            <span class="coords">
              {coordinates}{#if $clock}<span class="sep">·</span><time>{$clock} IST</time>{/if}
            </span>
          </p>
        </div>
      </footer>
    </div>
  </div>
{/if}

<style>
  .defs {
    position: absolute;
    width: 0;
    height: 0;
    overflow: hidden;
  }

  .skip {
    position: absolute;
    top: -100px;
    left: var(--gutter);
    z-index: 100;
    padding: var(--s-2) var(--s-3);
    background: var(--ink);
    color: var(--ink-inverse);
    font-size: var(--text-sm);
  }
  .skip:focus {
    top: var(--s-2);
  }

  .frame {
    --rail: 124px;
    position: relative;
  }

  /* ── Rail (desktop) ───────────────────────────────────────── */

  .rail {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    z-index: 40;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-5);
    width: var(--rail);
    padding: var(--s-5) 0 var(--s-5);
    background: color-mix(in srgb, var(--bg) 88%, transparent);
    -webkit-backdrop-filter: blur(6px);
    backdrop-filter: blur(6px);
  }
  /* A wooden-beam hairline down the left edge of the rail. */
  .rail::before {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    width: 3px;
    background: linear-gradient(
      180deg,
      transparent,
      var(--line-strong) 10%,
      var(--line) 30%,
      var(--line-strong) 55%,
      var(--line) 80%,
      transparent
    );
    opacity: 0.8;
  }

  .sheet-close {
    display: none;
  }

  .seal-link {
    display: block;
    transition: transform var(--dur) var(--ease-out);
  }
  .seal-link:hover {
    transform: rotate(4deg) scale(1.06);
  }

  .wordmark {
    writing-mode: vertical-rl;
    color: var(--ink);
    font-family: var(--font-display);
    font-size: 17px;
    font-weight: 500;
    letter-spacing: 0.32em;
    white-space: nowrap;
  }
  .wordmark:hover {
    color: var(--ink);
  }

  .nav ul {
    display: grid;
    grid-template-columns: repeat(4, 22px);
    direction: rtl;
    gap: var(--s-4) 8px;
    justify-content: center;
    list-style: none;
  }
  .nav li {
    direction: ltr;
  }
  .nav a {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.55em;
    height: 118px;
    padding: 2px 0;
    writing-mode: vertical-rl;
    color: var(--ink-2);
    line-height: 24px;
    white-space: nowrap;
  }
  .nav .jp {
    font-family: var(--font-display);
    font-size: 14px;
    font-weight: 500;
    letter-spacing: 0.1em;
    color: var(--ink);
  }
  .nav .en {
    font-size: 11.5px;
    letter-spacing: 0.1em;
  }
  /* A short brush stroke above the label marks the page you are on. */
  .nav a::before {
    content: "";
    position: absolute;
    top: -7px;
    right: 0;
    left: 0;
    height: 2px;
    background: var(--falu);
    transform: scaleX(0);
    transform-origin: right;
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .nav a:hover {
    color: var(--ink);
  }
  .nav a:hover::before {
    transform: scaleX(0.4);
  }
  .nav a[aria-current="page"] {
    color: var(--ink);
  }
  .nav a[aria-current="page"]::before {
    transform: scaleX(1);
  }

  .tools {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-2);
    margin-top: auto;
  }

  /* ── Column and main ──────────────────────────────────────── */

  .column {
    position: relative;
    z-index: 1;
    margin-right: var(--rail);
    padding-left: clamp(var(--gutter), 7vw, 136px);
    padding-right: clamp(var(--gutter), 3vw, 48px);
  }

  .main {
    max-width: 920px;
    min-height: 70vh;
    padding-bottom: var(--s-6);
  }

  /* ── Footer ───────────────────────────────────────────────── */

  .footer {
    max-width: 920px;
    margin-top: var(--s-6);
  }

  .hills {
    width: 100%;
    height: 30px;
    fill: color-mix(in srgb, var(--ink) 7%, transparent);
  }
  .beam {
    height: 4px;
    background:
      repeating-linear-gradient(
        90deg,
        transparent 0 38px,
        color-mix(in srgb, var(--ink) 14%, transparent) 38px 39px,
        transparent 39px 91px
      ),
      linear-gradient(
        90deg,
        var(--line-strong),
        var(--line) 22%,
        var(--line-strong) 47%,
        var(--line) 71%,
        var(--line-strong)
      );
    border-radius: 1px;
  }

  .footer-inner {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: var(--s-7) var(--s-8);
    padding: var(--s-7) 0 var(--s-6);
  }

  .haiku {
    margin: 0;
    color: var(--ink-2);
    font-family: var(--font-display);
    font-size: 1.0625rem;
    line-height: 2;
    letter-spacing: 0.04em;
  }

  .footer-cols {
    display: grid;
    grid-template-columns: repeat(2, minmax(120px, auto));
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
    font-size: 14px;
    background: linear-gradient(currentColor, currentColor) no-repeat 0 100% / 0 1px;
    transition: background-size var(--dur) var(--ease-out);
  }
  .footer-cols a:hover {
    color: var(--ink);
    background-size: 100% 1px;
  }

  .footer-base {
    grid-column: 1 / -1;
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: var(--s-2) var(--s-5);
    margin: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .sep {
    margin: 0 0.6em;
  }

  /* ── Mobile: dock and sheet ───────────────────────────────── */

  .dock,
  .scrim {
    display: none;
  }

  @media (max-width: 899px) {
    .frame {
      --rail: 0px;
    }

    .column {
      padding-left: var(--gutter);
      padding-right: var(--gutter);
      padding-bottom: 76px;
    }
    .main {
      padding-bottom: var(--s-8);
    }
    .footer-inner {
      grid-template-columns: 1fr;
      gap: var(--s-6);
    }

    .dock {
      position: fixed;
      right: 0;
      bottom: 0;
      left: 0;
      z-index: 50;
      display: flex;
      align-items: center;
      justify-content: flex-start;
      gap: var(--s-5);
      height: 60px;
      padding: 0 128px env(safe-area-inset-bottom) var(--gutter);
      background: color-mix(in srgb, var(--bg) 94%, transparent);
      -webkit-backdrop-filter: blur(8px);
      backdrop-filter: blur(8px);
    }
    .dock::before {
      content: "";
      position: absolute;
      top: 0;
      right: 0;
      left: 0;
      height: 2px;
      background: linear-gradient(90deg, var(--line), var(--line-strong), var(--line));
    }
    .dock-mark {
      display: inline-flex;
      align-items: center;
      gap: var(--s-3);
      color: var(--ink);
      font-family: var(--font-display);
      font-size: 15px;
      letter-spacing: 0.12em;
    }
    .dock-menu {
      display: inline-flex;
      align-items: center;
      gap: var(--s-2);
      min-height: 40px;
      padding: 0 var(--s-2);
      border: 0;
      background: linear-gradient(currentColor, currentColor) no-repeat 0 calc(100% - 6px) / 100% 1px;
      color: var(--ink);
      font-size: 13px;
      letter-spacing: 0.1em;
      white-space: nowrap;
      cursor: pointer;
    }
    .dock-menu span {
      font-family: var(--font-display);
      font-size: 15px;
    }

    .scrim {
      position: fixed;
      inset: 0;
      z-index: 55;
      display: block;
      border: 0;
      background: var(--scrim);
      opacity: 0;
      pointer-events: none;
      transition: opacity var(--dur-slow) var(--ease);
    }
    .scrim.open {
      opacity: 1;
      pointer-events: auto;
    }

    /* The rail becomes a sheet that slides up. A lattice of thin lines
       behind the links reads as a shoji screen. */
    .rail {
      top: auto;
      right: 0;
      bottom: 0;
      left: 0;
      z-index: 60;
      width: auto;
      max-height: 86vh;
      overflow-y: auto;
      align-items: stretch;
      gap: var(--s-5);
      padding: var(--s-5) var(--gutter) calc(var(--s-6) + env(safe-area-inset-bottom));
      background: var(--bg);
      border-radius: 2px 2px 0 0;
      transform: translateY(104%);
      visibility: hidden;
      transition:
        transform 650ms var(--ease-out),
        visibility 0s linear 650ms;
    }
    .rail.open {
      transform: none;
      visibility: visible;
      transition:
        transform 650ms var(--ease-out),
        visibility 0s;
    }
    .rail::before {
      top: 0;
      right: 0;
      bottom: auto;
      left: 0;
      width: auto;
      height: 4px;
      background: linear-gradient(90deg, var(--line), var(--line-strong), var(--line));
    }
    .rail::after {
      content: "";
      position: absolute;
      inset: 4px 0 0;
      z-index: -1;
      background-image:
        linear-gradient(var(--line) 1px, transparent 1px),
        linear-gradient(90deg, var(--line) 1px, transparent 1px);
      background-size: 100% 64px, 25% 100%;
      opacity: 0.45;
      pointer-events: none;
    }

    .sheet-close {
      display: inline-flex;
      align-items: center;
      gap: var(--s-2);
      align-self: flex-end;
      min-height: 40px;
      padding: 0 var(--s-2);
      border: 0;
      background: transparent;
      color: var(--ink-2);
      font-size: 13px;
      letter-spacing: 0.08em;
      cursor: pointer;
    }
    .sheet-close span {
      font-family: var(--font-display);
      font-size: 16px;
      color: var(--ink);
    }

    .seal-link,
    .wordmark {
      display: none;
    }

    .nav ul {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      direction: ltr;
      gap: 0 var(--s-5);
    }
    .nav a {
      height: auto;
      min-height: 52px;
      padding: var(--s-3) 0;
      writing-mode: horizontal-tb;
      gap: var(--s-3);
      line-height: 1.3;
      border-bottom: 1px solid var(--line);
    }
    .nav .jp {
      width: 2.6em;
      font-size: 17px;
    }
    .nav .en {
      font-size: 14px;
      letter-spacing: 0.06em;
    }
    .nav a::before {
      top: auto;
      right: auto;
      bottom: -1px;
      left: 0;
      width: 2.6em;
      transform-origin: left;
    }

    .tools {
      flex-direction: row;
      justify-content: flex-start;
      flex-wrap: wrap;
      gap: var(--s-3);
      margin-top: 0;
    }
  }

  /* Short desktop windows: tighten the rail so the tools stay reachable. */
  @media (min-width: 900px) and (max-height: 760px) {
    .rail {
      gap: var(--s-4);
      padding-top: var(--s-4);
    }
    .nav a {
      height: 104px;
    }
    .nav ul {
      gap: var(--s-3) 6px;
    }
  }
  @media (min-width: 900px) and (max-height: 620px) {
    .rail {
      overflow-y: auto;
      scrollbar-width: none;
    }
  }
</style>
