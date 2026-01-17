<script>
  import "../app.css";
  import { page } from "$app/stores";
  import { auth } from "$lib/stores/auth";

  $: isDashboard = $page.url.pathname === "/dashboard";
  $: isFullWidth = $page.url.pathname === "/login" || $page.url.pathname === "/editor";
  $: isHomePage = $page.url.pathname === "/";
  $: currentPath = $page.url.pathname;
  
  const navItems = [
    { href: '/work', label: 'Work' },
    { href: '/projects', label: 'Projects' },
    { href: '/education', label: 'Education' },
    { href: '/blog', label: 'Blog' },
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
    <header class="top-bar" class:transparent={isHomePage}>
      <a href="/" class="toolbar-logo">
        <span class="logo-icon">◈</span>
        <span class="logo-text">Kashif</span>
      </a>
      
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
          <span class="nav-divider"></span>
          <a href="https://linkedin.com/in/kashifulhaque" target="_blank" rel="noopener noreferrer" class="btn-talk">
            Get in touch
          </a>
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
        <a href="/leetcode" class="mobile-menu-link" on:click={closeMenu}>LeetCode</a>
        <a href="/tensara" class="mobile-menu-link" on:click={closeMenu}>Tensara</a>
        <a href="/news" class="mobile-menu-link" on:click={closeMenu}>Hacker News</a>
        <div class="mobile-menu-divider"></div>
        <a href="https://linkedin.com/in/kashifulhaque" class="mobile-menu-link cta" target="_blank" rel="noopener noreferrer" on:click={closeMenu}>
          Get in touch
        </a>
      </nav>
    {/if}

    <!-- Main -->
    <main class="main-content" class:full-width={isHomePage}>
      <slot />
    </main>

    <!-- Footer -->
    {#if !isHomePage}
      <footer class="site-footer">
        <div class="footer-content">
          <span class="footer-text">© {new Date().getFullYear()} Kashiful Haque</span>
          <div class="footer-links">
            <a href="https://github.com/kashifulhaque" target="_blank" rel="noopener noreferrer">GitHub</a>
            <span class="footer-divider">·</span>
            <a href="https://hf.co/ifkash" target="_blank" rel="noopener noreferrer">Hugging Face</a>
            <span class="footer-divider">·</span>
            <a href="https://linkedin.com/in/kashifulhaque" target="_blank" rel="noopener noreferrer">LinkedIn</a>
          </div>
        </div>
      </footer>
    {/if}
  </div>
{/if}

<style>
  .site-wrapper {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  /* ═══════════════════════════════════════════════════════════════════════
     TOP BAR - Giga Style
     ═══════════════════════════════════════════════════════════════════════ */
  
  .top-bar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 2rem;
    background: rgba(13, 13, 13, 0.9);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-bottom: 1px solid var(--color-border);
    transition: background var(--duration-base) var(--ease-out);
  }
  
  .top-bar.transparent {
    background: transparent;
    border-bottom-color: transparent;
  }
  
  .toolbar-logo {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  
  .toolbar-logo:hover {
    opacity: 0.8;
  }
  
  .logo-icon {
    font-size: 1.25rem;
    color: var(--white);
  }
  
  .logo-text {
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
    letter-spacing: -0.01em;
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
    font-size: 0.875rem;
    font-weight: 400;
    color: var(--gray-400);
    transition: color var(--duration-fast) var(--ease-out);
    position: relative;
  }
  
  .nav-link:hover,
  .nav-link.active {
    color: var(--white);
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
  
  .nav-link-ext {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--gray-500);
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .nav-link-ext:hover {
    color: var(--white);
  }
  
  .nav-divider {
    width: 1px;
    height: 20px;
    background: var(--color-border);
  }
  
  .btn-talk {
    display: inline-flex;
    align-items: center;
    padding: 0.5rem 1rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--black);
    background: var(--white);
    border-radius: var(--radius-full);
    transition: all var(--duration-fast) var(--ease-out);
  }
  
  .btn-talk:hover {
    background: var(--gray-200);
    transform: translateY(-1px);
  }
  
  /* Mobile Menu Button */
  .mobile-menu-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.5rem;
    background: none;
    border: none;
    color: var(--gray-400);
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
    background: rgba(0, 0, 0, 0.6);
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
    top: 73px;
    left: 0;
    right: 0;
    background: rgba(13, 13, 13, 0.98);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-bottom: 1px solid var(--color-border);
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
    padding: 1rem 2rem;
    font-size: 0.9375rem;
    font-weight: 400;
    color: var(--gray-400);
    transition: all var(--duration-fast) var(--ease-out);
  }
  
  .mobile-menu-link:hover,
  .mobile-menu-link.active {
    color: var(--white);
    background: var(--glass-bg);
  }
  
  .mobile-menu-link.cta {
    color: var(--white);
    font-weight: 500;
  }
  
  .mobile-menu-divider {
    height: 1px;
    background: var(--color-border);
    margin: 0.75rem 2rem;
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
    padding: 7rem 2rem 6rem;
    animation: fade-up var(--duration-slow) var(--ease-out);
  }
  
  .main-content.full-width {
    max-width: none;
    padding: 0;
  }
  
  @media (min-width: 1024px) {
    .main-content:not(.full-width) {
      padding: 8rem 2rem 8rem;
    }
  }

  /* ═══════════════════════════════════════════════════════════════════════
     FOOTER
     ═══════════════════════════════════════════════════════════════════════ */
  
  .site-footer {
    border-top: 1px solid var(--color-border);
    padding: 2.5rem 2rem;
    background: var(--black);
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
    font-size: 0.8125rem;
    color: var(--gray-600);
  }
  
  .footer-links {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.8125rem;
  }
  
  .footer-links a {
    color: var(--gray-500);
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .footer-links a:hover {
    color: var(--white);
  }
  
  .footer-divider {
    color: var(--gray-700);
  }
</style>
