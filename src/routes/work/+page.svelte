<svelte:head>
  <title>Work — Kashif</title>
  <meta name="description" content="Professional experience and roles." />
</svelte:head>

<script lang="ts">
  import { reveal, splitReveal } from '$lib/motion';
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

<header class="page-head">
  <p class="eyebrow">01 / Work</p>
  <h1 class="page-title" use:splitReveal>Work</h1>
  <p class="lead">Roles and companies</p>
</header>

<ol class="timeline" use:reveal={{ children: '.stop', gap: 90 }}>
  {#each jobs as job, i}
    <li class="stop" class:current={job.status === 'current'}>
      <a href={job.url} target="_blank" rel="noopener noreferrer" class="job">
        <span class="job-period">{job.period}</span>
        <span class="job-node" aria-hidden="true">
          <span class="led" class:led-hollow={job.status !== 'current'}></span>
        </span>
        <span class="job-main">
          <span class="job-role">{job.role}</span>
          <span class="job-meta">
            <span class="job-company">{job.company}</span>
            <span class="job-sep" aria-hidden="true">·</span>
            <span class="job-location">{job.location}</span>
          </span>
        </span>
        <span class="job-arrow" aria-hidden="true">↗</span>
      </a>
    </li>
  {/each}
</ol>

<style>
  .timeline {
    position: relative;
    margin: 0;
    padding: var(--s-7) 0 var(--s-4);
    list-style: none;
  }

  .stop {
    position: relative;
  }

  .job {
    display: grid;
    grid-template-columns: 136px 24px minmax(0, 1fr) 20px;
    align-items: baseline;
    column-gap: var(--s-5);
    padding: var(--s-5) 0 var(--s-6);
    color: var(--ink);
  }

  /* The rail: one continuous hairline drawn per stop so it joins up. */
  .job-node {
    position: relative;
    align-self: stretch;
    display: flex;
    justify-content: center;
  }
  .job-node::before {
    content: '';
    position: absolute;
    top: 0;
    bottom: calc(-1 * var(--s-6));
    left: 50%;
    width: 1px;
    background: var(--line);
    transform: translateX(-0.5px);
  }
  .stop:first-child .job-node::before {
    top: 16px;
  }
  .stop:last-child .job-node::before {
    bottom: auto;
    height: 16px;
  }
  .job-node .led {
    position: relative;
    z-index: 1;
    margin-top: 13px; /* centred on the role's first line */
    /* A ring of page colour so the rail breaks around each marker. */
    outline: 4px solid var(--bg);
  }
  .job-node .led-hollow {
    background: var(--bg);
  }

  .job-period {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    line-height: 1.4;
    white-space: nowrap;
    padding-top: 0.45em;
  }
  .current .job-period {
    color: var(--moss);
  }

  .job-main {
    display: grid;
    gap: var(--s-2);
    min-width: 0;
  }

  .job-role {
    color: var(--ink);
    font-size: var(--text-xl);
    font-weight: 500;
    line-height: 1.15;
    letter-spacing: -0.025em;
    transition: color var(--dur-fast) var(--ease);
  }

  .job-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--s-2);
    color: var(--ink-2);
    font-size: var(--text-base);
  }
  .job-sep,
  .job-location {
    color: var(--ink-3);
  }
  .job-location {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .job-arrow {
    justify-self: end;
    color: var(--ink-3);
    font-size: var(--text-sm);
    transition:
      transform var(--dur-fast) var(--ease),
      color var(--dur-fast) var(--ease);
  }

  .job:hover .job-role,
  .job:hover .job-arrow {
    color: var(--fjord);
  }
  .job:hover .job-arrow {
    transform: translate(3px, -3px);
  }

  @media (max-width: 720px) {
    .timeline {
      padding-top: var(--s-6);
    }

    .job {
      grid-template-columns: 16px minmax(0, 1fr) 16px;
      grid-template-areas:
        'node period arrow'
        'node main   main';
      column-gap: var(--s-4);
      row-gap: var(--s-2);
      padding: var(--s-4) 0 var(--s-5);
    }
    .job-node {
      grid-area: node;
      grid-row: 1 / span 2;
    }
    .job-node::before {
      bottom: calc(-1 * var(--s-5));
    }
    .stop:first-child .job-node::before {
      top: 9px;
    }
    .stop:last-child .job-node::before {
      height: 9px;
    }
    .job-node .led {
      margin-top: 6px; /* centred on the period line */
    }
    .job-period {
      grid-area: period;
      padding-top: 0;
    }
    .job-main {
      grid-area: main;
    }
    .job-arrow {
      grid-area: arrow;
    }
    .job-role {
      font-size: var(--text-lg);
    }
  }
</style>
