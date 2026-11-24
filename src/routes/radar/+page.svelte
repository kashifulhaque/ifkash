<svelte:head>
  <title>Research radar — Kashif</title>
  <meta
    name="description"
    content="New papers on post-training, reinforcement learning, LLM inference, and agents, found daily and summarized in one line each."
  />
</svelte:head>

<script lang="ts">
  /* Research radar: papers the Worker's daily cron found with Exa
   * (api/src/handlers/radar.rs). The page only reads the stored list. */
  import { onMount } from 'svelte';
  import { splitReveal } from '$lib/motion';
  import LoadingState from '$lib/components/LoadingState.svelte';
  import { exaApi, type Radar } from '$lib/exaApi';

  let radar: Radar | null = null;
  let error = '';
  let topic = 'all';

  onMount(async () => {
    try {
      radar = await exaApi.radar();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Could not load the radar.';
    }
  });

  $: labels = new Map((radar?.topics ?? []).map((t) => [t.id, t.label]));
  $: counts = new Map(
    (radar?.topics ?? []).map((t) => [t.id, radar!.items.filter((i) => i.topic === t.id).length])
  );
  $: shown = (radar?.items ?? []).filter((i) => topic === 'all' || i.topic === topic);

  // Group by publication week, newest first.
  $: weeks = (() => {
    const groups: { key: string; label: string; items: typeof shown }[] = [];
    for (const item of shown) {
      const d = new Date(item.published + 'T00:00:00Z');
      if (Number.isNaN(d.getTime())) continue;
      const monday = new Date(d);
      monday.setUTCDate(d.getUTCDate() - ((d.getUTCDay() + 6) % 7));
      const key = monday.toISOString().slice(0, 10);
      let g = groups.find((x) => x.key === key);
      if (!g) {
        g = {
          key,
          label: `Week of ${monday.toLocaleDateString('en-GB', { day: 'numeric', month: 'long', timeZone: 'UTC' })}`,
          items: []
        };
        groups.push(g);
      }
      g.items.push(item);
    }
    return groups;
  })();

  const fmtDay = (iso: string) =>
    new Date(iso + 'T00:00:00Z').toLocaleDateString('en-GB', { day: 'numeric', month: 'short', timeZone: 'UTC' });

  /** First three authors, then "et al." */
  const shortAuthors = (a: string) => {
    const names = a
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s && s !== 'et al.');
    return names.length > 3 ? `${names.slice(0, 3).join(', ')}, et al.` : names.join(', ');
  };

  const ago = (secs: number) => {
    if (!secs) return 'not yet';
    const h = Math.round((Date.now() / 1000 - secs) / 3600);
    return h < 1 ? 'just now' : h < 24 ? `${h} h ago` : `${Math.round(h / 24)} d ago`;
  };

  const isNew = (found: number) => Date.now() / 1000 - found < 36 * 3600;
</script>

<header class="page-head">
  <p class="eyebrow">Research radar</p>
  <h1 class="page-title" use:splitReveal>What's new this week</h1>
  <p class="lead">
    Papers on the topics I work on, found every morning and summarized in one line. Search by
    <a class="prose-link" href="https://exa.ai" target="_blank" rel="noopener">Exa</a>.
  </p>
</header>

{#if error}
  <p class="state">{error}</p>
{:else if !radar}
  <LoadingState label="Reading the radar" />
{:else}
  <div class="filters" role="group" aria-label="Filter by topic">
    <button type="button" class:on={topic === 'all'} on:click={() => (topic = 'all')}>
      All <span>{radar.items.length}</span>
    </button>
    {#each radar.topics as t}
      <button type="button" class:on={topic === t.id} on:click={() => (topic = t.id)}>
        {t.label} <span>{counts.get(t.id) ?? 0}</span>
      </button>
    {/each}
    <span class="updated">Updated {ago(radar.updated)}</span>
  </div>

  {#if !shown.length}
    <p class="state">Nothing here yet. The radar checks for new papers every morning.</p>
  {/if}

  {#each weeks as week (week.key)}
    <section class="week">
      <h2 class="eyebrow">{week.label}</h2>
      <ol class="rows">
        {#each week.items as item (item.url)}
          <li>
            <a class="paper" href={item.url} target="_blank" rel="noopener">
              <span class="p-date">{fmtDay(item.published)}</span>
              <span class="p-body">
                <span class="p-title">{item.title}</span>
                {#if item.summary}<span class="p-summary">{item.summary}</span>{/if}
                <span class="p-meta">
                  <span class="tag">{labels.get(item.topic) ?? item.topic}</span>
                  {#if isNew(item.found)}<span class="led" aria-label="new"></span>{/if}
                  {#if item.authors}<span class="p-authors">{shortAuthors(item.authors)}</span>{/if}
                </span>
              </span>
              <span class="p-arrow" aria-hidden="true">↗</span>
            </a>
          </li>
        {/each}
      </ol>
    </section>
  {/each}
{/if}

<style>
  .state {
    margin: var(--s-7) 0;
    color: var(--ink-3);
  }

  .filters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2);
    margin: var(--s-6) 0 var(--s-2);
  }

  .filters button {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    height: 30px;
    padding: 0 var(--s-3);
    border: 1px solid var(--line);
    border-radius: var(--radius-pill);
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
  }

  .filters button span {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .filters button:hover {
    border-color: var(--line-strong);
    color: var(--ink);
  }

  .filters button.on {
    border-color: var(--fjord);
    background: var(--fjord-wash);
    color: var(--fjord);
  }

  .updated {
    margin-left: auto;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .week {
    margin-top: var(--s-7);
  }

  .week h2 {
    margin: 0 0 var(--s-3);
  }

  .paper {
    display: grid;
    grid-template-columns: 56px minmax(0, 1fr) 16px;
    align-items: baseline;
    column-gap: var(--s-5);
    padding: var(--s-5) 0;
    color: var(--ink);
    text-decoration: none;
  }

  .p-date {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .p-body {
    display: grid;
    /* An explicit column, or the one-line author list sets the track to its
       full width and pushes the row past a phone screen. */
    grid-template-columns: minmax(0, 1fr);
    gap: var(--s-2);
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .p-title {
    color: var(--ink);
    font-size: var(--text-md);
    font-weight: 500;
    line-height: 1.3;
    letter-spacing: -0.01em;
    transition: color var(--dur-fast) var(--ease);
  }

  .p-summary {
    max-width: var(--measure);
    color: var(--ink-2);
    line-height: 1.55;
  }

  .p-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2) var(--s-3);
  }

  .p-authors {
    overflow: hidden;
    color: var(--ink-3);
    font-size: var(--text-sm);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .p-arrow {
    color: var(--ink-3);
    transition: color var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  }

  .paper:hover .p-title,
  .paper:hover .p-arrow {
    color: var(--fjord);
  }

  .paper:hover .p-arrow {
    transform: translate(2px, -2px);
  }

  @media (max-width: 720px) {
    .paper {
      grid-template-columns: minmax(0, 1fr) 16px;
      row-gap: var(--s-2);
    }

    .p-date {
      grid-column: 1 / -1;
    }

    .updated {
      width: 100%;
      margin-left: 0;
    }
  }
</style>
