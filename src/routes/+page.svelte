<script>
  import { dev } from "$app/environment";

  // Public, unauthenticated resume PDF. Served from `/api/cv` (not `/api/resume`,
  // which is behind Cloudflare Access for the editor) so no login is required.
  const resumeUrl = dev
    ? "http://localhost:8787/api/cv?format=view"
    : "/api/cv?format=view";

  const tickerItems = [
    "LLM pre-training",
    "RL pipelines",
    "inference systems",
    "C++",
    "Rust",
    "Python",
    "Typst",
    "Svelte",
    "Cloudflare Workers",
    "WASM",
  ];

  const indexRows = [
    { n: "01", name: "Work", desc: "Roles and companies", href: "/work" },
    {
      n: "02",
      name: "Projects",
      desc: "Side projects and open source",
      href: "/projects",
    },
    {
      n: "03",
      name: "Tools",
      desc: "Small in-browser utilities",
      href: "/tools",
    },
    {
      n: "04",
      name: "Education",
      desc: "Academic background",
      href: "/education",
    },
    {
      n: "05",
      name: "Blog",
      desc: "Writing on blog.ifkash.dev",
      href: "/blog",
    },
    { n: "06", name: "Game", desc: "A browser FPS", href: "/game" },
  ];
</script>

<svelte:head>
  <title>Kashiful Haque</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, ML Engineer."
  />
</svelte:head>

<section class="hero">
  <p class="hero-eyebrow">
    <span class="led" aria-hidden="true"></span>
    ML Engineer · Bangalore, IN
  </p>

  <h1 class="hero-title power-on">
    Kashiful<br />Haque<a href="/fitness" class="dot" aria-label="." rel="nofollow"
      >.</a
    >
  </h1>

  <p class="hero-tagline">
    ML Engineer with 4 YOE pre-training and post-training LLMs with RL
    pipelines, and building high-performance inference systems in C++ and
    Rust. Also building LLM apps on the day job.
  </p>

  <div class="quick-actions">
    <a href={resumeUrl} class="btn btn-primary">Read Resume &nearr;</a>
    <a href="/game" class="btn">Enter Game Mode &nearr;</a>
  </div>

  <div class="status-row">
    <span class="status-chip"><span class="led" aria-hidden="true"></span>Online</span>
    <span class="status-chip">EXP · 4+ YRS</span>
    <span class="status-chip">Stack · C++ / Rust / Py</span>
    <span class="status-chip">Focus · LLM systems</span>
  </div>
</section>

<div class="ticker" aria-hidden="true">
  <div class="ticker-track">
    {#each [...tickerItems, ...tickerItems] as item}
      <span class="ticker-item">{item}</span>
      <span class="ticker-dot"></span>
    {/each}
  </div>
</div>

<section class="index-section">
  <p class="label index-label">Site index</p>
  <div class="index-list">
    {#each indexRows as row, i}
      <a href={row.href} class="index-row stagger" style="--i: {i}">
        <span class="index-n">{row.n}</span>
        <span class="index-name">{row.name}</span>
        <span class="index-desc">{row.desc}</span>
        <span class="index-arrow" aria-hidden="true">&rarr;</span>
      </a>
    {/each}
  </div>
</section>

<style>
  /* ─── Hero ─────────────────────────────────────────────────── */

  .hero {
    min-height: 72vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 48px 0 40px;
  }

  .hero-eyebrow {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-family: var(--font-mono-g);
    font-size: 0.75rem;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--ink-soft);
    margin-bottom: 24px;
  }

  .hero-title {
    font-family: var(--font-dots);
    font-size: clamp(4rem, 13vw, 10.5rem);
    font-weight: 600;
    line-height: 0.88;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--ink);
  }

  /* The trailing period is a deliberately unmarked link to the fitness hub —
     looks like plain punctuation, but it's clickable. */
  .dot {
    color: var(--signal);
    border-bottom: none;
    cursor: pointer;
    text-shadow: 0 0 18px var(--signal-glow);
    transition: color 0.15s;
  }

  .dot:hover {
    color: var(--signal-hi);
    border-bottom: none;
  }

  .hero-tagline {
    max-width: 640px;
    margin: 32px 0 0;
    font-family: var(--font-sans-g);
    font-size: clamp(1rem, 1.6vw, 1.15rem);
    line-height: 1.6;
    color: var(--ink-soft);
  }

  /* ─── Quick actions ────────────────────────────────────────── */

  .quick-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-top: 36px;
  }

  /* ─── Status row ───────────────────────────────────────────── */

  .status-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 28px;
  }

  .status-chip {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 7px 12px;
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono-g);
    font-size: 0.66rem;
    font-weight: 500;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--ink-soft);
  }

  /* ─── Ticker ───────────────────────────────────────────────── */

  .ticker {
    overflow: hidden;
    border-top: 1px solid var(--line-soft);
    border-bottom: 1px solid var(--line-soft);
    padding: 12px 0;
    margin: 8px 0 0;
    mask-image: linear-gradient(
      to right,
      transparent,
      black 8%,
      black 92%,
      transparent
    );
    -webkit-mask-image: linear-gradient(
      to right,
      transparent,
      black 8%,
      black 92%,
      transparent
    );
  }

  .ticker-track {
    display: inline-flex;
    align-items: center;
    gap: 28px;
    white-space: nowrap;
    animation: ticker-scroll 36s linear infinite;
    will-change: transform;
  }

  .ticker-item {
    font-family: var(--font-mono-g);
    font-size: 0.72rem;
    font-weight: 500;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--ink-soft);
  }

  .ticker-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--signal);
    box-shadow: 0 0 4px var(--signal-glow);
    flex-shrink: 0;
  }

  /* ─── Site index ───────────────────────────────────────────── */

  .index-section {
    padding: 72px 0 24px;
  }

  .index-label {
    margin-bottom: 20px;
    color: var(--ink-mute);
  }

  .index-list {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--line-soft);
  }

  .index-row {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr) auto 24px;
    align-items: baseline;
    gap: 20px;
    padding: 18px 12px;
    margin: 0 -12px;
    border-bottom: 1px solid var(--line-soft);
    color: var(--ink);
    transition:
      background 0.15s,
      color 0.15s;
  }

  .index-row:hover {
    background: var(--ink);
    color: var(--void);
    border-bottom-color: var(--ink);
  }

  .index-n {
    font-family: var(--font-mono-g);
    font-size: 0.72rem;
    font-weight: 500;
    letter-spacing: 0.12em;
    color: var(--signal);
  }

  .index-row:hover .index-n {
    color: var(--void);
  }

  .index-name {
    font-family: var(--font-dots);
    font-size: clamp(1.7rem, 4vw, 2.6rem);
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    line-height: 1;
  }

  .index-desc {
    font-family: var(--font-mono-g);
    font-size: 0.68rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-mute);
    white-space: nowrap;
  }

  .index-row:hover .index-desc {
    color: var(--void);
  }

  .index-arrow {
    font-family: var(--font-mono-g);
    font-size: 1rem;
    color: var(--ink-mute);
    transition:
      transform 0.15s,
      color 0.15s;
  }

  .index-row:hover .index-arrow {
    color: var(--void);
    transform: translateX(4px);
  }

  /* ─── Responsive ───────────────────────────────────────────── */

  @media (max-width: 900px) {
    .hero {
      min-height: 0;
      padding: 32px 0 28px;
    }

    .hero-tagline {
      margin: 24px 0 0;
    }
  }

  @media (max-width: 640px) {
    .index-row {
      grid-template-columns: 36px minmax(0, 1fr) 20px;
    }

    .index-desc {
      display: none;
    }
  }

  @media (max-width: 480px) {
    .hero-title {
      font-size: clamp(3.2rem, 17vw, 4.6rem);
      line-height: 0.9;
    }

    .quick-actions :global(.btn) {
      width: 100%;
      max-width: none;
    }
  }
</style>
