<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let pointerLocked: boolean;
  export let isTouch: boolean;
  export let interactPrompt: string | null = null;
  export let health = 100;
  export let hitCount = 0;
  export let deathCount = 0;
  export let muted = false;
  export let ammo = 12;
  export let reserve = 24;
  export let reloading = false;
  export let score = 0;
  export let streak = 0;
  export let aiming = false;
  export let yaw = 0;
  export let hitMarker: { id: number; headshot: boolean; killed: boolean } | null = null;

  const dispatch = createEventDispatcher();

  const COMPASS_POINTS = [
    { label: 'N', angle: 0 },
    { label: 'NE', angle: 45 },
    { label: 'E', angle: 90 },
    { label: 'SE', angle: 135 },
    { label: 'S', angle: 180 },
    { label: 'SW', angle: 225 },
    { label: 'W', angle: 270 },
    { label: 'NW', angle: 315 }
  ];

  // Camera yaw → compass heading in degrees (0 = north = -z)
  $: heading = ((-yaw * 180) / Math.PI + 360 * 10) % 360;

  function compassOffset(angle: number, head: number): number {
    let d = angle - head;
    if (d > 180) d -= 360;
    if (d < -180) d += 360;
    return d;
  }
</script>

<div class="hud" aria-hidden="true">
  {#if !aiming}
    <div class="crosshair"></div>
  {/if}

  {#key hitMarker?.id}
    {#if hitMarker}
      <div
        class="hit-marker"
        class:headshot={hitMarker.headshot}
        class:killed={hitMarker.killed}
      >
        <span></span><span></span><span></span><span></span>
      </div>
    {/if}
  {/key}

  <div class="compass">
    {#each COMPASS_POINTS as p}
      {@const off = compassOffset(p.angle, heading)}
      {#if Math.abs(off) < 60}
        <span class="compass-point" class:cardinal={p.label.length === 1} style="left: calc(50% + {off * 2.2}px)">{p.label}</span>
      {/if}
    {/each}
    <div class="compass-tick"></div>
  </div>

  <div class="score">
    <div class="score-value">{score}</div>
    {#if streak >= 2}
      <div class="streak">×{streak} STREAK</div>
    {/if}
  </div>

  <div class="ammo" class:reloading class:empty={!reloading && ammo === 0 && reserve === 0}>
    {#if reloading}RELOADING…{:else}{ammo} / {reserve}{/if}
  </div>

  {#key hitCount}
    {#if hitCount > 0}<div class="damage-flash"></div>{/if}
  {/key}

  {#key deathCount}
    {#if deathCount > 0}<div class="death-flash"><span>WASTED</span></div>{/if}
  {/key}

  <div class="health">
    <span class="health-label">HP</span>
    <div class="health-bar">
      <div
        class="health-fill"
        class:low={health < 35}
        style="width: {Math.max(0, health)}%"
      ></div>
    </div>
  </div>

  <button class="mute" on:click={() => dispatch('mute')}>
    {muted ? '🔇' : '🔊'}
  </button>

  {#if interactPrompt}
    <div class="interact">
      {#if isTouch}TAP <span class="key">USE</span>{:else}PRESS <span class="key">E</span>{/if}
      TO {interactPrompt}
    </div>
  {:else if pointerLocked || isTouch}
    <div class="hint">SHOOT A TARGET · LOOT THE CRATE · THEY SHOOT BACK</div>
  {/if}
</div>

<style>
  .hud {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 10;
  }

  .crosshair {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 18px;
    height: 18px;
    transform: translate(-50%, -50%);
  }

  .crosshair::before,
  .crosshair::after {
    content: '';
    position: absolute;
    background: rgba(255, 255, 255, 0.9);
    box-shadow: 0 0 4px rgba(0, 0, 0, 0.5);
  }

  .crosshair::before {
    left: 50%;
    top: 0;
    bottom: 0;
    width: 2px;
    transform: translateX(-50%);
  }

  .crosshair::after {
    top: 50%;
    left: 0;
    right: 0;
    height: 2px;
    transform: translateY(-50%);
  }

  .hit-marker {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 26px;
    height: 26px;
    transform: translate(-50%, -50%) rotate(45deg);
    animation: marker-fade 0.3s ease-out forwards;
  }

  .hit-marker span {
    position: absolute;
    background: rgba(255, 255, 255, 0.95);
    width: 2px;
    height: 8px;
  }

  .hit-marker span:nth-child(1) { left: 12px; top: 0; }
  .hit-marker span:nth-child(2) { left: 12px; bottom: 0; }
  .hit-marker span:nth-child(3) { top: 12px; left: 0; width: 8px; height: 2px; }
  .hit-marker span:nth-child(4) { top: 12px; right: 0; width: 8px; height: 2px; }

  .hit-marker.headshot span { background: #ffd23f; }
  .hit-marker.killed { width: 34px; height: 34px; }
  .hit-marker.killed span { background: #ff4d4d; height: 11px; }
  .hit-marker.killed span:nth-child(1) { left: 16px; }
  .hit-marker.killed span:nth-child(2) { left: 16px; }
  .hit-marker.killed span:nth-child(3) { top: 16px; width: 11px; height: 2px; }
  .hit-marker.killed span:nth-child(4) { top: 16px; width: 11px; height: 2px; }

  @keyframes marker-fade {
    0% { opacity: 1; }
    70% { opacity: 1; }
    100% { opacity: 0; }
  }

  .compass {
    position: absolute;
    top: 14px;
    left: 50%;
    transform: translateX(-50%);
    width: 280px;
    height: 28px;
    overflow: hidden;
    background: rgba(0, 0, 0, 0.25);
    border-radius: 4px;
    mask-image: linear-gradient(to right, transparent, #000 20%, #000 80%, transparent);
    -webkit-mask-image: linear-gradient(to right, transparent, #000 20%, #000 80%, transparent);
  }

  .compass-point {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    font-family: var(--font-display, monospace);
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.7);
  }

  .compass-point.cardinal {
    font-size: 1.05rem;
    color: #fff;
  }

  .compass-tick {
    position: absolute;
    left: 50%;
    bottom: 0;
    width: 2px;
    height: 7px;
    transform: translateX(-50%);
    background: #ffd23f;
  }

  .score {
    position: absolute;
    top: 18px;
    right: 76px;
    text-align: right;
    font-family: var(--font-display, monospace);
    color: #fff;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.6);
  }

  .score-value {
    font-size: 1.5rem;
  }

  .streak {
    font-size: 0.95rem;
    color: #ffd23f;
    animation: pulse 1.2s ease-in-out infinite;
  }

  .ammo {
    position: absolute;
    right: 24px;
    bottom: 24px;
    font-family: var(--font-display, monospace);
    font-size: 1.6rem;
    color: #fff;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.6);
  }

  .ammo.reloading {
    font-size: 1.1rem;
    color: #ffd23f;
    animation: blink 0.7s ease-in-out infinite;
  }

  .ammo.empty {
    color: #ff5252;
  }

  .damage-flash {
    position: absolute;
    inset: 0;
    background: radial-gradient(ellipse at center, transparent 40%, rgba(255, 30, 30, 0.45) 100%);
    animation: fade-out 0.5s ease-out forwards;
  }

  .death-flash {
    position: absolute;
    inset: 0;
    background: rgba(120, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    animation: fade-out 1.6s ease-out forwards;
  }

  .death-flash span {
    font-family: var(--font-display, monospace);
    font-size: clamp(3rem, 10vw, 6rem);
    letter-spacing: 0.15em;
    color: #fff;
    text-shadow: 0 4px 12px rgba(0, 0, 0, 0.8);
  }

  @keyframes fade-out {
    from { opacity: 1; }
    to { opacity: 0; }
  }

  .health {
    position: absolute;
    left: 24px;
    bottom: 24px;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .health-label {
    font-family: var(--font-display, monospace);
    font-size: 1.2rem;
    color: #fff;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.6);
  }

  .health-bar {
    width: 180px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.8);
    background: rgba(0, 0, 0, 0.4);
  }

  .health-fill {
    height: 100%;
    background: #5be36b;
    transition: width 0.2s ease-out;
  }

  .health-fill.low {
    background: #ff4d4d;
    animation: blink 0.7s ease-in-out infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .mute {
    position: absolute;
    top: 18px;
    right: 18px;
    pointer-events: auto;
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid rgba(255, 255, 255, 0.4);
    font-size: 1.1rem;
    width: 42px;
    height: 42px;
    cursor: pointer;
    border-radius: 4px;
  }

  .hint,
  .interact {
    position: absolute;
    bottom: 48px;
    left: 50%;
    transform: translateX(-50%);
    font-family: var(--font-display, monospace);
    font-size: 1.3rem;
    letter-spacing: 0.08em;
    color: #fff;
    text-shadow: 0 2px 6px rgba(0, 0, 0, 0.6);
    white-space: nowrap;
  }

  .interact {
    font-size: 1.6rem;
    animation: pulse 1.2s ease-in-out infinite;
  }

  .key {
    display: inline-block;
    padding: 0 10px;
    border: 2px solid #fff;
    border-radius: 4px;
    margin: 0 4px;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }

  @media (max-width: 600px) {
    .hint { font-size: 0.9rem; bottom: 150px; }
    .interact { font-size: 1.1rem; bottom: 150px; }
    .health { left: 14px; bottom: 14px; }
    .health-bar { width: 120px; }
    .ammo { right: auto; left: 14px; bottom: 44px; font-size: 1.2rem; }
    .compass { width: 200px; }
    .score { right: 70px; top: 14px; }
    .score-value { font-size: 1.2rem; }
  }
</style>
