<script>
  import "../app.css";
  import { page } from "$app/stores";

  $: isDashboard = $page.url.pathname === "/dashboard";
</script>

{#if isDashboard}
  <slot />
{:else}
  <div
    class="min-h-screen bg-[var(--color-background)] text-[var(--color-headline)] selection:bg-white/20 selection:text-[var(--color-background)]"
  >
    <div class="mx-auto max-w-7xl lg:px-8">
      <div class="lg:grid lg:grid-cols-[1fr_2fr] lg:gap-12 xl:gap-24">
        <!-- Sidebar (Fixed on Desktop) -->
        <header
          class=" header_section lg:py-24 py-12 px-6 lg:px-0 lg:sticky lg:top-0 lg:max-h-screen lg:flex lg:flex-col lg:justify-between"
        >
          <div>
            <h1 class="text-4xl font-bold tracking-tighter sm:text-5xl mb-3">
              <a href="/">Kashiful Haque</a>
            </h1>
            <h2
              class="text-lg font-medium tracking-tight text-[var(--color-headline)] mb-6"
            >
              Applied ML Engineer
            </h2>
            <p
              class="max-w-xs text-[var(--color-paragraph)] leading-relaxed mb-8"
            >
              Building agentic coding environments and inference-optimized ML
              systems.
            </p>

            <nav class="hidden lg:block">
              <ul class="flex flex-col gap-3">
                <li>
                  <a
                    href="/work"
                    class="nav-link text-sm uppercase tracking-widest font-bold"
                    class:active={$page.url.pathname.startsWith("/work")}
                    >Work</a
                  >
                </li>
                <li>
                  <a
                    href="/projects"
                    class="nav-link text-sm uppercase tracking-widest font-bold"
                    class:active={$page.url.pathname.startsWith("/projects")}
                    >Projects</a
                  >
                </li>
                <li>
                  <a
                    href="/education"
                    class="nav-link text-sm uppercase tracking-widest font-bold"
                    class:active={$page.url.pathname.startsWith("/education")}
                    >Education</a
                  >
                </li>
                <li>
                  <a
                    href="/blog"
                    class="nav-link text-sm uppercase tracking-widest font-bold"
                    class:active={$page.url.pathname.startsWith("/blog")}
                    >Blog</a
                  >
                </li>
              </ul>
            </nav>
          </div>

          <div
            class="footer_section hidden lg:flex gap-5 text-sm font-medium text-[var(--color-paragraph)]"
          >
            <a
              href="https://github.com/kashifulhaque"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-[var(--color-headline)] transition-colors"
              >GitHub</a
            >
            <a
              href="https://linkedin.com/in/kashifulhaque"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-[var(--color-headline)] transition-colors"
              >LinkedIn</a
            >
            <a
              href="/leetcode"
              class="hover:text-[var(--color-headline)] transition-colors"
              >LeetCode</a
            >
            <a
              href="/news"
              class="hover:text-[var(--color-headline)] transition-colors"
              >News</a
            >
          </div>
        </header>

        <!-- Main Content (Scrollable) -->
        <main class="lg:py-24 px-6 lg:px-0 pb-12">
          <!-- Mobile Nav -->
          <nav
            class="lg:hidden mb-12 flex items-center gap-3 text-sm font-bold uppercase tracking-widest text-[var(--color-headline)]"
          >
            <a
              href="/work"
              class:active={$page.url.pathname.startsWith("/work")}
              class="nav-link">Work</a
            >
            <a
              href="/projects"
              class:active={$page.url.pathname.startsWith("/projects")}
              class="nav-link">Projects</a
            >
            <a
              href="/education"
              class:active={$page.url.pathname.startsWith("/education")}
              class="nav-link">Education</a
            >
            <a
              href="/blog"
              class:active={$page.url.pathname.startsWith("/blog")}
              class="nav-link">Blog</a
            >
          </nav>

          <slot />

          <!-- Mobile Footer -->
          <footer
            class="mt-16 pt-8 border-t border-[var(--color-border)] lg:hidden flex gap-2 text-sm font-medium text-[var(--color-paragraph)]"
          >
            <a
              href="https://github.com/kashifulhaque"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-[var(--color-headline)] transition-colors"
              >GitHub</a
            >
            <a
              href="https://linkedin.com/in/kashifulhaque"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-[var(--color-headline)] transition-colors"
              >LinkedIn</a
            >
            <a
              href="/leetcode"
              class="hover:text-[var(--color-headline)] transition-colors"
              >LeetCode</a
            >
            <a
              href="/news"
              class="hover:text-[var(--color-headline)] transition-colors"
              >News</a
            >
          </footer>
        </main>
      </div>
    </div>
  </div>
{/if}

<style>
  .nav-link {
    display: inline-flex;
    align-items: center;
    color: var(--color-paragraph);
    transition: all 0.2s ease-in-out;
  }
  .nav-link:hover,
  .nav-link.active {
    color: var(--color-headline);
    padding-left: 0.5rem; /* Slight shift indicator */
  }
  .nav-link.active::before {
    content: "";
    display: block;
    width: 4px;
    height: 4px;
    background: currentColor;
    border-radius: 50%;
    margin-right: 8px;
  }
  .header_section{
    height:100vh;
  }
  @media screen and (max-width:500px){
  .header_section{
    height:30vh;
  }
  
}
  .footer_section{
    position: absolute;
    bottom:10%;
  }
</style>
