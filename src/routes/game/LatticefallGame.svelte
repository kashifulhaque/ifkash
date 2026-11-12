<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { name } from '$lib/content';
  import type { Engine, EngineStats } from '$lib/latticefall/engine';
  import Hud from '$lib/latticefall/ui/hud/Hud.svelte';
  import { createHudController } from '$lib/latticefall/ui/hud/hudStore';
  import { createHudSettings } from '$lib/latticefall/ui/hud/hudSettings';
  import { DEMO_NOTICES, demoHud } from '$lib/latticefall/ui/hud/hudDemo';
  import { codex } from '$lib/latticefall/codex/store';
  import { unlockFromEvents } from '$lib/latticefall/codex/pickups';
  import Ledger from '$lib/latticefall/ui/portfolio/Ledger.svelte';
  import UnlockToast from '$lib/latticefall/ui/portfolio/UnlockToast.svelte';
  import Menus from '$lib/latticefall/ui/menus/Menus.svelte';
  import { createMenuSettings } from '$lib/latticefall/ui/menus/menuSettings';

  /** Show the dev readout (tick, fps, state hash). Set with `&dev=1`. */
  export let showDev = false;

  let canvas: HTMLCanvasElement;
  let engine: Engine | null = null;
  let destroyed = false;
  let status: 'loading' | 'ready' | 'failed' = 'loading';
  let failure = '';
  let stats: EngineStats = { tick: 0, fps: 0, hash: '', paused: false };
  let paused = false;

  // The HUD reads the world through the engine; `&hud-demo=1` shows a fixed full HUD instead.
  const hud = createHudController();
  const hudSettings = createHudSettings(browser ? window.location.search : '');
  // The menus open on Escape or Start, and when the player rests at a Lantern (WP-046).
  const menuSettings = createMenuSettings();
  let menus: Menus | undefined;
  let menuOpen = false;
  let demoTimer: ReturnType<typeof setInterval> | undefined;
  const hudDemo = browser && new URLSearchParams(window.location.search).get('hud-demo') === '1';

  // WP-049: pickups unlock codex entries (the core decides; the store records), and the
  // Wayfarer's Ledger (`L`) shows the résumé, contact and what has been found.
  let ledgerOpen = false;
  let toast: UnlockToast;
  const hudSink = {
    ...hud.sink,
    events(words: ArrayLike<number>) {
      hud.sink.events(words);
      for (const entry of unlockFromEvents(words, codex)) toast?.show(entry);
    }
  };

  onMount(async () => {
    if (!browser) return;
    try {
      // Dynamic import keeps Three.js and the WASM glue out of every other route.
      const { startEngine } = await import('$lib/latticefall/engine');
      if (destroyed) return;
      const started = await startEngine(canvas, {
        onStats: (s) => (stats = s),
        hud: hudDemo ? undefined : hudSink,
        menu: { ui: (a) => menus?.handleUi(a), rested: () => menus?.handleRested() }
      });
      if (destroyed) {
        started.dispose();
        return;
      }
      engine = started;
      status = 'ready';
      if (hudDemo) {
        hud.sink.state(demoHud());
        const show = () => DEMO_NOTICES.forEach((text) => hud.notify(text));
        show();
        demoTimer = setInterval(show, 2500); // Keeps the notifications on screen for screenshots.
        hud.sink.lock({ x: window.innerWidth * 0.5, y: window.innerHeight * 0.42 });
      }
    } catch (err) {
      console.error('Latticefall engine failed to start', err);
      failure = err instanceof Error ? err.message : String(err);
      status = 'failed';
    }
  });

  onDestroy(() => {
    destroyed = true;
    hud.dispose();
    clearInterval(demoTimer);
    engine?.dispose();
    engine = null;
  });

  function togglePause(): void {
    paused = !paused;
    engine?.setPaused(paused);
  }

  function downloadReplay(): void {
    try {
      engine?.downloadReplay();
    } catch (err) {
      console.error('Replay export failed', err);
    }
  }

  function onKey(e: KeyboardEvent): void {
    if (menuOpen) return; // The menus handle their own keys.
    if (e.code === 'KeyP' && !e.repeat && !e.ctrlKey && !e.metaKey && !e.altKey) togglePause();
    if (e.code === 'KeyL' && !e.repeat && !e.ctrlKey && !e.metaKey && !e.altKey) ledgerOpen = !ledgerOpen;
  }
</script>

<svelte:head>
  <title>{name} · Latticefall</title>
</svelte:head>

<svelte:window on:keydown={onKey} />

<div class="lf-root">
  <canvas bind:this={canvas} class="lf-canvas"></canvas>

  {#if status === 'loading'}
    <p class="lf-note" aria-live="polite">Loading Latticefall…</p>
  {:else if status === 'failed'}
    <p class="lf-note" role="alert">Latticefall could not start: {failure}</p>
  {:else}
    <p class="lf-hint"><kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> move · click to capture the mouse · <kbd>P</kbd> pause · <kbd>L</kbd> Ledger</p>
  {/if}

  {#if status === 'ready'}
    <Hud {hud} settings={hudSettings} />
    <UnlockToast bind:this={toast} />
    {#if ledgerOpen}<Ledger on:close={() => (ledgerOpen = false)} />{/if}
  {/if}

  {#if status === 'ready' && engine}
    <Menus
      bind:this={menus}
      bind:open={menuOpen}
      host={engine.menus}
      {engine}
      {hudSettings}
      {menuSettings}
      onPlain={() => goto('/')}
      onQuit={() => goto('/')}
    />
  {/if}

  {#if showDev && status === 'ready'}
    <dl class="lf-dev" aria-label="Developer readout">
      <dt>tick</dt><dd>{stats.tick}</dd>
      <dt>fps</dt><dd>{stats.fps}</dd>
      <dt>hash</dt><dd>{stats.hash}</dd>
      {#if paused}<dt>state</dt><dd>paused</dd>{/if}
    </dl>
    <button type="button" class="lf-replay" on:click={downloadReplay}>Download replay</button>
  {/if}
</div>

<style>
  .lf-root {
    position: fixed;
    inset: 0;
    width: 100vw;
    height: 100dvh;
    overflow: hidden;
    background: var(--bg);
    color: var(--ink);
  }

  .lf-canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  .lf-note,
  .lf-hint {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    margin: 0;
    padding: var(--s-2) var(--s-4);
    background: var(--surface);
    border: 1px solid var(--line);
    color: var(--ink-2);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
  }

  .lf-note {
    top: 50%;
  }

  .lf-hint {
    top: var(--s-4);
  }

  kbd {
    font-family: var(--font-mono);
    margin-right: 2px;
  }

  .lf-dev {
    position: absolute;
    top: var(--s-4);
    left: var(--s-4);
    display: grid;
    grid-template-columns: auto auto;
    gap: 2px var(--s-3);
    margin: 0;
    padding: var(--s-2) var(--s-3);
    background: var(--surface);
    border: 1px solid var(--line);
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .lf-replay {
    position: absolute;
    top: var(--s-4);
    right: var(--s-4);
    padding: var(--s-2) var(--s-3);
    background: var(--surface);
    border: 1px solid var(--line-strong);
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    cursor: pointer;
  }

  .lf-replay:hover {
    border-color: var(--ink-3);
  }

  .lf-dev dt {
    color: var(--ink-3);
  }

  .lf-dev dd {
    margin: 0;
  }
</style>
