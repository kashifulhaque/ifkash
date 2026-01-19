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
      role: "ML Engineer",
      company: "Wand AI",
      url: "https://wand.ai",
      period: "2025 — Present",
      location: "Palo Alto, CA",
    },
    {
      role: "Engineer III",
      company: "American Express",
      url: "https://www.americanexpress.com",
      period: "2025",
      location: "Bangalore, India",
    },
    {
      role: "Software Engineer",
      company: "Fiery (Epson)",
      url: "https://www.fiery.com",
      period: "2023 — 2025",
      location: "Bangalore, India",
    },
    {
      role: "Intern",
      company: "Corteva Agriscience",
      url: "https://www.corteva.in",
      period: "2022",
      location: "Hyderabad, India",
    },
  ];
</script>

<svelte:head>
  <title>Work — Kashif</title>
  <meta name="description" content="Professional experience and roles." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <span class="category-badge">Experience</span>
    <h1 class="page-title">Work</h1>
    <p class="page-desc">Building ML systems and shipping code.</p>
  </header>

  <section class="jobs">
    {#each jobs as job, i}
      <a
        href={job.url}
        target="_blank"
        rel="noopener noreferrer"
        class="job"
        style="--delay: {i * 50}ms"
      >
        <div class="job-main">
          <span class="job-role">{job.role}</span>
          <span class="job-company">{job.company}</span>
        </div>
        <div class="job-meta">
          <span class="job-period">{job.period}</span>
          <span class="job-location">{job.location}</span>
        </div>
        <span class="job-arrow">→</span>
      </a>
    {/each}
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 3rem;
  }

  .page-header {
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--color-border);
  }

  .page-title {
    font-family: var(--font-serif);
    font-size: clamp(2.5rem, 6vw, 3.5rem);
    font-weight: 400;
    letter-spacing: -0.02em;
    color: var(--white);
    margin-bottom: 0.75rem;
  }

  .page-desc {
    font-size: 1rem;
    color: var(--gray-500);
  }

  .jobs {
    display: flex;
    flex-direction: column;
  }

  .job {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.5rem;
    padding: 1.5rem 0;
    border-bottom: 1px solid var(--color-border);
    animation: fade-up var(--duration-slow) var(--ease-out) backwards;
    animation-delay: var(--delay);
    transition: all var(--duration-fast) var(--ease-out);
    position: relative;
  }

  @media (min-width: 640px) {
    .job {
      grid-template-columns: 1fr auto auto;
      gap: 2rem;
      align-items: baseline;
    }
  }

  .job:hover {
    padding-left: 1rem;
    background: var(--glass-bg);
    margin-left: -1rem;
    margin-right: -1rem;
    padding-right: 2.5rem;
  }

  .job:first-child {
    border-top: 1px solid var(--color-border);
  }

  .job-main {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .job-role {
    font-size: 1rem;
    font-weight: 500;
    color: var(--white);
  }

  .job-company {
    font-size: 0.875rem;
    color: var(--gray-500);
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
    color: var(--gray-400);
    font-variant-numeric: tabular-nums;
  }

  .job-location {
    font-size: 0.75rem;
    color: var(--gray-600);
  }

  .job-arrow {
    display: none;
    position: absolute;
    right: 1rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--gray-600);
    opacity: 0;
    transition: all var(--duration-fast) var(--ease-out);
  }

  @media (min-width: 640px) {
    .job-arrow {
      display: block;
    }
  }

  .job:hover .job-arrow {
    opacity: 1;
    color: var(--white);
    transform: translateY(-50%) translateX(4px);
  }

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
