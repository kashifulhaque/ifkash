<script lang="ts">
  import { onMount } from "svelte";

  type Post = {
    id: string;
    title: string;
    brief: string;
    url: string;
    slug: string;
    coverImage?: { url?: string } | null;
    publishedAt: string;
  };

  let posts: Post[] = [];
  let loading = true;
  let error = "";

  const options: Intl.DateTimeFormatOptions = {
    month: "short",
    day: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: true,
    timeZone: "UTC",
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
              slug
              coverImage { url }
              publishedAt
            }
          }
        }
      }
    }
  `;

  onMount(() => {
    const ctrl = new AbortController();
    (async () => {
      try {
        const res = await fetch("https://gql.hashnode.com/", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ query }),
          signal: ctrl.signal,
        });
        if (!res.ok) throw new Error("Bad response");
        const result = await res.json();
        const edges = result?.data?.publication?.posts?.edges ?? [];
        posts = edges.map((e: any) => e.node) as Post[];
      } catch (e) {
        if ((e as Error).name !== "AbortError") {
          error = "Failed to load blog posts.";
          console.error(e);
        }
      } finally {
        loading = false;
      }
    })();
    return () => ctrl.abort();
  });
</script>

<svelte:head>
  <title>Blog — Kashif</title>
  <meta
    name="description"
    content="Latest writing, notes, and experiments by Kashif."
  />
</svelte:head>

<div class="max-w-3xl mx-auto">
  <!-- Title -->
  <section class="mb-12" aria-labelledby="blog-title">
    <h1
      id="blog-title"
      class="text-3xl font-bold tracking-tight text-[var(--color-headline)]"
    >
      Blog
    </h1>
    <p class="mt-2 text-lg text-[var(--color-paragraph)]">
      Notes from the build log, occasional essays, and rough cuts worth sharing.
    </p>
  </section>

  <!-- States -->
  {#if loading}
    <div class="space-y-6">
      {#each Array(3) as _, i}
        <div
          class="animate-pulse flex flex-col gap-4 p-6 border border-[var(--color-border)] rounded-xl"
        >
          <div class="h-6 w-3/4 bg-[var(--color-border)] rounded"></div>
          <div class="h-4 w-full bg-[var(--color-border)] rounded"></div>
          <div class="h-4 w-5/6 bg-[var(--color-border)] rounded"></div>
        </div>
      {/each}
    </div>
  {:else if error}
    <div
      class="p-4 rounded-xl bg-red-900/10 border border-red-900/20 text-red-400 text-sm"
    >
      {error}
    </div>
  {:else if posts.length === 0}
    <div class="text-[var(--color-paragraph)]">No posts yet. Soon.</div>
  {:else}
    <!-- Posts -->
    <section class="space-y-8">
      {#each posts as post}
        <a href="/blog/{post.slug}" class="block group">
          <article class="flex flex-col gap-2">
            <h2
              class="text-xl font-semibold text-[var(--color-headline)] group-hover:text-[var(--color-highlight)] transition-colors"
            >
              {post.title}
            </h2>
            <div class="text-xs text-[var(--color-paragraph)] mb-1">
              {new Date(post.publishedAt).toLocaleDateString("en-US", {
                year: "numeric",
                month: "long",
                day: "numeric",
              })}
            </div>
          </article>
        </a>
      {/each}
    </section>
  {/if}
</div>
