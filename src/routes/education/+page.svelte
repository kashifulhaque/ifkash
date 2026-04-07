<svelte:head>
  <title>Education — Kashif</title>
  <meta name="description" content="Academic background." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <h1 class="page-title">Education</h1>
    <p class="page-desc">Formal studies and academic foundation.</p>
  </header>

  <section class="education">
    <article class="edu-item">
      <div class="edu-main">
        <h2 class="edu-school">Indian Institute of Technology Madras</h2>
        <p class="edu-degree">B.Sc. Data Science and Applications</p>
      </div>
      <div class="edu-meta">
        <span class="edu-year">2020 — 2024</span>
        <span class="edu-location">Chennai, India</span>
      </div>
    </article>
  </section>

  <section class="focus-section">
    <h3 class="focus-heading">Areas of Focus</h3>
    <div class="focus-tags">
      {#each ['Machine Learning', 'Deep Learning', 'Statistics', 'Linear Algebra', 'Data Structures', 'Algorithms', 'Python', 'Distributed Systems'] as skill, i}
        <span class="tag stagger" style="--i: {i}">{skill}</span>
      {/each}
    </div>
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-2xl);
  }

  .page-header {
    padding-bottom: var(--space-xl);
    border-bottom: 1px solid var(--border);
  }

  .page-title {
    font-size: clamp(2rem, 5vw + 0.5rem, 3rem);
    font-weight: 400;
    letter-spacing: -0.03em;
    color: var(--text-primary);
    margin-bottom: 0.375rem;
  }

  .page-desc {
    font-size: 1rem;
    color: var(--text-tertiary);
  }

  .edu-item {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1rem;
    padding: 1.5rem;
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }

  @media (min-width: 640px) {
    .edu-item {
      grid-template-columns: 1fr auto;
      gap: 2rem;
      align-items: start;
    }
  }

  .edu-main {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .edu-school {
    font-size: 1.125rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .edu-degree {
    font-size: 0.9375rem;
    color: var(--text-secondary);
  }

  .edu-meta {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    text-align: left;
  }

  @media (min-width: 640px) {
    .edu-meta {
      text-align: right;
    }
  }

  .edu-year {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .edu-location {
    font-size: 0.75rem;
    color: var(--text-faint);
  }

  .focus-section {
    border-top: 1px solid var(--border);
    padding-top: var(--space-xl);
  }

  .focus-heading {
    font-family: var(--font-sans);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: var(--space-md);
  }

  .focus-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .tag {
    padding: 0.4rem 0.75rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    transition: color var(--dur-instant) var(--ease-out-quart),
                border-color var(--dur-instant) var(--ease-out-quart);
  }

  .tag:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
  }
</style>
