<script lang="ts">
  // The Latticefall menus: pause, equipment, inventory, the Lantern menu, and settings
  // (docs/game/07-presentation.md, "Menus"). DOM over the canvas, drawn with the site's tokens.
  // The menus pause the game, read the character from the core (`host.read()`), and change it
  // only through `host`, which queues core `Command`s.
  //
  // Input: keyboard (arrows, Enter or Space, Escape, Tab), gamepad (D-pad or left stick, A to
  // confirm, B to go back, Start to close; see `nav.ts`), and screen readers (a modal dialog,
  // labelled lists and buttons, and a polite live region that announces screens, items, and
  // results).
  import { onDestroy, tick } from 'svelte';
  import type { Readable } from 'svelte/store';
  import type { InputController } from '../../input';
  import type { HudSettingsStore } from '../hud/hudSettings';
  import type { MenuHost } from './menuHost';
  import { emptyMenuState, type MenuState } from './menuState';
  import type { MenuSettings, MenuSettingsStore } from './menuSettings';
  import { focusFirst, moveForKey, navState, navigate, startGamepadNav, type NavMove } from './nav';
  import Equipment from './Equipment.svelte';
  import Inventory from './Inventory.svelte';
  import LanternMenu from './LanternMenu.svelte';
  import Settings from './Settings.svelte';
  import MapScreen from '../map/MapScreen.svelte';
  import type { MapHost } from '../map/mapState';

  export let host: MenuHost;
  /** What the map reads and the commands it sends (WP-069). The Map entry is off without it. */
  export let mapHost: MapHost | undefined = undefined;
  export let engine: { setPaused(paused: boolean): void; input: InputController };
  export let hudSettings: HudSettingsStore;
  export let menuSettings: MenuSettingsStore;
  /** Leave for the plain portfolio site. */
  export let onPlain: () => void;
  /** Leave the game for the title (the plain site until a title screen exists). */
  export let onQuit: () => void;
  /** True while a menu is open. Bind to it to pause your own input. */
  export let open = false;

  type Screen = 'pause' | 'equipment' | 'inventory' | 'lantern' | 'settings' | 'map';
  const COLS: Record<Screen, string> = {
    pause: 'main',
    equipment: 'slots,cands',
    inventory: 'items,detail',
    lantern: 'tabs,body',
    settings: 'tabs,body',
    map: 'tabs,body'
  };
  const TITLES: Record<Screen, string> = {
    pause: 'Paused',
    equipment: 'Equipment',
    inventory: 'Inventory',
    lantern: 'Lantern',
    settings: 'Settings',
    map: 'Map'
  };

  let screen: Screen | null = null;
  let state: MenuState = emptyMenuState();
  let root: HTMLElement;
  let screenEl: HTMLElement;
  let child: { back?: () => boolean } | undefined;
  let live = '';
  let returnTo: HTMLElement | null = null;
  let stopPad: (() => void) | null = null;

  $: textScale = ($menuSettings as MenuSettings).accessibility.textSize / 100;

  /** The engine calls this for a press of the menu or map action. */
  export function handleUi(action: 'menu' | 'map'): void {
    if (action === 'menu' && screen === null) show('pause');
    else if (action === 'map' && screen === null && mapHost) show('map');
  }

  /** The engine calls this when the player rests at a Lantern. */
  export function handleRested(): void {
    // Travel rests again at the new Lantern; the menu is already open, so keep its place.
    if (screen === 'lantern') refresh();
    else void show('lantern');
  }

  function announce(text: string): void {
    live = '';
    setTimeout(() => (live = text), 40);
  }

  function refresh(): void {
    state = host.read();
  }

  async function show(next: Screen): Promise<void> {
    if (screen === null) {
      returnTo = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      engine.setPaused(true);
      open = true;
      stopPad = startGamepadNav(onPadMove);
    }
    refresh();
    screen = next;
    announce(`${TITLES[next]} menu`);
    await tick();
    focusFirst(screenEl);
  }

  function close(): void {
    if (screen === null) return;
    screen = null;
    open = false;
    stopPad?.();
    stopPad = null;
    engine.setPaused(false);
    returnTo?.blur?.();
    returnTo = null;
  }

  /** Back one step: out of a sub-step, then to the pause menu, then out of the menus. */
  function back(): void {
    if (child?.back?.()) return;
    if (screen === 'equipment' || screen === 'inventory' || screen === 'settings' || screen === 'map') void show('pause');
    else close();
  }

  function onPadMove(m: NavMove): void {
    if (screen === null || navState.capturing) return;
    if (m === 'back') back();
    else if (m === 'menu') close();
    else navigate(screenEl, m);
  }

  function onKey(e: KeyboardEvent): void {
    if (screen === null || navState.capturing) return;
    const typing = e.target instanceof HTMLInputElement && e.target.type === 'text';
    if (e.key === 'Tab') {
      // Keep Tab inside the dialog.
      const items = [...root.querySelectorAll<HTMLElement>('button, input, select, [tabindex="0"]')].filter(
        (el) => !el.hasAttribute('disabled') && el.offsetParent !== null
      );
      if (items.length === 0) return;
      const first = items[0];
      const last = items[items.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
      return;
    }
    if (typing) return;
    const m = moveForKey(e);
    if (m === null) return;
    e.preventDefault();
    e.stopPropagation();
    if (m === 'back') back();
    else navigate(screenEl, m);
  }

  onDestroy(() => {
    stopPad?.();
    if (screen !== null) engine.setPaused(false);
  });

  // Pause menu entries, in the order the presentation spec gives.
  $: entries = [
    { id: 'resume', label: 'Resume', note: '', off: false },
    { id: 'equipment', label: 'Equipment', note: '', off: false },
    { id: 'inventory', label: 'Inventory', note: '', off: false },
    { id: 'map', label: 'Map', note: mapHost ? '' : 'Not built yet', off: !mapHost },
    { id: 'codex', label: 'Codex', note: 'Not built yet', off: true },
    { id: 'settings', label: 'Settings', note: '', off: false },
    { id: 'plain', label: 'Read the plain site', note: '', off: false },
    {
      id: 'quit',
      label: 'Quit to title',
      note: state.bossFight ? 'Unavailable during a boss fight' : '',
      off: state.bossFight
    }
  ];

  function pick(id: string): void {
    switch (id) {
      case 'resume':
        close();
        break;
      case 'equipment':
      case 'inventory':
      case 'settings':
      case 'map':
        void show(id);
        break;
      case 'plain':
        close();
        onPlain();
        break;
      case 'quit':
        close();
        onQuit();
        break;
    }
  }
</script>

<svelte:window on:keydown|capture={onKey} />

<div class="lf-menu-live" aria-live="polite" role="status">{live}</div>

{#if screen !== null}
  <div
    class="lf-menu scrim"
    class:calm={($menuSettings).accessibility.reducedMotion}
    bind:this={root}
    style="--mt: {textScale}"
  >
    <div class="panel" role="dialog" aria-modal="true" aria-labelledby="lf-menu-title">
      <header class="head">
        <h2 id="lf-menu-title">{TITLES[screen]}</h2>
        <span class="hint" aria-hidden="true">
          <kbd>Arrows</kbd> move · <kbd>Enter</kbd> choose · <kbd>Esc</kbd> back
        </span>
      </header>
      <div class="screen" bind:this={screenEl} data-cols={COLS[screen]}>
        {#if screen === 'pause'}
          <ul class="main" aria-label="Pause menu">
            {#each entries as e (e.id)}
              <li>
                <button
                  type="button"
                  data-nav="main"
                  class="entry"
                  aria-disabled={e.off ? 'true' : undefined}
                  on:click={() => !e.off && pick(e.id)}
                >
                  <span>{e.label}</span>
                  {#if e.note}<small>{e.note}</small>{/if}
                </button>
              </li>
            {/each}
          </ul>
        {:else if screen === 'equipment'}
          <Equipment bind:this={child} {host} {state} {announce} {refresh} />
        {:else if screen === 'inventory'}
          <Inventory
            bind:this={child}
            {host}
            {state}
            {announce}
            {refresh}
            go={(s) => show(s)}
            useAndClose={() => close()}
          />
        {:else if screen === 'lantern'}
          <LanternMenu bind:this={child} {host} {mapHost} {state} {announce} {refresh} leave={close} />
        {:else if screen === 'map' && mapHost}
          <MapScreen bind:this={child} host={mapHost} {announce} />
        {:else if screen === 'settings'}
          <Settings bind:this={child} {engine} {hudSettings} {menuSettings} {announce} />
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .lf-menu-live {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  .scrim {
    position: absolute;
    inset: 0;
    z-index: 20;
    display: grid;
    place-items: center;
    padding: var(--s-4);
    background: var(--scrim);
    font-family: var(--font-sans);
    font-size: calc(var(--text-base) * var(--mt, 1));
    color: var(--ink);
  }

  .panel {
    width: min(58em, 100%);
    max-height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-md);
    box-shadow: 0 18px 48px var(--shadow-color);
    overflow: hidden;
  }

  .head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-4);
    padding: 0.75em 1.25em;
    border-bottom: 1px solid var(--line);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.375em;
    font-weight: 500;
  }

  .hint {
    font-size: 0.75em;
    color: var(--ink-3);
  }

  .hint kbd {
    font-family: var(--font-mono);
    padding: 0 4px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-xs);
    background: var(--surface-2);
  }

  .screen {
    padding: 1em 1.25em;
    overflow-y: auto;
    min-height: 0;
  }

  .main {
    list-style: none;
    margin: 0 auto;
    padding: 0;
    width: min(26em, 100%);
    display: grid;
    gap: 0.25em;
  }

  .entry {
    width: 100%;
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1em;
    text-align: left;
  }

  .entry small {
    color: var(--ink-3);
    font-size: 0.75em;
  }

  /* Shared control styles for every menu screen. */
  .lf-menu :global(button),
  .lf-menu :global(select) {
    font: inherit;
    color: var(--ink);
    background: var(--surface-2);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    padding: 0.5em 0.75em;
    cursor: pointer;
    min-height: 2.5em;
  }

  .lf-menu :global(button:hover),
  .lf-menu :global(select:hover) {
    border-color: var(--line-strong);
    background: var(--hover-wash);
  }

  .lf-menu :global(button:focus-visible),
  .lf-menu :global(input:focus-visible),
  .lf-menu :global(select:focus-visible) {
    outline: 2px solid var(--fjord);
    outline-offset: 2px;
    border-color: var(--fjord);
    background: var(--fjord-wash);
  }

  .lf-menu :global(button[aria-disabled='true']),
  .lf-menu :global(button:disabled) {
    color: var(--ink-3);
    cursor: not-allowed;
    border-style: dashed;
  }

  .lf-menu :global(.primary) {
    background: var(--fjord);
    border-color: var(--fjord-strong);
    color: var(--ink-inverse);
  }

  .lf-menu :global(.cols) {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 1.25em;
  }

  .lf-menu :global(.cols > *) {
    min-width: 0;
  }

  .lf-menu :global(.list) {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.25em;
    align-content: start;
  }

  .lf-menu :global(.list button) {
    width: 100%;
    text-align: left;
    display: flex;
    justify-content: space-between;
    gap: 0.75em;
    align-items: baseline;
  }

  .lf-menu :global(.list .sub) {
    color: var(--ink-3);
    font-size: 0.8125em;
  }

  .lf-menu :global(h3) {
    margin: 0 0 0.5em;
    font-size: 0.75em;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-3);
  }

  .lf-menu :global(.group + .group) {
    margin-top: 1em;
  }

  .lf-menu :global(.note) {
    margin: 0.5em 0 0;
    color: var(--ink-2);
    font-size: 0.875em;
    max-width: 52ch;
  }

  .lf-menu :global(.sr-only) {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  .lf-menu :global(.stats) {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 0.25em 1em;
    margin: 0;
    padding: 0.75em;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    font-size: 0.9375em;
  }

  .lf-menu :global(.stats dt) {
    color: var(--ink-2);
  }

  .lf-menu :global(.stats dd) {
    margin: 0;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .lf-menu :global(.stats .change) {
    min-width: 3.5em;
  }

  /* Gains and losses carry a sign and a word-free arrow as well as a color. */
  .lf-menu :global(.up) {
    color: var(--success);
  }

  .lf-menu :global(.down) {
    color: var(--danger);
  }

  @media (max-width: 640px) {
    .lf-menu :global(.cols) {
      grid-template-columns: 1fr;
    }

    .hint {
      display: none;
    }
  }

  @media (prefers-reduced-motion: no-preference) {
    .scrim:not(.calm) .panel {
      animation: lf-menu-in var(--dur) var(--ease-out);
    }
  }

  @keyframes lf-menu-in {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
</style>
