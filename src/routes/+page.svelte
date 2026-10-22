<script>
  import { onMount } from "svelte";
  import { dev } from "$app/environment";
  import EdgeField from "$lib/components/EdgeField.svelte";
  import Horizon from "$lib/components/Horizon.svelte";
  import { loadProgress, WONDER_COUNT } from "$lib/game/progress";
  import { seedLink } from "$lib/game/seed";

  // Progress from the tiny-planet game lives in localStorage, so it is read
  // after mount; the server render never shows the badge.
  /** @type {{ text: string, href: string } | null} */
  let badge = null;
  onMount(() => {
    const progress = loadProgress();
    if (!progress.complete) return;
    badge = {
      text: `${progress.found.length} of ${WONDER_COUNT} small wonders`,
      href: progress.seed ? seedLink(progress.seed, "", progress.depth) : "/game",
    };
  });

  const resumeUrl = dev
    ? "http://localhost:8787/api/cv?format=view"
    : "/api/cv?format=view";

  const indexRows = [
    { n: "01", name: "Work", desc: "Roles and companies", href: "/work" },
    { n: "02", name: "Projects", desc: "Open source and independent work", href: "/projects" },
    { n: "03", name: "RL", desc: "Reinforcement learning experiments", href: "/rl" },
    { n: "04", name: "Tools", desc: "Useful in-browser utilities", href: "/tools" },
    { n: "05", name: "Education", desc: "Academic background", href: "/education" },
    { n: "06", name: "Blog", desc: "Notes on engineering and ML", href: "/blog" },
    { n: "07", name: "Game", desc: "A tiny planet to wander", href: "/game" },
    { n: "08", name: "Roadmap", desc: "Interview course anchored to shipped work", href: "/roadmap" },
  ];

  const facts = [
    { k: "Focus", v: "LLM systems" },
    { k: "Experience", v: "4+ years" },
    { k: "Core stack", v: "C++, Rust, Python" },
    { k: "Based in", v: "Bengaluru, India" },
  ];
</script>

<svelte:head>
  <title>Kashiful Haque</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, an ML engineer in Bangalore."
  />
</svelte:head>

<section class="hero">
  <p class="eyebrow hero-eyebrow">ML engineer — ifkash.dev</p>

  <h1 class="display">
    Kashiful Haque<a href="/fitness" class="period" aria-label="Fitness dashboard">.</a>
  </h1>

  <div class="hero-body">
    <p class="hero-copy">
      I build <em class="serif">pre-training and post-training</em> systems for
      large language models, reinforcement learning pipelines, and
      high-performance inference software in C++ and Rust.
    </p>

    <div class="hero-actions">
      <a href={resumeUrl} class="btn btn-primary">Read résumé</a>
      <a href="/game" class="btn">Visit the tiny planet</a>
      {#if badge}
        <a href={badge.href} class="tag wonder" title="Every small wonder on the tiny planet has been found">
          <span aria-hidden="true">✦</span>
          {badge.text}
        </a>
      {/if}
    </div>
  </div>
</section>

<Horizon />

<dl class="facts">
  {#each facts as f}
    <div>
      <dt>{f.k}</dt>
      <dd>{f.v}</dd>
    </div>
  {/each}
</dl>

<section class="index" aria-labelledby="index-heading">
  <header class="index-head">
    <h2 id="index-heading">Index</h2>
    <p>Selected work, experiments, and tools.</p>
  </header>

  <ol class="rows">
    {#each indexRows as row, i}
      <li class="stagger" style="--i: {i}">
        <a href={row.href} class="index-row">
          <span class="n">{row.n}</span>
          <span class="name">{row.name}</span>
          <span class="desc">{row.desc}</span>
          <span class="arrow" aria-hidden="true">→</span>
        </a>
      </li>
    {/each}
  </ol>
</section>

<EdgeField />

<style>
  /* ── Hero ─────────────────────────────────────────────── */

  .hero {
    padding: clamp(56px, 9vw, 120px) 0 var(--s-7);
  }

  .hero-eyebrow {
    margin-bottom: var(--s-5);
  }

  .display {
    font-size: clamp(3rem, 1.2rem + 7.6vw, 7.5rem);
    line-height: 0.95;
    letter-spacing: -0.055em;
  }

  :global(html[data-design="broadsheet"]) .display {
    line-height: 0.96;
    letter-spacing: -0.03em;
    font-variation-settings: "opsz" 72;
  }

  :global(html[data-design="atelier"]) .display {
    line-height: 1;
    letter-spacing: -0.034em;
    font-variation-settings: "SOFT" 45, "WONK" 1, "opsz" 144, "wght" 560;
  }

  .period {
    color: var(--falu);
    transition: opacity var(--dur-fast) var(--ease);
  }
  .period:hover {
    color: var(--falu);
    opacity: 0.7;
  }

  .hero-body {
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
    align-items: end;
    gap: var(--s-6) var(--s-8);
    margin-top: var(--s-7);
  }

  .hero-copy {
    max-width: 34ch;
    margin: 0;
    color: var(--ink);
    font-size: clamp(1.25rem, 1rem + 0.9vw, 1.625rem);
    line-height: 1.35;
    letter-spacing: -0.018em;
  }

  .hero-copy .serif {
    font-size: 1.06em;
  }

  .hero-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    align-items: center;
    gap: var(--s-2);
  }

  .wonder {
    height: 40px;
    padding: 0 var(--s-4);
    font-size: var(--text-xs);
    text-decoration: none;
  }
  .wonder span {
    color: var(--clay);
  }
  .wonder:hover {
    border-color: var(--line-strong);
    color: var(--ink);
  }

  /* ── Facts ────────────────────────────────────────────── */

  .facts {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    margin: 0;
    border-bottom: 1px solid var(--line);
  }

  .facts div {
    padding: var(--s-5) var(--s-5) var(--s-5) 0;
  }
  .facts div + div {
    padding-left: var(--s-5);
    border-left: 1px solid var(--line);
  }

  .facts dt {
    margin-bottom: var(--s-1);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .facts dd {
    margin: 0;
    color: var(--ink);
    font-size: 15px;
  }

  /* ── Index ────────────────────────────────────────────── */

  .index {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 3fr);
    gap: var(--s-6) var(--s-8);
    padding: var(--s-9) 0;
  }

  .index-head h2 {
    font-size: var(--text-xl);
  }

  .index-head p {
    max-width: 22ch;
    margin: var(--s-2) 0 0;
    color: var(--ink-3);
    font-size: 15px;
  }

  .index-row {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr) minmax(0, 1.2fr) 24px;
    align-items: baseline;
    gap: var(--s-5);
    padding: var(--s-5) 0;
    color: var(--ink);
    text-decoration: none;
  }

  .n,
  .desc {
    color: var(--ink-3);
    font-size: var(--text-sm);
  }

  .n {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .name {
    font-size: var(--text-lg);
    font-weight: 500;
    letter-spacing: -0.02em;
    transition: color var(--dur-fast) var(--ease), transform var(--dur) var(--ease-out);
  }

  :global(html[data-design="broadsheet"]) .index-row .name {
    font-family: var(--font-display);
    font-weight: 450;
  }

  :global(html[data-design="atelier"]) .index-row .name {
    font-family: var(--font-display);
    font-weight: 520;
    letter-spacing: -0.02em;
    font-variation-settings: "SOFT" 30, "WONK" 1, "opsz" 32, "wght" 520;
  }

  .arrow {
    justify-self: end;
    color: var(--ink-3);
    transition: transform var(--dur) var(--ease-out), color var(--dur-fast) var(--ease);
  }

  .index-row:hover .name {
    color: var(--fjord);
    transform: translateX(4px);
  }
  .index-row:hover .arrow {
    color: var(--fjord);
    transform: translateX(3px);
  }

  @media (max-width: 900px) {
    .hero-body {
      grid-template-columns: 1fr;
    }
    .hero-actions {
      justify-content: flex-start;
    }
    .facts {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .facts div:nth-child(3) {
      padding-left: 0;
      border-left: 0;
    }
    .facts div:nth-child(n + 3) {
      border-top: 1px solid var(--line);
    }
    .index {
      grid-template-columns: 1fr;
      padding: var(--s-8) 0;
    }
  }

  @media (max-width: 560px) {
    .index-row {
      grid-template-columns: 32px minmax(0, 1fr) 20px;
      gap: var(--s-3);
    }
    .desc {
      display: none;
    }
  }
</style>
