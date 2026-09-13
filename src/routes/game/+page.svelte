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
  import NavigationPanel from '$lib/game/ui/NavigationPanel.svelte';

  let canvas: HTMLCanvasElement;
  let game: Game | null = null;
  let GameConstructor: typeof Game | null = null;
  let destroyed = false;
  let isTouch = false;
  let justFound = false;
  let saving = false;
  let travellingTo: SpaceDestination | null = null;
  $: travelColor = travellingTo ? `#${(travellingTo.color & 0xffffff).toString(16).padStart(6, '0')}` : '#256a8c';
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
        gameState.update((s) => ({ ...s, openWonder: wonder, help: false, journalOpen: false, navigationOpen: false }));
      },
      onGlobe: (globeView) => gameState.update((s) => ({ ...s, globeView })),
      onIntroEnd: () => gameState.update((s) => ({ ...s, intro: false })),
      onHelp: () => toggleHelp(),
      onEscape: () => {
        if ($gameState.openWonder || $gameState.help || $gameState.journalOpen || $gameState.navigationOpen) closeOverlays();
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
      onNavigate: (destinations) => {
        game?.setOverlayOpen(true);
        gameState.update((s) => ({
          ...s,
          destinations,
          navigationOpen: true,
          openWonder: null,
          help: false,
          journalOpen: false
        }));
      },
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
    travellingTo = null;
    const current = locationPlanet();
    createPlanet(current.seed, current.depth, null);
  }

  function closeOverlays(): void {
    justFound = false;
    gameState.update((s) => ({
      ...s,
      openWonder: null,
      help: false,
      journalOpen: false,
      navigationOpen: false,
      destinations: []
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
    gameState.update((s) => ({ ...s, journalOpen: true, help: false, openWonder: null, navigationOpen: false }));
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
    gameState.update((s) => ({ ...s, help: true, openWonder: null, journalOpen: false, navigationOpen: false }));
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

  async function moveToPlanet(destination: SpaceDestination, spendFuel: boolean): Promise<void> {
    if (!game || travellingTo) return;
    if (spendFuel && !game.travelTo(destination)) {
      showToast({ kicker: 'Flight computer', text: 'Not enough fuel for that route' });
      return;
    }
    game.setOverlayOpen(true);
    travellingTo = destination;
    gameState.update((s) => ({ ...s, navigationOpen: false, destinations: [] }));
    await new Promise((resolve) => setTimeout(resolve, 950));
    if (destroyed) return;
    createPlanet(destination.seed, destination.depth, 'push');
    travellingTo = null;
    showToast({
      kicker: spendFuel ? `Arrived · depth ${destination.depth}` : 'Emergency recall',
      text: spendFuel ? `${destination.name} · ${destination.kind}` : 'Teleported safely back to Earth'
    });
  }

  function returnToEarth(): void {
    if ($gameState.space.isEarth) return;
    void moveToPlanet(
      { seed: EARTH_SEED, name: 'Earth', kind: 'Homeworld', depth: 0, fuelCost: 0, color: 0x256a8c },
      false
    );
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
        on:navigation={() => game?.openNavigation()}
        on:earth={returnToEarth}
      />
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

    {#if $gameState.ready && $gameState.isTouch && !$gameState.openWonder && !$gameState.help && !$gameState.journalOpen && !$gameState.navigationOpen && !$gameState.photo && !travellingTo}
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

    {#if $gameState.navigationOpen}
      <NavigationPanel
        space={$gameState.space}
        destinations={$gameState.destinations}
        on:travel={(event) => void moveToPlanet(event.detail, true)}
        on:earth={returnToEarth}
        on:close={closeOverlays}
      />
    {/if}

    {#if travellingTo}
      <div class="warp" style:--destination={travelColor} role="status" aria-live="assertive">
        <div class="streaks" aria-hidden="true"></div>
        <svg class="warp-ship" viewBox="0 0 120 80" aria-hidden="true">
          <path d="M60 5 76 44 111 63 72 60 60 76 48 60 9 63 44 44Z" />
          <ellipse cx="60" cy="35" rx="9" ry="16" />
          <path class="flame" d="m53 61 7 16 7-16" />
        </svg>
        <div class="warp-copy">
          <p>{travellingTo.depth === 0 ? 'Emergency recall' : `Jumping to depth ${travellingTo.depth}`}</p>
          <h2>{travellingTo.name}</h2>
          <span>{travellingTo.kind}</span>
        </div>
      </div>
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

  .warp {
    position: absolute;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    overflow: hidden;
    color: #f2efe6;
    background:
      radial-gradient(circle at 50% 45%, color-mix(in srgb, var(--destination) 34%, transparent), transparent 18%),
      #050914;
    animation: warp-arrive 0.95s ease-in-out both;
  }
  .streaks,
  .streaks::before,
  .streaks::after {
    position: absolute;
    inset: -70%;
    content: '';
    background-image:
      radial-gradient(circle, rgba(255, 255, 255, 0.9) 0 1px, transparent 1.5px),
      radial-gradient(circle, rgba(126, 219, 209, 0.75) 0 1px, transparent 1.5px);
    background-position: 0 0, 31px 43px;
    background-size: 67px 79px, 97px 113px;
    transform: perspective(280px) rotateX(62deg) scale(0.35);
    animation: star-rush 0.42s linear infinite;
  }
  .streaks::before {
    transform: rotate(41deg);
  }
  .streaks::after {
    transform: rotate(-37deg);
  }
  .warp-ship {
    position: relative;
    width: min(34vw, 190px);
    overflow: visible;
    fill: #d8cfbd;
    stroke: #f2efe6;
    stroke-width: 1.2;
    filter: drop-shadow(0 0 22px color-mix(in srgb, var(--destination) 65%, white));
    animation: ship-launch 0.95s cubic-bezier(0.3, 0, 0.6, 1) both;
  }
  .warp-ship ellipse {
    fill: color-mix(in srgb, var(--destination) 70%, #dffcff);
  }
  .warp-ship .flame {
    fill: #e9c46a;
    stroke: #fff2c4;
  }
  .warp-copy {
    position: absolute;
    bottom: max(13vh, 64px);
    z-index: 1;
    text-align: center;
    text-shadow: 0 2px 16px #050914;
  }
  .warp-copy p,
  .warp-copy span {
    margin: 0;
    color: rgba(242, 239, 230, 0.68);
    font-size: 0.68rem;
    letter-spacing: 0.18em;
    text-transform: uppercase;
  }
  .warp-copy h2 {
    margin: 7px 0 5px;
    font-family: var(--planet-serif);
    font-size: clamp(1.7rem, 5vw, 2.7rem);
    font-weight: 400;
  }
  @keyframes star-rush {
    from {
      transform: perspective(280px) rotateX(62deg) translateY(-8%) scale(0.25);
      opacity: 0.35;
    }
    to {
      transform: perspective(280px) rotateX(62deg) translateY(28%) scale(0.8);
      opacity: 1;
    }
  }
  @keyframes ship-launch {
    0% {
      transform: translateY(38vh) scale(1.45);
    }
    68% {
      transform: translateY(-2vh) scale(0.75);
    }
    100% {
      transform: translateY(-45vh) scale(0.08);
      opacity: 0.15;
    }
  }
  @keyframes warp-arrive {
    0%,
    100% {
      opacity: 0;
    }
    12%,
    88% {
      opacity: 1;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .warp,
    .warp-ship,
    .streaks,
    .streaks::before,
    .streaks::after {
      animation: none;
    }
  }
  .noscript {
    padding: 40px;
    font-family: var(--font-mono);
    color: #f2efe6;
  }
</style>
