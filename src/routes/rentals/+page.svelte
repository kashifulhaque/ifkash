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
    images: string[];
    rent?: string | null;
    deposit?: string | null;
    location?: string | null;
    landmark?: string | null;
    comments_summary?: string | null;
    // internal UI state
    _activeIndex?: number;
    _loaded?: boolean;
  };

  let isLoading = true;
  let error = '';
  let posts: RentalPost[] = [];

  onMount(async () => {
    try {
      const res = await fetch(`${API_BASE}/api/rentals/latest`);
      if (!res.ok) throw new Error(`Bad response: ${res.status}`);
      const payload = await res.json();
      // expected shape: { posts: RentalPost[] }
      posts = (payload.posts ?? []).map((p: RentalPost) => ({ ...p, _activeIndex: 0, _loaded: false, images: p.images ?? [] }));
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
      // reddit uses seconds
      const d = new Date((ts ?? 0) * 1000);
      return d.toLocaleString('en-GB', { year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    } catch {
      return '';
    }
  }

  function placeholder() { return '/images/placeholder-3bhk.png'; }

  function currentImage(p: RentalPost) {
    if (p.images && p.images.length > 0) {
      const idx = Math.max(0, Math.min((p._activeIndex ?? 0), p.images.length - 1));
      return p.images[idx];
    }
    return placeholder();
  }

  function prev(p: RentalPost) {
    if (!p.images || p.images.length <= 1) return;
    p._activeIndex = ((p._activeIndex ?? 0) - 1 + p.images.length) % p.images.length;
  }
  function next(p: RentalPost) {
    if (!p.images || p.images.length <= 1) return;
    p._activeIndex = ((p._activeIndex ?? 0) + 1) % p.images.length;
  }

  function setIndex(p: RentalPost, i: number) {
    p._activeIndex = i;
  }

  function onImgLoad(p: RentalPost) {
    p._loaded = true;
  }
</script>

<div class="min-h-screen selection:text-white" style="background-color: var(--color-background); color: var(--color-paragraph);">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur" style="backdrop-filter: blur(12px); background-color: rgba(22,22,26,0.6);">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight" style="color: var(--color-headline);">ifkash.dev</a>
          <a href="/" class="rounded-full px-3 py-1 text-sm hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">Home</a>
        </nav>
      </div>
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
    </header>

    <section class="pt-14 sm:pt-20" aria-labelledby="rentals-title">
      <h1 id="rentals-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight text-neutral-100">
        Latest full 3BHK rentals — r/bangalorerentals
      </h1>
      <p class="mt-2 text-neutral-400">Showing posts from the last two weeks that look like whole 3BHKs for rent.</p>
    </section>

    {#if isLoading}
      <div class="mt-8 grid gap-3">
        {#each Array(6) as _}
          <div class="animate-pulse rounded-xl border border-neutral-900 bg-neutral-900/40 p-4">
            <div class="h-40 w-full rounded bg-neutral-800/60"></div>
            <div class="mt-3 h-5 w-1/2 rounded bg-neutral-800/60"></div>
            <div class="mt-2 h-4 w-1/3 rounded bg-neutral-800/60"></div>
          </div>
        {/each}
      </div>
    {:else if error}
      <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-red-300">{error}</div>
    {:else if posts.length === 0}
      <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-neutral-300">No matching 3BHK rentals found in the last two weeks.</div>
    {:else}
      <ol class="mt-8 space-y-5">
        {#each posts as post (post.id)}
          <li>
            <article class="group rounded-2xl border border-neutral-800 bg-neutral-900/60 p-4 hover:border-neutral-700 hover:bg-neutral-900 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">
              <div class="grid grid-cols-3 gap-4">
                <div class="col-span-1">
                  <div class="relative rounded-lg overflow-hidden border border-neutral-800">
                    <a href={post.url} target="_blank" rel="noopener noreferrer" class="block">
                      <img
                        src={currentImage(post)}
                        alt={post.title}
                        loading="lazy"
                        class="h-32 w-full object-cover rounded-lg border-none transition-opacity duration-250"
                        on:error={(e) => (e.currentTarget.src = placeholder())}
                        on:load={() => onImgLoad(post)}
                        style="opacity: {post._loaded ? 1 : 0.01};"
                      />
                    </a>

                    {#if post.images && post.images.length > 1}
                      <button
                        class="absolute left-1 top-1/2 -translate-y-1/2 bg-black/30 hover:bg-black/45 p-1 rounded-full focus:outline-none"
                        on:click={() => prev(post)}
                        aria-label="previous image"
                      >‹</button>
                      <button
                        class="absolute right-1 top-1/2 -translate-y-1/2 bg-black/30 hover:bg-black/45 p-1 rounded-full focus:outline-none"
                        on:click={() => next(post)}
                        aria-label="next image"
                      >›</button>

                      <div class="absolute bottom-1 left-1 right-1 flex justify-center gap-1 px-1">
                        {#each post.images.slice(0, 6) as thumb, i}
                          <button
                            class="w-6 h-6 rounded overflow-hidden border-2"
                            on:click={() => setIndex(post, i)}
                            aria-label="select image"
                            style="border-color: {post._activeIndex === i ? 'white' : 'transparent'}"
                          >
                            <img src={thumb} alt="thumb" class="w-full h-full object-cover" on:error={(e)=>e.currentTarget.src=placeholder()} />
                          </button>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>

                <div class="col-span-2">
                  <h2 class="text-lg font-medium text-neutral-100 group-hover:underline">
                    <a href={post.url} target="_blank" rel="noopener noreferrer">{post.title}</a>
                  </h2>

                  <p class="mt-1 text-sm text-neutral-400">{post.selftext ? (post.selftext.length > 220 ? post.selftext.slice(0, 220) + '…' : post.selftext) : ''}</p>

                  <div class="mt-3 flex flex-wrap gap-2 text-sm">
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
                    <details class="mt-3 text-sm text-neutral-400">
                      <summary class="cursor-pointer">Comments summary</summary>
                      <div class="mt-2 prose max-w-none text-neutral-300 whitespace-pre-wrap">{post.comments_summary}</div>
                    </details>
                  {/if}

                  <p class="mt-3 text-xs text-neutral-500">
                    Posted: {formatDate(post.created_utc)} ·
                    <a class="ml-1 hover:underline text-neutral-300" href={post.url} target="_blank" rel="noopener noreferrer">Open on Reddit ↗</a>
                  </p>
                </div>
              </div>
            </article>
          </li>
        {/each}
      </ol>
    {/if}

    <footer class="mt-16">
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm text-neutral-500">
        <a class="hover:text-neutral-300" href="/">Back to home</a>
        <div class="text-neutral-600">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>

<style>
  /* keep small, specific tweaks here — main look handled by your global styles */
  .prose { color: var(--color-paragraph); }
  /* small styling to keep carousel thumbnails tidy */
  button[aria-label="select image"] { background: transparent; border-color: transparent; }
  button[aria-label="select image"] img { display: block; }
</style>
