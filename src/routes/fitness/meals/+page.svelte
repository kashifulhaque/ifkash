<svelte:head>
  <title>Meal Tracker — Kashif</title>
  <meta name="description" content="Snap a photo of your meal and get an estimated nutritional breakdown, logged by day." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { Camera, ImageUp, Trash2, LogOut, Loader, Save, Utensils, Flame, Dumbbell } from 'lucide-svelte';
  import {
    mealsApi,
    setToken,
    loadToken,
    AuthError,
    type Meal,
    type Totals,
    type DaySummary
  } from '$lib/mealsApi';
  // The workout auth helpers reuse the splitter's credential store. We mirror
  // the meals Google token into it on sign-in so workoutApi's calls (which read
  // from that store) authenticate as the same user from the meals page too.
  import { setToken as setSplitterToken, loadToken as loadSplitterToken } from '$lib/splitterApi';
  import { profileApi } from '$lib/profileApi';
  import { workoutApi } from '$lib/workoutApi';
  import { isLocalDev } from '$lib/apiBase';
  import { scheduleTokenRefresh } from '$lib/fitnessAuth';
  import NutritionRings from '$lib/components/NutritionRings.svelte';
  import {
    DEFAULT_PROFILE,
    computeMetrics,
    nutritionTargets,
    type Profile,
    type NutritionTargets
  } from '$lib/fitnessMetrics';
  import {
    dayBurnKcal,
    type SessionSummary,
    type SessionDetail,
    type BodyweightEntry
  } from '$lib/workout';

  const clientId = env.PUBLIC_GOOGLE_CLIENT_ID ?? '';
  const localDev = isLocalDev();

  let signedIn = false;
  let gisButton: HTMLDivElement;

  // local 'YYYY-MM-DD' for the date picker (defaults to today).
  function todayLocal(): string {
    const d = new Date();
    const off = d.getTimezoneOffset();
    return new Date(d.getTime() - off * 60000).toISOString().slice(0, 10);
  }

  let date = todayLocal();
  let meals: Meal[] = [];
  let totals: Totals = { calories: 0, protein_g: 0, carbs_g: 0, fat_g: 0 };

  // Past days with totals, for the history list. `photoUrls` holds object URLs
  // fetched with auth (an <img src> can't carry the bearer token).
  let days: DaySummary[] = [];
  let photoUrls: Record<number, string> = {};

  // Body profile drives the daily nutrition targets the rings fill toward.
  let profile: Profile = { ...DEFAULT_PROFILE };
  $: weightKg = (profile.latest_weight_g ?? 0) / 1000;
  $: targets = (weightKg > 0
    ? nutritionTargets(weightKg, profile)
    : null) as NutritionTargets | null;

  let errorMsg = '';
  let loading = false;
  let analyzing = false;

  // upload form — two inputs so the user can choose camera vs. photo library.
  let cameraInput: HTMLInputElement;
  let uploadInput: HTMLInputElement;
  let hint = '';

  // ---- auth ----------------------------------------------------------------

  let refreshTimer: ReturnType<typeof setTimeout> | undefined;
  function scheduleRefresh() {
    clearTimeout(refreshTimer);
    refreshTimer = scheduleTokenRefresh(loadToken(), clientId, onCredential);
  }

  function onCredential(resp: { credential: string }) {
    setToken(resp.credential);
    // Same Google ID token — mirror it into the splitter store so the shared
    // workoutApi (which reads from there) authenticates from the meals page.
    setSplitterToken(resp.credential);
    signedIn = true;
    scheduleRefresh();
    refresh();
    loadProfile();
    loadWorkoutSummary();
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
    setSplitterToken(null);
    signedIn = false;
    meals = [];
    totals = { calories: 0, protein_g: 0, carbs_g: 0, fat_g: 0 };
    workoutSessionDetails = [];
    sessions = [];
    bodyweights = [];
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
        meals = [];
        setTimeout(renderGoogleButton, 0);
      } else {
        errorMsg = e instanceof Error ? e.message : 'something went wrong';
      }
      return undefined;
    }
  }

  // ---- data ----------------------------------------------------------------

  async function refresh() {
    loading = true;
    const day = await guard(() => mealsApi.list(date));
    loading = false;
    if (day) {
      meals = day.meals;
      totals = day.totals;
      revokePhotos();
      loadPhotos();
    }
    loadDays();
    // The active day changed → today's logged workout burn may differ.
    refreshWorkoutBurn();
  }

  /** Fetch each meal's photo (auth'd) into an object URL for <img>. */
  async function loadPhotos() {
    for (const m of meals) {
      if (m.photo_r2_key && !photoUrls[m.id]) {
        const url = await guard(() => mealsApi.photoUrl(m.id));
        if (url) photoUrls = { ...photoUrls, [m.id]: url };
      }
    }
  }

  function revokePhotos() {
    for (const u of Object.values(photoUrls)) URL.revokeObjectURL(u);
    photoUrls = {};
  }

  /** Recent days (with totals) for the history list. */
  async function loadDays() {
    const d = await guard(() => mealsApi.listDays(14));
    if (d) days = d;
  }

  function pickDay(d: string) {
    if (d === date) return;
    date = d;
    refresh();
  }

  /** Pull the body profile (+ latest bodyweight) so the rings have targets. */
  async function loadProfile() {
    const token = loadToken();
    if (!token) return;
    const p = await guard(() => profileApi.get(token));
    if (p) profile = p;
  }

  // ---- workout summary (powers the day's "burnt" estimate) ------------------
  // Pulled from the workout API so the meals page can answer "how much have I
  // eaten vs. burnt today?" without leaving the meal tracker.

  let sessions: SessionSummary[] = [];
  let bodyweights: BodyweightEntry[] = [];
  // Details for every session row whose date matches the active meal day — used
  // to Sum logged lifting + cardio burn for the whole-day energy estimate.
  let workoutSessionDetails: SessionDetail[] = [];

  async function loadWorkoutSummary() {
    const [s, b] = await Promise.all([
      guard(() => workoutApi.listSessions()),
      guard(() => workoutApi.listBodyweight())
    ]);
    if (s) sessions = s;
    if (b) bodyweights = b;
    await refreshWorkoutBurn();
  }

  /** Fetch the details for every session logged on the active day and stash
   *  them so the reactive `workoutBurn` can total them. */
  async function refreshWorkoutBurn() {
    const todays = sessions.filter((s) => s.date === date);
    const details = await Promise.all(
      todays.map((s) => guard(() => workoutApi.getSession(s.id)))
    );
    workoutSessionDetails = details.filter((d): d is SessionDetail => !!d);
  }

  /** Bodyweight in kg for the active day, falling back to the latest entry and
   *  then the profile's last-known weight so we always have something for the
   *  BMR/TDEE math. */
  $: burnWeightKg = (() => {
    const bwToday = bodyweights.find((b) => b.date === date);
    if (bwToday) return bwToday.weight_g / 1000;
    if (bodyweights.length) return bodyweights[bodyweights.length - 1].weight_g / 1000;
    return (profile.latest_weight_g ?? 0) / 1000;
  })();
  $: metrics = burnWeightKg > 0 ? computeMetrics(burnWeightKg, profile) : null;
  $: workoutBurn = burnWeightKg > 0 ? dayBurnKcal(burnWeightKg, workoutSessionDetails) : 0;
  // Whole-day burn estimate: TDEE (maintenance = BMR × activity) plus the
  // kcal explicitly logged from today's workout sessions. TDEE's activity
  // factor already covers general daily movement; adding the logged workout
  // sessions gives credit for an above-average training day.
  $: totalBurn = (metrics?.tdee ?? 0) + workoutBurn;
  $: eaten = totals.calories ?? 0;
  $: netCalories = Math.round(eaten - totalBurn);
  $: burnPct = totalBurn > 0 ? Math.min(1, eaten / totalBurn) : 0;

  // ---- photo → downscale → analyze ----------------------------------------

  /** Downscale to <= maxEdge and re-encode as JPEG to keep upload + token cost low. */
  function downscale(file: File, maxEdge = 1024, quality = 0.82): Promise<Blob> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      const url = URL.createObjectURL(file);
      img.onload = () => {
        URL.revokeObjectURL(url);
        let { width, height } = img;
        if (width > maxEdge || height > maxEdge) {
          const scale = maxEdge / Math.max(width, height);
          width = Math.round(width * scale);
          height = Math.round(height * scale);
        }
        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext('2d');
        if (!ctx) return reject(new Error('canvas unavailable'));
        ctx.drawImage(img, 0, 0, width, height);
        canvas.toBlob(
          (b) => (b ? resolve(b) : reject(new Error('encode failed'))),
          'image/jpeg',
          quality
        );
      };
      img.onerror = () => {
        URL.revokeObjectURL(url);
        reject(new Error('could not read image'));
      };
      img.src = url;
    });
  }

  async function onPhotoChosen(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    analyzing = true;
    errorMsg = '';
    try {
      const blob = await downscale(file);
      const meal = await guard(() => mealsApi.analyze(blob, hint, date));
      if (meal) {
        hint = '';
        await refresh();
      }
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : 'could not analyze photo';
    } finally {
      analyzing = false;
      input.value = ''; // allow re-selecting the same file
    }
  }

  // ---- edit / delete -------------------------------------------------------

  async function saveMeal(m: Meal) {
    const res = await guard(() =>
      mealsApi.update(m.id, {
        description: m.description ?? '',
        calories: m.calories ?? 0,
        protein_g: m.protein_g ?? 0,
        carbs_g: m.carbs_g ?? 0,
        fat_g: m.fat_g ?? 0
      })
    );
    if (res) await refresh();
  }

  async function deleteMeal(id: number) {
    if (!confirm('Delete this meal?')) return;
    const res = await guard(() => mealsApi.remove(id));
    if (res) await refresh();
  }

  const round = (v: number | null) => (v == null ? 0 : Math.round(v));

  // ---- lifecycle -----------------------------------------------------------

  onMount(async () => {
    if (localDev) {
      // Local dev: the Worker bypasses Google auth (LOCAL_DEV in api/.dev.vars),
      // so skip sign-in entirely and boot straight into the data.
      setToken('local-dev');
      setSplitterToken('local-dev');
      signedIn = true;
      refresh();
      loadProfile();
      loadWorkoutSummary();
      return;
    }
    if (!clientId) return;
    // Load GIS regardless so the silent token-refresh prompt can fire even when
    // we're already signed in (the sign-in button itself stays hidden).
    try {
      await loadGis();
    } catch {
      /* GIS blocked */
    }
    const stored = loadToken();
    if (stored) {
      signedIn = true;
      // Reuse whatever credential the splitter/workout page may already hold
      // so workoutApi authenticates from the meals session too.
      if (!loadSplitterToken()) setSplitterToken(stored);
      scheduleRefresh();
      refresh();
      loadProfile();
      loadWorkoutSummary();
    } else {
      renderGoogleButton();
    }
  });
</script>

<div class="meals">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/fitness">Fitness</a>
      <span class="separator">/</span>
      <span>Meals</span>
    </div>
    <div class="header-top">
      <h1 class="page-title">Meal Tracker 🍽️</h1>
      {#if signedIn}
        <div class="header-actions">
          <button class="ghost-btn" on:click={signOut} title="Sign out">
            <LogOut size={15} /> Sign out
          </button>
        </div>
      {/if}
    </div>
    <p class="page-desc">Snap a meal, get an estimated nutrition breakdown, logged by day.</p>
  </header>

  {#if errorMsg}
    <p class="error">{errorMsg}</p>
  {/if}

  {#if !clientId && !localDev}
    <p class="note">Set <code>PUBLIC_GOOGLE_CLIENT_ID</code> to enable sign-in.</p>
  {:else if !signedIn}
    <div class="gate">
      <p class="note">Sign in with Google to track your meals. Your log is private to your account.</p>
      <div class="gis" bind:this={gisButton}></div>
    </div>
  {:else}
    <!-- capture / upload -->
    <section class="card capture">
      <div class="day-row">
        <label class="lbl">Day
          <input class="in" type="date" bind:value={date} on:change={refresh} />
        </label>
        <span class="totals-pill" title="Day totals">
          {round(totals.protein_g)}g protein · {round(totals.carbs_g)}g carbs ·
          {round(totals.fat_g)}g fat · {round(totals.calories)} kcal
        </span>
      </div>
      <input
        class="in"
        placeholder="Optional hint (e.g. 'large bowl, brown rice')"
        bind:value={hint}
      />
      <input
        bind:this={cameraInput}
        type="file"
        accept="image/*"
        capture="environment"
        class="hidden-file"
        on:change={onPhotoChosen}
      />
      <input
        bind:this={uploadInput}
        type="file"
        accept="image/*"
        class="hidden-file"
        on:change={onPhotoChosen}
      />
      {#if analyzing}
        <button class="primary big" disabled>
          <Loader size={16} class="spin" /> Analyzing…
        </button>
      {:else}
        <div class="capture-btns">
          <button class="primary big" on:click={() => cameraInput?.click()}>
            <Camera size={16} /> Take photo
          </button>
          <button class="primary big" on:click={() => uploadInput?.click()}>
            <ImageUp size={16} /> Upload
          </button>
        </div>
      {/if}
      <p class="disclaimer">Macros are AI estimates — edit any value after analysis.</p>
    </section>

    <!-- daily totals -->
    <section class="card totals">
      <h3 class="panel-title"><Utensils size={15} /> {date} totals</h3>
      {#if targets}
        <NutritionRings {totals} {targets} />
      {:else}
        <div class="totals-grid">
          <div class="tot cal"><span class="tval">{round(totals.calories)}</span><span class="tlabel">kcal</span></div>
          <div class="tot"><span class="tval">{round(totals.protein_g)}g</span><span class="tlabel">protein</span></div>
          <div class="tot"><span class="tval">{round(totals.carbs_g)}g</span><span class="tlabel">carbs</span></div>
          <div class="tot"><span class="tval">{round(totals.fat_g)}g</span><span class="tlabel">fat</span></div>
        </div>
        <p class="disclaimer">Log a bodyweight on the Workout page to see goal rings.</p>
      {/if}
    </section>

    <!-- Energy balance — eaten vs whole-day burn (TDEE + today's logged workout). -->
    {#if metrics}
      <section class="card energy">
        <h3 class="panel-title"><Flame size={15} /> Energy balance</h3>
        <div class="energy-grid">
          <div class="energy-cell in">
            <span class="e-val">{round(totals.calories)}</span>
            <span class="e-label">Eaten</span>
            <span class="e-sub">kcal today</span>
          </div>
          <div class="energy-cell out tdee">
            <span class="e-val">{metrics.tdee}</span>
            <span class="e-label">TDEE</span>
            <span class="e-sub">maintenance · BMR {metrics.bmr}</span>
          </div>
          <div class="energy-cell out">
            <span class="e-val">{workoutBurn}</span>
            <span class="e-label">Workout</span>
            <span class="e-sub">{workoutSessionDetails.length} {workoutSessionDetails.length === 1 ? 'session' : 'sessions'} logged</span>
          </div>
          <div class="energy-cell sum" class:surplus={netCalories > 0} class:deficit={netCalories < 0}>
            <span class="e-val">{netCalories > 0 ? '+' : ''}{netCalories}</span>
            <span class="e-label">{netCalories === 0 ? 'Balanced' : netCalories > 0 ? 'Surplus' : 'Deficit'}</span>
            <span class="e-sub">of {round(totalBurn)}</span>
          </div>
        </div>
        <div class="balance-bar" title="{round(eaten)} eaten / {round(totalBurn)} burnt">
          <div class="balance-fill" style="width: {Math.round(burnPct * 100)}%"></div>
        </div>
        <p class="energy-note">
          Whole-day estimate at {burnWeightKg.toFixed(1)} kg — <strong>{round(totals.calories)}</strong>
          eaten vs <strong>{round(totalBurn)}</strong> burnt
          (TDEE {metrics.tdee}{workoutBurn > 0 ? ` + ${workoutBurn} workout` : ''}).
          <a href="/fitness/workout" class="energy-link">Log a workout <Dumbbell size={12} /></a>
        </p>
      </section>
    {/if}

    <!-- meal log -->
    {#if loading}
      <p class="note">Loading…</p>
    {:else if !meals.length}
      <p class="note">No meals logged for this day yet.</p>
    {:else}
      <ul class="meal-list">
        {#each meals as m (m.id)}
          <li class="card meal">
            {#if photoUrls[m.id]}
              <img class="meal-photo" src={photoUrls[m.id]} alt={m.description ?? 'meal photo'} />
            {:else if m.photo_r2_key}
              <div class="meal-photo placeholder"><Camera size={18} /></div>
            {/if}
            <div class="meal-body">
              <div class="meal-head">
                <input class="in desc" bind:value={m.description} placeholder="Description" />
                <button class="icon-btn" title="Delete" on:click={() => deleteMeal(m.id)}>
                  <Trash2 size={14} />
                </button>
              </div>
              <div class="macro-row">
                <label class="lbl">kcal<input class="in tiny" type="number" min="0" bind:value={m.calories} /></label>
                <label class="lbl">protein<input class="in tiny" type="number" min="0" bind:value={m.protein_g} /></label>
                <label class="lbl">carbs<input class="in tiny" type="number" min="0" bind:value={m.carbs_g} /></label>
                <label class="lbl">fat<input class="in tiny" type="number" min="0" bind:value={m.fat_g} /></label>
                <button class="primary small" on:click={() => saveMeal(m)}><Save size={13} /> Save</button>
              </div>
              {#if m.items?.length}
                <p class="items">{m.items.map((it) => it.name).join(' · ')}</p>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {/if}

    <!-- Past days — tap to jump back to that day's log. -->
    {#if days.length > 1}
      <section class="card history">
        <h3 class="panel-title"><Utensils size={15} /> History</h3>
        <ul class="day-list">
          {#each days as d}
            <li>
              <button
                class="day-row"
                class:active={d.eaten_on === date}
                on:click={() => pickDay(d.eaten_on)}
              >
                <span class="day-date">{d.eaten_on}</span>
                <span class="day-cal">{round(d.calories)} kcal</span>
                <span class="day-count">{d.count} {d.count === 1 ? 'meal' : 'meals'}</span>
              </button>
            </li>
          {/each}
        </ul>
      </section>
    {/if}
  {/if}
</div>

<style>
  .meals {
    max-width: 48rem;
    margin: 0 auto;
    padding: 24px 20px 80px;
    font-family: var(--font-body);
    color: var(--ink);
  }
  .page-header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-bottom: 1.25rem;
    margin-bottom: 18px;
    border-bottom: 1px solid var(--border);
  }
  .breadcrumb { font-size: 0.875rem; color: var(--text-tertiary); }
  .breadcrumb a { color: var(--text-tertiary); transition: color 0.15s; border-bottom: none; }
  .breadcrumb a:hover { color: var(--text-primary); }
  .separator { margin: 0 0.5rem; }

  .header-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.625rem;
  }
  .page-title {
    font-size: 2.25rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    margin: 0;
  }
  .page-desc {
    font-size: 0.95rem;
    line-height: 1.55;
    color: var(--text-secondary);
    margin: 0;
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
    cursor: var(--cursor-pointer);
  }
  .ghost-btn:hover { color: var(--text-primary); border-color: var(--border-strong); }
  .note { color: var(--ink-mute); font-size: 0.9rem; }
  .error { color: #e06c6c; font-size: 0.88rem; }
  .disclaimer { color: var(--ink-mute); font-size: 0.78rem; margin: 4px 0 0; }
  code { font-family: var(--font-mono); font-size: 0.85em; }

  .gate { padding: 32px 0; }
  .gis { margin-top: 14px; }

  .panel-title {
    display: flex; align-items: center; gap: 7px;
    font-family: var(--font-mono);
    font-size: 0.72rem; letter-spacing: 0.12em; text-transform: uppercase;
    color: var(--ink-soft);
    margin: 0 0 12px;
  }

  .card { border: 1px solid var(--rule-soft); padding: 16px; margin-bottom: 16px; }
  .capture { display: flex; flex-direction: column; gap: 10px; }
  .day-row { display: flex; gap: 10px; align-items: flex-end; flex-wrap: wrap; }
  .totals-pill {
    font-family: var(--font-mono); font-size: 0.72rem; color: var(--ink-soft);
    border: 1px solid var(--rule-soft); border-radius: 999px; padding: 6px 12px;
    align-self: center;
  }
  .hidden-file { display: none; }
  .capture-btns { display: flex; gap: 10px; }
  .capture-btns .big { flex: 1; }

  /* totals */
  .totals-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }
  .tot { display: flex; flex-direction: column; align-items: center; gap: 2px; border: 1px solid var(--rule-soft); padding: 12px 6px; }
  .tot.cal { border-color: var(--blueprint); }
  .tval { font-family: var(--font-mono); font-size: 1.15rem; color: var(--ink); }
  .tlabel { font-family: var(--font-mono); font-size: 0.62rem; letter-spacing: 0.08em; text-transform: uppercase; color: var(--ink-mute); }

  /* energy balance card */
  .energy-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }
  .energy-cell {
    display: flex; flex-direction: column; align-items: center; gap: 2px;
    border: 1px solid var(--rule-soft); padding: 12px 6px;
  }
  .energy-cell.tdee { border-color: var(--rule-soft); }
  .energy-cell.sum { border-color: var(--blueprint); background: var(--blueprint-tint, rgba(110,168,254,0.08)); }
  .energy-cell.deficit { border-color: #5bb98c; }
  .energy-cell.surplus { border-color: #e06c6c; }
  .e-val { font-family: var(--font-mono); font-size: 1.15rem; color: var(--ink); }
  .energy-cell.deficit .e-val { color: #5bb98c; }
  .energy-cell.surplus .e-val { color: #e06c6c; }
  .e-label { font-family: var(--font-mono); font-size: 0.62rem; letter-spacing: 0.08em; text-transform: uppercase; color: var(--ink-mute); }
  .e-sub { font-family: var(--font-mono); font-size: 0.6rem; color: var(--ink-mute); text-align: center; }

  .balance-bar {
    margin: 12px 0 0; height: 8px; border: 1px solid var(--rule-soft);
    background: var(--bg); overflow: hidden;
  }
  .balance-fill { height: 100%; background: var(--blueprint); transition: width 0.4s ease; }

  .energy-note { font-size: 0.82rem; line-height: 1.5; color: var(--ink-mute); margin: 10px 0 0; }
  .energy-note strong { color: var(--ink); }
  .energy-link {
    display: inline-flex; align-items: center; gap: 4px; margin-left: 6px;
    color: var(--blueprint); border-bottom: 1px solid transparent; font-family: var(--font-mono); font-size: 0.72rem;
  }
  .energy-link:hover { border-bottom-color: var(--blueprint); }

  @media (max-width: 520px) {
    .energy-grid { grid-template-columns: repeat(2, 1fr); }
  }

  /* meal list */
  .meal-list { list-style: none; margin: 0; padding: 0; }
  .meal { display: flex; gap: 14px; align-items: flex-start; }
  .meal-photo {
    width: 84px; height: 84px; flex-shrink: 0;
    object-fit: cover; border-radius: 6px; border: 1px solid var(--rule-soft);
  }
  .meal-photo.placeholder {
    display: flex; align-items: center; justify-content: center;
    color: var(--ink-mute); background: var(--bg);
  }
  .meal-body { flex: 1; min-width: 0; }
  .meal-head { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
  .desc { flex: 1; min-width: 0; }
  .macro-row { display: flex; gap: 10px; flex-wrap: wrap; align-items: flex-end; }
  .items { font-family: var(--font-mono); font-size: 0.72rem; color: var(--ink-mute); margin: 10px 0 0; }

  /* history */
  .day-list { list-style: none; margin: 0; padding: 0; }
  .day-row {
    width: 100%; display: flex; align-items: baseline; gap: 12px;
    background: none; border: none; border-bottom: 1px solid var(--rule-soft);
    padding: 10px 4px; cursor: var(--cursor-pointer); text-align: left;
    color: var(--ink); font-family: var(--font-body);
  }
  .day-list li:last-child .day-row { border-bottom: none; }
  .day-row:hover { background: var(--bg); }
  .day-row.active { color: var(--blueprint); }
  .day-date { font-family: var(--font-mono); font-size: 0.82rem; flex: 1; }
  .day-cal { font-family: var(--font-mono); font-size: 0.82rem; }
  .day-count { font-family: var(--font-mono); font-size: 0.68rem; color: var(--ink-mute); }

  /* form controls */
  .lbl { display: flex; flex-direction: column; gap: 4px; font-size: 0.72rem; color: var(--ink-soft); }
  .in {
    background: var(--bg); color: var(--ink);
    border: 1px solid var(--rule-soft); padding: 8px 10px; font-family: var(--font-body); font-size: 0.9rem;
  }
  .in:focus { outline: none; border-color: var(--blueprint); }
  .tiny { width: 80px; }

  button.primary, button.ghost {
    display: inline-flex; align-items: center; gap: 7px; cursor: var(--cursor-pointer);
    font-family: var(--font-mono); font-size: 0.72rem; letter-spacing: 0.08em; text-transform: uppercase;
    padding: 8px 14px; border: 1px solid var(--blueprint); background: transparent; color: var(--blueprint);
    transition: background 0.12s, color 0.12s;
  }
  button.primary:hover:not(:disabled) { background: var(--blueprint); color: var(--bg); }
  button.primary:disabled { opacity: 0.4; cursor: not-allowed; }
  button.primary.big { justify-content: center; padding: 12px; font-size: 0.78rem; }
  button.primary.small { padding: 6px 10px; font-size: 0.66rem; }
  button.ghost { border-color: var(--rule-soft); color: var(--ink-soft); }
  button.ghost:hover { color: var(--ink); }
  .icon-btn { background: none; border: 1px solid var(--rule-soft); color: var(--ink-mute); cursor: var(--cursor-pointer); padding: 7px; display: inline-flex; }
  .icon-btn:hover { color: #e06c6c; border-color: #e06c6c; }

  :global(.spin) { animation: spin 0.9s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  @media (max-width: 520px) {
    .meals { padding: 20px 14px 64px; }
    .totals-grid { grid-template-columns: repeat(2, 1fr); }
    .capture-btns { flex-direction: column; }
    .meal { gap: 10px; }
    .meal-photo { width: 64px; height: 64px; }
    .tiny { width: 64px; }
    .page-title { font-size: 1.75rem; }
  }
</style>
