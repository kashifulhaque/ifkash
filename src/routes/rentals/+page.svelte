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

<div class="max-w-3xl mx-auto">
  <!-- Title -->
  <section class="mb-12" aria-labelledby="rentals-title">
    <h1 id="rentals-title" class="text-3xl font-bold tracking-tight text-[var(--color-headline)]">Rentals</h1>
    <p class="mt-2 text-lg text-[var(--color-paragraph)]">Latest full 3BHK rentals from r/bangalorerentals (last 2 weeks).</p>
  </section>

  {#if isLoading}
    <div class="space-y-4">
      {#each Array(4) as _}
        <div class="animate-pulse p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]">
          <div class="h-6 w-3/4 bg-[var(--color-border)] rounded mb-3"></div>
          <div class="h-4 w-full bg-[var(--color-border)] rounded mb-2"></div>
          <div class="h-4 w-5/6 bg-[var(--color-border)] rounded"></div>
        </div>
      {/each}
    </div>
  {:else if error}
    <div class="p-4 rounded-xl bg-red-900/10 border border-red-900/20 text-red-400 text-sm">{error}</div>
  {:else if posts.length === 0}
    <div class="text-[var(--color-paragraph)]">No matching 3BHK rentals found in the last two weeks.</div>
  {:else}
    <div class="space-y-6">
      {#each posts as post (post.id)}
        <article class="p-5 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] hover:border-[var(--color-secondary)] transition-colors">
          <div class="space-y-4">
            <h2 class="text-lg font-medium text-[var(--color-headline)]">
              <a href={post.url} target="_blank" rel="noopener noreferrer" class="hover:text-[var(--color-highlight)] transition-colors">
                {post.title}
              </a>
            </h2>

            <p class="text-sm text-[var(--color-paragraph)] whitespace-pre-line">
              {post.selftext ? (post.selftext.length > 280 ? post.selftext.slice(0, 280) + '…' : post.selftext) : ''}
            </p>

            <div class="flex flex-wrap gap-2 text-xs font-mono">
              {#if post.rent}
                <div class="px-2 py-1 rounded bg-[var(--color-background)] border border-[var(--color-border)] text-[var(--color-secondary)]">Rent: <span class="text-[var(--color-headline)]">{post.rent}</span></div>
              {/if}
              {#if post.deposit}
                <div class="px-2 py-1 rounded bg-[var(--color-background)] border border-[var(--color-border)] text-[var(--color-secondary)]">Deposit: <span class="text-[var(--color-headline)]">{post.deposit}</span></div>
              {/if}
              {#if post.location}
                <div class="px-2 py-1 rounded bg-[var(--color-background)] border border-[var(--color-border)] text-[var(--color-secondary)]">Loc: <span class="text-[var(--color-headline)]">{post.location}</span></div>
              {/if}
              {#if post.landmark}
                <div class="px-2 py-1 rounded bg-[var(--color-background)] border border-[var(--color-border)] text-[var(--color-secondary)]">Landmark: <span class="text-[var(--color-headline)]">{post.landmark}</span></div>
              {/if}
            </div>

            {#if post.comments_summary}
              <details class="text-sm text-[var(--color-secondary)]">
                <summary class="cursor-pointer hover:text-[var(--color-paragraph)] transition-colors">Comments summary</summary>
                <div class="mt-2 prose prose-sm max-w-none text-[var(--color-paragraph)] bg-[var(--color-background)] p-3 rounded border border-[var(--color-border)]">
                  {post.comments_summary}
                </div>
              </details>
            {/if}

            <div class="text-xs text-[var(--color-secondary)] flex items-center justify-between border-t border-[var(--color-border)] pt-3">
              <span>Posted {formatDate(post.created_utc)}</span>
              <a class="hover:text-[var(--color-headline)] transition-colors" href={post.url} target="_blank" rel="noopener noreferrer">Open on Reddit ↗</a>
            </div>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</div>

<style>
  /* small styling tweaks */
  .prose { color: var(--color-paragraph); }
</style>
