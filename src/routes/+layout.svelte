<script>
  import "../app.css";
  import { page } from "$app/stores";
  import { auth } from "$lib/stores/auth";
  import { theme } from "$lib/stores/theme";
  import { onMount } from "svelte";
  import AiAgent from "$lib/components/AiAgent.svelte";
  import SignalField from "$lib/components/SignalField.svelte";

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
    { href: "/", label: "Index", short: "00" },
    { href: "/work", label: "Work", short: "01" },
    { href: "/projects", label: "Projects", short: "02" },
    { href: "/education", label: "Education", short: "03" },
    { href: "/blog", label: "Writing", short: "04" },
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
    <SignalField intensity="medium" fixed={true} />

    <header class="top-bar">
      <div class="bar-inner">
        <a href="/" class="logo" aria-label="Home">
          <span class="logo-mark" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="22" height="22" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.2" opacity="0.35"/>
              <circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="1.4"/>
              <circle cx="12" cy="12" r="1.4" fill="currentColor"/>
              <line x1="12" y1="2" x2="12" y2="6" stroke="currentColor" stroke-width="1.2" opacity="0.55"/>
              <line x1="12" y1="18" x2="12" y2="22" stroke="currentColor" stroke-width="1.2" opacity="0.55"/>
              <line x1="2" y1="12" x2="6" y2="12" stroke="currentColor" stroke-width="1.2" opacity="0.55"/>
              <line x1="18" y1="12" x2="22" y2="12" stroke="currentColor" stroke-width="1.2" opacity="0.55"/>
            </svg>
          </span>
          <span class="logo-text">
            <span class="logo-word">kashif</span>
            <span class="logo-tag">/ml engineer</span>
          </span>
        </a>

        <nav class="nav-desktop" aria-label="Primary">
          {#each navItems as item}
            <a
              href={item.href}
              class="nav-link"
              class:active={item.href === "/"
                ? currentPath === "/"
                : currentPath.startsWith(item.href)}
            >
              <span class="nav-num">{item.short}</span>
              <span class="nav-label">{item.label}</span>
            </a>
          {/each}
        </nav>

        <div class="nav-actions">
          <div class="extras" aria-label="Tools">
            {#each extraItems as item}
              <a href={item.href} class="extra-link">{item.label}</a>
            {/each}
          </div>

          <button
            class="theme-toggle"
            on:click={theme.toggle}
            aria-label="Toggle theme"
            title={$theme === 'light' ? 'Switch to dark mode' : 'Switch to light mode'}
          >
            {#if $theme === 'light'}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
              </svg>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
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
            <a href="/login" class="login-link" aria-label="Login">
              <span>access</span>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M5 12h14M13 5l7 7-7 7"/></svg>
            </a>
          {/if}
        </div>

        <div class="mobile-right">
          <button
            class="theme-toggle"
            on:click={theme.toggle}
            aria-label="Toggle theme"
          >
            {#if $theme === 'light'}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
              </svg>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
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
      <nav class="mobile-menu" aria-label="Mobile">
        {#each navItems as item}
          <a
            href={item.href}
            class="mobile-link"
            class:active={item.href === "/"
              ? currentPath === "/"
              : currentPath.startsWith(item.href)}
            on:click={closeMenu}
          >
            <span class="mobile-num">{item.short}</span>
            <span>{item.label}</span>
          </a>
        {/each}
        <div class="mobile-divider"></div>
        <div class="mobile-extras">
          {#each extraItems as item}
            <a href={item.href} class="mobile-extra" on:click={closeMenu}>
              {item.label}
            </a>
          {/each}
        </div>
        {#if !$auth.isAuthenticated}
          <a href="/login" class="mobile-link" on:click={closeMenu}>
            <span class="mobile-num">→</span>
            <span>Access</span>
          </a>
        {/if}
      </nav>
    {/if}

    <main class="main-content">
      <slot />
    </main>

    <footer class="site-footer">
      <div class="footer-inner">
        <div class="footer-col footer-brand">
          <div class="footer-logo">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.2" opacity="0.35"/>
              <circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="1.4"/>
              <circle cx="12" cy="12" r="1.4" fill="currentColor"/>
            </svg>
            <span>Kashiful Haque</span>
          </div>
          <p class="footer-tag">
            ML systems, agent infrastructure, low-level inference.
            Built quietly. Shipped at scale.
          </p>
          <div class="footer-status">
            <span class="status-dot"></span>
            <span>online · {new Date().getFullYear()}</span>
          </div>
        </div>

        <div class="footer-col">
          <span class="footer-eyebrow">Site</span>
          <a href="/" class="footer-link">Index</a>
          <a href="/work" class="footer-link">Work</a>
          <a href="/projects" class="footer-link">Projects</a>
          <a href="/education" class="footer-link">Education</a>
          <a href="/blog" class="footer-link">Writing</a>
        </div>

        <div class="footer-col">
          <span class="footer-eyebrow">Tools</span>
          <a href="/leetcode" class="footer-link">LeetCode</a>
          <a href="/tensara" class="footer-link">Tensara</a>
          <a href="/news" class="footer-link">Hacker News</a>
          <a href="/matmul" class="footer-link">Matmul</a>
          <a href="/zen" class="footer-link">Zen</a>
        </div>

        <div class="footer-col">
          <span class="footer-eyebrow">Channels</span>
          <a href="https://github.com/kashifulhaque" class="footer-link" target="_blank" rel="noopener noreferrer">GitHub
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M7 17L17 7M9 7h8v8"/></svg>
          </a>
          <a href="https://hf.co/ifkash" class="footer-link" target="_blank" rel="noopener noreferrer">Hugging Face
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M7 17L17 7M9 7h8v8"/></svg>
          </a>
          <a href="https://linkedin.com/in/kashifulhaque" class="footer-link" target="_blank" rel="noopener noreferrer">LinkedIn
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M7 17L17 7M9 7h8v8"/></svg>
          </a>
        </div>
      </div>

      <div class="footer-base">
        <span>&copy; {new Date().getFullYear()} Kashiful Haque</span>
        <span class="footer-base-mid">create momentum. never stagnation.</span>
        <span class="footer-base-end">
          <span class="dot"></span> all systems nominal
        </span>
      </div>
    </footer>

    <AiAgent />
  </div>
{/if}

<style>
  .site-wrapper {
    position: relative;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    isolation: isolate;
  }

  /* ───────── TOP BAR ──────────────────────────────────────── */

  .top-bar {
    position: sticky;
    top: 0;
    z-index: 100;
    background: var(--nav-bg);
    backdrop-filter: blur(20px) saturate(140%);
    -webkit-backdrop-filter: blur(20px) saturate(140%);
    border-bottom: 1px solid var(--border);
    transition: background var(--dur-base) var(--ease-out-quart),
                border-color var(--dur-base) var(--ease-out-quart);
  }

  .bar-inner {
    max-width: var(--canvas-wide);
    margin: 0 auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.85rem 1.5rem;
    gap: 1.5rem;
  }

  @media (min-width: 1024px) {
    .bar-inner {
      padding: 0.85rem 2rem;
    }
  }

  .logo {
    display: inline-flex;
    align-items: center;
    gap: 0.6rem;
    color: var(--text-primary);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .logo-mark {
    display: inline-flex;
    color: var(--accent);
    transition: transform var(--dur-base) var(--ease-out-expo);
  }

  .logo:hover .logo-mark {
    transform: rotate(45deg);
  }

  .logo-text {
    display: flex;
    flex-direction: column;
    line-height: 1.05;
  }

  .logo-word {
    font-size: 0.95rem;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .logo-tag {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--text-faint);
  }

  /* Desktop nav */

  .nav-desktop {
    display: none;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem;
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: 999px;
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
  }

  @media (min-width: 900px) {
    .nav-desktop {
      display: flex;
    }
  }

  .nav-link {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.45rem 0.85rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-tertiary);
    border-radius: 999px;
    transition: background var(--dur-instant) var(--ease-out-quart),
                color var(--dur-instant) var(--ease-out-quart);
    position: relative;
  }

  .nav-link:hover {
    color: var(--text-primary);
    background: var(--surface-sunken);
  }

  .nav-link.active {
    color: var(--text-primary);
    background: var(--surface-sunken);
  }

  .nav-link.active::after {
    content: "";
    position: absolute;
    bottom: 4px;
    left: 50%;
    transform: translateX(-50%);
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 6px var(--accent);
  }

  .nav-num {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    color: var(--text-faint);
    letter-spacing: 0.06em;
  }

  .nav-link.active .nav-num,
  .nav-link:hover .nav-num {
    color: var(--accent);
  }

  /* Right actions */

  .nav-actions {
    display: none;
    align-items: center;
    gap: 0.75rem;
  }

  @media (min-width: 900px) {
    .nav-actions {
      display: flex;
    }
  }

  .extras {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding-right: 0.25rem;
    border-right: 1px solid var(--border);
    margin-right: 0.25rem;
  }

  .extra-link {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--text-faint);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .extra-link:hover {
    color: var(--accent);
  }

  .theme-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: color var(--dur-instant) var(--ease-out-quart),
                border-color var(--dur-instant) var(--ease-out-quart),
                background var(--dur-instant) var(--ease-out-quart);
  }

  .theme-toggle:hover {
    color: var(--accent);
    border-color: var(--accent);
  }

  .login-link {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.4rem 0.85rem;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: 999px;
    transition: all var(--dur-instant) var(--ease-out-quart);
  }

  .login-link:hover {
    color: var(--accent);
    border-color: var(--accent);
  }

  .login-link svg {
    transition: transform var(--dur-fast) var(--ease-out-quart);
  }

  .login-link:hover svg {
    transform: translateX(2px);
  }

  /* Mobile right group */

  .mobile-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  @media (min-width: 900px) {
    .mobile-right {
      display: none;
    }
  }

  .mobile-menu-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .mobile-menu-btn:hover {
    color: var(--text-primary);
  }

  .mobile-overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay-bg);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    z-index: 150;
    animation: fade-in var(--dur-fast) var(--ease-out-expo);
  }

  @media (min-width: 900px) {
    .mobile-overlay {
      display: none;
    }
  }

  .mobile-menu {
    position: fixed;
    top: 64px;
    left: 1rem;
    right: 1rem;
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    z-index: 200;
    padding: 0.75rem;
    animation: enter var(--dur-fast) var(--ease-out-expo);
    box-shadow: var(--shadow-lg);
  }

  @media (min-width: 900px) {
    .mobile-menu {
      display: none;
    }
  }

  .mobile-link {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    padding: 0.85rem 1rem;
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-secondary);
    border-radius: var(--radius-md);
    transition: color var(--dur-instant) var(--ease-out-quart),
                background var(--dur-instant) var(--ease-out-quart);
  }

  .mobile-link:hover,
  .mobile-link.active {
    color: var(--text-primary);
    background: var(--surface-sunken);
  }

  .mobile-link.active {
    color: var(--accent);
  }

  .mobile-num {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    color: var(--text-faint);
    letter-spacing: 0.08em;
    width: 1.5rem;
  }

  .mobile-link.active .mobile-num {
    color: var(--accent);
  }

  .mobile-divider {
    height: 1px;
    background: var(--border);
    margin: 0.5rem 1rem;
  }

  .mobile-extras {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
  }

  .mobile-extra {
    padding: 0.35rem 0.7rem;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-faint);
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: 999px;
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .mobile-extra:hover {
    color: var(--accent);
  }

  /* ───────── MAIN ─────────────────────────────────────────── */

  .main-content {
    flex: 1;
    width: 100%;
    max-width: var(--canvas-wide);
    margin: 0 auto;
    padding: var(--space-xl) 1.5rem var(--space-3xl);
    position: relative;
    z-index: 1;
    animation: enter var(--dur-slow) var(--ease-out-expo);
  }

  @media (min-width: 1024px) {
    .main-content {
      padding: var(--space-2xl) 2rem var(--space-4xl);
    }
  }

  /* ───────── FOOTER ───────────────────────────────────────── */

  .site-footer {
    position: relative;
    z-index: 1;
    border-top: 1px solid var(--border);
    background: var(--surface-glass);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    transition: border-color var(--dur-base) var(--ease-out-quart);
  }

  .footer-inner {
    max-width: var(--canvas-wide);
    margin: 0 auto;
    padding: var(--space-2xl) 1.5rem var(--space-xl);
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-xl);
  }

  @media (min-width: 720px) {
    .footer-inner {
      grid-template-columns: 1.4fr 1fr 1fr 1fr;
      gap: var(--space-lg);
    }
  }

  @media (min-width: 1024px) {
    .footer-inner {
      padding: var(--space-3xl) 2rem var(--space-xl);
    }
  }

  .footer-col {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .footer-brand {
    gap: 1rem;
    max-width: 360px;
  }

  .footer-logo {
    display: inline-flex;
    align-items: center;
    gap: 0.6rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .footer-logo svg {
    color: var(--accent);
  }

  .footer-tag {
    font-size: 0.875rem;
    line-height: 1.55;
    color: var(--text-tertiary);
    max-width: 32ch;
  }

  .footer-status {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--text-faint);
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 6px var(--accent);
    animation: pulse-soft 1.6s ease-in-out infinite;
  }

  .footer-eyebrow {
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--text-faint);
    margin-bottom: 0.35rem;
  }

  .footer-link {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.875rem;
    color: var(--text-secondary);
    transition: color var(--dur-instant) var(--ease-out-quart);
    width: max-content;
  }

  .footer-link:hover {
    color: var(--accent);
  }

  .footer-link svg {
    color: var(--text-faint);
    transition: transform var(--dur-fast) var(--ease-out-quart),
                color var(--dur-instant) var(--ease-out-quart);
  }

  .footer-link:hover svg {
    color: var(--accent);
    transform: translate(2px, -2px);
  }

  .footer-base {
    max-width: var(--canvas-wide);
    margin: 0 auto;
    padding: 1rem 1.5rem 1.5rem;
    border-top: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    align-items: flex-start;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--text-faint);
  }

  @media (min-width: 720px) {
    .footer-base {
      flex-direction: row;
      justify-content: space-between;
      align-items: center;
      padding: 1rem 2rem 1.5rem;
    }
  }

  .footer-base-mid {
    color: var(--text-tertiary);
    font-style: italic;
    text-transform: none;
    letter-spacing: 0.02em;
    font-family: var(--font-serif);
    font-size: 0.875rem;
  }

  .footer-base-end {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .footer-base-end .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 6px var(--accent);
  }
</style>
