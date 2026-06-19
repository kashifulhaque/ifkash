<svelte:head>
  <title>Fitness — Kashif</title>
  <meta name="description" content="Private fitness hub — workout plan, training log and meal tracker." />
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<script lang="ts">
  type Card = {
    name: string;
    desc: string;
    href: string;
    emoji: string;
  };

  const cards: Card[] = [
    {
      name: 'Workout',
      emoji: '🏋️',
      desc: 'The push/pull/legs split, the cardio rules and the numbers — plus a log to track every session and progressive overload.',
      href: '/fitness/workout'
    },
    {
      name: 'Meals',
      emoji: '🍽️',
      desc: 'Snap a photo of a meal and get an estimated nutrition breakdown — calories and macros, logged by day with running totals.',
      href: '/fitness/meals'
    }
  ];
</script>

<header class="page-header">
  <h1 class="section-title">Fitness.</h1>
  <p class="section-subtitle">Training and nutrition in one place. Sign in with Google on each.</p>
</header>

<section class="cards">
  {#each cards as card}
    <a class="card" href={card.href}>
      <span class="card-emoji" aria-hidden="true">{card.emoji}</span>
      <div class="card-body">
        <span class="card-name">{card.name}</span>
        <p class="card-desc">{card.desc}</p>
      </div>
      <span class="card-arrow" aria-hidden="true">→</span>
    </a>
  {/each}
</section>

<style>
  .page-header {
    padding-top: 16px;
  }

  .cards {
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin-top: 24px;
    max-width: 760px;
  }

  .card {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 20px;
    padding: 24px;
    border: 1px solid var(--rule-soft);
    border-bottom: 1px solid var(--rule-soft);
    background: transparent;
    transition: border-color 0.15s, background 0.15s;
  }

  .card:hover {
    border-color: var(--blueprint);
    background: var(--blueprint-tint);
  }

  .card-emoji {
    font-size: 1.8rem;
    line-height: 1;
  }

  .card-name {
    font-family: var(--font-display);
    font-size: 1.5rem;
    text-transform: uppercase;
    letter-spacing: 0.02em;
    color: var(--ink);
    line-height: 1.05;
  }

  .card:hover .card-name {
    color: var(--blueprint);
  }

  .card-desc {
    font-family: var(--font-body);
    font-size: 0.95rem;
    line-height: 1.55;
    color: var(--ink-soft);
    margin: 6px 0 0;
  }

  .card-arrow {
    font-family: var(--font-mono);
    font-size: 1.2rem;
    color: var(--ink-mute);
  }

  .card:hover .card-arrow {
    color: var(--blueprint);
  }

  @media (max-width: 768px) {
    .card {
      padding: 18px;
      gap: 14px;
    }

    .card-name {
      font-size: 1.25rem;
    }

    .card-arrow {
      display: none;
    }
  }
</style>
