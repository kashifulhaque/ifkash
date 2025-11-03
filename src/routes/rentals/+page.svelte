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

  // tiny inline svg fallback (guaranteed to load)
  const svgFallback = `data:image/svg+xml;utf8,` +
    encodeURIComponent(
      `<svg xmlns="http://www.w3.org/2000/svg" width="600" height="400" viewBox="0 0 600 400">
         <rect width="100%" height="100%" fill="#111"/>
         <text x="50%" y="50%" fill="#888" font-family="Arial, Helvetica, sans-serif" font-size="20" dominant-baseline="middle" text-anchor="middle">no image</text>
       </svg>`
    );

  function isImageUrl(u: string | undefined | null) {
    if (!u || typeof u !== 'string') return false;
    // sanitize a little
    const s = u.replace(/&amp;/g, '&');
    // allow common image extensions with optional query params
    return /\.(jpe?g|png|webp|gif|bmp|avif|svg)(\?.*)?$/i.test(s);
  }

  function sanitizeUrl(u: string) {
    try {
      return decodeURIComponent(u.replace(/&amp;/g, '&'));
    } catch { return u.replace(/&amp;/g, '&'); }
  }

  // take raw KV images array and produce a safe list for the carousel
  function normalizeImages(raw: string[] | undefined | null) {
    if (!raw || !Array.isArray(raw)) return [];
    // map+sanitize -> filter image-like -> dedupe -> limit
    const cleaned = raw
      .map(sanitizeUrl)
      .filter(Boolean)
      .filter(isImageUrl);
    const deduped = Array.from(new Set(cleaned));
    return deduped.slice(0, 12);
  }

  onMount(async () => {
    try {
      const res = await fetch(`${API_BASE}/api/rentals/latest`);
      if (!res.ok) throw new Error(`Bad response: ${res.status}`);
      const payload = await res.json();
      posts = (payload.posts ?? []).map((p: RentalPost) => {
        const imgs = normalizeImages(p.images);
        // if none found, attempt to pick any reddit thumb-like URL
        let finalImgs = imgs;
        if (finalImgs.length === 0 && Array.isArray(p.images) && p.images.length > 0) {
          const thumbs = p.images
            .map(sanitizeUrl)
            .filter(u => /thumbs\.redditmedia|b\.thumbs|a\.thumbs|preview\.redd\.it/i.test(u));
          finalImgs = Array.from(new Set(thumbs)).slice(0, 6);
        }
        return { ...p, images: finalImgs, _activeIndex: 0, _loaded: false };
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

  function placeholder() { return svgFallback; }

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
    p._loaded = false;
  }
  function next(p: RentalPost) {
    if (!p.images || p.images.length <= 1) return;
    p._activeIndex = ((p._activeIndex ?? 0) + 1) % p.images.length;
    p._loaded = false;
  }

  function setIndex(p: RentalPost, i: number) {
    p._activeIndex = i;
    p._loaded = false;
  }

  function onImgLoad(p: RentalPost) {
    p._loaded = true;
  }

  // make sure we don't get into infinite fallback loops;
  // this replaces broken src with a guaranteed-to-load svg (once)
  function handleImageError(e: Event) {
    const img = e.currentTarget as HTMLImageElement;
    if (!img) return;
    // if we've already tried fallback, set final safe svg and remove handler
    if (img.dataset.fallback === '1') {
      img.onerror = null;
      img.src = svgFallback;
      return;
    }
    img.dataset.fallback = '1';
    img.onerror = null; // avoid re-entrancy loop
    img.src = svgFallback;
  }
</script>

<!-- same UI template as before, but with safer img handlers -->
{#if isLoading}
  <!-- ...loading skeletons (omitted for brevity) ... -->
{:else if error}
  <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-red-300">{error}</div>
{:else if posts.length === 0}
  <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-neutral-300">No matching 3BHK rentals found in the last two weeks.</div>
{:else}
  <ol class="mt-8 space-y-5">
    {#each posts as post (post.id)}
      <li>
        <article class="group rounded-2xl border border-neutral-800 bg-neutral-900/60 p-4">
          <div class="grid grid-cols-3 gap-4">
            <div class="col-span-1">
              <div class="relative rounded-lg overflow-hidden border border-neutral-800">
                <a href={post.url} target="_blank" rel="noopener noreferrer" class="block">
                  <img
                    src={currentImage(post)}
                    alt={post.title}
                    loading="lazy"
                    class="h-32 w-full object-cover rounded-lg border-none transition-opacity duration-250"
                    on:error={handleImageError}
                    on:load={() => onImgLoad(post)}
                    style="opacity: {post._loaded ? 1 : 0.02};"
                  />
                </a>

                {#if post.images && post.images.length > 1}
                  <button class="absolute left-1 top-1/2 -translate-y-1/2 bg-black/30 p-1 rounded-full" on:click={() => prev(post)}>‹</button>
                  <button class="absolute right-1 top-1/2 -translate-y-1/2 bg-black/30 p-1 rounded-full" on:click={() => next(post)}>›</button>

                  <div class="absolute bottom-1 left-1 right-1 flex justify-center gap-1 px-1">
                    {#each post.images.slice(0, 6) as thumb, i}
                      <button class="w-6 h-6 rounded overflow-hidden border-2" on:click={() => setIndex(post, i)} style="border-color: {post._activeIndex === i ? 'white' : 'transparent'}">
                        <img src={thumb} alt="thumb" class="w-full h-full object-cover" on:error={handleImageError} />
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
            </div>

            <div class="col-span-2">
              <h2 class="text-lg font-medium text-neutral-100">
                <a href={post.url} target="_blank" rel="noopener noreferrer">{post.title}</a>
              </h2>
              <p class="mt-1 text-sm text-neutral-400">{post.selftext ? (post.selftext.length > 220 ? post.selftext.slice(0,220)+'…' : post.selftext) : ''}</p>

              <!-- meta badges -->
              <div class="mt-3 flex flex-wrap gap-2 text-sm">
                {#if post.rent}<div class="rounded px-2 py-1 border border-neutral-800 text-neutral-200">Rent: <span class="font-semibold">{post.rent}</span></div>{/if}
                {#if post.deposit}<div class="rounded px-2 py-1 border border-neutral-800 text-neutral-200">Deposit: <span class="font-semibold">{post.deposit}</span></div>{/if}
                {#if post.location}<div class="rounded px-2 py-1 border border-neutral-800 text-neutral-200">Location: <span class="font-semibold">{post.location}</span></div>{/if}
                {#if post.landmark}<div class="rounded px-2 py-1 border border-neutral-800 text-neutral-200">Landmark: <span class="font-semibold">{post.landmark}</span></div>{/if}
              </div>

              {#if post.comments_summary}
                <details class="mt-3 text-sm text-neutral-400">
                  <summary class="cursor-pointer">Comments summary</summary>
                  <div class="mt-2 prose max-w-none text-neutral-300 whitespace-pre-wrap">{post.comments_summary}</div>
                </details>
              {/if}

              <p class="mt-3 text-xs text-neutral-500">Posted: {formatDate(post.created_utc)} · <a class="ml-1 hover:underline text-neutral-300" href={post.url} target="_blank" rel="noopener noreferrer">Open on Reddit ↗</a></p>
            </div>
          </div>
        </article>
      </li>
    {/each}
  </ol>
{/if}
