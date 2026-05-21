<svelte:head>
  <title>Blog — Kashif</title>
  <meta
    name="description"
    content="Writing, notes, and experiments live on blog.ifkash.dev."
  />
</svelte:head>

<header class="page-header">
  <h1 class="section-title">Blog.</h1>
</header>

<section class="blog-card">
  <p class="eyebrow">External archive</p>
  <h2>Writing lives on Hashnode.</h2>
  <p>
    The blog is hosted separately at <strong>blog.ifkash.dev</strong>. Head
    there for posts, notes, and experiments.
  </p>
  <a
    class="blog-button"
    href="https://blog.ifkash.dev"
    target="_blank"
    rel="noopener noreferrer"
  >
    Visit blog.ifkash.dev
  </a>
</section>

<style>
  .page-header {
    padding-top: 16px;
  }

  .blog-card {
    margin-top: 32px;
    padding: clamp(28px, 5vw, 48px);
    border: 1px solid var(--rule-soft);
    background: var(--blueprint-tint);
  }

  .eyebrow {
    margin-bottom: 16px;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--blueprint);
  }

  .blog-card h2 {
    margin-bottom: 16px;
    font-family: var(--font-display);
    font-size: clamp(2rem, 6vw, 4rem);
    line-height: 0.95;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--blueprint);
  }

  .blog-card p {
    max-width: 620px;
    margin-bottom: 28px;
    font-family: var(--font-body);
    font-size: clamp(1rem, 1.6vw, 1.15rem);
    line-height: 1.65;
    color: var(--ink-soft);
  }

  .blog-card strong {
    color: var(--ink);
  }

  .blog-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 44px;
    padding: 0 18px;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--blueprint);
    border: 1px solid var(--blueprint);
    background: var(--bg);
    transition: background 0.15s, color 0.15s;
  }

  .blog-button:hover {
    color: var(--bg);
    background: var(--blueprint);
    border-bottom-color: var(--blueprint);
  }

  @media (max-width: 768px) {
    .blog-card {
      margin-top: 24px;
    }
  }
</style>
