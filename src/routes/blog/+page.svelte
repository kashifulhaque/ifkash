<svelte:head>
  <title>Blog — Kashif</title>
  <meta name="description" content="Latest writing, notes, and experiments by Kashif." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';

  type Post = {
    id: string;
    title: string;
    brief: string;
    url: string;
    coverImage?: { url?: string } | null;
    publishedAt: string;
  };

  let posts: Post[] = [];
  let loading = true;
  let error = '';

  const options: Intl.DateTimeFormatOptions = {
    month: 'short',
    day: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: true,
    timeZone: 'UTC'
  };

  const query = `
    query {
      publication(host: "blog.ifkash.dev") {
        posts(first: 10) {
          edges {
            node {
              id
              title
              brief
              url
              coverImage { url }
              publishedAt
            }
          }
        }
      }
    }
  `;

  onMount(async () => {
    const ctrl = new AbortController();
    try {
      const res = await fetch('https://gql.hashnode.com/', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query }),
        signal: ctrl.signal
      });
      if (!res.ok) throw new Error('Bad response');
      const result = await res.json();
      const edges = result?.data?.publication?.posts?.edges ?? [];
      posts = edges.map((e: any) => e.node) as Post[];
    } catch (e) {
      error = 'Failed to load blog posts.';
      console.error(e);
    } finally {
      loading = false;
    }
    return () => ctrl.abort();
  });
</script>

<div class="min-h-screen selection:text-white" style="background-color: var(--color-background); color: var(--color-paragraph);">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header / Crumb -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur" style="backdrop-filter: blur(12px); background-color: rgba(22, 22, 26, 0.6);">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight" style="color: var(--color-headline);">ifkash.dev</a>
          <a href="/" class="rounded-full px-3 py-1 text-sm focus:outline-none transition-colors" style="color: var(--color-paragraph);" onmouseover="this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.backgroundColor='transparent'">Home</a>
        </nav>
      </div>
      <div class="h-px w-full" style="background: linear-gradient(to right, transparent, var(--color-secondary), transparent);"></div>
    </header>

    <!-- Title -->
    <section class="pt-14 sm:pt-20" aria-labelledby="blog-title">
      <h1 id="blog-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight" style="color: var(--color-headline);">Blog</h1>
      <p class="mt-3 max-w-2xl" style="color: var(--color-paragraph);">Notes from the build log, occasional essays, and rough cuts worth sharing.</p>
    </section>

    <!-- States -->
    {#if loading}
      <div class="mt-8 grid gap-4">
        {#each Array(4) as _, i}
          <div class="animate-pulse rounded-2xl border p-5" style="border-color: var(--color-secondary); background-color: rgba(114, 117, 126, 0.1);">
            <div class="h-40 w-full rounded-xl" style="background-color: rgba(114, 117, 126, 0.3);"></div>
            <div class="mt-4 h-5 w-2/3 rounded" style="background-color: rgba(114, 117, 126, 0.3);"></div>
            <div class="mt-3 h-4 w-full rounded" style="background-color: rgba(114, 117, 126, 0.3);"></div>
            <div class="mt-2 h-4 w-5/6 rounded" style="background-color: rgba(114, 117, 126, 0.3);"></div>
          </div>
        {/each}
      </div>
    {:else if error}
      <div class="mt-8 rounded-xl border p-4 text-sm" style="border-color: var(--color-secondary); background-color: rgba(114, 117, 126, 0.1); color: #fca5a5;">{error}</div>
    {:else if posts.length === 0}
      <div class="mt-8 rounded-xl border p-4 text-sm" style="border-color: var(--color-secondary); background-color: rgba(114, 117, 126, 0.1); color: var(--color-paragraph);">No posts yet. Soon.</div>
    {:else}
      <!-- Posts -->
      <section class="mt-8 grid gap-4">
        {#each posts as post}
          <a
            href={post.url}
            rel="noopener noreferrer"
            target="_blank"
            class="group block overflow-hidden rounded-2xl border focus:outline-none transition-colors" style="border-color: var(--color-secondary); background-color: rgba(114, 117, 126, 0.1);" onmouseover="this.style.borderColor='var(--color-highlight)'; this.style.backgroundColor='rgba(127, 90, 240, 0.1)'" onmouseout="this.style.borderColor='var(--color-secondary)'; this.style.backgroundColor='rgba(114, 117, 126, 0.1)'"
          >
            {#if post.coverImage?.url}
              <div class="relative">
                <img
                  src={post.coverImage.url}
                  alt="Cover image for {post.title}"
                  class="h-48 w-full object-cover"
                  loading="lazy"
                  decoding="async"
                />
                <div class="pointer-events-none absolute inset-x-0 bottom-0 h-20" style="background: linear-gradient(to top, rgba(114, 117, 126, 0.8), transparent);"></div>
              </div>
            {/if}
            <div class="p-5">
              <div class="flex items-center justify-between gap-3">
                <h2 class="text-lg font-medium group-hover:underline" style="color: var(--color-headline);">{post.title}</h2>
                <span class="shrink-0 text-xs" style="color: var(--color-secondary);">{new Date(post.publishedAt).toLocaleString('en-US', options).replace(',','')}</span>
              </div>
              <p class="mt-2 line-clamp-3 text-sm" style="color: var(--color-paragraph);">{post.brief}</p>
              <div class="mt-3 inline-flex items-center gap-2 text-sm" style="color: var(--color-paragraph);">
                <span>Read on Hashnode</span>
                <span aria-hidden="true" class="transition -translate-y-px group-hover:translate-x-0.5">→</span>
              </div>
            </div>
          </a>
        {/each}
      </section>
    {/if}

    <!-- Footer -->
    <footer class="mt-16">
      <div class="h-px w-full" style="background: linear-gradient(to right, transparent, var(--color-secondary), transparent);"></div>
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm" style="color: var(--color-secondary);">
        <a class="transition-colors" onmouseover="this.style.color='var(--color-paragraph)'" onmouseout="this.style.color='var(--color-secondary)'" href="/">Back to home</a>
        <div style="color: var(--color-secondary);">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>
