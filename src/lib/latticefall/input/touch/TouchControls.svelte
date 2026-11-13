<script lang="ts">
  // Touch mode's overlay (docs/game/07-presentation.md, "Mobile"): a floating stick on the left
  // half, camera drag on the right half, and buttons for the eight actions. It only calls the
  // input sink, so touch makes the same ActionFrame as the keyboard and the gamepad.
  import { onDestroy, onMount } from 'svelte';
  import { TOUCH_BUTTONS } from './buttons';
  import type { TouchSink } from './sink';
  import { LOOK_GAIN, STICK_RADIUS, clampKnob, stickFromOffset } from './touchMath';

  export let sink: TouchSink;
  /** Hides the overlay and drops every hold, such as while a menu is open. */
  export let hidden = false;

  type Role = { kind: 'stick' } | { kind: 'look' } | { kind: 'button'; source: string; action: (typeof TOUCH_BUTTONS)[number]['action'] };
  const fingers = new Map<number, Role>();
  const lastLook = new Map<number, [number, number]>();
  let anchor: { x: number; y: number } | null = null;
  let stickId = -1;
  let knob: [number, number] = [0, 0];
  let pressedActions = new Set<string>();
  let root: HTMLDivElement;

  function rootPoint(e: PointerEvent): { x: number; y: number } {
    const r = root.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  }

  /** Keeps a finger's later events on `el`. A pointer the browser already dropped is ignored. */
  function capture(el: HTMLElement, id: number): void {
    try {
      el.setPointerCapture(id);
    } catch {
      // The pointer ended before the handler ran.
    }
  }

  function down(e: PointerEvent): void {
    if (e.pointerType === 'mouse' && e.button !== 0) return;
    const p = rootPoint(e);
    const half = root.clientWidth / 2;
    capture(root, e.pointerId);
    if (p.x < half && stickId < 0) {
      stickId = e.pointerId;
      anchor = p;
      knob = [0, 0];
      fingers.set(e.pointerId, { kind: 'stick' });
    } else {
      fingers.set(e.pointerId, { kind: 'look' });
      lastLook.set(e.pointerId, [e.clientX, e.clientY]);
    }
  }

  function move(e: PointerEvent): void {
    const role = fingers.get(e.pointerId);
    if (!role) return;
    if (role.kind === 'stick' && anchor) {
      const p = rootPoint(e);
      const dx = p.x - anchor.x;
      const dy = p.y - anchor.y;
      knob = clampKnob(dx, dy);
      const [sx, sy] = stickFromOffset(dx, dy);
      sink.setStick(sx, sy);
    } else if (role.kind === 'look') {
      const last = lastLook.get(e.pointerId);
      if (last) sink.look((e.clientX - last[0]) * LOOK_GAIN, (e.clientY - last[1]) * LOOK_GAIN);
      lastLook.set(e.pointerId, [e.clientX, e.clientY]);
    }
  }

  function up(e: PointerEvent): void {
    const role = fingers.get(e.pointerId);
    if (!role) return;
    fingers.delete(e.pointerId);
    lastLook.delete(e.pointerId);
    if (role.kind === 'stick') {
      stickId = -1;
      anchor = null;
      knob = [0, 0];
      sink.setStick(0, 0);
    } else if (role.kind === 'button') {
      sink.release(role.action, role.source);
      pressedActions.delete(role.action);
      pressedActions = pressedActions;
    }
  }

  function buttonDown(e: PointerEvent, action: (typeof TOUCH_BUTTONS)[number]['action']): void {
    e.stopPropagation();
    e.preventDefault();
    const target = e.currentTarget as HTMLElement;
    capture(target, e.pointerId);
    const source = `Touch:${action}:${e.pointerId}`;
    fingers.set(e.pointerId, { kind: 'button', source, action });
    sink.press(action, source);
    pressedActions.add(action);
    pressedActions = pressedActions;
  }

  function buttonUp(e: PointerEvent): void {
    e.stopPropagation();
    up(e);
  }

  function releaseAll(): void {
    for (const role of fingers.values()) {
      if (role.kind === 'button') sink.release(role.action, role.source);
    }
    fingers.clear();
    lastLook.clear();
    stickId = -1;
    anchor = null;
    knob = [0, 0];
    pressedActions = new Set();
    sink.setStick(0, 0);
  }

  $: if (hidden) releaseAll();

  onMount(() => {
    // Lock-on is on by default in touch mode (docs/game/07-presentation.md, "Mobile"): one lock
    // tap at the start. The core takes it only when a target is in reach.
    sink.pulse('lock');
    window.addEventListener('blur', releaseAll);
  });
  onDestroy(() => {
    if (typeof window !== 'undefined') window.removeEventListener('blur', releaseAll);
    releaseAll();
  });
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="touch"
  class:hidden
  bind:this={root}
  on:pointerdown={down}
  on:pointermove={move}
  on:pointerup={up}
  on:pointercancel={up}
  on:contextmenu|preventDefault
  role="group"
  aria-label="Touch controls"
>
  {#if anchor}
    <div class="base" style="left:{anchor.x}px; top:{anchor.y}px; width:{STICK_RADIUS * 2}px; height:{STICK_RADIUS * 2}px">
      <div class="knob" style="transform: translate({knob[0]}px, {knob[1]}px)"></div>
    </div>
  {:else}
    <div class="stick-hint" aria-hidden="true">Move</div>
  {/if}

  <button
    type="button"
    class="menu-btn"
    aria-label="Menu"
    on:pointerdown|stopPropagation={() => sink.pulse('menu')}>Menu</button
  >

  <div class="cluster">
    {#each TOUCH_BUTTONS as b (b.action)}
      <button
        type="button"
        class="tbtn"
        class:on={pressedActions.has(b.action)}
        style="grid-column: {2 - b.col}; grid-row: {4 - b.row}"
        aria-label={b.title}
        on:pointerdown={(e) => buttonDown(e, b.action)}
        on:pointerup={buttonUp}
        on:pointercancel={buttonUp}
        on:contextmenu|preventDefault>{b.label}</button
      >
    {/each}
  </div>
</div>

<style>
  .touch {
    position: absolute;
    inset: 0;
    z-index: 5;
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
    -webkit-touch-callout: none;
    font-family: var(--font-sans);
  }

  .touch.hidden {
    display: none;
  }

  .base {
    position: absolute;
    transform: translate(-50%, -50%);
    border: 2px solid color-mix(in srgb, var(--ink) 40%, transparent);
    border-radius: 50%;
    background: color-mix(in srgb, var(--surface) 35%, transparent);
    pointer-events: none;
  }

  .knob {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 44px;
    height: 44px;
    margin: -22px 0 0 -22px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--ink) 55%, transparent);
  }

  .stick-hint {
    position: absolute;
    left: var(--s-5);
    bottom: calc(var(--s-6) + env(safe-area-inset-bottom, 0px));
    width: 96px;
    height: 96px;
    display: grid;
    place-items: center;
    border: 2px dashed color-mix(in srgb, var(--ink) 30%, transparent);
    border-radius: 50%;
    color: color-mix(in srgb, var(--ink) 55%, transparent);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    pointer-events: none;
  }

  .cluster {
    position: absolute;
    right: calc(var(--s-3) + env(safe-area-inset-right, 0px));
    bottom: calc(var(--s-4) + env(safe-area-inset-bottom, 0px));
    display: grid;
    grid-template-columns: repeat(2, 56px);
    grid-template-rows: repeat(4, 56px);
    gap: var(--s-2);
  }

  .menu-btn {
    position: absolute;
    top: calc(var(--s-3) + env(safe-area-inset-top, 0px));
    right: calc(var(--s-3) + env(safe-area-inset-right, 0px));
    min-width: 56px;
    height: 40px;
    padding: 0 var(--s-3);
    border: 1px solid color-mix(in srgb, var(--ink) 45%, transparent);
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--surface) 70%, transparent);
    color: var(--ink);
    font-family: inherit;
    font-size: var(--text-xs);
    touch-action: none;
  }

  .tbtn {
    width: 56px;
    height: 56px;
    padding: 0;
    border: 1px solid color-mix(in srgb, var(--ink) 45%, transparent);
    border-radius: 50%;
    background: color-mix(in srgb, var(--surface) 70%, transparent);
    color: var(--ink);
    font-family: inherit;
    font-size: var(--text-xs);
    touch-action: none;
  }

  .tbtn.on {
    background: var(--fjord);
    border-color: var(--fjord);
    color: var(--ink-inverse);
  }

  /* The HUD keeps its quick slots in the bottom corners, so on a tall phone the touch controls
     ride above them. */
  @media (min-height: 560px) {
    .cluster {
      bottom: calc(168px + env(safe-area-inset-bottom, 0px));
    }
    .stick-hint {
      bottom: calc(180px + env(safe-area-inset-bottom, 0px));
    }
  }

  /* Landscape phones are short: use three columns so the cluster stays low. */
  @media (max-height: 460px) {
    .cluster {
      grid-template-columns: repeat(4, 48px);
      grid-template-rows: repeat(2, 48px);
    }
    .tbtn {
      width: 48px;
      height: 48px;
      grid-column: auto !important;
      grid-row: auto !important;
    }
  }
</style>
