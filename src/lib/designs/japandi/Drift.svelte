<script lang="ts">
  /**
   * A very slow ambient layer: a few drifting shapes on a canvas, chosen by
   * the month (petals in spring, fireflies in summer, maple leaves in
   * autumn, snow in winter). It draws at most 10 shapes, pauses while the
   * tab is hidden, and does not run when the visitor prefers reduced motion.
   */
  import { onMount } from "svelte";
  import { prefersReducedMotion } from "$lib/motion";

  type Kind = "petal" | "firefly" | "leaf" | "snow";

  interface Mote {
    x: number;
    y: number;
    size: number;
    speed: number;
    sway: number;
    phase: number;
    spin: number;
    angle: number;
    tone: number;
  }

  const COUNT = 10;

  let canvas: HTMLCanvasElement;

  function seasonFor(month: number): Kind {
    if (month >= 2 && month <= 4) return "petal";
    if (month >= 5 && month <= 7) return "firefly";
    if (month >= 8 && month <= 10) return "leaf";
    return "snow";
  }

  onMount(() => {
    if (prefersReducedMotion()) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const kind = seasonFor(new Date().getMonth());
    let width = 0;
    let height = 0;
    let dpr = 1;

    function resize() {
      dpr = Math.min(window.devicePixelRatio || 1, 1.5);
      width = window.innerWidth;
      height = window.innerHeight;
      canvas.width = Math.round(width * dpr);
      canvas.height = Math.round(height * dpr);
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
    }
    resize();

    const motes: Mote[] = Array.from({ length: COUNT }, (_, i) => ({
      x: Math.random() * width,
      y: Math.random() * height,
      size: kind === "snow" ? 1.6 + Math.random() * 1.6 : 6 + Math.random() * 6,
      speed: 9 + Math.random() * 10,
      sway: 14 + Math.random() * 22,
      phase: Math.random() * Math.PI * 2 + i,
      spin: (Math.random() - 0.5) * 0.5,
      angle: Math.random() * Math.PI * 2,
      tone: Math.random(),
    }));

    // Colors come from the live tokens, so they follow the theme.
    let colors: string[] = [];
    function readColors() {
      const style = getComputedStyle(document.documentElement);
      const pick = (name: string) => style.getPropertyValue(name).trim() || "#999";
      if (kind === "petal") colors = [pick("--falu"), pick("--clay")];
      else if (kind === "leaf") colors = [pick("--clay"), pick("--falu"), "#b98a3a"];
      else if (kind === "firefly") colors = [pick("--fjord"), pick("--clay")];
      else colors = [pick("--ink-3")];
    }
    readColors();

    function drawLeaf(m: Mote) {
      const s = m.size;
      ctx!.beginPath();
      ctx!.moveTo(0, -s);
      ctx!.bezierCurveTo(s * 0.9, -s * 0.5, s * 0.7, s * 0.6, 0, s);
      ctx!.bezierCurveTo(-s * 0.7, s * 0.6, -s * 0.9, -s * 0.5, 0, -s);
      ctx!.fill();
      ctx!.globalAlpha *= 0.6;
      ctx!.beginPath();
      ctx!.moveTo(0, -s);
      ctx!.lineTo(0, s * 1.15);
      ctx!.strokeStyle = ctx!.fillStyle;
      ctx!.lineWidth = 0.8;
      ctx!.stroke();
    }

    function drawPetal(m: Mote) {
      const s = m.size * 0.8;
      ctx!.beginPath();
      ctx!.moveTo(0, -s);
      ctx!.bezierCurveTo(s * 0.8, -s * 0.9, s * 0.8, s * 0.5, 0, s);
      ctx!.bezierCurveTo(-s * 0.8, s * 0.5, -s * 0.8, -s * 0.9, 0, -s);
      ctx!.fill();
    }

    let last = performance.now();
    let elapsed = 0;
    let frame = 0;
    let raf = 0;

    function tick(now: number) {
      raf = requestAnimationFrame(tick);
      const dt = Math.min((now - last) / 1000, 0.1);
      last = now;
      elapsed += dt;
      if (++frame % 240 === 0) readColors();

      ctx!.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx!.clearRect(0, 0, width, height);

      for (const m of motes) {
        const color = colors[Math.floor(m.tone * colors.length) % colors.length];
        const sway = Math.sin(elapsed * 0.35 + m.phase) * m.sway;

        if (kind === "firefly") {
          // Fireflies wander slowly and pulse instead of falling.
          m.x += Math.cos(elapsed * 0.15 + m.phase) * 6 * dt;
          m.y += Math.sin(elapsed * 0.11 + m.phase * 1.7) * 5 * dt;
          const glow = 0.25 + 0.35 * (0.5 + 0.5 * Math.sin(elapsed * 0.7 + m.phase));
          ctx!.globalAlpha = glow;
          ctx!.fillStyle = color;
          ctx!.beginPath();
          ctx!.arc(m.x, m.y, 2.2, 0, Math.PI * 2);
          ctx!.fill();
          ctx!.globalAlpha = glow * 0.25;
          ctx!.beginPath();
          ctx!.arc(m.x, m.y, 7, 0, Math.PI * 2);
          ctx!.fill();
          if (m.x < -20) m.x = width + 20;
          if (m.x > width + 20) m.x = -20;
          if (m.y < -20) m.y = height + 20;
          if (m.y > height + 20) m.y = -20;
          continue;
        }

        m.y += m.speed * dt;
        m.angle += m.spin * dt;
        if (m.y > height + 24) {
          m.y = -24;
          m.x = Math.random() * width;
        }

        ctx!.save();
        ctx!.translate(m.x + sway, m.y);
        ctx!.rotate(m.angle + Math.sin(elapsed * 0.4 + m.phase) * 0.5);
        ctx!.fillStyle = color;
        ctx!.globalAlpha = kind === "snow" ? 0.35 : 0.3;
        if (kind === "leaf") drawLeaf(m);
        else if (kind === "petal") drawPetal(m);
        else {
          ctx!.beginPath();
          ctx!.arc(0, 0, m.size, 0, Math.PI * 2);
          ctx!.fill();
        }
        ctx!.restore();
      }
    }

    function start() {
      if (raf) return;
      last = performance.now();
      raf = requestAnimationFrame(tick);
    }
    function stop() {
      cancelAnimationFrame(raf);
      raf = 0;
    }
    function onVisibility() {
      if (document.hidden) stop();
      else start();
    }

    document.addEventListener("visibilitychange", onVisibility);
    window.addEventListener("resize", resize);
    start();

    return () => {
      stop();
      document.removeEventListener("visibilitychange", onVisibility);
      window.removeEventListener("resize", resize);
    };
  });
</script>

<canvas bind:this={canvas} class="drift" aria-hidden="true"></canvas>

<style>
  .drift {
    position: fixed;
    inset: 0;
    z-index: 0;
    pointer-events: none;
  }
</style>
