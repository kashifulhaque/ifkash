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
        <svg class="score-ring" viewBox="0 0 64 64" aria-hidden="true">
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
    gap: var(--s-5);
    padding: var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
  }

  .report-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
  }
  .report-head h2 {
    margin: 0;
    font-size: var(--text-md);
    letter-spacing: -0.01em;
  }

  /* Segmented period switch. aria-pressed carries the state; the pressed
     button also changes fill and weight, not colour alone. */
  .seg {
    display: inline-flex;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
  }
  .seg button {
    min-height: 36px;
    padding: 0 var(--s-3);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
  }
  .seg button:hover { background: var(--hover-wash); color: var(--ink); }
  .seg button[aria-pressed='true'] {
    background: var(--fjord-wash);
    color: var(--fjord);
    font-weight: 500;
  }

  /* ── Period navigation ── */
  .nav {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2);
  }
  .nav-label {
    display: flex;
    align-items: baseline;
    justify-content: center;
    gap: var(--s-2);
    min-width: 8.5rem;
    margin: 0;
    color: var(--ink);
    font-size: 15px;
  }
  .nav-label strong { font-weight: 500; }
  .nav-rel {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .nav-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-2);
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease),
      background-color var(--dur-fast) var(--ease);
  }
  .nav-btn.small { width: 36px; height: 36px; }
  .nav-btn:hover:not(:disabled) {
    border-color: var(--ink);
    background: var(--hover-wash);
    color: var(--ink);
  }
  .nav-btn:disabled { opacity: 0.35; cursor: default; }
  .link-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    min-height: 32px;
    padding: 0 var(--s-1);
    border: 0;
    background: transparent;
    color: var(--ink);
    font-size: var(--text-sm);
    text-decoration: underline;
    text-decoration-color: var(--line-strong);
    text-decoration-thickness: 1px;
    text-underline-offset: 0.22em;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), text-decoration-color var(--dur-fast) var(--ease);
  }
  .link-btn:hover { color: var(--fjord); text-decoration-color: var(--fjord); }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--s-2);
    padding: var(--s-4) 0;
    border-top: 1px solid var(--line);
  }
  .empty p { margin: 0; color: var(--ink-3); font-size: var(--text-sm); }

  /* ── Verdict ── */
  .verdict { display: flex; align-items: center; gap: var(--s-4); }
  .score-ring { flex: none; width: 64px; height: 64px; }
  .ring-track { fill: none; stroke: var(--surface-2); stroke-width: 5; }
  .ring-fill { fill: none; stroke: var(--fjord); stroke-width: 5; stroke-linecap: butt; }
  .ring-num {
    fill: var(--ink);
    font-family: var(--font-mono);
    font-size: 15px;
    font-variant-numeric: tabular-nums;
  }
  .verdict-text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .verdict-grade {
    margin: 0;
    color: var(--ink);
    font-size: var(--text-lg);
    font-weight: 500;
    line-height: 1.25;
    letter-spacing: -0.015em;
  }
  .verdict-grade::first-letter { text-transform: uppercase; }
  .verdict-sum {
    margin: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    line-height: 1.5;
  }

  /* ── Tiles: label, large mono value, note, change ── */
  .tiles {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    column-gap: var(--s-5);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    padding: var(--s-3) 0;
    border-top: 1px solid var(--line);
  }
  .t-label {
    order: -1;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .t-val {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xl);
    font-variant-numeric: tabular-nums;
    line-height: 1.15;
    letter-spacing: -0.03em;
    white-space: nowrap;
  }
  .t-sub {
    color: var(--ink-3);
    font-size: var(--text-xs);
    line-height: 1.4;
  }
  .t-delta {
    margin-top: 2px;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .t-delta.up { color: var(--moss); }
  .t-delta.down { color: var(--clay); }

  .compare {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2);
    color: var(--ink-3);
    font-size: var(--text-sm);
  }
  .compare-lead {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .compare-label { color: var(--ink); font-weight: 500; }
  .compare-none { color: var(--ink-3); font-size: var(--text-xs); }

  /* ── Notes: three ruled columns ── */
  .notes {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
    gap: var(--s-5);
  }
  .note-col {
    padding-top: var(--s-3);
    border-top: 2px solid var(--line-strong);
  }
  .note-col.good { border-top-color: var(--moss); }
  .note-col.improve { border-top-color: var(--clay); }
  .note-col h3 {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0 0 var(--s-2);
    color: var(--ink);
    font-size: 14px;
    font-weight: 500;
    letter-spacing: 0;
  }
  .note-col h3 :global(svg) { color: var(--ink-3); }
  .note-col.good h3 :global(svg) { color: var(--moss); }
  .note-col.improve h3 :global(svg) { color: var(--clay); }
  .note-col ul {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .note-col li {
    color: var(--ink-2);
    font-size: var(--text-sm);
    line-height: 1.55;
  }

  /* ── Calendar ── */
  .cal {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 4px;
  }
  .cal-wd {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 11px;
    text-align: center;
  }
  .cal-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    min-height: 48px;
    padding: var(--s-1) 2px;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .cal.month .cal-cell { min-height: 44px; }
  .cal-cell.pad { border: 0; background: transparent; }
  button.cal-cell {
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease);
  }
  button.cal-cell:hover { border-color: var(--ink-3); }
  .cal-cell.empty-day { color: var(--ink-3); }
  .cal-cell.future { border-style: dashed; background: transparent; }
  .cal-cell.today { border-color: var(--fjord); }
  .cal-dom { font-size: var(--text-xs); }
  .cal-tag { color: inherit; font-size: 10px; letter-spacing: 0.02em; opacity: 0.85; }
  /* One hue, lighter to heavier. The day-label tag carries identity, so the
     shade only has to say "how much". */
  .lvl-1 { background: color-mix(in srgb, var(--fjord) 12%, var(--surface)); }
  .lvl-2 { background: color-mix(in srgb, var(--fjord) 26%, var(--surface)); }
  .lvl-3 { background: color-mix(in srgb, var(--fjord) 44%, var(--surface)); }
  .cal-key {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 6px;
    margin: calc(-1 * var(--s-3)) 0 0;
    color: var(--ink-3);
    font-size: var(--text-xs);
    line-height: 1.5;
  }
  .swatch {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 1px solid var(--line);
    border-radius: var(--radius-xs);
  }

  /* ── Breakdown fold ── */
  .more { border-top: 1px solid var(--line); }
  .more summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 48px;
    color: var(--ink);
    font-size: 15px;
    font-weight: 500;
    list-style: none;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease);
  }
  .more summary:hover { color: var(--fjord); }
  .more summary::-webkit-details-marker { display: none; }
  .more summary::after {
    content: '+';
    width: 1.25rem;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-md);
    text-align: center;
  }
  .more[open] summary::after { content: '−'; }
  .more-body { padding-top: var(--s-2); }
  .more h4 {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px var(--s-2);
    margin: var(--s-6) 0 var(--s-3);
    color: var(--ink);
    font-size: 14px;
    font-weight: 500;
    letter-spacing: 0;
  }
  .more h4:first-child { margin-top: var(--s-2); }
  .more h4 small {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
  }
  .hint {
    margin: var(--s-3) 0 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.55;
  }

  .lifts,
  .muscles,
  .facts { margin: 0; padding: 0; list-style: none; }
  .lifts { border-top: 1px solid var(--line); }
  .lift {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(0, 1.5fr) 4rem;
    align-items: baseline;
    gap: var(--s-3);
    padding: var(--s-2) 0;
    border-bottom: 1px solid var(--line);
    font-size: var(--text-sm);
  }
  .lift-name { color: var(--ink); }
  .lift-name em {
    margin-left: 4px;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 11px;
    font-style: normal;
  }
  .pr-tag {
    display: inline-flex;
    align-items: center;
    height: 18px;
    margin-left: 4px;
    padding: 0 6px;
    border: 1px solid var(--moss);
    border-radius: var(--radius-pill);
    color: var(--moss);
    font-family: var(--font-mono);
    font-size: 11px;
    white-space: nowrap;
  }
  .lift-sets {
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .lift-change {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .lift-change.up { color: var(--moss); }
  .lift-change.down { color: var(--clay); }

  .muscles { display: flex; flex-direction: column; gap: 6px; }
  .muscle { display: flex; align-items: center; gap: var(--s-3); }
  .m-name {
    flex: none;
    width: 6rem;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .muscle.low .m-name { color: var(--clay); }
  .m-track {
    position: relative;
    flex: 1;
    height: 6px;
    border-radius: var(--radius-xs);
    background: var(--surface-2);
  }
  .m-fill {
    position: absolute;
    inset: 0 auto 0 0;
    border-radius: var(--radius-xs);
    background: var(--fjord);
  }
  .m-tick {
    position: absolute;
    top: -4px;
    bottom: -4px;
    width: 2px;
    margin-left: -1px;
    background: var(--ink);
  }
  .m-num {
    flex: none;
    min-width: 3.5rem;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .m-num small { color: var(--ink-3); font-size: var(--text-xs); }

  .side {
    width: 100%;
    border-collapse: collapse;
    border-top: 1px solid var(--line);
    font-size: var(--text-sm);
  }
  .side th,
  .side td {
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--line);
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 400;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .side thead th {
    background: var(--surface-2);
    color: var(--ink-3);
    font-size: var(--text-xs);
  }
  .side tbody th {
    color: var(--ink-2);
    font-family: var(--font-sans);
    text-align: left;
  }
  .side th:first-child,
  .side td:first-child { padding-left: var(--s-2); }

  .facts { display: flex; flex-direction: column; gap: var(--s-2); }
  .facts li { color: var(--ink-2); font-size: var(--text-sm); line-height: 1.55; }
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
    .report { gap: var(--s-4); padding: var(--s-4); }
    .tiles { grid-template-columns: repeat(2, minmax(0, 1fr)); column-gap: var(--s-4); }
    .t-val { font-size: var(--text-lg); }
    .notes { gap: var(--s-4); }
    .cal-cell { min-height: 44px; }
    .cal-tag { font-size: 9px; }
    .lift { grid-template-columns: minmax(0, 1fr) 3.5rem; row-gap: 2px; }
    .lift-sets { grid-column: 1; grid-row: 2; }
    .lift-change { grid-column: 2; grid-row: 1; }
    .m-name { width: 5rem; }
  }

  @media (prefers-reduced-motion: no-preference) {
    .ring-fill { transition: stroke-dasharray var(--dur-slow) var(--ease-out); }
  }
</style>
