<svelte:head>
  <title>Blog — Kashif</title>
  <meta
    name="description"
    content="Writing, notes, and experiments from Kashiful Haque."
  />
</svelte:head>

<header class="page-header">
  <h1 class="section-title">Blog</h1>
  <p class="section-subtitle">Writing, notes, and experiments</p>
</header>

<section class="blog-module" aria-labelledby="archive-heading">
  <p class="archive-label">External archive</p>
  <h2 id="archive-heading">Writing lives on Hashnode</h2>
  <p class="blog-copy">
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
</section>

<style>
  .page-header {
    padding-top: 16px;
  }

  .blog-module {
    max-width: 680px;
    padding: 48px 0 16px;
  }

  .archive-label {
    margin-bottom: 16px;
    color: var(--ink-3);
    font-size: 13px;
    font-weight: 500;
  }

  .blog-module h2 {
    max-width: 18ch;
    margin-bottom: 20px;
  }

  .blog-copy {
    max-width: 58ch;
    margin-bottom: 28px;
    color: var(--ink-2);
    font-size: 16px;
    line-height: 1.6;
  }

  @media (max-width: 768px) {
    .blog-module {
      padding-top: 32px;
    }
  }
</style>
