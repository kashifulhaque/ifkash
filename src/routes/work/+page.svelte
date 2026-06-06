<svelte:head>
  <title>Work — Kashif</title>
  <meta name="description" content="Professional experience and roles." />
</svelte:head>

<script lang="ts">
  type Job = {
    role: string;
    company: string;
    url: string;
    period: string;
    location: string;
    status: 'current' | 'past';
  };

  const jobs: Job[] = [
    {
      role: 'AI/ML Engineer',
      company: 'Wand AI',
      url: 'https://wand.ai',
      period: '2025 — Present',
      location: 'Bangalore, India',
      status: 'current'
    },
    {
      role: 'Software Engineer I',
      company: 'American Express',
      url: 'https://www.americanexpress.com',
      period: '2025',
      location: 'Bangalore, India',
      status: 'past'
    },
    {
      role: 'Associate Software Engineer',
      company: 'Fiery (Epson)',
      url: 'https://www.fiery.com',
      period: '2023 — 2025',
      location: 'Bangalore, India',
      status: 'past'
    },
    {
      role: 'Intern',
      company: 'Corteva Agriscience',
      url: 'https://www.corteva.in',
      period: '2022',
      location: 'Hyderabad, India',
      status: 'past'
    }
  ];
</script>

<header class="page-header">
  <h1 class="section-title">Work.</h1>
</header>

<section class="jobs-list">
  {#each jobs as job, i}
    <a
      href={job.url}
      target="_blank"
      rel="noopener noreferrer"
      class="job-row stagger"
      style="--i: {i}"
    >
      <span class="job-status" class:current={job.status === 'current'}></span>
      <div class="job-main">
        <span class="job-role">{job.role}</span>
        <span class="job-company">{job.company}</span>
      </div>
      <span class="job-period">{job.period}</span>
      <span class="job-location">{job.location}</span>
    </a>
  {/each}
</section>

<style>
  .page-header {
    padding-top: 16px;
    padding-bottom: 0;
  }

  .jobs-list {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--rule-soft);
    padding-top: 0;
  }

  .job-row {
    display: grid;
    grid-template-columns: 14px minmax(0, 1fr) auto auto;
    align-items: baseline;
    gap: 24px;
    padding: 18px 0;
    border-bottom: 1px solid var(--rule-soft);
    border-left: none !important;
    border-right: none !important;
    transition: background 0.15s, padding 0.15s;
    color: var(--ink);
  }

  .job-row:hover {
    background: var(--blueprint-tint);
    padding-left: 12px;
    padding-right: 12px;
    margin-left: -12px;
    margin-right: -12px;
    border-bottom-color: var(--rule-soft);
  }

  .job-status {
    width: 12px;
    height: 12px;
    border: 1px solid var(--ink-mute);
    background: transparent;
    transform: translateY(2px);
  }

  .job-status.current {
    background: var(--blueprint);
    border-color: var(--blueprint);
  }

  .job-main {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .job-role {
    font-family: var(--font-display);
    font-size: 1.4rem;
    text-transform: uppercase;
    letter-spacing: 0.02em;
    color: var(--ink);
    line-height: 1.05;
  }

  .job-row:hover .job-role {
    color: var(--blueprint);
  }

  .job-company {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-soft);
  }

  .job-period {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.06em;
    color: var(--ink);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    text-align: right;
  }

  .job-location {
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
    .job-row {
      grid-template-columns: 12px minmax(0, 1fr);
      grid-template-rows: auto auto;
      gap: 4px 16px;
      padding: 14px 0;
    }

    .job-status {
      grid-row: 1;
    }

    .job-main {
      grid-row: 1;
      grid-column: 2;
    }

    .job-period {
      grid-row: 2;
      grid-column: 2;
      text-align: left;
    }

    .job-location {
      display: none;
    }

    .job-row:hover {
      padding-left: 8px;
      margin-left: -8px;
      padding-right: 8px;
      margin-right: -8px;
    }
  }

  @media (max-width: 480px) {
    .job-role {
      font-size: 1.15rem;
    }
  }
</style>
