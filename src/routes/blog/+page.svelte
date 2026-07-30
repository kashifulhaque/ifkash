<svelte:head>
  <title>Blog — Kashif</title>
  <meta
    name="description"
    content="Writing, notes, and experiments live on blog.ifkash.dev."
  />
</svelte:head>

<header class="page-header">
  <h1 class="section-title">Blog</h1>
  <p class="section-subtitle">Writing, notes, and experiments</p>
</header>

<section class="blog-module">
  <span class="tick tl" aria-hidden="true"></span>
  <span class="tick tr" aria-hidden="true"></span>
  <span class="tick bl" aria-hidden="true"></span>
  <span class="tick br" aria-hidden="true"></span>

  <p class="eyebrow"><span class="led" aria-hidden="true"></span>External archive</p>
  <h2 class="blog-headline">Writing lives<br />on Hashnode</h2>
  <p class="blog-copy">
    The blog is hosted separately at <strong>blog.ifkash.dev</strong>. Head
    there for posts, notes, and experiments.
  </p>
  <a
    class="blog-button"
    href="https://blog.ifkash.dev"
    target="_blank"
    rel="noopener noreferrer"
  >
    Visit blog.ifkash.dev &nearr;
  </a>
</section>

<style>
  .page-header {
    padding-top: 16px;
  }

  .blog-module {
    position: relative;
    margin-top: 32px;
    padding: clamp(32px, 6vw, 64px);
    border: 1px solid var(--line-soft);
    background:
      radial-gradient(circle, var(--dots) 1px, transparent 1.4px);
    background-size: 14px 14px;
    background-color: var(--panel);
  }

  /* corner registration ticks */
  .tick {
    position: absolute;
    width: 14px;
    height: 14px;
    border: 1px solid var(--ink-mute);
  }

  .tick.tl {
    top: -1px;
    left: -1px;
    border-right: none;
    border-bottom: none;
  }

  .tick.tr {
    top: -1px;
    right: -1px;
    border-left: none;
    border-bottom: none;
  }

  .tick.bl {
    bottom: -1px;
    left: -1px;
    border-right: none;
    border-top: none;
  }

  .tick.br {
    bottom: -1px;
    right: -1px;
    border-left: none;
    border-top: none;
  }

  .eyebrow {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 20px;
    font-family: var(--font-mono-g);
    font-size: 0.68rem;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--ink-soft);
  }

  .blog-headline {
    margin-bottom: 20px;
    font-family: var(--font-dots);
    font-size: clamp(2.6rem, 7vw, 5rem);
    font-weight: 600;
    line-height: 0.92;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--ink);
  }

  .blog-copy {
    max-width: 560px;
    margin-bottom: 32px;
    font-family: var(--font-sans-g);
    font-size: clamp(1rem, 1.6vw, 1.1rem);
    line-height: 1.65;
    color: var(--ink-soft);
  }

  .blog-copy strong {
    color: var(--ink);
    font-weight: 600;
  }

  .blog-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 44px;
    padding: 0 20px;
    font-family: var(--font-mono-g);
    font-size: 0.72rem;
    font-weight: 500;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--void);
    background: var(--ink);
    border: 1px solid var(--ink);
    border-radius: var(--radius-sm);
    transition:
      background 0.15s,
      color 0.15s,
      border-color 0.15s;
  }

  .blog-button:hover {
    color: #ffffff;
    background: var(--signal);
    border-color: var(--signal);
  }

  @media (max-width: 768px) {
    .blog-module {
      margin-top: 24px;
    }
  }
</style>
