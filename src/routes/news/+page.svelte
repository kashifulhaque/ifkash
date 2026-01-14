<svelte:head>
  <title>News — Kashif</title>
  <meta name="description" content="Top Hacker News stories." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { getApiBase } from '$lib/apiBase';

  const API_BASE = getApiBase();

  interface Story {
    by: string;
    id: number;
    score: number;
    time: number;
    title: string;
    url: string;
  }

  let stories: Story[] = [];
  let loading = true;
  let error = '';

  onMount(async () => {
    try {
      const res = await fetch(`${API_BASE}/api/hn`);
      if (!res.ok) throw new Error('Failed');
      stories = await res.json();
    } catch {
      error = 'Failed to load stories.';
    } finally {
      loading = false;
    }
  });

  function getHost(url: string): string {
    try {
      return new URL(url).hostname.replace('www.', '');
    } catch {
      return '';
    }
  }

  function timeAgo(timestamp: number): string {
    const seconds = Math.floor(Date.now() / 1000 - timestamp);
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
    return `${Math.floor(seconds / 86400)}d`;
  }
</script>

<div class="page">
  <header class="page-header">
    <h1 class="page-title">Hacker News</h1>
    <p class="page-desc">What's buzzing in tech.</p>
  </header>

  {#if loading}
    <div class="loading">
      {#each Array(8) as _, i}
        <div class="skeleton" style="--delay: {i * 50}ms"></div>
      {/each}
    </div>
  {:else if error}
    <p class="error">{error}</p>
  {:else}
    <section class="stories">
      {#each stories as story, i (story.id)}
        <article class="story" style="--delay: {i * 30}ms">
          <div class="story-score">{story.score}</div>
          <div class="story-content">
            <a href={story.url} target="_blank" rel="noopener noreferrer" class="story-title">
              {story.title}
            </a>
            <div class="story-meta">
              <span class="story-host">{getHost(story.url)}</span>
              <span class="story-divider">·</span>
              <span>{story.by}</span>
              <span class="story-divider">·</span>
              <span>{timeAgo(story.time)}</span>
              <span class="story-divider">·</span>
              <a 
                href={`https://news.ycombinator.com/item?id=${story.id}`}
                target="_blank"
                rel="noopener noreferrer"
                class="story-discuss"
              >
                discuss
              </a>
            </div>
          </div>
        </article>
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

  .stories {
    display: flex;
    flex-direction: column;
  }
  
  .story {
    display: flex;
    gap: 1rem;
    padding: 1rem 0;
    border-bottom: 1px solid var(--gray-900);
    animation: fade-up var(--duration-base) var(--ease-out) backwards;
    animation-delay: var(--delay);
  }
  
  .story:last-child {
    border-bottom: none;
  }
  
  .story-score {
    flex-shrink: 0;
    width: 40px;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--gray-500);
    text-align: right;
    padding-top: 0.125rem;
  }
  
  .story-content {
    flex: 1;
    min-width: 0;
  }
  
  .story-title {
    display: block;
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--white);
    line-height: 1.4;
    margin-bottom: 0.375rem;
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .story-title:hover {
    color: var(--gray-400);
  }
  
  .story-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
    font-size: 0.75rem;
    color: var(--gray-600);
  }
  
  .story-host {
    color: var(--gray-500);
  }
  
  .story-divider {
    color: var(--gray-800);
  }
  
  .story-discuss {
    color: var(--gray-600);
    transition: color var(--duration-fast) var(--ease-out);
  }
  
  .story-discuss:hover {
    color: var(--white);
  }

  .loading {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .skeleton {
    height: 64px;
    background: var(--gray-900);
    border-radius: var(--radius-sm);
    animation: pulse 2s ease-in-out infinite;
    animation-delay: var(--delay);
  }
  
  .error {
    color: var(--gray-500);
  }
  
  @keyframes fade-up {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
