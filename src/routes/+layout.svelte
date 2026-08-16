<script lang="ts">
  import "../app.css";
  import { page } from "$app/stores";
  import AiAgent from "$lib/components/AiAgent.svelte";
  import WeatherHeadsUp from "$lib/components/WeatherHeadsUp.svelte";
  import { toggleTheme } from "$lib/stores/theme";

  $: currentPath = $page.url.pathname;
  // Tool sub-pages (e.g. /tools/pdf-annotator) render full-width like the editor;
  // the /tools listing stays in the normal container.
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

  let mobileMenuOpen = false;

  function isActive(href: string) {
    return href === "/" ? currentPath === "/" : currentPath.startsWith(href);
  }

  function toggleMenu() {
    mobileMenuOpen = !mobileMenuOpen;
  }

  function closeMenu() {
    mobileMenuOpen = false;
  }
</script>

{#if isFullWidth}
  <slot />
{:else}
  <div class="site-wrapper">
    <header class="site-header">
      <div class="header-inner frame">
        <a href="/" class="logo" aria-label="ifkash.dev home">
          <span class="logo-word">ifkash</span><span class="logo-tld">.dev</span>
        </a>

        <nav class="nav-desktop" aria-label="Primary">
          {#each navItems as item}
            <a
              href={item.href}
              class="nav-link"
              class:active={isActive(item.href)}
            >
              {item.label}
            </a>
          {/each}
        </nav>

        <div class="header-actions">
          <button
            class="theme-toggle"
            on:click={toggleTheme}
            aria-label="Toggle color theme"
            title="Toggle theme"
          >
            <svg
              class="icon-moon"
              viewBox="0 0 24 24"
              width="14"
              height="14"
              aria-hidden="true"
            >
              <path
                d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            <svg
              class="icon-sun"
              viewBox="0 0 24 24"
              width="14"
              height="14"
              aria-hidden="true"
            >
              <circle
                cx="12"
                cy="12"
                r="4"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              />
              <path
                d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
              />
            </svg>
          </button>

          <a href="/api/cv?format=view" class="btn btn--header header-cta"
            >Resume</a
          >

          <button
            class="nav-toggle"
            on:click={toggleMenu}
            aria-label={mobileMenuOpen ? "Close menu" : "Open menu"}
            aria-expanded={mobileMenuOpen}
          >
            {#if mobileMenuOpen}
              <svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
                <path
                  d="M6 6l12 12M18 6L6 18"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                />
              </svg>
            {:else}
              <svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
                <path
                  d="M4 7h16M4 12h16M4 17h16"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                />
              </svg>
            {/if}
          </button>
        </div>
      </div>
    </header>

    {#if mobileMenuOpen}
      <div class="mobile-menu">
        <ul class="mobile-menu__list">
          {#each navItems as item}
            <li>
              <a
                href={item.href}
                class="mobile-menu__item"
                class:active={isActive(item.href)}
                on:click={closeMenu}
              >
                <span class="mobile-menu__label">{item.label}</span>
              </a>
            </li>
          {/each}
        </ul>
        <a
          href="https://github.com/kashifulhaque"
          target="_blank"
          rel="noopener noreferrer"
          class="mobile-menu__cta"
          on:click={closeMenu}>GitHub</a
        >
        <div class="mobile-menu__foot">
          <a href="/admin" on:click={closeMenu}>Admin</a>
        </div>
      </div>
    {/if}

    <main class="main-content">
      <div class="container">
        <slot />
      </div>
    </main>

    <footer class="site-footer">
      <div class="frame site-footer__inner">
        <div class="site-footer__brand">
          <p class="label col-heading">ifkash.dev</p>
          <p class="site-footer__mission">
            ML systems, open source, and small tools — built in Bangalore.
          </p>
          <p class="label">Est. 2023</p>
        </div>
        <div class="site-footer__links">
          <p class="label col-heading">Elsewhere</p>
          <a
            href="https://github.com/kashifulhaque"
            target="_blank"
            rel="noopener noreferrer">GitHub</a
          >
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
      </div>
      <div class="frame site-footer__bottom">
        <span>© {new Date().getFullYear()} Kashiful Haque</span>
        <span class="label">Bangalore, IN</span>
      </div>
    </footer>

    <AiAgent />
    <WeatherHeadsUp />
  </div>
{/if}

<style>
  .site-wrapper {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  /* ─── Header ───────────────────────────────────────────────── */

  .site-header {
    position: sticky;
    top: 0;
    z-index: 50;
    isolation: isolate;
    height: 69px;
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--border);
  }

  .site-header::before {
    content: "";
    position: absolute;
    inset: 0;
    z-index: -10;
    background: var(--header-bg);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  .header-inner {
    width: 100%;
    height: 69px;
    display: flex;
    align-items: center;
    gap: var(--content-sm);
  }

  .logo {
    display: flex;
    align-items: baseline;
    gap: 0;
    border-bottom: none;
    white-space: nowrap;
    margin-right: auto;
    color: var(--foreground);
  }

  .logo:hover {
    border-bottom: none;
    color: var(--foreground);
  }

  .logo-word {
    font-family: var(--font-serif);
    font-size: 1.35rem;
    font-weight: 400;
    letter-spacing: -0.03em;
    line-height: 1;
    color: var(--foreground);
  }

  .logo-tld {
    font-family: var(--font-label);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted-foreground);
    margin-left: 2px;
  }

  .logo:hover .logo-word {
    color: var(--accent-2);
  }

  .nav-desktop {
    display: none;
    align-items: center;
    gap: 4px;
  }

  .nav-link {
    display: inline-flex;
    align-items: center;
    height: 36px;
    padding-inline: 16px;
    border-radius: var(--radius-sm);
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 400;
    letter-spacing: normal;
    text-transform: none;
    color: var(--foreground);
    border-bottom: none;
    transition:
      color 0.15s var(--ease-inout),
      background-color 0.15s var(--ease-inout);
  }

  .nav-link:hover {
    background: var(--input);
    color: var(--foreground);
    border-bottom: none;
  }

  .nav-link.active {
    color: var(--accent);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-left: auto;
  }

  .header-cta {
    display: none;
  }

  .theme-toggle,
  .nav-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background: transparent;
    border: 0;
    border-radius: var(--radius-md);
    color: var(--foreground);
    cursor: var(--cursor-pointer);
    transition:
      color 0.15s var(--ease-inout),
      background-color 0.15s var(--ease-inout);
  }

  .theme-toggle:hover,
  .nav-toggle:hover {
    background: var(--input);
    color: var(--foreground);
  }

  .theme-toggle .icon-sun {
    display: none;
  }
  .theme-toggle .icon-moon {
    display: inline-flex;
  }
  :global(html[data-theme="light"]) .theme-toggle .icon-moon {
    display: none;
  }
  :global(html[data-theme="light"]) .theme-toggle .icon-sun {
    display: inline-flex;
  }

  /* ─── Mobile menu ──────────────────────────────────────────── */

  .mobile-menu {
    position: fixed;
    inset: 69px 0 0;
    z-index: 40;
    background: var(--background);
    overflow-y: auto;
    overscroll-behavior: contain;
  }

  .mobile-menu__list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .mobile-menu__list > li + li {
    border-top: 1px solid var(--rule);
  }

  .mobile-menu__item {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    min-height: 60px;
    padding: 16px;
    color: var(--foreground);
    border-bottom: none;
  }

  .mobile-menu__item:hover {
    border-bottom: none;
    color: var(--foreground);
  }

  .mobile-menu__item:hover .mobile-menu__label {
    text-decoration: underline;
  }

  .mobile-menu__item.active .mobile-menu__label {
    color: var(--accent);
  }

  .mobile-menu__label {
    font-family: var(--font-serif);
    font-size: 20px;
    line-height: 28px;
    font-weight: 400;
  }

  .mobile-menu__cta {
    display: block;
    margin: 24px 16px;
    padding: 10px 16px;
    border-radius: var(--radius-sm);
    background: var(--foreground);
    color: var(--background);
    font-size: 14px;
    text-align: center;
    border-bottom: none;
  }

  .mobile-menu__cta:hover {
    background: var(--accent-2);
    color: var(--background);
    border-bottom: none;
  }

  .mobile-menu__foot {
    padding: 0 16px 32px;
  }

  .mobile-menu__foot a {
    font-family: var(--font-label);
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted-foreground);
    border-bottom: none;
  }

  .mobile-menu__foot a:hover {
    color: var(--foreground);
    border-bottom: none;
  }

  /* ─── Main ─────────────────────────────────────────────────── */

  .main-content {
    flex: 1;
    width: 100%;
  }

  .main-content :global(.container) {
    padding-top: 40px;
    padding-bottom: var(--section-pb);
    animation: enter var(--dur-slow) var(--ease-out-expo);
  }

  /* ─── Footer ───────────────────────────────────────────────── */

  .site-footer {
    border-top: 1px solid var(--border);
    padding-top: var(--section-pt);
  }

  .site-footer__inner {
    display: grid;
    grid-template-columns: 1fr;
    gap: 64px;
    padding-bottom: var(--section-gap-sm);
  }

  .site-footer__mission {
    max-width: 36ch;
    margin: 0 0 16px;
    font-size: 16px;
    color: var(--muted-foreground);
  }

  .site-footer .col-heading {
    margin-bottom: 16px;
  }

  .site-footer__links {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .site-footer__links a {
    font-size: 16px;
    color: var(--foreground);
    border-bottom: none;
  }

  .site-footer__links a:hover {
    color: var(--accent-2);
    border-bottom: none;
  }

  .site-footer__bottom {
    border-top: 1px solid var(--border);
    padding-block: 20px;
    font-size: 13px;
    color: var(--muted-foreground);
    display: flex;
    flex-direction: column;
    gap: 12px;
    text-align: center;
  }

  /* Desktop nav at xl (1280), not lg */
  @media (min-width: 1280px) {
    .site-header {
      border-bottom: 0;
    }

    .nav-desktop {
      display: flex;
    }

    .nav-toggle {
      display: none;
    }

    .header-cta {
      display: inline-flex;
    }

    .mobile-menu {
      display: none;
    }

    .site-footer__inner {
      grid-template-columns: 2fr 1fr;
      gap: 32px;
    }

    .site-footer__bottom {
      flex-direction: row;
      justify-content: space-between;
      text-align: left;
    }
  }

  @media (min-width: 1024px) {
    .site-footer__inner {
      grid-template-columns: 2fr 1fr;
      gap: 32px;
    }

    .site-footer__bottom {
      flex-direction: row;
      justify-content: space-between;
      text-align: left;
    }
  }
</style>
