<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { getApiBase } from '$lib/apiBase';

  export let finalScore = 0;
  export let allSectionsCleared = false;
  export let sectionsCount = 0;
  export let daily = false;
  export let dailyDay: string | null = null;

  const dispatch = createEventDispatcher();
  const clientId = env.PUBLIC_GOOGLE_CLIENT_ID ?? '';
  const CRED_KEY = 'game_google_credential';

  type Entry = { name: string; picture: string | null; score: number };
  let leaderboard: Entry[] = [];
  let leaderboardError = false;
  let submitState: 'idle' | 'submitting' | 'done' | 'error' = 'idle';
  let best = 0;
  let rank = 0;
  let playerName = '';
  let signedIn = false;
  let gisButton: HTMLDivElement;

  // Daily runs report to a separate per-day board; normal runs to the all-time one.
  const boardUrl = daily
    ? `${getApiBase()}/api/game/daily/leaderboard?day=${encodeURIComponent(dailyDay ?? '')}`
    : `${getApiBase()}/api/game/leaderboard`;
  const submitUrl = daily ? `${getApiBase()}/api/game/daily/score` : `${getApiBase()}/api/game/score`;

  async function fetchLeaderboard() {
    try {
      const res = await fetch(boardUrl);
      leaderboard = (await res.json()).entries ?? [];
      leaderboardError = false;
    } catch {
      leaderboardError = true;
    }
  }

  async function submitScore(idToken: string) {
    submitState = 'submitting';
    try {
      const body = daily
        ? { id_token: idToken, score: finalScore, day: dailyDay }
        : { id_token: idToken, score: finalScore };
      const res = await fetch(submitUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body)
      });
      if (res.status === 401) {
        // Token expired — forget it and show the button again
        localStorage.removeItem(CRED_KEY);
        signedIn = false;
        submitState = 'idle';
        renderGoogleButton();
        return;
      }
      if (!res.ok) throw new Error(String(res.status));
      const data = await res.json();
      best = data.best;
      rank = data.rank;
      playerName = data.name ?? '';
      submitState = 'done';
      fetchLeaderboard();
    } catch {
      submitState = 'error';
    }
  }

  function onCredential(resp: { credential: string }) {
    try {
      localStorage.setItem(CRED_KEY, resp.credential);
    } catch {}
    signedIn = true;
    submitScore(resp.credential);
  }

  function renderGoogleButton() {
    const google = (window as any).google;
    if (!google?.accounts?.id || !gisButton) return;
    google.accounts.id.initialize({ client_id: clientId, callback: onCredential });
    gisButton.innerHTML = '';
    google.accounts.id.renderButton(gisButton, { theme: 'filled_black', size: 'large', text: 'signin_with' });
  }

  function loadGis(): Promise<void> {
    if ((window as any).google?.accounts?.id) return Promise.resolve();
    return new Promise((resolve, reject) => {
      const s = document.createElement('script');
      s.src = 'https://accounts.google.com/gsi/client';
      s.async = true;
      s.onload = () => resolve();
      s.onerror = () => reject();
      document.head.appendChild(s);
    });
  }

  onMount(async () => {
    fetchLeaderboard();
    if (!clientId) return;
    const stored = localStorage.getItem(CRED_KEY);
    if (stored) {
      signedIn = true;
      submitScore(stored); // falls back to the button on 401
    } else {
      try {
        await loadGis();
        renderGoogleButton();
      } catch {
        /* GIS blocked — leaderboard still shows */
      }
    }
  });
</script>

<div class="death">
  <div class="inner">
    <h1 class="wasted">WASTED</h1>
    <p class="score">SCORE · {finalScore}</p>
    {#if daily}
      <p class="daily-tag">DAILY CHALLENGE · {dailyDay}</p>
    {/if}

    {#if allSectionsCleared}
      <p class="badge-100">★ 100% KASHIF — ALL SECTIONS VIEWED ★</p>
    {:else}
      <p class="sections-progress">SECTIONS VIEWED · {sectionsCount}/6</p>
    {/if}

    <button class="respawn" on:click={() => dispatch('respawn')}>RESPAWN</button>

    <div class="save">
      {#if !clientId}
        <p class="note">set PUBLIC_GOOGLE_CLIENT_ID to enable the leaderboard</p>
      {:else if submitState === 'submitting'}
        <p class="note">SAVING SCORE…</p>
      {:else if submitState === 'done'}
        <p class="note saved">
          SAVED{playerName ? ` · ${playerName.toUpperCase()}` : ''} — BEST {best} · RANK #{rank}
        </p>
      {:else if submitState === 'error'}
        <p class="note">COULDN'T SAVE — IS THE API RUNNING?</p>
      {:else if !signedIn}
        <p class="note">SIGN IN TO SAVE YOUR HIGH SCORE</p>
      {/if}
      <div class="gis" bind:this={gisButton}></div>
    </div>

    {#if leaderboard.length > 0}
      <div class="board">
        <p class="board-title">{daily ? `DAILY · ${dailyDay}` : 'LEADERBOARD'}</p>
        <ol>
          {#each leaderboard as entry, i}
            <li>
              <span class="pos">{i + 1}</span>
              <span class="name">{entry.name}</span>
              <span class="pts">{entry.score}</span>
            </li>
          {/each}
        </ol>
      </div>
    {:else if leaderboardError}
      <p class="note">LEADERBOARD UNAVAILABLE</p>
    {/if}
  </div>
</div>

<style>
  .death {
    position: absolute;
    inset: 0;
    z-index: 30;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(30, 4, 4, 0.78);
    backdrop-filter: blur(3px);
    overflow-y: auto;
  }

  .inner {
    text-align: center;
    padding: 24px;
    max-width: 460px;
    width: 100%;
  }

  .wasted {
    font-family: var(--font-display, monospace);
    font-size: clamp(3rem, 9vw, 5.5rem);
    letter-spacing: 0.06em;
    color: #ff5252;
    text-shadow: 4px 4px 0 rgba(0, 0, 0, 0.5);
  }

  .score {
    margin-top: 10px;
    font-family: var(--font-mono, monospace);
    font-size: 1.1rem;
    letter-spacing: 0.15em;
    color: #fff;
  }

  .daily-tag {
    margin-top: 6px;
    font-family: var(--font-mono, monospace);
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    color: #ffd23f;
  }

  .sections-progress {
    margin-top: 12px;
    font-family: var(--font-mono, monospace);
    font-size: 0.72rem;
    letter-spacing: 0.12em;
    color: rgba(255, 255, 255, 0.6);
  }

  .badge-100 {
    margin-top: 14px;
    font-family: var(--font-display, monospace);
    font-size: 0.95rem;
    letter-spacing: 0.1em;
    color: #9ccc65;
    text-shadow: 0 0 12px rgba(156, 204, 101, 0.6);
  }

  .respawn {
    margin-top: 26px;
    font-family: var(--font-display, monospace);
    font-size: 1.3rem;
    letter-spacing: 0.1em;
    color: #fff;
    background: none;
    border: 2px solid #fff;
    padding: 10px 34px;
    cursor: pointer;
    animation: blink 1.4s ease-in-out infinite;
  }

  .respawn:hover {
    color: #ffd23f;
    border-color: #ffd23f;
  }

  .save {
    margin-top: 22px;
    min-height: 48px;
  }

  .note {
    font-family: var(--font-mono, monospace);
    font-size: 0.72rem;
    letter-spacing: 0.12em;
    color: rgba(255, 255, 255, 0.6);
    margin-bottom: 10px;
  }

  .saved {
    color: #9ccc65;
  }

  .gis {
    display: flex;
    justify-content: center;
  }

  .board {
    margin-top: 24px;
    text-align: left;
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid rgba(255, 255, 255, 0.25);
    padding: 14px 18px;
  }

  .board-title {
    font-family: var(--font-mono, monospace);
    font-size: 0.75rem;
    letter-spacing: 0.18em;
    color: #ffd23f;
    margin-bottom: 8px;
    text-align: center;
  }

  ol {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    display: flex;
    gap: 10px;
    font-family: var(--font-mono, monospace);
    font-size: 0.78rem;
    color: rgba(255, 255, 255, 0.85);
    padding: 3px 0;
  }

  .pos {
    width: 2ch;
    color: rgba(255, 255, 255, 0.45);
  }

  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pts {
    color: #ffd23f;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }
</style>
