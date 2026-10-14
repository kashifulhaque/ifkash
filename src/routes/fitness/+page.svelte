<svelte:head>
  <title>Fitness — Kashif</title>
  <meta name="description" content="Private fitness hub — workout plan and training log." />
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<script lang="ts">
  import { Dumbbell, TrendingUp } from "lucide-svelte";

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
      desc: 'The four-day upper/lower split, the cardio rules and the week checklist — plus a log to track every session and progressive overload.',
      href: '/fitness/workout'
    },
    {
      name: 'Insights',
      icon: TrendingUp,
      desc: 'Is it working? The bodyweight trend, the day/week/month training report, every fitted rate and projection, and the full session history.',
      href: '/fitness/workout/insights'
    }
  ];
</script>

<header class="page-head">
  <p class="eyebrow"><span class="led" aria-hidden="true"></span>Private</p>
  <h1 class="section-title">Fitness</h1>
  <p class="section-subtitle">Training in one place. Sign in with Google to log your sessions.</p>
</header>

<section class="entries" aria-label="Fitness tools">
  <ol class="rows">
    {#each cards as card, i}
      <li class="stagger" style="--i: {i}">
        <a class="entry" href={card.href}>
          <span class="entry-index" aria-hidden="true">{String(i + 1).padStart(2, '0')}</span>
          <div class="entry-body">
            <span class="entry-name">
              <span class="entry-icon" aria-hidden="true">
                <svelte:component this={card.icon} size={16} strokeWidth={1.5} />
              </span>
              {card.name}
            </span>
            <p class="entry-desc">{card.desc}</p>
          </div>
          <span class="entry-arrow" aria-hidden="true">&rarr;</span>
        </a>
      </li>
    {/each}
  </ol>
</section>

<style>
  .page-head .eyebrow {
    margin: 0;
  }

  .entries {
    padding-top: var(--s-7);
  }

  .entries .rows {
    border-top-color: var(--ink);
  }

  .entry {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr) 24px;
    align-items: baseline;
    gap: var(--s-5);
    padding: var(--s-6) 0;
    color: var(--ink);
    transition: background-color var(--dur-fast) var(--ease);
  }

  .entry-index {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.02em;
    font-variant-numeric: tabular-nums;
  }

  .entry-name {
    display: inline-flex;
    align-items: center;
    gap: var(--s-3);
    color: var(--ink);
    font-size: var(--text-xl);
    font-weight: 500;
    line-height: 1.1;
    letter-spacing: -0.022em;
    transition: color var(--dur-fast) var(--ease);
  }

  .entry-icon {
    display: inline-grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    color: var(--ink-3);
    transition:
      color var(--dur-fast) var(--ease),
      border-color var(--dur-fast) var(--ease);
  }

  .entry-desc {
    max-width: var(--measure);
    margin: var(--s-3) 0 0;
    color: var(--ink-2);
    font-size: var(--text-base);
    line-height: 1.6;
  }

  .entry-arrow {
    justify-self: end;
    color: var(--ink-3);
    font-size: var(--text-md);
    transition:
      color var(--dur-fast) var(--ease),
      transform var(--dur-fast) var(--ease);
  }

  .entry:hover .entry-name,
  .entry:hover .entry-arrow {
    color: var(--fjord);
  }

  .entry:hover .entry-icon {
    border-color: var(--fjord);
    color: var(--fjord);
  }

  .entry:hover .entry-arrow {
    transform: translateX(3px);
  }

  @media (max-width: 640px) {
    .entry {
      grid-template-columns: 32px minmax(0, 1fr);
      gap: var(--s-3);
      padding: var(--s-5) 0;
    }

    .entry-name {
      font-size: var(--text-lg);
    }

    .entry-arrow {
      display: none;
    }
  }
</style>
