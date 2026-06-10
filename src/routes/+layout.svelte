<script>
  import "../app.css";
  import { page } from "$app/stores";
  import AiAgent from "$lib/components/AiAgent.svelte";
  import WeatherHeadsUp from "$lib/components/WeatherHeadsUp.svelte";

  $: currentPath = $page.url.pathname;
  // Tool sub-pages (e.g. /tools/pdf-annotator) render full-width like the editor;
  // the /tools listing stays in the normal container.
  $: isFullWidth =
    currentPath === "/editor" ||
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
        <a href="/" class="logo">
          <span class="logo-icon" aria-hidden="true"></span> IFKASH
        </a>

        <nav class="header-nav">
          {#each navItems as item}
            <a
              href={item.href}
              class:active={item.href === "/"
                ? currentPath === "/"
                : currentPath.startsWith(item.href)}
            >
              {item.label}
            </a>
          {/each}

          <a href="/admin" class="header-login" aria-label="Admin">&rarr;</a>
        </nav>

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
    </header>

    {#if mobileMenuOpen}
      <div
        class="mobile-overlay"
        role="button"
        tabindex="0"
        on:click={closeMenu}
        on:keydown={(e) => e.key === "Escape" && closeMenu()}
        aria-label="Close menu"
      ></div>
      <nav class="mobile-menu">
        {#each navItems as item}
          <a
            href={item.href}
            class="mobile-link"
            class:active={item.href === "/"
              ? currentPath === "/"
              : currentPath.startsWith(item.href)}
            on:click={closeMenu}
          >
            {item.label}
          </a>
        {/each}
        <div class="mobile-divider"></div>
        <a
          href="https://github.com/kashifulhaque"
          target="_blank"
          rel="noopener noreferrer"
          class="mobile-link"
          on:click={closeMenu}>GitHub</a
        >
        <a href="/admin" class="mobile-link" on:click={closeMenu}>Admin</a>
      </nav>
    {/if}

    <main class="main-content">
      <div class="container">
        <slot />
      </div>
    </main>

    <footer class="site-footer">
      <div class="container footer-inner">
        <p>© {new Date().getFullYear()} · Kashiful Haque</p>
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
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    border-bottom: 1px solid var(--rule-soft);
  }

  .header-inner {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 32px;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .logo {
    font-family: var(--font-display);
    font-size: 1.6rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--ink);
    border-bottom: none;
    display: flex;
    align-items: baseline;
    gap: 8px;
    white-space: nowrap;
  }

  .logo:hover {
    color: var(--blueprint);
    border-bottom: none;
  }

  .logo-icon {
    display: inline-block;
    width: 12px;
    height: 12px;
    background: var(--blueprint);
    transform: translateY(2px);
  }

  .header-nav {
    display: flex;
    gap: 28px;
    align-items: center;
  }

  .header-nav a {
    font-family: var(--font-mono);
    font-size: 0.8rem;
    font-weight: 500;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-soft);
    border-bottom: none;
    transition: color 0.15s;
  }

  .header-nav a:hover {
    color: var(--blueprint);
    border-bottom: none;
  }

  .header-nav a.active {
    color: var(--blueprint);
  }

  .header-login {
    color: var(--ink-mute) !important;
    font-size: 1rem !important;
    opacity: 0.7;
  }

  .header-login:hover {
    opacity: 1;
  }

  .mobile-menu-btn {
    display: none;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--rule-soft);
    color: var(--ink);
    font-family: var(--font-display);
    font-size: 1.4rem;
    line-height: 1;
    cursor: pointer;
    transition:
      border-color 0.15s,
      color 0.15s;
  }

  .mobile-menu-btn:hover {
    border-color: var(--blueprint);
    color: var(--blueprint);
  }

  /* ─── Mobile menu ──────────────────────────────────────────── */

  .mobile-overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay-bg);
    z-index: 150;
    animation: fade-in var(--dur-fast) var(--ease-out-expo);
  }

  .mobile-menu {
    position: fixed;
    top: 64px;
    left: 0;
    right: 0;
    background: var(--bg);
    border-bottom: 1px solid var(--rule-soft);
    z-index: 200;
    padding: 8px 0;
    animation: enter var(--dur-fast) var(--ease-out-expo);
    max-height: calc(100vh - 64px);
    overflow-y: auto;
  }

  .mobile-link {
    display: block;
    padding: 14px 32px;
    font-family: var(--font-mono);
    font-size: 0.85rem;
    font-weight: 500;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-soft);
    border-bottom: none;
    transition:
      color 0.15s,
      background 0.15s;
  }

  .mobile-link:hover,
  .mobile-link.active {
    color: var(--blueprint);
    background: var(--blueprint-tint);
    border-bottom: none;
  }

  .mobile-divider {
    height: 1px;
    background: var(--rule-soft);
    margin: 8px 32px;
  }

  /* ─── Main ─────────────────────────────────────────────────── */

  .main-content {
    flex: 1;
    width: 100%;
    padding-top: 64px;
  }

  .main-content .container {
    padding-top: 32px;
    padding-bottom: 80px;
    animation: enter var(--dur-slow) var(--ease-out-expo);
  }

  /* ─── Footer ───────────────────────────────────────────────── */

  .site-footer {
    border-top: 1px solid var(--rule-soft);
    padding: 32px 0;
  }

  .footer-inner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 16px;
  }

  .footer-inner p {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.06em;
    color: var(--ink-mute);
    text-transform: uppercase;
  }

  .footer-links {
    display: flex;
    gap: 24px;
  }

  .footer-links a {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ink-soft);
    border-bottom: none;
  }

  .footer-links a:hover {
    color: var(--blueprint);
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
      padding: 0 16px;
    }

    .logo {
      font-size: 1.2rem;
      gap: 6px;
    }

    .main-content {
      padding-top: 56px;
    }

    .mobile-menu {
      top: 56px;
      max-height: calc(100vh - 56px);
    }

    .footer-inner {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }
  }

  @media (max-width: 480px) {
    .header-inner {
      padding: 0 12px;
      gap: 8px;
    }

    .logo {
      font-size: 1rem;
      letter-spacing: 0.02em;
    }

    .logo-icon {
      width: 10px;
      height: 10px;
    }

    .footer-links {
      gap: 16px;
      flex-wrap: wrap;
    }
  }
</style>
