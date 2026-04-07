<script lang="ts">
  import { onMount } from "svelte";

  type Post = {
    id: string;
    title: string;
    brief: string;
    url: string;
    slug: string;
    publishedAt: string;
  };

  let posts: Post[] = [];
  let loading = true;
  let error = "";

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
          error = "Failed to load posts.";
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
  <meta name="description" content="Writing, notes, and experiments." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <h1 class="page-title">Blog</h1>
    <p class="page-desc">Notes from the build log and rough cuts worth sharing.</p>
  </header>

  {#if loading}
    <div class="loading">
      {#each Array(3) as _, i}
        <div class="skeleton" style="--delay: {i * 120}ms">
          <div class="skel-title"></div>
          <div class="skel-date"></div>
        </div>
      {/each}
    </div>
  {:else if error}
    <p class="error-msg">{error}</p>
  {:else if posts.length === 0}
    <p class="empty-msg">No posts yet.</p>
  {:else}
    <section class="posts">
      {#each posts as post, i}
        <a href="/blog/{post.slug}" class="post stagger" style="--i: {i}">
          <h2 class="post-title">{post.title}</h2>
          <time class="post-date">
            {new Date(post.publishedAt).toLocaleDateString("en-US", {
              year: "numeric",
              month: "short",
              day: "numeric",
            })}
          </time>
        </a>
      {/each}
    </section>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-2xl);
  }

  .page-header {
    padding-bottom: var(--space-xl);
    border-bottom: 1px solid var(--border);
  }

  .page-title {
    font-size: clamp(2rem, 5vw + 0.5rem, 3rem);
    font-weight: 400;
    letter-spacing: -0.03em;
    color: var(--text-primary);
    margin-bottom: 0.375rem;
  }

  .page-desc {
    font-size: 1rem;
    color: var(--text-tertiary);
  }

  .posts {
    display: flex;
    flex-direction: column;
  }

  .post {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 1rem 0;
    border-bottom: 1px solid var(--border-subtle);
    transition: background var(--dur-instant) var(--ease-out-quart);
    border-radius: var(--radius-sm);
  }

  @media (min-width: 480px) {
    .post {
      flex-direction: row;
      justify-content: space-between;
      align-items: baseline;
    }
  }

  .post:hover {
    background: var(--surface-sunken);
    padding-left: 1rem;
    margin-left: -1rem;
    margin-right: -1rem;
    padding-right: 1rem;
  }

  .post:last-child {
    border-bottom: none;
  }

  .post-title {
    font-family: var(--font-sans);
    font-size: 1rem;
    font-weight: 500;
    color: var(--text-primary);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .post:hover .post-title {
    color: var(--accent);
  }

  .post-date {
    font-size: 0.8125rem;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .loading {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .skeleton {
    padding: 1rem 0;
    animation: pulse 2s ease-in-out infinite;
    animation-delay: var(--delay);
  }

  .skel-title {
    height: 1rem;
    width: 60%;
    background: var(--surface-sunken);
    border-radius: var(--radius-sm);
    margin-bottom: 0.5rem;
  }

  .skel-date {
    height: 0.75rem;
    width: 18%;
    background: var(--surface-sunken);
    border-radius: var(--radius-sm);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .error-msg {
    color: var(--text-tertiary);
    padding: 2rem;
    text-align: center;
  }

  .empty-msg {
    color: var(--text-faint);
  }
</style>
