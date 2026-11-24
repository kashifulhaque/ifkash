<script lang="ts">
  /**
   * Japandi home page: an ensō that paints itself, the name beside it, a
   * grid of tatami-proportioned index tiles, and a stone garden of facts.
   * Slow seasonal shapes drift behind the page.
   */
  import { onMount } from "svelte";
  import { animate, stagger } from "animejs";
  import { magnetic, prefersReducedMotion, reveal, splitReveal } from "$lib/motion";
  import { readRelicBadge } from "../relics";
  import { facts, indexRows, resumeUrl } from "../shared";
  import { ensoCenterline, ensoShape } from "./enso";
  import Drift from "./Drift.svelte";
  import Seal from "./Seal.svelte";

  // A short Japanese caption for each index tile, keyed by name.
  const captions: Record<string, string> = {
    Work: "仕事",
    Projects: "作品",
    RL: "学習",
    Tools: "道具",
    Education: "学歴",
    Blog: "記",
    Game: "遊び",
    Roadmap: "道",
  };

  // Raked sand: horizontal sine lines with a period of 46 units. The group
  // drifts by exactly one period, so the loop has no seam.
  const PERIOD = 46;
  const rows = [8, 18, 28, 38, 48];
  function sandLine(y: number): string {
    const waves = Math.ceil(760 / (PERIOD / 2));
    return `M-60 ${y} q ${PERIOD / 4} -2.4 ${PERIOD / 2} 0` + ` t ${PERIOD / 2} 0`.repeat(waves - 1);
  }
  const sand = rows.map(sandLine);
  // Stone position and size for each fact, left to right across the tiles.
  const stones = [
    { x: 150, r: 15 },
    { x: 430, r: 11 },
    { x: 250, r: 13 },
    { x: 500, r: 16 },
  ];

  let ensoWrap: HTMLElement;
  let maskPath: SVGPathElement;
  let sealStamp: HTMLElement;
  let painted = false;

  /** Paints the ensō with the mask, then stamps the seal. */
  function paint() {
    if (prefersReducedMotion() || !maskPath) return;
    const length = maskPath.getTotalLength();
    maskPath.style.strokeDasharray = `${length}`;
    maskPath.style.strokeDashoffset = `${length}`;
    sealStamp.style.opacity = "0";
    const finish = () => {
      maskPath.style.removeProperty("stroke-dasharray");
      maskPath.style.removeProperty("stroke-dashoffset");
      sealStamp.style.removeProperty("opacity");
      sealStamp.style.removeProperty("transform");
    };
    const brush = animate(maskPath, {
      strokeDashoffset: [length, 0],
      duration: 2600,
      delay: 250,
      ease: "inOutSine",
      onComplete: () => {
        finish();
        animate(sealStamp, {
          opacity: [0, 1],
          scale: [1.35, 1],
          duration: 420,
          ease: "outBack",
          onComplete: finish,
        });
      },
    });
    // Show the finished ensō even if a throttled renderer skips the frames.
    setTimeout(() => {
      if (brush.completed) return;
      brush.cancel();
      finish();
    }, 4800);
    painted = true;
  }

  /**
   * Slides each tile in like a shoji panel and fades it up, once, when the
   * list scrolls into view.
   */
  function shoji(node: HTMLElement) {
    if (prefersReducedMotion()) return {};
    const tiles = Array.from(node.querySelectorAll<HTMLElement>("li"));
    tiles.forEach((t) => {
      t.style.clipPath = "inset(0 100% 0 0)";
      t.style.opacity = "0";
    });
    const show = () =>
      tiles.forEach((t) => {
        t.style.removeProperty("clip-path");
        t.style.removeProperty("opacity");
      });
    let fallback: ReturnType<typeof setTimeout> | undefined;
    const observer = new IntersectionObserver(
      (entries) => {
        if (!entries.some((e) => e.isIntersecting)) return;
        observer.disconnect();
        const run = animate(tiles, {
          clipPath: ["inset(0 100% 0 0)", "inset(0 0% 0 0)"],
          opacity: [0, 1],
          duration: 900,
          delay: stagger(110),
          ease: "outExpo",
          onComplete: show,
        });
        fallback = setTimeout(() => {
          if (run.completed) return;
          run.cancel();
          show();
        }, 900 + 110 * tiles.length + 600);
      },
      { rootMargin: "0px 0px -8% 0px" },
    );
    observer.observe(node);
    return {
      destroy() {
        observer.disconnect();
        clearTimeout(fallback);
      },
    };
  }

  let badge: { text: string; href: string } | null = null;
  onMount(() => {
    badge = readRelicBadge();
    paint();
  });
</script>

<Drift />

<section class="hero">
  <div class="enso" bind:this={ensoWrap}>
    <svg viewBox="0 0 400 400" aria-hidden="true" focusable="false">
      <defs>
        <filter id="enso-brush" x="-15%" y="-15%" width="130%" height="130%">
          <feTurbulence type="fractalNoise" baseFrequency="0.02" numOctaves="2" seed="7" result="warp" />
          <feDisplacementMap in="SourceGraphic" in2="warp" scale="11" xChannelSelector="R" yChannelSelector="G" result="rough" />
          <feTurbulence type="fractalNoise" baseFrequency="0.55 0.9" numOctaves="2" seed="2" result="grain" />
          <feColorMatrix in="grain" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 -3.2 3.0" result="dry" />
          <feComposite in="rough" in2="dry" operator="in" />
        </filter>
        <mask id="enso-mask" maskUnits="userSpaceOnUse" x="0" y="0" width="400" height="400">
          <path
            bind:this={maskPath}
            d={ensoCenterline}
            fill="none"
            stroke="#fff"
            stroke-width="70"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </mask>
      </defs>
      <path class="ink" d={ensoShape} filter="url(#enso-brush)" mask="url(#enso-mask)" />
      <g class="flecks">
        <circle cx="58" cy="118" r="1.8" />
        <circle cx="46" cy="136" r="1.1" />
        <circle cx="342" cy="316" r="1.4" />
      </g>
    </svg>
    <div class="stamp" bind:this={sealStamp}>
      <Seal size={46} />
    </div>
  </div>

  <div class="hero-text">
    <p class="eyebrow">ML engineer · ifkash.dev</p>
    <h1 class="display">
      <span use:splitReveal={{ delay: 200 }}>Kashiful Haque</span><a
        href="/fitness"
        class="period"
        aria-label="Fitness dashboard">。</a
      >
    </h1>
    <p class="hero-copy" use:reveal={{ delay: 500, y: 10, duration: 1200 }}>
      I build <em>pre-training and post-training</em> systems for large language models,
      reinforcement learning pipelines, and high-performance inference software in C++ and Rust.
    </p>
    <div class="hero-actions" use:reveal={{ delay: 800, y: 10, duration: 1200 }}>
      <a href={resumeUrl} class="btn btn-primary" use:magnetic>Read résumé</a>
      <a href="/game" class="btn" use:magnetic>Play Cinderheart</a>
      {#if badge}
        <a href={badge.href} class="tag relic" title="Relics from the portfolio you have found in Cinderheart">
          <span aria-hidden="true">✦</span>
          {badge.text}
        </a>
      {/if}
    </div>
  </div>

  <p class="vert" lang="ja" aria-hidden="true">言語モデルと強化学習</p>
</section>

<section class="index" aria-labelledby="index-title">
  <header class="section-head">
    <h2 id="index-title">Index</h2>
    <span class="jp" lang="ja" aria-hidden="true">目次</span>
  </header>
  <ul class="tiles" use:shoji>
    {#each indexRows as row}
      <li>
        <a href={row.href} class="tile">
          <span class="n">{row.n}</span>
          <span class="kan" lang="ja" aria-hidden="true">{captions[row.name] ?? ""}</span>
          <span class="nm">{row.name}</span>
          <span class="ds">{row.desc}</span>
        </a>
      </li>
    {/each}
  </ul>
</section>

<section class="garden-wrap" aria-labelledby="garden-title">
  <header class="section-head">
    <h2 id="garden-title">At a glance</h2>
    <span class="jp" lang="ja" aria-hidden="true">庭</span>
  </header>
  <dl class="garden" use:reveal={{ children: ".fact", gap: 140, y: 12, duration: 1100 }}>
    {#each facts as f, i}
      <div class="fact">
        <dt>{f.k}</dt>
        <dd>{f.v}</dd>
        <svg class="sand" viewBox="0 0 600 56" preserveAspectRatio="xMidYMid slice" aria-hidden="true" focusable="false">
          <g class="rake">
            {#each sand as d}
              <path {d} />
            {/each}
          </g>
          <g class="stone" transform={`translate(${stones[i % stones.length].x} 28)`}>
            <circle class="ring" r={stones[i % stones.length].r + 14} />
            <circle class="ring" r={stones[i % stones.length].r + 7} />
            <ellipse class="rock" rx={stones[i % stones.length].r} ry={stones[i % stones.length].r * 0.78} />
            <ellipse class="sheen" cx={-stones[i % stones.length].r * 0.28} cy={-stones[i % stones.length].r * 0.3} rx={stones[i % stones.length].r * 0.35} ry={stones[i % stones.length].r * 0.18} />
          </g>
        </svg>
      </div>
    {/each}
  </dl>
</section>

<style>
  /* ── Hero ─────────────────────────────────────────────── */

  .hero {
    position: relative;
    display: grid;
    grid-template-columns: minmax(200px, 360px) minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--s-7) var(--s-8);
    padding: clamp(40px, 8vw, 104px) 0 var(--s-9);
  }

  .enso {
    position: relative;
    width: 100%;
    aspect-ratio: 1;
    color: var(--ink);
  }
  .enso svg {
    width: 100%;
    height: 100%;
    overflow: visible;
  }
  .enso .ink {
    fill: currentColor;
    opacity: 0.92;
  }
  .enso .flecks circle {
    fill: currentColor;
    opacity: 0.45;
  }
  .stamp {
    position: absolute;
    right: 4%;
    bottom: 8%;
  }

  .hero-text .eyebrow {
    margin-bottom: var(--s-5);
    letter-spacing: 0.14em;
  }

  .display {
    font-size: clamp(2.5rem, 1.2rem + 4.6vw, 4.75rem);
    line-height: 1.12;
    letter-spacing: 0.02em;
    font-weight: 500;
  }
  .period {
    color: var(--falu);
    font-weight: 400;
  }
  .period:hover {
    color: var(--falu);
    opacity: 0.7;
  }

  .hero-copy {
    max-width: 34ch;
    margin: var(--s-7) 0 0;
    color: var(--ink);
    font-size: clamp(1.0625rem, 0.95rem + 0.5vw, 1.25rem);
    line-height: 2;
    letter-spacing: 0.02em;
  }
  .hero-copy em {
    font-style: normal;
    font-family: var(--font-display);
    background: linear-gradient(var(--falu), var(--falu)) no-repeat 0 96% / 100% 1px;
  }

  .hero-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-3) var(--s-5);
    margin-top: var(--s-7);
  }

  .relic {
    height: 32px;
    text-decoration: none;
  }
  .relic span {
    color: var(--clay);
  }

  .vert {
    align-self: start;
    margin: var(--s-5) 0 0;
    writing-mode: vertical-rl;
    color: var(--ink-3);
    font-family: var(--font-display);
    font-size: 15px;
    letter-spacing: 0.5em;
  }

  /* ── Section heads ────────────────────────────────────── */

  .section-head {
    display: flex;
    align-items: baseline;
    gap: var(--s-4);
    margin-bottom: var(--s-6);
    padding-bottom: var(--s-3);
    border-bottom: 1px solid var(--line);
  }
  .section-head h2 {
    font-size: var(--text-lg);
    font-weight: 500;
    letter-spacing: 0.08em;
  }
  .section-head .jp {
    color: var(--ink-3);
    font-family: var(--font-display);
    font-size: 15px;
    letter-spacing: 0.3em;
  }

  /* ── Index tiles ──────────────────────────────────────── */

  .tiles {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--s-5);
    padding-bottom: var(--s-7);
    list-style: none;
  }
  /* Even tiles sit lower, like the brick pattern of tatami mats. */
  .tiles li:nth-child(even) {
    margin-top: var(--s-6);
  }
  .tiles li:nth-child(odd):last-child {
    margin-bottom: 0;
  }

  .tile {
    position: relative;
    display: grid;
    grid-template-rows: auto 1fr auto auto;
    aspect-ratio: 2 / 1;
    padding: var(--s-4) var(--s-5);
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--line);
    color: var(--ink);
    transition:
      transform var(--dur) var(--ease-out),
      box-shadow var(--dur) var(--ease-out);
  }
  /* Light wood-grain edges along the top and bottom of each tile. */
  .tile::before,
  .tile::after {
    content: "";
    position: absolute;
    right: 0;
    left: 0;
    height: 3px;
    background: repeating-linear-gradient(
      90deg,
      var(--line-strong) 0 11px,
      var(--line) 11px 17px,
      color-mix(in srgb, var(--line-strong) 70%, var(--line)) 17px 31px,
      var(--line) 31px 36px
    );
    opacity: 0.75;
  }
  .tile::before {
    top: 0;
  }
  .tile::after {
    bottom: 0;
  }
  .tile:hover {
    color: var(--ink);
    transform: translateY(-4px);
    box-shadow:
      inset 0 0 0 1px var(--line-strong),
      0 14px 22px -16px var(--shadow-color);
  }

  .tile .n {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
  }
  .tile .kan {
    position: absolute;
    top: var(--s-4);
    right: var(--s-5);
    writing-mode: vertical-rl;
    color: var(--ink-3);
    font-family: var(--font-display);
    font-size: 22px;
    letter-spacing: 0.2em;
    opacity: 0.8;
    transition: color var(--dur) var(--ease), opacity var(--dur) var(--ease);
  }
  .tile:hover .kan {
    color: var(--falu);
    opacity: 1;
  }
  .tile .nm {
    align-self: end;
    font-family: var(--font-display);
    font-size: 1.5rem;
    font-weight: 500;
    letter-spacing: 0.06em;
    line-height: 1.2;
  }
  .tile .ds {
    margin-top: var(--s-1);
    color: var(--ink-2);
    font-size: 13px;
    line-height: 1.5;
    padding-right: 3em;
  }

  /* ── Stone garden ─────────────────────────────────────── */

  .garden {
    margin: 0;
    padding-bottom: var(--s-6);
  }
  .fact {
    position: relative;
    display: grid;
    grid-template-columns: minmax(8.5rem, 11rem) minmax(0, 1fr);
    grid-template-areas:
      "k sand"
      "v sand";
    align-items: center;
    column-gap: var(--s-5);
    padding: var(--s-3) 0;
  }
  .fact + .fact {
    margin-top: var(--s-2);
  }
  .fact dt {
    grid-area: k;
    align-self: end;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.06em;
  }
  .fact dd {
    grid-area: v;
    align-self: start;
    margin: 2px 0 0;
    color: var(--ink);
    font-family: var(--font-display);
    font-size: 1.125rem;
    letter-spacing: 0.04em;
  }
  .sand {
    grid-area: sand;
    width: 100%;
    height: 56px;
    overflow: hidden;
    -webkit-mask-image: linear-gradient(90deg, transparent, #000 10%, #000 90%, transparent);
    mask-image: linear-gradient(90deg, transparent, #000 10%, #000 90%, transparent);
  }
  .rake path {
    fill: none;
    stroke: var(--line-strong);
    stroke-width: 1;
    stroke-linecap: round;
    opacity: 0.7;
  }
  .rake {
    animation: rake-drift 14s linear infinite;
  }
  @keyframes rake-drift {
    to {
      transform: translateX(-46px);
    }
  }
  .stone .ring {
    fill: var(--bg);
    stroke: var(--line-strong);
    stroke-width: 1;
    opacity: 0.95;
  }
  .stone .ring:nth-child(2) {
    fill: none;
  }
  .stone .rock {
    fill: var(--ink-3);
  }
  .stone .sheen {
    fill: var(--bg);
    opacity: 0.35;
  }

  /* ── Narrow screens ───────────────────────────────────── */

  @media (max-width: 1100px) {
    .hero {
      grid-template-columns: minmax(180px, 300px) minmax(0, 1fr);
    }
    .vert {
      display: none;
    }
  }
  @media (max-width: 720px) {
    .hero {
      grid-template-columns: 1fr;
      gap: var(--s-6);
      padding-top: var(--s-6);
    }
    .enso {
      width: min(260px, 70vw);
    }
    .tiles {
      grid-template-columns: 1fr;
      gap: var(--s-4);
    }
    .tiles li:nth-child(even) {
      margin-top: 0;
    }
    .fact {
      grid-template-columns: 1fr;
      grid-template-areas: "k" "v" "sand";
      row-gap: var(--s-1);
    }
  }
</style>
