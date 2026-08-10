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
  <p class="eyebrow hero-eyebrow">ML Engineer · Bangalore, IN</p>

  <h1 class="display hero-title power-on">
    <span class="block">
      <em class="headline-italic accent-em">Kashiful</em>
    </span>
    <span class="block">
      Haque<a href="/fitness" class="dot" aria-label="." rel="nofollow">.</a>
    </span>
  </h1>

  <p class="lead hero-tagline">
    ML Engineer with 4 YOE pre-training and post-training LLMs with RL
    pipelines, and building high-performance inference systems in C++ and
    Rust. Also building LLM apps on the day job.
  </p>

  <div class="btn-row quick-actions">
    <a href={resumeUrl} class="btn btn-primary">Read Resume</a>
    <a href="/game" class="btn btn-gradient-ring">
      <span class="btn-gradient-text">Enter Game Mode</span>
    </a>
  </div>

  <div class="status-row">
    <span class="label label--accent">Online</span>
    <span class="label">Exp · 4+ yrs</span>
    <span class="label">Stack · C++ / Rust / Py</span>
    <span class="label">Focus · LLM systems</span>
  </div>
</section>

<div class="ticker" aria-hidden="true">
  <div class="ticker-track">
    {#each [...tickerItems, ...tickerItems] as item}
      <span class="ticker-item">{item}</span>
      <span class="ticker-sep" aria-hidden="true">▪</span>
    {/each}
  </div>
</div>

<section class="index-section">
  <p class="eyebrow index-label">Site index</p>
  <div class="index-list">
    {#each indexRows as row, i}
      <a href={row.href} class="index-row stagger" style="--i: {i}">
        <span class="index-n label label--accent">{row.n}</span>
        <span class="index-name">{row.name}</span>
        <span class="index-desc label">{row.desc}</span>
        <span class="index-arrow" aria-hidden="true">→</span>
      </a>
    {/each}
  </div>
</section>

<style>
  .hero {
    min-height: 70vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: var(--spacing-hero-top, 48px) 0 40px;
  }

  .hero-eyebrow {
    margin-bottom: 24px;
  }

  .hero-title {
    margin: 0;
  }

  /* Trailing period → hidden fitness hub link */
  .dot {
    color: var(--accent);
    border-bottom: none;
    cursor: var(--cursor-pointer);
    text-shadow: var(--accent-text-glow);
    transition: color 0.15s var(--ease-inout);
    font-style: normal;
  }

  .dot:hover {
    color: var(--accent-2);
    border-bottom: none;
  }

  .hero-tagline {
    max-width: 36em;
    margin: 28px 0 0;
  }

  .quick-actions {
    margin-top: 36px;
  }

  .status-row {
    display: flex;
    flex-wrap: wrap;
    gap: 16px 24px;
    margin-top: 32px;
    padding-top: 24px;
    border-top: 1px solid var(--border);
  }

  .ticker {
    margin: 8px 0 0;
    border-top: 1px solid var(--border);
  }

  .ticker-sep {
    color: var(--muted-foreground);
    opacity: 0.5;
    font-size: 8px;
    align-self: center;
  }

  .index-section {
    padding: var(--section-pt) 0 24px;
  }

  .index-label {
    margin-bottom: 8px;
  }

  .index-list {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--border);
  }

  .index-row {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr) auto;
    align-items: baseline;
    gap: 16px;
    padding-block: 20px;
    border-bottom: 1px solid var(--border);
    color: var(--foreground);
    transition: color 0.15s var(--ease-inout);
  }

  .index-row:hover .index-name {
    color: var(--accent-2);
  }

  .index-row:hover .index-arrow {
    color: var(--accent);
    transform: translateX(2px);
  }

  .index-n {
    align-self: baseline;
  }

  .index-name {
    font-family: var(--font-serif);
    font-size: 24px;
    line-height: 1.2;
    letter-spacing: -0.025em;
    color: var(--foreground);
  }

  .index-desc {
    display: none;
    color: var(--muted-foreground);
  }

  .index-arrow {
    color: var(--muted-foreground);
    transition:
      transform 0.15s var(--ease-inout),
      color 0.15s var(--ease-inout);
  }

  @media (min-width: 1024px) {
    .index-row {
      grid-template-columns: 80px minmax(0, 1fr) auto auto;
      gap: 24px;
    }

    .index-desc {
      display: inline;
      justify-self: end;
    }
  }

  @media (max-width: 900px) {
    .hero {
      min-height: 0;
      padding: 32px 0 28px;
    }
  }
</style>
