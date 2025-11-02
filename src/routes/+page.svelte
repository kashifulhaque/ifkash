<svelte:head>
  <title>Kashif</title>
  <meta name="description" content="Engineer III at American Express. I design robust systems and ship thoughtful tools.">
</svelte:head>

<script>
  let mobileOpen = false;

  const links = {
    github: 'https://github.com/kashifulhaque',
    linkedin: 'https://www.linkedin.com/in/kashifulhaque',
    emailPrimary: 'haque.kashiful7@gmail.com',
    emailAlt: 'me@ifkash.dev',
    resume: '/assets/Kashiful_Haque.pdf'
  };

  const quick = [
    { label: 'GitHub', href: links.github },
    { label: 'LinkedIn', href: links.linkedin },
    { label: `${links.emailPrimary}`, href: `mailto:${links.emailPrimary}` },
    { label: `${links.emailAlt}`, href: `mailto:${links.emailAlt}` }
  ];

  function toggleMobile() {
    mobileOpen = !mobileOpen;
  }

  function closeMobile() {
    mobileOpen = false;
  }
</script>

<!-- Page container -->
<div class="min-h-screen selection:text-white" style="background-color: var(--color-background); color: var(--color-paragraph); selection: var(--color-secondary)">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header / Nav -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur" style="backdrop-filter: blur(12px); background-color: rgba(22, 22, 26, 0.6);">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight" style="color: var(--color-headline)">ifkash.dev</a>

          <!-- Desktop nav -->
          <ul class="hidden sm:flex gap-3 text-sm">
            <li><a class="rounded-full px-3 py-1 focus:outline-none transition-colors" style="color: var(--color-paragraph);" onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.backgroundColor='transparent'" href="/work">Work</a></li>
            <li><a class="rounded-full px-3 py-1 focus:outline-none transition-colors" style="color: var(--color-paragraph);" onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.backgroundColor='transparent'" href="/education">Education</a></li>
            <li><a class="rounded-full px-3 py-1 focus:outline-none transition-colors" style="color: var(--color-paragraph);" onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.backgroundColor='transparent'" href="/projects">Projects</a></li>
          </ul>

          <!-- Resume CTA (desktop) -->
          <a
            href={links.resume}
            target="_blank"
            rel="noopener noreferrer"
            class="hidden sm:inline-flex ml-1 items-center gap-2 rounded-full border px-3 py-1.5 text-sm focus:outline-none transition-colors"
            style="background-color: var(--color-button); color: var(--color-button-text); border-color: var(--color-button);"
            onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.8)'"
            onmouseout="this.style.backgroundColor='var(--color-button)'"
            aria-label="Resume"
          >
            <span>Resume</span>
            <span aria-hidden="true">↗</span>
          </a>

          <!-- Mobile menu button -->
          <button
            class="sm:hidden p-2 rounded-lg focus:outline-none transition-colors"
            style="color: var(--color-paragraph);"
            onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'"
            onmouseout="this.style.backgroundColor='transparent'"
            on:click={toggleMobile}
            aria-label="Toggle menu"
            aria-expanded={mobileOpen}
          >
            {#if mobileOpen}
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            {:else}
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            {/if}
          </button>
        </nav>

        <!-- Mobile menu -->
        {#if mobileOpen}
          <div class="sm:hidden pb-4 animate-fade-in">
            <ul class="flex flex-col gap-2">
              <li>
                <a
                  class="block rounded-lg px-4 py-2 focus:outline-none transition-colors"
                  style="color: var(--color-paragraph);"
                  onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'"
                  onmouseout="this.style.backgroundColor='transparent'"
                  href="/work"
                  on:click={closeMobile}
                >
                  Work
                </a>
              </li>
              <li>
                <a
                  class="block rounded-lg px-4 py-2 focus:outline-none transition-colors"
                  style="color: var(--color-paragraph);"
                  onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'"
                  onmouseout="this.style.backgroundColor='transparent'"
                  href="/education"
                  on:click={closeMobile}
                >
                  Education
                </a>
              </li>
              <li>
                <a
                  class="block rounded-lg px-4 py-2 focus:outline-none transition-colors"
                  style="color: var(--color-paragraph);"
                  onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'"
                  onmouseout="this.style.backgroundColor='transparent'"
                  href="/projects"
                  on:click={closeMobile}
                >
                  Projects
                </a>
              </li>
              <li class="pt-2 border-t" style="border-color: var(--color-secondary);">
                <a
                  href={links.resume}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="flex items-center justify-between rounded-lg border px-4 py-2 focus:outline-none transition-colors"
                  style="background-color: var(--color-button); color: var(--color-button-text); border-color: var(--color-button);"
                  onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.8)'"
                  onmouseout="this.style.backgroundColor='var(--color-button)'"
                  on:click={closeMobile}
                >
                  <span>Resume</span>
                  <span aria-hidden="true">↗</span>
                </a>
              </li>
            </ul>
          </div>
        {/if}
      </div>
      <div class="h-px w-full" style="background: linear-gradient(to right, transparent, var(--color-secondary), transparent);"></div>
    </header>

    <!-- Hero -->
    <section class="pt-14 sm:pt-20" id="about">
      <h1 class="text-4xl sm:text-5xl font-semibold leading-tight tracking-tight" style="color: var(--color-headline);">
        <span style="color: var(--color-paragraph);">I am</span> Kashif
      </h1>
      <p class="mt-4 max-w-2xl" style="color: var(--color-paragraph);">
        I design reliable systems and ship useful tools. Currently Engineer III at American Express.
        I enjoy ML systems, infra, and the craft of clean, measurable engineering.
      </p>

      <!-- Quick actions -->
      <div class="mt-6 flex flex-wrap gap-2">
        {#each quick as q}
          <a
            class="group inline-flex items-center gap-2 rounded-full border px-4 py-2 text-sm focus:outline-none transition-colors"
            style="border-color: var(--color-secondary); background-color: rgba(114, 117, 126, 0.1); color: var(--color-paragraph);"
            onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'; this.style.borderColor='var(--color-highlight)'"
            onmouseout="this.style.backgroundColor='rgba(114, 117, 126, 0.1)'; this.style.borderColor='var(--color-secondary)'"
            href={q.href}
            rel="noopener noreferrer"
            target="_blank"
          >
            <span class="i-tab" aria-hidden="true">↗</span>
            <span>{q.label}</span>
          </a>
        {/each}
      </div>
    </section>

    <!-- Writing -->
    <section id="writing" class="mt-14">
      <div class="flex items-end justify-between">
        <!-- Left -->
        <h2 class="text-xl font-semibold" style="color: var(--color-headline);">Writing</h2>

        <!-- Right -->
        <div class="flex gap-4">
          <a
            href="/blog"
            class="text-sm transition-colors"
            style="color: var(--color-paragraph);"
            onmouseover="this.style.color='var(--color-highlight)'"
            onmouseout="this.style.color='var(--color-paragraph)'"
            rel="noopener noreferrer"
          >
            Blog →
          </a>
          <a
            href="/notes"
            class="text-sm transition-colors"
            style="color: var(--color-paragraph);"
            onmouseover="this.style.color='var(--color-highlight)'"
            onmouseout="this.style.color='var(--color-paragraph)'"
            rel="noopener noreferrer"
            target="_blank"
          >
            Notes ↗
          </a>
        </div>
      </div>
    </section>

    <!-- Contact / Footer -->
    <footer id="contact" class="mt-16">
      <div class="h-px w-full" style="background: linear-gradient(to right, transparent, var(--color-secondary), transparent);"></div>
      <div class="mt-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 text-sm" style="color: var(--color-secondary);">
        <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3">
          <a class="transition-colors" onmouseover="this.style.color='var(--color-paragraph)'" onmouseout="this.style.color='var(--color-secondary)'" href="/news">tech news</a>
          <span class="hidden sm:inline" aria-hidden="true">•</span>
          <a class="transition-colors" onmouseover="this.style.color='var(--color-paragraph)'" onmouseout="this.style.color='var(--color-secondary)'" href="/leetcode" rel="noopener noreferrer">leetcode stats</a>
          <span class="hidden sm:inline" aria-hidden="true">•</span>
          <a class="transition-colors" onmouseover="this.style.color='var(--color-paragraph)'" onmouseout="this.style.color='var(--color-secondary)'" href="/dashboard" rel="noopener noreferrer">my dashboard</a>
        </div>
        <div style="color: var(--color-secondary);">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>

<style>
  /* Tiny affordance for the arrow chip */
  .i-tab { transition: transform 150ms ease; }
  a.group:hover .i-tab { transform: translateY(-1px); }

  /* Mobile menu animation */
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .animate-fade-in {
    animation: fade-in 200ms ease-out;
  }
</style>
