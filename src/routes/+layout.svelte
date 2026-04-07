<script>
  import "../app.css";
  import { page } from "$app/stores";
  import { auth } from "$lib/stores/auth";
  import { theme } from "$lib/stores/theme";
  import { onMount } from "svelte";
  import AiAgent from "$lib/components/AiAgent.svelte";

  onMount(() => {
    theme.init();
  });

  $: isDashboard = $page.url.pathname === "/dashboard";
  $: isFullWidth =
    $page.url.pathname === "/login" ||
    $page.url.pathname === "/editor" ||
    $page.url.pathname === "/zen";
  $: currentPath = $page.url.pathname;

  const navItems = [
    { href: "/", label: "Home" },
    { href: "/work", label: "Work" },
    { href: "/projects", label: "Projects" },
    { href: "/education", label: "Education" },
    { href: "/blog", label: "Blog" },
  ];

  const extraItems = [
    { href: "/leetcode", label: "LC" },
    { href: "/tensara", label: "TS" },
    { href: "/news", label: "HN" },
  ];

  let mobileMenuOpen = false;

  function toggleMenu() {
    mobileMenuOpen = !mobileMenuOpen;
  }

  function closeMenu() {
    mobileMenuOpen = false;
  }
</script>

{#if isDashboard || isFullWidth}
  <slot />
{:else}
  <div class="site-wrapper">
    <header class="top-bar">
      <a href="/" class="logo">KH</a>

      <nav class="nav-desktop">
        {#each navItems as item}
          <a
            href={item.href}
            class="nav-link"
            class:active={item.href === "/"
              ? currentPath === "/"
              : currentPath.startsWith(item.href)}
          >
            {item.label}
          </a>
        {/each}
      </nav>

      <div class="nav-actions">
        {#each extraItems as item}
          <a href={item.href} class="nav-link-sm">{item.label}</a>
        {/each}

        <button
          class="theme-toggle"
          on:click={theme.toggle}
          aria-label="Toggle theme"
          title={$theme === 'light' ? 'Switch to dark mode' : 'Switch to light mode'}
        >
          {#if $theme === 'light'}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
            </svg>
          {:else}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="5"/>
              <line x1="12" y1="1" x2="12" y2="3"/>
              <line x1="12" y1="21" x2="12" y2="23"/>
              <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
              <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
              <line x1="1" y1="12" x2="3" y2="12"/>
              <line x1="21" y1="12" x2="23" y2="12"/>
              <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
              <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
            </svg>
          {/if}
        </button>

        {#if !$auth.isAuthenticated}
          <a href="/login" class="nav-link-sm faint">&rarr;</a>
        {/if}
      </div>

      <div class="mobile-right">
        <button
          class="theme-toggle"
          on:click={theme.toggle}
          aria-label="Toggle theme"
        >
          {#if $theme === 'light'}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
            </svg>
          {:else}
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="5"/>
              <line x1="12" y1="1" x2="12" y2="3"/>
              <line x1="12" y1="21" x2="12" y2="23"/>
              <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
              <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
              <line x1="1" y1="12" x2="3" y2="12"/>
              <line x1="21" y1="12" x2="23" y2="12"/>
              <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
              <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
            </svg>
          {/if}
        </button>

        <button
          class="mobile-menu-btn"
          on:click={toggleMenu}
          aria-label="Toggle menu"
        >
          <svg
            width="22"
            height="22"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            {#if mobileMenuOpen}
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            {:else}
              <line x1="4" y1="8" x2="20" y2="8" />
              <line x1="4" y1="16" x2="20" y2="16" />
            {/if}
          </svg>
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
        {#each extraItems as item}
          <a href={item.href} class="mobile-link" on:click={closeMenu}>
            {item.label}
          </a>
        {/each}
        {#if !$auth.isAuthenticated}
          <a href="/login" class="mobile-link" on:click={closeMenu}>Login</a>
        {/if}
      </nav>
    {/if}

    <main class="main-content">
      <slot />
    </main>

    <footer class="site-footer">
      <div class="footer-inner">
        <span class="footer-copy">&copy; {new Date().getFullYear()}</span>
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
  </div>
{/if}

<style>
  .site-wrapper {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .top-bar {
    position: sticky;
    top: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 2rem;
    background: var(--nav-bg);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border-bottom: 1px solid var(--border);
    transition: background var(--dur-base) var(--ease-out-quart),
                border-color var(--dur-base) var(--ease-out-quart);
  }

  .logo {
    font-family: var(--font-serif);
    font-size: 1.125rem;
    font-weight: 500;
    color: var(--text-primary);
    letter-spacing: -0.03em;
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .logo:hover {
    color: var(--accent);
  }

  .nav-desktop {
    display: none;
    align-items: center;
    gap: 2.5rem;
  }

  @media (min-width: 768px) {
    .nav-desktop {
      display: flex;
    }
  }

  .nav-link {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-tertiary);
    letter-spacing: 0.01em;
    transition: color var(--dur-instant) var(--ease-out-quart);
    position: relative;
  }

  .nav-link:hover {
    color: var(--text-primary);
  }

  .nav-link.active {
    color: var(--text-primary);
  }

  .nav-link.active::after {
    content: "";
    position: absolute;
    bottom: -6px;
    left: 0;
    right: 0;
    height: 1.5px;
    background: var(--accent);
    border-radius: 1px;
  }

  .nav-actions {
    display: none;
    align-items: center;
    gap: 1.25rem;
  }

  @media (min-width: 768px) {
    .nav-actions {
      display: flex;
    }
  }

  .nav-link-sm {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-faint);
    letter-spacing: 0.04em;
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .nav-link-sm:hover {
    color: var(--text-primary);
  }

  .nav-link-sm.faint {
    opacity: 0.6;
  }

  .nav-link-sm.faint:hover {
    opacity: 1;
  }

  /* Theme toggle */
  .theme-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: color var(--dur-instant) var(--ease-out-quart),
                border-color var(--dur-instant) var(--ease-out-quart),
                background var(--dur-instant) var(--ease-out-quart);
  }

  .theme-toggle:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
    background: var(--surface-sunken);
  }

  /* Mobile right group */
  .mobile-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  @media (min-width: 768px) {
    .mobile-right {
      display: none;
    }
  }

  .mobile-menu-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.5rem;
    background: none;
    border: none;
    color: var(--text-tertiary);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .mobile-menu-btn:hover {
    color: var(--text-primary);
  }

  .mobile-overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay-bg);
    z-index: 150;
    animation: fade-in var(--dur-fast) var(--ease-out-expo);
  }

  @media (min-width: 768px) {
    .mobile-overlay {
      display: none;
    }
  }

  .mobile-menu {
    position: fixed;
    top: 57px;
    left: 0;
    right: 0;
    background: var(--paper);
    border-bottom: 1px solid var(--border);
    z-index: 200;
    padding: 0.75rem 0;
    animation: enter var(--dur-fast) var(--ease-out-expo);
    max-height: calc(100vh - 57px);
    overflow-y: auto;
  }

  @media (min-width: 768px) {
    .mobile-menu {
      display: none;
    }
  }

  .mobile-link {
    display: block;
    padding: 0.75rem 2rem;
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--text-secondary);
    transition: color var(--dur-instant) var(--ease-out-quart),
                background var(--dur-instant) var(--ease-out-quart);
  }

  .mobile-link:hover,
  .mobile-link.active {
    color: var(--text-primary);
    background: var(--surface-sunken);
  }

  .mobile-divider {
    height: 1px;
    background: var(--border);
    margin: 0.5rem 2rem;
  }

  .main-content {
    flex: 1;
    width: 100%;
    max-width: 680px;
    margin: 0 auto;
    padding: var(--space-2xl) 1.5rem var(--space-3xl);
    animation: enter var(--dur-slow) var(--ease-out-expo);
  }

  @media (min-width: 1024px) {
    .main-content {
      padding: var(--space-3xl) 2rem var(--space-4xl);
    }
  }

  .site-footer {
    border-top: 1px solid var(--border);
    padding: var(--space-xl) 1.5rem;
    transition: border-color var(--dur-base) var(--ease-out-quart);
  }

  .footer-inner {
    max-width: 680px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    align-items: center;
    text-align: center;
  }

  @media (min-width: 640px) {
    .footer-inner {
      flex-direction: row;
      justify-content: space-between;
      text-align: left;
    }
  }

  .footer-copy {
    font-size: 0.8125rem;
    color: var(--text-faint);
  }

  .footer-links {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    font-size: 0.8125rem;
  }

  .footer-links a {
    color: var(--text-tertiary);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .footer-links a:hover {
    color: var(--accent);
  }
</style>
