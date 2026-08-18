<svelte:head>
  <title>Fitness — Kashif</title>
  <meta name="description" content="Private fitness hub — workout plan and training log." />
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<script lang="ts">
  import { Dumbbell } from "lucide-svelte";

  type Card = {
    name: string;
    desc: string;
    href: string;
    icon: typeof Dumbbell;
  };

  const cards: Card[] = [
    {
      name: 'Workout',
      icon: Dumbbell,
      desc: 'The push/pull/legs split, the cardio rules and the numbers — plus a log to track every session and progressive overload.',
      href: '/fitness/workout'
    }
  ];
</script>

<header class="page-header">
  <h1 class="section-title">Fitness</h1>
  <p class="section-subtitle">Training in one place. Sign in with Google to log your sessions.</p>
</header>

<section class="cards">
  {#each cards as card, i}
    <a class="card stagger" style="--i: {i}" href={card.href}>
      <span class="card-icon" aria-hidden="true">
        <svelte:component this={card.icon} size={18} strokeWidth={1.8} />
      </span>
      <div class="card-body">
        <span class="card-name">{card.name}</span>
        <p class="card-desc">{card.desc}</p>
      </div>
      <span class="card-arrow" aria-hidden="true">&rarr;</span>
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
    margin-top: 8px;
  }

  .card {
    display: grid;
    grid-template-columns: 36px minmax(0, 1fr) 24px;
    align-items: center;
    gap: 20px;
    padding: 22px 12px;
    margin: 0 -12px;
    border-bottom: 1px solid var(--line-soft);
    color: var(--ink);
    background: transparent;
    transition:
      background 0.15s,
      color 0.15s;
  }

  .card:first-child {
    border-top: 1px solid var(--line-soft);
  }

  .card:hover {
    background: var(--ink);
    color: var(--void);
    border-bottom-color: var(--ink);
  }

  .card-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    color: var(--signal);
  }

  .card:hover .card-icon {
    border-color: var(--void);
    color: var(--void);
  }

  .card-name {
    font-family: var(--font-dots);
    font-size: 1.75rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: inherit;
    line-height: 1;
  }

  .card-desc {
    font-family: var(--font-sans-g);
    font-size: 0.92rem;
    line-height: 1.55;
    color: var(--ink-soft);
    margin: 8px 0 0;
    max-width: 640px;
  }

  .card:hover .card-desc {
    color: var(--void);
  }

  .card-arrow {
    font-family: var(--font-mono-g);
    font-size: 1rem;
    color: var(--ink-mute);
    transition:
      color 0.15s,
      transform 0.15s;
  }

  .card:hover .card-arrow {
    color: var(--void);
    transform: translateX(4px);
  }

  @media (max-width: 768px) {
    .card {
      grid-template-columns: 36px minmax(0, 1fr);
      padding: 18px 10px;
      margin: 0 -10px;
      gap: 14px;
    }

    .card-name {
      font-size: 1.4rem;
    }

    .card-arrow {
      display: none;
    }
  }
</style>
