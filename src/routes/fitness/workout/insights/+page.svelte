<svelte:head>
  <title>Workout insights — Kashif</title>
  <meta name="description" content="Bodyweight trend, training report, the fitted rates and projections, and the full session history." />
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<script lang="ts">
  /* ─────────────────────────────────────────────────────────
   * WORKOUT INSIGHTS — everything the logging page leaves out
   *
   *   · progress card: trend weight, rate, pace, and the chart
   *   · the day / week / month training report
   *   · the numbers: bodyweight, fitted rates, energy balance,
   *     projections, signals, weekly averages, cadence
   *   · the full session history (open a session, delete one)
   *
   * The logging page at /fitness/workout keeps only what a session
   * needs. Both pages share the same data; this one is read-only
   * except for deleting a session from the history.
   * ───────────────────────────────────────────────────────── */

  import { onMount } from 'svelte';
  import { env } from '$env/dynamic/public';
  import { LogOut, Trash2 } from 'lucide-svelte';
  import LoadingState from '$lib/components/LoadingState.svelte';
  import WeightChart from '$lib/components/WeightChart.svelte';
  import WorkoutReport from '$lib/components/WorkoutReport.svelte';
  import { dropSession } from '$lib/workoutReport';
  import { setToken, loadToken, AuthError } from '$lib/splitterApi';
  import { isLocalDev } from '$lib/apiBase';
  import { scheduleTokenRefresh, loadGis, renderGoogleButton } from '$lib/fitnessAuth';
  import { workoutApi } from '$lib/workoutApi';
  import { profileApi } from '$lib/profileApi';
  import { DEFAULT_PROFILE, type Profile } from '$lib/fitnessMetrics';
  import {
    weightInsights,
    trainingCadence,
    weightForBmi,
    type RateBand,
    type Pace
  } from '$lib/fitnessInsights';
  import {
    gramsToKg,
    weeklyAverages,
    exerciseKind,
    groupSets,
    type SessionSummary,
    type SessionDetail,
    type BodyweightEntry,
    type WeeklyAverage,
    type TrainingLog
  } from '$lib/workout';

  const clientId = env.PUBLIC_GOOGLE_CLIENT_ID ?? '';

  // ---- state ---------------------------------------------------------------

  let signedIn = false;
  let gisButton: HTMLDivElement;
  let errorMsg = '';

  function today(): string {
    const d = new Date();
    const off = d.getTimezoneOffset();
    return new Date(d.getTime() - off * 60000).toISOString().slice(0, 10);
  }

  let profile: Profile = { ...DEFAULT_PROFILE };
  let sessions: SessionSummary[] = [];
  let bodyweight: BodyweightEntry[] = [];
  let log: TrainingLog = { sets: [], cardio: [] };
  let logLoading = false;

  async function loadProfile() {
    const p = await guard(() => profileApi.get(loadToken() ?? ''));
    if (p) profile = p;
  }

  async function refreshLists() {
    const [s, b] = await Promise.all([
      guard(() => workoutApi.listSessions()),
      guard(() => workoutApi.listBodyweight())
    ]);
    if (s) sessions = s;
    if (b) bodyweight = b;
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

  // The one-word read on the 4-week rate against the goal's rate. Bands are
  // deliberately coarse; the numbers behind them live in the details fold.
  const PACE_LABEL: Record<Pace, string> = {
    ahead: 'ahead of goal',
    'on pace': 'on pace',
    behind: 'behind goal',
    flat: 'flat',
    gaining: 'gaining'
  };
  const PACE_NOTE: Record<Pace, string> = {
    ahead: 'Faster than the goal asks. Fine for a short block; if the key lifts start dropping, eat a little more.',
    'on pace': 'Losing at the rate the goal asks for. Keep doing what this month did.',
    behind: 'Losing, but slower than the goal asks. Steps and the food rules are the levers, not more gym time.',
    flat: 'No movement over the last month. A 3-week stall on a cut means intake has drifted up to maintenance.',
    gaining: 'Trending up over the last month. Intake is above maintenance, whatever the training looks like.'
  };
  const PACE_WARN: Pace[] = ['behind', 'flat', 'gaining'];

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
      log = dropSession(log, id);
      await refreshLists();
    }
  }

  function setLabel(reps: number, weight_g: number, name: string): string {
    if (exerciseKind(name) === 'time') return `${reps}s`;
    return weight_g > 0 ? `${gramsToKg(weight_g)}kg × ${reps}` : `BW × ${reps}`;
  }

  // ---- auth (mirrors the workout page's Google Identity flow) --------------

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
    openId = null;
    openDetail = null;
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
        showSignIn();
      } else {
        errorMsg = e instanceof Error ? e.message : 'something went wrong';
      }
      return undefined;
    }
  }

  async function bootSignedIn() {
    await refreshLists();
    await loadProfile();
    await loadLog();
  }

  onMount(async () => {
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
      <a href="/fitness/workout">Workout</a>
      <span class="separator">/</span>
      <span>Insights</span>
    </div>
    <div class="header-top">
      <h1 class="page-title">Insights 📈</h1>
      <div class="header-actions">
        {#if signedIn}
          <button class="ghost-btn" on:click={signOut} title="Sign out">
            <LogOut size={15} /> Sign out
          </button>
        {/if}
      </div>
    </div>
    <p class="page-desc">
      Is it working? The trend weight, the training report, every fitted number, and the full
      history. Log sessions on the <a href="/fitness/workout">workout page</a>.
    </p>
  </header>

  {#if errorMsg}
    <div class="error-banner">{errorMsg}</div>
  {/if}

  {#if !signedIn}
    <div class="signin-card">
      <p>Sign in with Google to see your trend, report and history.</p>
      <div bind:this={gisButton}></div>
    </div>
  {:else}
    <!-- Progress: the three numbers that answer "is this working?" and the
         chart. Everything else is behind the details fold on purpose. -->
    <section class="progress-card">
      {#if insights.points.length}
        <div class="trend-stats">
          <div class="trend-stat">
            <span class="ts-val">{insights.trendKg}<small> kg</small></span>
            <span class="ts-label">trend weight</span>
            <span class="ts-sub">scale {insights.latestKg} kg · {fmtDate(insights.latestDate)}</span>
          </div>
          <div
            class="trend-stat"
            class:down={(insights.primary.kgPerWeek ?? 0) < 0}
            class:up={(insights.primary.kgPerWeek ?? 0) > 0}
          >
            <span class="ts-val">{fmtSigned(insights.primary.kgPerWeek, 2)}<small> kg/wk</small></span>
            <span class="ts-label">{insights.primary.label}</span>
            <span class="ts-sub">goal {fmtSigned(insights.targetKgPerWeek, 2)} kg/wk</span>
          </div>
          <div class="trend-stat" class:warn={insights.pace !== null && PACE_WARN.includes(insights.pace)}>
            <span class="ts-val pace">{insights.pace ? PACE_LABEL[insights.pace] : '—'}</span>
            <span class="ts-label">pace</span>
            <span class="ts-sub">{cadence.perWeek} sessions/wk · last 4 weeks</span>
          </div>
        </div>

        {#if insights.pace}
          <p class="assess" class:warn={PACE_WARN.includes(insights.pace)}>{PACE_NOTE[insights.pace]}</p>
        {/if}

        <WeightChart
          points={insights.points}
          ema={insights.ema}
          targetKg={normalBmiKg}
          targetLabel={`BMI 25 · ${normalBmiKg} kg`}
        />
      {:else}
        <p class="hint">No bodyweight entries yet — add one up top and the trend appears here.</p>
      {/if}
    </section>

    <!-- Training report: a day, week, or month read against the plan and
         compared with another period. -->
    <WorkoutReport {log} {bodyweight} today={today()} loading={logLoading} />

    <!-- Everything the progress card left out: the fitted rates, projections,
         signals, weekly averages, cadence. Open by default — this page exists
         for them. -->
    <section class="numbers">
      <div class="numbers-head">
        <h2>All the numbers</h2>
        {#if insights.trendKg !== null}
          <span class="numbers-meta">{fmtSigned(insights.totalChange)} kg since start</span>
        {/if}
      </div>
      <div class="numbers-body">
        {#if insights.points.length}
          <h4 class="ins-head first">Bodyweight</h4>
          <div class="rate-grid">
            <div class="rate-cell" class:down={(weekDelta ?? 0) < 0} class:up={(weekDelta ?? 0) > 0}>
              <span class="r-val">{fmtDelta(weekDelta)}<small> kg</small></span>
              <span class="r-label">vs last week</span>
              <span class="r-sub">weekly averages</span>
            </div>
            <div
              class="rate-cell"
              class:down={(insights.totalChange ?? 0) < 0}
              class:up={(insights.totalChange ?? 0) > 0}
            >
              <span class="r-val">{fmtSigned(insights.totalChange)}<small> kg</small></span>
              <span class="r-label">since start</span>
              <span class="r-sub">
                {fmtSigned(insights.totalChangePct)}% · {insights.consistency.spanDays} days · from {insights.startKg} kg
              </span>
            </div>
          </div>

          {#if insights.primary.band}
            <p class="assess quiet" class:warn={BAND_WARN.includes(insights.primary.band)}>
              <span class="assess-chip">
                {BAND_LABEL[insights.primary.band]}
                {#if insights.primary.pctPerWeek !== null}
                  · {fmtSigned(insights.primary.pctPerWeek, 2)}%/wk
                {/if}
              </span>
              {BAND_NOTE[insights.primary.band]}
            </p>
          {/if}

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
        {/if}

        {#if cadence.total}
          <h4 class="ins-head">Training cadence</h4>
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

          <h4 class="ins-head">Day balance <small>last 4 weeks</small></h4>
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
            Each day runs once a week, so four weeks of the plan is four of each. Sessions from
            the earlier push/pull/legs plan don't appear here.
          </p>
        {/if}
        {#if !insights.points.length && !cadence.total}
          <p class="hint">Nothing to read yet — log a bodyweight and a session on the workout page.</p>
        {/if}
      </div>
    </section>

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

  /* ── Progress card ──────────────────────────────────────── */
  .progress-card {
    border: 1px solid var(--border);
    border-radius: 0.625rem;
    padding: 1rem;
  }
  .progress-card .trend-stats { grid-template-columns: repeat(3, 1fr); }
  .ts-val.pace { font-size: 0.95rem; text-transform: uppercase; letter-spacing: 0.04em; }
  .trend-stat.warn .ts-val { color: #e67e22; }
  .progress-card .hint { margin: 0; }

  /* ── The numbers ────────────────────────────────────────── */
  .numbers { border: 1px solid var(--border); border-radius: 0.625rem; }
  .numbers-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.875rem 1rem;
    border-bottom: 1px solid var(--border);
  }
  .numbers-head h2 { font-size: 1rem; font-weight: 600; color: var(--text-primary); margin: 0; }
  .numbers-meta {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-tertiary);
    white-space: nowrap;
  }
  .numbers-body { padding: 0 1rem 1.25rem; }
  .numbers-body .ins-head.first { margin-top: 1.25rem; }

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
  /* Inside the numbers fold the verdict is supporting detail, not a banner. */
  .assess.quiet { margin-top: 0.85rem; border-color: var(--border-subtle); background: transparent; }
  .assess.quiet.warn { border-color: #e67e22; }
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
  .ins-head.first { margin-top: 0; }

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

  /* Day balance bars — one row per lifting day. */
  .focus-bars { display: flex; flex-direction: column; gap: 0.4rem; }
  .focus-row { display: flex; align-items: center; gap: 0.6rem; }
  .focus-label {
    flex: none;
    width: 4.2rem;
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
    /* Two numbers side by side, the pace verdict on its own full-width row. */
    .progress-card .trend-stats { grid-template-columns: repeat(2, 1fr); }
    .progress-card .trend-stat:last-child { grid-column: 1 / -1; }

    /* Four forecast cells in a tidy 2×2 rather than auto-fit's 3 + 1. */
    .forecast { grid-template-columns: repeat(2, 1fr); }
  }
</style>
