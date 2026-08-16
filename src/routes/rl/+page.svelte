<svelte:head>
  <title>RL — Kashif</title>
  <meta name="description" content="Reinforcement learning experiments." />
</svelte:head>

<script lang="ts">
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
      desc: 'A snake agent trained from scratch with DQN. PyTorch, 11-dim state, experience replay, 256-unit network. Record: 44.',
      page: '/rl/snake-dqn',
      status: 'active',
      links: [
        { label: 'Play', url: '/rl/snake-dqn/play' }
      ]
    }
  ];
</script>

<header class="page-header">
  <h1 class="section-title">RL</h1>
  <p class="section-subtitle">Reinforcement learning experiments</p>
</header>

<section class="experiments-list">
  {#each experiments as experiment, i}
    <article class="experiment-row stagger" style="--i: {i}">
      <span
        class="experiment-led"
        class:lit={experiment.status === 'active'}
        class:shipped={experiment.status === 'shipped'}
        class:archived={experiment.status === 'archived'}
        aria-hidden="true"
      ></span>
      <div class="experiment-body">
        <div class="experiment-top">
          {#if experiment.page}
            <a href={experiment.page} class="experiment-name-link">
              <span class="experiment-name">{experiment.name}</span>
            </a>
          {:else}
            <span class="experiment-name">{experiment.name}</span>
          {/if}
          <div class="experiment-links">
            {#each experiment.links as link}
              <a
                href={link.url}
                target="_blank"
                rel="noopener noreferrer"
                class="experiment-link"
              >
                {link.label}
              </a>
            {/each}
          </div>
        </div>
        <p class="experiment-desc">{experiment.desc}</p>
      </div>
    </article>
  {/each}
</section>

<style>
  .page-header {
    padding-top: 16px;
  }

  .experiments-list {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--line-soft);
  }

  .experiment-row {
    display: grid;
    grid-template-columns: 14px minmax(0, 1fr);
    align-items: start;
    gap: 20px;
    padding: 22px 12px;
    margin: 0 -12px;
    border-bottom: 1px solid var(--line-soft);
    transition: background 0.15s;
  }

  .experiment-row:hover {
    background: var(--blueprint-tint);
  }

  .experiment-led {
    width: 6px;
    height: 6px;
    border-radius: 0;
    border: 1px solid var(--ink-mute);
    background: transparent;
    transform: translateY(10px);
    justify-self: center;
    flex-shrink: 0;
  }

  .experiment-led.lit {
    background: var(--accent);
    border-color: var(--accent);
    box-shadow: none;
    animation: none;
  }

  .experiment-led.shipped {
    background: var(--ok);
    border-color: var(--ok);
    box-shadow: none;
  }

  .experiment-led.archived {
    background: transparent;
    border-style: dashed;
    border-color: var(--ink-mute);
  }

  .experiment-body {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
  }

  .experiment-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .experiment-name {
    font-family: var(--font-serif);
    font-size: 1.5rem;
    font-weight: 400;
    letter-spacing: -0.025em;
    text-transform: none;
    color: var(--foreground);
    line-height: 1.2;
  }

  .experiment-name-link {
    border-bottom: none;
  }

  .experiment-name-link:hover .experiment-name {
    color: var(--accent-2);
  }

  .experiment-links {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .experiment-link {
    font-family: var(--font-label);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    padding: 5px 11px;
    background: transparent;
    color: var(--muted-foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    text-decoration: none;
    transition:
      color 0.15s var(--ease-inout),
      border-color 0.15s var(--ease-inout);
  }

  .experiment-link:hover {
    color: var(--foreground);
    background: transparent;
    border-color: var(--border-strong);
  }

  .experiment-desc {
    font-family: var(--font-sans);
    font-size: 0.95rem;
    line-height: 1.6;
    color: var(--muted-foreground);
    max-width: 720px;
  }

  @media (max-width: 768px) {
    .experiment-row {
      gap: 16px;
      padding: 18px 10px;
      margin: 0 -10px;
    }

    .experiment-name {
      font-size: 1.5rem;
    }

    .experiment-top {
      flex-direction: column;
      align-items: flex-start;
      gap: 10px;
    }
  }
</style>
