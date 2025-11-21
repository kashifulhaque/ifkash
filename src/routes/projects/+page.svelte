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
  };

  const projects: Project[] = [
    {
      title: 'smoltorch',
      links: [
        { text: 'git repo', url: 'https://github.com/kashifulhaque/smoltorch' },
        { text: 'PyPI', url: 'https://pypi.org/project/smoltorch/' },
        { text: 'blog', url: 'https://blog.ifkash.dev/smoltorch' }
      ],
      description:
        'A minimalist deep learning library that implements automatic differentiation (autograd) and neural networks from scratch using only NumPy. Designed to be educational, transparent, functional, and minimal (~500 lines of readable, well-documented Python code).'
    },
    {
      title: 'Boo',
      links: [
        { text: 'git repo', url: 'https://github.com/VVIP-Kitchen/boo' },
        { text: 'blog', url: 'https://vvip.ifkash.dev/blog/boo' },
        { text: 'website', url: 'https://boo.ifkash.dev' }
      ],
      description:
        "Innovative AI-Powered Discord Bot who engages in natural conversations, interprets and responds to image inputs, and generates high-quality images on demand."
    },
    {
      title: 'NoPokeDB – lightweight vector DB',
      links: [
        { text: 'git repo', url: 'https://github.com/kashifulhaque/nopokedb' },
        { text: 'PyPI', url: 'https://pypi.org/project/nopokedb/' }
      ],
      description: 'A lightweight, disk-backed vector database implementation using hnswlib + SQLite with durability guarantees and operational log for crash recovery. 2K+ PyPI downloads'
    },
    {
      title: 'NumPy-in-Rust sketch',
      links: [{ text: 'git repo', url: 'https://github.com/kashifulhaque/tinyndarray' }],
      description: 'A learning project implementing NumPy-like ndarray functionality in Rust with Python bindings.'
    }
  ];
</script>

<div class="min-h-screen selection:text-white" style="background-color: var(--color-background); color: var(--color-paragraph);">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur" style="backdrop-filter: blur(12px); background-color: rgba(22, 22, 26, 0.6);">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight" style="color: var(--color-headline);">ifkash.dev</a>
          <a href="/" class="rounded-full px-3 py-1 text-sm focus:outline-none transition-colors" style="color: var(--color-paragraph);" onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.backgroundColor='transparent'">Home</a>
        </nav>
      </div>
      <div class="h-px w-full" style="background: linear-gradient(to right, transparent, var(--color-secondary), transparent);"></div>
    </header>

    <!-- Title -->
    <section class="pt-14 sm:pt-20" aria-labelledby="projects-title">
      <h1 id="projects-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight" style="color: var(--color-headline);">Projects</h1>
      <p class="mt-2 max-w-2xl" style="color: var(--color-paragraph);">A few things I've built or am actively shaping.</p>
    </section>

    <!-- Projects grid -->
    <section class="mt-8 grid gap-4">
      {#each projects as p}
        <article class="group rounded-2xl border p-5 focus:outline-none transition-colors" style="border-color: var(--color-secondary); background-color: rgba(114, 117, 126, 0.1);" onmouseover="this.style.borderColor='var(--color-highlight)'; this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.borderColor='var(--color-secondary)'; this.style.backgroundColor='rgba(114, 117, 126, 0.1)'">
          <header class="flex items-start justify-between gap-3">
            <h2 class="text-lg font-medium" style="color: var(--color-headline);">{p.title}</h2>
            {#if p.links && p.links.length}
              <div class="shrink-0 flex flex-wrap gap-2">
                {#each p.links as link}
                  <a href={link.url} target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-1 rounded-full border px-3 py-1 text-xs transition-colors" style="border-color: var(--color-secondary); background-color: rgba(114, 117, 126, 0.1); color: var(--color-paragraph);" onmouseover="this.style.borderColor='var(--color-highlight)'; this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.borderColor='var(--color-secondary)'; this.style.backgroundColor='rgba(114, 117, 126, 0.1)'">
                    <span>{link.text}</span>
                    <span aria-hidden="true">↗</span>
                  </a>
                {/each}
              </div>
            {/if}
          </header>

          <div class="mt-2 text-xs" style="color: var(--color-paragraph);">
            {#if p.htmlDescription}
              <div class="prose prose-invert prose-neutral max-w-none prose-a:underline">{@html p.htmlDescription}</div>
            {:else if p.description}
              <p>{p.description}</p>
            {/if}
          </div>
        </article>
      {/each}
    </section>

    <!-- Footer -->
    <footer class="mt-16">
      <div class="h-px w-full" style="background: linear-gradient(to right, transparent, var(--color-secondary), transparent);"></div>
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm" style="color: var(--color-secondary);">
        <a class="transition-colors" onmouseover="this.style.color='var(--color-paragraph)'" onmouseout="this.style.color='var(--color-secondary)'" href="/">Back to home</a>
        <div style="color: var(--color-secondary);">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>
