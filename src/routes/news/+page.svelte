<svelte:head>
  <title>News — Kashif</title>
  <meta name="description" content="Top Hacker News stories, pulled fresh." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { getApiBase } from '$lib/apiBase';

  export const API_BASE = getApiBase();

  interface Story {
    by: string;
    id: number;
    score: number;
    time: number;
    title: string;
    type: string;
    url: string;
  }

  let stories: Story[] = [];
  let isLoading = true;
  let error = '';

  onMount(async () => {
    const url = new URL(window.location.href);
    if (url.searchParams.has('api')) {
      url.searchParams.delete('api');
      history.replaceState(history.state, '', url.toString());
    }

    try {
      const res = await fetch(`${API_BASE}/api/hn`);
      if (!res.ok) throw new Error('Bad response');
      stories = await res.json();
    } catch (e) {
      console.error(e);
      error = 'Failed to fetch Hacker News stories.';
    } finally {
      isLoading = false;
    }
  });

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
</script>

<div class="max-w-3xl mx-auto">
  <!-- Title -->
  <section class="mb-12" aria-labelledby="news-title">
    <h1 id="news-title" class="text-3xl font-bold tracking-tight text-[var(--color-headline)]">Hacker News</h1>
    <p class="mt-2 text-lg text-[var(--color-paragraph)]">A slice of what’s buzzing in tech right now.</p>
  </section>

  <!-- Content -->
  {#if isLoading}
    <div class="space-y-4">
      {#each Array(6) as _}
        <div class="animate-pulse p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]">
          <div class="h-5 w-3/4 bg-[var(--color-border)] rounded mb-2"></div>
          <div class="h-4 w-1/3 bg-[var(--color-border)] rounded"></div>
        </div>
      {/each}
    </div>
  {:else if error}
    <div class="p-4 rounded-xl bg-red-900/10 border border-red-900/20 text-red-400 text-sm">{error}</div>
  {:else if stories.length === 0}
    <div class="text-[var(--color-paragraph)]">No stories found.</div>
  {:else}
    <div class="space-y-4">
      {#each stories as story (story.id)}
        <article class="p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] hover:border-[var(--color-secondary)] transition-colors">
          <h2 class="text-lg font-medium text-[var(--color-headline)] mb-2">
            <a href={story.url} target="_blank" rel="noopener noreferrer" class="hover:text-[var(--color-highlight)] transition-colors">
              {story.title}
            </a>
          </h2>
          <div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-[var(--color-secondary)]">
            <span>{story.score} points by {story.by}</span>
            <span>•</span>
            <span>{formatDate(story.time)}</span>
            <span>•</span>
            <a
              href={`https://news.ycombinator.com/item?id=${story.id}`}
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-[var(--color-headline)] hover:underline"
            >
              Discuss ↗
            </a>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</div>
