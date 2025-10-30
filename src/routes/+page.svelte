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
<div class="min-h-screen bg-neutral-950 text-neutral-200 selection:bg-neutral-800 selection:text-white">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header / Nav -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur supports-[backdrop-filter]:bg-neutral-950/60">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight text-neutral-100">ifkash.dev</a>

          <!-- Desktop nav -->
          <ul class="hidden sm:flex gap-3 text-sm">
            <li><a class="rounded-full px-3 py-1 hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600" href="/work">Work</a></li>
            <li><a class="rounded-full px-3 py-1 hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600" href="/education">Education</a></li>
            <li><a class="rounded-full px-3 py-1 hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600" href="/projects">Projects</a></li>
          </ul>

          <!-- Resume CTA (desktop) -->
          <a
            href={links.resume}
            target="_blank"
            rel="noopener noreferrer"
            class="hidden sm:inline-flex ml-1 items-center gap-2 rounded-full border border-neutral-700 bg-neutral-900 px-3 py-1.5 text-sm text-neutral-100 hover:border-neutral-600 hover:bg-neutral-800 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600"
            aria-label="Resume"
          >
            <span>Resume</span>
            <span aria-hidden="true">↗</span>
          </a>

          <!-- Mobile menu button -->
          <button
            class="sm:hidden p-2 rounded-lg hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600"
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
                  class="block rounded-lg px-4 py-2 hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600"
                  href="/work"
                  on:click={closeMobile}
                >
                  Work
                </a>
              </li>
              <li>
                <a
                  class="block rounded-lg px-4 py-2 hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600"
                  href="/education"
                  on:click={closeMobile}
                >
                  Education
                </a>
              </li>
              <li>
                <a
                  class="block rounded-lg px-4 py-2 hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600"
                  href="/projects"
                  on:click={closeMobile}
                >
                  Projects
                </a>
              </li>
              <li class="pt-2 border-t border-neutral-800">
                <a
                  href={links.resume}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="flex items-center justify-between rounded-lg border border-neutral-700 bg-neutral-900 px-4 py-2 hover:border-neutral-600 hover:bg-neutral-800 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600"
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
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
    </header>

    <!-- Hero -->
    <section class="pt-14 sm:pt-20" id="about">
      <h1 class="text-4xl sm:text-5xl font-semibold leading-tight tracking-tight text-neutral-100">
        <span class="text-neutral-400">I am</span> Kashif
      </h1>
      <p class="mt-4 text-neutral-400 max-w-2xl">
        I design reliable systems and ship useful tools. Currently Engineer III at American Express.
        I enjoy ML systems, infra, and the craft of clean, measurable engineering.
      </p>

      <!-- Quick actions -->
      <div class="mt-6 flex flex-wrap gap-2">
        {#each quick as q}
          <a
            class="group inline-flex items-center gap-2 rounded-full border border-neutral-800 bg-neutral-900 px-4 py-2 text-sm hover:border-neutral-700 hover:bg-neutral-800 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600"
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

    <!-- Projects -->
    <section id="work" class="mt-14">
      <div class="flex items-end justify-between">
        <h2 class="text-xl font-semibold text-neutral-100">Projects</h2>
        <a href="/projects" class="text-sm text-neutral-400 hover:text-neutral-200">All work →</a>
      </div>
    </section>

    <!-- Writing -->
    <section id="writing" class="mt-14">
      <div class="flex items-end justify-between">
        <!-- Left -->
        <h2 class="text-xl font-semibold text-neutral-100">Writing</h2>

        <!-- Right -->
        <div class="flex gap-4">
          <a
            href="/blog"
            class="text-sm text-neutral-400 hover:text-neutral-200"
            rel="noopener noreferrer"
          >
            Blog →
          </a>
          <a
            href="/notes"
            class="text-sm text-neutral-400 hover:text-neutral-200"
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
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
      <div class="mt-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 text-sm text-neutral-500">
        <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3">
          <a class="hover:text-neutral-300" href="/news">tech news</a>
          <span class="hidden sm:inline" aria-hidden="true">•</span>
          <a class="hover:text-neutral-300" href="/leetcode" rel="noopener noreferrer">leetcode stats</a>
          <span class="hidden sm:inline" aria-hidden="true">•</span>
          <a class="hover:text-neutral-300" href="/dashboard" rel="noopener noreferrer">my dashboard</a>
          <span class="hidden sm:inline" aria-hidden="true">•</span>
          <a class="hover:text-neutral-300" href="/docs">api docs</a>
        </div>
        <div class="text-neutral-600">© {new Date().getFullYear()} Kashif</div>
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
