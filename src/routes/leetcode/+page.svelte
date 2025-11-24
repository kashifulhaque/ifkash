<svelte:head>
  <title>LeetCode — Kashif</title>
  <meta name="description" content="Recent LeetCode activity and problem stats." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { getApiBase } from '$lib/apiBase';

  export const API_BASE = getApiBase();

  type DiffKey = 'easy' | 'medium' | 'hard' | 'total';
  type Band = { complete: number; total: number };
  type Stats = Record<DiffKey, Band>;

  type Submission = {
    id: string;
    title: string;
    titleSlug: string;
    timestamp: string; // seconds
  };

  let username = 'ifkash';
  $: username = $page.url.searchParams.get('u') || 'ifkash';

  let stats: Stats = {
    easy: { complete: 0, total: 0 },
    medium: { complete: 0, total: 0 },
    hard: { complete: 0, total: 0 },
    total: { complete: 0, total: 0 }
  };

  let recent: Submission[] = [];

  let loadingStats = true;
  let loadingSubs = true;
  let errStats = '';
  let errSubs = '';

  const count = 20;
  const difficulties: DiffKey[] = ['easy','medium','hard'];

  onMount(async () => {
    const url = new URL(window.location.href);
    if (url.searchParams.has('api')) {
      url.searchParams.delete('api');
      history.replaceState(history.state, '', url.toString());
    }

    // Fire both requests in parallel
    const statsReq = fetch(`${API_BASE}/api/lc/profile?username=${encodeURIComponent(username)}&count=${count}`);
    const subsReq = fetch(`${API_BASE}/api/lc/submissions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, count })
    });

    try {
      const [sRes, subRes] = await Promise.allSettled([statsReq, subsReq]);

      // Stats
      if (sRes.status === 'fulfilled') {
        if (!sRes.value.ok) throw new Error('Bad stats response');
        stats = await sRes.value.json();
      } else {
        throw sRes.reason;
      }
    } catch (e) {
      console.error(e);
      errStats = 'Failed to load stats.';
    } finally {
      loadingStats = false;
    }

    try {
      // Submissions
      const sub = await subsReq;
      if (!sub.ok) throw new Error('Bad submissions response');
      const payload = await sub.json();
      recent = payload?.data?.recentAcSubmissionList ?? [];
    } catch (e) {
      console.error(e);
      errSubs = 'Failed to load submissions.';
    } finally {
      loadingSubs = false;
    }
  });

  function pct(done: number, tot: number) {
    if (!tot) return 0;
    return Math.min(100, Math.round((done / tot) * 100));
  }
</script>

<div class="max-w-3xl mx-auto">
  <!-- Title -->
  <section class="mb-12" aria-labelledby="lc-title">
    <div class="flex flex-col sm:flex-row sm:items-baseline justify-between gap-4">
      <div>
        <h1 id="lc-title" class="text-3xl font-bold tracking-tight text-[var(--color-headline)]">Leetcode</h1>
        <p class="mt-2 text-lg text-[var(--color-paragraph)]">Recent ACs and problem totals.</p>
      </div>
      <a href="https://leetcode.com/u/{username}" target="_blank" rel="noopener noreferrer" class="self-start text-sm px-3 py-1 rounded-full bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-paragraph)] hover:text-[var(--color-headline)] hover:border-[var(--color-highlight)] transition-colors">
        @{username}
      </a>
    </div>
  </section>

  <!-- Stats Cards -->
  <section class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-12">
    {#if loadingStats}
      {#each Array(3) as _}
        <div class="animate-pulse p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]">
          <div class="h-4 w-16 bg-[var(--color-border)] rounded mb-4"></div>
          <div class="h-6 w-24 bg-[var(--color-border)] rounded"></div>
        </div>
      {/each}
    {:else if errStats}
      <div class="sm:col-span-3 p-4 rounded-xl bg-red-900/10 border border-red-900/20 text-red-400 text-sm">{errStats}</div>
    {:else}
      {#each difficulties as key}
        <div class="p-5 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)]">
          <div class="text-xs font-bold uppercase tracking-wider text-[var(--color-secondary)] mb-2">{key}</div>
          <div class="text-2xl font-semibold text-[var(--color-headline)] mb-4">{stats[key].complete}<span class="text-sm text-[var(--color-secondary)] font-normal ml-1">/ {stats[key].total}</span></div>
          <div class="h-1.5 w-full rounded-full bg-[var(--color-background)] overflow-hidden">
            <div class="h-full rounded-full bg-[var(--color-highlight)] transition-all duration-500" style={`width:${pct(stats[key].complete, stats[key].total)}%`}></div>
          </div>
        </div>
      {/each}
    {/if}
  </section>

  <!-- Recent submissions -->
  <section>
    <div class="flex items-baseline justify-between mb-6">
      <h2 class="text-xl font-semibold text-[var(--color-headline)]">Recent Submissions</h2>
      <span class="text-sm text-[var(--color-secondary)]">last {count}</span>
    </div>

    {#if loadingSubs}
      <div class="space-y-3">
        {#each Array(5) as _}
          <div class="animate-pulse h-10 w-full bg-[var(--color-surface)] rounded border border-[var(--color-border)]"></div>
        {/each}
      </div>
    {:else if errSubs}
      <div class="p-4 rounded-xl bg-red-900/10 border border-red-900/20 text-red-400 text-sm">{errSubs}</div>
    {:else if recent.length === 0}
      <div class="text-[var(--color-paragraph)]">No recent submissions.</div>
    {:else}
      <div class="rounded-xl border border-[var(--color-border)] overflow-hidden">
        <table class="w-full text-sm text-left">
          <thead class="bg-[var(--color-surface)] text-[var(--color-paragraph)] font-medium border-b border-[var(--color-border)]">
            <tr>
              <th class="px-6 py-3">Date</th>
              <th class="px-6 py-3">Problem</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-[var(--color-border)]">
            {#each recent as { id, title, titleSlug, timestamp }}
              <tr class="hover:bg-[var(--color-surface)] transition-colors">
                <td class="px-6 py-3 text-[var(--color-secondary)] whitespace-nowrap font-mono text-xs">
                  {new Date(parseInt(timestamp) * 1000).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' })}
                </td>
                <td class="px-6 py-3">
                  <a href={`https://leetcode.com/problems/${titleSlug}/`} target="_blank" rel="noopener noreferrer" class="text-[var(--color-headline)] hover:text-[var(--color-highlight)] hover:underline decoration-1 underline-offset-2 transition-colors">
                    {title}
                  </a>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>
</div>
