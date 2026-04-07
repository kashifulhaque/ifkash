<script>
  import { auth } from "$lib/stores/auth";
  import { dev } from "$app/environment";

  const resumeUrl = dev
    ? "http://localhost:8787/api/resume?format=view"
    : "/api/resume?format=view";
</script>

<svelte:head>
  <title>Kashiful Haque</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, ML Engineer."
  />
</svelte:head>

<div class="home">
  <section class="hero">
    <h1 class="hero-name">Kashiful Haque</h1>
    <p class="hero-bio">
      Building low-level ML infrastructure and production alignment workflows.
      Focused on transformer pre-training, reinforcement learning, and
      high-performance inference systems in C++ and Rust.
    </p>
  </section>

  <section class="quick-links">
    <a href={resumeUrl} class="quick-link">
      <span class="quick-link-text">Resume</span>
      <span class="quick-link-icon">&nearr;</span>
    </a>
    {#if $auth.isAuthenticated}
      <a href="/editor" class="quick-link">
        <span class="quick-link-text">Editor</span>
        <span class="quick-link-icon">&rarr;</span>
      </a>
    {/if}
  </section>

  <section class="now">
    <dl class="now-list">
      <div class="now-item">
        <dt>Currently</dt>
        <dd>ML Engineer @ wand.ai</dd>
      </div>
      <div class="now-item">
        <dt>Latest</dt>
        <dd>
          <a
            href="https://marketplace.visualstudio.com/items?itemName=ifkash.kernel-orbit"
            class="now-link"
            target="_blank"
            rel="noopener noreferrer"
          >
            Kernel Orbit: VS Code Extension
          </a>
        </dd>
      </div>
    </dl>
  </section>
</div>

<style>
  .home {
    display: flex;
    flex-direction: column;
    gap: var(--space-3xl);
  }

  .hero {
    padding-top: var(--space-xl);
  }

  .hero-name {
    font-size: clamp(2.5rem, 6vw + 1rem, 4.5rem);
    font-weight: 400;
    letter-spacing: -0.035em;
    line-height: 1.05;
    color: var(--text-primary);
    margin-bottom: var(--space-lg);
  }

  .hero-bio {
    font-size: 1.125rem;
    line-height: 1.7;
    color: var(--text-secondary);
    max-width: 52ch;
  }

  .quick-links {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .quick-link {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    transition: background var(--dur-instant) var(--ease-out-quart);
  }

  .quick-link + .quick-link {
    border-top: 1px solid var(--border);
  }

  .quick-link:hover {
    background: var(--surface-sunken);
  }

  .quick-link-text {
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .quick-link-icon {
    font-size: 1rem;
    color: var(--text-faint);
    transition: color var(--dur-instant) var(--ease-out-quart),
                transform var(--dur-fast) var(--ease-out-quart);
  }

  .quick-link:hover .quick-link-icon {
    color: var(--accent);
    transform: translateX(3px);
  }

  .now {
    border-top: 1px solid var(--border);
    padding-top: var(--space-xl);
  }

  .now-list {
    margin: 0;
  }

  .now-item {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 0.75rem 0;
  }

  .now-item + .now-item {
    border-top: 1px solid var(--border-subtle);
  }

  .now-item dt {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .now-item dd {
    margin: 0;
    font-size: 0.9375rem;
    color: var(--text-secondary);
    text-align: right;
  }

  .now-link {
    color: var(--text-secondary);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .now-link:hover {
    color: var(--accent);
  }
</style>
