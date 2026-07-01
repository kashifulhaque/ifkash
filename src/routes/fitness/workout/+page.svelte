<svelte:head>
  <title>Workout — Kashif</title>
  <meta name="description" content="A 5–6 day push/pull/legs program with cardio mixed in — built to gain muscle while cutting." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { LogOut, Check, Loader, Plus, Trash2 } from 'lucide-svelte';
  import { setToken, loadToken, AuthError } from '$lib/splitterApi';
  import { scheduleTokenRefresh } from '$lib/fitnessAuth';
  import { workoutApi, type ExercisePayload, type CardioPayload } from '$lib/workoutApi';
  import { profileApi } from '$lib/profileApi';
  import {
    DEFAULT_PROFILE,
    computeMetrics,
    nutritionTargets,
    metKcal,
    strengthKcal,
    GOAL_LABELS,
    ACTIVITY_OPTIONS,
    type Profile,
    type Goal
  } from '$lib/fitnessMetrics';
  import {
    DAY_TEMPLATES,
    setsFromScheme,
    kgToGrams,
    gramsToKg,
    weeklyAverages,
    CARDIO_OPTIONS,
    CARDIO_DEFAULTS,
    cardioMet,
    type DayLabel,
    type SessionSummary,
    type SessionDetail,
    type BodyweightEntry,
    type WeeklyAverage
  } from '$lib/workout';

  type Focus = DayLabel;

  const clientId = env.PUBLIC_GOOGLE_CLIENT_ID ?? '';

  const cardio: Record<Focus, string> = {
    Push: '15 min cycle',
    Pull: '15 min crosstrainer',
    Legs: '20 min treadmill incline walk (incline 8–10%)'
  };

  const split: { day: string; focus: Focus | 'Optional'; detail: string }[] = [
    { day: 'Day 1', focus: 'Push', detail: 'chest / shoulders / triceps' },
    { day: 'Day 2', focus: 'Pull', detail: 'back / biceps' },
    { day: 'Day 3', focus: 'Legs', detail: 'quads / hamstrings / abs' },
    { day: 'Day 4', focus: 'Push', detail: 'chest / shoulders / triceps' },
    { day: 'Day 5', focus: 'Pull', detail: 'back / biceps' },
    { day: 'Day 6', focus: 'Optional', detail: 'Legs or full body + extra cardio' }
  ];

  // ---- state ---------------------------------------------------------------

  let signedIn = false;
  let gisButton: HTMLDivElement;
  let errorMsg = '';

  let active: Focus = 'Push';
  const FOCUSES: Focus[] = ['Push', 'Pull', 'Legs'];
  let expanded: Record<string, boolean> = {};

  function today(): string {
    const d = new Date();
    const off = d.getTimezoneOffset();
    return new Date(d.getTime() - off * 60000).toISOString().slice(0, 10);
  }
  let sessionDate = today();

  // bodyweight (kg). `suggested` = prefilled from a prior day; not committed
  // until the user edits it.
  let bw = '';
  let bwSuggested = true;
  let lastSavedBw = '';

  // A set's `suggested` flag means it was prefilled from a *previous* day as a
  // hint — it is excluded from saves until the user actually edits it.
  type SetRow = { weight: string; reps: string; suggested: boolean };
  type ExerciseRow = { name: string; scheme: string; sets: SetRow[] };
  let exercises: ExerciseRow[] = [];

  // Cardio bouts logged for the session. `kcalEdited` tracks whether the user
  // has overridden the auto-estimate: while false, kcal follows the live
  // MET-based estimate (kind × minutes × bodyweight); once typed, we stop.
  type CardioRow = { kind: string; minutes: string; kcal: string; kcalEdited: boolean };
  let cardioRows: CardioRow[] = [];

  let saveStatus: 'idle' | 'saving' | 'saved' | 'error' = 'idle';

  // ---- body profile + live metrics -----------------------------------------

  let profile: Profile = { ...DEFAULT_PROFILE };

  // The weight to compute metrics from: whatever is in the bodyweight input,
  // falling back to the latest logged entry. Updates live as the user types.
  $: metricKg = (() => {
    const typed = parseFloat(str(bw));
    if (Number.isFinite(typed) && typed > 0) return typed;
    return (profile.latest_weight_g ?? 0) / 1000;
  })();
  $: metrics = metricKg > 0 ? computeMetrics(metricKg, profile) : null;
  $: targets = metricKg > 0 ? nutritionTargets(metricKg, profile) : null;
  const GOALS: Goal[] = ['cut_moderate', 'cut_aggressive', 'maintain'];

  async function loadProfile() {
    const p = await guard(() => profileApi.get(loadToken() ?? ''));
    if (p) profile = p;
  }

  let profileSaveTimer: ReturnType<typeof setTimeout> | undefined;
  function onProfileChange() {
    clearTimeout(profileSaveTimer);
    profileSaveTimer = setTimeout(async () => {
      const token = loadToken();
      if (!token) return;
      await guard(() =>
        profileApi.save(token, {
          height_cm: profile.height_cm,
          sex: profile.sex,
          age_years: profile.age_years,
          activity: profile.activity,
          goal: profile.goal
        })
      );
    }, 600);
  }

  // ---- plan templates ------------------------------------------------------

  function blankSet(): SetRow {
    return { weight: '', reps: '', suggested: false };
  }

  function templateRows(focus: Focus): ExerciseRow[] {
    return DAY_TEMPLATES[focus].map((ex) => ({
      name: ex.name,
      scheme: ex.scheme,
      sets: Array.from({ length: setsFromScheme(ex.scheme) }, blankSet)
    }));
  }

  // ---- cardio --------------------------------------------------------------

  /** The MET-estimated kcal for a cardio row at the current bodyweight. */
  function cardioEstimate(row: { kind: string; minutes: string }): number {
    const mins = parseFloat(str(row.minutes));
    if (!Number.isFinite(mins) || mins <= 0) return 0;
    return metKcal(cardioMet(row.kind), metricKg, mins);
  }

  /** One cardio row seeded from the day's suggested bout (kcal auto-estimated). */
  function defaultCardio(focus: Focus): CardioRow {
    const d = CARDIO_DEFAULTS[focus];
    return { kind: d.kind, minutes: String(d.minutes), kcal: '', kcalEdited: false };
  }
  function blankCardio(): CardioRow {
    return { kind: CARDIO_OPTIONS[0].value, minutes: '', kcal: '', kcalEdited: false };
  }

  // Editing: while kcal is untouched it tracks the estimate; typing kcal pins it.
  function onCardioChange(row: CardioRow) {
    if (!row.kcalEdited) {
      const est = cardioEstimate(row);
      row.kcal = est > 0 ? String(est) : '';
    }
    cardioRows = cardioRows;
    scheduleSave();
  }
  function onCardioKcal(row: CardioRow) {
    row.kcalEdited = str(row.kcal).trim() !== '';
    scheduleSave();
  }
  function addCardio() {
    cardioRows = [...cardioRows, blankCardio()];
  }
  function removeCardio(i: number) {
    cardioRows = cardioRows.filter((_, idx) => idx !== i);
    scheduleSave();
  }

  // The kcal actually attributed to a cardio row: the user's value if entered,
  // otherwise the live estimate — so a bout always contributes to the total.
  function cardioKcal(row: CardioRow): number {
    const typed = parseFloat(str(row.kcal));
    if (Number.isFinite(typed) && typed > 0) return Math.round(typed);
    return cardioEstimate(row);
  }

  // ---- session burn totals -------------------------------------------------

  $: totalSets = exercises.reduce(
    (n, ex) => n + ex.sets.filter((s) => !s.suggested && (str(s.reps).trim() !== '' || str(s.weight).trim() !== '')).length,
    0
  );
  $: liftKcal = metricKg > 0 ? strengthKcal(metricKg, totalSets) : 0;
  $: cardioBurn = cardioRows.reduce((n, r) => n + cardioKcal(r), 0);
  $: sessionKcal = liftKcal + cardioBurn;

  // ---- data loading + prefill ---------------------------------------------

  let sessions: SessionSummary[] = [];
  let bodyweight: BodyweightEntry[] = [];

  const detailCache = new Map<number, SessionDetail>();
  async function detailFor(id: number): Promise<SessionDetail | undefined> {
    if (detailCache.has(id)) return detailCache.get(id);
    const d = await guard(() => workoutApi.getSession(id));
    if (d) detailCache.set(id, d);
    return d;
  }

  /** Group a detail's flat set rows by exercise, preserving order. */
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

  /**
   * Build the editable rows for the active day. If a session already exists for
   * (date, active) restore it as committed values; otherwise prefill set weights
   * (only) from the most recent prior session of the same day. Bodyweight is
   * prefilled from today's entry or the latest prior one.
   */
  async function loadDayData() {
    const rows = templateRows(active);
    let cardio: CardioRow[] = [defaultCardio(active)];

    if (signedIn) {
      const todays = sessions.find((s) => s.date === sessionDate && s.day_label === active);
      if (todays) {
        const d = await detailFor(todays.id);
        if (d) {
          fillRows(rows, d, true);
          const restored = cardioFromDetail(d);
          if (restored.length) cardio = restored;
        }
      } else {
        const prior = sessions.find((s) => s.day_label === active && s.date < sessionDate);
        if (prior) {
          const d = await detailFor(prior.id);
          if (d) fillRows(rows, d, false);
        }
      }

      const bwToday = bodyweight.find((b) => b.date === sessionDate);
      if (bwToday) {
        bw = gramsToKg(bwToday.weight_g);
        bwSuggested = false;
        lastSavedBw = bw;
      } else if (bodyweight.length) {
        bw = gramsToKg(bodyweight[bodyweight.length - 1].weight_g);
        bwSuggested = true;
        lastSavedBw = '';
      } else {
        bw = '';
        bwSuggested = true;
        lastSavedBw = '';
      }
    }

    exercises = rows;
    cardioRows = cardio;
  }

  /** Restore committed cardio rows from a saved session detail. */
  function cardioFromDetail(detail: SessionDetail): CardioRow[] {
    return (detail.cardio ?? []).map((c) => ({
      kind: c.kind || CARDIO_OPTIONS[0].value,
      minutes: c.minutes > 0 ? String(c.minutes) : '',
      kcal: c.kcal > 0 ? String(c.kcal) : '',
      kcalEdited: c.kcal > 0
    }));
  }

  /** Fill `rows` from a saved session. `committed` true → keep reps+weight as
   *  real values; false → prefill weights only, as suggestions. */
  function fillRows(rows: ExerciseRow[], detail: SessionDetail, committed: boolean) {
    const groups = groupSets(detail);
    for (const g of groups) {
      let row = rows.find((r) => r.name === g.exercise);
      if (!row) {
        if (!committed) continue; // don't invent off-template rows from suggestions
        row = { name: g.exercise, scheme: '', sets: [] };
        rows.push(row);
      }
      g.sets.forEach((s, j) => {
        if (!row!.sets[j]) row!.sets[j] = blankSet();
        row!.sets[j].weight = s.weight_g > 0 ? gramsToKg(s.weight_g) : '';
        row!.sets[j].reps = committed && s.reps > 0 ? String(s.reps) : '';
        row!.sets[j].suggested = !committed;
      });
      if (committed) expanded[g.exercise] = true;
    }
  }

  // ---- editing -------------------------------------------------------------

  function touchSet(s: SetRow) {
    s.suggested = false;
    exercises = exercises;
    scheduleSave();
  }

  function onBwInput() {
    bwSuggested = false;
    scheduleSave();
  }

  // The bodyweight field is prefilled with yesterday's weight as a faded hint;
  // tapping it should clear that so you can type today's fresh, not edit a stale
  // number. Only clears the hint — a real, committed value is left alone.
  function onBwFocus() {
    if (bwSuggested) bw = '';
  }

  function addSet(i: number) {
    exercises[i].sets = [...exercises[i].sets, blankSet()];
    exercises = exercises;
  }
  function removeSet(i: number, j: number) {
    exercises[i].sets = exercises[i].sets.filter((_, idx) => idx !== j);
    exercises = exercises;
    scheduleSave();
  }
  function addExercise() {
    exercises = [...exercises, { name: '', scheme: '', sets: [blankSet()] }];
  }
  function removeExercise(i: number) {
    exercises = exercises.filter((_, idx) => idx !== i);
    scheduleSave();
  }

  function selectDay(focus: Focus) {
    if (focus === active) return;
    active = focus;
    expanded = {};
    loadDayData();
  }

  function onDateChange() {
    loadDayData();
  }

  // ---- auto-save -----------------------------------------------------------

  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  function scheduleSave() {
    if (!signedIn) return;
    saveStatus = 'saving';
    clearTimeout(saveTimer);
    saveTimer = setTimeout(flush, 700);
  }

  // Number inputs can hand back numbers (not strings) via bind:value, so coerce
  // before trimming.
  const str = (v: unknown): string => (v === null || v === undefined ? '' : String(v));

  function buildPayload(): ExercisePayload[] {
    return exercises
      .map((ex) => ({
        exercise: ex.name.trim(),
        sets: ex.sets
          .filter((s) => !s.suggested && (str(s.reps).trim() !== '' || str(s.weight).trim() !== ''))
          .map((s) => ({
            reps: Math.max(0, parseInt(str(s.reps), 10) || 0),
            weight_g: kgToGrams(str(s.weight))
          }))
      }))
      .filter((ex) => ex.exercise !== '' && ex.sets.length > 0);
  }

  function buildCardio(): CardioPayload[] {
    return cardioRows
      .map((r) => ({
        kind: r.kind.trim(),
        minutes: Math.max(0, parseInt(str(r.minutes), 10) || 0),
        kcal: cardioKcal(r)
      }))
      .filter((r) => r.minutes > 0 || r.kcal > 0);
  }

  // Serialize saves: if a flush is already in flight, mark one pending and run
  // it once the current finishes — so two debounced saves never overlap (which,
  // combined with the backend's atomic batch, kills the duplicate-set bug).
  let flushing = false;
  let flushPending = false;

  async function flush() {
    if (!signedIn) return;
    if (flushing) {
      flushPending = true;
      return;
    }
    flushing = true;
    try {
      await doFlush();
    } finally {
      flushing = false;
      if (flushPending) {
        flushPending = false;
        flush();
      }
    }
  }

  async function doFlush() {
    const payload = buildPayload();
    const cardio = buildCardio();
    let ok = true;

    if (payload.length > 0 || cardio.length > 0) {
      const r = await guard(() =>
        workoutApi.upsertSession({
          day_label: active,
          date: sessionDate,
          notes: '',
          exercises: payload,
          cardio
        })
      );
      ok = ok && r !== undefined;
    }

    if (!bwSuggested && str(bw).trim() !== '' && str(bw) !== str(lastSavedBw)) {
      const grams = kgToGrams(str(bw));
      if (grams > 0) {
        const r = await guard(() => workoutApi.addBodyweight(sessionDate, grams));
        if (r !== undefined) lastSavedBw = bw;
        else ok = false;
      }
    }

    saveStatus = ok ? 'saved' : 'error';
    if (ok) {
      detailCache.clear();
      await refreshLists();
    }
  }

  // ---- history + trend -----------------------------------------------------

  let weekly: WeeklyAverage[] = [];
  $: weekly = weeklyAverages(bodyweight);

  // date → bodyweight grams, so each history row can show that day's weight.
  $: bwByDate = new Map(bodyweight.map((b) => [b.date, b.weight_g]));

  $: sparkPoints = (() => {
    if (weekly.length < 2) return '';
    const w = 320, h = 64, pad = 6;
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

  let openId: number | null = null;
  let openDetail: SessionDetail | null = null;
  let loadingDetail = false;

  async function toggleSession(id: number) {
    if (openId === id) {
      openId = null;
      openDetail = null;
      return;
    }
    openId = id;
    openDetail = null;
    loadingDetail = true;
    openDetail = (await detailFor(id)) ?? null;
    loadingDetail = false;
  }

  async function deleteSession(id: number) {
    const s = sessions.find((x) => x.id === id);
    const ok = await guard(() => workoutApi.deleteSession(id));
    if (ok) {
      // Drop the matching bodyweight entry for that date too, so the two stay
      // in lockstep (one merged history row).
      const bwEntry = s ? bodyweight.find((b) => b.date === s.date) : undefined;
      if (bwEntry) await guard(() => workoutApi.deleteBodyweight(bwEntry.id));
      if (openId === id) {
        openId = null;
        openDetail = null;
      }
      detailCache.delete(id);
      await refreshLists();
      await loadDayData();
    }
  }

  function setLabel(reps: number, weight_g: number): string {
    return weight_g > 0 ? `${gramsToKg(weight_g)}kg × ${reps}` : `BW × ${reps}`;
  }

  async function refreshLists() {
    const [s, b] = await Promise.all([
      guard(() => workoutApi.listSessions()),
      guard(() => workoutApi.listBodyweight())
    ]);
    if (s) sessions = s;
    if (b) bodyweight = b;
  }

  // ---- auth (mirrors the splitter / tracker Google Identity flow) ----------

  let refreshTimer: ReturnType<typeof setTimeout> | undefined;
  function scheduleRefresh() {
    clearTimeout(refreshTimer);
    refreshTimer = scheduleTokenRefresh(loadToken(), clientId, onCredential);
  }

  function onCredential(resp: { credential: string }) {
    setToken(resp.credential);
    signedIn = true;
    scheduleRefresh();
    bootSignedIn();
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
    detailCache.clear();
    openId = null;
    openDetail = null;
    saveStatus = 'idle';
    exercises = templateRows(active);
    cardioRows = [defaultCardio(active)];
    setTimeout(renderGoogleButton, 0);
  }

  async function guard<T>(fn: () => Promise<T>): Promise<T | undefined> {
    errorMsg = '';
    try {
      return await fn();
    } catch (e) {
      if (e instanceof AuthError) {
        signedIn = false;
        sessions = [];
        bodyweight = [];
        saveStatus = 'idle';
        setTimeout(renderGoogleButton, 0);
      } else {
        errorMsg = e instanceof Error ? e.message : 'something went wrong';
      }
      return undefined;
    }
  }

  async function bootSignedIn() {
    await refreshLists();
    await loadProfile();
    await loadDayData();
  }

  onMount(async () => {
    exercises = templateRows(active);
    cardioRows = [defaultCardio(active)];
    await loadGis();
    if (loadToken()) {
      signedIn = true;
      scheduleRefresh();
      await bootSignedIn();
    } else {
      renderGoogleButton();
    }
  });
</script>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/fitness">Fitness</a>
      <span class="separator">/</span>
      <span>Workout</span>
    </div>
    <div class="header-top">
      <h1 class="page-title">Workout 🏋️</h1>
      <div class="header-actions">
        {#if signedIn}
          <span class="save-pill" class:on={saveStatus !== 'idle'} data-status={saveStatus}>
            {#if saveStatus === 'saving'}
              <Loader size={13} /> Saving…
            {:else if saveStatus === 'saved'}
              <Check size={13} /> Saved
            {:else if saveStatus === 'error'}
              Save failed
            {/if}
          </span>
          <button class="ghost-btn" on:click={signOut} title="Sign out">
            <LogOut size={15} /> Sign out
          </button>
        {/if}
      </div>
    </div>
    <p class="page-desc">
      Push/pull/legs split. Pick a day, then {signedIn ? 'tap an exercise and log your sets — it saves as you type.' : 'sign in to log your sets right here.'}
    </p>
  </header>

  {#if errorMsg}
    <div class="error-banner">{errorMsg}</div>
  {/if}

  <!-- Week at a glance — each chip jumps to that day's exercises. -->
  <div class="week-strip">
    {#each split as row}
      <button
        class="day-chip"
        class:active={row.focus === active}
        class:rest={row.focus === 'Optional'}
        on:click={() => row.focus !== 'Optional' && selectDay(row.focus)}
        disabled={row.focus === 'Optional'}
        title={row.detail}
      >
        <span class="chip-day">{row.day}</span>
        <span class="chip-focus">{row.focus}</span>
      </button>
    {/each}
  </div>

  <!-- Focus tabs -->
  <div class="tabs" role="tablist">
    {#each FOCUSES as f}
      <button
        class="tab"
        class:active={f === active}
        role="tab"
        aria-selected={f === active}
        on:click={() => selectDay(f)}
      >
        {f}
      </button>
    {/each}
  </div>

  {#if !signedIn}
    <div class="signin-card">
      <p>Sign in with Google to log and sync your sets, reps and bodyweight.</p>
      <div bind:this={gisButton}></div>
    </div>
  {:else}
    <!-- Session bar: date + bodyweight -->
    <div class="session-bar">
      <label>
        Date
        <input type="date" bind:value={sessionDate} on:change={onDateChange} />
      </label>
      <label>
        Bodyweight (kg)
        <input
          type="number"
          inputmode="decimal"
          min="0"
          step="0.1"
          placeholder="e.g. 87.5"
          class:suggested={bwSuggested}
          bind:value={bw}
          on:focus={onBwFocus}
          on:input={onBwInput}
        />
      </label>
    </div>

    <!-- Live body metrics — recompute from the bodyweight above + profile. -->
    {#if metrics && targets}
      <div class="metrics-card">
        <div class="metrics-grid">
          <div class="metric">
            <span class="m-val">{metrics.bmi}</span>
            <span class="m-label">BMI</span>
            <span class="m-sub">{metrics.bmiCategory}</span>
          </div>
          <div class="metric">
            <span class="m-val">{metrics.bmr}</span>
            <span class="m-label">BMR</span>
            <span class="m-sub">kcal/day</span>
          </div>
          <div class="metric">
            <span class="m-val">{metrics.tdee}</span>
            <span class="m-label">TDEE</span>
            <span class="m-sub">maintenance</span>
          </div>
          <div class="metric goal">
            <span class="m-val">{targets.calories}</span>
            <span class="m-label">Target</span>
            <span class="m-sub">kcal/day</span>
          </div>
        </div>
        <p class="macro-line">
          Daily macros: <strong>{targets.protein_g}g</strong> protein ·
          <strong>{targets.carbs_g}g</strong> carbs ·
          <strong>{targets.fat_g}g</strong> fat
        </p>
      </div>

      <!-- Profile — the inputs the metrics math is built on. -->
      <details class="fold">
        <summary>Profile</summary>
        <div class="fold-body">
          <div class="profile-grid">
            <label>
              Height (cm)
              <input type="number" min="50" max="300" bind:value={profile.height_cm} on:input={onProfileChange} />
            </label>
            <label>
              Age
              <input type="number" min="1" max="120" bind:value={profile.age_years} on:input={onProfileChange} />
            </label>
            <label>
              Sex
              <select bind:value={profile.sex} on:change={onProfileChange}>
                <option value="male">male</option>
                <option value="female">female</option>
              </select>
            </label>
            <label>
              Activity
              <select bind:value={profile.activity} on:change={onProfileChange}>
                {#each ACTIVITY_OPTIONS as a}
                  <option value={a.value}>{a.label}</option>
                {/each}
              </select>
            </label>
            <label class="wide">
              Goal
              <select bind:value={profile.goal} on:change={onProfileChange}>
                {#each GOALS as g}
                  <option value={g}>{GOAL_LABELS[g]}</option>
                {/each}
              </select>
            </label>
          </div>
        </div>
      </details>
    {/if}
  {/if}

  <!-- Active day's exercises -->
  <section class="day-card">
    <div class="day-card-head">
      <h2>{active}</h2>
      <span class="cardio-pill">🚴 {cardio[active]}</span>
    </div>

    {#if !signedIn}
      <!-- read-only plan view -->
      <ul class="exercise-list">
        {#each exercises as ex, i}
          <li>
            <span class="ex-num">{i + 1}</span>
            <span class="ex-name">{ex.name}</span>
            <span class="ex-scheme">{ex.scheme}</span>
          </li>
        {/each}
      </ul>
    {:else}
      <!-- interactive logging view -->
      <div class="log-list">
        {#each exercises as ex, i}
          <div class="log-ex" class:open={expanded[ex.name || `_${i}`]}>
            <button
              type="button"
              class="log-ex-head"
              on:click={() => (expanded = { ...expanded, [ex.name || `_${i}`]: !expanded[ex.name || `_${i}`] })}
            >
              <span class="ex-num">{i + 1}</span>
              {#if ex.scheme}
                <span class="ex-name">{ex.name}</span>
                <span class="ex-target">{ex.scheme}</span>
              {:else}
                <input
                  class="ex-name-input"
                  placeholder="Exercise name"
                  bind:value={ex.name}
                  on:click|stopPropagation
                  on:input={scheduleSave}
                />
              {/if}
              <span class="chevron" aria-hidden="true">{expanded[ex.name || `_${i}`] ? '−' : '+'}</span>
            </button>

            {#if expanded[ex.name || `_${i}`]}
              <div class="sets">
                {#each ex.sets as s, j}
                  <div class="set-row">
                    <span class="set-num">{j + 1}</span>
                    <input
                      class="num"
                      class:suggested={s.suggested}
                      type="number"
                      inputmode="decimal"
                      min="0"
                      step="0.5"
                      placeholder="kg"
                      bind:value={s.weight}
                      on:input={() => touchSet(s)}
                    />
                    <span class="times">×</span>
                    <input
                      class="num"
                      type="number"
                      inputmode="numeric"
                      min="0"
                      placeholder="reps"
                      bind:value={s.reps}
                      on:input={() => touchSet(s)}
                    />
                    <button class="icon-btn" on:click={() => removeSet(i, j)} title="Remove set">
                      <Trash2 size={14} />
                    </button>
                  </div>
                {/each}
                <div class="set-actions">
                  <button class="text-btn" on:click={() => addSet(i)}>
                    <Plus size={14} /> set
                  </button>
                  {#if !ex.scheme}
                    <button class="text-btn danger" on:click={() => removeExercise(i)}>
                      <Trash2 size={13} /> remove exercise
                    </button>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/each}

        <button class="text-btn add-ex" on:click={addExercise}>
          <Plus size={15} /> add exercise
        </button>
      </div>

      <!-- Cardio log — kind + minutes + kcal (auto-estimated, editable). -->
      <div class="cardio-log">
        <div class="cardio-log-head">
          <span class="cardio-log-title">🚴 Cardio</span>
          <span class="cardio-log-hint">kcal auto-estimates from time &amp; bodyweight — tap to override</span>
        </div>
        {#each cardioRows as c, i}
          <div class="cardio-row">
            <select class="cardio-kind" bind:value={c.kind} on:change={() => onCardioChange(c)}>
              {#each CARDIO_OPTIONS as o}
                <option value={o.value}>{o.value}</option>
              {/each}
            </select>
            <div class="cardio-field">
              <input
                class="num"
                type="number"
                inputmode="numeric"
                min="0"
                placeholder="min"
                bind:value={c.minutes}
                on:input={() => onCardioChange(c)}
              />
              <span class="unit">min</span>
            </div>
            <div class="cardio-field">
              <input
                class="num"
                class:suggested={!c.kcalEdited}
                type="number"
                inputmode="numeric"
                min="0"
                placeholder={cardioEstimate(c) > 0 ? `≈${cardioEstimate(c)}` : 'kcal'}
                bind:value={c.kcal}
                on:input={() => onCardioKcal(c)}
              />
              <span class="unit">kcal</span>
            </div>
            <button class="icon-btn" on:click={() => removeCardio(i)} title="Remove cardio">
              <Trash2 size={14} />
            </button>
          </div>
        {/each}
        <button class="text-btn add-ex cardio-add" on:click={addCardio}>
          <Plus size={14} /> cardio
        </button>
      </div>
    {/if}
  </section>

  <!-- Session energy burn — rough MET-based estimate from the sets + cardio. -->
  {#if signedIn && metricKg > 0}
    <div class="burn-card">
      <div class="burn-grid">
        <div class="burn-metric">
          <span class="b-val">{liftKcal}</span>
          <span class="b-label">Lifting</span>
          <span class="b-sub">{totalSets} {totalSets === 1 ? 'set' : 'sets'}</span>
        </div>
        <div class="burn-metric">
          <span class="b-val">{cardioBurn}</span>
          <span class="b-label">Cardio</span>
          <span class="b-sub">kcal</span>
        </div>
        <div class="burn-metric total">
          <span class="b-val">{sessionKcal}</span>
          <span class="b-label">Session</span>
          <span class="b-sub">kcal burnt</span>
        </div>
      </div>
      <p class="burn-note">
        Rough estimate at {metricKg.toFixed(1)} kg — lifting assumes ~2.5 min per logged set at
        vigorous effort. Cardio counts your logged (or estimated) kcal.
      </p>
    </div>
  {/if}

  <!-- Reference / rationale, collapsed by default. -->
  <details class="fold">
    <summary>Notes &amp; rationale</summary>
    <div class="notes-body">
      <div class="note">
        <h3>Cardio rule</h3>
        <p>
          15–20 min post-lift, every day. Rotate cycle / crosstrainer / treadmill so it
          never gets boring. Never before lifting — it kills strength on the compounds.
        </p>
      </div>
      <div class="note">
        <h3>Progressive overload</h3>
        <p>
          The whole game. Same weight every week = no growth. Add weight or reps in small
          steps, every week.
        </p>
      </div>
      <div class="note">
        <h3>On the numbers</h3>
        <p>
          178–179 cm, 86–90 kg fluctuating is mostly water and food noise. Ignore daily
          swings, track the weekly average. ~88 kg → 80 kg in two months means ~1 kg/week —
          aggressive but doable with protein-heavy eating plus lifting. Don't go below
          0.7–1 kg/week or you'll burn muscle.
        </p>
      </div>
      <div class="note">
        <h3>Quick math check</h3>
        <p>
          8 kg in 8 weeks = 1 kg/week ≈ 1,100 kcal/day deficit — steep while lifting hard.
          Better: a 700–800 kcal deficit leaning on protein lands loss closer to
          0.7–0.8 kg/week and lets recomp do the rest.
        </p>
      </div>
      <div class="note">
        <h3>Why the plan got leaner</h3>
        <p>
          Trimmed each day to its essentials. Rear delts moved to Pull day — they're a
          pulling muscle, so anatomically that's where they belong. Abs consolidated onto
          Legs day, since you're less smashed after legs than after pull. Lat pulldown and
          row machine dropped to 3×10 from 4×10 — still plenty of back volume. Walking
          lunges cut entirely: squat + leg press is already redundant quad work, and lunges
          were the first thing getting skipped anyway.
        </p>
      </div>
    </div>
  </details>

  {#if signedIn}
    <!-- Bodyweight trend -->
    <details class="fold">
      <summary>Bodyweight trend</summary>
      <div class="fold-body">
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
        {:else}
          <p class="hint">No bodyweight entries yet — add one up top.</p>
        {/if}
      </div>
    </details>

    <!-- History -->
    <details class="fold">
      <summary>History</summary>
      <div class="fold-body">
        {#if sessions.length === 0}
          <p class="hint">No sessions logged yet.</p>
        {:else}
          <div class="history-list">
            {#each sessions as s}
              <div class="history-item">
                <button class="history-head" on:click={() => toggleSession(s.id)}>
                  <span class="hist-date">{s.date}</span>
                  {#if s.day_label}<span class="hist-day">{s.day_label}</span>{/if}
                  {#if bwByDate.get(s.date) !== undefined}
                    <span class="hist-bw">{gramsToKg(bwByDate.get(s.date) ?? 0)} kg</span>
                  {/if}
                </button>
                <button class="icon-btn" on:click={() => deleteSession(s.id)} title="Delete session + bodyweight">
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
                      {#if openDetail.cardio && openDetail.cardio.length}
                        <div class="detail-ex">
                          <span class="detail-ex-name">🚴 Cardio</span>
                          <div class="detail-sets">
                            {#each openDetail.cardio as c}
                              <span class="set-badge">
                                {c.kind}{c.minutes > 0 ? ` · ${c.minutes} min` : ''} · {c.kcal} kcal
                              </span>
                            {/each}
                          </div>
                        </div>
                      {/if}
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </details>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    max-width: 48rem;
    animation: fade-up var(--dur-base, 0.4s) var(--ease-out-quart, ease);
  }

  /* ── Header ─────────────────────────────────────────────── */
  .page-header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-bottom: 1.25rem;
    border-bottom: 1px solid var(--border);
  }
  .breadcrumb { font-size: 0.875rem; color: var(--text-tertiary); }
  .breadcrumb a { color: var(--text-tertiary); transition: color 0.15s; }
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

  .save-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-tertiary);
    opacity: 0;
    transition: opacity 0.2s;
  }
  .save-pill.on { opacity: 1; }
  .save-pill[data-status='saved'] { color: var(--blueprint); }
  .save-pill[data-status='error'] { color: #e74c3c; }
  .save-pill :global(svg) { animation: none; }
  .save-pill[data-status='saving'] :global(svg) { animation: spin 0.9s linear infinite; }

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
  .ghost-btn:hover { color: var(--text-primary); border-color: var(--border-strong); }

  .page-desc {
    font-size: 0.95rem;
    line-height: 1.55;
    color: var(--text-secondary);
    margin: 0;
  }

  .error-banner {
    padding: 0.7rem 1rem;
    border: 1px solid #c0392b;
    background: rgba(192, 57, 43, 0.1);
    border-radius: 0.5rem;
    color: #e74c3c;
    font-size: 0.875rem;
  }

  /* ── Week strip ─────────────────────────────────────────── */
  .week-strip {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 0.5rem;
  }
  .day-chip {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.25rem;
    padding: 0.6rem 0.65rem;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
  }
  .day-chip:hover:not(:disabled) { border-color: var(--blueprint); }
  .day-chip.active { border-color: var(--blueprint); background: var(--blueprint-tint); }
  .day-chip.rest { opacity: 0.5; cursor: default; }
  .chip-day {
    font-family: var(--font-mono);
    font-size: 0.6rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }
  .chip-focus { font-size: 0.85rem; font-weight: 600; color: var(--text-primary); }
  .day-chip.active .chip-focus { color: var(--blueprint); }

  /* ── Tabs ───────────────────────────────────────────────── */
  .tabs {
    display: flex;
    gap: 0.25rem;
    padding: 0.25rem;
    background: var(--blueprint-tint);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
  }
  .tab {
    flex: 1;
    padding: 0.55rem;
    background: transparent;
    border: none;
    border-radius: 0.375rem;
    font-family: var(--font-mono);
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
    cursor: pointer;
    transition: color 0.15s, background 0.15s;
  }
  .tab:hover { color: var(--text-primary); }
  .tab.active { color: var(--bg, #111); background: var(--blueprint); }

  /* ── Sign-in ────────────────────────────────────────────── */
  .signin-card {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    align-items: flex-start;
    padding: 1.5rem;
    border: 1px solid var(--border);
    border-radius: 0.625rem;
    background: var(--surface-raised);
  }
  .signin-card p { color: var(--text-secondary); margin: 0; font-size: 0.95rem; }

  /* ── Session bar ────────────────────────────────────────── */
  .session-bar {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
  }
  .session-bar label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.75rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-tertiary);
    flex: 1;
    min-width: 9rem;
  }

  input {
    font-family: inherit;
    font-size: 0.9375rem;
    color: var(--text-primary);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    padding: 0.5rem 0.625rem;
    width: 100%;
  }
  input:focus { outline: none; border-color: var(--border-strong); }

  /* ── Body metrics ───────────────────────────────────────── */
  .metrics-card {
    border: 1px solid var(--border);
    border-radius: 0.625rem;
    padding: 1rem;
  }
  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.75rem;
  }
  .metric {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.15rem;
    padding: 0.75rem 0.4rem;
    border: 1px solid var(--border-subtle);
    border-radius: 0.5rem;
  }
  .metric.goal { border-color: var(--blueprint); background: var(--blueprint-tint); }
  .m-val { font-size: 1.25rem; font-weight: 700; color: var(--text-primary); }
  .m-label { font-size: 0.7rem; letter-spacing: 0.06em; text-transform: uppercase; color: var(--text-secondary); }
  .m-sub { font-size: 0.66rem; color: var(--text-tertiary); }
  .macro-line { margin: 0.85rem 0 0; font-size: 0.85rem; color: var(--text-secondary); text-align: center; }
  .macro-line strong { color: var(--text-primary); }

  .profile-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
  }
  .profile-grid label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.72rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }
  .profile-grid label.wide { grid-column: 1 / -1; }
  .profile-grid select {
    font-family: inherit;
    font-size: 0.9rem;
    color: var(--text-primary);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    padding: 0.5rem 0.625rem;
    width: 100%;
  }
  .profile-grid select:focus { outline: none; border-color: var(--border-strong); }

  @media (max-width: 520px) {
    .metrics-grid { grid-template-columns: repeat(2, 1fr); }
  }
  input.suggested { color: var(--text-tertiary); font-style: italic; }

  /* ── Day card ───────────────────────────────────────────── */
  .day-card {
    border: 1px solid var(--border);
    border-radius: 0.625rem;
    overflow: hidden;
  }
  .day-card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
    padding: 0.85rem 1rem;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--blueprint-tint);
  }
  .day-card-head h2 { font-size: 1.25rem; font-weight: 700; color: var(--text-primary); margin: 0; }
  .cardio-pill { font-size: 0.78rem; color: var(--text-secondary); font-style: italic; }

  /* read-only plan list */
  .exercise-list { display: flex; flex-direction: column; list-style: none; padding: 0; margin: 0; }
  .exercise-list li {
    display: grid;
    grid-template-columns: 1.5rem 1fr auto;
    align-items: baseline;
    gap: 0.75rem;
    padding: 0.7rem 1rem;
    border-bottom: 1px solid var(--border-subtle);
  }
  .exercise-list li:last-child { border-bottom: none; }
  .ex-num { font-family: var(--font-mono); font-size: 0.75rem; color: var(--text-tertiary); }
  .ex-name { font-size: 0.9375rem; line-height: 1.4; color: var(--text-secondary); }
  .ex-scheme {
    flex-shrink: 0;
    font-family: var(--font-mono);
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-primary);
    background: var(--surface-raised);
    padding: 0.2rem 0.5rem;
    border-radius: 0.25rem;
  }

  /* interactive log list */
  .log-list { display: flex; flex-direction: column; }
  .log-ex { border-bottom: 1px solid var(--border-subtle); }
  .log-ex:last-of-type { border-bottom: none; }
  .log-ex-head {
    display: grid;
    grid-template-columns: 1.5rem 1fr auto auto;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.75rem 1rem;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
  }
  .log-ex.open .log-ex-head { background: var(--blueprint-tint); }
  .log-ex-head .ex-name { color: var(--text-primary); }
  .ex-target {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-tertiary);
  }
  .ex-name-input { font-weight: 600; }
  .chevron {
    font-family: var(--font-mono);
    font-size: 1rem;
    color: var(--text-tertiary);
    width: 1rem;
    text-align: center;
  }

  .sets {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.5rem 1rem 0.9rem 1rem;
  }
  .set-row { display: flex; align-items: center; gap: 0.5rem; }
  .set-num {
    width: 1.25rem;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--text-faint);
    flex-shrink: 0;
  }
  .num { width: 5rem; text-align: center; }
  .times { color: var(--text-faint); }

  .set-actions { display: flex; gap: 1rem; flex-wrap: wrap; padding-left: 1.75rem; }

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
    transition: color 0.15s;
  }
  .icon-btn:hover { color: #e74c3c; }

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
  }
  .text-btn:hover { color: var(--text-primary); }
  .text-btn.danger:hover { color: #e74c3c; }
  .add-ex {
    margin: 0.5rem 1rem 0.9rem;
    border: 1px dashed var(--border);
    border-radius: 0.375rem;
    padding: 0.5rem 0.75rem;
    align-self: flex-start;
  }

  /* ── Cardio log ─────────────────────────────────────────── */
  .cardio-log {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.9rem 1rem 1rem;
    border-top: 1px solid var(--border-subtle);
    background: var(--blueprint-tint);
  }
  .cardio-log-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  .cardio-log-title {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-secondary);
  }
  .cardio-log-hint { font-size: 0.72rem; color: var(--text-tertiary); font-style: italic; }
  .cardio-row { display: flex; align-items: center; gap: 0.5rem; }
  .cardio-kind {
    flex: 1;
    min-width: 0;
    font-family: inherit;
    font-size: 0.9rem;
    color: var(--text-primary);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    padding: 0.5rem 0.5rem;
  }
  .cardio-kind:focus { outline: none; border-color: var(--border-strong); }
  .cardio-field { display: flex; align-items: center; gap: 0.3rem; }
  .cardio-field .unit { font-size: 0.72rem; color: var(--text-faint); }
  .cardio-add { margin: 0.25rem 0 0; align-self: flex-start; }

  /* ── Session burn ───────────────────────────────────────── */
  .burn-card {
    border: 1px solid var(--border);
    border-radius: 0.625rem;
    padding: 1rem;
  }
  .burn-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.75rem;
  }
  .burn-metric {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.15rem;
    padding: 0.75rem 0.4rem;
    border: 1px solid var(--border-subtle);
    border-radius: 0.5rem;
  }
  .burn-metric.total { border-color: var(--blueprint); background: var(--blueprint-tint); }
  .b-val { font-size: 1.25rem; font-weight: 700; color: var(--text-primary); }
  .b-label { font-size: 0.7rem; letter-spacing: 0.06em; text-transform: uppercase; color: var(--text-secondary); }
  .b-sub { font-size: 0.66rem; color: var(--text-tertiary); }
  .burn-note { margin: 0.85rem 0 0; font-size: 0.78rem; line-height: 1.5; color: var(--text-tertiary); text-align: center; }

  /* ── Folds (notes / history / trend) ────────────────────── */
  .fold { border: 1px solid var(--border); border-radius: 0.625rem; }
  .fold summary {
    padding: 0.85rem 1rem;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
    cursor: pointer;
    list-style: none;
  }
  .fold summary::-webkit-details-marker { display: none; }
  .fold summary::before { content: '+ '; color: var(--text-tertiary); }
  .fold[open] summary::before { content: '– '; }
  .fold[open] summary { color: var(--text-primary); border-bottom: 1px solid var(--border-subtle); }

  .notes-body {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
    padding: 1.25rem 1rem;
  }
  .note h3 { font-size: 0.95rem; font-weight: 600; color: var(--text-primary); margin: 0 0 0.4rem; }
  .note p { font-size: 0.875rem; line-height: 1.6; color: var(--text-secondary); margin: 0; }

  .fold-body { padding: 1.25rem 1rem; }

  .spark { width: 100%; height: 64px; }
  .spark polyline { fill: none; stroke: var(--blueprint, #6ea8fe); stroke-width: 2; vector-effect: non-scaling-stroke; }

  .hint { font-size: 0.8125rem; color: var(--text-tertiary); margin: 0 0 0.75rem; }

  .weekly-list { display: flex; flex-direction: column; }
  .weekly-row {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 0.875rem;
  }
  .weekly-row:last-child { border-bottom: none; }
  .weekly-week { color: var(--text-secondary); flex: 1; }
  .weekly-avg { font-family: var(--font-mono); font-weight: 600; color: var(--text-primary); }
  .weekly-count { color: var(--text-faint); font-size: 0.75rem; }

  .history-list { display: flex; flex-direction: column; gap: 0.5rem; }
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
  .hist-date { font-family: var(--font-mono); font-size: 0.875rem; color: var(--text-primary); }
  .hist-day {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--blueprint, #6ea8fe);
  }
  .hist-bw {
    font-family: var(--font-mono);
    font-size: 0.75rem;
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
  .detail-ex { display: flex; flex-direction: column; gap: 0.375rem; }
  .detail-ex-name { font-size: 0.875rem; color: var(--text-secondary); }
  .detail-sets { display: flex; flex-wrap: wrap; gap: 0.375rem; }
  .set-badge {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--text-primary);
    background: var(--surface-raised);
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
  }

  @keyframes fade-up {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  @media (max-width: 640px) {
    .page { gap: 1.15rem; }
    .page-title { font-size: 1.75rem; }
    .week-strip { grid-template-columns: repeat(3, 1fr); gap: 0.4rem; }
    .day-chip { padding: 0.5rem 0.5rem; }
    .notes-body { grid-template-columns: 1fr; gap: 1rem; }

    /* Session bar: stack date + bodyweight so neither gets squeezed. */
    .session-bar { gap: 0.75rem; }
    .session-bar label { min-width: 100%; }

    /* Exercise rows: switch the header from grid to wrapping flex so a long
       name keeps the chevron on the first line and the target/scheme drops to
       its own line instead of crushing everything. */
    .log-ex-head {
      display: flex;
      flex-wrap: wrap;
      gap: 0.5rem;
      padding: 0.75rem 0.75rem;
    }
    .log-ex-head .ex-name,
    .log-ex-head .ex-name-input { flex: 1; min-width: 0; }
    .chevron { order: 2; }
    .ex-target { order: 3; flex-basis: 100%; padding-left: 2rem; }
    .day-card-head { padding: 0.75rem 0.75rem; }
    .sets { padding: 0.5rem 0.75rem 0.9rem; }
    .set-row { gap: 0.4rem; }
    .num { width: 100%; min-width: 0; flex: 1; }
    .set-num { width: 1rem; }
  }

  @media (max-width: 380px) {
    .week-strip { grid-template-columns: repeat(2, 1fr); }
    .metrics-grid { gap: 0.5rem; }
  }
</style>
