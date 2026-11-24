<script lang="ts">
  /**
   * Ember home, a title screen: a gold logo with one slow light sweep, a
   * softly pulsing "Press any key" prompt, and the main menu. Pressing a key or
   * clicking plays the banner reveal and shows the menu. The title screen and
   * the reveal play once per browser session; later visits show the menu at once. The menu is never locked
   * away: it appears after a short delay, and moving focus into it shows it
   * at once. Without JavaScript or with reduced motion it is always visible.
   */
  import { onMount } from "svelte";
  import { prefersReducedMotion } from "$lib/motion";
  import { menuNav } from "./menu";
  import { readRelicBadge, type RelicBadge } from "../relics";
  import { facts, indexRows, resumeUrl } from "../shared";

  const AUTO_REVEAL_MS = 2000;
  const SEEN_KEY = "ember-title-seen";

  let armed = false;
  let revealed = false;
  let badge: RelicBadge | null = null;
  let toast = false;
  let desc = "";

  const labelFor = (name: string) => (name === "Game" ? "Cinderheart" : name);

  function reveal() {
    if (revealed) return;
    revealed = true;
    try {
      sessionStorage.setItem(SEEN_KEY, "1");
    } catch {
      /* Storage can be blocked; the reveal then plays again next visit. */
    }
  }

  function onKey(e: KeyboardEvent) {
    if (revealed || !armed) return;
    if (["Shift", "Control", "Alt", "Meta", "Tab", "CapsLock"].includes(e.key)) return;
    // Space would also scroll the page past the menu it reveals.
    if (e.key === " " && !(e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement)) {
      e.preventDefault();
    }
    reveal();
  }

  onMount(() => {
    badge = readRelicBadge();
    let toastTimer: ReturnType<typeof setTimeout> | undefined;
    let toastHide: ReturnType<typeof setTimeout> | undefined;
    if (badge) {
      toastTimer = setTimeout(() => (toast = true), 1400);
      toastHide = setTimeout(() => (toast = false), 9000);
    }
    if (prefersReducedMotion()) return () => {
      clearTimeout(toastTimer);
      clearTimeout(toastHide);
    };
    try {
      if (sessionStorage.getItem(SEEN_KEY)) {
        return () => {
          clearTimeout(toastTimer);
          clearTimeout(toastHide);
        };
      }
    } catch {
      /* Treat blocked storage as a first visit. */
    }
    armed = true;
    const auto = setTimeout(reveal, AUTO_REVEAL_MS);
    return () => {
      clearTimeout(auto);
      clearTimeout(toastTimer);
      clearTimeout(toastHide);
    };
  });
</script>

<svelte:window on:keydown={onKey} />

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<section class="screen" class:armed class:revealed on:pointerdown={reveal}>
  <header class="titleblock">
    <p class="kicker">ifkash.dev</p>
    <h1 class="logo"><span class="logo-ink">Kashiful Haque</span></h1>
    <span class="streak" aria-hidden="true"></span>
    <div class="rule" aria-hidden="true"><span></span></div>
    <p class="sub">ML Engineer <span aria-hidden="true">◆</span> LLM systems, RL, fast inference</p>
  </header>

  {#if armed && !revealed}
    <button type="button" class="press" on:click={reveal}>Press any key</button>
  {/if}

  <nav
    class="mainmenu"
    aria-label="Main menu"
    on:focusin={reveal}
    use:menuNav={{ global: true, onPick: (el) => (desc = el?.dataset.desc ?? "") }}
  >
    <ul>
      {#each indexRows as row, i}
        <li class:game={row.name === "Game"} style="--i: {i}">
          <a href={row.href} data-desc={row.desc}>{labelFor(row.name)}</a>
        </li>
      {/each}
      <li style="--i: {indexRows.length}">
        <a href={resumeUrl} data-desc="Read the résumé" data-sveltekit-reload>Résumé</a>
      </li>
    </ul>
    <span class="selector" aria-hidden="true"></span>
  </nav>
  <p class="desc" aria-hidden="true">{desc || "Choose a path."}</p>
</section>

<section class="status ember-frame" aria-labelledby="status-title">
  <h2 id="status-title" class="status-title">Status</h2>
  <div class="status-rule" aria-hidden="true"></div>
  <dl class="sheet">
    {#each facts as f}
      <div class="attr">
        <dt>{f.k}</dt>
        <span class="leader" aria-hidden="true"></span>
        <dd>{f.v}</dd>
      </div>
    {/each}
    {#if badge}
      <div class="attr">
        <dt>Relics</dt>
        <span class="leader" aria-hidden="true"></span>
        <dd><a href={badge.href}>{badge.text}</a></dd>
      </div>
    {/if}
  </dl>
</section>

<div class="sr-only" role="status">{#if badge && toast}Relic progress: {badge.text}{/if}</div>
{#if badge && toast}
  <a class="toast" href={badge.href}>
    <span class="toast-gem" aria-hidden="true"></span>
    <span class="toast-text">
      <span class="toast-kicker">Relic progress</span>
      <span class="toast-line">{badge.text}</span>
    </span>
  </a>
{/if}

<style>
  .screen {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    min-height: 100svh;
    padding: clamp(72px, 14vh, 150px) 20px 56px;
    text-align: center;
  }

  /* ── Title ───────────────────────────────────────────────── */

  .titleblock {
    position: relative;
    display: grid;
    justify-items: center;
    gap: 14px;
    width: 100%;
  }
  .kicker {
    margin: 0;
    color: var(--fjord);
    font-family: var(--font-display);
    font-size: 0.78rem;
    letter-spacing: 0.3em;
    margin-right: -0.3em;
    text-transform: uppercase;
  }
  :global(html[data-design="ember"][data-theme="dark"]) .kicker {
    color: #d3b56a;
  }

  .logo {
    margin: 0;
    font-family: var(--font-display);
    font-size: clamp(2.1rem, 0.6rem + 7.4vw, 5.6rem);
    font-weight: 600;
    line-height: 1.05;
    letter-spacing: 0.1em;
    margin-right: -0.1em;
    text-transform: uppercase;
    filter: drop-shadow(0 0 10px rgb(var(--ember-glow-rgb) / 14%));
  }
  .logo-ink {
    background: linear-gradient(
      105deg,
      var(--logo-a) 0%,
      var(--logo-a) 38%,
      var(--logo-c) 50%,
      var(--logo-a) 62%,
      var(--logo-a) 100%
    );
    background-size: 260% 100%;
    background-position: 100% 0;
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    color: var(--ember-title);
    animation: sweep 7s ease-in-out 2.5s 1 both;
  }
  .screen {
    --logo-a: #9a741c;
    --logo-c: #5b3f0a;
  }
  :global(html[data-design="ember"][data-theme="dark"]) .screen {
    --logo-a: #c2a35e;
    --logo-c: #eadba8;
  }

  .sub {
    margin: 0;
    color: var(--ink-2);
    font-size: 1.3rem;
    font-style: italic;
  }
  .sub span {
    margin: 0 0.5em;
    color: var(--fjord);
    font-size: 0.6em;
    vertical-align: 0.2em;
  }

  .rule {
    position: relative;
    width: min(520px, 80%);
    height: 1px;
    margin-top: 6px;
    background: linear-gradient(90deg, transparent, var(--line-strong) 22%, var(--line-strong) 78%, transparent);
  }
  .rule span {
    position: absolute;
    top: -4px;
    left: 50%;
    width: 9px;
    height: 9px;
    margin-left: -4.5px;
    border: 1px solid var(--fjord);
    background: var(--bg);
    transform: rotate(45deg);
  }

  .streak {
    position: absolute;
    top: 50%;
    left: 50%;
    width: min(900px, 96vw);
    height: 2px;
    margin: -1px 0 0 calc(min(900px, 96vw) / -2);
    background: linear-gradient(90deg, transparent, var(--fjord), transparent);
    box-shadow: 0 0 16px 2px rgb(var(--ember-glow-rgb) / 12%);
    opacity: 0;
    transform: scaleX(0);
    pointer-events: none;
  }

  /* ── Press any key ───────────────────────────────────────── */

  .press {
    margin-top: clamp(40px, 9vh, 90px);
    padding: 12px 22px;
    border: 0;
    background: transparent;
    color: var(--ember-title);
    font-family: var(--font-display);
    font-size: 0.85rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    cursor: pointer;
    animation: prompt 4.5s ease-in-out infinite;
  }

  /* ── Main menu ───────────────────────────────────────────── */

  .mainmenu {
    position: relative;
    width: min(100%, 400px);
    margin-top: clamp(28px, 6vh, 56px);
  }
  .mainmenu ul {
    list-style: none;
  }
  .mainmenu a {
    position: relative;
    z-index: 1;
    display: block;
    padding: 9px 28px;
    color: var(--ink-2);
    font-family: var(--font-display);
    font-size: 1rem;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    transition: color var(--dur-fast) var(--ease);
  }
  .mainmenu a:hover,
  .mainmenu a:focus-visible {
    color: var(--ember-title);
  }
  .mainmenu a:focus-visible {
    outline: none;
  }
  .mainmenu .game a {
    color: var(--fjord);
  }
  :global(html[data-design="ember"]:not([data-theme="dark"])) .mainmenu .game a {
    color: var(--fjord-strong);
  }
  .mainmenu .game a:hover,
  .mainmenu .game a:focus-visible {
    color: var(--ember-title);
  }

  .selector {
    position: absolute;
    top: 0;
    right: 0;
    left: 0;
    z-index: 0;
    height: var(--sh, 0);
    transform: translateY(var(--sy, 0));
    background: linear-gradient(90deg, transparent, rgb(var(--ember-glow-rgb) / 10%) 25%, rgb(var(--ember-glow-rgb) / 10%) 75%, transparent);
    opacity: 0;
    pointer-events: none;
    transition: transform 280ms var(--ease-out), height 280ms var(--ease-out), opacity 200ms var(--ease);
  }
  .selector:global([data-instant]) {
    transition: none;
  }
  .selector:global([data-state="rest"]) {
    opacity: 0;
  }
  .selector::before,
  .selector::after {
    content: "";
    position: absolute;
    top: 50%;
    width: 6px;
    height: 6px;
    margin-top: -3px;
    background: var(--fjord-strong);
    transform: rotate(45deg);
  }
  .selector::before {
    left: 24%;
  }
  .selector::after {
    right: 24%;
  }

  .desc {
    min-height: 1.6em;
    margin: 20px 0 0;
    color: var(--ink-3);
    font-size: 1.15rem;
    font-style: italic;
  }

  /* ── Armed (animated) states ─────────────────────────────── */

  .armed .logo {
    animation: logo-in 1.6s ease-out both;
  }
  .armed:not(.revealed) .logo {
    letter-spacing: 0.05em;
    margin-right: -0.05em;
  }
  .armed.revealed .logo {
    animation: banner 2s cubic-bezier(0.2, 0.6, 0.2, 1) both;
  }
  .armed.revealed .streak {
    animation: streak 2.2s cubic-bezier(0.2, 0.6, 0.2, 1) both;
  }
  .armed:not(.revealed) .mainmenu,
  .armed:not(.revealed) .desc {
    opacity: 0;
    pointer-events: none;
  }
  .armed.revealed .mainmenu li {
    animation: item-in 700ms var(--ease-out) both;
    animation-delay: calc(700ms + var(--i) * 70ms);
  }
  .armed.revealed .desc {
    animation: item-in 700ms var(--ease-out) 1.4s both;
  }

  @keyframes logo-in {
    from { opacity: 0; filter: blur(4px); }
    to { opacity: 1; }
  }
  @keyframes banner {
    from { letter-spacing: 0.05em; margin-right: -0.05em; filter: blur(2px); }
    to { letter-spacing: 0.1em; margin-right: -0.1em; filter: drop-shadow(0 0 10px rgb(var(--ember-glow-rgb) / 14%)); }
  }
  @keyframes streak {
    0% { opacity: 0; transform: scaleX(0); }
    30% { opacity: 0.45; }
    100% { opacity: 0; transform: scaleX(1); }
  }
  @keyframes item-in {
    from { opacity: 0; filter: blur(4px); }
    to { opacity: 1; filter: none; }
  }
  @keyframes sweep {
    0% { background-position: 100% 0; }
    55%, 100% { background-position: 0% 0; }
  }
  @keyframes prompt {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 0.9; }
  }

  /* ── Status sheet ────────────────────────────────────────── */

  .status {
    position: relative;
    width: min(560px, calc(100% - 40px));
    margin: 0 auto 80px;
    padding: 40px 36px 34px;
    background: var(--ember-panel);
    border: 1px solid var(--line);
  }
  .status-title {
    margin: 0;
    color: var(--ink);
    font-size: 1.05rem;
    letter-spacing: 0.2em;
    text-align: center;
  }
  .status-rule {
    position: relative;
    height: 1px;
    margin: 14px 0 12px;
    background: linear-gradient(90deg, transparent, var(--line-strong) 20%, var(--line-strong) 80%, transparent);
  }
  .sheet {
    margin: 0;
  }
  .attr {
    display: flex;
    align-items: baseline;
    gap: 12px;
    padding: 9px 0;
  }
  .attr dt {
    color: var(--ink-3);
    font-family: var(--font-display);
    font-size: 0.74rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .leader {
    flex: 1;
    min-width: 12px;
    border-bottom: 1px dotted var(--line);
    transform: translateY(-4px);
  }
  .attr dd {
    margin: 0;
    color: var(--ink);
    font-size: 1.25rem;
    font-weight: 600;
    text-align: right;
  }
  .attr dd a {
    color: var(--fjord-strong);
    text-decoration: underline;
    text-decoration-color: var(--line-strong);
    text-underline-offset: 0.2em;
  }

  /* ── Pickup toast ────────────────────────────────────────── */

  .toast {
    position: fixed;
    bottom: 64px;
    left: 24px;
    z-index: 60;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 22px 12px 16px;
    border: 1px solid var(--line-strong);
    background: linear-gradient(90deg, var(--ember-panel), color-mix(in srgb, var(--ember-panel) 40%, transparent));
    box-shadow: 0 12px 30px -10px var(--shadow-color);
    color: var(--ink);
    text-align: left;
    animation: toast-in 700ms var(--ease-out) both, toast-out 600ms ease-in 7s forwards;
  }
  .toast-gem {
    width: 12px;
    height: 12px;
    background: var(--fjord);
    transform: rotate(45deg);
  }
  .toast-text {
    display: grid;
    gap: 2px;
  }
  .toast-kicker {
    color: var(--fjord);
    font-family: var(--font-display);
    font-size: 0.64rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }
  .toast-line {
    font-size: 1.1rem;
  }
  @keyframes toast-in {
    from { opacity: 0; transform: translateX(-40px); }
    to { opacity: 1; transform: none; }
  }
  @keyframes toast-out {
    to { opacity: 0; transform: translateX(-20px); }
  }

  @media (max-width: 999px) {
    .screen {
      min-height: calc(100svh - 56px);
      padding-top: clamp(40px, 9vh, 90px);
    }
    .toast {
      right: 16px;
      bottom: 16px;
      left: 16px;
    }
  }

  @media (max-width: 480px) {
    .logo {
      letter-spacing: 0.07em;
      margin-right: -0.07em;
    }
    .armed:not(.revealed) .logo {
      letter-spacing: 0.04em;
      margin-right: -0.04em;
    }
    @keyframes banner {
      from { letter-spacing: 0.04em; filter: blur(2px); }
      to { letter-spacing: 0.07em; filter: drop-shadow(0 0 10px rgb(var(--ember-glow-rgb) / 14%)); }
    }
    .status {
      padding: 34px 22px 26px;
    }
    .mainmenu a {
      letter-spacing: 0.1em;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .logo-ink,
    .press,
    .toast {
      animation: none;
    }
  }
</style>
