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
    _activeIndex?: number;
    _loaded?: boolean;
  };

  let isLoading = true;
  let error = '';
  let posts: RentalPost[] = [];

  // guaranteed-to-load inline SVG fallback (prevents infinite broken-src attempts)
  const svgFallback = `data:image/svg+xml;utf8,` +
    encodeURIComponent(
      `<svg xmlns='http://www.w3.org/2000/svg' width='800' height='450' viewBox='0 0 800 450'>
         <rect width='100%' height='100%' fill='#0b0b0b'/>
         <g fill='#6b6b6b' font-family='Arial, Helvetica, sans-serif' font-size='18'>
           <text x='50%' y='48%' dominant-baseline='middle' text-anchor='middle'>no image</text>
           <text x='50%' y='58%' dominant-baseline='middle' text-anchor='middle' font-size='12' fill='#4b4b4b'>—</text>
         </g>
       </svg>`
    );

  // basic heuristics for identifying real image urls
  function isImageUrl(u: string | undefined | null) {
    if (!u || typeof u !== 'string') return false;
    const s = u.replace(/&amp;/g, '&').split('#')[0];
    // common image exts
    if (/\.(jpe?g|png|webp|gif|bmp|avif|svg)(\?.*)?$/i.test(s)) return true;
    return false;
  }

  // if url contains reddit preview id but without extension (eg v.redd.it/...), try to construct a preview variant
  function tryRedditPreview(u: string) {
    // many preview URLs live at external-preview.redd.it/<id>.png?format=pjpg&auto=webp&s=<hash>
    // heuristic: if u contains 'v.redd.it' or '/gallery/' try to find a preview candidate by replacing host
    try {
      const parsed = new URL(u, 'https://reddit.com');
      const p = parsed.pathname;
      // if looks like v.redd.it/<id> produce external-preview.redd.it/<id>.jpg?format=pjpg&auto=webp
      const vMatch = p.match(/\/([A-Za-z0-9_-]{6,})($|\/)/);
      if (vMatch) {
        const id = vMatch[1];
        return `https://external-preview.redd.it/${id}.jpg?format=pjpg&auto=webp`;
      }
    } catch {
      // noop
    }
    return null;
  }

  function sanitize(u: string) {
    if (!u) return u;
    // decode simple entities, remove odd amp fragments
    const s = u.replace(/&amp;/g, '&');
    try { return decodeURIComponent(s); } catch { return s; }
  }

  // normalize the raw images array from KV into a small, sane list for the carousel
  function normalizeImages(raw: string[] | undefined | null) {
    if (!raw || !Array.isArray(raw)) return [];
    // keep an ordered preference:
    // 1) external-preview.redd.it / preview.redd.it variants with format params
    // 2) urls with image extensions
    // 3) try to construct reddit preview from v.redd.it
    const seen = new Set<string>();
    const out: string[] = [];

    // helper that pushes unique sanitized url
    function push(u?: string | null) {
      if (!u) return;
      const s = sanitize(u);
      if (!s) return;
      if (seen.has(s)) return;
      seen.add(s);
      out.push(s);
    }

    // first pass: prefer external-preview / preview variants
    for (const r of raw) {
      const s = sanitize(r);
      if (!s) continue;
      if (/external-preview\.redd\.it|preview\.redd\.it/i.test(s) && isImageUrl(s)) push(s);
      // sometimes preview links lack format param; add it
      if (/external-preview\.redd\.it|preview\.redd\.it/i.test(s) && !/\.(jpe?g|png|webp)/i.test(s)) {
        // append common preview params
        push(s + (s.includes('?') ? '&' : '?') + 'format=pjpg&auto=webp');
      }
    }

    // second pass: any explicit image ext urls
    for (const r of raw) {
      const s = sanitize(r);
      if (!s) continue;
      if (isImageUrl(s)) push(s);
    }

    // third pass: attempt heuristic preview construction for v.redd.it/gallery-like entries
    for (const r of raw) {
      const s = sanitize(r);
      if (!s) continue;
      if (/v\.redd\.it|reddit\.com\/gallery/i.test(s)) {
        const preview = tryRedditPreview(s);
        if (preview) push(preview);
      }
    }

    // final fallback: attempt any b.thumbs or redditmedia thumbs
    for (const r of raw) {
      const s = sanitize(r);
      if (!s) continue;
      if (/redditmedia\.com|thumbs\.redditmedia|b\.thumbs|a\.thumbs/i.test(s) && isImageUrl(s)) push(s);
    }

    return out.slice(0, 12); // cap thumbnails
  }

  onMount(async () => {
    try {
      const res = await fetch(`${API_BASE}/api/rentals/latest`);
      if (!res.ok) throw new Error(`Bad response: ${res.status}`);
      const payload = await res.json();
      posts = (payload.posts ?? []).map((p: RentalPost) => {
        const imgs = normalizeImages(p.images);
        return { ...p, images: imgs, _activeIndex: 0, _loaded: false };
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

  // UI helpers
  function currentImage(p: RentalPost) {
    if (p.images && p.images.length > 0) {
      const idx = Math.max(0, Math.min((p._activeIndex ?? 0), p.images.length - 1));
      return p.images[idx];
    }
    return svgFallback;
  }

  function prev(p: RentalPost) {
    if (!p.images || p.images.length <= 1) return;
    p._loaded = false;
    p._activeIndex = ((p._activeIndex ?? 0) - 1 + p.images.length) % p.images.length;
  }
  function next(p: RentalPost) {
    if (!p.images || p.images.length <= 1) return;
    p._loaded = false;
    p._activeIndex = ((p._activeIndex ?? 0) + 1) % p.images.length;
  }
  function setIndex(p: RentalPost, i: number) {
    p._loaded = false;
    p._activeIndex = i;
  }

  function handleImgLoad(p: RentalPost) {
    p._loaded = true;
  }

  // one-time safe fallback to avoid infinite onerror loops
  function handleImageError(e: Event) {
    const img = e.currentTarget as HTMLImageElement;
    if (!img) return;
    if (img.dataset.fallback === '1') {
      img.onerror = null;
      img.src = svgFallback;
      return;
    }
    img.dataset.fallback = '1';
    img.onerror = null;
    img.src = svgFallback;
  }

  // keyboard nav support for focused radials — optional nicety
  function onKeyNav(e: KeyboardEvent, post: RentalPost) {
    if (e.key === 'ArrowLeft') prev(post);
    if (e.key === 'ArrowRight') next(post);
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
            <div class="h-48 w-full rounded bg-neutral-800/60"></div>
            <div class="mt-3 h-6 w-1/2 rounded bg-neutral-800/60"></div>
            <div class="mt-2 h-4 w-1/3 rounded bg-neutral-800/60"></div>
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
              <div class="grid grid-cols-3 gap-6">
                <!-- LEFT: big carousel area -->
                <div class="col-span-1">
                  <div
                    class="relative rounded-lg overflow-hidden border border-neutral-800"
                    tabindex="0"
                    on:keydown={(e) => onKeyNav(e, post)}
                  >
                    <!-- Main image with smooth fade-in -->
                    <img
                      src={currentImage(post)}
                      alt={post.title}
                      loading="lazy"
                      class="w-full h-48 sm:h-56 object-cover transition-opacity duration-300"
                      style="opacity: {post._loaded ? 1 : 0.02};"
                      on:load={() => handleImgLoad(post)}
                      on:error={handleImageError}
                    />

                    <!-- visible arrows -->
                    {#if post.images && post.images.length > 1}
                      <button class="absolute left-2 top-1/2 -translate-y-1/2 bg-black/40 hover:bg-black/60 text-white p-2 rounded-full shadow" aria-label="previous" on:click={() => prev(post)}>‹</button>
                      <button class="absolute right-2 top-1/2 -translate-y-1/2 bg-black/40 hover:bg-black/60 text-white p-2 rounded-full shadow" aria-label="next" on:click={() => next(post)}>›</button>

                      <!-- index counter -->
                      <div class="absolute left-2 bottom-2 bg-neutral-900/70 px-2 py-0.5 rounded text-xs text-neutral-200">
                        {post._activeIndex + 1}/{post.images.length}
                      </div>
                    {/if}

                    <!-- thumbnail rail (hide when there's no thumbnails) -->
                    {#if post.images && post.images.length > 1}
                      <div class="absolute bottom-2 left-1/2 -translate-x-1/2 flex items-center gap-2">
                        {#each post.images.slice(0, 6) as thumb, i}
                          <button class="w-8 h-8 rounded overflow-hidden border-2" on:click={() => setIndex(post, i)} style="border-color: {post._activeIndex === i ? 'white' : 'transparent'}" aria-label={"thumb " + (i+1)}>
                            <img src={thumb} alt={"thumb " + (i+1)} class="w-full h-full object-cover" on:error={handleImageError} />
                          </button>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>

                <!-- RIGHT: text and meta -->
                <div class="col-span-2">
                  <h2 class="text-lg font-medium text-neutral-100 group-hover:underline">
                    <a href={post.url} target="_blank" rel="noopener noreferrer">{post.title}</a>
                  </h2>

                  <p class="mt-2 text-sm text-neutral-400">
                    {post.selftext ? (post.selftext.length > 280 ? post.selftext.slice(0, 280) + '…' : post.selftext) : ''}
                  </p>

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
  /* ensure thumbnail buttons look tight */
  button[aria-label^="thumb"] { padding: 0; background: transparent; border-color: transparent; }
  button[aria-label^="thumb"] img { display:block; width:100%; height:100%; object-fit:cover; }
  /* accessible focus */
  article:focus { outline: 2px solid rgba(255,255,255,0.06); }
</style>
