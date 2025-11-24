<svelte:head>
  <title>Kashif — Applied ML Engineer</title>
  <meta name="description" content="Personal website of Kashiful Haque, Applied ML Engineer.">
</svelte:head>

<section class="mb-16">
  <h1 class="text-4xl font-bold tracking-tight text-[var(--color-headline)] mb-4">
    Hi, I'm Kashif.
  </h1>
  <p class="text-lg text-[var(--color-paragraph)] leading-relaxed max-w-2xl">
    I'm an Applied ML Engineer based in Palo Alto and Bangalore. I build robust systems and ship thoughtful tools, currently optimizing LLM inference and bridging models with production infrastructure at <a href="https://wand.ai" target="_blank" class="text-[var(--color-headline)] hover:text-[var(--color-highlight)] underline decoration-[var(--color-secondary)] underline-offset-4 decoration-1 transition-all">Wand AI</a>.
  </p>
</section>

<section class="grid grid-cols-1 md:grid-cols-2 gap-4">
  
  <a href="/work" class="card group">
    <div class="flex justify-between items-center mb-2">
      <h3 class="text-lg font-semibold text-[var(--color-headline)] group-hover:text-[var(--color-highlight)] transition-colors">Work</h3>
      <span class="text-[var(--color-secondary)] text-sm">→</span>
    </div>
    <p class="text-sm text-[var(--color-paragraph)]">
      Experience at Wand AI, American Express, and Fiery.
    </p>
  </a>

  <a href="/projects" class="card group">
    <div class="flex justify-between items-center mb-2">
      <h3 class="text-lg font-semibold text-[var(--color-headline)] group-hover:text-[var(--color-highlight)] transition-colors">Projects</h3>
      <span class="text-[var(--color-secondary)] text-sm">→</span>
    </div>
    <p class="text-sm text-[var(--color-paragraph)]">
      Smoltorch, Boo Bot, NoPokeDB, and more.
    </p>
  </a>

  <a href="/leetcode" class="card group">
    <div class="flex justify-between items-center mb-2">
      <h3 class="text-lg font-semibold text-[var(--color-headline)] group-hover:text-[var(--color-highlight)] transition-colors">LeetCode</h3>
      <span class="text-[var(--color-secondary)] text-sm">→</span>
    </div>
    <p class="text-sm text-[var(--color-paragraph)]">
      Problem solving stats and recent submissions.
    </p>
  </a>

  <a href="/news" class="card group">
    <div class="flex justify-between items-center mb-2">
      <h3 class="text-lg font-semibold text-[var(--color-headline)] group-hover:text-[var(--color-highlight)] transition-colors">Tech News</h3>
      <span class="text-[var(--color-secondary)] text-sm">→</span>
    </div>
    <p class="text-sm text-[var(--color-paragraph)]">
      Latest top stories from Hacker News.
    </p>
  </a>

</section>

<style>
  .card {
    display: block;
    padding: 1.5rem;
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 0.75rem;
    transition: all 0.2s ease;
  }
  .card:hover {
    border-color: var(--color-secondary);
    transform: translateY(-2px);
  }
</style>
