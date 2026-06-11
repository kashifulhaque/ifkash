<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let interactPrompt: string | null = null;

  // (dx, dy) look deltas and (x, y) move vector are pushed up to the Game's InputManager
  const dispatch = createEventDispatcher<{
    move: { x: number; y: number };
    look: { dx: number; dy: number };
    fire: void;
    interact: void;
    jump: void;
    crouch: { active: boolean };
    aim: { active: boolean };
    reload: void;
  }>();

  let crouched = false;
  let aiming = false;

  function toggleCrouch() {
    crouched = !crouched;
    dispatch('crouch', { active: crouched });
  }

  function toggleAim() {
    aiming = !aiming;
    dispatch('aim', { active: aiming });
  }

  const LOOK_SENSITIVITY = 2.2;
  const STICK_RADIUS = 50;

  let joyTouchId: number | null = null;
  let lookTouchId: number | null = null;
  let joyOrigin = { x: 0, y: 0 };
  let knob = { x: 0, y: 0 };
  let lastLook = { x: 0, y: 0 };
  let joyActive = false;

  function onZoneTouchStart(e: TouchEvent) {
    e.preventDefault();
    for (const t of Array.from(e.changedTouches)) {
      const isLeft = t.clientX < window.innerWidth / 2;
      if (isLeft && joyTouchId === null) {
        joyTouchId = t.identifier;
        joyOrigin = { x: t.clientX, y: t.clientY };
        knob = { x: 0, y: 0 };
        joyActive = true;
      } else if (!isLeft && lookTouchId === null) {
        lookTouchId = t.identifier;
        lastLook = { x: t.clientX, y: t.clientY };
      }
    }
  }

  function onZoneTouchMove(e: TouchEvent) {
    e.preventDefault();
    for (const t of Array.from(e.changedTouches)) {
      if (t.identifier === joyTouchId) {
        let dx = t.clientX - joyOrigin.x;
        let dy = t.clientY - joyOrigin.y;
        const len = Math.hypot(dx, dy);
        if (len > STICK_RADIUS) {
          dx = (dx / len) * STICK_RADIUS;
          dy = (dy / len) * STICK_RADIUS;
        }
        knob = { x: dx, y: dy };
        dispatch('move', { x: dx / STICK_RADIUS, y: -dy / STICK_RADIUS });
      } else if (t.identifier === lookTouchId) {
        dispatch('look', {
          dx: (t.clientX - lastLook.x) * LOOK_SENSITIVITY,
          dy: (t.clientY - lastLook.y) * LOOK_SENSITIVITY
        });
        lastLook = { x: t.clientX, y: t.clientY };
      }
    }
  }

  function onZoneTouchEnd(e: TouchEvent) {
    for (const t of Array.from(e.changedTouches)) {
      if (t.identifier === joyTouchId) {
        joyTouchId = null;
        joyActive = false;
        knob = { x: 0, y: 0 };
        dispatch('move', { x: 0, y: 0 });
      } else if (t.identifier === lookTouchId) {
        lookTouchId = null;
      }
    }
  }
</script>

<div
  class="touch-zone"
  on:touchstart={onZoneTouchStart}
  on:touchmove={onZoneTouchMove}
  on:touchend={onZoneTouchEnd}
  on:touchcancel={onZoneTouchEnd}
>
  {#if joyActive}
    <div class="stick-base" style="left: {joyOrigin.x}px; top: {joyOrigin.y}px">
      <div class="stick-knob" style="transform: translate({knob.x}px, {knob.y}px)"></div>
    </div>
  {/if}
</div>

<div class="buttons">
  {#if interactPrompt}
    <button class="btn use" on:touchstart|preventDefault={() => dispatch('interact')}>USE</button>
  {/if}
  <button class="btn fire" on:touchstart|preventDefault={() => dispatch('fire')}>FIRE</button>
  <div class="btn-row">
    <button class="btn small" class:on={aiming} on:touchstart|preventDefault={toggleAim}>AIM</button>
    <button class="btn small" on:touchstart|preventDefault={() => dispatch('reload')}>LOAD</button>
  </div>
  <div class="btn-row">
    <button class="btn small" on:touchstart|preventDefault={() => dispatch('jump')}>JUMP</button>
    <button class="btn small" class:on={crouched} on:touchstart|preventDefault={toggleCrouch}>DUCK</button>
  </div>
</div>

<style>
  .touch-zone {
    position: absolute;
    inset: 0;
    z-index: 15;
    touch-action: none;
  }

  .stick-base {
    position: fixed;
    width: 100px;
    height: 100px;
    margin: -50px 0 0 -50px;
    border: 2px solid rgba(255, 255, 255, 0.4);
    border-radius: 50%;
    pointer-events: none;
  }

  .stick-knob {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 44px;
    height: 44px;
    margin: -22px 0 0 -22px;
    background: rgba(255, 255, 255, 0.5);
    border-radius: 50%;
  }

  .buttons {
    position: absolute;
    right: 20px;
    bottom: 28px;
    z-index: 20;
    display: flex;
    flex-direction: column;
    gap: 14px;
    align-items: center;
  }

  .btn {
    font-family: var(--font-display, monospace);
    font-size: 1.1rem;
    letter-spacing: 0.08em;
    color: #fff;
    border-radius: 50%;
    cursor: pointer;
    touch-action: none;
    -webkit-user-select: none;
    user-select: none;
  }

  .fire {
    width: 84px;
    height: 84px;
    background: rgba(255, 77, 77, 0.55);
    border: 3px solid rgba(255, 255, 255, 0.7);
  }

  .use {
    width: 64px;
    height: 64px;
    background: rgba(80, 200, 120, 0.55);
    border: 3px solid rgba(255, 255, 255, 0.7);
    animation: pulse 1.2s ease-in-out infinite;
  }

  .btn-row {
    display: flex;
    gap: 10px;
  }

  .small {
    width: 56px;
    height: 56px;
    font-size: 0.8rem;
    background: rgba(70, 130, 220, 0.5);
    border: 2px solid rgba(255, 255, 255, 0.6);
  }

  .small.on {
    background: rgba(255, 210, 63, 0.6);
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.08); }
  }
</style>
