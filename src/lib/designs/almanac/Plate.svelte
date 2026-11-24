<script lang="ts">
  /**
   * The frontispiece: an engraved orrery. Strokes draw themselves once on
   * load, the rings turn slowly, and the plate drifts a little as you scroll.
   * Without JavaScript or with reduced motion it is a still drawing.
   */
  import { onMount } from "svelte";
  import { animate, stagger, utils } from "animejs";
  import { prefersReducedMotion } from "$lib/motion";

  const C = 300;
  const rad = (deg: number) => (deg * Math.PI) / 180;
  const pt = (r: number, deg: number) => [C + r * Math.sin(rad(deg)), C - r * Math.cos(rad(deg))];
  const f = (n: number) => n.toFixed(2);

  /** Degree scale: a tick for each degree, longer at 5, 10, and 30. */
  const scale = (() => {
    let d = "";
    for (let a = 0; a < 360; a++) {
      const len = a % 30 === 0 ? 14 : a % 10 === 0 ? 9 : a % 5 === 0 ? 6 : 3.5;
      const [x1, y1] = pt(270, a);
      const [x2, y2] = pt(270 - len, a);
      d += `M${f(x1)} ${f(y1)}L${f(x2)} ${f(y2)}`;
    }
    return d;
  })();

  const months = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII"];
  const monthMarks = months.map((m, i) => {
    const a = i * 30 + 15;
    const [x, y] = pt(244, a);
    return { m, x: f(x), y: f(y), a };
  });

  /** Short radial hatching between two rings, like engraved shading. */
  const hatch = (() => {
    let d = "";
    for (let a = 0; a < 360; a += 4) {
      const [x1, y1] = pt(226, a);
      const [x2, y2] = pt(a % 12 === 0 ? 212 : 218, a);
      d += `M${f(x1)} ${f(y1)}L${f(x2)} ${f(y2)}`;
    }
    return d;
  })();

  /** Sun rays, long and short in turn. */
  const rays = (() => {
    let d = "";
    for (let a = 0; a < 360; a += 10) {
      const [x1, y1] = pt(26, a);
      const [x2, y2] = pt(a % 30 === 0 ? 44 : 36, a);
      d += `M${f(x1)} ${f(y1)}L${f(x2)} ${f(y2)}`;
    }
    return d;
  })();

  /** Cross-hatching for the planet bodies, as clipped diagonal lines. */
  const diag = (r: number) => {
    let d = "";
    for (let o = -r; o <= r; o += 2.6) d += `M${f(-r)} ${f(o + r)}L${f(r)} ${f(o - r)}`;
    return d;
  };

  const orbits = [
    { r: 190, a: 40, size: 11, period: 220, hatched: true, moon: true },
    { r: 150, a: 200, size: 7, period: 150, hatched: false, moon: false },
    { r: 112, a: 120, size: 9, period: 100, hatched: true, moon: false },
    { r: 78, a: 300, size: 5, period: 70, hatched: false, moon: false },
  ].map((o) => {
    const [x, y] = pt(o.r, o.a);
    return { ...o, x, y };
  });

  /** A loose constellation, as fixed points joined by thin lines. */
  const stars = [
    [96, 112], [132, 90], [168, 104], [196, 76], [226, 94],
  ];
  const starLine = stars.map((p, i) => `${i ? "L" : "M"}${p[0]} ${p[1]}`).join("");

  let root: HTMLElement | undefined;
  let svg: SVGSVGElement | undefined;
  let visible = true;
  let still = false;

  onMount(() => {
    if (!root || !svg) return;
    still = prefersReducedMotion();
    const cleanups: (() => void)[] = [];

    // Pause the turning rings when the plate is offscreen or the tab is hidden.
    const io = new IntersectionObserver(([e]) => (visible = e.isIntersecting), { threshold: 0 });
    io.observe(root);
    const onVis = () => (visible = !document.hidden && visible);
    document.addEventListener("visibilitychange", onVis);
    cleanups.push(() => io.disconnect(), () => document.removeEventListener("visibilitychange", onVis));

    if (still) return () => cleanups.forEach((c) => c());

    // Draw the strokes in, longest rings first, then fade in the dashed orbits.
    const draws = Array.from(svg.querySelectorAll<SVGElement>(".draw"));
    const soft = Array.from(svg.querySelectorAll<SVGElement>(".soft"));
    utils.set(draws, { strokeDasharray: 1, strokeDashoffset: 1 });
    utils.set(soft, { opacity: 0 });
    const done = () => {
      draws.forEach((el) => {
        el.style.removeProperty("stroke-dasharray");
        el.style.removeProperty("stroke-dashoffset");
      });
      soft.forEach((el) => el.style.removeProperty("opacity"));
    };
    const a1 = animate(draws, {
      strokeDashoffset: [1, 0],
      duration: 1800,
      delay: stagger(55),
      ease: "inOutSine",
      onComplete: done,
    });
    const a2 = animate(soft, {
      opacity: [0, 1],
      duration: 1200,
      delay: stagger(140, { start: 900 }),
      ease: "outQuad",
    });
    // Never leave the plate half drawn if a throttled renderer skips the frames.
    const fallback = setTimeout(() => {
      a1.cancel();
      a2.cancel();
      done();
    }, 6000);
    cleanups.push(() => {
      clearTimeout(fallback);
      a1.cancel();
      a2.cancel();
    });

    // A gentle parallax: the plate drifts up more slowly than the page.
    let raf = 0;
    const onScroll = () => {
      if (raf || !visible || !root) return;
      raf = requestAnimationFrame(() => {
        raf = 0;
        if (root) root.style.setProperty("--drift", `${Math.min(window.scrollY, 900) * 0.08}px`);
      });
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    cleanups.push(() => {
      window.removeEventListener("scroll", onScroll);
      if (raf) cancelAnimationFrame(raf);
    });

    return () => cleanups.forEach((c) => c());
  });
</script>

<figure class="plate" bind:this={root} class:paused={!visible} class:still>
  <svg
    bind:this={svg}
    viewBox="0 0 600 600"
    role="img"
    aria-label="An engraved orrery: the sun at the center, four planets on concentric orbits, and a ring of months."
  >
    <defs>
      <clipPath id="body-a"><circle r="11" /></clipPath>
      <clipPath id="body-b"><circle r="9" /></clipPath>
    </defs>

    <g class="art" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
      <!-- Plate edge -->
      <rect class="draw" pathLength="1" x="6" y="6" width="588" height="588" stroke-width="0.9" />
      <rect class="draw" pathLength="1" x="12" y="12" width="576" height="576" stroke-width="0.5" />

      <!-- Fixed outer scale -->
      <circle class="draw" pathLength="1" cx={C} cy={C} r="272" stroke-width="1" />
      <path class="draw" pathLength="1" d={scale} stroke-width="0.6" />
      <circle class="draw" pathLength="1" cx={C} cy={C} r="256" stroke-width="0.6" />
      {#each monthMarks as mk}
        <text
          class="month soft"
          x={mk.x}
          y={mk.y}
          transform={`rotate(${mk.a} ${mk.x} ${mk.y})`}
          text-anchor="middle"
          dominant-baseline="central"
          fill="currentColor"
          stroke="none">{mk.m}</text
        >
      {/each}

      <!-- Axes -->
      <path class="draw" pathLength="1" d={`M${C} 44V${C - 52}M${C} ${C + 52}V556M44 ${C}H${C - 52}M${C + 52} ${C}H556`} stroke-width="0.5" />

      <!-- Turning zodiac band -->
      <g class="spin spin-a">
        <circle class="draw" pathLength="1" cx={C} cy={C} r="226" stroke-width="0.8" />
        <path class="draw" pathLength="1" d={hatch} stroke-width="0.5" />
        <circle class="draw" pathLength="1" cx={C} cy={C} r="210" stroke-width="0.6" />
      </g>

      <!-- Constellation, fixed -->
      <g class="soft" transform="translate(190 120)" stroke-width="0.5" opacity="0.8">
        <path d={starLine} stroke-dasharray="2 3" />
        {#each stars as s}
          <circle cx={s[0]} cy={s[1]} r="1.8" fill="currentColor" stroke="none" />
        {/each}
      </g>

      <!-- Orbits and planets -->
      {#each orbits as o, i}
        <g class="spin" style={`--period:${o.period}s;animation-direction:${i % 2 ? "reverse" : "normal"}`}>
          <circle class="draw" pathLength="1" cx={C} cy={C} r={o.r} stroke-width="0.7" />
          <circle class="soft" cx={C} cy={C} r={o.r - 5} stroke-width="0.4" stroke-dasharray="1 4" />
          <g transform={`translate(${f(o.x)} ${f(o.y)})`}>
            <circle r={o.size} fill="var(--bg)" stroke-width="0.9" />
            {#if o.hatched}
              <g clip-path={o.size > 10 ? "url(#body-a)" : "url(#body-b)"} stroke-width="0.5" opacity="0.7">
                <path d={diag(o.size)} />
              </g>
            {:else}
              <circle r={o.size * 0.45} fill="currentColor" stroke="none" opacity="0.8" />
            {/if}
            {#if o.moon}
              <circle class="moonlet" cx="0" cy="-22" r="3" fill="var(--bg)" stroke-width="0.8" />
              <circle r="17" stroke-width="0.4" stroke-dasharray="1 3" />
            {/if}
          </g>
        </g>
      {/each}

      <!-- Sun -->
      <g class="sun">
        <path class="draw" pathLength="1" d={rays} stroke-width="0.7" />
        <circle class="draw" pathLength="1" cx={C} cy={C} r="22" stroke-width="1" />
        <circle class="draw" pathLength="1" cx={C} cy={C} r="16" stroke-width="0.5" />
        <circle class="soft" cx={C} cy={C} r="9" fill="var(--falu)" stroke="none" />
      </g>
    </g>
  </svg>
  <figcaption>
    <span class="fig">Plate I.</span> <em class="serif">The ifkash system, drawn to no particular scale.</em>
  </figcaption>
</figure>

<style>
  .plate {
    margin: 0;
    color: var(--ink);
    transform: translateY(calc(var(--drift, 0px) * -1));
    will-change: transform;
  }
  svg {
    display: block;
    width: 100%;
    height: auto;
    overflow: visible;
  }
  .month {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.08em;
    opacity: 0.8;
  }

  .spin {
    transform-box: view-box;
    transform-origin: 300px 300px;
    animation: turn var(--period, 240s) linear infinite;
  }
  .spin-a {
    --period: 320s;
  }
  .plate.paused .spin,
  .plate.still .spin {
    animation-play-state: paused;
  }
  .plate.still .spin {
    animation: none;
  }
  @keyframes turn {
    to {
      transform: rotate(360deg);
    }
  }

  figcaption {
    margin-top: var(--s-3);
    color: var(--ink-3);
    font-size: 13px;
    text-align: center;
  }
  .fig {
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
</style>
