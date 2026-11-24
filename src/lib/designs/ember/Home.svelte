<script>
  /**
   * PLACEHOLDER: a copy of the Almanac home until the Ember redesign lands. Replace this file freely.
   Almanac home page: an oversized Fraunces name, a short statement, the
   * facts row, and the Horizon scene.
   */
  import { onMount } from "svelte";
  import Horizon from "$lib/components/Horizon.svelte";
  import { createTimeline } from "animejs";
  import { countUp, magnetic, prefersReducedMotion, reveal, splitReveal } from "$lib/motion";
  import { readRelicBadge } from "../relics";
  import { facts, resumeUrl } from "../shared";

  /**
   * Brings the hero in as one sequence: eyebrow, then the name (rolled up by
   * splitReveal), then the copy, the full stop, and the actions.
   * @param {HTMLElement} node
   */
  function heroIntro(node) {
    if (prefersReducedMotion()) return {};
    const q = (/** @type {string} */ sel) => node.querySelector(sel);
    const lift = { opacity: [0, 1], translateY: [14, 0], duration: 700, ease: "outExpo" };
    const settle = () => node.querySelectorAll("[data-intro]").forEach((el) => {
      /** @type {HTMLElement} */ (el).style.removeProperty("opacity");
      /** @type {HTMLElement} */ (el).style.removeProperty("transform");
    });
    const tl = createTimeline({ onComplete: settle });
    const parts = [".hero-eyebrow", ".hero-copy", ".period", ".hero-actions"];
    parts.forEach((sel) => {
      const el = /** @type {HTMLElement | null} */ (q(sel));
      if (!el) return;
      el.dataset.intro = "";
      el.style.opacity = "0";
    });
    tl.add(".hero-eyebrow", lift, 0)
      .add(".hero-copy", lift, 450)
      .add(".period", { opacity: [0, 1], duration: 500, ease: "outQuad" }, 900)
      .add(".hero-actions", lift, 600);
    // Show the hero even if a throttled renderer never runs the frames.
    const fallback = setTimeout(() => { tl.cancel(); settle(); }, 2500);
    return {
      destroy() {
        clearTimeout(fallback);
        tl.cancel();
      },
    };
  }

  /** @type {{ text: string, href: string } | null} */
  let badge = null;
  onMount(() => {
    badge = readRelicBadge();
  });
</script>


<section class="hero" use:heroIntro>
  <p class="eyebrow hero-eyebrow">ML engineer — ifkash.dev</p>

  <h1 class="display">
    <span use:splitReveal={{ delay: 120 }}>Kashiful Haque</span><a href="/fitness" class="period" aria-label="Fitness dashboard">.</a>
  </h1>

  <div class="hero-body">
    <p class="hero-copy">
      I build <em class="serif">pre-training and post-training</em> systems for
      large language models, reinforcement learning pipelines, and
      high-performance inference software in C++ and Rust.
    </p>

    <div class="hero-actions">
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
</section>

<Horizon />

<dl class="facts" use:reveal={{ children: "div", gap: 80 }}>
  {#each facts as f}
    <div>
      <dt>{f.k}</dt>
      <dd use:countUp>{f.v}</dd>
    </div>
  {/each}
</dl>

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

  .relic {
    height: 40px;
    padding: 0 var(--s-4);
    font-size: var(--text-xs);
    text-decoration: none;
  }
  .relic span {
    color: var(--clay);
  }
  .relic:hover {
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
  }
</style>
