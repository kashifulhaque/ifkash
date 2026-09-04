<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { browser, dev } from '$app/environment';
  import { goto } from '$app/navigation';
  import { name } from '$lib/content';
  import { gameState, initialState } from '$lib/game/store';
  import { WONDERS } from '$lib/game/wonders';
  import { resolveSeedLabel } from '$lib/game/seed';
  import type { Game, GameCallbacks } from '$lib/game/Game';
  import Hud from '$lib/game/ui/Hud.svelte';
  import WonderPanel from '$lib/game/ui/WonderPanel.svelte';
  import HelpOverlay from '$lib/game/ui/HelpOverlay.svelte';
  import TouchControls from '$lib/game/ui/TouchControls.svelte';
  import PhotoBar from '$lib/game/ui/PhotoBar.svelte';

  let canvas: HTMLCanvasElement;
  let game: Game | null = null;
  let destroyed = false;
  let justFound = false;
  let saving = false;
  /** Toast to show once the wonder panel closes, for example after the seventh find. */
  let pendingToast: string | null = null;
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  function makeCallbacks(): GameCallbacks {
    return {
      onBiome: (biome) => gameState.update((s) => ({ ...s, biome })),
      onPrompt: (prompt) => gameState.update((s) => ({ ...s, prompt })),
      onFound: (found) => {
        justFound = true;
        gameState.update((s) => ({ ...s, found }));
        if (found.length >= WONDERS.length) pendingToast = 'Every small wonder found. A badge now waits on the home page.';
      },
      onOpenWonder: (wonder) => {
        game?.setOverlayOpen(true);
        gameState.update((s) => ({ ...s, openWonder: wonder, help: false }));
      },
      onGlobe: (globeView) => gameState.update((s) => ({ ...s, globeView })),
      onIntroEnd: () => gameState.update((s) => ({ ...s, intro: false })),
      onHelp: () => toggleHelp(),
      onEscape: () => {
        if ($gameState.openWonder || $gameState.help) closeOverlays();
        else if ($gameState.photo) game?.setPhoto(false);
      },
      onPhoto: (photo) => gameState.update((s) => ({ ...s, photo }))
    };
  }

  onMount(async () => {
    if (!browser) return;
    const isTouch = window.matchMedia('(pointer: coarse)').matches || 'ontouchstart' in window;
    const seed = resolveSeedLabel(window.location.search);
    gameState.update((s) => ({ ...s, isTouch, total: WONDERS.length, seed }));
    const mod = await import('$lib/game/Game');
    // The layout's full-width branch can remount this page mid-import.
    if (destroyed || !canvas) return;
    try {
      game = new mod.Game(canvas, makeCallbacks(), { seed });
    } catch (err) {
      console.error('WebGL init failed', err);
      gameState.update((s) => ({ ...s, webglFailed: true }));
      goto('/');
      return;
    }
    gameState.update((s) => ({ ...s, ready: true, found: game!.found, muted: game!.audio.muted }));
    if (dev) (window as unknown as Record<string, unknown>).__game = game;
  });

  onDestroy(() => {
    destroyed = true;
    clearTimeout(toastTimer);
    game?.dispose();
    game = null;
    gameState.set({ ...initialState });
  });

  function closeOverlays() {
    justFound = false;
    gameState.update((s) => ({ ...s, openWonder: null, help: false }));
    game?.setOverlayOpen(false);
    if (pendingToast) {
      showToast(pendingToast);
      pendingToast = null;
    }
  }

  function showToast(text: string) {
    clearTimeout(toastTimer);
    const id = Date.now();
    gameState.update((s) => ({ ...s, toast: { id, text } }));
    toastTimer = setTimeout(() => gameState.update((s) => (s.toast?.id === id ? { ...s, toast: null } : s)), 7000);
  }

  /** Photo mode: render a frame and download it as a PNG named after the seed. */
  async function savePhoto() {
    if (!game || saving) return;
    saving = true;
    try {
      const blob = await game.snapshot();
      if (!blob) return;
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `tiny-planet-${game.seed}.png`;
      a.click();
      setTimeout(() => URL.revokeObjectURL(url), 2000);
    } finally {
      saving = false;
    }
  }

  function toggleHelp() {
    if ($gameState.help) {
      closeOverlays();
      return;
    }
    game?.setOverlayOpen(true);
    gameState.update((s) => ({ ...s, help: true, openWonder: null }));
  }

  function toggleMute() {
    if (!game) return;
    const muted = game.toggleMute();
    gameState.update((s) => ({ ...s, muted }));
  }

  function interactFromHud() {
    if (!game) return;
    game.startAudio();
    game.input.queueInteract();
  }
</script>

<svelte:head>
  <title>{name} · Tiny Planet</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, ML Engineer. Wander a tiny low-poly planet and find seven small wonders that hold the portfolio."
  />
</svelte:head>

<div class="game-root">
  {#if !$gameState.webglFailed}
    <canvas bind:this={canvas} class="game-canvas" class:grab={$gameState.ready}></canvas>

    {#if !$gameState.ready}
      <div class="loading" aria-live="polite">
        <div class="orbit"><span></span></div>
        <p>Shaping a small world…</p>
      </div>
    {/if}

    {#if $gameState.ready && !$gameState.photo}
      <Hud
        found={$gameState.found.length}
        total={$gameState.total}
        biome={$gameState.biome}
        prompt={$gameState.prompt}
        muted={$gameState.muted}
        globeView={$gameState.globeView}
        intro={$gameState.intro}
        isTouch={$gameState.isTouch}
        on:globe={() => game?.toggleGlobe()}
        on:mute={toggleMute}
        on:help={toggleHelp}
        on:photo={() => game?.togglePhoto()}
        on:interact={interactFromHud}
      />
    {/if}

    {#if $gameState.ready && $gameState.photo}
      <PhotoBar isTouch={$gameState.isTouch} {saving} on:exit={() => game?.setPhoto(false)} on:save={savePhoto} />
    {/if}

    {#if $gameState.toast}
      {#key $gameState.toast.id}
        <div class="toast" role="status">
          <span class="spark">✦</span>
          <span>{$gameState.toast.text}</span>
          <a href="/">See it</a>
        </div>
      {/key}
    {/if}

    {#if $gameState.ready && $gameState.isTouch && !$gameState.openWonder && !$gameState.help && !$gameState.photo}
      <TouchControls
        on:move={(e) => {
          if (!game) return;
          game.input.touchMove = e.detail;
          if (e.detail.x !== 0 || e.detail.y !== 0) game.input.notifyTouchMove();
        }}
        on:hop={() => game?.input.queueHop()}
        on:run={(e) => game && (game.input.touchRun = e.detail.active)}
      />
    {/if}

    {#if $gameState.openWonder}
      <WonderPanel wonder={$gameState.openWonder} {justFound} on:close={closeOverlays} />
    {/if}

    {#if $gameState.help}
      <HelpOverlay isTouch={$gameState.isTouch} found={$gameState.found.length} total={$gameState.total} seed={$gameState.seed} on:close={closeOverlays} />
    {/if}
  {/if}

  <noscript>
    <div class="noscript">
      <p>This page is a small browser game and needs JavaScript.</p>
      <p>Kashiful Haque — ML Engineer. github.com/kashifulhaque · linkedin.com/in/kashifulhaque</p>
    </div>
  </noscript>
</div>

<style>
  @font-face {
    font-family: 'Cardo';
    src: url('/fonts/Cardo-Regular.ttf') format('truetype');
    font-weight: 400;
    font-style: normal;
    font-display: swap;
  }

  .game-root {
    position: fixed;
    inset: 0;
    width: 100vw;
    height: 100dvh;
    overflow: hidden;
    background: #0d1c28;
    --planet-serif: 'Cardo', Georgia, 'Times New Roman', serif;
    --planet-sans: 'Inter Variable', Inter, ui-sans-serif, system-ui, sans-serif;
  }

  .game-root :global(button) {
    font-family: var(--planet-sans);
    cursor: pointer;
  }

  .game-canvas {
    width: 100%;
    height: 100%;
    display: block;
    touch-action: none;
    cursor: default;
  }
  .game-canvas.grab {
    cursor: grab;
  }
  .game-canvas.grab:active {
    cursor: grabbing;
  }

  .loading {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 18px;
    color: rgba(242, 239, 230, 0.7);
    font-family: var(--planet-sans);
    font-size: 0.8rem;
    letter-spacing: 0.08em;
  }
  .orbit {
    width: 54px;
    height: 54px;
    border-radius: 50%;
    border: 1px solid rgba(242, 239, 230, 0.25);
    position: relative;
    animation: spin 1.6s linear infinite;
  }
  .orbit span {
    position: absolute;
    top: -4px;
    left: 50%;
    width: 8px;
    height: 8px;
    margin-left: -4px;
    border-radius: 50%;
    background: #e9c46a;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .toast {
    position: absolute;
    top: 68px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 25;
    display: flex;
    align-items: center;
    gap: 10px;
    max-width: calc(100% - 32px);
    padding: 10px 16px;
    border-radius: 12px;
    background: rgba(18, 40, 46, 0.82);
    border: 1px solid rgba(233, 196, 106, 0.4);
    backdrop-filter: blur(10px);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
    color: #f2efe6;
    font-family: var(--planet-sans);
    font-size: 0.8rem;
    animation: toast-in 0.4s ease-out, toast-out 0.6s ease-in 6.4s forwards;
  }
  .toast .spark {
    color: #e9c46a;
  }
  .toast a {
    color: #e9c46a;
    text-decoration: underline;
    text-underline-offset: 3px;
    white-space: nowrap;
  }
  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translate(-50%, -8px);
    }
    to {
      opacity: 1;
      transform: translateX(-50%);
    }
  }
  @keyframes toast-out {
    to {
      opacity: 0;
    }
  }

  .noscript {
    padding: 40px;
    font-family: var(--font-mono);
    color: #f2efe6;
  }
</style>
