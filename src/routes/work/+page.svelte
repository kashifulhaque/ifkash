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
  };

  const jobs: Job[] = [
    {
      role: 'ML Engineer',
      company: 'Wand AI',
      url: 'https://wand.ai',
      period: '2025 — Present',
      location: 'Palo Alto, CA'
    },
    {
      role: 'Engineer III',
      company: 'American Express',
      url: 'https://www.americanexpress.com',
      period: '2025',
      location: 'Bangalore, India'
    },
    {
      role: 'Software Engineer',
      company: 'Fiery (Epson)',
      url: 'https://www.fiery.com',
      period: '2023 — 2025',
      location: 'Bangalore, India'
    },
    {
      role: 'Intern',
      company: 'Corteva Agriscience',
      url: 'https://www.corteva.in',
      period: '2022',
      location: 'Hyderabad, India'
    }
  ];
</script>

<div class="page">
  <header class="page-header">
    <h1 class="page-title">Work</h1>
    <p class="page-desc">Building ML systems and shipping code.</p>
  </header>

  <section class="jobs">
    {#each jobs as job, i}
      <a
        href={job.url}
        target="_blank"
        rel="noopener noreferrer"
        class="job stagger"
        style="--i: {i}"
      >
        <div class="job-main">
          <span class="job-role">{job.role}</span>
          <span class="job-company">{job.company}</span>
        </div>
        <div class="job-meta">
          <span class="job-period">{job.period}</span>
          <span class="job-location">{job.location}</span>
        </div>
      </a>
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

  .jobs {
    display: flex;
    flex-direction: column;
  }

  .job {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.375rem;
    padding: 1.25rem 0;
    border-bottom: 1px solid var(--border-subtle);
    transition: background var(--dur-instant) var(--ease-out-quart);
    border-radius: var(--radius-sm);
  }

  @media (min-width: 640px) {
    .job {
      grid-template-columns: 1fr auto;
      gap: 2rem;
      align-items: baseline;
    }
  }

  .job:hover {
    background: var(--surface-sunken);
    padding-left: 1rem;
    margin-left: -1rem;
    margin-right: -1rem;
    padding-right: 1rem;
  }

  .job:last-child {
    border-bottom: none;
  }

  .job-main {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .job-role {
    font-size: 1rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .job-company {
    font-size: 0.875rem;
    color: var(--text-tertiary);
  }

  .job-meta {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    text-align: left;
  }

  @media (min-width: 640px) {
    .job-meta {
      text-align: right;
    }
  }

  .job-period {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .job-location {
    font-size: 0.75rem;
    color: var(--text-faint);
  }
</style>
