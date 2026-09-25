<script lang="ts">
  /* ─────────────────────────────────────────────────────────
   * WORKOUT REPORT — a day, week, or month at a glance
   *
   * Reading order, most important first:
   *   · period switch + arrows   — what you're looking at
   *   · score + one-line summary — "was it a good one?"
   *   · four tiles, each with a change vs the compared period
   *   · three short note columns — went well / work on / next
   *   · the week strip or month calendar (tap a day to open it)
   *   · a closed "breakdown" fold for everything else
   *
   * All the maths is in `workoutReport.ts`; this only formats.
   * ───────────────────────────────────────────────────────── */

  import { onMount } from 'svelte';
  import { ChevronLeft, ChevronRight, Check, Target, ArrowRight } from 'lucide-svelte';
  import LoadingState from '$lib/components/LoadingState.svelte';
  import type { TrainingLog, BodyweightEntry } from '$lib/workout';
  import {
    periodRange,
    shiftAnchor,
    clipRange,
    elapsedDays,
    defaultCompareAnchor,
    loggedDates,
    buildReport,
    reportNotes,
    progressions,
    funFacts,
    calendar,
    grade,
    setText,
    type Period,
    type Range,
    type Report
  } from '$lib/workoutReport';

  export let log: TrainingLog;
  export let bodyweight: BodyweightEntry[] = [];
  export let today: string;
  export let loading = false;

  const PERIODS: { value: Period; label: string }[] = [
    { value: 'day', label: 'Day' },
    { value: 'week', label: 'Week' },
    { value: 'month', label: 'Month' }
  ];
  const STORE_KEY = 'workout-report-period';

  let period: Period = 'week';
  let anchor = today;
  /** null → the default comparison for the current range. */
  let cmpAnchor: string | null = null;
  let showAllLifts = false;
  /** Set when a restored "day" view is waiting for the log to pick its day. */
  let pickLatestDay = false;

  onMount(() => {
    try {
      const saved = localStorage.getItem(STORE_KEY);
      if (saved === 'day' || saved === 'week' || saved === 'month') {
        period = saved;
        pickLatestDay = saved === 'day';
      }
    } catch {
      /* storage blocked: keep the default */
    }
  });

  $: dates = loggedDates(log);
  $: if (pickLatestDay && dates.length) {
    anchor = lastLoggedBy(today) ?? today;
    pickLatestDay = false;
  }
  $: range = periodRange(period, anchor, today);
  // Where the compare arrows step from — always a whole period.
  $: cmpBase = periodRange(period, cmpAnchor ?? defaultCompareAnchor(range, log), today);
  // A period still in progress compares against the same number of days, so
  // Wednesday isn't measured against a whole finished week.
  $: cmpRange = range.current && period !== 'day' ? clipRange(cmpBase, elapsedDays(range, today)) : cmpBase;
  $: report = buildReport(log, range, bodyweight);
  $: cmp = buildReport(log, cmpRange, bodyweight);
  $: notes = reportNotes(report, cmp, log);
  // "Next session" only makes sense looking at now, not at a past month.
  $: nextSteps =
    range.current || (period === 'day' && anchor === dates[dates.length - 1])
      ? progressions(log, period === 'day' ? report.sessions[0]?.day_label : undefined).slice(0, 3)
      : [];
  $: cells = period === 'day' ? [] : calendar(log, range, today);
  $: facts = funFacts(report, log);
  $: dayLabel = report.sessions[0]?.day_label ?? '';
  $: liftRows = period === 'day' || showAllLifts ? report.lifts : report.lifts.filter((l) => l.key_lift);

  // ---- navigation ----------------------------------------------------------

  /** Latest logged date on or before `iso`. */
  const lastLoggedBy = (iso: string) => [...dates].reverse().find((d) => d <= iso);

  function setPeriod(p: Period) {
    pickLatestDay = false;
    if (p === 'day') {
      // Open the latest session inside the period being viewed, else the latest overall.
      anchor = lastLoggedBy(range.end < today ? range.end : today) ?? today;
    } else if (period === 'day') {
      anchor = range.start;
    }
    period = p;
    cmpAnchor = null;
    showAllLifts = false;
    try {
      localStorage.setItem(STORE_KEY, p);
    } catch {
      /* ignore */
    }
  }

  /**
   * Step the viewed (or compared) period. Days jump between logged sessions,
   * since stepping through rest days one at a time finds nothing.
   */
  function step(r: Range, dir: -1 | 1): string | null {
    if (r.period === 'day') {
      const d = dir < 0 ? lastLoggedBy(shiftAnchor(r, -1)) : dates.find((x) => x > r.start && x <= today);
      return d ?? null;
    }
    const next = shiftAnchor(r, dir);
    return next > today ? null : next;
  }

  $: canPrev = step(range, -1) !== null && (dates.length ? range.start > dates[0] : false);
  $: canNext = !range.current && step(range, 1) !== null;
  $: cmpCanPrev = step(cmpBase, -1) !== null && (dates.length ? cmpBase.start > dates[0] : false);
  $: cmpCanNext = step(cmpBase, 1) !== null && cmpBase.end < range.start;

  function go(dir: -1 | 1) {
    const a = step(range, dir);
    if (a) {
      anchor = a;
      cmpAnchor = null;
    }
  }
  function goCmp(dir: -1 | 1) {
    const a = step(cmpBase, dir);
    if (a) cmpAnchor = a;
  }
  function openDay(date: string) {
    period = 'day';
    anchor = date;
    cmpAnchor = null;
  }
  function goToday() {
    anchor = period === 'day' ? lastLoggedBy(today) ?? today : today;
    cmpAnchor = null;
  }

  // ---- formatting ----------------------------------------------------------

  const fmtKg = (kg: number) =>
    kg >= 10_000 ? `${(kg / 1000).toFixed(1)} t` : `${kg.toLocaleString('en-GB')} kg`;

  type Delta = { text: string; dir: 'up' | 'down' | 'same'; spoken: string } | null;

  function delta(now: number, before: number, mode: 'pct' | 'abs', unit = ''): Delta {
    if (!cmp.sessions.length) return null;
    const diff = now - before;
    if (mode === 'pct') {
      if (before <= 0) return null;
      const p = Math.round((diff / before) * 100);
      if (p === 0) return { text: '=', dir: 'same', spoken: 'no change' };
      return { text: `${p > 0 ? '+' : '−'}${Math.abs(p)}%`, dir: p > 0 ? 'up' : 'down', spoken: `${p > 0 ? 'up' : 'down'} ${Math.abs(p)} percent` };
    }
    const d = Math.round(diff * 10) / 10;
    if (d === 0) return { text: '=', dir: 'same', spoken: 'no change' };
    return {
      text: `${d > 0 ? '+' : '−'}${Math.abs(d)}${unit}`,
      dir: d > 0 ? 'up' : 'down',
      spoken: `${d > 0 ? 'up' : 'down'} ${Math.abs(d)}${unit}`
    };
  }

  type Tile = { label: string; value: string; sub: string; delta: Delta };
  $: tiles = ((r: Report, c: Report): Tile[] => [
    period === 'day'
      ? {
          label: 'exercises',
          value: r.plannedExercises ? `${r.doneExercises}/${r.plannedExercises}` : String(r.lifts.length),
          sub: r.plannedExercises ? 'of the plan' : 'logged',
          delta: delta(r.lifts.length, c.lifts.length, 'abs')
        }
      : period === 'week'
        ? {
            label: 'lifts',
            value: `${r.liftCount}/4`,
            sub: r.legacySessions ? 'old-plan sessions' : r.activeDays > r.liftCount ? `${r.activeDays} active days` : 'of the plan',
            delta: delta(r.liftCount, c.liftCount, 'abs')
          }
        : {
            label: 'sessions',
            value: String(r.sessions.length),
            sub: `plan ≈ ${Math.round(r.liftTarget)} lifts`,
            delta: delta(r.sessions.length, c.sessions.length, 'abs')
          },
    {
      label: 'sets',
      value: String(r.sets),
      sub: r.plannedSets ? `${r.doneSets} of ${r.plannedSets} planned` : 'logged',
      delta: delta(r.sets, c.sets, 'abs')
    },
    {
      label: 'volume',
      value: fmtKg(r.volumeKg),
      sub: 'load × reps',
      delta: delta(r.volumeKg, c.volumeKg, 'pct')
    },
    {
      label: 'cardio',
      value: `${r.cardioMin} min`,
      sub: r.cardioTarget ? `of ${r.cardioTarget} planned` : `${r.cardioKcal} kcal`,
      delta: delta(r.cardioMin, c.cardioMin, 'abs', ' min')
    }
  ])(report, cmp);

  $: summary = ((r: Report) => {
    const bits: string[] = [];
    if (period === 'day') {
      if (dayLabel) bits.push(dayLabel);
      if (r.plannedExercises) bits.push(`${r.doneExercises} of ${r.plannedExercises} exercises`);
    } else if (period === 'week') {
      bits.push(`${r.liftCount} of 4 lifts`);
    } else {
      bits.push(`${r.liftCount} lifting ${r.liftCount === 1 ? 'session' : 'sessions'}`);
    }
    bits.push(`${r.sets} sets`);
    if (r.prs.length) bits.push(`${r.prs.length} new ${r.prs.length === 1 ? 'best' : 'bests'}`);
    return bits.join(' · ');
  })(report);

  $: noun = period === 'day' ? 'session' : period;
  // The ring's arc: circumference of r = 26.
  const RING = 2 * Math.PI * 26;

  $: maxMuscle = Math.max(1, ...report.muscles.map((m) => Math.max(m.sets, m.target)));
  $: muscleRows = report.muscles.filter((m) => m.target > 0 || m.sets > 0);

  const pctText = (v: number) => `${Math.round(v * 100)}%`;
  const cellLabel = (labels: string[]) =>
    labels.map((l) => (l === 'Cardio' ? 'C' : l.replace(/[^A-Z]/g, ''))).join(' ') || '·';
  const fmtDay = (iso: string) =>
    new Date(iso + 'T00:00:00Z').toLocaleDateString('en-GB', {
      timeZone: 'UTC',
      weekday: 'short',
      day: 'numeric',
      month: 'short'
    });
  const WEEKDAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
</script>

<section class="report" aria-labelledby="report-title">
  <div class="report-head">
    <h2 id="report-title">Report</h2>
    <div class="seg" role="group" aria-label="Report period">
      {#each PERIODS as p}
        <button type="button" aria-pressed={period === p.value} on:click={() => setPeriod(p.value)}>
          {p.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="nav">
    <button
      type="button"
      class="nav-btn"
      on:click={() => go(-1)}
      disabled={!canPrev}
      aria-label={period === 'day' ? 'Previous session' : `Previous ${period}`}
    >
      <ChevronLeft size={16} />
    </button>
    <p class="nav-label" aria-live="polite">
      <strong>{range.label}</strong>
      {#if range.relative}<span class="nav-rel">{range.relative}</span>{/if}
    </p>
    <button
      type="button"
      class="nav-btn"
      on:click={() => go(1)}
      disabled={!canNext}
      aria-label={period === 'day' ? 'Next session' : `Next ${period}`}
    >
      <ChevronRight size={16} />
    </button>
    {#if !range.current && !(period === 'day' && anchor === lastLoggedBy(today))}
      <button type="button" class="link-btn" on:click={goToday}>
        {period === 'day' ? 'latest' : `this ${period}`}
      </button>
    {/if}
  </div>

  {#if loading && !dates.length}
    <div class="empty"><LoadingState label="Loading your log" /></div>
  {:else if !report.sessions.length}
    <div class="empty">
      <p>
        {#if period === 'day'}
          Nothing logged on this day — a rest day.
        {:else if range.current}
          Nothing logged {range.relative} yet. The first session starts the report.
        {:else}
          Nothing logged in {range.label}.
        {/if}
      </p>
      {#if dates.length && lastLoggedBy(range.end < today ? range.end : today)}
        <button
          type="button"
          class="link-btn"
          on:click={() => openDay(lastLoggedBy(range.end < today ? range.end : today) ?? today)}
        >
          Open the last session <ArrowRight size={13} />
        </button>
      {/if}
    </div>
  {:else}
    <!-- Verdict: the score and a one-line summary. -->
    {#if report.score !== null}
      <div class="verdict">
        <svg class="ring" viewBox="0 0 64 64" aria-hidden="true">
          <circle cx="32" cy="32" r="26" class="ring-track" />
          <circle
            cx="32"
            cy="32"
            r="26"
            class="ring-fill"
            stroke-dasharray="{(report.score / 100) * RING} {RING}"
            transform="rotate(-90 32 32)"
          />
          <text x="32" y="37" text-anchor="middle" class="ring-num">{report.score}</text>
        </svg>
        <div class="verdict-text">
          <p class="verdict-grade">
            <span class="sr-only">Score {report.score} out of 100:</span>
            {grade(report.score)} {noun}{range.current && period !== 'day' ? ' so far' : ''}
          </p>
          <p class="verdict-sum">{summary}</p>
        </div>
      </div>
    {/if}

    <ul class="tiles">
      {#each tiles as t}
        <li class="tile">
          <span class="t-val">{t.value}</span>
          <span class="t-label">{t.label}</span>
          <span class="t-sub">{t.sub}</span>
          {#if t.delta}
            <span class="t-delta {t.delta.dir}">
              <span aria-hidden="true">{t.delta.dir === 'up' ? '▲' : t.delta.dir === 'down' ? '▼' : ''} {t.delta.text}</span>
              <span class="sr-only">{t.delta.spoken} compared with {cmpRange.label}</span>
            </span>
          {/if}
        </li>
      {/each}
    </ul>

    <!-- Compare picker: defaults to the period before (or the last same-day session). -->
    <div class="compare">
      <span class="compare-lead">Compared with</span>
      <button
        type="button"
        class="nav-btn small"
        on:click={() => goCmp(-1)}
        disabled={!cmpCanPrev}
        aria-label="Compare with an earlier {period === 'day' ? 'session' : period}"
      >
        <ChevronLeft size={14} />
      </button>
      <strong class="compare-label" aria-live="polite">
        {#if period === 'day' && cmp.sessions[0]?.day_label}
          {cmp.sessions[0].day_label}, {cmpRange.label}
        {:else}
          {cmpRange.label}
        {/if}
      </strong>
      <button
        type="button"
        class="nav-btn small"
        on:click={() => goCmp(1)}
        disabled={!cmpCanNext}
        aria-label="Compare with a later {period === 'day' ? 'session' : period}"
      >
        <ChevronRight size={14} />
      </button>
      {#if cmpAnchor}
        <button type="button" class="link-btn" on:click={() => (cmpAnchor = null)}>reset</button>
      {/if}
      {#if !cmp.sessions.length}<span class="compare-none">nothing logged then</span>{/if}
    </div>

    {#if notes.good.length || notes.improve.length || nextSteps.length}
      <div class="notes">
        {#if notes.good.length}
          <div class="note-col good">
            <h3><Check size={14} strokeWidth={2.5} aria-hidden="true" /> Went well</h3>
            <ul>
              {#each notes.good as n}<li>{n.text}</li>{/each}
            </ul>
          </div>
        {/if}
        {#if notes.improve.length}
          <div class="note-col improve">
            <h3><Target size={14} aria-hidden="true" /> To work on</h3>
            <ul>
              {#each notes.improve as n}<li>{n.text}</li>{/each}
            </ul>
          </div>
        {/if}
        {#if nextSteps.length}
          <div class="note-col next">
            <h3><ArrowRight size={14} aria-hidden="true" /> Next {period === 'day' && dayLabel ? dayLabel : 'session'}</h3>
            <ul>
              {#each nextSteps as n}<li>{n.text}</li>{/each}
            </ul>
          </div>
        {/if}
      </div>
    {/if}
  {/if}

  {#if cells.length && dates.length}
    <!-- Week strip / month calendar. Logged days are buttons that open that day. -->
    <div class="cal" class:month={period === 'month'}>
      {#if period === 'month'}
        {#each WEEKDAYS as w}<span class="cal-wd" aria-hidden="true">{w}</span>{/each}
      {/if}
      {#each cells as c}
        {#if !c.inRange && period === 'month'}
          <span class="cal-cell pad" aria-hidden="true"></span>
        {:else if c.level > 0}
          <button
            type="button"
            class="cal-cell lvl-{c.level}"
            class:today={c.date === today}
            on:click={() => openDay(c.date)}
            aria-label="{fmtDay(c.date)}: {c.labels.join(', ') || 'logged'}, {c.sets} sets. Open this day."
            title="{fmtDay(c.date)} · {c.labels.join(', ')} · {c.sets} sets"
          >
            <span class="cal-dom">{period === 'week' ? WEEKDAYS[c.weekday] : c.dom}</span>
            <span class="cal-tag">{cellLabel(c.labels)}</span>
          </button>
        {:else}
          <span class="cal-cell empty-day" class:future={c.future} class:today={c.date === today}>
            <span class="cal-dom">{period === 'week' ? WEEKDAYS[c.weekday] : c.dom}</span>
            <span class="cal-tag" aria-hidden="true">{c.future ? '' : '–'}</span>
            <span class="sr-only">{fmtDay(c.date)}: {c.future ? 'upcoming' : 'rest'}</span>
          </span>
        {/if}
      {/each}
    </div>
    <p class="cal-key">
      <span class="swatch lvl-1" aria-hidden="true"></span>
      <span class="swatch lvl-2" aria-hidden="true"></span>
      <span class="swatch lvl-3" aria-hidden="true"></span>
      darker = more sets · UA = Upper A, LB = Lower B, C = cardio · tap a day to open it
    </p>
  {/if}

  {#if report.sessions.length}
    <details class="more">
      <summary>Breakdown</summary>
      <div class="more-body">
        <h4>
          {period === 'day' ? 'Exercises' : 'Key lifts'}
          <small>best set · change vs the last session before</small>
        </h4>
        {#if liftRows.length}
          <ul class="lifts">
            {#each liftRows as l}
              <li class="lift">
                <span class="lift-name">
                  {l.exercise}
                  {#if l.equipment}<em>{l.equipment}</em>{/if}
                  {#if l.pr}<span class="pr-tag">new best</span>{/if}
                </span>
                <span class="lift-sets">
                  {#if period === 'day'}
                    {l.lastSets.map((s) => setText(l.kind, s.reps, s.weight_g)).join(', ')}
                  {:else if l.best}
                    {setText(l.kind, l.best.reps, l.best.weight_g)} · {l.sets} sets
                  {/if}
                </span>
                <span
                  class="lift-change"
                  class:up={(l.change ?? 0) > 0.005}
                  class:down={(l.change ?? 0) < -0.005}
                >
                  {#if l.change === null}
                    first
                  {:else if Math.abs(l.change) < 0.005}
                    =
                  {:else}
                    <span aria-hidden="true">{l.change > 0 ? '▲' : '▼'}</span>
                    <span class="sr-only">{l.change > 0 ? 'up' : 'down'}</span>
                    {pctText(Math.abs(l.change))}
                  {/if}
                </span>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="hint">No key lifts logged in this {period}.</p>
        {/if}
        {#if period !== 'day' && report.lifts.length > report.lifts.filter((l) => l.key_lift).length}
          <button type="button" class="link-btn" on:click={() => (showAllLifts = !showAllLifts)}>
            {showAllLifts ? 'key lifts only' : `show all ${report.lifts.length} lifts`}
          </button>
        {/if}
        <p class="hint">
          Change compares the best set by estimated max (weight × reps on one scale), so
          24 kg × 8 and 22 kg × 10 read as about even. Only the same implement is compared.
        </p>

        {#if period !== 'day'}
          <h4>
            Sets per muscle
            <small>bar = done · tick = the plan's {period === 'week' ? 'week' : `${Math.round(report.weeks * 10) / 10} weeks`}</small>
          </h4>
          <ul class="muscles">
            {#each muscleRows as m}
              <li class="muscle" class:low={range.past && m.target >= 2 && m.sets < m.target * 0.6}>
                <span class="m-name">{m.muscle}</span>
                <span class="m-track" aria-hidden="true">
                  <span class="m-fill" style="width: {(m.sets / maxMuscle) * 100}%"></span>
                  {#if m.target > 0}
                    <span class="m-tick" style="left: {(m.target / maxMuscle) * 100}%"></span>
                  {/if}
                </span>
                <span class="m-num">{m.sets}<small> / {m.target}</small></span>
              </li>
            {/each}
          </ul>
          <p class="hint">
            A set counts fully for the muscle it's for and half for helpers, so a bench set is 1 chest
            plus ½ triceps and ½ shoulders.
          </p>
        {/if}

        <h4>Side by side</h4>
        <table class="side">
          <thead>
            <tr>
              <th scope="col"><span class="sr-only">Measure</span></th>
              <th scope="col">{range.label}</th>
              <th scope="col">{cmpRange.label}</th>
            </tr>
          </thead>
          <tbody>
            <tr><th scope="row">sessions</th><td>{report.sessions.length}</td><td>{cmp.sessions.length}</td></tr>
            <tr><th scope="row">sets</th><td>{report.sets}</td><td>{cmp.sets}</td></tr>
            <tr><th scope="row">reps</th><td>{report.reps}</td><td>{cmp.reps}</td></tr>
            <tr><th scope="row">volume</th><td>{fmtKg(report.volumeKg)}</td><td>{fmtKg(cmp.volumeKg)}</td></tr>
            <tr><th scope="row">cardio</th><td>{report.cardioMin} min</td><td>{cmp.cardioMin} min</td></tr>
            <tr><th scope="row">new bests</th><td>{report.prs.length}</td><td>{cmp.prs.length}</td></tr>
            <tr>
              <th scope="row">avg bodyweight</th>
              <td>{report.avgBodyweightKg ?? '—'}{report.avgBodyweightKg ? ' kg' : ''}</td>
              <td>{cmp.avgBodyweightKg ?? '—'}{cmp.avgBodyweightKg ? ' kg' : ''}</td>
            </tr>
            <tr>
              <th scope="row">score</th>
              <td>{report.score ?? '—'}</td>
              <td>{cmp.score ?? '—'}</td>
            </tr>
          </tbody>
        </table>

        {#if report.parts}
          <h4>How the score works</h4>
          <p class="hint">
            {#if period === 'day'}
              Plan completion {pctText(report.parts.completion)} (60%), cardio {pctText(report.parts.cardio)}
              (20%), key lifts held or improved {pctText(report.parts.progress)} (20%).
            {:else}
              Lift days {pctText(report.parts.attendance)} (40%), planned sets done
              {pctText(report.parts.completion)} (25%), cardio {pctText(report.parts.cardio)} (15%), key
              lifts held or improved {pctText(report.parts.progress)} (20%). Turning up counts most: a
              finished week of average sessions beats two great ones.
            {/if}
          </p>
        {/if}

        {#if facts.length}
          <h4>Fun facts</h4>
          <ul class="facts">
            {#each facts as f}
              <li><span class="fact-emoji" aria-hidden="true">{f.emoji}</span> {f.text}</li>
            {/each}
          </ul>
        {/if}
      </div>
    </details>
  {/if}
</section>

<style>
  .report {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    border: 1px solid var(--border);
    border-radius: 0.625rem;
    padding: 1rem;
  }

  .report-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  .report-head h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  /* Segmented period switch. aria-pressed carries the state, not colour alone:
     the pressed button also gets a solid border and bold label. */
  .seg {
    display: inline-flex;
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 0.15rem;
    gap: 0.15rem;
  }
  .seg button {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid transparent;
    border-radius: 999px;
    padding: 0.3rem 0.75rem;
    min-height: 2rem;
    cursor: var(--cursor-pointer);
  }
  .seg button:hover { color: var(--text-primary); }
  .seg button[aria-pressed='true'] {
    color: var(--blueprint);
    font-weight: 700;
    border-color: var(--blueprint);
    background: var(--blueprint-tint);
  }

  button:focus-visible {
    outline: 2px solid var(--blueprint);
    outline-offset: 2px;
  }

  .nav {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .nav-label {
    margin: 0;
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    min-width: 8.5rem;
    justify-content: center;
    font-size: 0.95rem;
    color: var(--text-primary);
  }
  .nav-rel {
    font-family: var(--font-mono);
    font-size: 0.66rem;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }
  .nav-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    cursor: var(--cursor-pointer);
  }
  .nav-btn.small { width: 1.6rem; height: 1.6rem; }
  .nav-btn:hover:not(:disabled) { color: var(--text-primary); border-color: var(--border-strong); }
  .nav-btn:disabled { opacity: 0.35; cursor: default; }
  .link-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.78rem;
    color: var(--blueprint);
    background: transparent;
    border: none;
    padding: 0.25rem 0.1rem;
    text-decoration: underline;
    text-underline-offset: 0.2em;
    cursor: var(--cursor-pointer);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.4rem;
    padding: 0.85rem;
    border: 1px dashed var(--border);
    border-radius: 0.5rem;
  }
  .empty p { margin: 0; font-size: 0.875rem; color: var(--text-secondary); }

  /* ── Verdict ── */
  .verdict { display: flex; align-items: center; gap: 0.9rem; }
  .ring { width: 4rem; height: 4rem; flex: none; }
  .ring-track { fill: none; stroke: var(--blueprint-tint-strong); stroke-width: 6; }
  .ring-fill { fill: none; stroke: var(--blueprint); stroke-width: 6; stroke-linecap: round; }
  .ring-num { font-family: var(--font-mono); font-size: 15px; font-weight: 700; fill: var(--text-primary); }
  .verdict-text { display: flex; flex-direction: column; gap: 0.15rem; min-width: 0; }
  .verdict-grade {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 700;
    color: var(--text-primary);
  }
  .verdict-grade::first-letter { text-transform: uppercase; }
  .verdict-sum {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 0.74rem;
    line-height: 1.45;
    color: var(--text-secondary);
  }

  /* ── Tiles ── */
  .tiles {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.6rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: 0.65rem 0.7rem;
    border: 1px solid var(--border-subtle);
    border-radius: 0.5rem;
    min-width: 0;
  }
  .t-val { font-family: var(--font-mono); font-size: 1.05rem; font-weight: 700; color: var(--text-primary); }
  .t-label { font-size: 0.62rem; letter-spacing: 0.05em; text-transform: uppercase; color: var(--text-tertiary); }
  .t-sub { font-family: var(--font-mono); font-size: 0.62rem; line-height: 1.35; color: var(--text-faint); }
  .t-delta {
    font-family: var(--font-mono);
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--text-tertiary);
    margin-top: 0.15rem;
  }
  .t-delta.up { color: var(--blueprint); }
  .t-delta.down { color: #e67e22; }

  .compare {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
    font-size: 0.78rem;
    color: var(--text-tertiary);
  }
  .compare-label { color: var(--text-secondary); font-weight: 600; }
  .compare-none { font-style: italic; }

  /* ── Notes ── */
  .notes {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: 0.6rem;
  }
  .note-col {
    padding: 0.75rem 0.85rem;
    border: 1px solid var(--border-subtle);
    border-left-width: 3px;
    border-radius: 0.5rem;
  }
  .note-col.good { border-left-color: var(--blueprint); }
  .note-col.improve { border-left-color: #e67e22; }
  .note-col.next { border-left-color: var(--border-strong); }
  .note-col h3 {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    margin: 0 0 0.45rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
  }
  .note-col.good h3 { color: var(--blueprint); }
  .note-col.improve h3 { color: #e67e22; }
  .note-col ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.4rem; }
  .note-col li { font-size: 0.82rem; line-height: 1.5; color: var(--text-secondary); }

  /* ── Calendar ── */
  .cal {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 0.3rem;
  }
  .cal-wd {
    font-family: var(--font-mono);
    font-size: 0.6rem;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    text-align: center;
    color: var(--text-tertiary);
  }
  .cal-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.1rem;
    min-height: 2.75rem;
    padding: 0.25rem 0.1rem;
    border: 1px solid var(--border-subtle);
    border-radius: 0.4rem;
    background: transparent;
    font-family: var(--font-mono);
    color: var(--text-primary);
  }
  .cal.month .cal-cell { min-height: 2.5rem; }
  .cal-cell.pad { border: none; }
  button.cal-cell { cursor: var(--cursor-pointer); }
  button.cal-cell:hover { border-color: var(--blueprint); }
  .cal-cell.empty-day { color: var(--text-faint); }
  .cal-cell.future { border-style: dashed; }
  .cal-cell.today { border-color: var(--blueprint); }
  .cal-dom { font-size: 0.7rem; font-weight: 600; }
  .cal-tag { font-size: 0.58rem; letter-spacing: 0.04em; color: inherit; opacity: 0.85; }
  /* One hue, lighter to heavier. The day-label tag carries identity, so the
     shade only has to say "how much". */
  .lvl-1 { background: rgb(165 163 255 / 14%); }
  .lvl-2 { background: rgb(165 163 255 / 30%); }
  .lvl-3 { background: rgb(165 163 255 / 50%); }
  .cal-key {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    flex-wrap: wrap;
    margin: -0.4rem 0 0;
    font-size: 0.7rem;
    color: var(--text-tertiary);
  }
  .swatch {
    display: inline-block;
    width: 0.7rem;
    height: 0.7rem;
    border-radius: 0.15rem;
    border: 1px solid var(--border-subtle);
  }

  /* ── Breakdown fold ── */
  .more { border-top: 1px solid var(--border-subtle); padding-top: 0.75rem; }
  .more summary {
    font-family: var(--font-mono);
    font-size: 0.74rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
    cursor: var(--cursor-pointer);
    list-style: none;
    display: flex;
    justify-content: space-between;
    padding: 0.2rem 0;
  }
  .more summary::-webkit-details-marker { display: none; }
  .more summary::after { content: '+'; color: var(--text-tertiary); }
  .more[open] summary::after { content: '–'; }
  .more summary:focus-visible { outline: 2px solid var(--blueprint); outline-offset: 2px; }
  .more-body { padding-top: 0.5rem; }
  .more h4 {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin: 1.25rem 0 0.55rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
  }
  .more h4:first-child { margin-top: 0.4rem; }
  .more h4 small {
    font-size: 0.66rem;
    font-weight: 400;
    letter-spacing: 0.02em;
    text-transform: none;
    color: var(--text-faint);
  }
  .hint { font-size: 0.78rem; line-height: 1.5; color: var(--text-tertiary); margin: 0.6rem 0 0; }

  .lifts, .muscles, .facts { list-style: none; margin: 0; padding: 0; }
  .lift {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(0, 1.5fr) 3.5rem;
    gap: 0.6rem;
    align-items: baseline;
    padding: 0.45rem 0;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 0.82rem;
  }
  .lift:last-child { border-bottom: none; }
  .lift-name { color: var(--text-primary); }
  .lift-name em {
    font-family: var(--font-mono);
    font-style: normal;
    font-size: 0.64rem;
    color: var(--text-tertiary);
    margin-left: 0.3rem;
  }
  .pr-tag {
    font-family: var(--font-mono);
    font-size: 0.58rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--blueprint);
    border: 1px solid var(--blueprint);
    border-radius: 999px;
    padding: 0 0.35rem;
    margin-left: 0.3rem;
    white-space: nowrap;
  }
  .lift-sets { font-family: var(--font-mono); font-size: 0.72rem; color: var(--text-secondary); }
  .lift-change {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    text-align: right;
    color: var(--text-tertiary);
  }
  .lift-change.up { color: var(--blueprint); }
  .lift-change.down { color: #e67e22; }

  .muscle { display: flex; align-items: center; gap: 0.6rem; padding: 0.2rem 0; }
  .m-name {
    flex: none;
    width: 5.5rem;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-secondary);
  }
  .muscle.low .m-name { color: #e67e22; }
  .m-track {
    position: relative;
    flex: 1;
    height: 0.5rem;
    border-radius: 0.25rem;
    background: var(--blueprint-tint);
  }
  .m-fill {
    position: absolute;
    inset: 0 auto 0 0;
    border-radius: 0.25rem;
    background: var(--blueprint);
  }
  .m-tick {
    position: absolute;
    top: -0.2rem;
    bottom: -0.2rem;
    width: 2px;
    margin-left: -1px;
    background: var(--text-primary);
  }
  .m-num {
    flex: none;
    min-width: 3.5rem;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 0.74rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .m-num small { font-weight: 400; color: var(--text-tertiary); }

  .side { width: 100%; border-collapse: collapse; font-size: 0.8rem; }
  .side th, .side td {
    padding: 0.35rem 0.4rem;
    border-bottom: 1px solid var(--border-subtle);
    text-align: right;
    font-family: var(--font-mono);
    font-size: 0.74rem;
    color: var(--text-primary);
  }
  .side thead th { color: var(--text-tertiary); font-weight: 600; font-size: 0.68rem; }
  .side tbody th {
    text-align: left;
    font-family: inherit;
    font-weight: 400;
    color: var(--text-secondary);
  }
  .side tr:last-child th, .side tr:last-child td { border-bottom: none; }

  .facts { display: flex; flex-direction: column; gap: 0.4rem; }
  .facts li { font-size: 0.82rem; line-height: 1.5; color: var(--text-secondary); }
  .fact-emoji { display: inline-block; width: 1.4rem; }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  @media (max-width: 640px) {
    .report { padding: 0.85rem; }
    .tiles { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .lift { grid-template-columns: minmax(0, 1fr) 3.25rem; }
    .lift-sets { grid-column: 1; grid-row: 2; }
    .lift-change { grid-column: 2; grid-row: 1; }
    .cal-tag { font-size: 0.52rem; }
  }

  @media (prefers-reduced-motion: no-preference) {
    .ring-fill { transition: stroke-dasharray 0.4s ease; }
  }
</style>
