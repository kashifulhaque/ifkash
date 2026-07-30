<svelte:head>
  <title>Education — Kashif</title>
  <meta name="description" content="Academic background." />
</svelte:head>

<header class="page-header">
  <h1 class="section-title">Education</h1>
  <p class="section-subtitle">Academic background</p>
</header>

<section class="edu-list">
  <article class="edu-row">
    <span class="edu-led" aria-hidden="true"></span>
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
    border-top: 1px solid var(--line-soft);
  }

  .edu-row {
    display: grid;
    grid-template-columns: 14px minmax(0, 1fr) auto auto;
    align-items: baseline;
    gap: 24px;
    padding: 22px 12px;
    margin: 0 -12px;
    border-bottom: 1px solid var(--line-soft);
  }

  .edu-led {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--ok);
    border: 1px solid var(--ok);
    box-shadow: 0 0 5px rgba(61, 220, 132, 0.4);
    transform: translateY(1px);
    justify-self: center;
  }

  .edu-main {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .edu-school {
    font-family: var(--font-dots);
    font-size: 1.75rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--ink);
    line-height: 1;
  }

  .edu-degree {
    font-family: var(--font-mono-g);
    font-size: 0.72rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-soft);
    line-height: 1.4;
  }

  .edu-period {
    font-family: var(--font-mono-g);
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    color: var(--ink);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    text-align: right;
  }

  .edu-location {
    font-family: var(--font-mono-g);
    font-size: 0.66rem;
    letter-spacing: 0.1em;
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
      gap: 6px 16px;
      padding: 18px 10px;
      margin: 0 -10px;
    }

    .edu-led {
      grid-row: 1;
      transform: translateY(6px);
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
      font-size: 1.35rem;
    }
  }
</style>
