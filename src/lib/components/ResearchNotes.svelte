<script lang="ts">
  /* Research-backed suggestions for the insights page. The signals come from
   * `researchSuggestions()` (fitnessResearch.ts); each is asked in its own request,
   * so every note lines up with the signal that raised it. */
  import CitedAnswer from '$lib/components/CitedAnswer.svelte';
  import { exaApi, type ResearchNote } from '$lib/exaApi';
  import type { Suggestion } from '$lib/fitnessResearch';

  export let suggestions: Suggestion[] = [];

  let notes: (ResearchNote | null)[] = [];
  let loading = false;
  let lastKey = '';

  $: key = JSON.stringify(suggestions.map((s) => s.signal));
  $: if (key !== lastKey) void load(key);

  async function load(k: string) {
    lastKey = k;
    if (!suggestions.length) {
      notes = [];
      return;
    }
    loading = true;
    const res = await Promise.all(
      suggestions.map((s) =>
        exaApi
          .research([s.signal])
          .then((r) => r[0] ?? null)
          .catch(() => null)
      )
    );
    if (k === lastKey) {
      notes = res;
      loading = false;
    }
  }
</script>

{#if suggestions.length}
  <section class="research">
    <div class="research-head">
      <h2>Suggestions from research</h2>
      <span class="research-meta">via Exa</span>
    </div>
    <div class="research-body">
      {#each suggestions as s, i}
        <div class="note">
          <p class="why">{s.why}</p>
          {#if loading}
            <p class="status">Looking up the research…</p>
          {:else if notes[i]}
            <h4 class="q">{notes[i]?.question}</h4>
            <CitedAnswer answer={notes[i]?.answer ?? ''} sources={notes[i]?.sources ?? []} external />
          {:else}
            <p class="status">No research note for this one right now.</p>
          {/if}
        </div>
      {/each}
    </div>
  </section>
{/if}

<style>
  .research {
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }

  .research-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-4) var(--s-5);
    border-bottom: 1px solid var(--line);
  }

  .research-head h2 {
    margin: 0;
    font-size: var(--text-md);
    letter-spacing: -0.01em;
  }

  .research-meta {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .research-body {
    padding: 0 var(--s-5);
  }

  .note {
    padding: var(--s-5) 0;
    border-top: 1px solid var(--line);
  }

  .note:first-child {
    border-top: 0;
  }

  .why {
    margin: 0 0 var(--s-2);
    color: var(--clay);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .q {
    margin: 0 0 var(--s-2);
    color: var(--ink);
    font-size: var(--text-base);
    font-weight: 500;
  }

  .status {
    margin: 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
  }
</style>
