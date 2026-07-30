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
    border-top: 1px solid var(--line-soft);
    padding-top: 0;
  }

  .job-row {
    display: grid;
    grid-template-columns: 14px minmax(0, 1fr) auto auto;
    align-items: baseline;
    gap: 24px;
    padding: 20px 12px;
    margin: 0 -12px;
    border-bottom: 1px solid var(--line-soft);
    transition:
      background 0.15s,
      color 0.15s;
    color: var(--ink);
  }

  .job-row:hover {
    background: var(--ink);
    color: var(--void);
    border-bottom-color: var(--ink);
  }

  .job-led {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    border: 1px solid var(--ink-mute);
    background: transparent;
    transform: translateY(1px);
    justify-self: center;
  }

  .job-led.lit {
    background: var(--signal);
    border-color: var(--signal);
    box-shadow:
      0 0 6px var(--signal-glow),
      0 0 2px var(--signal);
    animation: led-pulse 2.4s ease-in-out infinite;
  }

  .job-main {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .job-role {
    font-family: var(--font-dots);
    font-size: 1.75rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: inherit;
    line-height: 1;
  }

  .job-company {
    font-family: var(--font-mono-g);
    font-size: 0.72rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-soft);
  }

  .job-row:hover .job-company {
    color: var(--void);
  }

  .job-period {
    font-family: var(--font-mono-g);
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    color: var(--ink);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    text-align: right;
  }

  .job-row:hover .job-period {
    color: var(--void);
  }

  .job-location {
    font-family: var(--font-mono-g);
    font-size: 0.66rem;
    letter-spacing: 0.1em;
    color: var(--ink-mute);
    text-transform: uppercase;
    white-space: nowrap;
    min-width: 140px;
    text-align: right;
  }

  .job-row:hover .job-location {
    color: var(--void);
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
