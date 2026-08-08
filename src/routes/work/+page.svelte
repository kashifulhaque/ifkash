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
  <h1 class="section-title">Work</h1>
  <p class="section-subtitle">Roles and companies</p>
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
      <span
        class="job-led"
        class:lit={job.status === 'current'}
        aria-hidden="true"
      ></span>
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
    border-top: 1px solid var(--border);
    padding-top: 0;
  }

  .job-row {
    display: grid;
    grid-template-columns: 14px minmax(0, 1fr) auto auto;
    align-items: baseline;
    gap: 24px;
    padding: 20px 12px;
    margin: 0 -12px;
    border-bottom: 1px solid var(--border);
    transition: color 0.15s var(--ease-inout);
    color: var(--foreground);
  }

  .job-row:hover {
    color: var(--foreground);
  }

  .job-row:hover .job-role {
    color: var(--accent-2);
  }

  .job-led {
    width: 6px;
    height: 6px;
    border-radius: 0;
    border: 1px solid var(--ink-mute);
    background: transparent;
    transform: translateY(1px);
    justify-self: center;
  }

  .job-led.lit {
    background: var(--accent);
    border-color: var(--accent);
    box-shadow: none;
    animation: none;
  }

  .job-main {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .job-role {
    font-family: var(--font-serif);
    font-size: 1.5rem;
    font-weight: 400;
    letter-spacing: -0.025em;
    text-transform: none;
    color: inherit;
    line-height: 1.2;
  }

  .job-company {
    font-family: var(--font-label);
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }

  .job-row:hover .job-company {
    color: var(--muted-foreground);
  }

  .job-period {
    font-family: var(--font-label);
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted-foreground);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    text-align: right;
  }

  .job-row:hover .job-period {
    color: var(--muted-foreground);
  }

  .job-location {
    font-family: var(--font-label);
    font-size: 11px;
    letter-spacing: 0.1em;
    color: var(--ink-mute);
    text-transform: uppercase;
    white-space: nowrap;
    min-width: 140px;
    text-align: right;
  }

  .job-row:hover .job-location {
    color: var(--ink-mute);
  }

  @media (max-width: 768px) {
    .job-row {
      grid-template-columns: 12px minmax(0, 1fr);
      grid-template-rows: auto auto;
      gap: 6px 16px;
      padding: 16px 10px;
      margin: 0 -10px;
    }

    .job-led {
      grid-row: 1;
      transform: translateY(6px);
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
  }

  @media (max-width: 480px) {
    .job-role {
      font-size: 1.35rem;
    }
  }
</style>
