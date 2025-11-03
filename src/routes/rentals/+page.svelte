<svelte:head>
  <title>3BHK Rentals — Kashif</title>
  <meta name="description" content="Latest full 3BHK rentals from r/bangalorerentals (last 2 weeks)." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { getApiBase } from '$lib/apiBase';
  const API_BASE = getApiBase();

  type RentalPost = {
    id: string;
    title: string;
    url: string;
    created_utc: number | string;
    selftext: string;
    rent?: string | null;
    deposit?: string | null;
    location?: string | null;
    landmark?: string | null;
    comments_summary?: string | null;
  };

  let isLoading = true;
  let error = '';
  let posts: RentalPost[] = [];

  onMount(async () => {
    try {
      const res = await fetch(`${API_BASE}/api/rentals/latest`);
      if (!res.ok) throw new Error(`Bad response: ${res.status}`);
      const payload = await res.json();
      posts = (payload.posts ?? []).map((p: any) => {
        // keep only fields we care about / avoid carrying large image arrays
        const {
          id, title, url, created_utc, selftext,
          rent = null, deposit = null, location = null, landmark = null, comments_summary = null
        } = p;
        return { id, title, url, created_utc, selftext, rent, deposit, location, landmark, comments_summary };
      });
    } catch (e) {
      console.error(e);
      error = 'Failed to fetch latest rentals.';
    } finally {
      isLoading = false;
    }
  });

  function formatDate(utc: number | string) {
    try {
      const ts = typeof utc === 'string' ? parseFloat(utc) : utc;
      const d = new Date((ts ?? 0) * 1000);
      return d.toLocaleString('en-GB', { year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    } catch {
      return '';
    }
  }
</script>

<div class="min-h-screen selection:text-white" style="background-color: var(--color-background); color: var(--color-paragraph);">
  <div class="mx-auto max-w-4xl px-5 sm:px-6 pb-24">
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur" style="backdrop-filter: blur(12px); background-color: rgba(22,22,26,0.6);">
      <div class="mx-auto max-w-4xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight" style="color: var(--color-headline);">ifkash.dev</a>
          <a href="/" class="rounded-full px-3 py-1 text-sm hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">Home</a>
        </nav>
      </div>
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
    </header>

    <section class="pt-10 sm:pt-14" aria-labelledby="rentals-title">
      <h1 id="rentals-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight text-neutral-100">
        Latest full 3BHK rentals — r/bangalorerentals
      </h1>
      <p class="mt-2 text-neutral-400">Showing posts from the last two weeks that look like whole 3BHKs for rent.</p>
    </section>

    {#if isLoading}
      <div class="mt-8 grid gap-3">
        {#each Array(4) as _}
          <div class="animate-pulse rounded-2xl border border-neutral-900 bg-neutral-900/40 p-4">
            <div class="h-6 w-3/4 rounded bg-neutral-800/60"></div>
            <div class="mt-3 h-4 w-full rounded bg-neutral-800/60"></div>
            <div class="mt-2 h-4 w-5/6 rounded bg-neutral-800/60"></div>
          </div>
        {/each}
      </div>
    {:else if error}
      <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-red-300">{error}</div>
    {:else if posts.length === 0}
      <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-neutral-300">No matching 3BHK rentals found in the last two weeks.</div>
    {:else}
      <ol class="mt-8 space-y-6">
        {#each posts as post (post.id)}
          <li>
            <article class="group rounded-2xl border border-neutral-800 bg-neutral-900/60 p-4 hover:border-neutral-700 hover:bg-neutral-900 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">
              <div class="space-y-3">
                <h2 class="text-lg font-medium text-neutral-100 group-hover:underline">
                  <a href={post.url} target="_blank" rel="noopener noreferrer">{post.title}</a>
                </h2>

                <p class="text-sm text-neutral-400">
                  {post.selftext ? (post.selftext.length > 280 ? post.selftext.slice(0, 280) + '…' : post.selftext) : ''}
                </p>

                <div class="flex flex-wrap gap-2 text-sm">
                  {#if post.rent}
                    <div class="rounded px-2 py-1 border border-neutral-800 text-neutral-200">Rent: <span class="font-semibold">{post.rent}</span></div>
                  {/if}
                  {#if post.deposit}
                    <div class="rounded px-2 py-1 border border-neutral-800 text-neutral-200">Deposit: <span class="font-semibold">{post.deposit}</span></div>
                  {/if}
                  {#if post.location}
                    <div class="rounded px-2 py-1 border border-neutral-800 text-neutral-200">Location: <span class="font-semibold">{post.location}</span></div>
                  {/if}
                  {#if post.landmark}
                    <div class="rounded px-2 py-1 border border-neutral-800 text-neutral-200">Landmark: <span class="font-semibold">{post.landmark}</span></div>
                  {/if}
                </div>

                {#if post.comments_summary}
                  <details class="mt-2 text-sm text-neutral-400">
                    <summary class="cursor-pointer">Comments summary</summary>
                    <div class="mt-2 prose max-w-none text-neutral-300 whitespace-pre-wrap">{post.comments_summary}</div>
                  </details>
                {/if}

                <p class="text-xs text-neutral-500">
                  Posted: {formatDate(post.created_utc)} ·
                  <a class="ml-1 hover:underline text-neutral-300" href={post.url} target="_blank" rel="noopener noreferrer">Open on Reddit ↗</a>
                </p>
              </div>
            </article>
          </li>
        {/each}
      </ol>
    {/if}

    <footer class="mt-14">
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm text-neutral-500">
        <a class="hover:text-neutral-300" href="/">Back to home</a>
        <div class="text-neutral-600">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>

<style>
  /* small styling tweaks */
  .prose { color: var(--color-paragraph); }
  article:focus { outline: 2px solid rgba(255,255,255,0.06); }
</style>
