
<script lang="ts">
  export let intensity: 'soft' | 'medium' | 'strong' = 'medium';
  export let fixed: boolean = true;
</script>

<div class="signal-field" class:fixed class:soft={intensity === 'soft'} class:strong={intensity === 'strong'} aria-hidden="true">
  <div class="grid"></div>
  <div class="orb orb-a"></div>
  <div class="orb orb-b"></div>
  <div class="orb orb-c"></div>
  <div class="noise"></div>
  <div class="vignette"></div>
</div>

<style>
  .signal-field {
    position: absolute;
    inset: 0;
    z-index: 0;
    overflow: hidden;
    pointer-events: none;
  }

  .signal-field.fixed {
    position: fixed;
  }

  .grid {
    position: absolute;
    inset: -2px;
    background-image:
      linear-gradient(to right, var(--grid-line) 1px, transparent 1px),
      linear-gradient(to bottom, var(--grid-line) 1px, transparent 1px);
    background-size: 80px 80px;
    opacity: 0.6;
    mask-image: radial-gradient(ellipse 90% 70% at 50% 30%, #000 30%, transparent 80%);
    -webkit-mask-image: radial-gradient(ellipse 90% 70% at 50% 30%, #000 30%, transparent 80%);
  }

  .orb {
    position: absolute;
    border-radius: 50%;
    filter: blur(70px);
    opacity: 0.35;
    will-change: transform;
    animation: drift 18s ease-in-out infinite alternate;
  }

  .orb-a {
    width: 540px;
    height: 540px;
    top: -180px;
    left: -120px;
    background: radial-gradient(circle, var(--accent) 0%, transparent 65%);
    animation-duration: 22s;
  }

  .orb-b {
    width: 480px;
    height: 480px;
    top: 25%;
    right: -160px;
    background: radial-gradient(circle, var(--signal-violet) 0%, transparent 65%);
    animation-duration: 28s;
    animation-delay: -6s;
    opacity: 0.22;
  }

  .orb-c {
    width: 380px;
    height: 380px;
    bottom: -120px;
    left: 35%;
    background: radial-gradient(circle, var(--signal) 0%, transparent 65%);
    animation-duration: 24s;
    animation-delay: -10s;
    opacity: 0.28;
  }

  .soft .orb { opacity: 0.18; }
  .strong .orb { opacity: 0.5; }

  .noise {
    position: absolute;
    inset: 0;
    opacity: var(--noise-opacity);
    mix-blend-mode: overlay;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='180' height='180'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/></filter><rect width='100%25' height='100%25' filter='url(%23n)' opacity='1'/></svg>");
  }

  .vignette {
    position: absolute;
    inset: 0;
    background: radial-gradient(ellipse 100% 80% at 50% 50%, transparent 50%, var(--paper) 100%);
  }
</style>
