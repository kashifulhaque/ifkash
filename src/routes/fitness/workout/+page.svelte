<svelte:head>
  <title>Workout — Kashif</title>
  <meta name="description" content="A four-day upper/lower program with short cardio bouts — built to hold muscle while cutting." />
</svelte:head>

<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { LogOut, Check, Plus, Trash2, Bike } from 'lucide-svelte';
  import LoadingState from '$lib/components/LoadingState.svelte';
  import { patchLog } from '$lib/workoutReport';
  import { loadTarget, type LoadTarget } from '$lib/workoutTargets';
  import { setToken, loadToken, AuthError, OfflineError } from '$lib/splitterApi';
  import { getApiBase, isLocalDev } from '$lib/apiBase';
  import { adoptSession, startAuthWatch, loadGis, renderGoogleButton } from '$lib/fitnessAuth';
  import {
    ackBwDraft,
    ackDraft,
    bwDraftFor,
    cacheDetail,
    cachedDetail,
    draftDetail,
    draftFor,
    hasPending,
    listBwDrafts,
    listDrafts,
    mergeDraftSessions,
    putBwDraft,
    putDraft,
    readCache,
    writeCache
  } from '$lib/workoutSync';
  import { workoutApi, type ExercisePayload, type CardioPayload } from '$lib/workoutApi';
  import { profileApi } from '$lib/profileApi';
  import {
    DEFAULT_PROFILE,
    computeMetrics,
    nutritionTargets,
    metKcal,
    setsKcal,
    DEFAULT_BODY_KG,
    type SetForKcal,
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
    PLAN_RULES,
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
    /** How to run the slot: grip, setup, per-side count, superset partner. */
    note: string;
    sets: SetRow[];
  };
  let exercises: ExerciseRow[] = [];

  // Cardio bouts logged for the session. `kcalEdited` tracks whether the user
  // has overridden the auto-estimate: while false, kcal follows the live
  // MET-based estimate (kind × minutes × bodyweight); once typed, we stop.
  type CardioRow = { kind: string; minutes: string; kcal: string; kcalEdited: boolean };
  let cardioRows: CardioRow[] = [];

  let saveStatus: 'idle' | 'saving' | 'saved' | 'local' | 'error' = 'idle';
  /** The date and day the form is showing. Saves stamp this, not whatever the
   *  chip says after a switch, so a slow save cannot file Upper A under Lower A. */
  let formDate = sessionDate;
  let formDay: Focus = active;
  let formDirty = false;
  let loadGen = 0;
  let dayTouched = false;
  let booted = false;
  let needsReauth = false;
  let reauthButton: HTMLDivElement;
  let stopAuthWatch: (() => void) | undefined;

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
    if (p) {
      profile = p;
      persistCache();
    }
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

  /** The note and the substitute on one line, e.g. "each side · alt: dumbbells". */
  function exCue(ex: { note: string; alt: string }): string {
    return [ex.note, ex.alt && `alt: ${ex.alt}`].filter(Boolean).join(' · ');
  }

  function templateRows(focus: Focus): ExerciseRow[] {
    return DAY_TEMPLATES[focus].map((ex) => ({
      name: ex.name,
      scheme: ex.scheme,
      kind: ex.kind,
      equipment: ex.equipment ?? '',
      priority: ex.priority ?? false,
      alt: ex.alt ?? '',
      note: ex.note ?? '',
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

  // Every set the active day will save, in the shape the burn estimate reads.
  // Same filter and parsing as `buildPayload`, so the count matches the save.
  $: loggedSets = exercises.flatMap((ex): SetForKcal[] =>
    ex.sets
      .filter((s) => !s.suggested && (str(s.reps).trim() !== '' || str(s.weight).trim() !== ''))
      .map((s) => ({
        exercise: ex.name,
        equipment: ex.equipment,
        kind: ex.kind,
        reps: Math.max(0, parseInt(str(s.reps), 10) || 0),
        weight_g: kgToGrams(str(s.weight))
      }))
  );
  $: totalSets = loggedSets.length;
  // The burn uses the same bodyweight as the metrics, or a default before one
  // is logged, so the lifting number is never blank.
  $: burnKg = metricKg > 0 ? metricKg : DEFAULT_BODY_KG;
  $: liftKcal = setsKcal(loggedSets, burnKg);
  $: cardioBurn = cardioRows.reduce((n, r) => n + cardioKcal(r), 0);
  $: sessionKcal = liftKcal + cardioBurn;

  // The rest of the date: a second session already saved under another day
  // label (a cardio day after a lift, say). Read from the log, which is
  // patched after every save, so it never double-counts the active day.
  $: otherSetsList = log.sets
    .filter((s) => s.date === sessionDate && s.day_label !== active)
    .map((s): SetForKcal => ({
      exercise: s.exercise,
      equipment: s.equipment || exerciseEquipment(s.exercise),
      kind: exerciseKind(s.exercise),
      reps: s.reps,
      weight_g: s.weight_g
    }));
  $: otherSetsToday = otherSetsList.length;
  $: otherCardioToday = log.cardio
    .filter((c) => c.date === sessionDate && c.day_label !== active)
    .reduce((n, c) => n + c.kcal, 0);
  $: daySets = totalSets + otherSetsToday;
  $: dayLiftKcal = liftKcal + setsKcal(otherSetsList, burnKg);
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
      persistCache();
    } catch (e) {
      if (e instanceof AuthError) {
        await promptReauth();
      } else if (!(e instanceof OfflineError)) {
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
    if (id <= 0) return undefined;
    if (detailCache.has(id)) return detailCache.get(id);
    const stored = cachedDetail(id);
    if (stored) {
      detailCache.set(id, stored);
      return stored;
    }
    const d = await guard(() => workoutApi.getSession(id));
    if (d) {
      detailCache.set(id, d);
      persistCache();
    }
    return d;
  }

  /** Bodyweight for `date`: an unsynced local entry, then today's server entry,
   *  then the latest earlier one as a faded hint. */
  function applyBw(date: string) {
    const local = bwDraftFor(date);
    if (local) {
      bw = gramsToKg(local.weight_g);
      bwSuggested = false;
      // Still unsynced. Matching lastSavedBw keeps a later set-edit from
      // writing a second copy; the draft itself is what gets uploaded.
      lastSavedBw = bw;
      return;
    }
    const bwToday = bodyweight.find((b) => b.date === date);
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

  /**
   * Paint `focus` immediately, from the local draft when there is one. The
   * chip and the exercise list update in the same turn, so a slow history
   * fetch cannot leave Lower A selected over Upper A's rows.
   */
  function showDay(focus: Focus) {
    const rows = templateRows(focus);
    let cardio: CardioRow[] = [defaultCardio(focus)];
    const draft = draftFor(sessionDate, focus);
    if (draft) {
      fillRows(rows, draftDetail(draft), true);
      const restored = cardioFromDetail(draftDetail(draft));
      if (restored.length) cardio = restored;
    }
    exercises = rows;
    cardioRows = cardio;
    applyBw(sessionDate);
    formDate = sessionDate;
    formDay = focus;
    formDirty = false;
    if (!draft && log.sets.length) applyTargets();
  }

  /**
   * Fill the active day from the server when this device has no newer draft.
   * A generation counter drops the result if the user has moved on: the
   * history scan is a chain of requests, and the one that finishes last used
   * to overwrite the day they had since selected.
   */
  async function loadDayData() {
    const gen = ++loadGen;
    const focus = active;
    const date = sessionDate;
    if (draftFor(date, focus)) {
      if (gen === loadGen) showDay(focus);
      return;
    }
    const rows = templateRows(focus);
    let cardio: CardioRow[] = [defaultCardio(focus)];

    if (signedIn) {
      const todays = sessions.find((s) => s.date === date && s.day_label === focus && s.id > 0);
      if (todays) {
        const d = await detailFor(todays.id);
        if (gen !== loadGen) return;
        if (d) {
          fillRows(rows, d, true);
          const restored = cardioFromDetail(d);
          if (restored.length) cardio = restored;
        }
      } else {
        await prefillFromHistory(rows, () => gen !== loadGen);
        if (gen !== loadGen) return;
      }
    }

    if (gen !== loadGen) return;
    // They already started logging on this day. Replacing the rows now would
    // drop the sets they just typed.
    if (formDirty && formDate === date && formDay === focus) return;
    exercises = rows;
    cardioRows = cardio;
    applyBw(date);
    formDate = date;
    formDay = focus;
    formDirty = false;
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
  async function prefillFromHistory(rows: ExerciseRow[], cancelled: () => boolean) {
    const pending = new Map(rows.filter((r) => r.name).map((r) => [r.name, r.equipment]));
    if (pending.size === 0) return;
    const matches = (x: { exercise: string; equipment: Equipment }) => {
      const want = pending.get(x.exercise);
      return want !== undefined && (!want || !x.equipment || x.equipment === want);
    };
    const prior = sessions.filter((s) => s.date < sessionDate).slice(0, PREFILL_LOOKBACK);
    for (const s of prior) {
      if (cancelled() || pending.size === 0) break;
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
          note: '',
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
      { name: '', scheme: '', kind: 'weighted', equipment: '', priority: false, alt: '', note: '', sets: [blankSet()] }
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
    dayTouched = true;
    rememberDraft();
    active = focus;
    expanded = {};
    // Swap the list in this turn. The fetch that fills weights in is allowed
    // to finish later, and a stale one is discarded.
    showDay(focus);
    void loadDayData();
    void flush();
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
    // `sessionDate` is already the new value; `formDate` is still the old one,
    // so this files the edits under the day they were typed on.
    rememberDraft();
    showDay(active);
    void loadDayData();
    void flush();
  }

  // ---- auto-save -----------------------------------------------------------

  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  function scheduleSave() {
    if (!signedIn) return;
    formDirty = true;
    rememberDraft();
    saveStatus = 'saving';
    clearTimeout(saveTimer);
    saveTimer = setTimeout(flush, 700);
  }

  /** Copy the form into local storage under the day it belongs to. */
  function rememberDraft() {
    if (!formDirty) return;
    const payload = buildPayload();
    const cardio = buildCardio();
    putDraft(
      payload.length || cardio.length
        ? { date: formDate, day_label: formDay, exercises: payload, cardio, updated: Date.now() }
        : null,
      { date: formDate, day: formDay }
    );
    if (!bwSuggested && str(bw).trim() !== '' && str(bw) !== str(lastSavedBw)) {
      const grams = kgToGrams(str(bw));
      if (grams > 0) {
        putBwDraft({ date: formDate, weight_g: grams, updated: Date.now() });
        lastSavedBw = bw;
      }
    }
    sessions = mergeDraftSessions(sessions.filter((s) => s.id > 0));
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
    rememberDraft();
    const result = await syncPending();
    if (result === 'synced') {
      await refreshLists();
      saveStatus = hasPending() ? 'local' : 'saved';
    } else if (result === 'idle') {
      if (saveStatus === 'saving') saveStatus = hasPending() ? 'local' : 'saved';
    } else if (result === 'local') {
      saveStatus = 'local';
    } else if (result === 'auth') {
      saveStatus = 'local';
      await promptReauth();
    } else {
      saveStatus = 'error';
    }
  }

  /**
   * Upload every draft, oldest edit first. A draft is removed only when the
   * copy that was sent is still the latest one on the device.
   */
  async function syncPending(): Promise<'synced' | 'idle' | 'local' | 'auth' | 'error'> {
    const drafts = [...listDrafts()].sort((a, b) => a.updated - b.updated);
    const weights = [...listBwDrafts()].sort((a, b) => a.updated - b.updated);
    if (!drafts.length && !weights.length) return 'idle';
    if (!loadToken()) return 'auth';

    for (const d of drafts) {
      try {
        const r = await workoutApi.upsertSession({
          day_label: d.day_label,
          date: d.date,
          notes: '',
          exercises: d.exercises,
          cardio: d.cardio
        });
        log = patchLog(
          log,
          { session_id: r.id, date: d.date, day_label: d.day_label },
          d.exercises,
          d.cardio
        );
        const detail = draftDetail(d);
        detail.session.id = r.id;
        detailCache.set(r.id, detail);
        const idx = sessions.findIndex((s) => s.date === d.date && s.day_label === d.day_label);
        const summary = {
          id: r.id,
          day_label: d.day_label,
          date: d.date,
          notes: '',
          created: sessions[idx]?.created ?? new Date(d.updated).toISOString()
        };
        sessions = idx >= 0 ? sessions.map((s, i) => (i === idx ? summary : s)) : [summary, ...sessions];
        ackDraft(d.date, d.day_label, d.updated);
      } catch (e) {
        persistCache();
        if (e instanceof OfflineError) return 'local';
        if (e instanceof AuthError) return 'auth';
        return 'error';
      }
    }

    for (const w of [...listBwDrafts()].sort((a, b) => a.updated - b.updated)) {
      try {
        const r = await workoutApi.addBodyweight(w.date, w.weight_g);
        bodyweight = [
          ...bodyweight.filter((b) => b.date !== w.date),
          { id: r.id, date: w.date, weight_g: w.weight_g }
        ];
        ackBwDraft(w.date, w.updated);
        if (w.date === formDate) lastSavedBw = gramsToKg(w.weight_g);
      } catch (e) {
        persistCache();
        if (e instanceof OfflineError) return 'local';
        if (e instanceof AuthError) return 'auth';
        return 'error';
      }
    }

    sessions = mergeDraftSessions(sessions.filter((s) => s.id > 0));
    persistCache();
    return 'synced';
  }

  function persistCache() {
    let cache = readCache();
    cache = {
      ...cache,
      sessions: sessions.filter((s) => s.id > 0),
      bodyweight,
      log,
      profile
    };
    for (const d of detailCache.values()) cache = cacheDetail(d, cache);
    writeCache(cache);
  }

  function hydrateFromCache() {
    const cache = readCache();
    sessions = mergeDraftSessions(cache.sessions);
    if (cache.bodyweight.length) bodyweight = cache.bodyweight;
    if (cache.log.sets.length || cache.log.cardio.length) log = cache.log;
    for (const d of cache.details) detailCache.set(d.session.id, d);
    if (cache.profile) profile = cache.profile;
  }

  /** This week's lifts, for the day chips and the opening day. */
  $: week = weekChecklist(sessions, LIFT_DAYS, today());

  async function refreshLists() {
    const [s, b] = await Promise.all([
      guard(() => workoutApi.listSessions()),
      guard(() => workoutApi.listBodyweight())
    ]);
    if (s) sessions = mergeDraftSessions(s);
    else sessions = mergeDraftSessions(sessions.filter((row) => row.id > 0));
    if (b) bodyweight = b;
    if (s || b) persistCache();
  }

  // ---- auth ----------------------------------------------------------------

  function onCredential(resp: { credential: string }) {
    setToken(resp.credential);
    signedIn = true;
    needsReauth = false;
    // A silent refresh must not reload the day they are logging.
    void adoptSession().then(() => flush());
    if (!booted) {
      booted = true;
      void bootSignedIn();
    }
  }

  const showSignIn = () => setTimeout(() => renderGoogleButton(gisButton, clientId, onCredential), 0);

  async function promptReauth() {
    if (needsReauth) return;
    needsReauth = true;
    if (hasPending()) saveStatus = 'local';
    await tick();
    renderGoogleButton(reauthButton, clientId, onCredential);
  }

  function signOut() {
    const token = loadToken();
    setToken(null);
    if (token && token !== 'local-dev') {
      void fetch(`${getApiBase()}/api/auth/session`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${token}` }
      }).catch(() => {});
    }
    signedIn = false;
    needsReauth = false;
    booted = false;
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
        // Keep the form. The sets are on this device; signing out here used
        // to hide them behind the sign-in card.
        await promptReauth();
        return undefined;
      }
      if (e instanceof OfflineError) return undefined;
      errorMsg = e instanceof Error ? e.message : 'something went wrong';
      return undefined;
    }
  }

  async function bootSignedIn() {
    hydrateFromCache();
    active = pickDay();
    showDay(active);
    await refreshLists();
    if (!dayTouched) {
      const next = pickDay();
      if (next !== active) {
        active = next;
        showDay(active);
      }
    }
    await loadProfile();
    await loadDayData();
    await loadLog();
    applyTargets();
    void adoptSession();
    void flush();
  }

  onMount(() => {
    const onOnline = () => void flush();
    const onHide = () => {
      if (formDirty) rememberDraft();
    };
    window.addEventListener('online', onOnline);
    window.addEventListener('pagehide', onHide);
    return () => {
      window.removeEventListener('online', onOnline);
      window.removeEventListener('pagehide', onHide);
      stopAuthWatch?.();
    };
  });

  onMount(async () => {
    hydrateFromCache();
    if (sessions.length) active = pickDay();
    showDay(active);
    if (isLocalDev()) {
      // Local dev: the Worker bypasses Google auth (LOCAL_DEV in api/.dev.vars),
      // so skip sign-in entirely and boot straight into the data.
      setToken('local-dev');
      signedIn = true;
      booted = true;
      await bootSignedIn();
      return;
    }
    try {
      await loadGis();
    } catch {
      // Google's script needs a network. A session stored earlier can still
      // open the log, and new sets stay on the device.
    }
    stopAuthWatch = startAuthWatch(clientId, onCredential);
    if (loadToken() || hasPending()) {
      signedIn = true;
      booted = true;
      if (!loadToken()) needsReauth = true;
      await bootSignedIn();
      if (needsReauth) await promptReauth();
    } else {
      renderGoogleButton(gisButton, clientId, onCredential);
    }
  });
</script>

<div class="page" class:split={signedIn}>
  <header class="head">
    <div class="breadcrumb">
      <a href="/fitness">Fitness</a>
      <span class="separator">/</span>
      <span>Workout</span>
    </div>
    <div class="header-top">
      <h1 class="title">Workout</h1>
      <div class="header-actions">
        {#if signedIn}
          <span
            class="save-pill"
            class:on={saveStatus !== 'idle'}
            data-status={saveStatus}
            title={saveStatus === 'local'
              ? 'Saved on this device. Syncs when the connection returns.'
              : undefined}
          >
            {#if saveStatus === 'saving'}
              <LoadingState label="Saving" />
            {:else if saveStatus === 'saved'}
              <Check size={13} /> Saved
            {:else if saveStatus === 'local'}
              <Check size={13} /> Saved here
            {:else if saveStatus === 'error'}
              Save failed
            {/if}
          </span>
          <button class="btn btn-sm btn--ghost signout" on:click={signOut} title="Sign out">
            <LogOut size={15} /> Sign out
          </button>
        {/if}
      </div>
    </div>
    <p class="page-desc">
      Four lifts a week, upper/lower. Pick a day, then {signedIn ? 'tap an exercise and log your sets — it saves as you type, on this device if the connection drops.' : 'sign in to log your sets right here.'}
      {#if signedIn}The trend, report and history are on the <a href="/fitness/workout/insights">insights page</a>.{/if}
    </p>
  </header>

  {#if needsReauth}
    <div class="reauth">
      <p>Sign in again to sync. Sets you log stay on this device until then.</p>
      <div bind:this={reauthButton}></div>
    </div>
  {/if}

  {#if errorMsg}
    <div class="error-banner">{errorMsg}</div>
  {/if}

  <div class="layout">
    <div class="main-col">
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
            <span class="led" class:led-hollow={!week.complete} aria-hidden="true"></span>
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
            <input class="field" type="date" bind:value={sessionDate} on:change={onDateChange} />
          </label>
          <label>
            Bodyweight (kg)
            <input
              class="field num-field"
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
          <span class="cardio-pill"><Bike size={14} aria-hidden="true" /> {DAY_INFO[active].cardio}</span>
        </div>

        {#if DAY_INFO[active].optional && exercises.length}
          <p class="empty-day">
            Zone 2 cardio first, then the pump work: light, high reps, nothing to failure. If a lifting
            day was missed this week, do that here instead.
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
                {#if ex.note || ex.alt}<span class="ex-alt">{exCue(ex)}</span>{/if}
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
                    <span class="ex-title">
                      <span class="ex-name">{ex.name}</span>
                      {#if ex.priority}
                        <span class="key-tag" title="Priority lift — progress this first">key</span>
                      {/if}
                    </span>
                    <span class="ex-target">
                      {ex.scheme}{#if loadTargets[i]}<span class="ex-load" title={loadTargets[i]?.reason}>@ {gramsToKg(Math.round((loadTargets[i]?.kg ?? 0) * 1000))} kg</span>{/if}
                    </span>
                  {:else}
                    <input
                      class="field ex-name-input"
                      placeholder="Exercise name"
                      bind:value={ex.name}
                      on:click|stopPropagation
                      on:input={() => onExerciseName(i)}
                    />
                  {/if}
                  {#if ex.note || ex.alt}<span class="ex-alt">{exCue(ex)}</span>{/if}
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
                      <div class="set-row" class:bare={ex.kind !== 'weighted'}>
                        <span class="set-num">{j + 1}</span>
                        {#if ex.kind === 'weighted'}
                          <input
                            class="field num"
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
                          class="field num"
                          class:suggested={s.suggested}
                          type="number"
                          inputmode="numeric"
                          min="0"
                          placeholder={ex.kind === 'time' ? 'sec' : 'reps'}
                          bind:value={s.reps}
                          on:input={() => touchSet(s)}
                        />
                        <button class="icon-btn" on:click={() => removeSet(i, j)} title="Remove set">
                          <Trash2 size={15} />
                        </button>
                      </div>
                    {/each}
                    <div class="set-actions">
                      <button class="btn btn-sm btn--ghost" on:click={() => addSet(i)}>
                        <Plus size={14} /> set
                      </button>
                      {#if !ex.scheme}
                        <button class="btn btn-sm btn--ghost danger" on:click={() => removeExercise(i)}>
                          <Trash2 size={13} /> remove exercise
                        </button>
                      {/if}
                    </div>
                  </div>
                {/if}
              </div>
            {/each}

            <button class="add-ex" on:click={addExercise}>
              <Plus size={15} /> add exercise
            </button>
          </div>

          <!-- Cardio log — kind + minutes + kcal (auto-estimated, editable). -->
          <div class="cardio-log">
            <div class="cardio-log-head">
              <span class="cardio-log-title"><Bike size={15} aria-hidden="true" /> Cardio</span>
              <span class="cardio-log-hint">kcal auto-estimates from time &amp; bodyweight — tap to override</span>
            </div>
            {#each cardioRows as c, i}
              <div class="cardio-row">
                <select class="field cardio-kind" bind:value={c.kind} on:change={() => onCardioChange(c)}>
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
                    class="field num"
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
                    class="field num"
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
                  <Trash2 size={15} />
                </button>
              </div>
            {/each}
            <button class="add-ex cardio-add" on:click={addCardio}>
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
          <span class="fold-meta">{LIFT_DAYS.length} lifts + conditioning</span>
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
                <span class="ov-cardio"><Bike size={12} aria-hidden="true" /> {DAY_INFO[label].cardio}</span>
              </span>
            </button>
          {/each}
          <ul class="plan-rules">
            {#each PLAN_RULES as rule}<li>{rule}</li>{/each}
          </ul>
        </div>
      </details>
    </div>

    {#if signedIn}
      <aside class="side-col">
        <!-- My details: the profile the body maths runs on, and the numbers it
             gives at the current bodyweight. -->
        <section class="panel">
          <div class="panel-head">
            <h2>My details</h2>
            {#if metrics}<span class="info-meta">at {metricKg.toFixed(1)} kg</span>{/if}
          </div>
          <div class="profile-grid">
            <label>
              Height (cm)
              <input class="field num-field" type="number" min="50" max="300" bind:value={profile.height_cm} on:input={onProfileChange} />
            </label>
            <label>
              Age
              <input class="field num-field" type="number" min="1" max="120" bind:value={profile.age_years} on:input={onProfileChange} />
            </label>
            <label>
              Sex
              <select class="field" bind:value={profile.sex} on:change={onProfileChange}>
                <option value="male">male</option>
                <option value="female">female</option>
              </select>
            </label>
            <label>
              Activity
              <select class="field" bind:value={profile.activity} on:change={onProfileChange}>
                {#each ACTIVITY_OPTIONS as a}
                  <option value={a.value}>{a.label}</option>
                {/each}
              </select>
            </label>
            <label class="wide">
              Goal
              <select class="field" bind:value={profile.goal} on:change={onProfileChange}>
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
            </p>
          {/if}
        </section>

        <!-- Burned today: a rough burn for this date. The active day's rows count
             live; any other session already logged on the same date comes from
             the log. -->
        <section class="panel">
          <div class="panel-head">
            <h2>Burned today</h2>
            <span class="info-meta">rough estimate</span>
          </div>
          <div class="rate-grid">
            <div class="rate-cell">
              <span class="r-val">{dayLiftKcal}<small> kcal</small></span>
              <span class="r-label">lifting</span>
              <span class="r-sub">{daySets} {daySets === 1 ? 'set' : 'sets'} · by load, reps &amp; implement</span>
            </div>
            <div class="rate-cell">
              <span class="r-val">{dayCardioKcal}<small> kcal</small></span>
              <span class="r-label">cardio</span>
              <span class="r-sub">logged or estimated</span>
            </div>
            <div class="rate-cell total">
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
          <span class="go-arrow" aria-hidden="true">&rarr;</span>
        </a>
      </aside>
    {/if}
  </div>
</div>

<style>
  /* ── Page frame ─────────────────────────────────────────── */
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--s-6);
    padding-top: var(--s-7);
    animation: rise var(--dur-slow) var(--ease-out) both;
  }

  /* Signed out: one calm column. Signed in, on wide screens: the logger on
     the left, the body maths and the insights link in a narrow column. */
  .layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--s-6);
    align-items: start;
  }
  .page:not(.split) .main-col { max-width: 760px; }
  .main-col,
  .side-col {
    display: flex;
    flex-direction: column;
    gap: var(--s-5);
    min-width: 0;
  }

  @media (min-width: 1024px) {
    .split .layout {
      grid-template-columns: minmax(0, 1fr) 340px;
      gap: var(--s-7);
    }
  }

  /* ── Header ─────────────────────────────────────────────── */
  .head {
    display: grid;
    gap: var(--s-3);
    padding-bottom: var(--s-5);
    border-bottom: 1px solid var(--line);
  }
  .separator { color: var(--line-strong); }

  .header-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3) var(--s-4);
    flex-wrap: wrap;
  }
  .title {
    margin: 0;
    color: var(--ink);
    font-size: var(--text-2xl);
    font-weight: 500;
    line-height: 1.05;
    letter-spacing: -0.04em;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  /* Fixed width so the sign-out button never shifts as the status changes. */
  .save-pill {
    display: inline-flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    width: 7.25rem;
    height: 20px;
    overflow: hidden;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    opacity: 0;
    transition: opacity var(--dur) var(--ease);
  }
  .save-pill.on { opacity: 1; }
  .save-pill :global(.loader) {
    transform: scale(0.78);
    transform-origin: right center;
  }
  .save-pill[data-status='saved'] { color: var(--moss); }
  .save-pill[data-status='local'] { color: var(--clay); }
  .save-pill[data-status='error'] { color: var(--danger); }

  .reauth {
    display: grid;
    gap: var(--s-3);
    padding: var(--s-4);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink-2);
    font-size: var(--text-sm);
  }
  .reauth p { margin: 0; }

  .signout { color: var(--ink-2); }

  .page-desc {
    max-width: var(--measure);
    margin: 0;
    color: var(--ink-2);
    font-size: var(--text-base);
    line-height: 1.55;
  }

  .error-banner {
    padding: var(--s-3) var(--s-4);
    border: 1px solid var(--danger);
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--danger) 7%, transparent);
    color: var(--danger);
    font-size: var(--text-sm);
  }

  /* ── Week strip ─────────────────────────────────────────── */
  .week-block { display: grid; gap: var(--s-3); }

  /* One hairline-ruled strip: the 1px gap shows the container's line colour
     between cells, so every arrangement (5 across, 4 + 1) stays crisp. */
  .week-strip {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 1px;
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--line);
  }
  .day-chip {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-2);
    min-height: 72px;
    padding: var(--s-3);
    border: 0;
    background: var(--surface);
    text-align: left;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease);
  }
  .day-chip::after {
    content: '';
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    height: 2px;
    background: transparent;
    transition: background-color var(--dur-fast) var(--ease);
  }
  .day-chip:focus-visible { outline-offset: -3px; }
  /* The spare slot sits on the page colour, so the eye reads four slots plus one. */
  .day-chip.optional { background: var(--bg); }
  .day-chip:hover { background: color-mix(in srgb, var(--ink) 4%, var(--surface)); }
  .day-chip.active { background: color-mix(in srgb, var(--fjord) 9%, var(--surface)); }
  .day-chip.active::after { background: var(--fjord); }

  .chip-day {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.2;
    white-space: nowrap;
  }
  .chip-focus {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--ink);
    font-size: 15px;
    font-weight: 500;
    line-height: 1.2;
    letter-spacing: -0.01em;
    white-space: nowrap;
  }
  .day-chip.active .chip-focus { color: var(--fjord); }
  .day-chip.done .chip-focus :global(svg) { color: var(--moss); }

  .week-line {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .week-line.complete { color: var(--moss); }

  /* ── Sign-in ────────────────────────────────────────────── */
  .signin-card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--s-4);
    padding: var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }
  .signin-card p { margin: 0; color: var(--ink-2); }

  /* ── Fields ─────────────────────────────────────────────── */
  .session-bar {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--s-3);
  }
  .session-bar label,
  .profile-grid label {
    display: grid;
    gap: 6px;
    min-width: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.3;
  }

  .field {
    min-height: 44px;
    font-family: var(--font-sans);
    font-size: 16px;
  }
  select.field { padding-right: var(--s-2); }
  .num-field,
  .num {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  /* A prefilled hint from a previous day: dashed and faint until it is edited. */
  .field.suggested {
    border-style: dashed;
    color: var(--ink-3);
  }

  /* Numbers are typed, not spun: hide the steppers so the digits get the room. */
  .num::-webkit-inner-spin-button,
  .num::-webkit-outer-spin-button,
  .num-field::-webkit-inner-spin-button,
  .num-field::-webkit-outer-spin-button {
    margin: 0;
    -webkit-appearance: none;
  }
  .num,
  .num-field {
    -moz-appearance: textfield;
    appearance: textfield;
  }

  /* ── Day card ───────────────────────────────────────────── */
  .day-card {
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }
  .day-card :focus-visible { outline-offset: -2px; }
  .day-card-head {
    display: grid;
    gap: var(--s-1);
    padding: var(--s-5) var(--s-5) var(--s-4);
    border-bottom: 1px solid var(--line);
  }
  .day-card-head h2 {
    margin: 0;
    font-size: var(--text-xl);
    line-height: 1.1;
    letter-spacing: -0.03em;
  }
  .cardio-pill {
    display: inline-flex;
    align-items: flex-start;
    gap: 6px;
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.45;
  }
  .cardio-pill :global(svg) { flex: none; margin-top: 2px; }

  .empty-day {
    margin: 0;
    padding: var(--s-4) var(--s-5);
    border-bottom: 1px solid var(--line);
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  /* Shared row vocabulary */
  .ex-num {
    flex: none;
    width: 1.5rem;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .ex-name {
    color: var(--ink-2);
    font-size: 15px;
    line-height: 1.35;
  }
  .key-lift .ex-name {
    color: var(--ink);
    font-weight: 500;
  }
  .key-tag {
    display: inline-flex;
    flex: none;
    align-items: center;
    height: 18px;
    padding: 0 6px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-pill);
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 400;
    line-height: 1;
    letter-spacing: 0.02em;
  }
  .ex-alt {
    color: var(--ink-3);
    font-size: var(--text-xs);
    line-height: 1.45;
  }

  /* Read-only plan list. Flex, not a fixed grid: the key tag and the scheme
     are both optional, and a fixed column count would push later children
     onto a second row when one is absent. */
  .exercise-list { margin: 0; padding: 0; list-style: none; }
  .exercise-list li {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 2px var(--s-3);
    padding: var(--s-4) var(--s-5);
    border-bottom: 1px solid var(--line);
  }
  .exercise-list li:last-child { border-bottom: 0; }
  .exercise-list .ex-name { flex: 1; min-width: 0; }
  .ex-scheme {
    flex: none;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
  }
  .exercise-list .ex-alt {
    flex-basis: 100%;
    padding-left: calc(1.5rem + var(--s-3));
  }
  .key-legend {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    margin: 0;
    padding: var(--s-4) var(--s-5);
    border-top: 1px solid var(--line);
    color: var(--ink-3);
    font-size: var(--text-xs);
    line-height: 1.5;
  }

  /* ── Interactive log list ───────────────────────────────── */
  .log-list { display: flex; flex-direction: column; }
  .log-ex { border-bottom: 1px solid var(--line); }

  /* Name on the first line; target, implement and the fallback note under
     it. The picker and chevron span both lines on wide screens. */
  .log-ex-head {
    display: grid;
    grid-template-columns: 1.5rem minmax(0, 1fr) auto 1.25rem;
    grid-template-areas:
      'num title equip chev'
      '. target equip chev'
      '. alt alt .';
    align-items: center;
    gap: 2px var(--s-3);
    width: 100%;
    min-height: 64px;
    padding: var(--s-3) var(--s-5);
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease);
  }
  .log-ex-head:hover { background: var(--hover-wash); }
  .log-ex-head .ex-num { grid-area: num; }
  .log-ex-head .ex-alt { grid-area: alt; margin-top: 2px; }
  .ex-title {
    grid-area: title;
    min-width: 0;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px var(--s-2);
  }
  .log-ex-head .ex-name {
    color: var(--ink);
    font-size: var(--text-base);
  }
  .ex-target {
    grid-area: target;
    min-width: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .ex-load { margin-left: 0.5em; color: var(--fjord); }
  .ex-name-input { grid-area: title; font-weight: 500; }

  /* Implement picker: quiet until set, since loads only compare within one. */
  .ex-equip {
    grid-area: equip;
    min-height: 40px;
    max-width: 8.5rem;
    padding: 0 var(--s-2);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    cursor: pointer;
  }
  .ex-equip.unset {
    border-style: dashed;
    color: var(--ink-3);
  }
  .chevron {
    grid-area: chev;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-md);
    line-height: 1;
    text-align: center;
  }
  .log-ex.open .chevron { color: var(--ink); }

  /* Sets: indented to the name column so the numbers hang under the lift. */
  .sets {
    display: grid;
    gap: var(--s-2);
    padding: var(--s-1) var(--s-5) var(--s-5) calc(var(--s-5) + 1.5rem + var(--s-3));
  }
  .load-note {
    max-width: var(--measure);
    margin: 0 0 var(--s-1);
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.5;
  }
  .set-row {
    display: grid;
    grid-template-columns: 1.5rem minmax(0, 1fr) 1rem minmax(0, 1fr) 44px;
    align-items: center;
    gap: var(--s-2);
    max-width: 26rem;
  }
  .set-row.bare { grid-template-columns: 1.5rem minmax(0, 1fr) 44px; max-width: 16rem; }
  .set-num {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .num {
    width: 100%;
    min-width: 0;
    padding: 0 var(--s-2);
    font-size: var(--text-md);
    text-align: center;
  }
  .times {
    color: var(--ink-3);
    text-align: center;
  }

  .set-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-top: var(--s-1);
    margin-left: calc(-1 * var(--s-3));
  }
  .set-actions .btn { min-height: 40px; }
  .set-actions .btn.danger:hover:not(:disabled) { color: var(--danger); }

  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-3);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background-color var(--dur-fast) var(--ease);
  }
  .icon-btn:hover {
    background: var(--hover-wash);
    color: var(--danger);
  }

  .add-ex {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    min-height: 48px;
    margin: var(--s-4) var(--s-5);
    border: 1px dashed var(--line-strong);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-2);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease),
      background-color var(--dur-fast) var(--ease);
  }
  .add-ex:hover {
    border-color: var(--ink-3);
    background: var(--hover-wash);
    color: var(--ink);
  }

  /* ── Cardio log ─────────────────────────────────────────── */
  .cardio-log {
    display: grid;
    gap: var(--s-3);
    padding: var(--s-5);
    border-top: 1px solid var(--line);
    background: var(--bg);
  }
  .cardio-log-head {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: 4px var(--s-3);
  }
  .cardio-log-title {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--ink);
    font-size: 15px;
    font-weight: 500;
  }
  .cardio-log-hint {
    color: var(--ink-3);
    font-size: var(--text-xs);
  }
  .cardio-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 7.5rem 7.5rem 44px;
    align-items: center;
    gap: var(--s-2);
  }
  .cardio-kind { min-width: 0; }
  .cardio-field { position: relative; min-width: 0; }
  .cardio-field .num {
    padding-right: 2.9rem;
    text-align: right;
  }
  .cardio-field .unit {
    position: absolute;
    top: 50%;
    right: var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    pointer-events: none;
    transform: translateY(-50%);
  }
  .add-ex.cardio-add { margin: 0; }

  /* ── Week overview fold ─────────────────────────────────── */
  .fold {
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }
  .fold :focus-visible { outline-offset: -2px; }
  .fold summary {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    min-height: 56px;
    padding: 0 var(--s-5);
    color: var(--ink);
    list-style: none;
    cursor: pointer;
  }
  .fold summary::-webkit-details-marker { display: none; }
  .fold summary:hover .fold-title { color: var(--fjord); }
  .fold-title {
    flex: none;
    font-size: 15px;
    font-weight: 500;
    transition: color var(--dur-fast) var(--ease);
  }
  .fold-meta {
    margin-left: auto;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .fold summary::after {
    content: '+';
    width: 1.25rem;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-md);
    text-align: center;
  }
  .fold[open] summary::after { content: '−'; }
  .fold[open] summary { border-bottom: 1px solid var(--line); }

  .overview { display: flex; flex-direction: column; }
  .ov-day {
    position: relative;
    display: grid;
    grid-template-columns: 5.5rem minmax(0, 1fr);
    gap: var(--s-4);
    width: 100%;
    padding: var(--s-4) var(--s-5);
    border: 0;
    border-bottom: 1px solid var(--line);
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease);
  }
  .ov-day:last-child { border-bottom: 0; }
  .ov-day:hover { background: var(--hover-wash); }
  .ov-day::before {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    width: 2px;
    background: transparent;
  }
  .ov-day.active { background: var(--fjord-wash); }
  .ov-day.active::before { background: var(--fjord); }
  .ov-when {
    padding-top: 3px;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .ov-body {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }
  .ov-title {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--ink);
    font-size: 15px;
    font-weight: 500;
  }
  .ov-day.active .ov-title { color: var(--fjord); }
  .ov-day.done .ov-title :global(svg) { color: var(--moss); }
  .ov-day.optional .ov-title { color: var(--ink-2); }
  .plan-rules {
    margin: 8px 0 0;
    padding-left: 18px;
    font-size: 13px;
    line-height: 1.5;
    color: var(--ink-2);
  }
  .ov-lifts {
    display: flex;
    flex-wrap: wrap;
    gap: 2px var(--s-3);
  }
  .ov-lift {
    color: var(--ink-2);
    font-size: var(--text-sm);
    line-height: 1.45;
  }
  .ov-lift.key { color: var(--ink); }
  .ov-lift em {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 11px;
    font-style: normal;
  }
  .ov-cardio {
    display: inline-flex;
    align-items: flex-start;
    gap: 6px;
    color: var(--ink-3);
    font-size: var(--text-xs);
    line-height: 1.45;
  }
  .ov-cardio :global(svg) { flex: none; margin-top: 2px; }

  /* ── Side panels (details, burned today) ────────────────── */
  .panel {
    padding: var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }
  .panel-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  .panel-head h2 {
    margin: 0;
    font-size: var(--text-md);
    letter-spacing: -0.01em;
  }
  .info-meta {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .profile-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--s-3);
    margin-bottom: var(--s-5);
  }
  .profile-grid label.wide { grid-column: 1 / -1; }

  /* Stats: label above a large mono value, ruled with hairlines. */
  .rate-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    column-gap: var(--s-5);
  }
  .rate-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    padding: var(--s-3) 0;
    border-top: 1px solid var(--line);
  }
  .rate-cell.total { grid-column: 1 / -1; }
  .r-label {
    order: -1;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .r-val {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xl);
    font-weight: 400;
    font-variant-numeric: tabular-nums;
    line-height: 1.15;
    letter-spacing: -0.03em;
    white-space: nowrap;
  }
  .r-val small {
    color: var(--ink-3);
    font-size: var(--text-xs);
    letter-spacing: 0;
  }
  .r-sub {
    color: var(--ink-3);
    font-size: var(--text-xs);
    line-height: 1.4;
  }
  .hint {
    margin: var(--s-4) 0 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .link-card {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    grid-template-areas: 'title arrow' 'meta arrow';
    align-items: center;
    gap: 2px var(--s-3);
    min-height: 64px;
    padding: var(--s-4) var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
    color: var(--ink);
    text-decoration: none;
    transition: border-color var(--dur-fast) var(--ease);
  }
  .link-card:hover { border-color: var(--line-strong); color: var(--ink); }
  .link-title {
    grid-area: title;
    font-size: 15px;
    font-weight: 500;
    transition: color var(--dur-fast) var(--ease);
  }
  .link-card:hover .link-title { color: var(--fjord); }
  .link-meta {
    grid-area: meta;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .go-arrow {
    grid-area: arrow;
    color: var(--ink-3);
    transition: transform var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
  }
  .link-card:hover .go-arrow {
    color: var(--fjord);
    transform: translateX(3px);
  }

  /* Mid widths: the side column is full width, so spread the stats out. */
  @media (min-width: 641px) and (max-width: 1023px) {
    .rate-grid { grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr)); }
    .rate-cell.total { grid-column: auto; }
  }

  /* ── Phone (the gym) ────────────────────────────────────── */
  @media (max-width: 640px) {
    .page { gap: var(--s-5); padding-top: var(--s-6); }
    .layout,
    .main-col,
    .side-col { gap: var(--s-4); }
    .signout { min-height: 40px; }
    /* Keep the title and sign-out on one line: the status floats in the gap
       to the button's left instead of claiming its own width. */
    .header-actions { position: relative; }
    .save-pill {
      position: absolute;
      top: 50%;
      right: 100%;
      width: 6.75rem;
      margin-right: var(--s-1);
      transform: translateY(-50%);
    }

    /* Four lifts across, the spare slot as a full-width row beneath. */
    .week-strip { grid-template-columns: repeat(4, minmax(0, 1fr)); }
    .day-chip { min-height: 64px; padding: var(--s-3) var(--s-2) var(--s-3) var(--s-3); }
    .day-chip.optional {
      grid-column: 1 / -1;
      flex-direction: row;
      align-items: center;
      min-height: 48px;
    }

    .day-card-head { padding: var(--s-4); }
    .empty-day,
    .exercise-list li,
    .key-legend { padding-right: var(--s-4); padding-left: var(--s-4); }

    /* The implement picker drops under the name so a long lift name keeps
       the full width of the row. */
    /* The name takes the full first line; the implement picker drops to
       the target's line so a long lift name never gets squeezed. */
    .log-ex-head {
      grid-template-columns: 1.25rem minmax(0, 1fr) auto 1.25rem;
      grid-template-areas:
        'num title title chev'
        '. target equip chev'
        '. alt alt .';
      column-gap: var(--s-2);
      padding: var(--s-3) var(--s-4);
    }
    .log-ex-head .ex-num { width: 1.25rem; }
    .ex-equip {
      max-width: 8rem;
      font-size: 16px;
    }

    .sets { padding: 0 var(--s-4) var(--s-4); }
    .set-row,
    .set-row.bare { max-width: none; }
    .set-actions { margin-left: 0; }
    .add-ex { margin: var(--s-4); }

    .cardio-log { padding: var(--s-4); }
    .cardio-row { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 44px; }
    .cardio-kind { grid-column: 1 / -1; }

    .fold summary { padding: 0 var(--s-4); }
    .ov-day {
      grid-template-columns: 1fr;
      gap: var(--s-1);
      padding: var(--s-3) var(--s-4);
    }

    .panel { padding: var(--s-4); }
    .link-card { padding: var(--s-4); }
  }

  @media (max-width: 380px) {
    .chip-focus { font-size: 14px; }
    .day-chip { padding-left: var(--s-2); }
  }
</style>
