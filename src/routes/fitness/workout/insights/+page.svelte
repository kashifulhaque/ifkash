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
  import { LogOut, Trash2, Bike } from 'lucide-svelte';
  import LoadingState from '$lib/components/LoadingState.svelte';
  import WeightChart from '$lib/components/WeightChart.svelte';
  import WorkoutReport from '$lib/components/WorkoutReport.svelte';
  import { dropSession } from '$lib/workoutReport';
  import { setToken, loadToken, AuthError, OfflineError } from '$lib/splitterApi';
  import { isLocalDev } from '$lib/apiBase';
  import { adoptSession, startAuthWatch, loadGis, renderGoogleButton } from '$lib/fitnessAuth';
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

  let booted = false;
  let stopAuthWatch: (() => void) | undefined;

  function onCredential(resp: { credential: string }) {
    setToken(resp.credential);
    signedIn = true;
    void adoptSession();
    if (!booted) {
      booted = true;
      void bootSignedIn();
    }
  }

  const showSignIn = () => setTimeout(() => renderGoogleButton(gisButton, clientId, onCredential), 0);

  function signOut() {
    setToken(null);
    signedIn = false;
    booted = false;
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
        showSignIn();
      } else if (e instanceof OfflineError) {
        errorMsg = "Couldn't reach the server. This page updates when the connection returns.";
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
    void adoptSession();
  }

  onMount(() => () => stopAuthWatch?.());

  onMount(async () => {
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
      /* Google's script needs a network. A stored session can still load. */
    }
    stopAuthWatch = startAuthWatch(clientId, onCredential);
    if (loadToken()) {
      signedIn = true;
      booted = true;
      await bootSignedIn();
    } else {
      renderGoogleButton(gisButton, clientId, onCredential);
    }
  });
</script>


<div class="page">
  <header class="head">
    <div class="breadcrumb">
      <a href="/fitness">Fitness</a>
      <span class="separator">/</span>
      <a href="/fitness/workout">Workout</a>
      <span class="separator">/</span>
      <span>Insights</span>
    </div>
    <div class="header-top">
      <h1 class="title">Insights</h1>
      <div class="header-actions">
        {#if signedIn}
          <button class="btn btn-sm btn--ghost signout" on:click={signOut} title="Sign out">
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
    <section class="progress-card" class:has-data={insights.points.length > 0}>
      {#if insights.points.length}
        <div class="progress-side">
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
        </div>

        <div class="progress-chart">
          <WeightChart
            points={insights.points}
            ema={insights.ema}
            targetKg={normalBmiKg}
            targetLabel={`BMI 25 · ${normalBmiKg} kg`}
          />
        </div>
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
          <div class="ins-block">
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
          </div>

          <div class="ins-block">
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
          </div>

          <div class="ins-block">
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
          </div>

          <div class="ins-block">
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
          </div>

          <div class="ins-block">
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
          </div>

          <div class="ins-block">
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
          </div>
        {/if}

        {#if cadence.total}
          <div class="ins-block">
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
          </div>

          <div class="ins-block">
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
          </div>
        {/if}
        {#if !insights.points.length && !cadence.total}
          <p class="hint empty">Nothing to read yet — log a bodyweight and a session on the workout page.</p>
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
          <p class="hint empty">No sessions logged yet.</p>
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
                            <span class="detail-ex-name"><Bike size={14} aria-hidden="true" /> Cardio</span>
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
  /* ── Page frame ─────────────────────────────────────────── */
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--s-6);
    padding-top: var(--s-7);
    animation: rise var(--dur-slow) var(--ease-out) both;
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

  .signin-card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--s-4);
    max-width: 760px;
    padding: var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }
  .signin-card p { margin: 0; color: var(--ink-2); }

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

  /* ── Progress card ──────────────────────────────────────── */
  .progress-card {
    display: grid;
    gap: var(--s-5);
    padding: var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }
  .progress-side,
  .progress-chart { min-width: 0; }
  .progress-side { display: grid; gap: var(--s-4); align-content: start; }

  /* Wide screens: the three numbers stacked beside the chart. */
  @media (min-width: 960px) {
    .progress-card.has-data {
      grid-template-columns: 17rem minmax(0, 1fr);
      gap: var(--s-7);
      padding: var(--s-6);
    }
    .progress-card .trend-stats { grid-template-columns: minmax(0, 1fr); }
  }

  .trend-stats {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    column-gap: var(--s-5);
  }
  .trend-stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    padding: var(--s-3) 0;
    border-top: 1px solid var(--line);
  }
  .ts-label {
    order: -1;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .ts-val {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xl);
    font-weight: 400;
    font-variant-numeric: tabular-nums;
    line-height: 1.15;
    letter-spacing: -0.03em;
    white-space: nowrap;
  }
  .ts-val small {
    color: var(--ink-3);
    font-size: var(--text-xs);
    letter-spacing: 0;
  }
  .ts-val.pace {
    font-family: var(--font-sans);
    font-size: var(--text-lg);
    font-weight: 500;
    line-height: 1.3;
    letter-spacing: -0.015em;
  }
  .ts-val.pace::first-letter { text-transform: uppercase; }
  .ts-sub {
    color: var(--ink-3);
    font-size: var(--text-xs);
    line-height: 1.4;
  }
  .trend-stat.down .ts-val { color: var(--fjord); }
  .trend-stat.up .ts-val,
  .trend-stat.warn .ts-val { color: var(--clay); }

  /* The one-line verdict on the current rate: a quiet ruled note. */
  .assess {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px var(--s-2);
    margin: 0;
    padding: 2px 0 2px var(--s-3);
    border-left: 2px solid var(--fjord);
    color: var(--ink-2);
    font-size: var(--text-sm);
    line-height: 1.55;
  }
  .assess.warn { border-left-color: var(--clay); }
  .assess.quiet { margin-top: var(--s-4); border-left-color: var(--line-strong); }
  .assess.quiet.warn { border-left-color: var(--clay); }
  .assess-chip {
    flex: none;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  /* ── All the numbers ────────────────────────────────────── */
  .numbers {
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }
  .numbers-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-4) var(--s-5);
    border-bottom: 1px solid var(--line);
  }
  .numbers-head h2 {
    margin: 0;
    font-size: var(--text-md);
    letter-spacing: -0.01em;
  }
  .numbers-meta {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .numbers-body { padding: 0 var(--s-5) var(--s-2); }

  /* Each block is self-contained, so wide screens flow them into two ruled
     columns rather than one long scroll. */
  .ins-block {
    padding: var(--s-5) 0;
    border-top: 1px solid var(--line);
    break-inside: avoid;
  }
  .ins-block:first-child { border-top: 0; }
  @media (min-width: 1100px) {
    .numbers-body {
      columns: 2;
      column-gap: var(--s-8);
      column-rule: 1px solid var(--line);
    }
    .ins-block { border-top: 0; border-bottom: 1px solid var(--line); }
  }

  .ins-head {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px var(--s-2);
    margin: 0 0 var(--s-3);
    color: var(--ink);
    font-size: 15px;
    font-weight: 500;
    letter-spacing: -0.005em;
  }
  .ins-head small {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
    letter-spacing: 0;
  }

  .rate-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr));
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
  .rate-cell.down .r-val { color: var(--fjord); }
  .rate-cell.up .r-val { color: var(--clay); }
  .rate-cell.muted .r-val { color: var(--ink-3); }

  .ins-rows { display: flex; flex-direction: column; border-top: 1px solid var(--line); }
  .ins-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-2) 0;
    border-bottom: 1px solid var(--line);
    color: var(--ink-2);
    font-size: var(--text-sm);
    line-height: 1.5;
  }
  .ins-row em {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-style: normal;
  }
  .ins-row strong {
    flex: none;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 400;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .ins-row strong.warn { color: var(--clay); }

  .forecast {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(5rem, 1fr));
    gap: 1px;
    margin-top: var(--s-4);
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--line);
  }
  .fc {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--s-3);
    background: var(--surface);
  }
  .fc-val {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-md);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .fc-val small { color: var(--ink-3); font-size: var(--text-xs); }
  .fc-label {
    order: -1;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .signal-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .signal-list li {
    position: relative;
    padding-left: var(--s-4);
    color: var(--ink-2);
    font-size: var(--text-sm);
    line-height: 1.55;
  }
  .signal-list li::before {
    content: '';
    position: absolute;
    top: 0.62em;
    left: 2px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--line-strong);
  }
  .signal-list li.warn { color: var(--ink); }
  .signal-list li.warn::before { background: var(--clay); }

  /* Day balance bars — one row per lifting day. */
  .focus-bars { display: flex; flex-direction: column; gap: var(--s-2); }
  .focus-row { display: flex; align-items: center; gap: var(--s-3); }
  .focus-label {
    flex: none;
    width: 4.5rem;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .focus-track {
    flex: 1;
    height: 6px;
    overflow: hidden;
    border-radius: var(--radius-xs);
    background: var(--surface-2);
  }
  .focus-fill {
    display: block;
    height: 100%;
    background: var(--fjord);
  }
  .focus-count {
    flex: none;
    min-width: 1.5rem;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .hint {
    margin: 0 0 var(--s-3);
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.55;
  }
  .hint.empty { margin: 0; padding: var(--s-5) 0; }
  /* A hint explaining the block above it needs air; one introducing the block
     below it (straight after a heading) does not. */
  .rate-grid + .hint,
  .ins-rows + .hint,
  .forecast + .hint,
  .focus-bars + .hint { margin-top: var(--s-3); }
  .ins-block > .hint:last-child { margin-bottom: 0; }
  .progress-card > .hint { margin: 0; }

  .weekly-list { display: flex; flex-direction: column; border-top: 1px solid var(--line); }
  .weekly-row {
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
    padding: var(--s-2) 0;
    border-bottom: 1px solid var(--line);
    font-size: var(--text-sm);
  }
  .weekly-row:last-child { border-bottom: 0; }
  .weekly-week { flex: 1; min-width: 0; color: var(--ink-2); }
  .weekly-count {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    white-space: nowrap;
  }
  .weekly-delta {
    min-width: 3.25rem;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    text-align: right;
    white-space: nowrap;
  }
  .weekly-delta.down { color: var(--fjord); }
  .weekly-delta.up { color: var(--clay); }
  .weekly-avg {
    min-width: 4.5rem;
    color: var(--ink);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    text-align: right;
    white-space: nowrap;
  }

  /* ── History fold ───────────────────────────────────────── */
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
    font-size: var(--text-md);
    font-weight: 500;
    letter-spacing: -0.01em;
    transition: color var(--dur-fast) var(--ease);
  }
  .fold-meta {
    margin-left: auto;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
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
  .fold-body { padding: 0 var(--s-5) var(--s-4); }

  .history-list { display: flex; flex-direction: column; }
  .history-month {
    padding: var(--s-6) 0 var(--s-2);
    border-bottom: 1px solid var(--line);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .history-month:first-child { padding-top: var(--s-4); }
  .history-item {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 0 var(--s-2);
    margin: 0 calc(-1 * var(--s-3));
    padding: 0 var(--s-1) 0 var(--s-3);
    border-bottom: 1px solid var(--line);
    transition: background-color var(--dur-fast) var(--ease);
  }
  .history-item:hover { background: var(--hover-wash); }
  .history-item.open { background: var(--fjord-wash); }
  .history-head {
    display: flex;
    align-items: center;
    gap: var(--s-4);
    min-width: 0;
    min-height: 56px;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .hist-date {
    display: flex;
    flex: none;
    align-items: baseline;
    gap: var(--s-2);
    min-width: 6.5rem;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .hist-dom {
    min-width: 1.4rem;
    color: var(--ink);
    font-size: var(--text-md);
  }
  .hist-sub { color: var(--ink-3); font-size: var(--text-xs); }
  .history-item.today .hist-dom { color: var(--fjord); }
  .hist-day {
    flex: none;
    color: var(--ink-2);
    font-size: var(--text-sm);
    font-weight: 500;
  }
  .hist-bw {
    margin-left: auto;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .hist-bw small { color: var(--ink-3); font-size: var(--text-xs); }
  .hist-bw + .hist-chevron { margin-left: 0; }
  /* The chevron sits at the far right whether or not the day has a weight. */
  .hist-chevron {
    flex: none;
    width: 1.25rem;
    margin-left: auto;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-md);
    text-align: center;
  }
  .history-item.open .hist-chevron { color: var(--fjord); }
  .history-detail {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
    padding: var(--s-3) 0 var(--s-4);
    border-top: 1px solid var(--line);
  }
  .detail-stats {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .detail-ex { display: flex; flex-direction: column; gap: 6px; }
  .detail-ex-name {
    display: inline-flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    color: var(--ink);
    font-size: var(--text-sm);
    font-weight: 500;
  }
  .detail-ex-equip {
    display: inline-flex;
    align-items: center;
    height: 20px;
    padding: 0 7px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-pill);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 400;
  }
  .detail-sets { display: flex; flex-wrap: wrap; gap: 6px; }
  .set-badge {
    padding: 3px var(--s-2);
    border-radius: var(--radius-xs);
    background: var(--surface-2);
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  @media (max-width: 640px) {
    .page { gap: var(--s-5); padding-top: var(--s-6); }
    .signout { min-height: 40px; }

    .progress-card { padding: var(--s-4); gap: var(--s-4); }
    /* Two numbers side by side, the pace verdict on its own full-width row. */
    .trend-stats { grid-template-columns: repeat(2, minmax(0, 1fr)); column-gap: var(--s-4); }
    .trend-stat:last-child { grid-column: 1 / -1; }

    .numbers-head { padding: var(--s-4); }
    .numbers-body { padding: 0 var(--s-4) var(--s-1); }
    .rate-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); column-gap: var(--s-4); }
    .r-val { font-size: var(--text-lg); }

    /* Four forecast cells in a tidy 2×2 rather than auto-fit's 3 + 1. */
    .forecast { grid-template-columns: repeat(2, minmax(0, 1fr)); }

    .fold summary { padding: 0 var(--s-4); }
    .fold-body { padding: 0 var(--s-4) var(--s-3); }
    .history-head { gap: var(--s-3); }
    .hist-date { min-width: 5.5rem; }
  }
</style>
