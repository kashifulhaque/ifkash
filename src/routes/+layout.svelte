<script lang="ts">
  import "../app.css";
  import { page } from "$app/stores";
  import AiAgent from "$lib/components/AiAgent.svelte";
  import WeatherHeadsUp from "$lib/components/WeatherHeadsUp.svelte";
  import { theme, toggleTheme } from "$lib/stores/theme";

  $: currentPath = $page.url.pathname;
  $: isFullWidth =
    currentPath === "/game" ||
    currentPath === "/editor" ||
    currentPath === "/admin/notes" ||
    (currentPath.startsWith("/tools/") && currentPath !== "/tools");

  const navItems = [
    { href: "/", label: "Home" },
    { href: "/work", label: "Work" },
    { href: "/projects", label: "Projects" },
    { href: "/rl", label: "RL" },
    { href: "/tools", label: "Tools" },
    { href: "/education", label: "Education" },
    { href: "/blog", label: "Blog" },
  ];

  function isActive(href: string, path: string) {
    return href === "/" ? path === "/" : path.startsWith(href);
  }
</script>

{#if isFullWidth}
  <slot />
{:else}
  <div class="site-wrapper">
    <header class="masthead" id="top">
      <div class="masthead-brand">
        <a href="/" class="site-name" aria-label="ifkash.dev home">/ifkash.dev</a>
        <p>ML systems, open source, and useful tools.</p>
      </div>

      <div class="masthead-actions">
        <a
          href="https://github.com/kashifulhaque"
          target="_blank"
          rel="noopener noreferrer">GitHub</a
        >
        <a href="/api/cv?format=view">Resume</a>
        <button
          type="button"
          class="theme-toggle"
          on:click={toggleTheme}
          aria-label={$theme === "dark" ? "Use light theme" : "Use dark theme"}
          title={$theme === "dark" ? "Use light theme" : "Use dark theme"}
        >
          {#if $theme === "dark"}
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <circle cx="12" cy="12" r="4" />
              <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
            </svg>
          {:else}
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
            </svg>
          {/if}
        </button>
      </div>
    </header>

    <div class="shell">
      <nav class="sites" aria-label="Primary">
        <p class="nav-label">Pages</p>
        <ol>
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
        </ol>
      </nav>

      <main class="stage">
        <div class="stage-inner">
          <slot />
        </div>

        <footer class="site-footer">
          <div>
            <p class="footer-heading">ifkash.dev</p>
            <p>Built in Bangalore by Kashiful Haque.</p>
          </div>
          <div class="footer-links">
            <a
              href="https://hf.co/ifkash"
              target="_blank"
              rel="noopener noreferrer">Hugging Face</a
            >
            <a
              href="https://linkedin.com/in/kashifulhaque"
              target="_blank"
              rel="noopener noreferrer">LinkedIn</a
            >
            <a href="/admin">Admin</a>
          </div>
          <p class="copyright">© {new Date().getFullYear()} Kashiful Haque</p>
        </footer>
      </main>
    </div>

    <AiAgent />
    <WeatherHeadsUp />
  </div>
{/if}

<style>
  .site-wrapper {
    min-height: 100vh;
  }

  .masthead {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px 48px;
    padding: 40px 48px 32px;
    background: var(--background);
  }

  .masthead-brand {
    min-width: 0;
  }

  .site-name {
    display: inline-block;
    color: var(--foreground);
    font-size: 22px;
    font-weight: 400;
    line-height: 1.1;
    letter-spacing: -0.02em;
  }

  .masthead-brand p {
    margin: 5px 0 0;
    color: var(--ink-3);
    font-size: 13px;
    line-height: 1.5;
  }

  .masthead-actions {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: 16px;
  }

  .masthead-actions a,
  .theme-toggle {
    color: var(--ink-2);
    font-size: 13px;
    font-weight: 500;
  }

  .masthead-actions a:hover,
  .theme-toggle:hover {
    color: var(--foreground);
  }

  .theme-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    margin-left: -4px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: transparent;
  }

  .theme-toggle:hover {
    background: var(--hover);
  }

  .theme-toggle svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .shell {
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    align-items: start;
    width: 100%;
    min-width: 0;
  }

  .sites {
    position: sticky;
    top: 0;
    z-index: 20;
    width: 100%;
    min-width: 0;
    max-height: 100vh;
    padding: 24px 24px 48px 48px;
    overflow: auto;
    background: var(--background);
  }

  .nav-label {
    margin: 0 0 16px;
    color: var(--ink-3);
    font-size: 13px;
    font-weight: 500;
  }

  .sites ol {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .sites li + li {
    margin-top: 8px;
  }

  .sites a {
    display: block;
    color: var(--ink-2);
    font-size: 16px;
    font-weight: 500;
  }

  .sites a:hover,
  .sites a[aria-current="page"] {
    color: var(--foreground);
  }

  .stage {
    width: 100%;
    min-width: 0;
    padding: 24px 48px 56px 0;
  }

  .stage-inner,
  .site-footer {
    width: min(100%, 1120px);
  }

  .site-footer {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 32px 64px;
    margin-top: 112px;
    padding-top: 32px;
    border-top: 1px solid var(--border);
  }

  .footer-heading {
    margin-bottom: 4px;
    color: var(--foreground);
    font-size: 14px;
    font-weight: 500;
  }

  .site-footer p,
  .footer-links a {
    margin: 0;
    color: var(--ink-2);
    font-size: 13px;
  }

  .footer-links {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 8px 16px;
  }

  .footer-links a:hover {
    color: var(--foreground);
  }

  .copyright {
    grid-column: 1 / -1;
    padding-top: 32px;
    color: var(--ink-3) !important;
  }

  @media (max-width: 960px) {
    .masthead {
      flex-direction: column;
      gap: 20px;
      padding: 24px 24px 16px;
    }

    .shell {
      grid-template-columns: 1fr;
    }

    .sites {
      top: 0;
      max-height: none;
      padding: 8px 0 12px;
      overflow: visible;
      border-bottom: 1px solid var(--border);
      background: var(--header-bg);
      backdrop-filter: blur(16px);
      -webkit-backdrop-filter: blur(16px);
    }

    .nav-label {
      display: none;
    }

    .sites ol {
      width: 100%;
      min-width: 0;
      max-width: 100%;
      display: flex;
      gap: 4px;
      padding: 0 12px;
      overflow-x: auto;
      overscroll-behavior-x: contain;
      scrollbar-width: none;
    }

    .sites ol::-webkit-scrollbar {
      display: none;
    }

    .sites li,
    .sites li + li {
      flex: 0 0 auto;
      margin: 0;
    }

    .sites a {
      min-height: 40px;
      padding: 8px 12px;
      white-space: nowrap;
    }

    .stage {
      padding: 32px 24px 48px;
    }

    .site-footer {
      margin-top: 80px;
    }
  }

  @media (max-width: 560px) {
    .masthead-actions {
      width: 100%;
    }

    .theme-toggle {
      margin-left: auto;
    }

    .site-footer {
      grid-template-columns: 1fr;
      gap: 24px;
    }

    .footer-links {
      justify-content: flex-start;
    }

    .copyright {
      grid-column: 1;
    }
  }
</style>
