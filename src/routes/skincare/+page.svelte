<script lang="ts">
  type MorningStep = {
    step: number;
    product: string;
  };

  type NightEntry = {
    day: string;
    treatment: string;
  };

  const morningSteps: MorningStep[] = [
    { step: 1, product: "Rice Water cleanser" },
    { step: 2, product: "Minimalist Vitamin C serum" },
    { step: 3, product: "Deconstruct SPF 55+" },
  ];

  const nightRoutine: NightEntry[] = [
    { day: "Mon", treatment: "Retinol cream" },
    { day: "Tue", treatment: "Niacinamide" },
    { day: "Wed", treatment: "AHA" },
    { day: "Thu", treatment: "Niacinamide" },
    { day: "Fri", treatment: "Retinol cream" },
    { day: "Sat", treatment: "Niacinamide" },
    { day: "Sun", treatment: "AHA" },
  ];

  // Tag badge color mapping for treatments
  const tagMap: Record<string, string> = {
    "Retinol cream": "retinol",
    Niacinamide: "niacinamide",
    AHA: "aha",
  };

  // Get today's day abbreviation
  const days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  const today = days[new Date().getDay()];
</script>

<svelte:head>
  <title>Skincare — Kashif</title>
  <meta
    name="description"
    content="Personal skincare routine — morning and night."
  />
</svelte:head>

<div class="page">
  <header class="page-header">
    <h1 class="page-title">Skincare</h1>
  </header>

  <!-- Morning Section -->
  <section class="routine-section">
    <div class="section-label">
      <span class="section-icon">☀</span>
      <h2 class="section-title">
        Morning <span class="freq-badge">daily</span>
      </h2>
    </div>

    <ol class="morning-list" aria-label="Morning skincare steps">
      {#each morningSteps as item, i}
        <li class="morning-item" style="--delay: {i * 60}ms">
          <span class="step-num">{item.step}</span>
          <span class="step-product">{item.product}</span>
        </li>
      {/each}
    </ol>
  </section>

  <!-- Night Section -->
  <section class="routine-section">
    <div class="section-label">
      <span class="section-icon">☽</span>
      <h2 class="section-title">Night</h2>
    </div>

    <p class="cleanse-note">Always cleanse before starting night routine.</p>

    <div class="night-table" role="table" aria-label="Night routine by day">
      <div class="table-header" role="row">
        <span role="columnheader">Day</span>
        <span role="columnheader">Treatment</span>
      </div>
      {#each nightRoutine as entry, i}
        <div
          class="table-row"
          class:today={entry.day === today}
          role="row"
          style="--delay: {i * 50}ms"
        >
          <span class="day-cell" role="cell">
            {entry.day}
            {#if entry.day === today}
              <span class="today-dot" aria-label="today"></span>
            {/if}
          </span>
          <span class="treatment-cell" role="cell">
            <span class="tag tag--{tagMap[entry.treatment]}"
              >{entry.treatment}</span
            >
          </span>
        </div>
      {/each}
    </div>
  </section>

  <!-- Notes Section -->
  <section class="notes-section">
    <p class="body-note">
      Nivea lotion on body, Niacinamide on arms specifically.
    </p>
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 3rem;
  }

  /* ─── Header ─── */
  .page-header {
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--gray-800);
  }

  .page-title {
    font-size: 2.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--white);
    margin-bottom: 0.5rem;
  }

  .page-desc {
    font-size: 1rem;
    color: var(--gray-500);
  }

  /* ─── Section ─── */
  .routine-section {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .section-label {
    display: flex;
    align-items: center;
    gap: 0.625rem;
  }

  .section-icon {
    font-size: 1rem;
    color: var(--gray-500);
    line-height: 1;
  }

  .section-title {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--white);
    letter-spacing: -0.01em;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 0;
  }

  .freq-badge {
    font-size: 0.6875rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--gray-600);
    border: 1px solid var(--gray-800);
    border-radius: 999px;
    padding: 0.125rem 0.5rem;
    vertical-align: middle;
  }

  /* ─── Morning list ─── */
  .morning-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
  }

  .morning-item {
    display: flex;
    align-items: baseline;
    gap: 1.25rem;
    padding: 1rem 0;
    border-bottom: 1px solid var(--gray-900);
    animation: fade-up var(--duration-slow) var(--ease-out) backwards;
    animation-delay: var(--delay);
    transition: all var(--duration-fast) var(--ease-out);
  }

  .morning-item:last-child {
    border-bottom: none;
  }

  .morning-item:hover {
    padding-left: 1rem;
    background: var(--glass-bg);
    margin-left: -1rem;
    margin-right: -1rem;
    padding-right: 1rem;
  }

  .step-num {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--gray-700);
    min-width: 1.25rem;
    font-variant-numeric: tabular-nums;
  }

  .step-product {
    font-size: 1rem;
    font-weight: 400;
    color: var(--gray-200);
  }

  /* ─── Cleanse note ─── */
  .cleanse-note {
    font-size: 0.875rem;
    color: var(--gray-600);
    margin: 0;
  }

  /* ─── Night table ─── */
  .night-table {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .table-header {
    display: grid;
    grid-template-columns: 6rem 1fr;
    padding: 0.625rem 1.25rem;
    background: var(--gray-950);
    border-bottom: 1px solid var(--gray-800);
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--gray-600);
  }

  .table-row {
    display: grid;
    grid-template-columns: 6rem 1fr;
    align-items: center;
    padding: 0.875rem 1.25rem;
    border-bottom: 1px solid var(--gray-900);
    animation: fade-up var(--duration-slow) var(--ease-out) backwards;
    animation-delay: var(--delay);
    transition: background var(--duration-fast) var(--ease-out);
  }

  .table-row:last-child {
    border-bottom: none;
  }

  .table-row:hover {
    background: var(--glass-bg);
  }

  .table-row.today {
    background: rgba(255, 255, 255, 0.03);
  }

  .day-cell {
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--gray-400);
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .table-row.today .day-cell {
    color: var(--white);
  }

  .today-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--white);
    display: inline-block;
    flex-shrink: 0;
  }

  .treatment-cell {
    font-size: 0.9375rem;
  }

  /* ─── Treatment tags ─── */
  .tag {
    display: inline-block;
    font-size: 0.8125rem;
    font-weight: 500;
    padding: 0.2rem 0.625rem;
    border-radius: 999px;
    border: 1px solid transparent;
  }

  .tag--retinol {
    color: var(--gray-300);
    background: rgba(255, 255, 255, 0.05);
    border-color: var(--gray-700);
  }

  .tag--niacinamide {
    color: var(--gray-300);
    background: rgba(255, 255, 255, 0.05);
    border-color: var(--gray-700);
  }

  .tag--aha {
    color: var(--gray-300);
    background: rgba(255, 255, 255, 0.05);
    border-color: var(--gray-700);
  }

  /* Subtle differentiation on hover row to distinguish treatments */
  .table-row.today .tag {
    border-color: var(--gray-500);
    color: var(--white);
  }

  /* ─── Notes section ─── */
  .notes-section {
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--gray-900);
  }

  .body-note {
    font-size: 0.9375rem;
    color: var(--gray-500);
    line-height: 1.65;
    margin: 0;
  }

  .body-note.tip {
    color: var(--gray-400);
  }

  /* ─── Animations ─── */
  @keyframes fade-up {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
