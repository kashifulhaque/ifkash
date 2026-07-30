<script>
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
    { href: "/tools", label: "Tools" },
    { href: "/education", label: "Education" },
    { href: "/blog", label: "Blog" },
  ];

  let mobileMenuOpen = false;

  function isActive(href) {
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
      <div class="header-inner">
        <a href="/" class="logo" aria-label="ifkash.dev home">
          <span class="logo-led" aria-hidden="true"></span>
          <span class="logo-word">IFKASH</span><span class="logo-tld">.DEV</span>
        </a>

        <nav class="header-nav" aria-label="Primary">
          {#each navItems as item}
            <a href={item.href} class:active={isActive(item.href)}>
              {#if isActive(item.href)}<span
                  class="nav-led"
                  aria-hidden="true"
                ></span>{/if}
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

          <button
            class="mobile-menu-btn"
            on:click={toggleMenu}
            aria-label="Toggle menu"
            aria-expanded={mobileMenuOpen}
          >
            {#if mobileMenuOpen}
              <span aria-hidden="true">×</span>
            {:else}
              <span aria-hidden="true">≡</span>
            {/if}
          </button>
        </div>
      </div>
    </header>

    {#if mobileMenuOpen}
      <div class="mobile-menu">
        <nav class="mobile-nav" aria-label="Mobile">
          {#each navItems as item, i}
            <a
              href={item.href}
              class="mobile-link"
              class:active={isActive(item.href)}
              on:click={closeMenu}
            >
              <span class="mobile-index">0{i + 1}</span>
              <span class="mobile-word">{item.label}</span>
              {#if isActive(item.href)}<span
                  class="led"
                  aria-hidden="true"
                ></span>{/if}
            </a>
          {/each}
        </nav>
        <div class="mobile-foot">
          <a
            href="https://github.com/kashifulhaque"
            target="_blank"
            rel="noopener noreferrer"
            on:click={closeMenu}>GitHub</a
          >
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
      <div class="container footer-inner">
        <p class="footer-line">
          <span class="led" aria-hidden="true"></span>
          © {new Date().getFullYear()} · Kashiful Haque
        </p>
        <div class="footer-links">
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
          <a href="/admin" aria-label="Admin">Admin</a>
        </div>
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
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 100;
    background: var(--header-bg);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--line-soft);
  }

  .header-inner {
    max-width: var(--maxw);
    margin: 0 auto;
    padding: 0 32px;
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
    border-bottom: none;
    white-space: nowrap;
  }

  .logo:hover {
    border-bottom: none;
  }

  .logo-led {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--signal);
    box-shadow:
      0 0 6px var(--signal-glow),
      0 0 2px var(--signal);
    animation: led-pulse 2.4s ease-in-out infinite;
  }

  .logo-word {
    font-family: var(--font-dots);
    font-size: 1.55rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    line-height: 1;
    color: var(--ink);
    transform: translateY(1px);
  }

  .logo-tld {
    font-family: var(--font-mono-g);
    font-size: 0.68rem;
    letter-spacing: 0.1em;
    color: var(--ink-mute);
    transform: translateY(3px);
  }

  .logo:hover .logo-word {
    color: var(--accent-hover);
  }

  .header-nav {
    display: flex;
    gap: 26px;
    align-items: center;
  }

  .header-nav a {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-family: var(--font-mono-g);
    font-size: 0.72rem;
    font-weight: 500;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--ink-soft);
    border-bottom: none;
    transition: color 0.15s;
  }

  .header-nav a:hover {
    color: var(--ink);
    border-bottom: none;
  }

  .header-nav a.active {
    color: var(--ink);
  }

  .nav-led {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--signal);
    box-shadow: 0 0 4px var(--signal-glow);
  }

  .mobile-menu-btn {
    display: none;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    color: var(--ink);
    font-family: var(--font-mono-g);
    font-size: 1.15rem;
    line-height: 1;
    cursor: pointer;
    transition:
      border-color 0.15s,
      color 0.15s,
      background 0.15s;
  }

  .mobile-menu-btn:hover {
    border-color: var(--ink);
    color: var(--void);
    background: var(--ink);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .theme-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    color: var(--ink-soft);
    cursor: pointer;
    transition:
      color 0.15s,
      border-color 0.15s,
      background 0.15s;
  }

  .theme-toggle:hover {
    color: var(--void);
    border-color: var(--ink);
    background: var(--ink);
  }

  /* Icon swap driven purely by [data-theme] — no hydration flash. */
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

  /* ─── Mobile menu — full-screen console overlay ────────────── */

  .mobile-menu {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    background: var(--void);
    background-image: radial-gradient(
      circle,
      var(--dots) 1px,
      transparent 1.4px
    );
    background-size: var(--cell) var(--cell);
    padding: 96px 32px 40px;
    animation: fade-in var(--dur-fast) var(--ease-out-expo);
  }

  .mobile-nav {
    display: flex;
    flex-direction: column;
  }

  .mobile-link {
    display: flex;
    align-items: baseline;
    gap: 18px;
    padding: 14px 0;
    border-bottom: 1px solid var(--line-soft);
    color: var(--ink);
  }

  .mobile-link:hover {
    border-bottom-color: var(--line-soft);
  }

  .mobile-index {
    font-family: var(--font-mono-g);
    font-size: 0.72rem;
    letter-spacing: 0.12em;
    color: var(--signal);
  }

  .mobile-word {
    font-family: var(--font-dots);
    font-size: clamp(2.4rem, 9vw, 3.6rem);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    line-height: 1;
    color: var(--ink);
  }

  .mobile-link .led {
    align-self: center;
    margin-left: auto;
  }

  .mobile-link.active .mobile-word {
    color: var(--signal-hi);
  }

  .mobile-foot {
    display: flex;
    gap: 24px;
  }

  .mobile-foot a {
    font-family: var(--font-mono-g);
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--ink-soft);
    border-bottom: none;
  }

  .mobile-foot a:hover {
    color: var(--ink);
    border-bottom: none;
  }

  /* ─── Main ─────────────────────────────────────────────────── */

  .main-content {
    flex: 1;
    width: 100%;
    padding-top: 60px;
  }

  .main-content .container {
    padding-top: 40px;
    padding-bottom: 96px;
    animation: enter var(--dur-slow) var(--ease-out-expo);
  }

  /* ─── Footer ───────────────────────────────────────────────── */

  .site-footer {
    border-top: 1px solid var(--line-soft);
    padding: 28px 0;
  }

  .footer-inner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 16px;
  }

  .footer-line {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    letter-spacing: 0.12em;
    color: var(--ink-soft);
    text-transform: uppercase;
  }

  .footer-links {
    display: flex;
    gap: 24px;
  }

  .footer-links a {
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-soft);
    border-bottom: none;
    transition: color 0.15s;
  }

  .footer-links a:hover {
    color: var(--ink);
    border-bottom: none;
  }

  /* ─── Responsive ───────────────────────────────────────────── */

  @media (max-width: 900px) {
    .header-nav {
      display: none;
    }

    .mobile-menu-btn {
      display: inline-flex;
    }
  }

  @media (max-width: 768px) {
    .header-inner {
      height: 56px;
      padding: 0 20px;
    }

    .logo-word {
      font-size: 1.35rem;
    }

    .main-content {
      padding-top: 56px;
    }

    .footer-inner {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }
  }

  @media (max-width: 480px) {
    .header-inner {
      padding: 0 16px;
      gap: 8px;
    }

    .footer-links {
      gap: 16px;
      flex-wrap: wrap;
    }
  }
</style>
