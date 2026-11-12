<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { name } from '$lib/content';
  import type { Engine, EngineStats } from '$lib/latticefall/engine';

  /** Show the dev readout (tick, fps, state hash). Set with `&dev=1`. */
  export let showDev = false;

  let canvas: HTMLCanvasElement;
  let engine: Engine | null = null;
  let destroyed = false;
  let status: 'loading' | 'ready' | 'failed' = 'loading';
  let failure = '';
  let stats: EngineStats = { tick: 0, fps: 0, hash: '', paused: false };
  let paused = false;

  onMount(async () => {
    if (!browser) return;
    try {
      // Dynamic import keeps Three.js and the WASM glue out of every other route.
      const { startEngine } = await import('$lib/latticefall/engine');
      if (destroyed) return;
      const started = await startEngine(canvas, { onStats: (s) => (stats = s) });
      if (destroyed) {
        started.dispose();
        return;
      }
      engine = started;
      status = 'ready';
    } catch (err) {
      console.error('Latticefall engine failed to start', err);
      failure = err instanceof Error ? err.message : String(err);
      status = 'failed';
    }
  });

  onDestroy(() => {
    destroyed = true;
    engine?.dispose();
    engine = null;
  });

  function togglePause(): void {
    paused = !paused;
    engine?.setPaused(paused);
  }

  function onKey(e: KeyboardEvent): void {
    if (e.code === 'KeyP' && !e.repeat && !e.ctrlKey && !e.metaKey && !e.altKey) togglePause();
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
    <p class="lf-hint"><kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> move · <kbd>Shift</kbd> run · <kbd>P</kbd> pause</p>
  {/if}

  {#if showDev && status === 'ready'}
    <dl class="lf-dev" aria-label="Developer readout">
      <dt>tick</dt><dd>{stats.tick}</dd>
      <dt>fps</dt><dd>{stats.fps}</dd>
      <dt>hash</dt><dd>{stats.hash}</dd>
      {#if paused}<dt>state</dt><dd>paused</dd>{/if}
    </dl>
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
    bottom: var(--s-5);
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

  .lf-dev dt {
    color: var(--ink-3);
  }

  .lf-dev dd {
    margin: 0;
  }
</style>
