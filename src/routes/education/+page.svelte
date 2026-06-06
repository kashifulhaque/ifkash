<svelte:head>
  <title>Education — Kashif</title>
  <meta name="description" content="Academic background." />
</svelte:head>

<header class="page-header">
  <h1 class="section-title">Education.</h1>
</header>

<section class="edu-list">
  <article class="edu-row">
    <span class="edu-status"></span>
    <div class="edu-main">
      <h2 class="edu-school">Indian Institute of Technology Madras</h2>
      <p class="edu-degree">BS, Data Science and Applications</p>
    </div>
    <span class="edu-period">2020 — 2024</span>
    <span class="edu-location">Chennai, India</span>
  </article>
</section>

<style>
  .page-header {
    padding-top: 16px;
  }

  .edu-list {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--rule-soft);
  }

  .edu-row {
    display: grid;
    grid-template-columns: 14px minmax(0, 1fr) auto auto;
    align-items: baseline;
    gap: 24px;
    padding: 20px 0;
    border-bottom: 1px solid var(--rule-soft);
  }

  .edu-status {
    width: 12px;
    height: 12px;
    background: var(--blueprint);
    border: 1px solid var(--blueprint);
    transform: translateY(2px);
  }

  .edu-main {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .edu-school {
    font-family: var(--font-display);
    font-size: 1.5rem;
    text-transform: uppercase;
    letter-spacing: 0.02em;
    color: var(--ink);
    line-height: 1.05;
  }

  .edu-degree {
    font-family: var(--font-body);
    font-size: 0.95rem;
    color: var(--ink-soft);
    line-height: 1.4;
  }

  .edu-period {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.06em;
    color: var(--ink);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    text-align: right;
  }

  .edu-location {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.06em;
    color: var(--ink-mute);
    text-transform: uppercase;
    white-space: nowrap;
    min-width: 140px;
    text-align: right;
  }

  @media (max-width: 768px) {
    .edu-row {
      grid-template-columns: 12px minmax(0, 1fr);
      grid-template-rows: auto auto auto;
      gap: 4px 16px;
      padding: 16px 0;
    }

    .edu-status {
      grid-row: 1;
    }

    .edu-main {
      grid-row: 1;
      grid-column: 2;
    }

    .edu-period {
      grid-row: 2;
      grid-column: 2;
      text-align: left;
    }

    .edu-location {
      grid-row: 3;
      grid-column: 2;
      text-align: left;
      min-width: 0;
    }

    .edu-school {
      font-size: 1.2rem;
    }
  }
</style>
