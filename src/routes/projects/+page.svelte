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
      title: 'smol-llama 🦙',
      links: [
        { text: 'HuggingFace', url: 'https://huggingface.co/ifkash/smol-llama' },
        { text: 'dataset', url: 'https://huggingface.co/datasets/ifkash/fineweb-6b' }
      ],
      description:
        'A 360M parameter LLaMA-style language model pre-trained from scratch on 6 billion tokens of web data. Features Grouped Query Attention (GQA), RoPE embeddings, and SwiGLU activations. Trained on a single H100 GPU in ~22 hours for $53, demonstrating that high-quality small language models can be trained efficiently.'
    },
    {
      title: 'FineWeb-6B Dataset',
      links: [
        { text: 'HuggingFace', url: 'https://huggingface.co/datasets/ifkash/fineweb-6b' }
      ],
      description:
        'A curated subset of the FineWeb dataset containing 6 billion high-quality web tokens (16.1 GB parquet, 11.3 GB tokenized). Includes pre-tokenized binary files with a custom 49K vocab BPE tokenizer, designed for efficient language model pre-training experiments without requiring massive computational resources.'
    },
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

<div class="max-w-3xl mx-auto">
  <!-- Title -->
  <section class="mb-12" aria-labelledby="projects-title">
    <h1 id="projects-title" class="text-3xl font-bold tracking-tight text-[var(--color-headline)]">Projects</h1>
    <p class="mt-2 text-lg text-[var(--color-paragraph)]">A few things I've built or am actively shaping.</p>
  </section>

  <!-- Projects grid -->
  <section class="grid gap-6">
    {#each projects as p}
      <article class="p-6 rounded-xl bg-[var(--color-surface)] border border-[var(--color-border)] hover:border-[var(--color-secondary)] transition-all">
        <header class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-4">
          <h2 class="text-xl font-semibold text-[var(--color-headline)]">{p.title}</h2>
          {#if p.links && p.links.length}
            <div class="flex flex-wrap gap-2">
              {#each p.links as link}
                <a href={link.url} target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-1 px-3 py-1 text-xs font-medium rounded-full bg-[var(--color-background)] text-[var(--color-paragraph)] border border-[var(--color-border)] hover:text-[var(--color-headline)] hover:border-[var(--color-highlight)] transition-colors">
                  {link.text}
                </a>
              {/each}
            </div>
          {/if}
        </header>

        <div class="text-sm leading-relaxed text-[var(--color-paragraph)]">
          {#if p.htmlDescription}
            <div class="prose prose-invert max-w-none">{@html p.htmlDescription}</div>
          {:else if p.description}
            <p>{p.description}</p>
          {/if}
        </div>
      </article>
    {/each}
  </section>
</div>
