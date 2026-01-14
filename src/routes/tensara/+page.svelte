<svelte:head>
  <title>Tensara — Kashif</title>
  <meta name="description" content="Tensara GPU computing stats and submissions." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { getApiBase } from '$lib/apiBase';

  const API_BASE = getApiBase();

  type Stats = {
    submissions: number;
    solvedProblems: number;
    ranking: number;
    rating: number;
  };

  type RecentSubmission = {
    id: string;
    problemId: string;
    problemName: string;
    date: string;
    status: string;
    runtime: string;
    gflops: string;
    gpuType: string;
    language: string;
  };

  type LanguagePercentage = {
    language: string;
    percentage: number;
  };

  type CommunityStats = {
    totalPosts: number;
    totalLikes: number;
  };

  type TensaraUser = {
    id: string;
    username: string;
    name: string;
    image: string;
    joinedAt: string;
    stats: Stats;
    recentSubmissions: RecentSubmission[];
    communityStats: CommunityStats;
    languagePercentage: LanguagePercentage[];
  };

  let username = 'kashifulhaque';
  $: username = $page.url.searchParams.get('u') || 'kashifulhaque';

  let userData: TensaraUser | null = null;
  let loading = true;
  let error = '';

  onMount(async () => {
    try {
      const res = await fetch(`${API_BASE}/api/tensara/profile?username=${encodeURIComponent(username)}`);
      if (!res.ok) throw new Error('Failed to load profile');
      userData = await res.json();
    } catch (err) {
      error = 'Failed to load Tensara profile.';
    } finally {
      loading = false;
    }
  });

  function formatDate(dateStr: string) {
    return new Date(dateStr).toLocaleDateString('en-US', { 
      month: 'short', 
      day: 'numeric',
      year: 'numeric'
    });
  }
</script>

<div class="page">
  <header class="page-header">
    <div class="page-header-main">
      <h1 class="page-title">Tensara</h1>
      <p class="page-desc">GPU computing stats and recent submissions.</p>
    </div>
    {#if userData}
      <a href={`https://tensara.org/user/${username}`} target="_blank" rel="noopener noreferrer" class="username">
        @{username}
      </a>
    {/if}
  </header>

  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
    </div>
  {:else if error}
    <p class="error">{error}</p>
  {:else if userData}
    <!-- User Info -->
    <section class="user-info">
      <img src={userData.image} alt={userData.name} class="user-avatar" />
      <div class="user-details">
        <h2 class="user-name">{userData.name}</h2>
        <p class="user-joined">Joined {formatDate(userData.joinedAt)}</p>
      </div>
    </section>

    <!-- Stats Grid -->
    <section class="stats-grid">
      <div class="stat-card" style="--delay: 0ms">
        <div class="stat-label">Rating</div>
        <div class="stat-value">{userData.stats.rating}</div>
      </div>
      <div class="stat-card" style="--delay: 50ms">
        <div class="stat-label">Ranking</div>
        <div class="stat-value">#{userData.stats.ranking}</div>
      </div>
      <div class="stat-card" style="--delay: 100ms">
        <div class="stat-label">Solved</div>
        <div class="stat-value">{userData.stats.solvedProblems}</div>
      </div>
      <div class="stat-card" style="--delay: 150ms">
        <div class="stat-label">Submissions</div>
        <div class="stat-value">{userData.stats.submissions}</div>
      </div>
    </section>

    <!-- Language Distribution -->
    {#if userData.languagePercentage.length > 0}
      <section class="languages">
        <h2 class="section-title">Languages</h2>
        <div class="lang-list">
          {#each userData.languagePercentage as lang, i}
            <div class="lang-item" style="--delay: {i * 50}ms">
              <div class="lang-header">
                <span class="lang-name">{lang.language}</span>
                <span class="lang-pct">{lang.percentage}%</span>
              </div>
              <div class="lang-bar">
                <div class="lang-fill" style="width: {lang.percentage}%"></div>
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Recent Submissions -->
    {#if userData.recentSubmissions.length > 0}
      <section class="submissions">
        <h2 class="section-title">Recent Submissions</h2>
        <div class="sub-list">
          {#each userData.recentSubmissions as sub, i}
            <a 
              href={`https://tensara.org/problems/${sub.problemId}`}
              target="_blank"
              rel="noopener noreferrer"
              class="sub-item"
              style="--delay: {i * 30}ms"
            >
              <div class="sub-main">
                <span class="sub-title">{sub.problemName}</span>
                <div class="sub-meta">
                  <span class="sub-lang">{sub.language}</span>
                  <span class="sub-gpu">{sub.gpuType}</span>
                  <span class="sub-status status-{sub.status.toLowerCase()}">{sub.status}</span>
                </div>
              </div>
              <div class="sub-stats">
                <div class="sub-stat">
                  <span class="sub-stat-label">Runtime</span>
                  <span class="sub-stat-value">{sub.runtime}</span>
                </div>
                <div class="sub-stat">
                  <span class="sub-stat-label">GFLOPS</span>
                  <span class="sub-stat-value">{sub.gflops}</span>
                </div>
              </div>
            </a>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Community Stats -->
    <section class="community">
      <h2 class="section-title">Community</h2>
      <div class="community-grid">
        <div class="community-stat">
          <span class="community-value">{userData.communityStats.totalPosts}</span>
          <span class="community-label">Posts</span>
        </div>
        <div class="community-stat">
          <span class="community-value">{userData.communityStats.totalLikes}</span>
          <span class="community-label">Likes</span>
        </div>
      </div>
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

  .loading {
    display: flex;
    justify-content: center;
    padding: 4rem 0;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--gray-800);
    border-top-color: var(--white);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error {
    color: var(--gray-500);
    font-size: 0.9375rem;
    text-align: center;
    padding: 2rem 0;
  }

  /* User Info */
  .user-info {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 1.5rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
    animation: fade-up var(--duration-slow) var(--ease-out);
  }

  .user-avatar {
    width: 72px;
    height: 72px;
    border-radius: 50%;
    border: 2px solid var(--gray-800);
  }

  .user-name {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--white);
    margin-bottom: 0.25rem;
  }

  .user-joined {
    font-size: 0.875rem;
    color: var(--gray-500);
  }

  /* Stats Grid */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 1rem;
  }

  .stat-card {
    padding: 1.25rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
    animation: fade-up var(--duration-slow) var(--ease-out) backwards;
    animation-delay: var(--delay);
  }

  .stat-label {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--gray-500);
    margin-bottom: 0.5rem;
  }

  .stat-value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--white);
  }

  /* Languages */
  .languages {
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

  .lang-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .lang-item {
    animation: fade-up var(--duration-base) var(--ease-out) backwards;
    animation-delay: var(--delay);
  }

  .lang-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .lang-name {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--gray-300);
  }

  .lang-pct {
    font-size: 0.75rem;
    color: var(--gray-500);
  }

  .lang-bar {
    height: 6px;
    background: var(--gray-900);
    border-radius: 3px;
    overflow: hidden;
  }

  .lang-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--gray-400), var(--white));
    transition: width 1s var(--ease-out);
  }

  /* Submissions */
  .submissions {
    border-top: 1px solid var(--gray-800);
    padding-top: 2rem;
  }

  .sub-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .sub-item {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
    animation: fade-up var(--duration-base) var(--ease-out) backwards;
    animation-delay: var(--delay);
    transition: all var(--duration-fast) var(--ease-out);
  }

  @media (min-width: 640px) {
    .sub-item {
      flex-direction: row;
      justify-content: space-between;
      align-items: center;
    }
  }

  .sub-item:hover {
    border-color: var(--gray-700);
    background: var(--gray-900);
  }

  .sub-main {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    flex: 1;
  }

  .sub-title {
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--gray-300);
  }

  .sub-item:hover .sub-title {
    color: var(--white);
  }

  .sub-meta {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .sub-lang,
  .sub-gpu {
    font-size: 0.75rem;
    color: var(--gray-500);
    padding: 0.25rem 0.5rem;
    background: var(--gray-900);
    border-radius: var(--radius-sm);
  }

  .sub-status {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-sm);
    font-weight: 500;
  }

  .status-accepted {
    color: #10b981;
    background: rgba(16, 185, 129, 0.1);
  }

  .status-rejected {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
  }

  .sub-stats {
    display: flex;
    gap: 1.5rem;
  }

  .sub-stat {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
  }

  .sub-stat-label {
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--gray-600);
    margin-bottom: 0.25rem;
  }

  .sub-stat-value {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--gray-300);
    font-variant-numeric: tabular-nums;
  }

  /* Community */
  .community {
    border-top: 1px solid var(--gray-800);
    padding-top: 2rem;
  }

  .community-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
  }

  .community-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1.5rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
  }

  .community-value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--white);
    margin-bottom: 0.5rem;
  }

  .community-label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--gray-500);
  }

  @keyframes fade-up {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
