<svelte:head>
  <title>Workout — Kashif</title>
  <meta name="description" content="A four-day upper/lower program with short cardio bouts — built to hold muscle while cutting." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { LogOut, Check, Plus, Trash2 } from 'lucide-svelte';
  import LoadingState from '$lib/components/LoadingState.svelte';
  import { patchLog } from '$lib/workoutReport';
  import { loadTarget, type LoadTarget } from '$lib/workoutTargets';
  import { setToken, loadToken, AuthError } from '$lib/splitterApi';
  import { isLocalDev } from '$lib/apiBase';
  import { scheduleTokenRefresh, loadGis, renderGoogleButton } from '$lib/fitnessAuth';
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
  import { weekChecklist } from '$lib/fitnessInsights';
  import {
    DAY_TEMPLATES,
    DAY_LABELS,
    LIFT_DAYS,
    DAY_INFO,
    setsFromScheme,
    kgToGrams,
    gramsToKg,
    CARDIO_OPTIONS,
    CARDIO_DEFAULTS,
    cardioMet,
    exerciseKind,
    exerciseEquipment,
    groupSets,
    EQUIPMENT_OPTIONS,
    type DayLabel,
    type ExerciseKind,
    type Equipment,
    type SessionSummary,
    type SessionDetail,
    type BodyweightEntry,
    type TrainingLog
  } from '$lib/workout';

  type Focus = DayLabel;

  const clientId = env.PUBLIC_GOOGLE_CLIENT_ID ?? '';

  // ---- state ---------------------------------------------------------------

  let signedIn = false;
  let gisButton: HTMLDivElement;
  let errorMsg = '';

  let active: Focus = LIFT_DAYS[0];
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
  type ExerciseRow = {
    name: string;
    scheme: string;
    kind: ExerciseKind;
    equipment: Equipment;
    /** Compound / previously-neglected lift — progress these first. */
    priority: boolean;
    /** Fallback or upgrade for when the prescribed implement is taken or free. */
    alt: string;
    sets: SetRow[];
  };
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

  /** Collapse/expand key. Includes the implement so the same exercise logged on
   *  two implements gets two independently-foldable blocks. */
  function rowKey(ex: ExerciseRow, i: number): string {
    return ex.name ? `${ex.name}|${ex.equipment}` : `_${i}`;
  }

  function templateRows(focus: Focus): ExerciseRow[] {
    return DAY_TEMPLATES[focus].map((ex) => ({
      name: ex.name,
      scheme: ex.scheme,
      kind: ex.kind,
      equipment: ex.equipment ?? '',
      priority: ex.priority ?? false,
      alt: ex.alt ?? '',
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

  // The rest of the date: a second session already saved under another day
  // label (a cardio day after a lift, say). Read from the log, which is
  // patched after every save, so it never double-counts the active day.
  $: otherSetsToday = log.sets.filter((s) => s.date === sessionDate && s.day_label !== active).length;
  $: otherCardioToday = log.cardio
    .filter((c) => c.date === sessionDate && c.day_label !== active)
    .reduce((n, c) => n + c.kcal, 0);
  $: daySets = totalSets + otherSetsToday;
  $: dayLiftKcal = metricKg > 0 ? strengthKcal(metricKg, daySets) : 0;
  $: dayCardioKcal = cardioBurn + otherCardioToday;
  $: dayKcal = dayLiftKcal + dayCardioKcal;

  // ---- data loading + prefill ---------------------------------------------

  let sessions: SessionSummary[] = [];
  let bodyweight: BodyweightEntry[] = [];

  // Every set and bout, oldest first, for the report card. Loaded once, then
  // patched locally after each save rather than refetched.
  let log: TrainingLog = { sets: [], cardio: [] };
  let logLoading = false;

  // Today's working load per row, from the log. Recomputed when the implement
  // or the date changes; `applyTargets` copies it into untouched set rows.
  $: loadTargets = exercises.map((ex): LoadTarget | null =>
    ex.kind === 'weighted' ? loadTarget(log, ex.name, ex.equipment, ex.scheme, sessionDate, ex.priority) : null
  );

  /**
   * Prefill every set of a row with its target load, as a suggestion. This
   * replaces the copy of last session's weights, which carried its ramp (15,
   * 15, 17.5, 20) forward; the plan wants one load for every working set.
   * Rows with any committed value are left alone.
   */
  function applyTargets() {
    exercises.forEach((ex) => {
      if (ex.kind !== 'weighted' || ex.sets.some((s) => !s.suggested && (str(s.weight) !== '' || str(s.reps) !== ''))) return;
      const t = loadTarget(log, ex.name, ex.equipment, ex.scheme, sessionDate, ex.priority);
      if (!t) return;
      for (const s of ex.sets) {
        s.weight = gramsToKg(Math.round(t.kg * 1000));
        s.suggested = true;
      }
    });
    exercises = exercises;
  }

  async function loadLog() {
    logLoading = true;
    try {
      log = await workoutApi.listLog();
    } catch (e) {
      if (e instanceof AuthError) {
        await guard(() => Promise.reject(e));
      } else {
        // A Worker without the /log route yet (Pages can deploy first): build
        // the log from per-session detail instead, which the cache shares.
        log = await logFromDetails();
      }
    } finally {
      logLoading = false;
    }
  }

  /** Fallback log from the last `LOG_FALLBACK` sessions, four requests at a time. */
  const LOG_FALLBACK = 60;
  async function logFromDetails(): Promise<TrainingLog> {
    const out: TrainingLog = { sets: [], cardio: [] };
    const recent = sessions.slice(0, LOG_FALLBACK);
    for (let i = 0; i < recent.length; i += 4) {
      const batch = await Promise.all(recent.slice(i, i + 4).map((s) => detailFor(s.id)));
      for (const d of batch) {
        if (!d) continue;
        const stamp = { session_id: d.session.id, date: d.session.date, day_label: d.session.day_label };
        for (const x of d.sets) out.sets.push({ ...stamp, ...x, equipment: x.equipment ?? '' });
        for (const c of d.cardio ?? []) out.cardio.push({ ...stamp, kind: c.kind, minutes: c.minutes, kcal: c.kcal });
      }
    }
    const byDate = (a: { date: string; session_id: number }, b: { date: string; session_id: number }) =>
      a.date === b.date ? a.session_id - b.session_id : a.date < b.date ? -1 : 1;
    out.sets.sort(byDate);
    out.cardio.sort(byDate);
    return out;
  }

  const detailCache = new Map<number, SessionDetail>();
  async function detailFor(id: number): Promise<SessionDetail | undefined> {
    if (detailCache.has(id)) return detailCache.get(id);
    const d = await guard(() => workoutApi.getSession(id));
    if (d) detailCache.set(id, d);
    return d;
  }

  /**
   * Build the editable rows for the active day. If a session already exists for
   * (date, active) restore it as committed values; otherwise prefill each
   * exercise's weights from the last session that contains it, whatever day it
   * was logged under. Bodyweight is prefilled from today's entry or the latest
   * prior one.
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
        await prefillFromHistory(rows);
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
    if (log.sets.length) applyTargets();
  }

  /**
   * Suggest weights for each template row from the most recent prior session
   * that logged that exercise on the row's implement. Sessions are scanned
   * newest-first across every day label, because the same lift appears on
   * more than one day and a plan change renames days without changing the
   * lifts. Sets with no implement recorded (logged before it was tracked)
   * count as a match; sets on a different implement don't, so a template
   * barbell slot isn't relabelled by last week's dumbbell session. Each
   * session costs one request, so the scan stops once every row has a
   * suggestion or after `PREFILL_LOOKBACK` sessions, whichever comes first.
   */
  const PREFILL_LOOKBACK = 20;
  async function prefillFromHistory(rows: ExerciseRow[]) {
    const pending = new Map(rows.filter((r) => r.name).map((r) => [r.name, r.equipment]));
    if (pending.size === 0) return;
    const matches = (x: { exercise: string; equipment: Equipment }) => {
      const want = pending.get(x.exercise);
      return want !== undefined && (!want || !x.equipment || x.equipment === want);
    };
    const prior = sessions.filter((s) => s.date < sessionDate).slice(0, PREFILL_LOOKBACK);
    for (const s of prior) {
      if (pending.size === 0) break;
      const d = await detailFor(s.id);
      if (!d) continue;
      const wanted = { ...d, sets: d.sets.filter(matches) };
      if (wanted.sets.length === 0) continue;
      fillRows(rows, wanted, false);
      for (const x of wanted.sets) pending.delete(x.exercise);
    }
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
    // A row can only absorb one group, so logging the same exercise on two
    // implements in one session restores as two blocks rather than clobbering.
    const claimed = new Set<ExerciseRow>();
    for (const g of groups) {
      let row = rows.find((r) => r.name === g.exercise && !claimed.has(r));
      if (!row) {
        if (!committed) continue; // don't invent off-template rows from suggestions
        row = {
          name: g.exercise,
          scheme: '',
          kind: exerciseKind(g.exercise),
          equipment: g.equipment || exerciseEquipment(g.exercise),
          priority: false,
          alt: '',
          sets: []
        };
        rows.push(row);
      }
      claimed.add(row);
      // Adopt the implement the weights were actually lifted on — otherwise a
      // suggested 52.5 kg would sit under a "Dumbbell" label, or vice versa.
      if (g.equipment) row.equipment = g.equipment;
      g.sets.forEach((s, j) => {
        if (!row!.sets[j]) row!.sets[j] = blankSet();
        row!.sets[j].weight = s.weight_g > 0 ? gramsToKg(s.weight_g) : '';
        row!.sets[j].reps = committed && s.reps > 0 ? String(s.reps) : '';
        row!.sets[j].suggested = !committed;
      });
      if (committed) expanded[rowKey(row, rows.indexOf(row))] = true;
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
    exercises = [
      ...exercises,
      { name: '', scheme: '', kind: 'weighted', equipment: '', priority: false, alt: '', sets: [blankSet()] }
    ];
  }

  /** Typing a known exercise name into a custom row adopts its default implement. */
  function onExerciseName(i: number) {
    const ex = exercises[i];
    if (!ex.equipment) {
      const guess = exerciseEquipment(ex.name.trim());
      if (guess) {
        ex.equipment = guess;
        exercises = exercises;
      }
    }
    scheduleSave();
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

  /**
   * The day to open on boot. If today is already logged, reopen it. Otherwise
   * open the first lifting day that has no session this week, which is the
   * plan's "missed a day, shift the week" rule made automatic: on Tuesday
   * after a Monday Upper A that is Lower A; on Wednesday after a skipped
   * Tuesday it is still Lower A. Once all four are logged, the spare slot is
   * cardio.
   */
  function pickDay(): Focus {
    const todayStr = today();
    const todays = sessions.find((s) => s.date === todayStr && s.day_label);
    if (todays && DAY_LABELS.includes(todays.day_label as Focus)) return todays.day_label as Focus;
    const done = weekChecklist(sessions, LIFT_DAYS, todayStr).done;
    return LIFT_DAYS.find((d) => !done.includes(d)) ?? 'Cardio';
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
        equipment: ex.equipment,
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
      if (r) {
        log = patchLog(log, { session_id: r.id, date: sessionDate, day_label: active }, payload, cardio);
      }
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

  /** This week's lifts, for the day chips and the opening day. */
  $: week = weekChecklist(sessions, LIFT_DAYS, today());

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

  const showSignIn = () => setTimeout(() => renderGoogleButton(gisButton, clientId, onCredential), 0);

  function signOut() {
    setToken(null);
    signedIn = false;
    sessions = [];
    bodyweight = [];
    log = { sets: [], cardio: [] };
    detailCache.clear();
    saveStatus = 'idle';
    exercises = templateRows(active);
    cardioRows = [defaultCardio(active)];
    showSignIn();
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
        showSignIn();
      } else {
        errorMsg = e instanceof Error ? e.message : 'something went wrong';
      }
      return undefined;
    }
  }

  async function bootSignedIn() {
    await refreshLists();
    active = pickDay();
    await loadProfile();
    await loadDayData();
    await loadLog();
    applyTargets();
  }

  onMount(async () => {
    exercises = templateRows(active);
    cardioRows = [defaultCardio(active)];
    if (isLocalDev()) {
      // Local dev: the Worker bypasses Google auth (LOCAL_DEV in api/.dev.vars),
      // so skip sign-in entirely and boot straight into the data.
      setToken('local-dev');
      signedIn = true;
      await bootSignedIn();
      return;
    }
    await loadGis();
    if (loadToken()) {
      signedIn = true;
      scheduleRefresh();
      await bootSignedIn();
    } else {
      renderGoogleButton(gisButton, clientId, onCredential);
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
              <LoadingState label="Saving" />
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
      Four lifts a week, upper/lower. Pick a day, then {signedIn ? 'tap an exercise and log your sets — it saves as you type.' : 'sign in to log your sets right here.'}
      {#if signedIn}The trend, report and history are on the <a href="/fitness/workout/insights">insights page</a>.{/if}
    </p>
  </header>

  {#if errorMsg}
    <div class="error-banner">{errorMsg}</div>
  {/if}

  <!-- The week: each chip jumps to that day's exercises and ticks once the day
       is logged this week. Four ticks is a finished week; the fifth is a bonus. -->
  <div class="week-block">
    <div class="week-strip">
      {#each DAY_LABELS as label}
        <button
          class="day-chip"
          class:active={label === active}
          class:done={signedIn && week.done.includes(label)}
          class:optional={DAY_INFO[label].optional}
          on:click={() => selectDay(label)}
          title={DAY_INFO[label].detail}
        >
          <span class="chip-day">{DAY_INFO[label].day}</span>
          <span class="chip-focus">
            {label}
            {#if signedIn && week.done.includes(label)}<Check size={13} strokeWidth={2.5} />{/if}
          </span>
        </button>
      {/each}
    </div>
    {#if signedIn}
      <p class="week-line" class:complete={week.complete}>
        {#if week.complete}
          Week done — {week.liftsDone} of {week.liftsTotal} lifts logged. Anything more is extra.
        {:else}
          {week.liftsDone} of {week.liftsTotal} lifts logged this week.
        {/if}
      </p>
    {/if}
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
  {/if}

  <!-- Active day's exercises -->
  <section class="day-card">
    <div class="day-card-head">
      <h2>{active}</h2>
      <span class="cardio-pill">🚴 {DAY_INFO[active].cardio}</span>
    </div>

    {#if DAY_INFO[active].optional && exercises.length}
      <p class="empty-day">
        Do the cardio first. The lifts are optional: do as many as you have time for, and log only
        the sets you do.
      </p>
    {/if}
    {#if exercises.length === 0}
      <p class="empty-day">
        No lifting today. Log the cardio bout below{signedIn ? '' : ' once signed in'}, or add an
        exercise if you end up doing one.
      </p>
    {/if}

    {#if !signedIn}
      <!-- read-only plan view -->
      <ul class="exercise-list">
        {#each exercises as ex, i}
          <li class:key-lift={ex.priority}>
            <span class="ex-num">{i + 1}</span>
            <span class="ex-name">{ex.name}</span>
            {#if ex.priority}<span class="key-tag" title="Priority lift — progress this first">key</span>{/if}
            <span class="ex-scheme">{ex.scheme}</span>
            {#if ex.alt}<span class="ex-alt">{ex.alt}</span>{/if}
          </li>
        {/each}
      </ul>
      {#if exercises.length}
        <p class="key-legend">
          <span class="key-tag">key</span> — the lifts that drive the result. Progress these first;
          the rest are accessories and are what to cut when you're short on time.
        </p>
      {/if}
    {:else}
      <!-- interactive logging view -->
      <div class="log-list">
        {#each exercises as ex, i}
          <div class="log-ex" class:open={expanded[rowKey(ex, i)]} class:key-lift={ex.priority}>
            <button
              type="button"
              class="log-ex-head"
              on:click={() => (expanded = { ...expanded, [rowKey(ex, i)]: !expanded[rowKey(ex, i)] })}
            >
              <span class="ex-num">{i + 1}</span>
              {#if ex.scheme}
                <span class="ex-name">{ex.name}</span>
                {#if ex.priority}
                  <span class="key-tag" title="Priority lift — progress this first">key</span>
                {/if}
                <span class="ex-target">
                  {ex.scheme}{#if loadTargets[i]}<span class="ex-load" title={loadTargets[i]?.reason}>@ {gramsToKg(Math.round((loadTargets[i]?.kg ?? 0) * 1000))} kg</span>{/if}
                </span>
              {:else}
                <input
                  class="ex-name-input"
                  placeholder="Exercise name"
                  bind:value={ex.name}
                  on:click|stopPropagation
                  on:input={() => onExerciseName(i)}
                />
              {/if}
              {#if ex.kind === 'weighted'}
                <!-- Which implement — loads are only comparable within one. -->
                <!-- svelte-ignore a11y-click-events-have-key-events -->
                <select
                  class="ex-equip"
                  class:unset={!ex.equipment}
                  bind:value={ex.equipment}
                  on:click|stopPropagation
                  on:change={() => {
                    applyTargets();
                    scheduleSave();
                  }}
                  title="Equipment used"
                  aria-label="Equipment for {ex.name || 'this exercise'}"
                >
                  <option value="">—</option>
                  {#each EQUIPMENT_OPTIONS as opt}
                    <option value={opt}>{opt}</option>
                  {/each}
                </select>
              {/if}
              <span class="chevron" aria-hidden="true">{expanded[rowKey(ex, i)] ? '−' : '+'}</span>
              {#if ex.alt}<span class="ex-alt">{ex.alt}</span>{/if}
            </button>

            {#if expanded[rowKey(ex, i)]}
              <div class="sets">
                {#if loadTargets[i]}
                  {@const t = loadTargets[i]}
                  <p class="load-note">
                    {t?.reason}
                    {#if t?.warmups.length}
                      Warm up first with {t.warmups.map((w, k) => `${k === t.warmups.length - 1 ? 5 : 8} × ${gramsToKg(Math.round(w * 1000))} kg`).join(', then ')}, and don't log the warm-ups.
                    {/if}
                  </p>
                {/if}
                {#each ex.sets as s, j}
                  <div class="set-row">
                    <span class="set-num">{j + 1}</span>
                    {#if ex.kind === 'weighted'}
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
                    {/if}
                    <input
                      class="num"
                      class:suggested={s.suggested}
                      type="number"
                      inputmode="numeric"
                      min="0"
                      placeholder={ex.kind === 'time' ? 'sec' : 'reps'}
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
              <!-- A restored bout may name a machine no longer on the list
                   (the gym only has three). Keep its own value selectable so
                   editing an old session doesn't silently retype it. -->
              {#if c.kind && !CARDIO_OPTIONS.some((o) => o.value === c.kind)}
                <option value={c.kind}>{c.kind}</option>
              {/if}
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

  <!-- The whole week on one screen: every day's lifts and cardio, ticked once
       logged this week. Each row opens that day above. -->
  <details class="fold">
    <summary>
      <span class="fold-title">Week overview</span>
      <span class="fold-meta">{LIFT_DAYS.length} lifts + 1 spare</span>
    </summary>
    <div class="fold-body overview">
      {#each DAY_LABELS as label}
        {@const done = signedIn && week.done.includes(label)}
        <button
          type="button"
          class="ov-day"
          class:done
          class:optional={DAY_INFO[label].optional}
          class:active={label === active}
          on:click={() => selectDay(label)}
          title="Open {label}"
        >
          <span class="ov-when">{DAY_INFO[label].day}</span>
          <span class="ov-body">
            <span class="ov-title">
              {label}
              {#if done}<Check size={13} strokeWidth={2.5} />{/if}
            </span>
            <span class="ov-lifts">
              {#if DAY_TEMPLATES[label].length}
                {#each DAY_TEMPLATES[label] as ex}
                  <span class="ov-lift" class:key={ex.priority}>{ex.name} <em>{ex.scheme}</em></span>
                {/each}
              {:else}
                <span class="ov-lift">{DAY_INFO[label].detail}</span>
              {/if}
            </span>
            <span class="ov-cardio">🚴 {DAY_INFO[label].cardio}</span>
          </span>
        </button>
      {/each}
    </div>
  </details>

  {#if signedIn}
    <!-- My details: the profile the body maths runs on, and the numbers it
         gives at the current bodyweight. -->
    <section class="info-card">
      <div class="info-head">
        <h2>My details</h2>
        {#if metrics}<span class="info-meta">at {metricKg.toFixed(1)} kg</span>{/if}
      </div>
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
      {#if metrics && targets}
        <div class="rate-grid">
          <div class="rate-cell">
            <span class="r-val">{metrics.bmi}</span>
            <span class="r-label">BMI</span>
            <span class="r-sub">{metrics.bmiCategory}</span>
          </div>
          <div class="rate-cell">
            <span class="r-val">{metrics.bmr}</span>
            <span class="r-label">BMR</span>
            <span class="r-sub">kcal/day</span>
          </div>
          <div class="rate-cell">
            <span class="r-val">{metrics.tdee}</span>
            <span class="r-label">TDEE</span>
            <span class="r-sub">maintenance</span>
          </div>
          <div class="rate-cell">
            <span class="r-val">{targets.protein_g}<small> g</small></span>
            <span class="r-label">protein</span>
            <span class="r-sub">per day · 2 g/kg</span>
          </div>
        </div>
        <p class="hint">
          The calorie target ({targets.calories} kcal) and macros are here if you want them, but the
          plan is run on the trend weight and the food rules, not on counting.
      {/if}
    </section>

    <!-- Burned today: a rough burn for this date. The active day's rows count
         live; any other session already logged on the same date comes from
         the log. -->
    <section class="info-card">
      <div class="info-head">
        <h2>Burned today</h2>
        <span class="info-meta">rough estimate</span>
      </div>
      <div class="rate-grid">
        <div class="rate-cell">
          <span class="r-val">{dayLiftKcal}<small> kcal</small></span>
          <span class="r-label">lifting</span>
          <span class="r-sub">{daySets} {daySets === 1 ? 'set' : 'sets'} · ~2.5 min each</span>
        </div>
        <div class="rate-cell">
          <span class="r-val">{dayCardioKcal}<small> kcal</small></span>
          <span class="r-label">cardio</span>
          <span class="r-sub">logged or estimated</span>
        </div>
        <div class="rate-cell">
          <span class="r-val">{dayKcal}<small> kcal</small></span>
          <span class="r-label">today</span>
          <span class="r-sub">total</span>
        </div>
      </div>
      {#if otherSetsToday || otherCardioToday}
        <p class="hint">Includes the other session logged on this date.</p>
      {/if}
    </section>

    <!-- Everything else — the trend, the training report, every fitted number
         and the history — lives on its own page. -->
    <a class="link-card" href="/fitness/workout/insights">
      <span class="link-title">Insights &amp; history</span>
      <span class="link-meta">trend · report · all the numbers · past sessions</span>
      <span class="link-arrow" aria-hidden="true">&rarr;</span>
    </a>
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
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.625rem;
  }
  /* Shares the global .page-title pen treatment; only the size is trimmed so
     the title sits level with the save pill beside it. */
  .page-title {
    font-size: 2.25rem;
  }

  .save-pill {
    display: inline-flex;
    align-items: center;
    justify-content: flex-start;
    gap: 0.3rem;
    width: 5.75rem;
    height: 1.125rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-tertiary);
    opacity: 0;
    transition: opacity 0.2s;
  }
  .save-pill.on { opacity: 1; }
  .save-pill :global(.loader) {
    transform: scale(0.78);
    transform-origin: left center;
  }
  .save-pill[data-status='saved'] { color: var(--blueprint); }
  .save-pill[data-status='error'] { color: #e74c3c; }

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
  .week-block { display: flex; flex-direction: column; gap: 0.6rem; }
  .week-strip {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 0.5rem;
  }
  /* A logged day keeps its outline but earns the accent tick; the optional day
     is dashed so the eye reads four solid slots plus one spare. */
  .day-chip.optional { border-style: dashed; }
  .day-chip.done { border-color: var(--blueprint); }
  .day-chip.done .chip-focus { color: var(--blueprint); }
  .chip-focus { display: inline-flex; align-items: center; gap: 0.3rem; }
  .week-line {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.03em;
    color: var(--text-tertiary);
  }
  .week-line.complete { color: var(--blueprint); }
  .day-chip {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.25rem;
    padding: 0.6rem 0.65rem;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    cursor: var(--cursor-pointer);
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
  }
  .day-chip:hover:not(:disabled) { border-color: var(--blueprint); }
  .day-chip.active { border-color: var(--blueprint); background: var(--blueprint-tint); }
  .chip-day {
    font-family: var(--font-mono);
    font-size: 0.6rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }
  .chip-focus { font-size: 0.85rem; font-weight: 600; color: var(--text-primary); }
  .day-chip.active .chip-focus { color: var(--blueprint); }

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

  /* ── Profile (inside the numbers fold) ──────────────────── */
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
  .empty-day {
    margin: 0;
    padding: 0.9rem 1rem;
    font-size: 0.875rem;
    line-height: 1.5;
    color: var(--text-tertiary);
    border-bottom: 1px solid var(--border-subtle);
  }

  /* read-only plan list */
  .exercise-list { display: flex; flex-direction: column; list-style: none; padding: 0; margin: 0; }
  /* Flex, not a fixed grid: the key tag and scheme are both optional, and a
     fixed column count pushes later children onto a second row when one is
     absent. */
  .exercise-list li {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    padding: 0.7rem 1rem;
    border-bottom: 1px solid var(--border-subtle);
    border-left: 2px solid transparent;
  }
  .exercise-list li .ex-num { flex: none; width: 1.5rem; }
  .exercise-list li .ex-name { flex: 1; min-width: 0; }
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
  .log-ex { border-bottom: 1px solid var(--border-subtle); border-left: 2px solid transparent; }
  .log-ex:last-of-type { border-bottom: none; }
  .log-ex-head .ex-num { flex: none; width: 1.5rem; }
  .log-ex-head .ex-name { flex: 1; min-width: 0; }
  .log-ex-head .chevron { margin-left: auto; }
  /* Flex for the same reason as the plan list — the target, equipment picker
     and key tag are each conditional, so a fixed column count wrapped the
     chevron onto its own row whenever one was missing. */
  .log-ex-head {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.75rem 1rem;
    background: transparent;
    border: none;
    cursor: var(--cursor-pointer);
    text-align: left;
  }
  .log-ex.open .log-ex-head { background: var(--blueprint-tint); }
  .log-ex-head .ex-name { color: var(--text-primary); }
  .ex-target {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-tertiary);
  }
  .ex-load { margin-left: 0.4em; color: var(--blueprint); }
  .load-note {
    margin: 0 0 0.5rem;
    font-size: 0.75rem;
    line-height: 1.45;
    color: var(--text-tertiary);
  }
  .ex-name-input { font-weight: 600; }
  /* The fallback for a contested implement: a full-width footnote under the
     row header, so the name, target, and picker keep their single line. */
  .ex-alt {
    flex-basis: 100%;
    padding-left: 2.25rem;
    font-size: 0.72rem;
    line-height: 1.4;
    color: var(--text-tertiary);
    font-style: italic;
  }
  .log-ex-head { flex-wrap: wrap; row-gap: 0.15rem; }
  .exercise-list li { flex-wrap: wrap; row-gap: 0.15rem; }

  /* Priority lifts — the compounds that carry the session. Marked with an
     accent rail rather than a colour swap, so the list still scans as one
     group and the tag stays readable next to the exercise name. */
  .key-tag {
    font-family: var(--font-mono);
    font-size: 0.62rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--blueprint);
    border: 1px solid var(--blueprint);
    border-radius: 999px;
    padding: 0.05rem 0.35rem;
    flex: none;
  }
  .log-ex.key-lift,
  .exercise-list li.key-lift { border-left-color: var(--blueprint); }
  .exercise-list li.key-lift .ex-name { color: var(--text-primary); }
  .key-legend {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
    margin: 0.75rem 0 0;
    font-size: 0.75rem;
    line-height: 1.5;
    color: var(--text-tertiary);
  }
  /* Implement picker — quiet until set, since loads only compare within one. */
  .ex-equip {
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text-secondary);
    background: var(--surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: 999px;
    padding: 0.15rem 0.4rem;
    cursor: var(--cursor-pointer);
    max-width: 7.5rem;
  }
  .ex-equip.unset { color: var(--text-tertiary); border-style: dashed; }
  .ex-equip:focus-visible { outline: 2px solid var(--blueprint); outline-offset: 2px; }
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
    cursor: var(--cursor-pointer);
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
    cursor: var(--cursor-pointer);
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

  /* ── Week overview ──────────────────────────────────────── */
  .overview { display: flex; flex-direction: column; gap: 0.5rem; padding: 0.75rem; }
  .ov-day {
    display: grid;
    grid-template-columns: 4rem minmax(0, 1fr);
    gap: 0.75rem;
    width: 100%;
    padding: 0.7rem 0.85rem;
    text-align: left;
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-left: 2px solid transparent;
    border-radius: 0.5rem;
    cursor: var(--cursor-pointer);
    transition: border-color 0.15s, background 0.15s;
  }
  .ov-day:hover { border-color: var(--border-strong); }
  .ov-day.active { border-left-color: var(--blueprint); background: var(--blueprint-tint); }
  .ov-day.optional { border-style: dashed; border-left-style: solid; }
  .ov-when {
    font-family: var(--font-mono);
    font-size: 0.66rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-tertiary);
    padding-top: 0.15rem;
  }
  .ov-body { display: flex; flex-direction: column; gap: 0.35rem; min-width: 0; }
  .ov-title {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .ov-day.done .ov-title { color: var(--blueprint); }
  .ov-lifts { display: flex; flex-wrap: wrap; gap: 0.25rem 0.75rem; }
  .ov-lift { font-size: 0.8rem; line-height: 1.45; color: var(--text-secondary); }
  .ov-lift.key { color: var(--text-primary); }
  .ov-lift em {
    font-family: var(--font-mono);
    font-style: normal;
    font-size: 0.68rem;
    color: var(--text-tertiary);
  }
  .ov-cardio { font-size: 0.74rem; font-style: italic; color: var(--text-tertiary); }

  /* ── Folds (rules / numbers / history) ──────────────────── */
  .fold { border: 1px solid var(--border); border-radius: 0.625rem; }
  .fold summary {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    padding: 0.85rem 1rem;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
    cursor: var(--cursor-pointer);
    list-style: none;
  }
  .fold summary::-webkit-details-marker { display: none; }
  .fold-title { flex: none; }
  .fold-meta {
    margin-left: auto;
    font-size: 0.7rem;
    font-weight: 500;
    letter-spacing: 0.02em;
    text-transform: none;
    color: var(--text-tertiary);
  }
  .fold summary::after { content: '+'; color: var(--text-tertiary); font-weight: 400; }
  .fold[open] summary::after { content: '–'; }
  .fold[open] summary { color: var(--text-primary); border-bottom: 1px solid var(--border-subtle); }

  .fold-body { padding: 1.25rem 1rem; }

  /* ── Body maths grid (details card, burned today) ──────── */
  .rate-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(7.5rem, 1fr));
    gap: 0.6rem;
  }
  .rate-cell {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: 0.65rem 0.7rem;
    border: 1px solid var(--border-subtle);
    border-radius: 0.5rem;
  }
  .r-val { font-family: var(--font-mono); font-size: 1rem; font-weight: 700; color: var(--text-primary); }
  .r-val small { font-size: 0.64rem; font-weight: 500; color: var(--text-tertiary); }
  .r-label { font-size: 0.62rem; letter-spacing: 0.05em; text-transform: uppercase; color: var(--text-tertiary); }
  .r-sub { font-family: var(--font-mono); font-size: 0.62rem; line-height: 1.35; color: var(--text-faint); }

  .hint { font-size: 0.8125rem; color: var(--text-tertiary); margin: 0 0 0.75rem; }
  .rate-grid + .hint { margin-top: 0.85rem; }

  /* ── Info cards (details, burned today) + insights link ─── */
  .info-card {
    border: 1px solid var(--border);
    border-radius: 0.625rem;
    padding: 1rem;
  }
  .info-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.875rem;
  }
  .info-head h2 { font-size: 1rem; font-weight: 600; color: var(--text-primary); margin: 0; }
  .info-meta {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-tertiary);
    white-space: nowrap;
  }
  .info-card .profile-grid { margin-bottom: 1rem; }
  .info-card .hint { margin: 0.85rem 0 0; }

  .link-card {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.875rem 1rem;
    border: 1px solid var(--border);
    border-radius: 0.625rem;
    color: var(--text-primary);
    text-decoration: none;
    transition: border-color 0.15s;
  }
  .link-card:hover { border-color: var(--border-strong); }
  .link-title { font-size: 0.95rem; font-weight: 600; }
  .link-meta {
    flex: 1;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .link-arrow { color: var(--text-tertiary); transition: transform 0.15s; }
  .link-card:hover .link-arrow { transform: translateX(3px); }

  @media (max-width: 640px) {
    .page { gap: 1.15rem; }
    .page-title { font-size: 1.75rem; }
    .week-strip { grid-template-columns: repeat(3, 1fr); gap: 0.4rem; }
    .day-chip { padding: 0.5rem 0.5rem; }
    .link-meta { display: none; }

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
    .ex-equip { order: 1; }
    .ex-target { order: 3; flex-basis: 100%; padding-left: 2rem; }
    .ex-alt { order: 4; padding-left: 2rem; }
    /* Weekday above the body rather than beside it at phone width. */
    .ov-day { grid-template-columns: 1fr; gap: 0.3rem; padding: 0.65rem 0.75rem; }
    .overview { padding: 0.5rem; }

    /* Cardio: the machine name shares a row with two number fields and a
       delete button, which crushes the select to a bare chevron at this width
       — and the names are long ("Crosstrainer (intervals)"). Give it its own
       line and let the numbers share the next. */
    .cardio-row { flex-wrap: wrap; }
    .cardio-kind { flex-basis: 100%; }
    .day-card-head { padding: 0.75rem 0.75rem; }
    .sets { padding: 0.5rem 0.75rem 0.9rem; }
    .set-row { gap: 0.4rem; }
    .num { width: 100%; min-width: 0; flex: 1; }
    .set-num { width: 1rem; }
  }

  @media (max-width: 380px) {
    .week-strip { grid-template-columns: repeat(2, 1fr); }
  }
</style>
