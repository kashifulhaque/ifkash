<script>
  import { dev } from "$app/environment";

  const resumeUrl = dev
    ? "http://localhost:8787/api/resume?format=view"
    : "/api/resume?format=view";
</script>

<svelte:head>
  <title>Kashiful Haque</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, ML Engineer."
  />
</svelte:head>

<section class="manual-masthead">
  <h1 class="manual-title">Kashiful Haque.</h1>
  <p class="manual-tagline">
    ML Engineer with 4 YOE pre-training and post-training LLMs with RL pipelines,
    and building high-performance inference systems in C++ and Rust.
    <br />
    Also building LLM apps on the day job.
  </p>
</section>

<section class="quick-actions">
  <a href={resumeUrl} class="btn">Read Resume &nearr;</a>
</section>

<style>
  /* ─── Masthead ─────────────────────────────────────────────── */

  .manual-masthead {
    padding: 64px 0 24px;
  }

  .manual-meta-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 16px;
    flex-wrap: wrap;
    margin-bottom: 32px;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--ink-mute);
  }

  .manual-meta-row .right {
    color: var(--blueprint);
  }

  .manual-title {
    display: block;
    font-family: var(--font-display);
    font-size: clamp(3.2rem, 11vw, 8rem);
    line-height: 0.86;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--blueprint);
  }

  .manual-tagline {
    max-width: 720px;
    margin: 28px 0 8px;
    font-family: var(--font-body);
    font-size: clamp(1.05rem, 1.7vw, 1.25rem);
    line-height: 1.55;
    color: var(--ink);
  }

  .manual-attribution {
    font-family: var(--font-body);
    font-size: 0.95rem;
    color: var(--ink-soft);
    font-style: italic;
  }

  /* ─── Preface ──────────────────────────────────────────────── */

  .preface {
    padding: 56px 0;
  }

  .preface-grid {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 48px;
  }

  .preface-eyebrow {
    font-family: var(--font-mono);
    font-size: 0.74rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--blueprint);
  }

  .preface-body {
    font-family: var(--font-body);
    font-size: 1rem;
    line-height: 1.7;
    text-align: justify;
    hyphens: auto;
    max-width: 760px;
  }

  .preface-body p {
    margin-bottom: 1em;
  }

  .preface-body p:first-of-type::first-letter {
    font-family: var(--font-display);
    float: left;
    font-size: 4.4rem;
    line-height: 0.85;
    padding: 0.06em 0.14em 0 0;
    color: var(--blueprint);
  }

  /* ─── Now ──────────────────────────────────────────────────── */

  .now {
    padding: 32px 0 56px;
  }

  .now-title {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--blueprint);
    margin-bottom: 24px;
  }

  .now-rows {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--rule-soft);
  }

  .now-row {
    display: grid;
    grid-template-columns: 140px 1fr auto;
    align-items: baseline;
    gap: 16px;
    padding: 14px 0;
    border-bottom: 1px solid var(--rule-soft);
    font-family: var(--font-mono);
    font-size: 0.92rem;
  }

  .now-label {
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-soft);
    font-size: 0.78rem;
  }

  .now-leader {
    border-bottom: 1px dotted var(--rule-soft);
    transform: translateY(-4px);
  }

  .now-value {
    color: var(--ink);
    text-align: right;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .now-value a {
    color: var(--ink);
    border-bottom: 1px solid var(--rule-soft);
  }

  .now-value a:hover {
    color: var(--blueprint);
    border-bottom-color: var(--blueprint);
  }

  /* ─── Quick actions ────────────────────────────────────────── */

  .quick-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    padding-top: 16px;
  }

  /* ─── Responsive ───────────────────────────────────────────── */

  @media (max-width: 900px) {
    .manual-masthead {
      padding: 32px 0 12px;
    }

    .preface {
      padding: 32px 0;
    }

    .preface-grid {
      grid-template-columns: 1fr;
      gap: 12px;
    }

    .preface-body {
      font-size: 0.98rem;
    }

    .preface-body p:first-of-type::first-letter {
      font-size: 3.2rem;
    }

    .manual-tagline {
      margin: 20px 0 6px;
      font-size: 1rem;
    }
  }

  @media (max-width: 480px) {
    .manual-title {
      font-size: clamp(2.4rem, 14vw, 3.6rem);
      line-height: 0.92;
    }

    .manual-meta-row {
      font-size: 0.6rem;
      letter-spacing: 0.12em;
      margin-bottom: 16px;
    }

    .now-row {
      grid-template-columns: 1fr;
      gap: 4px;
    }

    .now-leader {
      display: none;
    }

    .now-value {
      text-align: left;
      white-space: normal;
    }

    .quick-actions :global(.btn) {
      width: 100%;
      max-width: none;
    }
  }
</style>
