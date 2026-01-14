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
        <div class="skeleton" style="--delay: {i * 100}ms">
          <div class="skeleton-title"></div>
          <div class="skeleton-date"></div>
        </div>
      {/each}
    </div>
  {:else if error}
    <p class="error">{error}</p>
  {:else if posts.length === 0}
    <p class="empty">No posts yet.</p>
  {:else}
    <section class="posts">
      {#each posts as post, i}
        <a href="/blog/{post.slug}" class="post" style="--delay: {i * 50}ms">
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
    gap: 3rem;
  }
  
  .page-header {
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--gray-800);
  }
  
  .page-title {
    font-size: 2.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--white);
    margin-bottom: 0.5rem;
  }
  
  .page-desc {
    font-size: 1rem;
    color: var(--gray-500);
  }

  .posts {
    display: flex;
    flex-direction: column;
  }
  
  .post {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    padding: 1.25rem 0;
    border-bottom: 1px solid var(--gray-900);
    animation: fade-up var(--duration-slow) var(--ease-out) backwards;
    animation-delay: var(--delay);
    transition: all var(--duration-fast) var(--ease-out);
  }
  
  @media (min-width: 480px) {
    .post {
      flex-direction: row;
      justify-content: space-between;
      align-items: baseline;
    }
  }
  
  .post:hover {
    padding-left: 1rem;
    background: var(--glass-bg);
    margin-left: -1rem;
    margin-right: -1rem;
    padding-right: 1rem;
  }
  
  .post:last-child {
    border-bottom: none;
  }
  
  .post-title {
    font-size: 1rem;
    font-weight: 500;
    color: var(--white);
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .post:hover .post-title {
    color: var(--gray-300);
  }
  
  .post-date {
    font-size: 0.8125rem;
    color: var(--gray-600);
    font-variant-numeric: tabular-nums;
  }

  .loading {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .skeleton {
    padding: 1.25rem 0;
    animation: pulse 2s ease-in-out infinite;
    animation-delay: var(--delay);
  }
  
  .skeleton-title {
    height: 1rem;
    width: 60%;
    background: var(--gray-900);
    border-radius: var(--radius-sm);
    margin-bottom: 0.5rem;
  }
  
  .skeleton-date {
    height: 0.75rem;
    width: 20%;
    background: var(--gray-900);
    border-radius: var(--radius-sm);
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  .error {
    color: var(--gray-500);
    padding: 2rem;
    text-align: center;
  }
  
  .empty {
    color: var(--gray-600);
  }
  
  @keyframes fade-up {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
