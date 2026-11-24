<script>
  /**
   * Almanac home page: a title page with an engraved plate, the opening
   * paragraph with a drop cap, a table of contents with dotted leaders, and a
   * data table of particulars.
   */
  import { onMount } from "svelte";
  import { reveal, magnetic, prefersReducedMotion, splitReveal } from "$lib/motion";
  import { animate, stagger } from "animejs";
  import { readRelicBadge } from "../relics";
  import { facts, indexRows, navItems, resumeUrl } from "../shared";
  import Plate from "./Plate.svelte";

  // Page numbers follow the contents column; pages outside it continue the count.
  let extra = navItems.length;
  const toc = indexRows
    .map((row) => {
      const i = navItems.findIndex((n) => n.href === row.href);
      return { ...row, folio: String(i >= 0 ? i + 1 : ++extra).padStart(2, "0"), at: i >= 0 ? i : extra };
    })
    .sort((a, b) => a.at - b.at);

  /** @type {{ text: string, href: string } | null} */
  let badge = null;
  /** @type {HTMLElement | undefined} */
  let title;

  onMount(() => {
    badge = readRelicBadge();
    if (!title || prefersReducedMotion()) return;
    const targets = title.querySelectorAll("[data-in]");
    targets.forEach((el) => /** @type {HTMLElement} */ (el).style.setProperty("opacity", "0"));
    const a = animate(targets, {
      opacity: [0, 1],
      translateY: [12, 0],
      duration: 800,
      delay: stagger(160, { start: 500 }),
      ease: "outExpo",
    });
    const show = () => targets.forEach((el) => {
      /** @type {HTMLElement} */ (el).style.removeProperty("opacity");
      /** @type {HTMLElement} */ (el).style.removeProperty("transform");
    });
    const t = setTimeout(() => { a.cancel(); show(); }, 3000);
    return () => { clearTimeout(t); a.cancel(); show(); };
  });
</script>

<article class="titlepage" bind:this={title}>
  <div class="plate-wrap">
    <Plate />
  </div>

  <p class="eyebrow" data-in>ML engineer · ifkash.dev</p>

  <h1 class="display">
    <span use:splitReveal={{ delay: 200 }}>Kashiful Haque</span><a href="/fitness" class="period" aria-label="Fitness dashboard">.</a>
  </h1>
  <p class="subtitle" data-in><em class="serif">A set of notes on language models, learning, and fast inference.</em></p>

  <p class="opening" data-in>
    I build <em class="serif">pre-training and post-training</em> systems for
    large language models, reinforcement learning pipelines, and
    high-performance inference software in C++ and Rust.
  </p>

  <div class="actions" data-in>
    <a href={resumeUrl} class="btn btn-primary" use:magnetic>Read résumé</a>
    <a href="/game" class="btn" use:magnetic>Play Cinderheart</a>
  </div>
</article>

<section class="toc" aria-labelledby="toc-h">
  <header class="sec-head">
    <h2 id="toc-h">Table of contents</h2>
    <span class="sec-note">Page</span>
  </header>

  <ol class="toc-list" use:reveal={{ children: "li", gap: 55 }}>
    {#each toc as row}
      <li>
        <a href={row.href}>
          <span class="t-main">
            <span class="t-name">{row.name}</span>
            <span class="t-leader" aria-hidden="true"></span>
            <span class="t-folio">{row.folio}</span>
          </span>
          <span class="t-desc">{row.desc}</span>
        </a>
      </li>
    {/each}
  </ol>

  {#if badge}
    <aside class="marginal">
      <span class="m-mark" aria-hidden="true">✦</span>
      <a href={badge.href}>{badge.text}</a>
      <span class="m-sub">Relics from the portfolio, found in Cinderheart.</span>
    </aside>
  {/if}
</section>

<section class="particulars" aria-labelledby="part-h">
  <header class="sec-head">
    <h2 id="part-h">Particulars</h2>
    <span class="sec-note">Table I</span>
  </header>
  <table use:reveal>
    <tbody>
      {#each facts as f}
        <tr>
          <th scope="row">{f.k}</th>
          <td class="lead-cell" aria-hidden="true"></td>
          <td class="val">{f.v}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</section>

<style>
  .titlepage {
    position: relative;
    padding: var(--s-5) 0 var(--s-7);
    border-bottom: 1px solid var(--line);
  }
  .titlepage::after {
    content: "";
    display: block;
    clear: both;
  }

  /* The plate floats right and the text wraps around its circle. */
  .plate-wrap {
    float: right;
    width: min(54%, 520px);
    margin: 0 0 var(--s-5) var(--s-5);
    shape-outside: circle(50% at 50% 46%);
    shape-margin: 16px;
  }

  .eyebrow {
    margin-top: var(--s-6);
    color: var(--ink-3);
  }

  .display {
    margin: var(--s-4) 0 0;
    font-size: clamp(2.8rem, 1.4rem + 5vw, 5.6rem);
    line-height: 0.98;
    letter-spacing: -0.035em;
    text-wrap: balance;
  }
  .period {
    color: var(--falu);
    text-decoration: none;
  }
  .period:hover {
    color: var(--falu);
    opacity: 0.7;
  }

  .subtitle {
    max-width: 30ch;
    margin: var(--s-4) 0 0;
    color: var(--ink-2);
    font-size: 1.25rem;
    line-height: 1.35;
  }

  .opening {
    max-width: 38ch;
    margin: var(--s-6) 0 0;
    color: var(--ink);
    font-size: 1.1rem;
    line-height: 1.6;
  }
  .opening::first-letter {
    float: left;
    margin: 0.06em 0.1em -0.05em 0;
    color: var(--ink);
    font-family: var(--font-display);
    font-size: 4.6em;
    font-weight: 300;
    line-height: 0.82;
    font-variation-settings: "SOFT" 50, "WONK" 0, "opsz" 144, "wght" 300;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-top: var(--s-6);
  }

  /* ── Section heads ───────────────────────────────────────── */

  .sec-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-4);
    padding-bottom: var(--s-2);
    border-bottom: 3px double var(--ink);
  }
  .sec-head h2 {
    margin: 0;
    font-size: var(--text-xl);
    font-style: italic;
  }
  .sec-note {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  /* ── Table of contents ───────────────────────────────────── */

  .toc {
    position: relative;
    padding-top: var(--s-7);
  }
  .toc-list {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .toc-list li {
    border-bottom: 1px solid var(--line);
  }
  .toc-list a {
    display: block;
    padding: var(--s-3) 0;
    color: var(--ink);
    text-decoration: none;
  }
  .t-main {
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
  }
  .t-name {
    font-family: var(--font-display);
    font-size: 1.5rem;
    font-weight: 350;
    font-variation-settings: "SOFT" 50, "WONK" 0, "opsz" 72, "wght" 350;
    transition: font-style var(--dur-fast), letter-spacing var(--dur) var(--ease);
  }
  .t-leader {
    flex: 1;
    min-width: 16px;
    border-bottom: 1px dotted var(--line-strong);
    transform: translateY(-0.3em);
  }
  .t-folio {
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: 12.5px;
    font-variant-numeric: tabular-nums lining-nums;
    letter-spacing: 0.06em;
  }
  .t-desc {
    display: block;
    margin-top: 2px;
    color: var(--ink-3);
    font-size: 14px;
  }
  .toc-list a:hover .t-name {
    font-style: italic;
  }
  .toc-list a:hover .t-folio {
    color: var(--falu);
  }
  .toc-list a:focus-visible {
    outline-offset: 2px;
  }

  /* The relic count sits in the right margin on wide screens. */
  .marginal {
    display: grid;
    gap: 2px;
    margin-top: var(--s-5);
    padding-left: var(--s-3);
    border-left: 1px solid var(--falu);
    font-size: 13px;
  }
  .marginal a {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .m-mark {
    color: var(--falu);
  }
  .m-sub {
    color: var(--ink-3);
    font-style: italic;
  }
  @media (min-width: 1200px) {
    .marginal {
      position: absolute;
      top: calc(var(--s-7) + 54px);
      right: -214px;
      width: 180px;
      margin-top: 0;
    }
  }

  /* ── Particulars ─────────────────────────────────────────── */

  .particulars {
    padding-top: var(--s-7);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-variant-numeric: tabular-nums lining-nums;
  }
  tr {
    border-bottom: 1px solid var(--line);
  }
  th,
  td {
    padding: var(--s-3) 0;
    vertical-align: baseline;
  }
  th {
    width: 1%;
    padding-right: var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.1em;
    text-align: left;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .lead-cell {
    border-bottom: 1px dotted var(--line-strong);
    transform: translateY(-0.3em);
    background-clip: padding-box;
  }
  .val {
    width: 1%;
    padding-left: var(--s-2);
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: 13px;
    text-align: right;
    white-space: nowrap;
  }

  @media (max-width: 720px) {
    .plate-wrap {
      float: none;
      width: min(100%, 420px);
      margin: 0 auto var(--s-3);
      shape-outside: none;
    }
    .eyebrow {
      margin-top: var(--s-4);
    }
  }
</style>
