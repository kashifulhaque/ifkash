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

  function formatDate(d: string) {
    const date = new Date(d);
    return {
      day: date.toLocaleDateString("en-US", { day: "2-digit" }),
      mon: date.toLocaleDateString("en-US", { month: "short" }).toUpperCase(),
      year: date.getFullYear().toString(),
    };
  }
</script>

<svelte:head>
  <title>Writing — Kashiful Haque</title>
  <meta name="description" content="Notes on ML systems, agent infrastructure, and inference engineering." />
</svelte:head>

<div class="page">
  <header class="page-header">
    <span class="eyebrow">section · 04 · writing</span>
    <h1 class="page-title">
      Build logs &<br />
      <em>field notes.</em>
    </h1>
    <p class="page-desc">
      Slow takes, deep dives, and rough cuts worth shipping. Mostly on
      models, runtimes, and the seams between them.
    </p>
  </header>

  {#if loading}
    <div class="loading">
      {#each Array(4) as _, i}
        <div class="skeleton" style="--delay: {i * 120}ms">
          <div class="skel-date"></div>
          <div class="skel-body">
            <div class="skel-title"></div>
            <div class="skel-brief"></div>
          </div>
        </div>
      {/each}
    </div>
  {:else if error}
    <p class="error-msg">{error}</p>
  {:else if posts.length === 0}
    <p class="empty-msg">No posts yet — the build log is being warmed up.</p>
  {:else}
    <section class="posts">
      {#each posts as post, i}
        {@const d = formatDate(post.publishedAt)}
        <a href="/blog/{post.slug}" class="post stagger" style="--i: {i}">
          <time class="post-date" datetime={post.publishedAt}>
            <span class="date-day">{d.day}</span>
            <span class="date-mon">{d.mon}</span>
            <span class="date-year">{d.year}</span>
          </time>

          <div class="post-body">
            <div class="post-num">[ 0{i + 1} ]</div>
            <h2 class="post-title">{post.title}</h2>
            {#if post.brief}
              <p class="post-brief">{post.brief}</p>
            {/if}
          </div>

          <span class="post-arrow">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M5 12h14M13 5l7 7-7 7"/></svg>
          </span>
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
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    padding-bottom: var(--space-xl);
    border-bottom: 1px solid var(--border);
    max-width: 760px;
  }

  .page-title {
    font-size: clamp(2.5rem, 5vw + 0.5rem, 4.25rem);
    letter-spacing: -0.04em;
    line-height: 1.05;
    color: var(--text-primary);
    font-weight: 400;
  }

  .page-title em {
    font-family: var(--font-serif);
    font-style: italic;
    font-weight: 300;
    color: var(--accent);
    letter-spacing: -0.02em;
  }

  .page-desc {
    font-size: 1.0625rem;
    line-height: 1.55;
    color: var(--text-tertiary);
    max-width: 56ch;
  }

  .posts {
    display: flex;
    flex-direction: column;
  }

  .post {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.75rem;
    padding: var(--space-lg) 0;
    border-bottom: 1px solid var(--border);
    color: inherit;
    transition: padding var(--dur-fast) var(--ease-out-quart);
  }

  .post:first-child {
    border-top: 1px solid var(--border);
  }

  @media (min-width: 720px) {
    .post {
      grid-template-columns: 90px 1fr auto;
      gap: 2rem;
      padding: var(--space-lg) 1.25rem;
      margin-left: -1.25rem;
      margin-right: -1.25rem;
      border-radius: var(--radius-md);
      align-items: start;
    }
    .post:hover {
      background: var(--surface-mesh);
    }
  }

  .post-date {
    display: grid;
    grid-template-columns: auto auto auto;
    gap: 0.4rem;
    align-items: baseline;
    font-family: var(--font-mono);
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  @media (min-width: 720px) {
    .post-date {
      grid-template-columns: 1fr;
      gap: 0;
      padding-top: 0.4rem;
    }
  }

  .date-day {
    font-size: 1.75rem;
    font-weight: 400;
    color: var(--text-primary);
    letter-spacing: -0.04em;
    line-height: 1;
  }

  .post:hover .date-day {
    color: var(--accent);
  }

  .date-mon {
    font-size: 0.625rem;
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--text-faint);
  }

  .date-year {
    font-size: 0.625rem;
    color: var(--text-faint);
    letter-spacing: 0.04em;
  }

  .post-body {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    min-width: 0;
  }

  .post-num {
    font-family: var(--font-mono);
    font-size: 0.625rem;
    color: var(--text-faint);
    letter-spacing: 0.06em;
  }

  .post:hover .post-num {
    color: var(--accent);
  }

  .post-title {
    font-family: var(--font-sans);
    font-size: 1.25rem;
    font-weight: 500;
    color: var(--text-primary);
    letter-spacing: -0.02em;
    line-height: 1.25;
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .post:hover .post-title {
    color: var(--accent);
  }

  .post-brief {
    font-size: 0.9375rem;
    line-height: 1.55;
    color: var(--text-tertiary);
    max-width: 64ch;
  }

  .post-arrow {
    display: none;
    color: var(--text-faint);
    padding-top: 0.5rem;
    transition: color var(--dur-fast), transform var(--dur-fast);
  }

  @media (min-width: 720px) {
    .post-arrow {
      display: inline-flex;
    }
  }

  .post:hover .post-arrow {
    color: var(--accent);
    transform: translate(3px, -3px);
  }

  .loading {
    display: flex;
    flex-direction: column;
  }

  .skeleton {
    display: grid;
    grid-template-columns: 90px 1fr;
    gap: 2rem;
    padding: var(--space-lg) 0;
    border-bottom: 1px solid var(--border-subtle);
    animation: pulse 2s ease-in-out infinite;
    animation-delay: var(--delay);
  }

  .skel-date {
    height: 1.75rem;
    width: 70%;
    background: var(--surface-sunken);
    border-radius: var(--radius-sm);
  }

  .skel-body {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .skel-title {
    height: 1.25rem;
    width: 60%;
    background: var(--surface-sunken);
    border-radius: var(--radius-sm);
  }

  .skel-brief {
    height: 0.8125rem;
    width: 80%;
    background: var(--surface-sunken);
    border-radius: var(--radius-sm);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.45; }
  }

  .error-msg,
  .empty-msg {
    color: var(--text-tertiary);
    padding: 2rem;
    text-align: center;
    background: var(--surface-glass);
    border: 1px dashed var(--border);
    border-radius: var(--radius-lg);
  }

  .empty-msg {
    color: var(--text-faint);
  }
</style>
