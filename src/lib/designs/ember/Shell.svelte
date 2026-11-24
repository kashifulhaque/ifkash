<script lang="ts">
  /**
   * Ember shell, "Vigil". The site behaves like a game's title and pause
   * screens:
   * - a fixed background stage (halo, light rays, fog, drifting embers,
   *   vignette, and map contours in the light theme);
   * - a left rail that works as a vertical game menu with a sliding
   *   selector (a top bar and a full-screen Menu overlay on small screens);
   * - an item description panel beside any `.rows` list on wide screens;
   * - a bottom hints bar whose keys are real shortcuts.
   */
  import { onMount, tick } from "svelte";
  import { page } from "$app/stores";
  import { afterNavigate, goto, onNavigate } from "$app/navigation";
  import { prefersReducedMotion } from "$lib/motion";
  import CommandDeck from "$lib/components/CommandDeck.svelte";
  import WeatherHeadsUp from "$lib/components/WeatherHeadsUp.svelte";
  import DesignSwitch from "$lib/components/DesignSwitch.svelte";
  import ThemeToggle from "../ThemeToggle.svelte";
  import Embers from "./Embers.svelte";
  import { menuNav } from "./menu";
  import { clock } from "../clock";
  import { navItems, isActive, owner, footerGroups } from "../shared";
  import { DESIGN_METAS } from "../metas";

  /** True on routes that render without chrome (game, editor, tools). */
  export let fullWidth = false;

  const copy = DESIGN_METAS.ember.themeCopy;

  $: currentPath = $page.url.pathname;
  $: isHome = currentPath === "/";

  const menuItems = [
    ...navItems,
    { href: "/game", label: "Cinderheart" },
  ];

  const year = new Date().getFullYear();

  // ── Overlay menu ────────────────────────────────────────

  let menuOpen = false;
  let overlayEl: HTMLElement | undefined;
  let returnFocus: HTMLElement | null = null;

  async function openMenu() {
    if (menuOpen) return;
    returnFocus = document.activeElement as HTMLElement | null;
    menuOpen = true;
    document.body.style.overflow = "hidden";
    await tick();
    const first =
      overlayEl?.querySelector<HTMLElement>('a[aria-current="page"]') ??
      overlayEl?.querySelector<HTMLElement>("a[href]");
    first?.focus();
  }

  function closeMenu(restore = true) {
    if (!menuOpen) return;
    menuOpen = false;
    document.body.style.removeProperty("overflow");
    if (restore) returnFocus?.focus?.();
  }

  function trapTab(e: KeyboardEvent) {
    if (e.key !== "Tab" || !overlayEl) return;
    const items = Array.from(
      overlayEl.querySelectorAll<HTMLElement>("a[href], button, select, [tabindex]:not([tabindex='-1'])"),
    ).filter((el) => el.offsetParent !== null);
    if (!items.length) return;
    const first = items[0];
    const last = items[items.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  function onKey(e: KeyboardEvent) {
    if (fullWidth || e.defaultPrevented || e.metaKey || e.ctrlKey || e.altKey) return;
    const t = e.target as HTMLElement | null;
    const typing =
      !!t && (t.isContentEditable || /^(input|textarea|select)$/i.test(t.tagName));
    if (e.key === "Escape") {
      if (menuOpen) {
        e.preventDefault();
        closeMenu();
        return;
      }
      // Leave Escape to other dialogs, popovers, and the text fields.
      if (typing || t?.closest(".tools, dialog, [role='dialog']")) return;
      if (document.querySelector("dialog[open], .agent-panel")) return;
      openMenu();
      return;
    }
    if (typing || menuOpen) return;
    if (e.key === "h" || e.key === "H") {
      if (currentPath !== "/") goto("/");
    }
  }

  // ── Page transition ─────────────────────────────────────

  let veil: HTMLElement | undefined;

  onNavigate((nav) => {
    if (fullWidth || !veil || prefersReducedMotion()) return;
    if (nav.from?.url.pathname === nav.to?.url.pathname) return;
    veil.dataset.phase = "in";
    return new Promise<void>((resolve) => setTimeout(resolve, 200));
  });

  afterNavigate(async () => {
    closeMenu(false);
    if (veil && veil.dataset.phase === "in") {
      veil.dataset.phase = "out";
      const el = veil;
      setTimeout(() => {
        if (el.dataset.phase === "out") el.dataset.phase = "";
      }, 1000);
    }
    await tick();
    setupPanel();
  });

  // ── Item description panel ──────────────────────────────

  interface PanelData {
    id: string;
    n: string;
    title: string;
    desc: string;
    stats: { k: string; v: string }[];
    href: string;
  }

  let hasPanel = false;
  let panel: PanelData = { id: "", n: "", title: "", desc: "", stats: [], href: "" };
  let picked: Element | null = null;
  let wideQuery: MediaQueryList | undefined;

  const rowSelector = ".rows > li, .rows > .row";
  const clean = (s: string | null | undefined) => (s ?? "").replace(/\s+/g, " ").trim();

  /** Reads a row's visible text. It never clones markup into the panel. */
  function readRow(li: Element): PanelData {
    const text = (sel: string) => clean(li.querySelector(sel)?.textContent);
    const stats: { k: string; v: string }[] = [];
    const add = (k: string, v: string) => v && stats.push({ k, v });
    add("Period", text(".row-period"));
    add("Place", text(".row-location"));
    const tags = Array.from(li.querySelectorAll(".tag")).map((t) => clean(t.textContent));
    add("Status", tags.join(", "));
    add("Path", text(".row-path"));
    const links = Array.from(li.querySelectorAll(".row-link")).map((a) =>
      clean(a.textContent).replace(/[↗→]/g, "").trim(),
    );
    add("Links", links.join(", "));
    const anchor = li.querySelector<HTMLAnchorElement>("a.row-title, a.row, a[href]");
    const title =
      text(".row-name") || text(".entry-name") || text(".row-school") || text("h2, h3, .row-title") || clean(li.textContent).slice(0, 60);
    return {
      id: `${text(".row-n")}-${title}`,
      n: text(".row-n"),
      title,
      desc: text(".row-desc") || text(".row-degree") || text("p"),
      stats,
      href: anchor?.getAttribute("href") ?? "",
    };
  }

  function pick(li: Element | null) {
    if (!li || li === picked) return;
    picked?.classList.remove("ember-picked");
    picked = li;
    li.classList.add("ember-picked");
    panel = readRow(li);
  }

  function clearPicked() {
    picked?.classList.remove("ember-picked");
    picked = null;
  }

  function setupPanel() {
    clearPicked();
    const main = document.getElementById("main");
    const first = main?.querySelector(rowSelector) ?? null;
    const wide = !!wideQuery?.matches;
    hasPanel = wide && !!first && !isHome;
    if (hasPanel) pick(first);
  }

  function onPoint(e: Event) {
    if (!hasPanel) return;
    const li = (e.target as Element | null)?.closest(rowSelector);
    if (li) pick(li);
  }

  onMount(() => {
    wideQuery = window.matchMedia("(min-width: 1100px)");
    const onChange = () => setupPanel();
    wideQuery.addEventListener("change", onChange);
    setupPanel();
    return () => {
      wideQuery?.removeEventListener("change", onChange);
      document.body.style.removeProperty("overflow");
    };
  });
</script>

<svelte:window on:keydown={onKey} />

{#if fullWidth}
  <slot />
{:else}
  <a class="skip" href="#main">Skip to content</a>

  <!-- Background stage: decorative only. -->
  <div class="stage" class:home={isHome} aria-hidden="true">
    <div class="contours">
      <svg viewBox="0 0 1200 800" preserveAspectRatio="xMidYMid slice">
        <g fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round">
          <path d="M120 420C110 330 190 250 290 232 380 216 430 150 520 150 630 150 660 230 740 250 850 278 960 250 1030 330 1100 410 1070 520 1000 580 930 640 840 620 760 660 670 705 610 760 500 748 390 736 340 660 280 618 200 562 128 520 120 420Z" />
          <path d="M200 420C196 352 252 296 322 284 392 272 450 214 524 216 604 218 630 280 698 296 780 316 872 296 922 356 972 416 950 490 900 534 850 578 786 560 724 592 660 624 622 676 540 668 460 660 410 604 362 574 300 536 204 496 200 420Z" />
          <path d="M290 420C290 374 330 338 380 330 432 322 476 284 532 288 586 292 604 336 656 346 716 358 776 346 810 392 844 438 826 482 790 510 754 538 706 524 660 548 614 572 590 608 534 602 478 596 444 556 410 536 362 514 290 478 290 420Z" />
          <path d="M390 420C392 392 416 372 448 368 482 364 508 340 544 344 576 348 586 376 616 384 652 394 690 388 708 418 726 448 714 470 692 484 668 498 640 490 612 504 584 518 566 538 534 534 502 530 482 506 462 494 432 480 388 456 390 420Z" />
          <path d="M470 424C472 410 484 400 500 398 516 396 530 384 548 386 564 388 570 402 584 406 602 410 620 408 628 422 636 436 630 446 620 452 608 458 596 454 584 460 572 466 566 474 552 472 538 470 530 460 520 454 506 448 468 440 470 424Z" />
          <path d="M-20 180C100 150 160 190 250 150 340 112 420 90 520 110 620 130 700 80 800 70 900 60 1000 90 1220 40" />
          <path d="M-20 215C100 190 170 226 252 188 336 150 424 130 520 150 614 168 702 118 802 108 904 98 1004 130 1220 82" />
          <path d="M-20 700C120 720 200 680 300 700 420 724 480 780 600 790 740 800 820 740 940 760 1060 780 1140 740 1220 720" />
          <path d="M-20 740C130 756 210 718 304 738 418 760 490 820 604 830 738 840 826 780 944 798 1062 816 1144 780 1220 764" />
          <path d="M1060 -20C1040 80 1090 150 1070 250 1050 350 1130 420 1110 520 1090 620 1160 700 1140 820" stroke-dasharray="5 7" />
          <path d="M-20 560C80 540 160 590 240 570 330 548 360 480 450 470" stroke-dasharray="5 7" />
        </g>
      </svg>
    </div>
    <div class="halo"></div>
    <div class="rays"></div>
    <div class="fog fog-a"></div>
    <div class="fog fog-b"></div>
    <Embers />
    <div class="vignette"></div>
  </div>

  <div class="veil" bind:this={veil} aria-hidden="true"></div>

  <div class="frame" class:home={isHome} class:has-panel={hasPanel}>
    <header class="rail">
      <button type="button" class="menu-btn" aria-haspopup="dialog" on:click={openMenu}>
        <span class="diamond" aria-hidden="true"></span>Menu
      </button>

      <a href="/" class="brand" aria-label="ifkash.dev home">
        <svg viewBox="0 0 24 24" aria-hidden="true" class="sigil">
          <path d="M12 1.5 22.5 12 12 22.5 1.5 12Z" fill="none" stroke="currentColor" stroke-width="1" />
          <path d="M12 6.5 17.5 12 12 17.5 6.5 12Z" fill="currentColor" />
        </svg>
        <span class="brand-text">
          <span class="brand-name">Kashiful</span>
          <span class="brand-sub">ML Engineer</span>
        </span>
      </a>

      <nav class="menu" aria-label="Primary" use:menuNav={{ key: currentPath, global: true }}>
        <ul>
          {#each menuItems as item}
            <li class:game={item.href === "/game"}>
              <a
                href={item.href}
                aria-current={isActive(item.href, currentPath) ? "page" : undefined}
              >
                {item.label}
              </a>
            </li>
          {/each}
        </ul>
        <span class="selector" aria-hidden="true"></span>
      </nav>

      <div class="tools">
        <div class="tool tool-weather"><WeatherHeadsUp /></div>
        <div class="tool tool-command"><CommandDeck /></div>
        <div class="tool tool-design"><DesignSwitch /></div>
        <ThemeToggle class="opt">
          <span slot="default" let:night class="opt-inner">
            <span class="diamond" aria-hidden="true"></span>
            <span>{night ? copy.dark : copy.light}</span>
          </span>
        </ThemeToggle>
      </div>
    </header>

    <div class="col">
      <!-- svelte-ignore a11y-mouse-events-have-key-events -->
      <main id="main" class="main" on:mouseover={onPoint} on:focusin={onPoint}>
        <slot />
      </main>

      <footer class="footer">
        <div class="footer-rule" aria-hidden="true"></div>
        <nav class="footer-links" aria-label="Footer">
          {#each footerGroups as group}
            {#each group.links as link}
              {#if link.external}
                <a href={link.href} target="_blank" rel="noopener noreferrer">{link.label}</a>
              {:else}
                <a href={link.href}>{link.label}</a>
              {/if}
            {/each}
          {/each}
        </nav>
        <p class="footer-base">
          <span>© {year} {owner}</span>
          {#if $clock}<time class="footer-time">{$clock} IST</time>{/if}
        </p>
      </footer>
    </div>

    {#if hasPanel}
      <aside class="itempanel ember-frame" aria-hidden="true">
        {#key panel.id}
          <div class="ip-body">
            <p class="ip-kicker">{panel.n ? `No. ${panel.n}` : "Item"}</p>
            <h2 class="ip-title">{panel.title}</h2>
            <div class="ip-rule"></div>
            {#if panel.stats.length}
              <dl class="ip-stats">
                {#each panel.stats as s}
                  <div><dt>{s.k}</dt><dd>{s.v}</dd></div>
                {/each}
              </dl>
            {/if}
            {#if panel.desc}<p class="ip-desc">{panel.desc}</p>{/if}
            {#if panel.href}<p class="ip-open"><span class="diamond"></span>Enter to open</p>{/if}
          </div>
        {/key}
      </aside>
    {/if}
  </div>

  <div class="hints" aria-label="Keyboard shortcuts">
    <span><i class="diamond" aria-hidden="true"></i><kbd>↑</kbd><kbd>↓</kbd> Select</span>
    <span><i class="diamond" aria-hidden="true"></i><kbd>Enter</kbd> Open</span>
    <span><i class="diamond" aria-hidden="true"></i><kbd>Esc</kbd> Menu</span>
    <span><i class="diamond" aria-hidden="true"></i><kbd>H</kbd> Home</span>
    {#if $clock}<time class="hints-time">{$clock} IST</time>{/if}
  </div>

  {#if menuOpen}
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions a11y-no-static-element-interactions -->
    <div
      class="ember-overlay"
      role="dialog"
      aria-modal="true"
      aria-label="Menu"
      tabindex="-1"
      bind:this={overlayEl}
      on:keydown={trapTab}
      on:click|self={() => closeMenu()}
    >
      <div class="overlay-inner">
        <p class="overlay-kicker">Menu</p>
        <div class="overlay-rule" aria-hidden="true"></div>
        <nav class="menu overlay-menu" aria-label="Menu" use:menuNav={{ key: currentPath, global: true }}>
          <ul>
            <li>
              <a href="/" aria-current={currentPath === "/" ? "page" : undefined} on:click={() => closeMenu(false)}>Home</a>
            </li>
            {#each menuItems as item}
              <li class:game={item.href === "/game"}>
                <a
                  href={item.href}
                  aria-current={isActive(item.href, currentPath) ? "page" : undefined}
                  on:click={() => closeMenu(false)}
                >
                  {item.label}
                </a>
              </li>
            {/each}
          </ul>
          <span class="selector" aria-hidden="true"></span>
        </nav>
        <div class="overlay-opts tools">
          <div class="tool tool-design"><DesignSwitch /></div>
          <ThemeToggle class="opt">
            <span slot="default" let:night class="opt-inner">
              <span class="diamond" aria-hidden="true"></span>
              <span>{night ? copy.dark : copy.light}</span>
            </span>
          </ThemeToggle>
        </div>
        <button type="button" class="return" on:click={() => closeMenu()}>
          <span class="diamond" aria-hidden="true"></span>Return <kbd>Esc</kbd>
        </button>
      </div>
    </div>
  {/if}
{/if}

<style>
  .skip {
    position: absolute;
    top: -100px;
    left: 16px;
    z-index: 200;
    padding: 8px 14px;
    background: var(--ink);
    color: var(--ink-inverse);
    font-family: var(--font-display);
    font-size: 0.8rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }
  .skip:focus {
    top: 8px;
  }

  .diamond {
    display: inline-block;
    flex: none;
    width: 7px;
    height: 7px;
    background: currentColor;
    transform: rotate(45deg);
  }

  /* ── Stage ───────────────────────────────────────────────── */

  .stage {
    position: fixed;
    inset: 0;
    z-index: 0;
    overflow: hidden;
    pointer-events: none;
    --halo-x: 62%;
    --halo-y: 14%;
  }
  .stage.home {
    --halo-x: 50%;
    --halo-y: 32%;
  }

  .halo {
    position: absolute;
    left: var(--halo-x);
    top: var(--halo-y);
    width: min(120vw, 900px);
    aspect-ratio: 1;
    margin: calc(min(120vw, 900px) / -2) 0 0 calc(min(120vw, 900px) / -2);
    background: radial-gradient(closest-side, rgb(var(--ember-glow-rgb) / 26%), rgb(var(--ember-glow-rgb) / 7%) 55%, transparent 72%);
    animation: breathe 9s ease-in-out infinite;
    will-change: transform;
  }
  .stage.home .halo {
    background: radial-gradient(closest-side, rgb(var(--ember-glow-rgb) / 34%), rgb(var(--ember-glow-rgb) / 9%) 55%, transparent 72%);
  }

  /* Faint rays fanning out from the halo. */
  .rays {
    position: absolute;
    inset: 0;
    background: repeating-conic-gradient(
      from 188deg at var(--halo-x) var(--halo-y),
      rgb(var(--ember-glow-rgb) / 0%) 0deg 3deg,
      rgb(var(--ember-glow-rgb) / 9%) 4.5deg,
      rgb(var(--ember-glow-rgb) / 0%) 6.5deg 10deg
    );
    -webkit-mask-image: radial-gradient(ellipse 70% 80% at var(--halo-x) var(--halo-y), #000, transparent 75%);
    mask-image: radial-gradient(ellipse 70% 80% at var(--halo-x) var(--halo-y), #000, transparent 75%);
    opacity: 0.8;
    animation: pulse 14s ease-in-out infinite;
  }
  .stage:not(.home) .rays {
    opacity: 0.45;
  }

  .fog {
    position: absolute;
    left: 0;
    width: 200%;
    height: 46%;
    opacity: 0.8;
    will-change: transform;
  }
  .fog-a {
    bottom: -4%;
    background:
      radial-gradient(ellipse 22% 46% at 12% 60%, rgb(120 130 150 / 10%), transparent),
      radial-gradient(ellipse 18% 40% at 38% 50%, rgb(140 130 110 / 9%), transparent),
      radial-gradient(ellipse 24% 50% at 66% 62%, rgb(120 130 150 / 10%), transparent),
      radial-gradient(ellipse 18% 40% at 90% 52%, rgb(140 130 110 / 8%), transparent);
    animation: drift 85s linear infinite;
  }
  .fog-b {
    bottom: 8%;
    height: 38%;
    background:
      radial-gradient(ellipse 20% 42% at 20% 50%, rgb(110 120 140 / 8%), transparent),
      radial-gradient(ellipse 22% 46% at 52% 58%, rgb(150 135 105 / 7%), transparent),
      radial-gradient(ellipse 18% 40% at 80% 48%, rgb(110 120 140 / 8%), transparent);
    animation: drift 120s linear infinite reverse;
  }

  .vignette {
    position: absolute;
    inset: 0;
    background: radial-gradient(ellipse at 50% 45%, transparent 42%, var(--bg-deep) 135%);
    opacity: 0.95;
  }

  .contours {
    position: absolute;
    inset: 0;
    display: none;
    color: #5a4018;
    opacity: 0.2;
  }
  .contours svg {
    width: 100%;
    height: 100%;
    max-width: none;
  }
  :global(html[data-design="ember"]:not([data-theme="dark"])) .contours {
    display: block;
  }
  :global(html[data-design="ember"]:not([data-theme="dark"])) .halo {
    opacity: 0.7;
  }
  :global(html[data-design="ember"]:not([data-theme="dark"])) .fog {
    display: none;
  }
  :global(html[data-design="ember"]:not([data-theme="dark"])) .rays {
    opacity: 0.35;
  }
  :global(html[data-design="ember"]:not([data-theme="dark"])) .vignette {
    opacity: 0.55;
  }

  @keyframes breathe {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.06); }
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.55; }
    50% { opacity: 0.9; }
  }
  @keyframes drift {
    from { transform: translateX(0); }
    to { transform: translateX(-50%); }
  }

  /* ── Page transition veil ────────────────────────────────── */

  .veil {
    position: fixed;
    inset: 0;
    z-index: 150;
    visibility: hidden;
    opacity: 0;
    pointer-events: none;
    background: #050404;
  }
  .veil::after {
    content: "";
    position: absolute;
    inset: 0;
    background: radial-gradient(circle at 50% 48%, rgb(240 200 110 / 55%), rgb(240 160 70 / 18%) 22%, transparent 42%);
    opacity: 0;
  }
  .veil:global([data-phase="in"]) {
    visibility: visible;
    opacity: 1;
    transition: opacity 200ms ease-in;
  }
  .veil:global([data-phase="out"]) {
    visibility: visible;
    animation: veil-out 800ms ease-out forwards;
  }
  .veil:global([data-phase="out"])::after {
    animation: flare 800ms ease-out forwards;
  }
  @keyframes veil-out {
    from { opacity: 1; }
    to { opacity: 0; visibility: hidden; }
  }
  @keyframes flare {
    0% { opacity: 0.9; transform: scale(0.5); }
    100% { opacity: 0; transform: scale(1.8); }
  }

  /* ── Frame ───────────────────────────────────────────────── */

  .frame {
    position: relative;
    z-index: 1;
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    align-items: start;
    min-height: 100vh;
  }
  .frame.has-panel {
    grid-template-columns: 200px minmax(0, 1fr) 310px;
  }
  .frame.home {
    grid-template-columns: minmax(0, 1fr);
  }

  /* ── Rail ────────────────────────────────────────────────── */

  .rail {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    gap: 28px;
    height: 100vh;
    padding: 36px 22px 64px 28px;
    background: linear-gradient(90deg, rgb(0 0 0 / 48%), transparent);
  }
  :global(html[data-design="ember"]:not([data-theme="dark"])) .rail {
    background: linear-gradient(90deg, rgb(110 80 30 / 14%), transparent);
  }
  .rail::after {
    content: "";
    position: absolute;
    top: 6%;
    right: 0;
    bottom: 6%;
    width: 1px;
    background: linear-gradient(180deg, transparent, var(--line-strong) 25%, var(--line-strong) 75%, transparent);
  }

  .menu-btn {
    display: none;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--ink);
  }
  .brand:hover {
    color: var(--ember-title);
  }
  .sigil {
    width: 26px;
    height: 26px;
    color: var(--fjord-strong);
    filter: drop-shadow(0 0 6px rgb(var(--ember-glow-rgb) / 55%));
  }
  .brand-text {
    display: grid;
    gap: 2px;
  }
  .brand-name {
    font-family: var(--font-display);
    font-size: 1.05rem;
    font-weight: 600;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    line-height: 1;
  }
  .brand-sub {
    color: var(--ink-3);
    font-family: var(--font-display);
    font-size: 0.6rem;
    letter-spacing: 0.3em;
    text-transform: uppercase;
  }

  /* Menu: shared by the rail, the overlay, and (styled the same) the home page. */
  .menu {
    position: relative;
  }
  .menu ul {
    list-style: none;
  }
  .menu a {
    position: relative;
    z-index: 1;
    display: block;
    padding: 10px 14px 10px 24px;
    color: var(--ink-2);
    font-family: var(--font-display);
    font-size: 0.86rem;
    font-weight: 500;
    letter-spacing: 0.24em;
    text-transform: uppercase;
    transition: color var(--dur-fast) var(--ease), text-shadow var(--dur-fast) var(--ease);
  }
  .menu a:hover,
  .menu a:focus-visible {
    color: var(--ember-title);
    text-shadow: 0 0 14px rgb(var(--ember-glow-rgb) / 55%);
  }
  .menu a:focus-visible {
    outline: none;
  }
  .menu a[aria-current="page"] {
    color: var(--ember-title);
  }
  .menu li.game a {
    color: var(--fjord-strong);
  }
  :global(html[data-design="ember"]:not([data-theme="dark"])) .menu li.game a {
    color: #8c241e;
  }

  .selector {
    position: absolute;
    top: 0;
    right: 0;
    left: 0;
    z-index: 0;
    height: var(--sh, 0);
    transform: translateY(var(--sy, 0));
    background: linear-gradient(90deg, rgb(var(--ember-glow-rgb) / 24%), rgb(var(--ember-glow-rgb) / 10%) 55%, transparent);
    border-bottom: 1px solid transparent;
    border-image: linear-gradient(90deg, var(--fjord-strong), transparent 85%) 1;
    opacity: 0;
    pointer-events: none;
    transition:
      transform 280ms var(--ease-out),
      height 280ms var(--ease-out),
      opacity 200ms var(--ease);
  }
  .selector:global([data-instant]) {
    transition: none;
  }
  .selector:global([data-state="rest"]) {
    opacity: 0.6;
  }
  .selector::before {
    content: "";
    position: absolute;
    top: 50%;
    left: 6px;
    width: 7px;
    height: 7px;
    margin-top: -3.5px;
    background: var(--fjord-strong);
    box-shadow: 0 0 10px rgb(var(--ember-glow-rgb) / 80%);
    transform: rotate(45deg);
  }

  .tools {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    margin-top: auto;
  }
  .tool {
    display: flex;
    align-items: center;
    color: var(--ink-2);
  }
  .tools :global(.design-switch) {
    height: auto;
  }
  .tools :global(.design-switch select) {
    height: 34px;
    padding: 0 28px 0 24px;
    border: 0;
    border-radius: 0;
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-display);
    font-size: 0.72rem;
    font-weight: 500;
    letter-spacing: 0.24em;
    text-transform: uppercase;
  }
  .tools :global(.design-switch select:hover) {
    background: transparent;
    color: var(--ember-title);
    text-shadow: 0 0 12px rgb(var(--ember-glow-rgb) / 55%);
  }
  .tools :global(.design-switch select option) {
    background: var(--surface);
    color: var(--ink);
  }
  .tools :global(.design-switch .chevron) {
    right: 4px;
    color: var(--ink-3);
  }
  .tools :global(.design-switch)::before {
    content: "";
    position: absolute;
    left: 8px;
    top: 50%;
    width: 6px;
    height: 6px;
    margin-top: -3px;
    border: 1px solid var(--ink-3);
    transform: rotate(45deg);
    pointer-events: none;
  }
  .tools :global(.theme-toggle.opt) {
    width: auto;
    height: 34px;
    padding: 0 12px 0 10px;
    border: 0;
    color: var(--ink-2);
    font-family: var(--font-display);
    font-size: 0.72rem;
    font-weight: 500;
    letter-spacing: 0.24em;
    text-transform: uppercase;
  }
  .tools :global(.theme-toggle.opt:hover) {
    background: transparent;
    color: var(--ember-title);
    text-shadow: 0 0 12px rgb(var(--ember-glow-rgb) / 55%);
  }
  .opt-inner {
    display: inline-flex;
    align-items: center;
    gap: 12px;
  }
  .opt-inner .diamond {
    width: 6px;
    height: 6px;
    background: var(--fjord);
    box-shadow: 0 0 8px rgb(var(--ember-glow-rgb) / 70%);
  }
  .tools :global(.weather-trigger),
  .tools :global(.command-trigger) {
    font-family: var(--font-display);
  }

  /* ── Column and main ─────────────────────────────────────── */

  .col {
    min-width: 0;
    padding-bottom: 56px;
  }
  .main {
    max-width: 900px;
    min-height: 62vh;
    margin: 0 auto;
    padding: 0 36px 72px;
  }
  .frame.home .main {
    max-width: none;
    padding: 0;
  }

  /* ── Item description panel ──────────────────────────────── */

  .itempanel {
    position: sticky;
    top: 40px;
    min-height: 320px;
    margin: 40px 36px 0 0;
    padding: 34px 28px 30px;
    border: 1px solid var(--line);
    background: linear-gradient(180deg, rgb(var(--ember-glow-rgb) / 7%), transparent 40%), var(--ember-panel);
    box-shadow: 0 18px 40px -20px var(--shadow-color);
  }
  .ip-body {
    animation: ip-in 220ms var(--ease-out) both;
  }
  @keyframes ip-in {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: none; }
  }
  .ip-kicker {
    margin: 0 0 8px;
    color: var(--fjord);
    font-family: var(--font-mono);
    font-size: 0.7rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
  }
  .ip-title {
    margin: 0;
    color: var(--ember-title);
    font-size: 1.2rem;
    line-height: 1.3;
    letter-spacing: 0.14em;
    text-shadow: 0 0 16px rgb(var(--ember-glow-rgb) / 35%);
  }
  .ip-rule {
    position: relative;
    height: 1px;
    margin: 16px 0 14px;
    background: linear-gradient(90deg, var(--fjord), transparent);
  }
  .ip-rule::after {
    content: "";
    position: absolute;
    top: -3px;
    left: 0;
    width: 7px;
    height: 7px;
    background: var(--fjord-strong);
    transform: rotate(45deg);
  }
  .ip-stats {
    margin: 0 0 16px;
  }
  .ip-stats div {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    padding: 5px 0;
    border-bottom: 1px solid var(--line);
  }
  .ip-stats dt {
    color: var(--ink-3);
    font-family: var(--font-display);
    font-size: 0.68rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    white-space: nowrap;
    padding-top: 4px;
  }
  .ip-stats dd {
    margin: 0;
    color: var(--ink);
    font-size: 1.05rem;
    text-align: right;
  }
  .ip-desc {
    margin: 0 0 18px;
    color: var(--ink-2);
    font-size: 1.15rem;
    font-style: italic;
    line-height: 1.5;
  }
  .ip-open {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0;
    color: var(--fjord);
    font-family: var(--font-display);
    font-size: 0.68rem;
    letter-spacing: 0.22em;
    text-transform: uppercase;
  }
  .ip-open .diamond {
    width: 6px;
    height: 6px;
  }

  /* ── Footer ──────────────────────────────────────────────── */

  .footer {
    max-width: 900px;
    margin: 0 auto;
    padding: 0 36px;
    text-align: center;
  }
  .footer-rule {
    position: relative;
    height: 1px;
    margin-bottom: 26px;
    background: linear-gradient(90deg, transparent, var(--line-strong) 20%, var(--line-strong) 80%, transparent);
  }
  .footer-rule::after {
    content: "";
    position: absolute;
    top: -4px;
    left: 50%;
    width: 8px;
    height: 8px;
    margin-left: -4px;
    border: 1px solid var(--fjord);
    background: var(--bg);
    transform: rotate(45deg);
  }
  .footer-links {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 6px 26px;
  }
  .footer-links a {
    color: var(--ink-2);
    font-family: var(--font-display);
    font-size: 0.7rem;
    letter-spacing: 0.22em;
    text-transform: uppercase;
  }
  .footer-links a:hover {
    color: var(--ember-title);
  }
  .footer-base {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 4px 20px;
    margin: 18px 0 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.06em;
  }
  .frame.home .footer {
    display: none;
  }

  /* ── Hints bar ───────────────────────────────────────────── */

  .hints {
    position: fixed;
    right: 0;
    bottom: 0;
    left: 0;
    z-index: 30;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 34px;
    height: 44px;
    padding: 0 28px;
    background: linear-gradient(0deg, var(--bg-deep), transparent);
    color: var(--ink-2);
    font-family: var(--font-display);
    font-size: 0.68rem;
    letter-spacing: 0.22em;
    text-transform: uppercase;
    pointer-events: none;
  }
  .hints span {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .hints .diamond {
    width: 5px;
    height: 5px;
    margin-right: 4px;
    background: transparent;
    border: 1px solid var(--fjord);
  }
  .hints kbd {
    min-width: 0;
    padding: 1px 6px;
    border: 1px solid var(--line-strong);
    border-bottom-width: 1px;
    border-radius: 1px;
    background: transparent;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: 0.66rem;
    letter-spacing: 0.04em;
    text-transform: none;
  }
  .hints-time {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 0.68rem;
    letter-spacing: 0.06em;
  }

  /* ── Overlay menu ────────────────────────────────────────── */

  .ember-overlay {
    position: fixed;
    inset: 0;
    z-index: 120;
    display: grid;
    place-items: center;
    overflow-y: auto;
    padding: 24px;
    background:
      radial-gradient(ellipse at 50% 30%, rgb(var(--ember-glow-rgb) / 14%), transparent 60%),
      color-mix(in srgb, var(--bg-deep) 94%, transparent);
    animation: overlay-in 260ms var(--ease-out) both;
  }
  @keyframes overlay-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  .overlay-inner {
    display: grid;
    justify-items: center;
    gap: 18px;
    width: min(100%, 360px);
  }
  .overlay-kicker {
    margin: 0;
    color: var(--ember-title);
    font-family: var(--font-display);
    font-size: 1.5rem;
    font-weight: 600;
    letter-spacing: 0.4em;
    margin-right: -0.4em;
    text-transform: uppercase;
    text-shadow: 0 0 20px rgb(var(--ember-glow-rgb) / 45%);
  }
  .overlay-rule {
    width: 100%;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--fjord) 25%, var(--fjord) 75%, transparent);
  }
  .overlay-menu {
    width: 100%;
    text-align: center;
  }
  .overlay-menu a {
    padding: 11px 24px;
    font-size: 0.95rem;
  }
  .overlay-opts {
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: center;
    margin: 0;
  }
  .return {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border: 0;
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-display);
    font-size: 0.72rem;
    letter-spacing: 0.24em;
    text-transform: uppercase;
    cursor: pointer;
  }
  .return:hover {
    color: var(--ember-title);
  }
  .return .diamond {
    width: 6px;
    height: 6px;
    background: var(--fjord);
  }

  /* ── Home: no rail, tools float top right ────────────────── */

  @media (min-width: 1000px) {
    .frame.home .rail {
      position: fixed;
      top: 0;
      right: 0;
      left: auto;
      z-index: 40;
      flex-direction: row;
      width: auto;
      height: auto;
      padding: 14px 22px;
      background: none;
    }
    .frame.home .rail::after,
    .frame.home .brand,
    .frame.home .rail > .menu {
      display: none;
    }
    .frame.home .tools {
      flex-direction: row;
      align-items: center;
      margin: 0;
    }
  }

  /* ── Small screens: top bar plus overlay menu ────────────── */

  @media (max-width: 999px) {
    .frame,
    .frame.has-panel {
      grid-template-columns: minmax(0, 1fr);
    }
    .rail,
    .frame.home .rail {
      position: sticky;
      top: 0;
      z-index: 40;
      flex-direction: row;
      align-items: center;
      gap: 10px;
      width: auto;
      height: 56px;
      padding: 0 12px;
      background: color-mix(in srgb, var(--bg-deep) 88%, transparent);
    }
    .rail::after {
      top: auto;
      right: 8%;
      bottom: 0;
      left: 8%;
      width: auto;
      height: 1px;
      background: linear-gradient(90deg, transparent, var(--line-strong), transparent);
    }
    .menu-btn {
      display: inline-flex;
      align-items: center;
      gap: 10px;
      min-height: 40px;
      padding: 0 10px;
      border: 0;
      background: transparent;
      color: var(--ember-title);
      font-family: var(--font-display);
      font-size: 0.78rem;
      letter-spacing: 0.24em;
      text-transform: uppercase;
      cursor: pointer;
    }
    .menu-btn .diamond {
      width: 7px;
      height: 7px;
      box-shadow: 0 0 8px rgb(var(--ember-glow-rgb) / 70%);
    }
    .brand-sub,
    .rail > .menu {
      display: none;
    }
    .brand {
      margin-left: 4px;
    }
    .sigil {
      width: 20px;
      height: 20px;
    }
    .brand-name {
      font-size: 0.82rem;
    }
    .frame.home .brand {
      display: none;
    }
    .tools {
      flex-direction: row;
      align-items: center;
      margin: 0 0 0 auto;
    }
    .rail .tool-weather,
    .rail .tool-design {
      display: none;
    }
    .main {
      padding: 0 18px 56px;
    }
    .footer {
      padding: 0 18px;
    }
    .col {
      padding-bottom: 24px;
    }
    .hints {
      display: none;
    }
    .itempanel {
      display: none;
    }
  }

  @media (max-width: 420px) {
    .brand-text {
      display: none;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .halo,
    .rays,
    .fog {
      animation: none;
    }
    .ember-overlay,
    .ip-body {
      animation: none;
    }
    .selector {
      transition: none;
    }
  }
</style>
