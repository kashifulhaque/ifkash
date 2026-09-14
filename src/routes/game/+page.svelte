<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { browser, dev } from '$app/environment';
  import { goto } from '$app/navigation';
  import { name } from '$lib/content';
  import { gameState, initialState } from '$lib/game/store';
  import { WONDERS } from '$lib/game/wonders';
  import { normalizeSeedLabel } from '$lib/game/seed';
  import { EARTH_SEED, type SpaceDestination } from '$lib/game/space';
  import type { Game, GameCallbacks } from '$lib/game/Game';
  import Hud from '$lib/game/ui/Hud.svelte';
  import WonderPanel from '$lib/game/ui/WonderPanel.svelte';
  import HelpOverlay from '$lib/game/ui/HelpOverlay.svelte';
  import TouchControls from '$lib/game/ui/TouchControls.svelte';
  import PhotoBar from '$lib/game/ui/PhotoBar.svelte';
  import JournalPanel from '$lib/game/ui/JournalPanel.svelte';

  let canvas: HTMLCanvasElement;
  let game: Game | null = null;
  let GameConstructor: typeof Game | null = null;
  let destroyed = false;
  let isTouch = false;
  let justFound = false;
  let saving = false;
  type Toast = NonNullable<(typeof initialState)['toast']>;
  /** Toast to show once the wonder panel closes, for example after the seventh find. */
  let pendingToast: Omit<Toast, 'id'> | null = null;
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  function makeCallbacks(): GameCallbacks {
    return {
      onBiome: (biome) => gameState.update((s) => ({ ...s, biome })),
      onPrompt: (prompt) => gameState.update((s) => ({ ...s, prompt })),
      onFound: (found) => {
        justFound = true;
        gameState.update((s) => ({ ...s, found }));
        if (found.length >= WONDERS.length)
          pendingToast = { text: 'Every small wonder found. A badge now waits on the home page.', link: { href: '/', label: 'See it' } };
      },
      onOpenWonder: (wonder) => {
        game?.setOverlayOpen(true);
        gameState.update((s) => ({ ...s, openWonder: wonder, help: false, journalOpen: false }));
      },
      onGlobe: (globeView) => gameState.update((s) => ({ ...s, globeView })),
      onIntroEnd: () => gameState.update((s) => ({ ...s, intro: false })),
      onHelp: () => toggleHelp(),
      onEscape: () => {
        if ($gameState.openWonder || $gameState.help || $gameState.journalOpen) closeOverlays();
        else if ($gameState.photo) game?.setPhoto(false);
      },
      onPhoto: (photo) => gameState.update((s) => ({ ...s, photo })),
      onJournal: (journal) => gameState.update((s) => ({ ...s, journal })),
      onShards: (found, total) => gameState.update((s) => ({ ...s, shards: { found, total } })),
      onMilestone: (m) => {
        const toast = { kicker: 'Milestone', text: `${m.title} · ${m.blurb}` };
        if ($gameState.openWonder || m.id === 'wonders') pendingToast = toast;
        else showToast(toast);
      },
      onJournalOpen: () => toggleJournal(),
      onSpace: (space) => gameState.update((s) => ({ ...s, space })),
      onFlight: (flying) => {
        gameState.update((s) => ({
          ...s,
          flying,
          openWonder: null,
          help: false,
          journalOpen: false
        }));
      },
      onArrive: (destination) => moveToPlanet(destination, true),
      onSpaceNotice: (notice) => showToast(notice)
    };
  }

  function locationPlanet(): { seed: string; depth: number } {
    const params = new URLSearchParams(window.location.search);
    const seed = normalizeSeedLabel(params.get('seed')) ?? EARTH_SEED;
    const rawDepth = Number(params.get('depth'));
    const depth = seed === EARTH_SEED ? 0 : Number.isInteger(rawDepth) && rawDepth > 0 ? rawDepth : 1;
    return { seed, depth };
  }

  function updatePlanetUrl(seed: string, depth: number, mode: 'push' | 'replace'): void {
    const url = new URL(window.location.href);
    if (seed === EARTH_SEED) {
      url.searchParams.delete('seed');
      url.searchParams.delete('depth');
    } else {
      url.searchParams.set('seed', seed);
      url.searchParams.set('depth', String(depth));
    }
    window.history[mode === 'push' ? 'pushState' : 'replaceState']({}, '', `${url.pathname}${url.search}${url.hash}`);
  }

  function createPlanet(seed: string, depth: number, historyMode: 'push' | 'replace' | null): void {
    if (!GameConstructor || destroyed || !canvas) return;
    game?.dispose();
    game = null;
    const retainedToast = $gameState.toast;
    gameState.set({
      ...initialState,
      isTouch,
      total: WONDERS.length,
      seed,
      toast: retainedToast
    });
    if (historyMode) updatePlanetUrl(seed, depth, historyMode);
    try {
      const nextGame = new GameConstructor(canvas, makeCallbacks(), { seed, depth });
      game = nextGame;
      gameState.update((s) => ({
        ...s,
        ready: true,
        found: nextGame.found,
        muted: nextGame.audio.muted,
        journal: nextGame.journal,
        shards: nextGame.shardProgress,
        space: nextGame.spaceStatus
      }));
      if (dev) (window as unknown as Record<string, unknown>).__game = nextGame;
    } catch (err) {
      console.error('WebGL init failed', err);
      gameState.update((s) => ({ ...s, webglFailed: true }));
      goto('/');
    }
  }

  onMount(async () => {
    if (!browser) return;
    isTouch = window.matchMedia('(pointer: coarse)').matches || 'ontouchstart' in window;
    const mod = await import('$lib/game/Game');
    if (destroyed || !canvas) return;
    GameConstructor = mod.Game;
    const current = locationPlanet();
    createPlanet(current.seed, current.depth, null);
    window.addEventListener('popstate', handlePopState);
  });

  onDestroy(() => {
    destroyed = true;
    clearTimeout(toastTimer);
    if (browser) window.removeEventListener('popstate', handlePopState);
    game?.dispose();
    game = null;
    gameState.set({ ...initialState });
  });

  function handlePopState(): void {
    if (!GameConstructor) return;
    gameState.update((s) => ({ ...s, flying: false }));
    const current = locationPlanet();
    createPlanet(current.seed, current.depth, null);
  }

  function closeOverlays(): void {
    justFound = false;
    gameState.update((s) => ({
      ...s,
      openWonder: null,
      help: false,
      journalOpen: false
    }));
    game?.setOverlayOpen(false);
    if (pendingToast) {
      showToast(pendingToast);
      pendingToast = null;
    }
  }

  function showToast(toast: Omit<Toast, 'id'>): void {
    clearTimeout(toastTimer);
    const id = Date.now();
    gameState.update((s) => ({ ...s, toast: { id, ...toast } }));
    toastTimer = setTimeout(() => gameState.update((s) => (s.toast?.id === id ? { ...s, toast: null } : s)), 7000);
  }

  function toggleJournal(): void {
    if ($gameState.journalOpen) {
      closeOverlays();
      return;
    }
    game?.setOverlayOpen(true);
    gameState.update((s) => ({ ...s, journalOpen: true, help: false, openWonder: null }));
  }

  /** Photo mode: render a frame and download it as a PNG named after the seed. */
  async function savePhoto(): Promise<void> {
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

  function toggleHelp(): void {
    if ($gameState.help) {
      closeOverlays();
      return;
    }
    game?.setOverlayOpen(true);
    gameState.update((s) => ({ ...s, help: true, openWonder: null, journalOpen: false }));
  }

  function toggleMute(): void {
    if (!game) return;
    const muted = game.toggleMute();
    gameState.update((s) => ({ ...s, muted }));
  }

  function interactFromHud(): void {
    if (!game) return;
    game.startAudio();
    game.input.queueInteract();
  }

  function moveToPlanet(destination: SpaceDestination, flown: boolean): void {
    if (destroyed) return;
    createPlanet(destination.seed, destination.depth, 'push');
    showToast({
      kicker: flown ? `Arrived · depth ${destination.depth}` : 'Emergency recall',
      text: flown ? `${destination.name} · ${destination.kind}` : 'Teleported safely back to Earth'
    });
  }

  function returnToEarth(): void {
    if ($gameState.space.isEarth) return;
    moveToPlanet({ seed: EARTH_SEED, name: 'Earth', kind: 'Homeworld', depth: 0, fuelCost: 0, color: 0x256a8c }, false);
  }
</script>

<svelte:head>
  <title>{name} · {$gameState.space.planetName}</title>
  <meta
    name="description"
    content="Explore procedural low-poly planets, recover starship parts, craft a ship, gather fuel, and journey endlessly through space."
  />
</svelte:head>

<div class="game-root">
  {#if !$gameState.webglFailed}
    <canvas bind:this={canvas} class="game-canvas" class:grab={$gameState.ready && !$gameState.flying} class:flight={$gameState.flying}></canvas>

    {#if !$gameState.ready}
      <div class="loading" aria-live="polite">
        <div class="orbit"><span></span></div>
        <p>Shaping a small world…</p>
      </div>
    {/if}

    {#if $gameState.ready && !$gameState.photo && !$gameState.flying}
      <Hud
        found={$gameState.found.length}
        total={$gameState.total}
        shards={$gameState.shards}
        space={$gameState.space}
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
        on:journal={toggleJournal}
        on:interact={interactFromHud}
        on:refuel={() => game?.refuelShip()}
        on:launch={() => game?.launchFlight()}
        on:earth={returnToEarth}
      />
    {/if}
    {#if $gameState.ready && $gameState.flying && !$gameState.photo && !$gameState.help && !$gameState.journalOpen}
      <div class="flight-hud" aria-live="polite">
        <div class="flight-status">
          <span>Direct flight</span>
          <strong>{$gameState.space.fuel} fuel</strong>
        </div>
        <button class="land" on:click={() => game?.landFlight()}>Return to {$gameState.space.planetName}</button>
        <div class="crosshair" aria-hidden="true"><span></span></div>
        <p class="flight-hint">
          {#if $gameState.isTouch}
            Steer with the stick · boost with the rocket · approach a terrain world to land
          {:else}
            <kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> steer · <kbd>Shift</kbd> boost · <kbd>Space</kbd> return · approach a terrain world to land
          {/if}
        </p>
      </div>
    {/if}

    {#if $gameState.ready && $gameState.photo}
      <PhotoBar isTouch={$gameState.isTouch} {saving} on:exit={() => game?.setPhoto(false)} on:save={savePhoto} />
    {/if}

    {#if $gameState.toast}
      {#key $gameState.toast.id}
        <div class="toast" role="status">
          <span class="spark">✦</span>
          <span class="toast-text">
            {#if $gameState.toast.kicker}<span class="toast-kicker">{$gameState.toast.kicker}</span>{/if}
            <span>{$gameState.toast.text}</span>
          </span>
          {#if $gameState.toast.link}
            <a href={$gameState.toast.link.href}>{$gameState.toast.link.label}</a>
          {/if}
        </div>
      {/key}
    {/if}

    {#if $gameState.ready && $gameState.isTouch && !$gameState.openWonder && !$gameState.help && !$gameState.journalOpen && !$gameState.photo}
      <TouchControls
        flying={$gameState.flying}
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
      <HelpOverlay
        isTouch={$gameState.isTouch}
        found={$gameState.found.length}
        total={$gameState.total}
        seed={$gameState.seed}
        depth={$gameState.space.depth}
        on:close={closeOverlays}
      />
    {/if}

    {#if $gameState.journalOpen}
      <JournalPanel journal={$gameState.journal} found={$gameState.found} shards={$gameState.shards} seed={$gameState.seed} on:close={closeOverlays} />
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
    /* The rest of the site inherits Drawably Pen from <body>; the planet keeps
       its own type system, so re-anchor inheritance here rather than relying on
       every descendant to set a font. */
    font-family: var(--planet-sans);
  }

  /* app.css sets a font-family on p and h1–h6 by element, which outranks plain
     inheritance from .game-root — so the planet has to claim those tags back or
     its copy renders in the site's pen face. Components that ask for
     --planet-serif or mono still win on specificity. */
  .game-root :global(p),
  .game-root :global(h1),
  .game-root :global(h2),
  .game-root :global(h3),
  .game-root :global(h4),
  .game-root :global(h5),
  .game-root :global(h6) {
    font-family: var(--planet-sans);
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
  .game-canvas.flight {
    cursor: crosshair;
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
  .toast-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .toast-kicker {
    font-size: 0.6rem;
    letter-spacing: 0.2em;
    text-transform: uppercase;
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

  .flight-hud {
    position: absolute;
    inset: 0;
    z-index: 18;
    pointer-events: none;
    color: #f2efe6;
    text-shadow: 0 2px 12px rgba(5, 9, 20, 0.9);
  }
  .flight-status {
    position: absolute;
    top: 22px;
    left: 24px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 10px 14px;
    border: 1px solid rgba(126, 219, 209, 0.38);
    border-radius: 12px;
    background: rgba(6, 15, 24, 0.68);
    backdrop-filter: blur(8px);
  }
  .flight-status span {
    color: #7edbd1;
    font-size: 0.6rem;
    letter-spacing: 0.18em;
    text-transform: uppercase;
  }
  .flight-status strong {
    font-size: 0.82rem;
    font-weight: 600;
  }
  .land {
    position: absolute;
    top: 22px;
    right: 24px;
    padding: 10px 14px;
    border: 1px solid rgba(242, 239, 230, 0.3);
    border-radius: 999px;
    background: rgba(6, 15, 24, 0.68);
    backdrop-filter: blur(8px);
    color: #f2efe6;
    font-size: 0.7rem;
    letter-spacing: 0.06em;
    pointer-events: auto;
  }
  .land:hover {
    border-color: rgba(233, 196, 106, 0.75);
    color: #e9c46a;
  }
  .crosshair {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 34px;
    height: 34px;
    transform: translate(-50%, -50%);
    border: 1px solid rgba(242, 239, 230, 0.62);
    border-radius: 50%;
    box-shadow: 0 0 14px rgba(126, 219, 209, 0.35);
  }
  .crosshair::before,
  .crosshair::after,
  .crosshair span::before,
  .crosshair span::after {
    position: absolute;
    content: '';
    background: rgba(242, 239, 230, 0.72);
  }
  .crosshair::before,
  .crosshair::after {
    top: 50%;
    width: 9px;
    height: 1px;
  }
  .crosshair::before {
    right: 100%;
  }
  .crosshair::after {
    left: 100%;
  }
  .crosshair span::before,
  .crosshair span::after {
    left: 50%;
    width: 1px;
    height: 9px;
  }
  .crosshair span::before {
    bottom: 100%;
  }
  .crosshair span::after {
    top: 100%;
  }
  .flight-hint {
    position: absolute;
    bottom: 24px;
    left: 50%;
    max-width: calc(100% - 32px);
    margin: 0;
    padding: 8px 13px;
    transform: translateX(-50%);
    border-radius: 999px;
    background: rgba(6, 15, 24, 0.62);
    color: rgba(242, 239, 230, 0.82);
    font-size: 0.68rem;
    letter-spacing: 0.04em;
    text-align: center;
    white-space: nowrap;
  }
  .flight-hint kbd {
    display: inline-grid;
    min-width: 18px;
    height: 18px;
    margin-inline: 1px;
    place-items: center;
    border: 1px solid rgba(242, 239, 230, 0.3);
    border-radius: 4px;
    background: rgba(242, 239, 230, 0.08);
    font: 0.6rem var(--planet-sans);
  }
  @media (max-width: 640px) {
    .flight-status {
      top: 14px;
      left: 14px;
    }
    .land {
      top: 14px;
      right: 14px;
    }
    .flight-hint {
      bottom: 156px;
      white-space: normal;
    }
  }
  .noscript {
    padding: 40px;
    font-family: var(--font-mono);
    color: #f2efe6;
  }
</style>
