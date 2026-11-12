<script lang="ts">
  // Remappable controls: one row per action with its keyboard-or-mouse binding and its gamepad
  // binding, plus the hold-or-toggle and attack options. Edits go straight to the input
  // controller's settings (`input/bindings.ts`) and are saved by `input.save()`.
  import { onDestroy } from 'svelte';
  import {
    ALL_ACTIONS,
    TOGGLEABLE,
    defaultSettings,
    parseToken,
    type Action,
    type ButtonAction
  } from '../../input/bindings';
  import type { InputController } from '../../input';
  import { navState } from './nav';

  export let input: InputController;
  export let announce: (text: string) => void;

  const LABELS: Record<Action, string> = {
    moveForward: 'Move forward',
    moveBack: 'Move back',
    moveLeft: 'Move left',
    moveRight: 'Move right',
    light: 'Light attack',
    heavy: 'Heavy attack',
    guard: 'Guard or parry',
    dodge: 'Dodge, sprint',
    art: 'Art',
    jump: 'Jump',
    interact: 'Interact',
    useItem: 'Use item',
    lock: 'Lock on',
    switchL: 'Switch target left',
    switchR: 'Switch target right',
    cycleItem: 'Next quick item',
    cycleSpell: 'Next spell',
    cycleRight: 'Next right-hand weapon',
    cycleLeft: 'Next left-hand weapon',
    twoHand: 'Two-hand',
    map: 'Map',
    menu: 'Menu and pause'
  };
  const PAD_NAMES = ['A', 'B', 'X', 'Y', 'LB', 'RB', 'LT', 'RT', 'View', 'Menu', 'Left stick', 'Right stick', 'D-pad up', 'D-pad down', 'D-pad left', 'D-pad right', 'Home', 'Touchpad'];
  const MOUSE_NAMES = ['Left mouse', 'Middle mouse', 'Right mouse'];
  // Interact and Use item share a gamepad button by design (see `InputController.padDown`).
  const SHARE_PAD = new Set<Action>(['interact', 'useItem']);

  let bindings = { ...input.settings.bindings };
  let options = { ...input.settings.options, toggle: { ...input.settings.options.toggle } };
  let capturing: { action: Action; device: 'kb' | 'pad' } | null = null;
  let stopCapture: (() => void) | null = null;

  const DEVICES: ('kb' | 'pad')[] = ['kb', 'pad'];
  const isPad = (t: string) => parseToken(t).base.startsWith('Pad');

  export function friendly(token: string): string {
    const { shift, base } = parseToken(token);
    let name = base;
    if (base.startsWith('Pad')) name = PAD_NAMES[Number(base.slice(3))] ?? base;
    else if (base.startsWith('Mouse')) name = MOUSE_NAMES[Number(base.slice(5))] ?? base;
    else if (base.startsWith('Key')) name = base.slice(3);
    else if (base.startsWith('Digit')) name = base.slice(5);
    else if (base.startsWith('Arrow')) name = `${base.slice(5)} arrow`;
    else if (base === 'WheelUp') name = 'Wheel up';
    else if (base === 'WheelDown') name = 'Wheel down';
    return shift ? `Shift + ${name}` : name;
  }

  function show(action: Action, device: 'kb' | 'pad'): string {
    const t = bindings[action].filter((x) => isPad(x) === (device === 'pad'));
    return t.length ? t.map(friendly).join(', ') : 'Unbound';
  }

  function commit(): void {
    input.settings.bindings = { ...bindings };
    input.settings.options = { ...options, toggle: { ...options.toggle } };
    input.save();
    bindings = { ...input.settings.bindings };
  }

  function bind(action: Action, device: 'kb' | 'pad', token: string): void {
    const base = parseToken(token).base;
    const next = { ...bindings };
    next[action] = [...next[action].filter((t) => isPad(t) !== (device === 'pad')), token];
    // A token drives one action, so taking it from another action unbinds it there.
    for (const other of ALL_ACTIONS) {
      if (other === action) continue;
      if (device === 'pad' && SHARE_PAD.has(other) && SHARE_PAD.has(action)) continue;
      next[other] = next[other].filter((t) => parseToken(t).base !== base);
    }
    bindings = next;
    commit();
    announce(`${LABELS[action]} is now ${friendly(token)}`);
  }

  function endCapture(): void {
    stopCapture?.();
    stopCapture = null;
    capturing = null;
    navState.capturing = false;
  }

  function startCapture(action: Action, device: 'kb' | 'pad'): void {
    endCapture();
    capturing = { action, device };
    navState.capturing = true;
    announce(`Press a ${device === 'pad' ? 'gamepad button' : 'key or mouse button'} for ${LABELS[action]}. Escape cancels.`);
    const stops: (() => void)[] = [];
    if (device === 'kb') {
      const onKey = (e: KeyboardEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (e.key === 'Escape') return endCapture();
        if (['ShiftLeft', 'ShiftRight', 'ControlLeft', 'ControlRight', 'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight'].includes(e.code)) return;
        bind(action, 'kb', e.code);
        endCapture();
      };
      const onMouse = (e: MouseEvent) => {
        if (e.button > 2) return;
        e.preventDefault();
        e.stopPropagation();
        bind(action, 'kb', `${e.shiftKey ? 'Shift+' : ''}Mouse${e.button}`);
        endCapture();
      };
      window.addEventListener('keydown', onKey, true);
      window.addEventListener('mousedown', onMouse, true);
      stops.push(() => window.removeEventListener('keydown', onKey, true), () => window.removeEventListener('mousedown', onMouse, true));
      // The click that started capture has already been delivered; ignore its mouseup.
    } else {
      let raf = 0;
      const start = (navigator.getGamepads?.() ?? []).find((p) => p?.connected && p.mapping === 'standard');
      const held = new Set<number>();
      start?.buttons.forEach((b, i) => b.pressed && held.add(i));
      const poll = () => {
        raf = requestAnimationFrame(poll);
        const p = (navigator.getGamepads?.() ?? []).find((g) => g?.connected && g.mapping === 'standard');
        if (!p) return;
        p.buttons.forEach((b, i) => {
          if (!b.pressed) held.delete(i);
          else if (!held.has(i)) {
            bind(action, 'pad', `Pad${i}`);
            endCapture();
          }
        });
      };
      raf = requestAnimationFrame(poll);
      stops.push(() => cancelAnimationFrame(raf));
      // Keyboard Escape still cancels a gamepad capture.
      const onKey = (e: KeyboardEvent) => {
        if (e.key !== 'Escape') return;
        e.preventDefault();
        e.stopPropagation();
        endCapture();
      };
      window.addEventListener('keydown', onKey, true);
      stops.push(() => window.removeEventListener('keydown', onKey, true));
    }
    stopCapture = () => stops.forEach((s) => s());
  }

  function resetAll(): void {
    const d = defaultSettings();
    bindings = d.bindings;
    options = d.options;
    commit();
    announce('Controls reset to the defaults');
  }

  function setToggle(a: ButtonAction, on: boolean): void {
    options.toggle = { ...options.toggle, [a]: on || undefined };
    options = options;
    commit();
  }

  $: sensitivity = Math.round((options.mouseSensitivity / 0.0022) * 100);

  /** Escape cancels a capture before the menu sees it. */
  export function back(): boolean {
    if (!capturing) return false;
    endCapture();
    return true;
  }

  onDestroy(endCapture);
</script>

<div class="controls">
  <p class="note first">
    Choose a binding, then press the new key, mouse button, or gamepad button. Escape cancels. Taking an input
    from another action unbinds it there.
  </p>
  <table>
    <thead>
      <tr><th scope="col">Action</th><th scope="col">Keyboard and mouse</th><th scope="col">Gamepad</th></tr>
    </thead>
    <tbody>
      {#each ALL_ACTIONS as a (a)}
        <tr>
          <th scope="row">{LABELS[a]}</th>
          {#each DEVICES as dev (dev)}
            <td>
              <button
                type="button"
                data-nav="body"
                aria-label={`${LABELS[a]}, ${dev === 'kb' ? 'keyboard and mouse' : 'gamepad'}: ${show(a, dev)}. Change`}
                aria-pressed={capturing?.action === a && capturing.device === dev}
                on:click={() => startCapture(a, dev)}
              >
                {capturing?.action === a && capturing.device === dev ? 'Press an input…' : show(a, dev)}
              </button>
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>

  <h3>Options</h3>
  <ul class="list opts">
    {#each TOGGLEABLE as a (a)}
      <li>
        <label>
          <input type="checkbox" data-nav="body" checked={!!options.toggle[a]} on:change={(e) => setToggle(a, e.currentTarget.checked)} />
          Toggle instead of hold: {LABELS[a]}
        </label>
      </li>
    {/each}
    <li>
      <label>
        <input
          type="checkbox"
          data-nav="body"
          checked={options.oneButtonAttack}
          on:change={(e) => {
            options.oneButtonAttack = e.currentTarget.checked;
            commit();
          }}
        />
        One-button attack (tap is light, hold is heavy)
      </label>
    </li>
    <li>
      <label>
        <input
          type="checkbox"
          data-nav="body"
          checked={options.invertY}
          on:change={(e) => {
            options.invertY = e.currentTarget.checked;
            commit();
          }}
        />
        Invert camera Y
      </label>
    </li>
    <li class="slider">
      <label for="lf-sens">Mouse sensitivity <output>{sensitivity}%</output></label>
      <input
        id="lf-sens"
        type="range"
        data-nav="body"
        min="25"
        max="300"
        step="5"
        value={sensitivity}
        aria-valuetext={`${sensitivity} percent`}
        on:input={(e) => {
          options.mouseSensitivity = (Number(e.currentTarget.value) / 100) * 0.0022;
          commit();
        }}
      />
    </li>
  </ul>
  <p><button type="button" data-nav="body" on:click={resetAll}>Reset controls to the defaults</button></p>
</div>

<style>
  .first {
    margin-top: 0;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9375em;
  }

  th,
  td {
    padding: 0.25em 0.5em 0.25em 0;
    text-align: left;
    border-bottom: 1px solid var(--line);
  }

  thead th {
    font-size: 0.75em;
    color: var(--ink-3);
    font-weight: 600;
  }

  tbody th {
    font-weight: 400;
  }

  td button {
    width: 100%;
    text-align: left;
    min-height: 2em;
    padding: 0.25em 0.5em;
  }

  .opts label {
    display: flex;
    gap: 0.5em;
    align-items: center;
  }

  .slider {
    display: grid;
    gap: 0.25em;
  }

  input[type='range'] {
    width: 100%;
    accent-color: var(--fjord);
  }
</style>
