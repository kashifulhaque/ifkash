<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{ move: { x: number; y: number }; hop: void; run: { active: boolean } }>();

  const RADIUS = 44;
  let zone: HTMLDivElement;
  let active = false;
  let knobX = 0;
  let knobY = 0;
  let originX = 0;
  let originY = 0;
  let pointerId: number | null = null;
  let running = false;

  function onDown(e: PointerEvent) {
    if (pointerId !== null) return;
    pointerId = e.pointerId;
    zone.setPointerCapture(e.pointerId);
    const rect = zone.getBoundingClientRect();
    originX = rect.left + rect.width / 2;
    originY = rect.top + rect.height / 2;
    active = true;
    onMove(e);
  }

  function onMove(e: PointerEvent) {
    if (e.pointerId !== pointerId) return;
    let dx = e.clientX - originX;
    let dy = e.clientY - originY;
    const len = Math.hypot(dx, dy);
    if (len > RADIUS) {
      dx = (dx / len) * RADIUS;
      dy = (dy / len) * RADIUS;
    }
    knobX = dx;
    knobY = dy;
    dispatch('move', { x: dx / RADIUS, y: -dy / RADIUS });
  }

  function onUp(e: PointerEvent) {
    if (e.pointerId !== pointerId) return;
    pointerId = null;
    active = false;
    knobX = 0;
    knobY = 0;
    dispatch('move', { x: 0, y: 0 });
  }

  function toggleRun() {
    running = !running;
    dispatch('run', { active: running });
  }
</script>

<div class="touch">
  <div
    class="stick"
    class:active
    bind:this={zone}
    on:pointerdown={onDown}
    on:pointermove={onMove}
    on:pointerup={onUp}
    on:pointercancel={onUp}
    role="presentation"
  >
    <div class="knob" style="transform: translate({knobX}px, {knobY}px)"></div>
  </div>

  <div class="buttons">
    <button class="btn" class:on={running} on:click={toggleRun} aria-label="Toggle run" aria-pressed={running}>
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M13 4l-1 5 4 2-2 5-3-1-3 5M14 3.5a1 1 0 1 0 2 0 1 1 0 1 0-2 0" /></svg>
      <span>run</span>
    </button>
    <button class="btn big" on:pointerdown|preventDefault={() => dispatch('hop')} aria-label="Hop">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 19V6M6 12l6-6 6 6" /></svg>
      <span>hop</span>
    </button>
  </div>
</div>

<style>
  .touch {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 20;
    font-family: var(--planet-sans);
  }
  .stick {
    position: absolute;
    left: 24px;
    bottom: 26px;
    width: 120px;
    height: 120px;
    border-radius: 50%;
    background: rgba(10, 24, 34, 0.35);
    border: 1px solid rgba(255, 255, 255, 0.16);
    backdrop-filter: blur(6px);
    pointer-events: auto;
    touch-action: none;
    display: grid;
    place-items: center;
    transition: background 0.15s;
  }
  .stick.active {
    background: rgba(10, 24, 34, 0.5);
  }
  .knob {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: rgba(242, 239, 230, 0.85);
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.35);
  }
  .buttons {
    position: absolute;
    right: 22px;
    bottom: 30px;
    display: flex;
    align-items: flex-end;
    gap: 12px;
    pointer-events: auto;
  }
  .btn {
    width: 58px;
    height: 58px;
    border-radius: 50%;
    background: rgba(10, 24, 34, 0.45);
    border: 1px solid rgba(255, 255, 255, 0.18);
    backdrop-filter: blur(6px);
    color: rgba(242, 239, 230, 0.9);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    font-size: 0.55rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    touch-action: manipulation;
  }
  .btn.big {
    width: 70px;
    height: 70px;
  }
  .btn.on {
    border-color: rgba(233, 196, 106, 0.7);
    color: #e9c46a;
  }
  .btn svg {
    width: 20px;
    height: 20px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
