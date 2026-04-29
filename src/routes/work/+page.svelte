<svelte:head>
  <title>Work — Kashiful Haque</title>
  <meta name="description" content="Roles shipping ML systems, inference engines, and production software." />
</svelte:head>

<script lang="ts">
  type Job = {
    role: string;
    company: string;
    url: string;
    period: string;
    location: string;
    note: string;
    tag: string;
  };

  const jobs: Job[] = [
    {
      role: 'ML Engineer',
      company: 'Wand AI',
      url: 'https://wand.ai',
      period: '2025 — Present',
      location: 'Palo Alto, CA',
      tag: 'agents · platform',
      note: 'Building agent infrastructure for enterprise software workflows. Multi-agent orchestration, governance, and the surfaces engineers actually use.'
    },
    {
      role: 'Engineer III',
      company: 'American Express',
      url: 'https://www.americanexpress.com',
      period: '2025',
      location: 'Bangalore, India',
      tag: 'platform · backend',
      note: 'High-volume backend services in regulated, audited environments. Performance-critical paths under strict latency budgets.'
    },
    {
      role: 'Software Engineer',
      company: 'Fiery (Epson)',
      url: 'https://www.fiery.com',
      period: '2023 — 2025',
      location: 'Bangalore, India',
      tag: 'systems · imaging',
      note: 'Imaging pipelines and color systems for production print servers. C++ / Python interop and tooling for hardware-adjacent teams.'
    },
    {
      role: 'Intern',
      company: 'Corteva Agriscience',
      url: 'https://www.corteva.in',
      period: '2022',
      location: 'Hyderabad, India',
      tag: 'data · research',
      note: 'Applied data science for agronomy research. Forecasting and anomaly detection on field-trial datasets.'
    }
  ];
</script>

<div class="page">
  <header class="page-header">
    <span class="eyebrow">section · 01 · work</span>
    <h1 class="page-title">
      Roles where I<br />
      <em>moved the number.</em>
    </h1>
    <p class="page-desc">
      Production teams, regulated environments, and frontier infrastructure.
      Shipping at the speed pressure demands.
    </p>
  </header>

  <section class="jobs">
    {#each jobs as job, i}
      <article class="job stagger" style="--i: {i}">
        <a
          href={job.url}
          target="_blank"
          rel="noopener noreferrer"
          class="job-link"
        >
          <div class="job-num">[ 0{i + 1} ]</div>

          <div class="job-main">
            <div class="job-head">
              <h2 class="job-role">{job.role}</h2>
              <span class="job-arrow">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M7 17L17 7M9 7h8v8"/></svg>
              </span>
            </div>
            <div class="job-company-row">
              <span class="job-company">{job.company}</span>
              <span class="job-tag">{job.tag}</span>
            </div>
            <p class="job-note">{job.note}</p>
          </div>

          <div class="job-meta">
            <span class="job-period">{job.period}</span>
            <span class="job-location">{job.location}</span>
          </div>
        </a>
      </article>
    {/each}
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

  .jobs {
    display: flex;
    flex-direction: column;
  }

  .job {
    border-bottom: 1px solid var(--border);
  }

  .job:first-child {
    border-top: 1px solid var(--border);
  }

  .job-link {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-areas:
      "num main"
      "num meta";
    gap: 0.85rem 1.5rem;
    padding: 1.75rem 0;
    color: inherit;
    transition: padding var(--dur-fast);
  }

  @media (min-width: 720px) {
    .job-link {
      grid-template-columns: auto 1fr auto;
      grid-template-areas: "num main meta";
      gap: 1.5rem 2rem;
      padding: 1.75rem 1.25rem;
      margin-left: -1.25rem;
      margin-right: -1.25rem;
      border-radius: var(--radius-md);
    }
    .job-link:hover {
      background: var(--surface-mesh);
    }
  }

  .job-num {
    grid-area: num;
    font-family: var(--font-mono);
    font-size: 0.6875rem;
    color: var(--text-faint);
    letter-spacing: 0.06em;
    padding-top: 0.5rem;
  }

  .job-link:hover .job-num {
    color: var(--accent);
  }

  .job-main {
    grid-area: main;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 0;
  }

  .job-head {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    justify-content: space-between;
  }

  @media (min-width: 720px) {
    .job-head {
      justify-content: flex-start;
    }
  }

  .job-role {
    font-size: 1.375rem;
    font-weight: 500;
    letter-spacing: -0.025em;
    color: var(--text-primary);
    transition: color var(--dur-fast);
  }

  .job-link:hover .job-role {
    color: var(--accent);
  }

  .job-arrow {
    color: var(--text-faint);
    transition: color var(--dur-fast), transform var(--dur-fast);
  }

  @media (min-width: 720px) {
    .job-arrow {
      display: none;
    }
  }

  .job-link:hover .job-arrow {
    color: var(--accent);
    transform: translate(2px, -2px);
  }

  .job-company-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem;
  }

  .job-company {
    font-size: 0.9375rem;
    color: var(--text-secondary);
  }

  .job-tag {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--text-faint);
    padding: 0.2rem 0.55rem;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: 999px;
  }

  .job-note {
    font-size: 0.9375rem;
    line-height: 1.55;
    color: var(--text-tertiary);
    max-width: 60ch;
  }

  .job-meta {
    grid-area: meta;
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.5rem 1rem;
  }

  @media (min-width: 720px) {
    .job-meta {
      flex-direction: column;
      align-items: flex-end;
      gap: 0.25rem;
      text-align: right;
      padding-top: 0.45rem;
    }
  }

  .job-period {
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.005em;
  }

  .job-location {
    font-size: 0.75rem;
    color: var(--text-faint);
  }
</style>
