<svelte:head>
  <title>News — Kashif</title>
  <meta name="description" content="Top Hacker News stories, pulled fresh." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';

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
    try {
      const res = await fetch('/api/hn');
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

<div class="min-h-screen bg-neutral-950 text-neutral-200 selection:bg-neutral-800 selection:text-white">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur supports-[backdrop-filter]:bg-neutral-950/60">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight text-neutral-100">ifkash.dev</a>
          <a href="/" class="rounded-full px-3 py-1 text-sm hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">Home</a>
        </nav>
      </div>
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
    </header>

    <!-- Title -->
    <section class="pt-14 sm:pt-20" aria-labelledby="news-title">
      <h1 id="news-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight text-neutral-100">
        Hacker News Top Stories
      </h1>
      <p class="mt-2 text-neutral-400">A slice of what’s buzzing in tech right now.</p>
    </section>

    <!-- Content -->
    {#if isLoading}
      <div class="mt-8 grid gap-3">
        {#each Array(8) as _}
          <div class="animate-pulse rounded-xl border border-neutral-900 bg-neutral-900/40 p-4">
            <div class="h-5 w-2/3 rounded bg-neutral-800/60"></div>
            <div class="mt-2 h-4 w-1/2 rounded bg-neutral-800/60"></div>
          </div>
        {/each}
      </div>
    {:else if error}
      <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-red-300">{error}</div>
    {:else if stories.length === 0}
      <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-neutral-300">No stories found.</div>
    {:else}
      <ol class="mt-8 space-y-5">
        {#each stories as story (story.id)}
          <li>
            <article class="group rounded-2xl border border-neutral-800 bg-neutral-900/60 p-5 hover:border-neutral-700 hover:bg-neutral-900 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">
              <h2 class="text-lg font-medium text-neutral-100 group-hover:underline">
                <a href={story.url} target="_blank" rel="noopener noreferrer">{story.title}</a>
              </h2>
              <p class="mt-1 text-sm text-neutral-400">
                {story.score} points by {story.by} · {formatDate(story.time)} ·
                <a
                  href={`https://news.ycombinator.com/item?id=${story.id}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="hover:underline text-neutral-300"
                >
                  HN discussion ↗
                </a>
              </p>
            </article>
          </li>
        {/each}
      </ol>
    {/if}

    <!-- Footer -->
    <footer class="mt-16">
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm text-neutral-500">
        <a class="hover:text-neutral-300" href="/">Back to home</a>
        <div class="text-neutral-600">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>
