<script lang="ts">
  // Photo mode (WP-088): the game pauses (no simulation ticks while it is open), the camera is
  // free within 15 m of the player, the HUD is hidden unless asked for, three color grades are
  // on offer, and a capture button saves a PNG with a small ifkash.dev watermark. It writes no
  // simulation state: it sends no commands and only moves the camera, through `photo`.
  // Keys: W A S D move, Q and E down and up, Shift is faster, drag or the arrow keys look, the
  // wheel dollies, Enter or C captures, Escape or P leaves.
  import { onDestroy, tick } from 'svelte';
  import {
    GRADES,
    gradeFilter,
    lookPose,
    photoFileName,
    stepPose,
    type PhotoHost,
    type PhotoInput,
    type PhotoPose,
    type Point3
  } from './photoMath';

  export let photo: PhotoHost;
  /** The game canvas, so the chosen grade shows live. */
  export let canvas: HTMLCanvasElement | null = null;
  /** True while photo mode is on. Bind to it to hide the rest of the interface. */
  export let open = false;
  /** Show the HUD while taking pictures. Off by default: photo mode hides it. */
  export let showHud = false;

  let grade = 'natural';
  let watermark = true;
  let status = '';
  let pose: PhotoPose | null = null;
  let anchor: Point3 = { x: 0, y: 0, z: 0 };
  let raf = 0;
  let last = 0;
  let toolbar: HTMLElement | undefined;
  let dragging = false;
  const held = new Set<string>();
  let wheel = 0;

  const LOOK_PER_PX = 0.0035;
  const LOOK_KEY_RAD_PER_S = 1.6;

  /** Opens photo mode if it is closed, and closes it if it is open. */
  export function toggle(): void {
    if (open) close();
    else void enter();
  }

  async function enter(): Promise<void> {
    if (open) return;
    const start = photo.begin();
    if (!start) return;
    anchor = start.anchor;
    pose = start.pose;
    open = true;
    status = '';
    held.clear();
    last = performance.now();
    raf = requestAnimationFrame(frame);
    await tick();
    toolbar?.querySelector<HTMLElement>('button, select')?.focus();
  }

  function close(): void {
    if (!open) return;
    cancelAnimationFrame(raf);
    open = false;
    pose = null;
    held.clear();
    if (canvas) canvas.style.filter = '';
    photo.end();
  }

  onDestroy(() => {
    if (open) close();
  });

  $: if (canvas) canvas.style.filter = open ? gradeFilter(grade) : '';

  function frame(now: number): void {
    const dt = Math.min((now - last) / 1000, 0.1);
    last = now;
    if (pose) {
      const has = (...codes: string[]) => codes.some((c) => held.has(c));
      const axis = (pos: string[], neg: string[]) => (has(...pos) ? 1 : 0) - (has(...neg) ? 1 : 0);
      const input: PhotoInput = {
        forward: axis(['KeyW'], ['KeyS']) + wheel,
        right: axis(['KeyD'], ['KeyA']),
        up: axis(['KeyE'], ['KeyQ'])
      };
      wheel = 0;
      const turn = axis(['ArrowRight'], ['ArrowLeft']) * LOOK_KEY_RAD_PER_S * dt;
      const tilt = axis(['ArrowDown'], ['ArrowUp']) * LOOK_KEY_RAD_PER_S * dt;
      const fast = has('ShiftLeft', 'ShiftRight') ? 3 : 1;
      let next = pose;
      if (turn || tilt) next = lookPose(next, turn, tilt);
      if (input.forward || input.right || input.up) {
        next = stepPose(next, input, dt, anchor, 4 * fast);
      }
      if (next !== pose) {
        pose = next;
        photo.set(next);
      }
    }
    raf = requestAnimationFrame(frame);
  }

  const KEYS = new Set([
    'KeyW',
    'KeyA',
    'KeyS',
    'KeyD',
    'KeyQ',
    'KeyE',
    'ArrowUp',
    'ArrowDown',
    'ArrowLeft',
    'ArrowRight',
    'ShiftLeft',
    'ShiftRight'
  ]);

  function onKeyDown(e: KeyboardEvent): void {
    if (!open) return;
    const typing = e.target instanceof HTMLSelectElement || (e.target instanceof HTMLInputElement && e.target.type === 'text');
    if (e.code === 'Escape' || e.code === 'KeyP') {
      e.preventDefault();
      e.stopPropagation();
      close();
      return;
    }
    if (typing) return;
    if (e.code === 'KeyC' || (e.code === 'Enter' && !(e.target instanceof HTMLButtonElement))) {
      e.preventDefault();
      e.stopPropagation();
      void capture();
      return;
    }
    if (KEYS.has(e.code)) {
      e.preventDefault();
      e.stopPropagation();
      held.add(e.code);
    }
  }

  function onKeyUp(e: KeyboardEvent): void {
    held.delete(e.code);
  }

  function onPointerDown(e: PointerEvent): void {
    dragging = true;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent): void {
    if (!dragging || !pose) return;
    pose = lookPose(pose, e.movementX * LOOK_PER_PX, e.movementY * LOOK_PER_PX);
    photo.set(pose);
  }

  function onPointerUp(): void {
    dragging = false;
  }

  function onWheel(e: WheelEvent): void {
    e.preventDefault();
    wheel -= Math.sign(e.deltaY) * 4;
  }

  async function capture(): Promise<void> {
    status = 'Saving…';
    const blob = await photo.capture(grade, watermark);
    if (!blob) {
      status = 'This browser could not save the picture.';
      return;
    }
    const name = photoFileName();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = name;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 10_000);
    status = `Saved ${name}`;
  }
</script>

<svelte:window on:keydown|capture={onKeyDown} on:keyup={onKeyUp} on:blur={() => held.clear()} />

{#if open}
  <div
    class="photo"
    role="application"
    aria-label="Photo mode. Move with W A S D, Q and E for down and up, look by dragging or with the arrow keys."
    on:pointerdown={onPointerDown}
    on:pointermove={onPointerMove}
    on:pointerup={onPointerUp}
    on:pointercancel={onPointerUp}
    on:wheel={onWheel}
  >
    <div class="toolbar" bind:this={toolbar} role="toolbar" aria-label="Photo mode" on:pointerdown|stopPropagation>
      <label>
        <span>Grade</span>
        <select bind:value={grade}>
          {#each GRADES as g (g.id)}
            <option value={g.id}>{g.name}</option>
          {/each}
        </select>
      </label>
      <label class="check"><input type="checkbox" bind:checked={showHud} /> Show HUD</label>
      <label class="check"><input type="checkbox" bind:checked={watermark} /> Watermark</label>
      <button type="button" class="primary" on:click={capture}>Capture</button>
      <button type="button" on:click={close}>Close</button>
    </div>
    <p class="hint" aria-hidden="true">
      <kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> move · <kbd>Q</kbd><kbd>E</kbd> down, up · drag to look · <kbd>C</kbd> capture ·
      <kbd>Esc</kbd> leave
    </p>
    <p class="status" role="status" aria-live="polite">{status}</p>
  </div>
{/if}

<style>
  .photo {
    position: absolute;
    inset: 0;
    z-index: 15;
    cursor: grab;
    touch-action: none;
    font-family: var(--font-sans);
    color: var(--ink);
  }

  .photo:active {
    cursor: grabbing;
  }

  .toolbar {
    position: absolute;
    left: 50%;
    bottom: var(--s-4);
    transform: translateX(-50%);
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    max-width: calc(100vw - 2 * var(--s-4));
    padding: var(--s-2) var(--s-4);
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px var(--shadow-color);
    cursor: default;
    font-size: var(--text-sm);
  }

  label {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  label span {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  select,
  button {
    padding: 0.3em 0.7em;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    color: var(--ink);
    font: inherit;
    cursor: pointer;
  }

  button:hover,
  select:hover {
    border-color: var(--ink-3);
  }

  button.primary {
    border-color: var(--fjord);
    background: var(--fjord);
    color: var(--ink-inverse);
  }

  :is(select, button, input):focus-visible {
    outline: 2px solid var(--fjord);
    outline-offset: 2px;
  }

  .hint {
    position: absolute;
    top: var(--s-4);
    left: 50%;
    transform: translateX(-50%);
    margin: 0;
    padding: var(--s-1) var(--s-3);
    background: color-mix(in srgb, var(--surface) 85%, transparent);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    color: var(--ink-2);
    font-size: var(--text-xs);
    white-space: nowrap;
  }

  kbd {
    font-family: var(--font-mono);
    margin-right: 2px;
  }

  .status {
    position: absolute;
    left: 50%;
    bottom: calc(var(--s-4) + 3.5rem);
    transform: translateX(-50%);
    margin: 0;
    color: var(--ink);
    font-size: var(--text-sm);
    text-shadow: 0 1px 2px var(--shadow-color);
  }

  .status:empty {
    display: none;
  }

  @media (max-width: 640px) {
    .hint {
      display: none;
    }
  }
</style>
