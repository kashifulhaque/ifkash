<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import AiAgent from "$lib/components/AiAgent.svelte";
  import CommandDeck from "$lib/components/CommandDeck.svelte";
  import WeatherHeadsUp from "$lib/components/WeatherHeadsUp.svelte";
  import DesignSwitch from "$lib/components/DesignSwitch.svelte";
  import { design, type Design } from "$lib/stores/design";
  import { theme, toggleTheme } from "$lib/stores/theme";
  import { playDayNightCycle } from "$lib/dayNightCycle";

  $: currentPath = $page.url.pathname;
  $: isFullWidth =
    currentPath === "/game" ||
    currentPath === "/editor" ||
    currentPath === "/admin/notes" ||
    (currentPath.startsWith("/tools/") && currentPath !== "/tools");

  const navItems = [
    { href: "/work", label: "Work" },
    { href: "/projects", label: "Projects" },
    { href: "/rl", label: "RL" },
    { href: "/roadmap", label: "Roadmap" },
    { href: "/tools", label: "Tools" },
    { href: "/education", label: "Education" },
    { href: "/blog", label: "Blog" },
  ];

  function isActive(href: string, path: string) {
    return path === href || path.startsWith(href + "/");
  }

  // Local time in Bangalore, shown in the footer's coordinate line.
  let clock = "";
  onMount(() => {
    const fmt = new Intl.DateTimeFormat("en-GB", {
      hour: "2-digit",
      minute: "2-digit",
      timeZone: "Asia/Kolkata",
    });
    const tick = () => (clock = fmt.format(new Date()));
    tick();
    const id = setInterval(tick, 15_000);
    return () => clearInterval(id);
  });

  const year = new Date().getFullYear();

  const themeCopyFor: Record<
    Design,
    { toLight: string; toDark: string; light: string; dark: string }
  > = {
    nordisk: {
      toLight: "Switch to daylight",
      toDark: "Switch to polar night",
      light: "Daylight",
      dark: "Polar night",
    },
    broadsheet: {
      toLight: "Switch to day edition",
      toDark: "Switch to night edition",
      light: "Day edition",
      dark: "Night edition",
    },
    atelier: {
      toLight: "Switch to lamp light",
      toDark: "Switch to the kiln",
      light: "Lamp light",
      dark: "Kiln",
    },
  };

  $: themeCopy = themeCopyFor[$design];

  // Plays the full sunset or sunrise, and flips the theme partway through.
  let cycling = false;
  async function onThemeClick(event: MouseEvent) {
    if (cycling) return;
    cycling = true;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const to = $theme === "dark" ? "light" : "dark";
    try {
      await playDayNightCycle({
        origin: { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 },
        to,
        label: to === "dark" ? themeCopy.dark : themeCopy.light,
        onSwitch: toggleTheme,
      });
    } finally {
      cycling = false;
    }
  }
</script>

{#if isFullWidth}
  <slot />
{:else}
  <a class="skip" href="#main">Skip to content</a>

  <header class="topbar">
    <div class="topbar-inner">
      <a href="/" class="wordmark" aria-label="ifkash.dev home">
        <span class="dot" aria-hidden="true"></span>
        <span>ifkash</span>
      </a>

      <nav class="nav" aria-label="Primary">
        <ul>
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
        </ul>
      </nav>

      <div class="tools">
        <WeatherHeadsUp />
        <CommandDeck />
        <DesignSwitch />
        <button
          type="button"
          class="theme-toggle"
          on:click={onThemeClick}
          aria-label={$theme === "dark" ? themeCopy.toLight : themeCopy.toDark}
          title={$theme === "dark" ? themeCopy.light : themeCopy.dark}
        >
          <!-- A sun on the horizon: risen in daylight, set at night. -->
          <svg viewBox="0 0 24 24" aria-hidden="true" class:night={$theme === "dark"}>
            <defs>
              <clipPath id="horizon-clip"><rect x="0" y="0" width="24" height="16" /></clipPath>
            </defs>
            <g clip-path="url(#horizon-clip)">
              <circle class="sun" cx="12" cy="12" r="5" />
            </g>
            <path d="M3 16h18" />
            <path class="water" d="M7 19.5h10" />
          </svg>
        </button>
      </div>
    </div>
  </header>

  <main id="main" class="main">
    <slot />
  </main>

  <footer class="footer">
    <div class="footer-inner">
      <div class="footer-sign">
        <p class="footer-name">Kashiful Haque</p>
        <p class="footer-line">
          ML engineer building language-model systems, reinforcement learning
          pipelines, and fast inference in C++ and Rust.
        </p>
      </div>

      <nav class="footer-cols" aria-label="Footer">
        <div>
          <p class="eyebrow">Elsewhere</p>
          <ul>
            <li><a href="https://github.com/kashifulhaque" target="_blank" rel="noopener noreferrer">GitHub</a></li>
            <li><a href="https://hf.co/ifkash" target="_blank" rel="noopener noreferrer">Hugging Face</a></li>
            <li><a href="https://linkedin.com/in/kashifulhaque" target="_blank" rel="noopener noreferrer">LinkedIn</a></li>
          </ul>
        </div>
        <div>
          <p class="eyebrow">Here</p>
          <ul>
            <li><a href="/api/cv?format=view">Résumé</a></li>
            <li><a href="/game">Tiny planet</a></li>
            <li><a href="/fitness">Fitness</a></li>
            <li><a href="/admin">Admin</a></li>
          </ul>
        </div>
      </nav>

      <div class="footer-base">
        <span>© {year} Kashiful Haque</span>
        <span class="coords">
          Bengaluru 12.97° N, 77.59° E{#if clock}<span class="sep">·</span><time>{clock} IST</time>{/if}
        </span>
      </div>
    </div>
  </footer>

  <AiAgent />
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
    background: color-mix(in srgb, var(--bg) 90%, transparent);
    backdrop-filter: saturate(1.2) blur(12px);
    -webkit-backdrop-filter: saturate(1.2) blur(12px);
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
    font-size: 17px;
    font-weight: 500;
    letter-spacing: -0.025em;
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

  /* Component selectors repeat their scope class, so the Broadsheet
     masthead has to live here to outrank the Nordisk top bar. */
  :global(html[data-design="broadsheet"]) .topbar {
    background: var(--bg);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    border-bottom: 3px double var(--ink);
  }

  :global(html[data-design="broadsheet"]) .wordmark,
  :global(html[data-design="broadsheet"]) .footer-name {
    font-family: var(--font-display);
    font-weight: 450;
    letter-spacing: -0.02em;
  }

  :global(html[data-design="broadsheet"]) .footer {
    background: var(--bg);
    border-top: 3px double var(--ink);
  }

  /* Atelier reads as a plaster sheet: solid bar, stronger rule, italic wordmark. */
  :global(html[data-design="atelier"]) .topbar {
    background: var(--bg);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    border-bottom: 1px solid var(--line-strong);
  }

  :global(html[data-design="atelier"]) .wordmark,
  :global(html[data-design="atelier"]) .footer-name {
    font-family: var(--font-display);
    font-weight: 560;
    letter-spacing: -0.02em;
    font-optical-sizing: none;
    font-variation-settings: "SOFT" 20, "WONK" 0, "opsz" 48, "wght" 560;
  }

  :global(html[data-design="atelier"]) .wordmark {
    font-style: italic;
    font-weight: 520;
    font-variation-settings: "SOFT" 70, "WONK" 1, "opsz" 48, "wght" 520;
  }

  :global(html[data-design="atelier"]) .footer {
    background: var(--bg);
    border-top: 1px solid var(--line-strong);
  }

  .nav ul {
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

  .tools {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .theme-toggle {
    display: inline-grid;
    place-items: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-2);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
  }
  .theme-toggle:hover {
    color: var(--ink);
    background: var(--hover-wash);
  }
  .theme-toggle svg {
    width: 20px;
    height: 20px;
    overflow: visible;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
  }
  .theme-toggle .sun {
    fill: currentColor;
    stroke: none;
    transform: translateY(-1px);
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .theme-toggle .night .sun {
    transform: translateY(7px);
  }
  .theme-toggle .water {
    opacity: 0.5;
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
    background: var(--surface-2);
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
  }

  @media (max-width: 640px) {
    .footer-inner {
      grid-template-columns: 1fr;
    }
  }
</style>
