<svelte:head>
  <title>Workout Plan — Kashif</title>
  <meta name="description" content="A 5–6 day push/pull/legs program with cardio mixed in — built to gain muscle while cutting." />
</svelte:head>

<script lang="ts">
  import { PUSH as push, PULL as pull, LEGS as legs } from '$lib/workout';

  const split = [
    { day: 'Day 1', focus: 'Push', detail: 'chest / shoulders / triceps' },
    { day: 'Day 2', focus: 'Pull', detail: 'back / biceps' },
    { day: 'Day 3', focus: 'Legs', detail: '' },
    { day: 'Day 4', focus: 'Push', detail: '' },
    { day: 'Day 5', focus: 'Pull', detail: '' },
    { day: 'Day 6', focus: 'Optional', detail: 'Legs or full body + extra cardio' }
  ];
</script>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/">Home</a>
      <span class="separator">/</span>
      <span>Workout</span>
    </div>
    <h1 class="page-title">Workout Plan 🏋️</h1>
    <p class="page-desc">
      A 5–6 day push/pull/legs split with cardio mixed in. Push, pull and legs each hit
      twice across five days, so nothing trains two days straight. Built for the goal:
      build muscle while cutting.
    </p>
    <div class="cta-row">
      <a href="/workout/log" class="cta-link">Log a workout →</a>
    </div>
  </header>

  <section class="content">
    <div class="section">
      <h2>Week Structure</h2>
      <p>Five-day base, sixth day optional. Repeat and rotate.</p>
      <div class="split-table">
        {#each split as row}
          <div class="split-row">
            <div class="split-day">{row.day}</div>
            <div class="split-focus">{row.focus}</div>
            <div class="split-detail">{row.detail}</div>
          </div>
        {/each}
      </div>
      <p>
        This way nothing trains two days straight, and push/pull/legs each get hit twice
        in five days — good for building muscle while cutting.
      </p>
    </div>

    <div class="section">
      <h2>Day 1 &amp; 4 — Push</h2>
      <ul class="exercise-list">
        {#each push as ex}
          <li><span class="ex-name">{ex.name}</span><span class="ex-scheme">{ex.scheme}</span></li>
        {/each}
      </ul>
      <p class="cardio">Cardio: 15 min cycle</p>
    </div>

    <div class="section">
      <h2>Day 2 &amp; 5 — Pull</h2>
      <ul class="exercise-list">
        {#each pull as ex}
          <li><span class="ex-name">{ex.name}</span><span class="ex-scheme">{ex.scheme}</span></li>
        {/each}
      </ul>
      <p class="cardio">Cardio: 15 min crosstrainer</p>
    </div>

    <div class="section">
      <h2>Day 3 (&amp; 6) — Legs</h2>
      <ul class="exercise-list">
        {#each legs as ex}
          <li><span class="ex-name">{ex.name}</span><span class="ex-scheme">{ex.scheme}</span></li>
        {/each}
      </ul>
      <p class="cardio">Cardio: 20 min treadmill incline walk (incline 8–10%, beats the flat hamster wheel)</p>
    </div>

    <div class="section">
      <h2>Cardio Rule</h2>
      <p>
        15–20 min post-lift, every day. Rotate cycle / crosstrainer / treadmill so it
        never gets boring. Don't do cardio before lifting — it kills your strength for
        the compounds.
      </p>
    </div>

    <div class="section">
      <h2>On the Numbers</h2>
      <p>
        178–179 cm, 86–90 kg fluctuating is mostly water and food noise. Ignore daily
        swings, track the weekly average. Target 80 kg in two months from a ~88 kg
        average means about 1 kg/week loss — aggressive but doable with a protein-heavy
        diet plus lifting. Don't go below 0.7–1 kg/week, or you'll burn muscle along with
        the fat.
      </p>
    </div>

    <div class="section">
      <h2>Quick Math Check</h2>
      <p>
        8 kg in 8 weeks = 1 kg/week ≈ 7,700 kcal deficit/week ≈ 1,100 kcal/day deficit.
        That's steep while lifting hard. Better: aim for a 700–800 kcal deficit and lean
        on protein, so loss lands closer to 0.7–0.8 kg/week and recomp does the rest.
        Muscle gain offsets scale weight, so it'll feel slower than the number suggests —
        that's normal.
      </p>
    </div>

    <div class="section">
      <h2>Progressive Overload</h2>
      <p>
        The key. Same weight every week = no growth. Add weight or reps in small steps,
        every week.
      </p>
    </div>
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 3rem;
    max-width: 48rem;
    animation: fade-up var(--dur-base) var(--ease-out-quart);
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding-bottom: 2rem;
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

  .cta-row {
    margin-top: 0.5rem;
  }

  .cta-link {
    display: inline-block;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: 0.8125rem;
    font-weight: 500;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text-secondary);
    padding: 0.5rem 1rem;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    transition: color var(--dur-instant) var(--ease-out-quart), border-color var(--dur-instant) var(--ease-out-quart);
  }

  .cta-link:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 2.5rem;
  }

  .section h2 {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
    margin-bottom: 1rem;
  }

  .section p {
    font-size: 0.9375rem;
    line-height: 1.7;
    color: var(--text-secondary);
    margin-bottom: 1rem;
  }

  .section p:last-child {
    margin-bottom: 0;
  }

  .cardio {
    font-size: 0.875rem !important;
    color: var(--text-tertiary) !important;
    font-style: italic;
    margin-top: 1rem !important;
  }

  .split-table {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    overflow: hidden;
    margin: 1rem 0;
  }

  .split-row {
    display: grid;
    grid-template-columns: 5rem 6rem 1fr;
    gap: 1rem;
    padding: 0.875rem 1rem;
    align-items: baseline;
    border-bottom: 1px solid var(--border-subtle);
  }

  .split-row:last-child {
    border-bottom: none;
  }

  .split-day {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .split-focus {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .split-detail {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .exercise-list {
    display: flex;
    flex-direction: column;
    list-style: none;
    padding: 0;
    margin: 0;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .exercise-list li {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border-subtle);
  }

  .exercise-list li:last-child {
    border-bottom: none;
  }

  .ex-name {
    font-size: 0.9375rem;
    line-height: 1.5;
    color: var(--text-secondary);
  }

  .ex-scheme {
    flex-shrink: 0;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-primary);
    background: var(--surface-raised);
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

    .section h2 {
      font-size: 1.25rem;
    }

    .split-row {
      grid-template-columns: 4.5rem 1fr;
    }

    .split-detail {
      grid-column: 2;
    }
  }
</style>
