<svelte:head>
  <title>Blog — Kashif</title>
  <meta name="description" content="Redirecting to blog.ifkash.dev." />
</svelte:head>

<article class="article">
  <h1 class="article-title">Redirecting to blog.ifkash.dev</h1>
  <p class="article-subtitle">
    This article lives on the external blog archive.
  </p>
  <a href="/blog" class="back-link">&larr; Back to Blog</a>
</article>

<style>
  .article {
    max-width: 760px;
    margin: 0 auto;
    padding-top: 16px;
    animation: enter var(--dur-slow) var(--ease-out-expo);
  }

  .back-link {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-soft);
    border-bottom: none;
  }

  .back-link:hover {
    color: var(--blueprint);
    border-bottom: none;
  }

  .article-title {
    font-family: var(--font-display);
    font-size: clamp(2.4rem, 6vw, 4rem);
    line-height: 0.92;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--blueprint);
    margin-bottom: 16px;
  }

  .article-subtitle {
    font-family: var(--font-body);
    font-size: clamp(1.05rem, 1.7vw, 1.2rem);
    line-height: 1.55;
    color: var(--ink-soft);
    font-style: italic;
    margin-bottom: 24px;
  }

  @media (max-width: 768px) {
    .article-title {
      font-size: clamp(2rem, 9vw, 2.6rem);
    }

  }
</style>
