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
  <a href="/game" class="btn btn-primary">Enter Game Mode &nearr;</a>
</section>

<style>
  /* ─── Masthead ─────────────────────────────────────────────── */
  .manual-masthead {
    padding: 64px 0 24px;
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

    .quick-actions :global(.btn) {
      width: 100%;
      max-width: none;
    }
  }
</style>
