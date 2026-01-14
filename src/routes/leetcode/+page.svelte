<svelte:head>
  <title>LeetCode — Kashif</title>
  <meta name="description" content="LeetCode stats and submissions." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { getApiBase } from '$lib/apiBase';

  const API_BASE = getApiBase();

  type DiffKey = 'easy' | 'medium' | 'hard';
  type Band = { complete: number; total: number };
  
  const difficulties: DiffKey[] = ['easy', 'medium', 'hard'];

  type Submission = {
    id: string;
    title: string;
    titleSlug: string;
    timestamp: string;
  };

  let username = 'ifkash';
  $: username = $page.url.searchParams.get('u') || 'ifkash';

  let stats: Record<DiffKey, Band> = {
    easy: { complete: 0, total: 0 },
    medium: { complete: 0, total: 0 },
    hard: { complete: 0, total: 0 }
  };

  let recent: Submission[] = [];
  let loadingStats = true;
  let loadingSubs = true;
  let errStats = '';
  let errSubs = '';

  const count = 15;

  onMount(async () => {
    const statsReq = fetch(`${API_BASE}/api/lc/profile?username=${encodeURIComponent(username)}&count=${count}`);
    const subsReq = fetch(`${API_BASE}/api/lc/submissions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, count })
    });

    try {
      const sRes = await statsReq;
      if (!sRes.ok) throw new Error('Failed');
      stats = await sRes.json();
    } catch {
      errStats = 'Failed to load stats.';
    } finally {
      loadingStats = false;
    }

    try {
      const sub = await subsReq;
      if (!sub.ok) throw new Error('Failed');
      const payload = await sub.json();
      recent = payload?.data?.recentAcSubmissionList ?? [];
    } catch {
      errSubs = 'Failed to load submissions.';
    } finally {
      loadingSubs = false;
    }
  });

  function pct(done: number, tot: number) {
    return tot ? Math.round((done / tot) * 100) : 0;
  }
</script>

<div class="page">
  <header class="page-header">
    <div class="page-header-main">
      <h1 class="page-title">LeetCode</h1>
      <p class="page-desc">Problem stats and recent submissions.</p>
    </div>
    <a href="https://leetcode.com/u/{username}" target="_blank" rel="noopener noreferrer" class="username">
      @{username}
    </a>
  </header>

  <!-- Stats -->
  <section class="stats">
    {#if loadingStats}
      {#each ['E', 'M', 'H'] as _, i}
        <div class="stat skeleton" style="--delay: {i * 100}ms"></div>
      {/each}
    {:else if errStats}
      <p class="error">{errStats}</p>
    {:else}
      {#each difficulties as diff, i}
        <div class="stat" style="--delay: {i * 50}ms">
          <div class="stat-header">
            <span class="stat-label">{diff}</span>
            <span class="stat-pct">{pct(stats[diff].complete, stats[diff].total)}%</span>
          </div>
          <div class="stat-value">
            {stats[diff].complete}<span class="stat-total">/{stats[diff].total}</span>
          </div>
          <div class="stat-bar">
            <div class="stat-fill" style="width: {pct(stats[diff].complete, stats[diff].total)}%"></div>
          </div>
        </div>
      {/each}
    {/if}
  </section>

  <!-- Submissions -->
  <section class="submissions">
    <h2 class="section-title">Recent</h2>
    
    {#if loadingSubs}
      {#each Array(5) as _, i}
        <div class="sub-skeleton" style="--delay: {i * 50}ms"></div>
      {/each}
    {:else if errSubs}
      <p class="error">{errSubs}</p>
    {:else if recent.length === 0}
      <p class="empty">No submissions.</p>
    {:else}
      <div class="sub-list">
        {#each recent as sub, i}
          <a 
            href={`https://leetcode.com/problems/${sub.titleSlug}/`}
            target="_blank"
            rel="noopener noreferrer"
            class="sub-item"
            style="--delay: {i * 30}ms"
          >
            <span class="sub-title">{sub.title}</span>
            <span class="sub-date">
              {new Date(parseInt(sub.timestamp) * 1000).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
            </span>
          </a>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 3rem;
  }
  
  .page-header {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--gray-800);
  }
  
  @media (min-width: 480px) {
    .page-header {
      flex-direction: row;
      justify-content: space-between;
      align-items: flex-start;
    }
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
  
  .username {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--gray-500);
    padding: 0.5rem 0.875rem;
    background: var(--gray-900);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-sm);
    transition: all var(--duration-fast) var(--ease-out);
  }
  
  .username:hover {
    color: var(--white);
    border-color: var(--gray-700);
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }
  
  .stat {
    padding: 1.25rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
    animation: fade-up var(--duration-slow) var(--ease-out) backwards;
    animation-delay: var(--delay);
  }
  
  .stat.skeleton {
    height: 100px;
    animation: pulse 2s ease-in-out infinite;
  }
  
  .stat-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  
  .stat-label {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--gray-500);
  }
  
  .stat-pct {
    font-size: 0.6875rem;
    color: var(--gray-600);
  }
  
  .stat-value {
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--white);
    margin-bottom: 0.75rem;
  }
  
  .stat-total {
    font-size: 0.875rem;
    font-weight: 400;
    color: var(--gray-600);
  }
  
  .stat-bar {
    height: 3px;
    background: var(--gray-800);
    border-radius: 2px;
    overflow: hidden;
  }
  
  .stat-fill {
    height: 100%;
    background: var(--white);
    transition: width 1s var(--ease-out);
  }

  .submissions {
    border-top: 1px solid var(--gray-800);
    padding-top: 2rem;
  }
  
  .section-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--gray-600);
    margin-bottom: 1.5rem;
  }
  
  .sub-list {
    display: flex;
    flex-direction: column;
  }
  
  .sub-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.875rem 0;
    border-bottom: 1px solid var(--gray-900);
    animation: fade-up var(--duration-base) var(--ease-out) backwards;
    animation-delay: var(--delay);
    transition: all var(--duration-fast) var(--ease-out);
  }
  
  .sub-item:hover {
    padding-left: 0.75rem;
    background: var(--glass-bg);
    margin-left: -0.75rem;
    margin-right: -0.75rem;
    padding-right: 0.75rem;
  }
  
  .sub-item:last-child {
    border-bottom: none;
  }
  
  .sub-title {
    font-size: 0.9375rem;
    color: var(--gray-300);
  }
  
  .sub-item:hover .sub-title {
    color: var(--white);
  }
  
  .sub-date {
    font-size: 0.75rem;
    color: var(--gray-600);
    font-variant-numeric: tabular-nums;
  }
  
  .sub-skeleton {
    height: 48px;
    background: var(--gray-900);
    border-radius: var(--radius-sm);
    margin-bottom: 0.5rem;
    animation: pulse 2s ease-in-out infinite;
    animation-delay: var(--delay);
  }
  
  .error, .empty {
    color: var(--gray-500);
    font-size: 0.9375rem;
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
