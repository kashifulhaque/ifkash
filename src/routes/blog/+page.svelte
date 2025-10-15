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

<div class="min-h-screen bg-neutral-950 text-neutral-200 selection:bg-neutral-800 selection:text-white">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header / Crumb -->
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
    <section class="pt-14 sm:pt-20" aria-labelledby="blog-title">
      <h1 id="blog-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight text-neutral-100">Blog</h1>
      <p class="mt-3 text-neutral-400 max-w-2xl">Notes from the build log, occasional essays, and rough cuts worth sharing.</p>
    </section>

    <!-- States -->
    {#if loading}
      <div class="mt-8 grid gap-4">
        {#each Array(4) as _, i}
          <div class="animate-pulse rounded-2xl border border-neutral-900 bg-neutral-900/40 p-5">
            <div class="h-40 w-full rounded-xl bg-neutral-800/60"></div>
            <div class="mt-4 h-5 w-2/3 rounded bg-neutral-800/60"></div>
            <div class="mt-3 h-4 w-full rounded bg-neutral-800/60"></div>
            <div class="mt-2 h-4 w-5/6 rounded bg-neutral-800/60"></div>
          </div>
        {/each}
      </div>
    {:else if error}
      <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-red-300">{error}</div>
    {:else if posts.length === 0}
      <div class="mt-8 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-neutral-300">No posts yet. Soon.</div>
    {:else}
      <!-- Posts -->
      <section class="mt-8 grid gap-4">
        {#each posts as post}
          <a
            href={post.url}
            rel="noopener noreferrer"
            target="_blank"
            class="group block overflow-hidden rounded-2xl border border-neutral-800 bg-neutral-900/60 hover:border-neutral-700 hover:bg-neutral-900 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600"
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
                <div class="pointer-events-none absolute inset-x-0 bottom-0 h-20 bg-gradient-to-t from-neutral-900/80 to-transparent"></div>
              </div>
            {/if}
            <div class="p-5">
              <div class="flex items-center justify-between gap-3">
                <h2 class="text-lg font-medium text-neutral-100 group-hover:underline">{post.title}</h2>
                <span class="shrink-0 text-xs text-neutral-500">{new Date(post.publishedAt).toLocaleString('en-US', options).replace(',','')}</span>
              </div>
              <p class="mt-2 line-clamp-3 text-sm text-neutral-400">{post.brief}</p>
              <div class="mt-3 inline-flex items-center gap-2 text-sm text-neutral-400">
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
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 text-sm text-neutral-500">
        <a class="hover:text-neutral-300" href="/">Back to home</a>
        <div class="text-neutral-600">© {new Date().getFullYear()} Kashif</div>
      </div>
    </footer>
  </div>
</div>
