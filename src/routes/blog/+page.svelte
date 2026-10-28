<svelte:head>
  <title>Blog — Kashif</title>
  <meta
    name="description"
    content="Writing, notes, and experiments from Kashiful Haque."
  />
</svelte:head>

<script lang="ts">
  import { reveal, splitReveal } from '$lib/motion';
</script>

<header class="page-head">
  <p class="eyebrow">06 / Blog</p>
  <h1 class="page-title" use:splitReveal>Blog</h1>
  <p class="lead">Writing, notes, and experiments</p>
</header>

<section class="archive" aria-labelledby="archive-heading" use:reveal={{ delay: 150 }}>
  <p class="eyebrow archive-label">External archive</p>
  <div class="archive-body">
    <h2 id="archive-heading">Writing lives on <em class="serif">Hashnode</em></h2>
    <p class="archive-copy">
      Read my engineering notes, project write-ups, and experiments at
      blog.ifkash.dev.
    </p>
    <a
      class="btn btn-primary"
      href="https://blog.ifkash.dev"
      target="_blank"
      rel="noopener noreferrer"
    >
      Visit blog.ifkash.dev ↗
    </a>
  </div>
</section>

<style>
  .archive {
    display: grid;
    grid-template-columns: 184px minmax(0, 1fr);
    column-gap: var(--s-5);
    padding: var(--s-8) 0 var(--s-4);
  }

  .archive-label {
    align-self: start;
    padding-top: 0.6em;
  }

  .archive-body {
    max-width: var(--measure);
  }

  .archive-body h2 {
    max-width: 18ch;
    margin-bottom: var(--s-4);
  }

  .archive-copy {
    margin-bottom: var(--s-6);
    color: var(--ink-2);
    font-size: var(--text-md);
    line-height: 1.55;
  }

  @media (max-width: 720px) {
    .archive {
      grid-template-columns: minmax(0, 1fr);
      row-gap: var(--s-4);
      padding-top: var(--s-7);
    }

    .archive-label {
      padding-top: 0;
    }
  }
</style>
