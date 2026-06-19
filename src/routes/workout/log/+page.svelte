<svelte:head>
  <title>Workout Tracker — Kashif</title>
  <meta name="description" content="Log workouts and bodyweight, synced to your account." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { Plus, Trash2, LogOut, Dumbbell, Scale } from 'lucide-svelte';
  import { setToken, loadToken, AuthError } from '$lib/splitterApi';
  import { workoutApi, type ExercisePayload } from '$lib/workoutApi';
  import {
    DAY_TEMPLATES,
    setsFromScheme,
    kgToGrams,
    gramsToKg,
    weeklyAverages,
    type DayLabel,
    type SessionSummary,
    type SessionDetail,
    type BodyweightEntry,
    type WeeklyAverage
  } from '$lib/workout';

  const clientId = env.PUBLIC_GOOGLE_CLIENT_ID ?? '';

  let signedIn = false;
  let gisButton: HTMLDivElement;
  let errorMsg = '';

  function today(): string {
    return new Date().toISOString().slice(0, 10);
  }

  // ---- session builder -----------------------------------------------------

  type SetRow = { weight: string; reps: string };
  type ExerciseRow = { name: string; sets: SetRow[] };

  const DAY_OPTIONS: (DayLabel | 'Custom')[] = ['Push', 'Pull', 'Legs', 'Custom'];
  let dayLabel: DayLabel | 'Custom' = 'Push';
  let sessionDate = today();
  let sessionNotes = '';
  let exercises: ExerciseRow[] = [];

  function blankSet(): SetRow {
    return { weight: '', reps: '' };
  }

  /** Build editable rows from a day template (or a single blank row for Custom). */
  function applyTemplate(label: DayLabel | 'Custom') {
    dayLabel = label;
    if (label === 'Custom') {
      exercises = [{ name: '', sets: [blankSet()] }];
      return;
    }
    exercises = DAY_TEMPLATES[label].map((ex) => ({
      name: ex.name,
      sets: Array.from({ length: setsFromScheme(ex.scheme) }, blankSet)
    }));
  }

  function addExercise() {
    exercises = [...exercises, { name: '', sets: [blankSet()] }];
  }
  function removeExercise(i: number) {
    exercises = exercises.filter((_, idx) => idx !== i);
  }
  function addSet(i: number) {
    exercises[i].sets = [...exercises[i].sets, blankSet()];
    exercises = exercises;
  }
  function removeSet(i: number, j: number) {
    exercises[i].sets = exercises[i].sets.filter((_, idx) => idx !== j);
    exercises = exercises;
  }

  let savingSession = false;
  async function saveSession() {
    const payload: ExercisePayload[] = exercises
      .map((ex) => ({
        exercise: ex.name.trim(),
        // Keep only sets where at least reps or weight was entered.
        sets: ex.sets
          .filter((s) => s.reps.trim() !== '' || s.weight.trim() !== '')
          .map((s) => ({
            reps: Math.max(0, parseInt(s.reps, 10) || 0),
            weight_g: kgToGrams(s.weight)
          }))
      }))
      .filter((ex) => ex.exercise !== '' && ex.sets.length > 0);

    if (!sessionDate) {
      errorMsg = 'pick a date';
      return;
    }
    if (payload.length === 0) {
      errorMsg = 'log at least one exercise with a set';
      return;
    }

    savingSession = true;
    const ok = await guard(() =>
      workoutApi.createSession({
        day_label: dayLabel === 'Custom' ? '' : dayLabel,
        date: sessionDate,
        notes: sessionNotes.trim(),
        exercises: payload
      })
    );
    savingSession = false;
    if (ok) {
      sessionNotes = '';
      applyTemplate(dayLabel);
      await refreshSessions();
    }
  }

  // ---- bodyweight ----------------------------------------------------------

  let bwWeight = '';
  let bwDate = today();
  let bodyweight: BodyweightEntry[] = [];
  let weekly: WeeklyAverage[] = [];
  let savingBw = false;

  $: weekly = weeklyAverages(bodyweight);

  async function saveBodyweight() {
    const grams = kgToGrams(bwWeight);
    if (grams <= 0) {
      errorMsg = 'enter a valid weight';
      return;
    }
    savingBw = true;
    const ok = await guard(() => workoutApi.addBodyweight(bwDate, grams));
    savingBw = false;
    if (ok) {
      bwWeight = '';
      await refreshBodyweight();
    }
  }

  async function deleteBodyweight(id: number) {
    const ok = await guard(() => workoutApi.deleteBodyweight(id));
    if (ok) await refreshBodyweight();
  }

  // Sparkline geometry for the weekly-average trend.
  $: sparkPoints = (() => {
    if (weekly.length < 2) return '';
    const w = 320;
    const h = 64;
    const pad = 6;
    const vals = weekly.map((p) => p.avgKg);
    const min = Math.min(...vals);
    const max = Math.max(...vals);
    const span = max - min || 1;
    return weekly
      .map((p, i) => {
        const x = pad + (i / (weekly.length - 1)) * (w - 2 * pad);
        const y = pad + (1 - (p.avgKg - min) / span) * (h - 2 * pad);
        return `${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join(' ');
  })();

  // ---- history -------------------------------------------------------------

  let sessions: SessionSummary[] = [];
  let openId: number | null = null;
  let openDetail: SessionDetail | null = null;
  let loadingDetail = false;

  async function refreshSessions() {
    const list = await guard(() => workoutApi.listSessions());
    if (list) sessions = list;
  }

  async function refreshBodyweight() {
    const list = await guard(() => workoutApi.listBodyweight());
    if (list) bodyweight = list;
  }

  async function toggleSession(id: number) {
    if (openId === id) {
      openId = null;
      openDetail = null;
      return;
    }
    openId = id;
    openDetail = null;
    loadingDetail = true;
    const d = await guard(() => workoutApi.getSession(id));
    loadingDetail = false;
    if (d) openDetail = d;
  }

  async function deleteSession(id: number) {
    const ok = await guard(() => workoutApi.deleteSession(id));
    if (ok) {
      if (openId === id) {
        openId = null;
        openDetail = null;
      }
      await refreshSessions();
    }
  }

  /** Group a detail's flat set rows by exercise, preserving insertion order. */
  function groupSets(detail: SessionDetail) {
    const groups: { exercise: string; sets: { reps: number; weight_g: number }[] }[] = [];
    for (const s of detail.sets) {
      let g = groups.find((x) => x.exercise === s.exercise);
      if (!g) {
        g = { exercise: s.exercise, sets: [] };
        groups.push(g);
      }
      g.sets.push({ reps: s.reps, weight_g: s.weight_g });
    }
    return groups;
  }

  function setLabel(reps: number, weight_g: number): string {
    return weight_g > 0 ? `${gramsToKg(weight_g)}kg × ${reps}` : `BW × ${reps}`;
  }

  // ---- auth (mirrors the splitter's Google Identity flow) -------------------

  function onCredential(resp: { credential: string }) {
    setToken(resp.credential);
    signedIn = true;
    loadAll();
  }

  function renderGoogleButton() {
    const google = (window as any).google;
    if (!google?.accounts?.id || !gisButton) return;
    google.accounts.id.initialize({ client_id: clientId, callback: onCredential });
    gisButton.innerHTML = '';
    google.accounts.id.renderButton(gisButton, {
      theme: 'filled_black',
      size: 'large',
      text: 'signin_with'
    });
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

  function signOut() {
    setToken(null);
    signedIn = false;
    sessions = [];
    bodyweight = [];
    openId = null;
    openDetail = null;
    setTimeout(renderGoogleButton, 0);
  }

  /** Run an API call, dropping the session and re-showing sign-in on 401. */
  async function guard<T>(fn: () => Promise<T>): Promise<T | undefined> {
    errorMsg = '';
    try {
      return await fn();
    } catch (e) {
      if (e instanceof AuthError) {
        signedIn = false;
        sessions = [];
        bodyweight = [];
        setTimeout(renderGoogleButton, 0);
      } else {
        errorMsg = e instanceof Error ? e.message : 'something went wrong';
      }
      return undefined;
    }
  }

  async function loadAll() {
    await Promise.all([refreshSessions(), refreshBodyweight()]);
  }

  onMount(async () => {
    applyTemplate('Push');
    await loadGis();
    if (loadToken()) {
      signedIn = true;
      await loadAll();
    } else {
      renderGoogleButton();
    }
  });
</script>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/">Home</a>
      <span class="separator">/</span>
      <a href="/workout">Workout</a>
      <span class="separator">/</span>
      <span>Tracker</span>
    </div>
    <div class="header-top">
      <h1 class="page-title">Tracker 📓</h1>
      {#if signedIn}
        <button class="ghost-btn" on:click={signOut} title="Sign out">
          <LogOut size={16} /> Sign out
        </button>
      {/if}
    </div>
    <p class="page-desc">
      Log your sets, reps and weight per session, plus a bodyweight trend. Synced to your
      Google account so it follows you from the gym to your laptop.
    </p>
  </header>

  {#if errorMsg}
    <div class="error-banner">{errorMsg}</div>
  {/if}

  {#if !signedIn}
    <div class="signin-card">
      <p>Sign in to log and sync your workouts.</p>
      <div bind:this={gisButton}></div>
    </div>
  {:else}
    <section class="content">
      <!-- Log a session -->
      <div class="card">
        <h2><Dumbbell size={18} /> Log a session</h2>

        <div class="form-row">
          <label>
            Day
            <div class="day-tabs">
              {#each DAY_OPTIONS as label}
                <button
                  class="day-tab"
                  class:active={dayLabel === label}
                  on:click={() => applyTemplate(label)}
                >
                  {label}
                </button>
              {/each}
            </div>
          </label>
          <label>
            Date
            <input type="date" bind:value={sessionDate} />
          </label>
        </div>

        {#each exercises as ex, i}
          <div class="exercise-block">
            <div class="exercise-head">
              <input
                class="exercise-name"
                placeholder="Exercise name"
                bind:value={ex.name}
              />
              <button class="icon-btn" on:click={() => removeExercise(i)} title="Remove exercise">
                <Trash2 size={15} />
              </button>
            </div>
            <div class="sets">
              {#each ex.sets as s, j}
                <div class="set-row">
                  <span class="set-num">{j + 1}</span>
                  <input
                    class="num"
                    type="number"
                    inputmode="decimal"
                    min="0"
                    step="0.5"
                    placeholder="kg"
                    bind:value={s.weight}
                  />
                  <span class="times">×</span>
                  <input
                    class="num"
                    type="number"
                    inputmode="numeric"
                    min="0"
                    placeholder="reps"
                    bind:value={s.reps}
                  />
                  <button class="icon-btn" on:click={() => removeSet(i, j)} title="Remove set">
                    <Trash2 size={14} />
                  </button>
                </div>
              {/each}
              <button class="text-btn" on:click={() => addSet(i)}>
                <Plus size={14} /> set
              </button>
            </div>
          </div>
        {/each}

        <button class="text-btn add-ex" on:click={addExercise}>
          <Plus size={15} /> exercise
        </button>

        <textarea
          class="notes"
          placeholder="Notes (optional) — how it felt, PRs, etc."
          bind:value={sessionNotes}
          rows="2"
        ></textarea>

        <button class="primary-btn" on:click={saveSession} disabled={savingSession}>
          {savingSession ? 'Saving…' : 'Save session'}
        </button>
      </div>

      <!-- Bodyweight -->
      <div class="card">
        <h2><Scale size={18} /> Bodyweight</h2>
        <div class="form-row">
          <label>
            Weight (kg)
            <input
              type="number"
              inputmode="decimal"
              min="0"
              step="0.1"
              placeholder="e.g. 87.5"
              bind:value={bwWeight}
            />
          </label>
          <label>
            Date
            <input type="date" bind:value={bwDate} />
          </label>
          <button class="primary-btn inline" on:click={saveBodyweight} disabled={savingBw}>
            {savingBw ? 'Saving…' : 'Add'}
          </button>
        </div>

        {#if weekly.length}
          {#if sparkPoints}
            <svg class="spark" viewBox="0 0 320 64" preserveAspectRatio="none">
              <polyline points={sparkPoints} />
            </svg>
          {/if}
          <p class="hint">Weekly averages (track this, ignore daily swings):</p>
          <div class="weekly-list">
            {#each [...weekly].reverse() as w}
              <div class="weekly-row">
                <span class="weekly-week">week of {w.weekStart}</span>
                <span class="weekly-avg">{w.avgKg} kg</span>
                <span class="weekly-count">{w.count} {w.count === 1 ? 'entry' : 'entries'}</span>
              </div>
            {/each}
          </div>
          <details class="raw">
            <summary>All entries</summary>
            <div class="weekly-list">
              {#each [...bodyweight].reverse() as e}
                <div class="weekly-row">
                  <span class="weekly-week">{e.date}</span>
                  <span class="weekly-avg">{gramsToKg(e.weight_g)} kg</span>
                  <button class="icon-btn" on:click={() => deleteBodyweight(e.id)} title="Delete">
                    <Trash2 size={14} />
                  </button>
                </div>
              {/each}
            </div>
          </details>
        {:else}
          <p class="hint">No bodyweight entries yet.</p>
        {/if}
      </div>

      <!-- History -->
      <div class="card">
        <h2>History</h2>
        {#if sessions.length === 0}
          <p class="hint">No sessions logged yet.</p>
        {:else}
          <div class="history-list">
            {#each sessions as s}
              <div class="history-item">
                <button class="history-head" on:click={() => toggleSession(s.id)}>
                  <span class="hist-date">{s.date}</span>
                  {#if s.day_label}<span class="hist-day">{s.day_label}</span>{/if}
                  {#if s.notes}<span class="hist-notes">{s.notes}</span>{/if}
                </button>
                <button class="icon-btn" on:click={() => deleteSession(s.id)} title="Delete session">
                  <Trash2 size={15} />
                </button>
                {#if openId === s.id}
                  <div class="history-detail">
                    {#if loadingDetail}
                      <p class="hint">Loading…</p>
                    {:else if openDetail}
                      {#each groupSets(openDetail) as g}
                        <div class="detail-ex">
                          <span class="detail-ex-name">{g.exercise}</span>
                          <div class="detail-sets">
                            {#each g.sets as st}
                              <span class="set-badge">{setLabel(st.reps, st.weight_g)}</span>
                            {/each}
                          </div>
                        </div>
                      {/each}
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </section>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    max-width: 48rem;
    animation: fade-up var(--dur-base) var(--ease-out-quart);
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid var(--border);
  }

  .breadcrumb {
    font-size: 0.875rem;
    color: var(--text-tertiary);
  }
  .breadcrumb a {
    color: var(--text-tertiary);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }
  .breadcrumb a:hover {
    color: var(--text-primary);
  }
  .separator {
    margin: 0 0.5rem;
  }

  .header-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .page-title {
    font-size: 2.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    margin: 0;
  }

  .page-desc {
    font-size: 1.0625rem;
    line-height: 1.6;
    color: var(--text-secondary);
    margin: 0;
  }

  .error-banner {
    padding: 0.75rem 1rem;
    border: 1px solid #c0392b;
    background: rgba(192, 57, 43, 0.1);
    border-radius: 0.5rem;
    color: #e74c3c;
    font-size: 0.875rem;
  }

  .signin-card {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    align-items: flex-start;
    padding: 2rem;
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    background: var(--surface-raised);
  }
  .signin-card p {
    color: var(--text-secondary);
    margin: 0;
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.5rem;
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    background: var(--surface-raised);
  }

  .card h2 {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .form-row {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    align-items: flex-end;
  }
  .form-row label {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    font-size: 0.8125rem;
    color: var(--text-tertiary);
    flex: 1;
    min-width: 9rem;
  }

  input,
  textarea {
    font-family: inherit;
    font-size: 0.9375rem;
    color: var(--text-primary);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    padding: 0.5rem 0.625rem;
    width: 100%;
  }
  input:focus,
  textarea:focus {
    outline: none;
    border-color: var(--border-strong);
  }

  .day-tabs {
    display: flex;
    gap: 0.375rem;
    flex-wrap: wrap;
  }
  .day-tab {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    transition: all var(--dur-instant) var(--ease-out-quart);
  }
  .day-tab.active {
    color: var(--text-primary);
    border-color: var(--border-strong);
    background: var(--surface-sunken);
  }

  .exercise-block {
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
    padding: 0.875rem;
    border: 1px solid var(--border-subtle);
    border-radius: 0.5rem;
  }
  .exercise-head {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }
  .exercise-name {
    font-weight: 600;
  }

  .sets {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .set-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .set-num {
    width: 1.25rem;
    font-family: 'SF Mono', Monaco, monospace;
    font-size: 0.75rem;
    color: var(--text-faint);
    flex-shrink: 0;
  }
  .num {
    width: 5rem;
    text-align: center;
  }
  .times {
    color: var(--text-faint);
  }

  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.375rem;
    color: var(--text-faint);
    background: transparent;
    border: none;
    cursor: pointer;
    border-radius: 0.25rem;
    transition: color var(--dur-instant) var(--ease-out-quart);
  }
  .icon-btn:hover {
    color: #e74c3c;
  }

  .text-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0.25rem 0;
    align-self: flex-start;
  }
  .text-btn:hover {
    color: var(--text-primary);
  }
  .add-ex {
    border: 1px dashed var(--border);
    border-radius: 0.375rem;
    padding: 0.5rem 0.75rem;
  }

  .notes {
    resize: vertical;
  }

  .primary-btn {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--bg);
    background: var(--text-primary);
    border: none;
    border-radius: 0.375rem;
    padding: 0.625rem 1.25rem;
    cursor: pointer;
    align-self: flex-start;
    transition: opacity var(--dur-instant) var(--ease-out-quart);
  }
  .primary-btn:hover {
    opacity: 0.85;
  }
  .primary-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .primary-btn.inline {
    align-self: flex-end;
  }

  .ghost-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.8125rem;
    color: var(--text-tertiary);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    padding: 0.4rem 0.75rem;
    cursor: pointer;
  }
  .ghost-btn:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .spark {
    width: 100%;
    height: 64px;
  }
  .spark polyline {
    fill: none;
    stroke: var(--blueprint, #6ea8fe);
    stroke-width: 2;
    vector-effect: non-scaling-stroke;
  }

  .hint {
    font-size: 0.8125rem;
    color: var(--text-tertiary);
    margin: 0;
  }

  .weekly-list {
    display: flex;
    flex-direction: column;
  }
  .weekly-row {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 0.875rem;
  }
  .weekly-row:last-child {
    border-bottom: none;
  }
  .weekly-week {
    color: var(--text-secondary);
    flex: 1;
  }
  .weekly-avg {
    font-family: 'SF Mono', Monaco, monospace;
    font-weight: 600;
    color: var(--text-primary);
  }
  .weekly-count {
    color: var(--text-faint);
    font-size: 0.75rem;
  }

  .raw summary {
    font-size: 0.8125rem;
    color: var(--text-tertiary);
    cursor: pointer;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .history-item {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.5rem 0.75rem;
    align-items: center;
    padding: 0.75rem;
    border: 1px solid var(--border-subtle);
    border-radius: 0.5rem;
  }
  .history-head {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    flex-wrap: wrap;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    padding: 0;
  }
  .hist-date {
    font-family: 'SF Mono', Monaco, monospace;
    font-size: 0.875rem;
    color: var(--text-primary);
  }
  .hist-day {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--blueprint, #6ea8fe);
  }
  .hist-notes {
    font-size: 0.8125rem;
    color: var(--text-tertiary);
  }

  .history-detail {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border-subtle);
  }
  .detail-ex {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }
  .detail-ex-name {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }
  .detail-sets {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
  }
  .set-badge {
    font-family: 'SF Mono', Monaco, monospace;
    font-size: 0.75rem;
    color: var(--text-primary);
    background: var(--surface-sunken);
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
  }

  @keyframes fade-up {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  @media (max-width: 768px) {
    .page-title {
      font-size: 2rem;
    }
    .num {
      width: 4rem;
    }
  }
</style>
