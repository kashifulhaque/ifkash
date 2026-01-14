<script>
  import "../app.css";
  import { page } from "$app/stores";
  import { auth } from "$lib/stores/auth";

  $: isDashboard = $page.url.pathname === "/dashboard";
  $: isFullWidth = $page.url.pathname === "/login" || $page.url.pathname === "/editor";
  $: currentPath = $page.url.pathname;
  
  const navItems = [
    { href: '/work', label: 'Work' },
    { href: '/projects', label: 'Projects' },
    { href: '/education', label: 'Education' },
    { href: '/blog', label: 'Blog' },
  ];
  
  const extraItems = [
    { href: '/leetcode', label: 'LC' },
    { href: '/tensara', label: 'TS' },
    { href: '/news', label: 'HN' },
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
    <!-- Top Bar -->
    <header class="top-bar">
      <a href="/" class="toolbar-logo">KH</a>
      
      <nav class="nav-desktop">
        {#each navItems as item}
          <a
            href={item.href}
            class="nav-link"
            class:active={currentPath.startsWith(item.href)}
          >
            {item.label}
          </a>
        {/each}
      </nav>
      
      <div class="nav-actions">
        <a href="/leetcode" class="nav-link-ext">LC</a>
        <a href="/tensara" class="nav-link-ext">TS</a>
        <a href="/news" class="nav-link-ext">HN</a>
        {#if !$auth.isAuthenticated}
          <a href="/login" class="nav-link-ext dim">→</a>
        {/if}
      </div>
      
      <!-- Mobile Menu Button -->
      <button class="mobile-menu-btn" on:click={toggleMenu} aria-label="Toggle menu">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          {#if mobileMenuOpen}
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          {:else}
            <line x1="3" y1="12" x2="21" y2="12"></line>
            <line x1="3" y1="6" x2="21" y2="6"></line>
            <line x1="3" y1="18" x2="21" y2="18"></line>
          {/if}
        </svg>
      </button>
    </header>

    <!-- Mobile Menu Dropdown -->
    {#if mobileMenuOpen}
      <div 
        class="mobile-menu-overlay" 
        role="button" 
        tabindex="0"
        on:click={closeMenu}
        on:keydown={(e) => e.key === 'Escape' && closeMenu()}
        aria-label="Close menu"
      ></div>
      <nav class="mobile-menu">
        {#each navItems as item}
          <a
            href={item.href}
            class="mobile-menu-link"
            class:active={currentPath.startsWith(item.href)}
            on:click={closeMenu}
          >
            {item.label}
          </a>
        {/each}
        <div class="mobile-menu-divider"></div>
        {#each extraItems as item}
          <a
            href={item.href}
            class="mobile-menu-link"
            on:click={closeMenu}
          >
            {item.label}
          </a>
        {/each}
        {#if !$auth.isAuthenticated}
          <a href="/login" class="mobile-menu-link" on:click={closeMenu}>Login</a>
        {/if}
      </nav>
    {/if}

    <!-- Main -->
    <main class="main-content">
      <slot />
    </main>

    <!-- Footer -->
    <footer class="site-footer">
      <div class="footer-content">
        <span class="footer-text">© {new Date().getFullYear()}</span>
        <div class="footer-links">
          <a href="https://github.com/kashifulhaque" target="_blank" rel="noopener noreferrer">GitHub</a>
          <span class="footer-divider">/</span>
          <a href="https://hf.co/ifkash" target="_blank" rel="noopener noreferrer">Hugging Face</a>
          <span class="footer-divider">/</span>
          <a href="https://linkedin.com/in/kashifulhaque" target="_blank" rel="noopener noreferrer">LinkedIn</a>
        </div>
      </div>
    </footer>
  </div>
{/if}

<style>
  .site-wrapper {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  /* ═══════════════════════════════════════════════════════════════════════
     TOP BAR
     ═══════════════════════════════════════════════════════════════════════ */
  
  .top-bar {
    position: sticky;
    top: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 2rem;
    background: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--glass-border);
  }
  
  .toolbar-logo {
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
    letter-spacing: -0.01em;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  
  .toolbar-logo:hover {
    opacity: 0.8;
  }
  
  .nav-desktop {
    display: none;
    align-items: center;
    gap: 2rem;
  }
  
  @media (min-width: 768px) {
    .nav-desktop {
      display: flex;
    }
  }
  
  .nav-link {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--gray-500);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    transition: color var(--duration-fast) var(--ease-out);
    position: relative;
  }
  
  .nav-link:hover,
  .nav-link.active {
    color: var(--white);
  }
  
  .nav-link.active::after {
    content: '';
    position: absolute;
    bottom: -4px;
    left: 0;
    right: 0;
    height: 1px;
    background: var(--white);
  }
  
  .nav-actions {
    display: none;
    align-items: center;
    gap: 1.5rem;
  }
  
  @media (min-width: 768px) {
    .nav-actions {
      display: flex;
    }
  }
  
  .nav-link-ext {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--gray-600);
    letter-spacing: 0.05em;
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .nav-link-ext:hover {
    color: var(--white);
  }
  
  .nav-link-ext.dim {
    opacity: 0.75;
  }
  
  .nav-link-ext.dim:hover {
    opacity: 1;
  }
  
  /* Mobile Menu Button */
  .mobile-menu-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.5rem;
    background: none;
    border: none;
    color: var(--gray-500);
    cursor: pointer;
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .mobile-menu-btn:hover {
    color: var(--white);
  }
  
  @media (min-width: 768px) {
    .mobile-menu-btn {
      display: none;
    }
  }
  
  /* Mobile Menu Overlay */
  .mobile-menu-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 150;
    animation: fade-in 0.2s ease-out;
  }
  
  @media (min-width: 768px) {
    .mobile-menu-overlay {
      display: none;
    }
  }
  
  /* Mobile Menu */
  .mobile-menu {
    position: fixed;
    top: 73px; /* Height of top bar */
    left: 0;
    right: 0;
    background: rgba(0, 0, 0, 0.95);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--glass-border);
    z-index: 200;
    padding: 1rem 0;
    animation: slide-down 0.2s ease-out;
    max-height: calc(100vh - 73px);
    overflow-y: auto;
  }
  
  @media (min-width: 768px) {
    .mobile-menu {
      display: none;
    }
  }
  
  .mobile-menu-link {
    display: block;
    padding: 0.875rem 2rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--gray-400);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    transition: all var(--duration-fast) var(--ease-out);
    border-left: 2px solid transparent;
  }
  
  .mobile-menu-link:hover,
  .mobile-menu-link.active {
    color: var(--white);
    background: var(--glass-bg);
    border-left-color: var(--white);
  }
  
  .mobile-menu-divider {
    height: 1px;
    background: var(--glass-border);
    margin: 0.5rem 2rem;
  }
  
  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  
  @keyframes slide-down {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* ═══════════════════════════════════════════════════════════════════════
     MAIN CONTENT
     ═══════════════════════════════════════════════════════════════════════ */
  
  .main-content {
    flex: 1;
    width: 100%;
    max-width: 720px;
    margin: 0 auto;
    padding: 3rem 2rem 6rem;
    animation: fade-up var(--duration-slow) var(--ease-out);
  }
  
  @media (min-width: 1024px) {
    .main-content {
      padding: 4rem 2rem 8rem;
    }
  }

  /* ═══════════════════════════════════════════════════════════════════════
     FOOTER
     ═══════════════════════════════════════════════════════════════════════ */
  
  .site-footer {
    border-top: 1px solid var(--glass-border);
    padding: 2rem;
  }
  
  .footer-content {
    max-width: 720px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    align-items: center;
    text-align: center;
  }
  
  @media (min-width: 640px) {
    .footer-content {
      flex-direction: row;
      justify-content: space-between;
      text-align: left;
    }
  }
  
  .footer-text {
    font-size: 0.75rem;
    color: var(--gray-700);
  }
  
  .footer-links {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.75rem;
  }
  
  .footer-links a {
    color: var(--gray-600);
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .footer-links a:hover {
    color: var(--white);
  }
  
  .footer-divider {
    color: var(--gray-800);
  }
</style>
