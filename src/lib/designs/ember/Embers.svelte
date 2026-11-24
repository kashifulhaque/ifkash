<script lang="ts">
  /**
   * A canvas of slow, drifting embers. About 11 particles on the home page (6 on small screens), and 3 on inner
   * pages (none on small screens). Device pixel ratio 1. It pauses when the tab is hidden or the canvas is offscreen,
   * and draws one still frame under reduced motion.
   */
  import { onMount } from "svelte";
  import { prefersReducedMotion } from "$lib/motion";

  /** Inner pages pass true, which thins the embers out. */
  export let calm = false;

  let canvas: HTMLCanvasElement;
  let refit: (() => void) | undefined;
  $: calm, refit?.();

  onMount(() => {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const context = ctx;

    interface Mote {
      x: number;
      y: number;
      vy: number;
      r: number;
      sway: number;
      phase: number;
      alpha: number;
      age: number;
      life: number;
    }

    const reduced = prefersReducedMotion();
    let w = 0;
    let h = 0;
    let motes: Mote[] = [];
    let raf = 0;
    let last = 0;
    let onscreen = true;
    let running = false;

    const small = () => window.innerWidth < 640;
    const target = () => (calm ? (small() ? 0 : 3) : small() ? 6 : 11);

    function spawn(initial: boolean): Mote {
      const life = 11 + Math.random() * 9;
      return {
        x: Math.random() * w,
        y: initial ? Math.random() * h : h + 10,
        vy: 10 + Math.random() * 20,
        r: 0.6 + Math.random() * 0.9,
        sway: 8 + Math.random() * 18,
        phase: Math.random() * Math.PI * 2,
        alpha: 0.12 + Math.random() * 0.2,
        age: initial ? Math.random() * life : 0,
        life,
      };
    }

    function resize() {
      w = canvas.width = window.innerWidth;
      h = canvas.height = window.innerHeight;
      const n = target();
      while (motes.length < n) motes.push(spawn(true));
      motes.length = Math.min(motes.length, n);
      if (reduced) draw();
    }

    function draw() {
      const dark = document.documentElement.dataset.theme === "dark";
      const rgb = dark ? "240, 216, 144" : "138, 96, 24";
      context.clearRect(0, 0, w, h);
      for (const m of motes) {
        const t = m.age / m.life;
        const fade = Math.min(1, t * 6) * Math.min(1, (1 - t) * 3);
        const a = m.alpha * fade * (dark ? 1 : 0.55);
        const x = m.x + Math.sin(m.phase + m.age * 0.5) * m.sway;
        context.fillStyle = `rgba(${rgb}, ${a * 0.12})`;
        context.beginPath();
        context.arc(x, m.y, m.r * 2.6, 0, 6.2832);
        context.fill();
        context.fillStyle = `rgba(${rgb}, ${a})`;
        context.beginPath();
        context.arc(x, m.y, m.r, 0, 6.2832);
        context.fill();
      }
    }

    function frame(now: number) {
      raf = requestAnimationFrame(frame);
      const dt = Math.min(0.05, (now - last) / 1000);
      last = now;
      for (let i = 0; i < motes.length; i++) {
        const m = motes[i];
        m.age += dt;
        m.y -= m.vy * dt;
        if (m.age >= m.life || m.y < -10) motes[i] = spawn(false);
      }
      draw();
    }

    function sync() {
      const should = !reduced && onscreen && !document.hidden;
      if (should && !running) {
        running = true;
        last = performance.now();
        raf = requestAnimationFrame(frame);
      } else if (!should && running) {
        running = false;
        cancelAnimationFrame(raf);
      }
    }

    const observer = new IntersectionObserver((entries) => {
      onscreen = entries.some((e) => e.isIntersecting);
      sync();
    });
    refit = resize;
    observer.observe(canvas);
    document.addEventListener("visibilitychange", sync);
    window.addEventListener("resize", resize);
    resize();
    sync();

    return () => {
      refit = undefined;
      running = false;
      cancelAnimationFrame(raf);
      observer.disconnect();
      document.removeEventListener("visibilitychange", sync);
      window.removeEventListener("resize", resize);
    };
  });
</script>

<canvas bind:this={canvas} class="embers" aria-hidden="true"></canvas>

<style>
  .embers {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
</style>
