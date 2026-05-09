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

<header class="page-header">
  <div class="meta-row">
    <span>· Build Log ·</span>
    <span class="right">
      {#if !loading && !error}{posts.length} entries{:else}—{/if}
    </span>
  </div>
  <h1 class="section-title">Blog.</h1>
  <p class="section-subtitle">
    Notes from the build log and rough cuts worth sharing.
  </p>
  <div class="ascii-rule"></div>
</header>

{#if loading}
  <div class="posts-list" aria-busy="true">
    {#each Array(3) as _, i}
      <div class="post-row skeleton" style="--i: {i}">
        <span class="post-num">{String(i + 1).padStart(2, "0")}</span>
        <span class="skel-bar skel-title" style="width: {60 + i * 5}%"></span>
        <span class="skel-bar skel-date"></span>
      </div>
    {/each}
  </div>
{:else if error}
  <p class="error-msg">{error}</p>
{:else if posts.length === 0}
  <p class="empty-msg">[no entries yet]</p>
{:else}
  <section class="posts-list">
    {#each posts as post, i}
      <a
        href="/blog/{post.slug}"
        class="post-row stagger"
        style="--i: {i}"
      >
        <span class="post-num">{String(i + 1).padStart(2, "0")}</span>
        <span class="post-title">{post.title}</span>
        <span class="post-leader" aria-hidden="true"></span>
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

<style>
  .page-header {
    padding-top: 16px;
  }

  .meta-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 16px;
    flex-wrap: wrap;
    margin-bottom: 24px;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--ink-mute);
  }

  .meta-row .right {
    color: var(--blueprint);
  }

  .posts-list {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--rule-soft);
  }

  .post-row {
    display: grid;
    grid-template-columns: 56px minmax(0, auto) 1fr auto;
    align-items: baseline;
    gap: 16px;
    padding: 14px 0;
    border-bottom: 1px solid var(--rule-soft);
    border-left: none !important;
    border-right: none !important;
    transition: background 0.15s, padding 0.15s;
    cursor: pointer;
  }

  .post-row:hover {
    background: var(--blueprint-tint);
    padding-left: 12px;
    padding-right: 12px;
    margin-left: -12px;
    margin-right: -12px;
  }

  .post-num {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.1em;
    color: var(--blueprint);
    font-variant-numeric: tabular-nums;
  }

  .post-title {
    font-family: var(--font-display);
    font-size: 1.4rem;
    text-transform: uppercase;
    letter-spacing: 0.02em;
    color: var(--ink);
    line-height: 1.05;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .post-row:hover .post-title {
    color: var(--blueprint);
  }

  .post-leader {
    border-bottom: 1px dotted var(--rule-soft);
    transform: translateY(-4px);
  }

  .post-date {
    font-family: var(--font-mono);
    font-size: 0.74rem;
    letter-spacing: 0.08em;
    color: var(--ink-mute);
    font-variant-numeric: tabular-nums;
    text-transform: uppercase;
    white-space: nowrap;
    text-align: right;
    min-width: 80px;
  }

  /* ─── Skeleton ─────────────────────────────────────────────── */

  .skeleton {
    cursor: default;
    pointer-events: none;
  }

  .skel-bar {
    height: 14px;
    background: var(--bg-surface);
    animation: pulse 1.6s var(--ease-out-quart) infinite;
    animation-delay: calc(var(--i, 0) * 80ms);
  }

  .skel-title {
    align-self: center;
    grid-column: 2 / 4;
  }

  .skel-date {
    width: 80px;
    align-self: center;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  /* ─── Empty / error ────────────────────────────────────────── */

  .error-msg,
  .empty-msg {
    font-family: var(--font-mono);
    font-size: 0.85rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-mute);
    padding: 32px 0;
    text-align: center;
  }

  @media (max-width: 768px) {
    .post-row {
      grid-template-columns: 36px minmax(0, 1fr);
      grid-template-rows: auto auto;
      gap: 4px 12px;
      padding: 12px 0;
    }

    .post-num {
      grid-row: 1;
      grid-column: 1;
    }

    .post-title {
      grid-row: 1;
      grid-column: 2;
      font-size: 1.1rem;
    }

    .post-leader {
      display: none;
    }

    .post-date {
      grid-row: 2;
      grid-column: 2;
      text-align: left;
      min-width: 0;
    }

    .skel-title {
      grid-column: 2;
    }

    .skel-date {
      grid-column: 2;
      grid-row: 2;
    }
  }
</style>
