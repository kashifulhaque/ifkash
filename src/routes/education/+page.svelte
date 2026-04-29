<svelte:head>
  <title>Education — Kashiful Haque</title>
  <meta name="description" content="Academic background, areas of focus, and research interests." />
</svelte:head>

<script lang="ts">
  const focusAreas = [
    { title: 'Machine Learning', tag: 'core' },
    { title: 'Deep Learning', tag: 'core' },
    { title: 'Reinforcement Learning', tag: 'frontier' },
    { title: 'Distributed Systems', tag: 'systems' },
    { title: 'Linear Algebra', tag: 'math' },
    { title: 'Statistics', tag: 'math' },
    { title: 'Algorithms', tag: 'cs' },
    { title: 'Data Structures', tag: 'cs' },
    { title: 'C++ / Rust', tag: 'systems' },
    { title: 'Python', tag: 'cs' },
  ];
</script>

<div class="page">
  <header class="page-header">
    <span class="eyebrow">section · 03 · education</span>
    <h1 class="page-title">
      Where I learned<br />
      <em>to do the math.</em>
    </h1>
    <p class="page-desc">
      Formal study in data science and the foundations that keep the
      modeling honest.
    </p>
  </header>

  <section class="education">
    <article class="edu-card">
      <div class="edu-rail">
        <div class="edu-corner edu-corner-tl"></div>
        <div class="edu-corner edu-corner-tr"></div>
        <div class="edu-corner edu-corner-bl"></div>
        <div class="edu-corner edu-corner-br"></div>

        <div class="edu-num">[ 01 ]</div>

        <div class="edu-main">
          <h2 class="edu-school">Indian Institute of Technology Madras</h2>
          <p class="edu-degree">B.Sc. Data Science and Applications</p>
          <p class="edu-detail">
            Foundations in statistics, optimization, and software engineering
            with a focus on applied machine learning. Capstone work on
            production-scale data systems.
          </p>
        </div>

        <dl class="edu-meta">
          <div>
            <dt>Period</dt>
            <dd>2020 — 2024</dd>
          </div>
          <div>
            <dt>Located</dt>
            <dd>Chennai, India</dd>
          </div>
          <div>
            <dt>Track</dt>
            <dd>Data Science</dd>
          </div>
        </dl>
      </div>
    </article>
  </section>

  <section class="focus">
    <header class="focus-head">
      <span class="eyebrow">areas of focus</span>
      <h2 class="focus-title">
        Subjects I keep<br />
        <span class="serif-italic">coming back to.</span>
      </h2>
    </header>

    <div class="focus-grid">
      {#each focusAreas as area, i}
        <div class="focus-tile stagger" style="--i: {i}">
          <span class="focus-tag">{area.tag}</span>
          <span class="focus-name">{area.title}</span>
        </div>
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
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    padding-bottom: var(--space-xl);
    border-bottom: 1px solid var(--border);
    max-width: 760px;
  }

  .page-title {
    font-size: clamp(2.5rem, 5vw + 0.5rem, 4.25rem);
    letter-spacing: -0.04em;
    line-height: 1.05;
    color: var(--text-primary);
    font-weight: 400;
  }

  .page-title em {
    font-family: var(--font-serif);
    font-style: italic;
    font-weight: 300;
    color: var(--accent);
    letter-spacing: -0.02em;
  }

  .page-desc {
    font-size: 1.0625rem;
    line-height: 1.55;
    color: var(--text-tertiary);
    max-width: 56ch;
  }

  /* Edu card */

  .edu-card {
    position: relative;
  }

  .edu-rail {
    position: relative;
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-xl);
    padding: var(--space-2xl) var(--space-xl);
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    overflow: hidden;
  }

  .edu-rail::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--accent), transparent);
    opacity: 0.6;
  }

  @media (min-width: 720px) {
    .edu-rail {
      grid-template-columns: auto 1fr auto;
      gap: var(--space-2xl);
      align-items: start;
    }
  }

  .edu-corner {
    position: absolute;
    width: 16px;
    height: 16px;
    border-color: var(--accent);
    opacity: 0.7;
  }
  .edu-corner-tl { top: 12px; left: 12px; border-top: 1px solid; border-left: 1px solid; }
  .edu-corner-tr { top: 12px; right: 12px; border-top: 1px solid; border-right: 1px solid; }
  .edu-corner-bl { bottom: 12px; left: 12px; border-bottom: 1px solid; border-left: 1px solid; }
  .edu-corner-br { bottom: 12px; right: 12px; border-bottom: 1px solid; border-right: 1px solid; }

  .edu-num {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--accent);
    letter-spacing: 0.06em;
    padding-top: 0.25rem;
  }

  .edu-main {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 0;
  }

  .edu-school {
    font-size: clamp(1.5rem, 2.5vw + 0.5rem, 2rem);
    font-weight: 500;
    letter-spacing: -0.025em;
    color: var(--text-primary);
    line-height: 1.15;
  }

  .edu-degree {
    font-size: 1rem;
    color: var(--text-secondary);
    letter-spacing: -0.005em;
  }

  .edu-detail {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--text-tertiary);
    max-width: 60ch;
    margin-top: 0.4rem;
  }

  .edu-meta {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }

  @media (min-width: 720px) {
    .edu-meta {
      grid-template-columns: 1fr;
      gap: 1rem;
      padding-top: 0;
      border-top: none;
      padding-left: var(--space-xl);
      border-left: 1px solid var(--border);
      min-width: 160px;
    }
  }

  .edu-meta > div {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .edu-meta dt {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--text-faint);
  }

  .edu-meta dd {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  /* Focus */

  .focus {
    display: flex;
    flex-direction: column;
    gap: var(--space-xl);
  }

  .focus-head {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
  }

  .focus-title {
    font-size: clamp(1.75rem, 3vw + 0.5rem, 2.5rem);
    letter-spacing: -0.03em;
    line-height: 1.1;
    color: var(--text-primary);
    font-weight: 400;
  }

  .focus-title :global(.serif-italic) {
    color: var(--text-tertiary);
  }

  .focus-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.5rem;
  }

  @media (min-width: 600px) {
    .focus-grid {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  @media (min-width: 900px) {
    .focus-grid {
      grid-template-columns: repeat(5, 1fr);
    }
  }

  .focus-tile {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 1rem 1rem 0.85rem;
    background: var(--surface-glass);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: border-color var(--dur-instant), color var(--dur-instant), transform var(--dur-fast);
  }

  .focus-tile:hover {
    border-color: var(--border-bright);
    transform: translateY(-2px);
  }

  .focus-tag {
    font-family: var(--font-mono);
    font-size: 0.5625rem;
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--text-faint);
  }

  .focus-tile:hover .focus-tag {
    color: var(--accent);
  }

  .focus-name {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }
</style>
