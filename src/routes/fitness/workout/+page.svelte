<svelte:head>
  <title>Workout — Kashif</title>
  <meta name="description" content="A 5–6 day push/pull/legs program with cardio mixed in — built to gain muscle while cutting." />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { LogOut, Check, Plus, Trash2 } from 'lucide-svelte';
  import LoadingState from '$lib/components/LoadingState.svelte';
  import WeightChart from '$lib/components/WeightChart.svelte';
  import { setToken, loadToken, AuthError } from '$lib/splitterApi';
  import { isLocalDev } from '$lib/apiBase';
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
    weightInsights,
    trainingCadence,
    weightForBmi,
    type RateBand
  } from '$lib/fitnessInsights';
  import {
    DAY_TEMPLATES,
    setsFromScheme,
    kgToGrams,
    gramsToKg,
    weeklyAverages,
    CARDIO_OPTIONS,
    CARDIO_DEFAULTS,
    cardioMet,
    exerciseKind,
    exerciseEquipment,
    EQUIPMENT_OPTIONS,
    type DayLabel,
    type ExerciseKind,
    type Equipment,
    type SessionSummary,
    type SessionDetail,
    type BodyweightEntry,
    type WeeklyAverage
  } from '$lib/workout';

  type Focus = DayLabel;

  const clientId = env.PUBLIC_GOOGLE_CLIENT_ID ?? '';

  const cardio: Record<Focus, string> = {
    Push: '20 min crosstrainer, steady',
    Pull: '20 min crosstrainer intervals — 30s hard / 90s easy × 10',
    Legs: '15 min easy cycle (legs are already done)'
  };

  const split: { day: string; focus: Focus; detail: string }[] = [
    { day: 'Day 1', focus: 'Push', detail: 'chest / shoulders / triceps / abs' },
    { day: 'Day 2', focus: 'Pull', detail: 'back / rear delts / biceps / core' },
    { day: 'Day 3', focus: 'Legs', detail: 'quads / hamstrings / calves' },
    { day: 'Day 4', focus: 'Push', detail: 'chest / shoulders / triceps / abs' },
    { day: 'Day 5', focus: 'Pull', detail: 'back / rear delts / biceps / core' },
    { day: 'Day 6', focus: 'Legs', detail: 'quads / hamstrings / calves' }
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
  type ExerciseRow = {
    name: string;
    scheme: string;
    kind: ExerciseKind;
    equipment: Equipment;
    /** Compound / previously-neglected lift — progress these first. */
    priority: boolean;
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

  /** Group a detail's flat set rows by exercise + implement, preserving order.
   *  Splitting on equipment keeps a machine pec fly and dumbbell flyes as two
   *  separate blocks — their loads aren't comparable. */
  function groupSets(detail: SessionDetail) {
    const groups: {
      exercise: string;
      equipment: Equipment;
      sets: { reps: number; weight_g: number }[];
    }[] = [];
    for (const s of detail.sets) {
      const equipment = s.equipment ?? '';
      let g = groups.find((x) => x.exercise === s.exercise && x.equipment === equipment);
      if (!g) {
        g = { exercise: s.exercise, equipment, sets: [] };
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
      { name: '', scheme: '', kind: 'weighted', equipment: '', priority: false, sets: [blankSet()] }
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
   * The day to open on boot, read off the history: if today is already logged,
   * that's the current day — reopen it; otherwise continue the Push → Pull →
   * Legs rotation from the most recent session (so a fresh visit lands on the
   * right day instead of always defaulting to Push). No history → Day 1.
   */
  function pickDay(): Focus {
    const todayStr = today();
    const todays = sessions.find((s) => s.date === todayStr && s.day_label);
    if (todays) return todays.day_label as Focus;
    const last = sessions.find((s) => s.date < todayStr && s.day_label);
    if (!last) return 'Push';
    const i = FOCUSES.indexOf(last.day_label as Focus);
    return i >= 0 ? FOCUSES[(i + 1) % FOCUSES.length] : 'Push';
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

  // History grouped by month (newest first), each row carrying display-ready
  // date parts and that day's bodyweight — the list renders from this directly.
  type HistRow = SessionSummary & { dom: string; wd: string; mon: string; bw: number | undefined };
  $: historyGroups = (() => {
    const groups: { key: string; label: string; rows: HistRow[] }[] = [];
    for (const s of sessions) {
      const d = new Date(s.date + 'T00:00:00');
      const row: HistRow = {
        ...s,
        dom: String(d.getDate()),
        wd: d.toLocaleDateString('en-GB', { weekday: 'short' }),
        mon: d.toLocaleDateString('en-GB', { month: 'short' }),
        bw: bwByDate.get(s.date)
      };
      const key = s.date.slice(0, 7);
      let grp = groups.find((x) => x.key === key);
      if (!grp) {
        grp = {
          key,
          label: d.toLocaleDateString('en-GB', { month: 'long', year: 'numeric' }),
          rows: []
        };
        groups.push(grp);
      }
      grp.rows.push(row);
    }
    return groups;
  })();

  // ---- insights ------------------------------------------------------------
  // All the trend maths lives in `fitnessInsights.ts`; the page only formats it.
  $: insights = weightInsights(bodyweight, profile, weekly, today());
  $: cadence = trainingCadence(sessions, today());
  /** Upper edge of the normal BMI range at this height — the chart's goal line. */
  $: normalBmiKg = weightForBmi(24.9, profile.height_cm);

  const BAND_NOTE: Record<RateBand, string> = {
    gaining: 'Trending up over this window. Expected on a bulk; on a cut it means intake is above target.',
    holding: 'Holding steady — the trend is flat.',
    slow: 'Losing, but under 0.5 %/wk. Fine if that is the plan, otherwise the deficit is too small to show.',
    sustainable: 'In the 0.5–1.0 %/wk band — fast enough to matter, slow enough to keep muscle.',
    aggressive: 'Above 1.0 %/wk. Workable for a short block; watch the priority lifts for strength drops.',
    'very fast': 'Over 1.25 %/wk. Fast enough to cost muscle — consider easing the deficit.'
  };

  const BAND_LABEL: Record<RateBand, string> = {
    gaining: 'gaining',
    holding: 'flat',
    slow: 'slow',
    sustainable: 'on target',
    aggressive: 'aggressive',
    'very fast': 'too fast'
  };

  /** Bands to flag amber rather than treat as on-plan. */
  const BAND_WARN: RateBand[] = ['gaining', 'aggressive', 'very fast'];

  const fmtSigned = (v: number | null, dp = 1): string =>
    v === null ? '—' : (v > 0 ? '+' : '') + v.toFixed(dp);

  /** ISO date → "5 Sep 2026"; the projections are months out, so the year earns its place. */
  const fmtDate = (iso: string | null): string =>
    iso
      ? new Date(iso + 'T00:00:00').toLocaleDateString('en-GB', {
          day: 'numeric',
          month: 'short',
          year: 'numeric'
        })
      : '—';

  // How the measured deficit compares with the one the goal asks for. Only
  // called out past ±200 kcal/day, which is inside the noise of MET estimates.
  $: deficitNote = (() => {
    const gap = insights.deficitGap;
    if (gap === null || insights.plannedDeficit === 0) return '';
    if (gap > 200)
      return 'Losing faster than the plan implies — intake is under target, or your activity multiplier is set too low.';
    if (gap < -200)
      return 'Losing slower than the plan implies — intake is likely above target, or TDEE is overestimated.';
    return 'Measured loss matches the planned deficit.';
  })();

  // Newest first, each week annotated with its change vs the week before.
  $: weeklyWithDelta = [...weekly]
    .map((w, i) => ({
      ...w,
      delta: i > 0 ? Math.round((w.avgKg - weekly[i - 1].avgKg) * 10) / 10 : null
    }))
    .reverse();

  $: latestWeek = weekly.length ? weekly[weekly.length - 1] : null;
  $: weekDelta =
    weekly.length > 1
      ? Math.round((weekly[weekly.length - 1].avgKg - weekly[weekly.length - 2].avgKg) * 10) / 10
      : null;

  const fmtDelta = (d: number | null): string => (d === null ? '—' : (d > 0 ? '+' : '') + d.toFixed(1));

  let openId: number | null = null;
  let openDetail: SessionDetail | null = null;
  let loadingDetail = false;

  // One-line stats for the currently expanded history row.
  $: openStats = openDetail
    ? {
        exercises: new Set(openDetail.sets.map((x) => x.exercise)).size,
        sets: openDetail.sets.length,
        cardio: openDetail.cardio?.length ?? 0
      }
    : null;

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

  function setLabel(reps: number, weight_g: number, name: string): string {
    if (exerciseKind(name) === 'time') return `${reps}s`;
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
    active = pickDay();
    await loadProfile();
    await loadDayData();
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
        on:click={() => selectDay(row.focus)}
        title={row.detail}
      >
        <span class="chip-day">{row.day}</span>
        <span class="chip-focus">{row.focus}</span>
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
          <li class:key-lift={ex.priority}>
            <span class="ex-num">{i + 1}</span>
            <span class="ex-name">{ex.name}</span>
            {#if ex.priority}<span class="key-tag" title="Priority lift — progress this first">key</span>{/if}
            <span class="ex-scheme">{ex.scheme}</span>
          </li>
        {/each}
      </ul>
      <p class="key-legend">
        <span class="key-tag">key</span> — the lifts that drive the result. Progress these first;
        the rest are accessories and are what to cut when you're short on time.
      </p>
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
                <span class="ex-target">{ex.scheme}</span>
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
                  on:change={scheduleSave}
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
            </button>

            {#if expanded[rowKey(ex, i)]}
              <div class="sets">
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
        moderate effort (8-15 reps). Cardio counts your logged (or estimated) kcal.
      </p>
    </div>
  {/if}

  {#if signedIn}
    <!-- Bodyweight trend, insights and projections -->
    <details class="fold" open>
      <summary>
        <span class="fold-title">Bodyweight trend</span>
        {#if insights.trendKg !== null}
          <span class="fold-meta">{insights.trendKg} kg trend · {fmtSigned(insights.primary.kgPerWeek, 2)} kg/wk</span>
        {/if}
      </summary>
      <div class="fold-body">
        {#if insights.points.length}
          <div class="trend-stats">
            <div class="trend-stat">
              <span class="ts-val">{insights.trendKg}<small> kg</small></span>
              <span class="ts-label">trend weight</span>
              <span class="ts-sub">scale {insights.latestKg} kg · {fmtDate(insights.latestDate)}</span>
            </div>
            <div class="trend-stat" class:down={(weekDelta ?? 0) < 0} class:up={(weekDelta ?? 0) > 0}>
              <span class="ts-val">{fmtDelta(weekDelta)}<small> kg</small></span>
              <span class="ts-label">vs last week</span>
              <span class="ts-sub">weekly averages</span>
            </div>
            <div
              class="trend-stat"
              class:down={(insights.totalChange ?? 0) < 0}
              class:up={(insights.totalChange ?? 0) > 0}
            >
              <span class="ts-val">{fmtSigned(insights.totalChange)}<small> kg</small></span>
              <span class="ts-label">since start</span>
              <span class="ts-sub">
                {fmtSigned(insights.totalChangePct)}% · {insights.consistency.spanDays} days · from {insights.startKg} kg
              </span>
            </div>
            <div
              class="trend-stat"
              class:down={(insights.primary.kgPerWeek ?? 0) < 0}
              class:up={(insights.primary.kgPerWeek ?? 0) > 0}
            >
              <span class="ts-val">{fmtSigned(insights.primary.kgPerWeek, 2)}<small> kg/wk</small></span>
              <span class="ts-label">current rate</span>
              <span class="ts-sub">{insights.primary.label} · fitted</span>
            </div>
          </div>

          {#if insights.primary.band}
            <p class="assess" class:warn={BAND_WARN.includes(insights.primary.band)}>
              <span class="assess-chip">
                {BAND_LABEL[insights.primary.band]}
                {#if insights.primary.pctPerWeek !== null}
                  · {fmtSigned(insights.primary.pctPerWeek, 2)}%/wk
                {/if}
              </span>
              {BAND_NOTE[insights.primary.band]}
            </p>
          {/if}

          <WeightChart
            points={insights.points}
            ema={insights.ema}
            targetKg={normalBmiKg}
            targetLabel={`BMI 25 · ${normalBmiKg} kg`}
          />

          <h4 class="ins-head">Rate by window</h4>
          <div class="rate-grid">
            {#each insights.rates as r}
              <div
                class="rate-cell"
                class:muted={r.kgPerWeek === null}
                class:down={(r.kgPerWeek ?? 0) < 0}
                class:up={(r.kgPerWeek ?? 0) > 0}
              >
                <span class="r-val">{fmtSigned(r.kgPerWeek, 2)}<small> kg/wk</small></span>
                <span class="r-label">{r.label}</span>
                <span class="r-sub">
                  {#if r.kgPerWeek === null}
                    not enough data
                  {:else}
                    {fmtSigned(r.pctPerWeek, 2)}%/wk · {r.n} weigh-ins · fit {r.r2}
                  {/if}
                </span>
              </div>
            {/each}
          </div>
          <p class="hint">
            Each rate is a least-squares fit over its own window, not a first-to-last subtraction —
            one heavy meal can't move it. <strong>Fit</strong> is r²: how much of the movement the
            line explains, so a low number means the window is mostly noise.
          </p>

          <h4 class="ins-head">Energy balance</h4>
          <div class="ins-rows">
            <div class="ins-row">
              <span>Deficit implied by the trend</span>
              <strong>
                {insights.primary.kcalPerDay === null ? '—' : `${insights.primary.kcalPerDay} kcal/day`}
              </strong>
            </div>
            <div class="ins-row">
              <span>Deficit this goal asks for</span>
              <strong>{insights.plannedDeficit} kcal/day</strong>
            </div>
            <div class="ins-row">
              <span>Gap</span>
              <strong class:warn={Math.abs(insights.deficitGap ?? 0) > 200}>
                {insights.deficitGap === null ? '—' : `${fmtSigned(insights.deficitGap, 0)} kcal/day`}
              </strong>
            </div>
          </div>
          {#if deficitNote}<p class="hint">{deficitNote}</p>{/if}

          <h4 class="ins-head">
            Projections
            <small>at {fmtSigned(insights.primary.kgPerWeek, 2)} kg/wk</small>
          </h4>
          {#if (insights.primary.kgPerWeek ?? 0) < 0}
            <div class="ins-rows">
              {#each insights.projections as pr}
                <div class="ins-row">
                  <span>{pr.label} <em>{pr.targetKg} kg</em></span>
                  <strong>
                    {#if pr.reached}
                      already there
                    {:else if pr.date}
                      {fmtDate(pr.date)} · {pr.weeks} wk
                    {:else}
                      not on this trend
                    {/if}
                  </strong>
                </div>
              {/each}
            </div>
            {#if insights.forecast.length}
              <div class="forecast">
                {#each insights.forecast as f}
                  <div class="fc">
                    <span class="fc-val">{f.kg}<small> kg</small></span>
                    <span class="fc-label">in {f.weeks} wk</span>
                  </div>
                {/each}
              </div>
            {/if}
          {:else}
            <p class="hint">Projections need a downward trend — there is nothing to extrapolate yet.</p>
          {/if}
          <p class="hint">
            Straight-line extrapolation of the {insights.primary.label} fit at 7700 kcal per kg, with
            BMI targets taken from your height. A real cut slows as you get lighter, so read these as
            the optimistic end.
          </p>

          <h4 class="ins-head">Signals</h4>
          <ul class="signal-list">
            {#if insights.stalled}
              <li class="warn">
                The last two weeks are flat while the whole log is down — either a normal water-weight
                stall, or the deficit has drifted shut. Give it another week before changing anything.
              </li>
            {/if}
            {#if insights.volatility !== null}
              <li>
                Day-to-day swing around the trend is ±{insights.volatility} kg, so a single weigh-in
                inside that range carries no information.
              </li>
            {/if}
            <li>
              Logged {insights.consistency.last30} of the last 30 days ({insights.consistency.coverage30}%)
              · {insights.consistency.streak}-day streak · longest gap {insights.consistency.longestGap} days.
            </li>
            {#if (insights.consistency.daysSinceLast ?? 0) > 2}
              <li class="warn">
                Last weigh-in was {insights.consistency.daysSinceLast} days ago — the trend weight goes
                stale quickly.
              </li>
            {/if}
            {#if insights.extremes.best && insights.extremes.best.delta < 0}
              <li>
                Best week: {insights.extremes.best.delta.toFixed(1)} kg, week of {insights.extremes.best.weekStart}.
              </li>
            {/if}
            {#if insights.extremes.worst && insights.extremes.worst.delta > 0}
              <li>
                Biggest gain: +{insights.extremes.worst.delta.toFixed(1)} kg, week of {insights.extremes.worst.weekStart}.
              </li>
            {/if}
          </ul>

          <h4 class="ins-head">Weekly averages</h4>
          <p class="hint">Track this, ignore daily swings.</p>
          <div class="weekly-list">
            {#each weeklyWithDelta as w}
              <div class="weekly-row">
                <span class="weekly-week">week of {w.weekStart}</span>
                <span class="weekly-count">{w.count} {w.count === 1 ? 'entry' : 'entries'}</span>
                <span class="weekly-delta" class:down={(w.delta ?? 0) < 0} class:up={(w.delta ?? 0) > 0}>
                  {#if w.delta === null}—{:else}{w.delta < 0 ? '▼' : '▲'} {Math.abs(w.delta).toFixed(1)}{/if}
                </span>
                <span class="weekly-avg">{w.avgKg} kg</span>
              </div>
            {/each}
          </div>
        {:else}
          <p class="hint">No bodyweight entries yet — add one up top.</p>
        {/if}
      </div>
    </details>

    <!-- Training cadence — frequency and focus balance from the session list -->
    <details class="fold">
      <summary>
        <span class="fold-title">Training cadence</span>
        {#if cadence.total}<span class="fold-meta">{cadence.perWeek} sessions/wk</span>{/if}
      </summary>
      <div class="fold-body">
        {#if cadence.total}
          <div class="rate-grid">
            <div class="rate-cell">
              <span class="r-val">{cadence.last7}</span>
              <span class="r-label">last 7 days</span>
              <span class="r-sub">sessions logged</span>
            </div>
            <div class="rate-cell">
              <span class="r-val">{cadence.perWeek}<small> /wk</small></span>
              <span class="r-label">last 4 weeks</span>
              <span class="r-sub">{cadence.last28} sessions</span>
            </div>
            <div class="rate-cell" class:up={(cadence.daysSinceLast ?? 0) > 2}>
              <span class="r-val">{cadence.daysSinceLast ?? '—'}</span>
              <span class="r-label">days since last</span>
              <span class="r-sub">{cadence.streak}-day streak</span>
            </div>
            <div class="rate-cell">
              <span class="r-val">{cadence.total}</span>
              <span class="r-label">all time</span>
              <span class="r-sub">longest gap {cadence.longestGap} days</span>
            </div>
          </div>

          <h4 class="ins-head">Focus balance <small>last 4 weeks</small></h4>
          <div class="focus-bars">
            {#each cadence.focus as f}
              <div class="focus-row">
                <span class="focus-label">{f.label}</span>
                <span class="focus-track">
                  <span
                    class="focus-fill"
                    style="width: {Math.max(...cadence.focus.map((x) => x.count), 1) > 0
                      ? (f.count / Math.max(...cadence.focus.map((x) => x.count), 1)) * 100
                      : 0}%"
                  ></span>
                </span>
                <span class="focus-count">{f.count}</span>
              </div>
            {/each}
          </div>
          <p class="hint">
            The plan runs each focus twice a week, so four weeks of it is eight of each. A focus
            sitting well under the other two is the one going backwards.
          </p>
        {:else}
          <p class="hint">No sessions logged yet.</p>
        {/if}
      </div>
    </details>

    <!-- History -->
    <details class="fold">
      <summary>
        <span class="fold-title">History</span>
        {#if sessions.length}
          <span class="fold-meta">{sessions.length} {sessions.length === 1 ? 'session' : 'sessions'}</span>
        {/if}
      </summary>
      <div class="fold-body">
        {#if sessions.length === 0}
          <p class="hint">No sessions logged yet.</p>
        {:else}
          <div class="history-list">
            {#each historyGroups as grp}
              <div class="history-month">{grp.label}</div>
              {#each grp.rows as s (s.id)}
                <div class="history-item" class:open={openId === s.id} class:today={s.date === today()}>
                  <button
                    class="history-head"
                    on:click={() => toggleSession(s.id)}
                    aria-expanded={openId === s.id}
                  >
                    <span class="hist-date">
                      <span class="hist-dom">{s.dom}</span>
                      <span class="hist-sub">{s.wd} {s.mon}</span>
                    </span>
                    {#if s.day_label}<span class="hist-day">{s.day_label}</span>{/if}
                    {#if s.bw !== undefined}
                      <span class="hist-bw">{gramsToKg(s.bw)}<small> kg</small></span>
                    {/if}
                    <span class="hist-chevron" aria-hidden="true">{openId === s.id ? '−' : '+'}</span>
                  </button>
                  <button class="icon-btn" on:click={() => deleteSession(s.id)} title="Delete session + bodyweight">
                    <Trash2 size={15} />
                  </button>
                  {#if openId === s.id}
                    <div class="history-detail">
                      {#if loadingDetail}
                        <div class="hint"><LoadingState label="Loading session" /></div>
                      {:else if openDetail}
                        {#if openStats}
                          <div class="detail-stats">
                            {openStats.exercises} {openStats.exercises === 1 ? 'exercise' : 'exercises'} ·
                            {openStats.sets} {openStats.sets === 1 ? 'set' : 'sets'}
                            {#if openStats.cardio}&nbsp;· {openStats.cardio} cardio{/if}
                          </div>
                        {/if}
                        {#each groupSets(openDetail) as g}
                          <div class="detail-ex">
                            <span class="detail-ex-name">
                              {g.exercise}
                              {#if g.equipment}<span class="detail-ex-equip">{g.equipment}</span>{/if}
                            </span>
                            <div class="detail-sets">
                              {#each g.sets as st}
                                <span class="set-badge">{setLabel(st.reps, st.weight_g, g.exercise)}</span>
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
  .ex-name-input { font-weight: 600; }

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

  /* ── Bodyweight trend + insights ────────────────────────── */
  .trend-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(7.5rem, 1fr));
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  .trend-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.15rem;
    padding: 0.75rem 0.5rem;
    border: 1px solid var(--border-subtle);
    border-radius: 0.5rem;
    text-align: center;
  }
  .ts-val { font-family: var(--font-mono); font-size: 1.15rem; font-weight: 700; color: var(--text-primary); }
  .ts-val small { font-size: 0.68rem; font-weight: 500; color: var(--text-tertiary); }
  .ts-label { font-size: 0.64rem; letter-spacing: 0.05em; text-transform: uppercase; color: var(--text-tertiary); }
  .ts-sub {
    font-family: var(--font-mono);
    font-size: 0.62rem;
    line-height: 1.35;
    color: var(--text-faint);
  }
  .trend-stat.down .ts-val { color: var(--blueprint, #6ea8fe); }
  .trend-stat.up .ts-val { color: #e67e22; }

  /* The one-line verdict on the current rate. */
  .assess {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.5rem;
    margin: 0 0 1.25rem;
    padding: 0.7rem 0.85rem;
    border: 1px solid var(--blueprint);
    border-radius: 0.5rem;
    background: var(--blueprint-tint);
    font-size: 0.8125rem;
    line-height: 1.5;
    color: var(--text-secondary);
  }
  .assess.warn { border-color: #e67e22; background: rgb(230 126 34 / 10%); }
  .assess-chip {
    flex: none;
    font-family: var(--font-mono);
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-primary);
  }

  .ins-head {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    margin: 1.5rem 0 0.6rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
  }
  .ins-head small {
    font-size: 0.66rem;
    font-weight: 400;
    letter-spacing: 0.02em;
    text-transform: none;
    color: var(--text-faint);
  }

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
  .rate-cell.down .r-val { color: var(--blueprint, #6ea8fe); }
  .rate-cell.up .r-val { color: #e67e22; }
  .rate-cell.muted .r-val { color: var(--text-faint); }

  .ins-rows { display: flex; flex-direction: column; }
  .ins-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.85rem;
    padding: 0.45rem 0;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 0.84rem;
    color: var(--text-secondary);
  }
  .ins-row:last-child { border-bottom: none; }
  .ins-row em { font-family: var(--font-mono); font-style: normal; font-size: 0.72rem; color: var(--text-faint); }
  .ins-row strong {
    flex: none;
    font-family: var(--font-mono);
    font-size: 0.8rem;
    font-weight: 600;
    text-align: right;
    color: var(--text-primary);
  }
  .ins-row strong.warn { color: #e67e22; }

  .forecast {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(4.75rem, 1fr));
    gap: 0.5rem;
    margin-top: 0.85rem;
  }
  .fc {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.1rem;
    padding: 0.55rem 0.4rem;
    border: 1px dashed var(--border-subtle);
    border-radius: 0.5rem;
  }
  .fc-val { font-family: var(--font-mono); font-size: 0.95rem; font-weight: 700; color: var(--text-primary); }
  .fc-val small { font-size: 0.62rem; font-weight: 500; color: var(--text-tertiary); }
  .fc-label { font-size: 0.62rem; letter-spacing: 0.04em; text-transform: uppercase; color: var(--text-tertiary); }

  .signal-list { list-style: none; display: flex; flex-direction: column; gap: 0.5rem; margin: 0; padding: 0; }
  .signal-list li {
    position: relative;
    padding-left: 0.9rem;
    font-size: 0.8125rem;
    line-height: 1.5;
    color: var(--text-secondary);
  }
  .signal-list li::before {
    content: '·';
    position: absolute;
    left: 0.2rem;
    color: var(--text-faint);
  }
  .signal-list li.warn { color: #e67e22; }
  .signal-list li.warn::before { color: #e67e22; }

  /* Focus balance bars — one row per Push / Pull / Legs. */
  .focus-bars { display: flex; flex-direction: column; gap: 0.4rem; }
  .focus-row { display: flex; align-items: center; gap: 0.6rem; }
  .focus-label {
    flex: none;
    width: 3.2rem;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-secondary);
  }
  .focus-track {
    flex: 1;
    height: 0.5rem;
    border-radius: 0.25rem;
    background: var(--blueprint-tint);
    overflow: hidden;
  }
  .focus-fill { display: block; height: 100%; background: var(--blueprint); }
  .focus-count {
    flex: none;
    min-width: 1.5rem;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    text-align: right;
    color: var(--text-primary);
  }

  .hint { font-size: 0.8125rem; color: var(--text-tertiary); margin: 0 0 0.75rem; }
  /* A hint explaining the block above it needs air; one introducing the block
     below it (straight after a heading) does not. */
  .rate-grid + .hint,
  .ins-rows + .hint,
  .forecast + .hint,
  .focus-bars + .hint { margin-top: 0.85rem; }

  .weekly-list { display: flex; flex-direction: column; }
  .weekly-row {
    display: flex;
    align-items: baseline;
    gap: 0.85rem;
    padding: 0.45rem 0;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 0.875rem;
  }
  .weekly-row:last-child { border-bottom: none; }
  .weekly-week { color: var(--text-secondary); flex: 1; }
  .weekly-count { color: var(--text-faint); font-size: 0.72rem; }
  .weekly-delta {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-tertiary);
    min-width: 3rem;
    text-align: right;
  }
  .weekly-delta.down { color: var(--blueprint, #6ea8fe); }
  .weekly-delta.up { color: #e67e22; }
  .weekly-avg {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--text-primary);
    min-width: 4.25rem;
    text-align: right;
  }

  .history-list { display: flex; flex-direction: column; gap: 0.4rem; }
  .history-month {
    font-family: var(--font-mono);
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-tertiary);
    padding: 0.85rem 0.25rem 0.3rem;
    margin-top: 0.4rem;
    border-bottom: 1px solid var(--border-subtle);
  }
  .history-month:first-child { margin-top: 0; padding-top: 0; }
  .history-item {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.5rem 0.75rem;
    align-items: center;
    padding: 0.6rem 0.75rem;
    border: 1px solid var(--border-subtle);
    border-radius: 0.5rem;
    transition: border-color 0.15s, background 0.15s;
  }
  .history-item:hover { border-color: var(--border-strong); }
  .history-item.open { border-color: var(--blueprint); background: var(--blueprint-tint); }
  .history-head {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    min-width: 0;
    background: transparent;
    border: none;
    cursor: var(--cursor-pointer);
    text-align: left;
    padding: 0;
  }
  .hist-date {
    display: flex;
    align-items: baseline;
    gap: 0.35rem;
    flex: none;
    font-family: var(--font-mono);
  }
  .hist-dom { font-size: 1.05rem; font-weight: 700; color: var(--text-primary); min-width: 1.1rem; }
  .hist-sub { font-size: 0.68rem; color: var(--text-tertiary); }
  .history-item.today .hist-dom { color: var(--blueprint, #6ea8fe); }
  .hist-day {
    flex: none;
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--blueprint, #6ea8fe);
  }
  .hist-bw {
    margin-left: auto;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    color: var(--text-tertiary);
    white-space: nowrap;
  }
  .hist-bw small { font-size: 0.66rem; }
  .hist-chevron {
    flex: none;
    width: 1.1rem;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 1rem;
    color: var(--text-faint);
  }
  .history-item.open .hist-chevron { color: var(--blueprint, #6ea8fe); }
  .history-detail {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
    padding-top: 0.6rem;
    border-top: 1px solid var(--border-subtle);
  }
  .detail-stats {
    font-family: var(--font-mono);
    font-size: 0.7rem;
    letter-spacing: 0.02em;
    color: var(--text-tertiary);
  }
  .detail-ex { display: flex; flex-direction: column; gap: 0.375rem; }
  .detail-ex-name { font-size: 0.875rem; color: var(--text-secondary); }
  .detail-ex-equip {
    font-family: var(--font-mono);
    font-size: 0.68rem;
    color: var(--text-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: 999px;
    padding: 0.05rem 0.4rem;
    margin-left: 0.35rem;
  }
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

  @media (max-width: 640px) {
    .page { gap: 1.15rem; }
    .page-title { font-size: 1.75rem; }
    .week-strip { grid-template-columns: repeat(3, 1fr); gap: 0.4rem; }
    .day-chip { padding: 0.5rem 0.5rem; }

    /* Four forecast cells in a tidy 2×2 rather than auto-fit's 3 + 1. */
    .forecast { grid-template-columns: repeat(2, 1fr); }

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
    .metrics-grid { gap: 0.5rem; }
  }
</style>
