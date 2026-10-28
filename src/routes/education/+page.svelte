<svelte:head>
  <title>Education — Kashif</title>
  <meta name="description" content="Academic background." />
</svelte:head>

<script lang="ts">
  import { reveal, splitReveal } from '$lib/motion';
</script>

<header class="page-head">
  <p class="eyebrow">05 / Education</p>
  <h1 class="page-title" use:splitReveal>Education</h1>
  <p class="lead">Academic background</p>
</header>

<ol class="rows list" use:reveal={{ children: '.row' }}>
  <li class="row">
    <span class="row-period">2020 — 2024</span>
    <div class="row-main">
      <h2 class="row-school">Indian Institute of Technology Madras</h2>
      <p class="row-degree">BS, Data Science and Applications</p>
    </div>
    <span class="row-location">Chennai, India</span>
  </li>
</ol>

<style>
  .list {
    margin-top: var(--s-7);
  }

  .row {
    display: grid;
    grid-template-columns: 136px minmax(0, 1fr) auto;
    align-items: baseline;
    column-gap: var(--s-5);
    padding: var(--s-6) 0;
  }

  .row-period,
  .row-location {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .row-main {
    display: grid;
    gap: var(--s-2);
    min-width: 0;
  }

  .row-school {
    font-size: var(--text-xl);
    line-height: 1.15;
    letter-spacing: -0.025em;
  }

  .row-degree {
    margin: 0;
    color: var(--ink-2);
    font-size: var(--text-base);
  }

  @media (max-width: 720px) {
    .list {
      margin-top: var(--s-6);
    }

    .row {
      grid-template-columns: minmax(0, 1fr) auto;
      grid-template-areas:
        'period location'
        'main   main';
      row-gap: var(--s-3);
      padding: var(--s-5) 0;
    }
    .row-period {
      grid-area: period;
    }
    .row-location {
      grid-area: location;
    }
    .row-main {
      grid-area: main;
    }
    .row-school {
      font-size: var(--text-lg);
    }
  }
</style>
