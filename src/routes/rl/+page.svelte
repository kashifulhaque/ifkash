<svelte:head>
  <title>RL — Kashif</title>
  <meta name="description" content="Reinforcement learning experiments." />
</svelte:head>

<script lang="ts">
  import { reveal, splitReveal } from '$lib/motion';
  type Experiment = {
    name: string;
    desc: string;
    links: { label: string; url: string }[];
    page?: string;
    status?: 'active' | 'shipped' | 'archived';
  };

  const experiments: Experiment[] = [
    {
      name: 'Snake RL',
      desc: 'A snake agent trained from scratch with Dueling Double DQN. PyTorch, 28-dim egocentric state, target network, 256-unit network. Greedy eval mean: 102.',
      page: '/rl/snake-dqn',
      status: 'active',
      links: [
        { label: 'Play', url: '/rl/snake-dqn/play' }
      ]
    }
  ];

  const statusLabel = { active: 'Active', shipped: 'Shipped', archived: 'Archived' } as const;
  const pad = (n: number) => String(n).padStart(2, '0');
</script>

<header class="page-head">
  <p class="eyebrow">03 / RL</p>
  <h1 class="page-title" use:splitReveal>RL</h1>
  <p class="lead">Reinforcement learning experiments</p>
</header>

<ol class="rows list" use:reveal={{ children: '.row', gap: 45 }}>
  {#each experiments as experiment, i}
    <li class="row" class:linked={!!experiment.page}>
      <span class="row-n">{pad(i + 1)}</span>

      <div class="row-body">
        {#if experiment.page}
          <a href={experiment.page} class="row-title">
            <span class="row-name">{experiment.name}</span>
            <span class="row-arrow" aria-hidden="true">→</span>
          </a>
        {:else}
          <span class="row-title">
            <span class="row-name">{experiment.name}</span>
          </span>
        {/if}
        <p class="row-desc">{experiment.desc}</p>
      </div>

      <div class="row-meta">
        {#if experiment.status}
          <span class="tag status">
            <span
              class="led"
              class:led-hollow={experiment.status === 'archived'}
              aria-hidden="true"
            ></span>
            {statusLabel[experiment.status]}
          </span>
        {/if}
        {#if experiment.links.length}
          <ul class="row-links">
            {#each experiment.links as link}
              <li>
                <a href={link.url} target="_blank" rel="noopener noreferrer" class="row-link">
                  {link.label}<span aria-hidden="true">↗</span>
                </a>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </li>
  {/each}
</ol>

<style>
  .list {
    margin-top: var(--s-7);
  }

  .row {
    position: relative;
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr) minmax(176px, auto);
    align-items: baseline;
    column-gap: var(--s-5);
    padding: var(--s-5) 0;
  }

  .row-n {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .row-body {
    display: grid;
    gap: var(--s-2);
    min-width: 0;
  }

  .row-title {
    display: inline-flex;
    align-items: baseline;
    gap: var(--s-2);
    justify-self: start;
    color: var(--ink);
    text-decoration: none;
  }
  /* Stretch the detail link over the whole row; the outbound links sit above it. */
  a.row-title::after {
    content: '';
    position: absolute;
    inset: 0;
  }

  .row-name {
    font-size: var(--text-lg);
    font-weight: 500;
    line-height: 1.25;
    letter-spacing: -0.015em;
    transition: color var(--dur-fast) var(--ease);
  }

  .row-arrow {
    color: var(--ink-3);
    font-size: var(--text-base);
    transition:
      transform var(--dur-fast) var(--ease),
      color var(--dur-fast) var(--ease);
  }

  .row.linked:hover .row-name,
  .row.linked:hover .row-arrow {
    color: var(--fjord);
  }
  .row.linked:hover .row-arrow {
    transform: translateX(3px);
  }

  .row-desc {
    max-width: var(--measure);
    margin: 0;
    color: var(--ink-2);
    font-size: var(--text-base);
    line-height: 1.55;
  }

  .row-meta {
    display: grid;
    justify-items: end;
    align-content: start;
    gap: var(--s-3);
  }

  .status {
    color: var(--ink-2);
  }
  .status .led {
    width: 6px;
    height: 6px;
  }

  .row-links {
    position: relative;
    z-index: 1;
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: var(--s-1) var(--s-4);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .row-link {
    display: inline-flex;
    align-items: baseline;
    gap: 3px;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    text-decoration: none;
    transition: color var(--dur-fast) var(--ease);
  }
  .row-link span {
    color: var(--ink-3);
    transition: color var(--dur-fast) var(--ease);
  }
  .row-link:hover,
  .row-link:hover span {
    color: var(--fjord);
  }

  @media (max-width: 720px) {
    .list {
      margin-top: var(--s-6);
    }

    .row {
      grid-template-columns: 32px minmax(0, 1fr);
      row-gap: var(--s-3);
      column-gap: var(--s-3);
    }

    .row-meta {
      grid-column: 2;
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-items: start;
      gap: var(--s-3) var(--s-4);
    }

    .row-links {
      justify-content: flex-start;
    }

    .row-name {
      font-size: var(--text-md);
    }
  }
</style>
