<svelte:head>
  <title>LeetCode — Kashif</title>
  <meta name="description" content="Recent LeetCode activity and problem stats." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';

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
    // Fire both requests in parallel
    const statsReq = fetch(`/api/lc/profile?username=${encodeURIComponent(username)}&count=${count}`);
    const subsReq = fetch('/api/lc/submissions', {
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

<div class="min-h-screen bg-neutral-950 text-neutral-200 selection:bg-neutral-800 selection:text-white">
  <div class="mx-auto max-w-3xl px-5 sm:px-6 pb-24">
    <!-- Header -->
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
    <section class="pt-14 sm:pt-20" aria-labelledby="lc-title">
      <div class="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 id="lc-title" class="text-3xl sm:text-4xl font-semibold leading-tight tracking-tight text-neutral-100">LeetCode</h1>
          <p class="mt-2 text-neutral-400">Recent ACs and problem totals. <span class="rounded-full border border-neutral-800 bg-neutral-900 px-2 py-0.5 text-xs text-neutral-300">@{username}</span></p>
        </div>
      </div>
    </section>

    <!-- Stats Cards -->
    <section class="mt-8 grid grid-cols-1 sm:grid-cols-3 gap-3">
      {#if loadingStats}
        {#each Array(3) as _}
          <div class="animate-pulse rounded-2xl border border-neutral-900 bg-neutral-900/40 p-4">
            <div class="h-5 w-24 rounded bg-neutral-800/60"></div>
            <div class="mt-3 h-6 w-32 rounded bg-neutral-800/60"></div>
            <div class="mt-4 h-2 w-full rounded bg-neutral-800/60"></div>
          </div>
        {/each}
      {:else if errStats}
        <div class="sm:col-span-3 rounded-2xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-red-300">{errStats}</div>
      {:else}
        {#each difficulties as key}
          <div class="rounded-2xl border border-neutral-800 bg-neutral-900/60 p-4">
            <div class="text-xs uppercase tracking-wide text-neutral-500">{key}</div>
            <div class="mt-1 text-lg font-medium text-neutral-100">{stats[key].complete}/{stats[key].total}</div>
            <div class="mt-3 h-1.5 w-full overflow-hidden rounded-full bg-neutral-800">
              <div class="h-full rounded-full bg-neutral-200" style={`width:${pct(stats[key].complete, stats[key].total)}%`}></div>
            </div>
          </div>
        {/each}
      {/if}
    </section>

    <!-- Recent submissions -->
    <section class="mt-8">
      <div class="flex items-end justify-between">
        <h2 class="text-xl font-semibold text-neutral-100">Recent AC Submissions</h2>
        <span class="text-sm text-neutral-500">last {count}</span>
      </div>

      {#if loadingSubs}
        <div class="mt-4 grid gap-2">
          {#each Array(6) as _}
            <div class="animate-pulse rounded-lg border border-neutral-900 bg-neutral-900/40 p-3">
              <div class="h-4 w-24 rounded bg-neutral-800/60"></div>
              <div class="mt-2 h-4 w-2/3 rounded bg-neutral-800/60"></div>
            </div>
          {/each}
        </div>
      {:else if errSubs}
        <div class="mt-4 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-red-300">{errSubs}</div>
      {:else if recent.length === 0}
        <div class="mt-4 rounded-xl border border-neutral-800 bg-neutral-900/60 p-4 text-sm text-neutral-300">No recent submissions.</div>
      {:else}
        <div class="mt-4 overflow-hidden rounded-2xl border border-neutral-800">
          <table class="w-full text-sm">
            <thead class="bg-neutral-900/60 text-neutral-400">
              <tr>
                <th class="px-4 py-3 text-left font-medium">Date</th>
                <th class="px-4 py-3 text-left font-medium">Title</th>
              </tr>
            </thead>
            <tbody>
              {#each recent as { id, title, titleSlug, timestamp }}
                <tr class="odd:bg-neutral-950 even:bg-neutral-900/40 hover:bg-neutral-900">
                  <td class="px-4 py-2 text-neutral-400">{new Date(parseInt(timestamp) * 1000).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' })}</td>
                  <td class="px-4 py-2">
                    <a href={`https://leetcode.com/problems/${titleSlug}/`} target="_blank" rel="noopener noreferrer" class="text-neutral-100 hover:underline">{title}</a>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </section>

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
