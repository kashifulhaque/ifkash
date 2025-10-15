<svelte:head>
  <title>Projects — Kashif</title>
  <meta name="description" content="Selected side projects by Kashif." />
</svelte:head>

<script lang="ts">
  type Project = {
    title: string;
    links?: { text: string; url: string }[];
    description?: string;
    htmlDescription?: string;
    tech: string[];
  };

  const projects: Project[] = [
    {
      title: 'Boo',
      links: [{ text: 'git repo', url: 'https://github.com/VVIP-Kitchen/boo' }],
      description:
        "Innovative AI-Powered Discord Bot who engages in natural conversations, interprets and responds to image inputs, and generates high-quality images on demand.",
      tech: [
        'Go',
        'Docker',
        'Python',
        'Discord.py',
        'LLMs',
        "Llama 4",
        'OpenRouter',
      ]
    }
  ];
</script>

<div class="min-h-screen bg-neutral-950 text-neutral-200 selection:bg-neutral-800 selection:text-white">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur supports-[backdrop-filter]:bg-neutral-950/60">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight text-neutral-100">ifkash.dev</a>
          <a href="/" class="rounded-full px-3 py-1 text-sm hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">Home</a>
        </nav>
      </div>
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
    </header>

    <!-- Title -->
    <section class="pt-14 sm:pt-20" aria-labelledby="projects-title">
      <h1 id="projects-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight text-neutral-100">Projects</h1>
      <p class="mt-2 text-neutral-400 max-w-2xl">A few things I’ve built or am actively shaping.</p>
    </section>

    <!-- Projects grid -->
    <section class="mt-8 grid gap-4">
      {#each projects as p}
        <article class="group rounded-2xl border border-neutral-800 bg-neutral-900/60 p-5 hover:border-neutral-700 hover:bg-neutral-900 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">
          <header class="flex items-start justify-between gap-3">
            <h2 class="text-lg font-medium text-neutral-100">{p.title}</h2>
            {#if p.links && p.links.length}
              <div class="shrink-0 flex flex-wrap gap-2">
                {#each p.links as link}
                  <a href={link.url} target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-1 rounded-full border border-neutral-800 bg-neutral-900 px-3 py-1 text-xs text-neutral-300 hover:border-neutral-700 hover:bg-neutral-800">
                    <span>{link.text}</span>
                    <span aria-hidden="true">↗</span>
                  </a>
                {/each}
              </div>
            {/if}
          </header>

          <div class="mt-2 text-sm text-neutral-400">
            {#if p.htmlDescription}
              <div class="prose prose-invert prose-neutral max-w-none prose-a:underline">{@html p.htmlDescription}</div>
            {:else if p.description}
              <p>{p.description}</p>
            {/if}
          </div>

          {#if p.tech?.length}
            <div class="mt-4 flex flex-wrap gap-2">
              {#each p.tech as t}
                <span class="rounded-full border border-neutral-800 bg-neutral-900 px-2.5 py-1 text-xs text-neutral-300">{t}</span>
              {/each}
            </div>
          {/if}
        </article>
      {/each}
    </section>

    <!-- Footer -->
    <footer class="mt-16">
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm text-neutral-500">
        <a class="hover:text-neutral-300" href="/">Back to home</a>
        <div class="text-neutral-600">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>
