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

  const projects = [
    {
      title: 'boo — Discord AI bot',
      href: 'https://github.com/VVIP-Kitchen/boo',
      tagline: 'Multi‑service bot: LLM chat, image gen, voice, web search. Dockerized stack with Go + Python, Postgres, Redis.'
    },
    {
      "title": "NoPokeDB — lightweight vector DB",
      "href": "https://github.com/kashifulhaque/nopokedb",
      "tagline": "Disk-backed vector database in Python using hnswlib + SQLite; durability with oplog, crash recovery, 2K+ PyPI downloads."
    },
    {
      title: 'NumPy-in-Rust sketch',
      href: 'https://github.com/kashifulhaque/tinyndarray',
      tagline: 'Learning-by-building: a tiny ndarray + Python bridge to grok the internals.'
    }
  ];

  const quick = [
    { label: 'GitHub', href: links.github },
    { label: 'LinkedIn', href: links.linkedin },
    { label: `${links.emailPrimary}`, href: `mailto:${links.emailPrimary}` },
    { label: `${links.emailAlt}`, href: `mailto:${links.emailAlt}` }
  ];
</script>

<!-- Page container -->
<div class="min-h-screen bg-neutral-950 text-neutral-200 selection:bg-neutral-800 selection:text-white">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header / Nav -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur supports-[backdrop-filter]:bg-neutral-950/60">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight text-neutral-100">ifkash.dev</a>
          <ul class="flex gap-3 text-sm">
            <li><a class="rounded-full px-3 py-1 hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600" href="/work">Work</a></li>
            <li><a class="rounded-full px-3 py-1 hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600" href="/education">Education</a></li>
            <li><a class="rounded-full px-3 py-1 hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600" href="/projects">Projects</a></li>
          </ul>

          <!-- Resume CTA -->
          <a
            href={links.resume}
            target="_blank"
            rel="noopener noreferrer"
            class="ml-1 inline-flex items-center gap-2 rounded-full border border-neutral-700 bg-neutral-900 px-3 py-1.5 text-sm text-neutral-100 hover:border-neutral-600 hover:bg-neutral-800 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600"
            aria-label="Resume"
          >
            <span>Resume</span>
            <span aria-hidden="true">↗</span>
          </a>
        </nav>
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

      <!-- Now -->
      <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4">
        <div class="text-xs uppercase tracking-wider text-neutral-500">Now</div>
        <p class="mt-1 text-sm text-neutral-300">
          Mornings in the gym; days building <em>boo</em>, a Discord AI bot; nights learning ML systems and kernel land.
        </p>
      </div>
    </section>

    <!-- Projects -->
    <section id="work" class="mt-14">
      <div class="flex items-end justify-between">
        <h2 class="text-xl font-semibold text-neutral-100">Projects</h2>
        <a href="/projects" class="text-sm text-neutral-400 hover:text-neutral-200">All work →</a>
      </div>

      <div class="mt-6 grid gap-4">
        {#each projects as p}
          <a href={p.href} rel="noopener noreferrer" class="group rounded-2xl border border-neutral-800 bg-neutral-900/60 p-5 hover:border-neutral-700 hover:bg-neutral-900 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">
            <div class="flex items-center justify-between">
              <h3 class="text-base font-medium text-neutral-100 group-hover:underline">{p.title}</h3>
              <span class="text-neutral-600 group-hover:text-neutral-400">→</span>
            </div>
            <p class="mt-2 text-sm text-neutral-400">{p.tagline}</p>
          </a>
        {/each}
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
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm text-neutral-500">
        <div class="flex items-center gap-3">
          <a class="hover:text-neutral-300" href="/news">tech news</a>
          <span aria-hidden="true">•</span>
          <a class="hover:text-neutral-300" href="/leetcode" rel="noopener noreferrer">leetcode stats</a>
          <span aria-hidden="true">•</span>
          <a class="hover:text-neutral-300" href="/dashboard" rel="noopener noreferrer">my dashboard</a>
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
</style>
