<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser, dev } from '$app/environment';
  import { goto } from '$app/navigation';
  import { sections, name } from '$lib/content';
  import { gameState } from '$lib/game/store';
  import type { Game, GameCallbacks } from '$lib/game/Game';
  import type { Section } from '$lib/content';
  import Hud from '$lib/game/ui/Hud.svelte';
  import LootOverlay from '$lib/game/ui/LootOverlay.svelte';
  import TouchControls from '$lib/game/ui/TouchControls.svelte';
  import StartScreen from '$lib/game/ui/StartScreen.svelte';
  import DeathScreen from '$lib/game/ui/DeathScreen.svelte';
  import PerkOverlay from '$lib/game/ui/PerkOverlay.svelte';

  let canvas: HTMLCanvasElement;
  let game: Game | null = null;
  let GameClass: typeof import('$lib/game/Game').Game | null = null;
  let destroyed = false;
  let popupId = 1;

  function makeCallbacks(): GameCallbacks {
    return {
      onLockChange: (locked) => {
          gameState.update((s) => ({ ...s, pointerLocked: locked }));
          // Esc (pointer-lock loss) shows the pause screen — freeze the sim too
          if (!locked && $gameState.started && !$gameState.openSection && !$gameState.isTouch)
            game?.setPaused(true);
        },
        onInteractPrompt: (label) =>
          gameState.update((s) => ({ ...s, interactPrompt: label })),
        onOpenSection: (section: Section) => {
          game?.setPaused(true);
          gameState.update((s) => ({ ...s, openSection: section }));
        },
        onHealthChange: (health) => gameState.update((s) => ({ ...s, health })),
        onPlayerHit: () => gameState.update((s) => ({ ...s, hitCount: s.hitCount + 1 })),
        onPlayerDeath: () =>
          gameState.update((s) => ({
            ...s,
            deathCount: s.deathCount + 1,
            dead: true,
            finalScore: s.score
          })),
        onHitMarker: (headshot, killed) =>
          gameState.update((s) => ({
            ...s,
            hitMarker: { id: (s.hitMarker?.id ?? 0) + 1, headshot, killed }
          })),
        onAmmoChange: (ammo, reserve, reloading) =>
          gameState.update((s) => ({ ...s, ammo, reserve, reloading })),
        onScoreChange: (score, streak) => gameState.update((s) => ({ ...s, score, streak })),
        onComboChange: (combo, comboMult, comboTimer) =>
          gameState.update((s) => ({ ...s, combo, comboMult, comboTimer })),
        onWaveChange: (wave, waveKills, waveQuota, started) =>
          gameState.update((s) => ({
            ...s,
            wave,
            waveKills,
            waveQuota,
            waveBanner: started ? { id: (s.waveBanner?.id ?? 0) + 1, wave } : s.waveBanner
          })),
        onScorePopup: (amount, mult, headshot) =>
          gameState.update((s) => ({
            ...s,
            // Cap the list so it never grows unbounded; CSS animates each out
            scorePopups: [
              ...s.scorePopups.slice(-7),
              { id: popupId++, amount, mult, headshot }
            ]
          })),
        onAimChange: (aiming) => gameState.update((s) => ({ ...s, aiming })),
        onYawChange: (yaw) => gameState.update((s) => ({ ...s, yaw })),
        onSectionsChange: (sectionsOpened, allSectionsCleared) =>
          gameState.update((s) => ({ ...s, sectionsOpened, allSectionsCleared })),
        onPerkOffer: (perkOffer) => gameState.update((s) => ({ ...s, perkOffer }))
    };
  }

  // Build (or rebuild) the Game. Rebuilding is how we switch between the normal
  // run and the daily challenge, since daily mode is fixed at construction.
  async function buildGame(daily: boolean): Promise<boolean> {
    if (!GameClass || destroyed || !canvas) return false;
    game?.dispose();
    try {
      game = new GameClass(canvas, sections, makeCallbacks(), { daily });
    } catch (err) {
      console.error('WebGL init failed', err);
      gameState.update((s) => ({ ...s, webglFailed: true }));
      goto('/');
      return false;
    }
    gameState.update((s) => ({
      ...s,
      muted: game!.audio.muted,
      daily: game!.daily,
      dailyDay: game!.dailyDay,
      mutators: game!.mutators,
      sectionsOpened: [],
      allSectionsCleared: false,
      perkOffer: null
    }));
    if (dev) (window as unknown as Record<string, unknown>).__game = game;
    return true;
  }

  onMount(async () => {
    if (!browser) return;
    const isTouch = window.matchMedia('(pointer: coarse)').matches || 'ontouchstart' in window;
    gameState.update((s) => ({ ...s, isTouch }));
    const mod = await import('$lib/game/Game');
    // The layout's full-width branch flip can destroy and remount this page
    // mid-import; the stale instance must not touch the (unbound) canvas.
    if (destroyed || !canvas) return;
    GameClass = mod.Game;
    await buildGame(false);
  });

  onDestroy(() => {
    destroyed = true;
    game?.dispose();
    game = null;
    gameState.set({
      started: false,
      pointerLocked: false,
      interactPrompt: null,
      openSection: null,
      webglFailed: false,
      isTouch: false,
      textMode: false,
      health: 100,
      hitCount: 0,
      deathCount: 0,
      muted: false,
      dayNightCycle: false,
      ammo: 12,
      reserve: 24,
      reloading: false,
      score: 0,
      streak: 0,
      combo: 0,
      comboMult: 1,
      comboTimer: 0,
      wave: 1,
      waveKills: 0,
      waveQuota: 8,
      waveBanner: null,
      scorePopups: [],
      aiming: false,
      yaw: 0,
      hitMarker: null,
      dead: false,
      finalScore: 0,
      sectionsOpened: [],
      allSectionsCleared: false,
      daily: false,
      dailyDay: null,
      mutators: [],
      perkOffer: null,
      perksTaken: []
    });
  });

  async function start(daily = false) {
    // Switching into/out of daily means rebuilding the sim (mode is fixed at
    // construction). A plain resume keeps the current game.
    if (game && game.daily !== daily) {
      if (!(await buildGame(daily))) return;
    }
    gameState.update((s) => ({ ...s, started: true }));
    game?.setPaused(false);
    game?.startAudio();
    if (!$gameState.isTouch) game?.requestLock();
  }

  function choosePerk(perk: import('$lib/game/perks').Perk) {
    game?.choosePerk(perk);
    gameState.update((s) => ({ ...s, perksTaken: [...s.perksTaken, perk.id] }));
    if (!$gameState.isTouch) game?.requestLock();
  }

  function toggleMute() {
    if (!game) return;
    const muted = game.toggleMute();
    gameState.update((s) => ({ ...s, muted }));
  }

  function toggleCycle() {
    if (!game) return;
    game.setDayNightCycle(!game.dayNightCycle);
    gameState.update((s) => ({ ...s, dayNightCycle: game!.dayNightCycle }));
  }

  function respawn() {
    gameState.update((s) => ({ ...s, dead: false }));
    game?.respawn();
    if (!$gameState.isTouch) game?.requestLock();
  }

  function closeOverlay() {
    gameState.update((s) => ({ ...s, openSection: null }));
    game?.setPaused(false);
    if (!$gameState.isTouch) game?.requestLock();
  }

  $: showStart =
    !$gameState.webglFailed &&
    !$gameState.textMode &&
    !$gameState.openSection &&
    !$gameState.dead &&
    !$gameState.perkOffer &&
    (!$gameState.started || (!$gameState.isTouch && !$gameState.pointerLocked));
</script>

<svelte:head>
  <title>{name}</title>
  <meta
    name="description"
    content="Personal website of Kashiful Haque, ML Engineer. Playable FPS portfolio — shoot the targets, loot the crates."
  />
</svelte:head>

<div class="game-root">
  {#if !$gameState.webglFailed}
    <canvas bind:this={canvas} class="game-canvas"></canvas>

    <Hud
      pointerLocked={$gameState.pointerLocked}
      isTouch={$gameState.isTouch}
      interactPrompt={$gameState.interactPrompt}
      health={$gameState.health}
      hitCount={$gameState.hitCount}
      deathCount={$gameState.deathCount}
      muted={$gameState.muted}
      ammo={$gameState.ammo}
      reserve={$gameState.reserve}
      reloading={$gameState.reloading}
      score={$gameState.score}
      combo={$gameState.combo}
      comboMult={$gameState.comboMult}
      comboTimer={$gameState.comboTimer}
      wave={$gameState.wave}
      waveKills={$gameState.waveKills}
      waveQuota={$gameState.waveQuota}
      waveBanner={$gameState.waveBanner}
      scorePopups={$gameState.scorePopups}
      aiming={$gameState.aiming}
      yaw={$gameState.yaw}
      hitMarker={$gameState.hitMarker}
      sectionsOpened={$gameState.sectionsOpened}
      on:mute={toggleMute}
    />

    {#if $gameState.isTouch && $gameState.started && !$gameState.openSection}
      <TouchControls
        interactPrompt={$gameState.interactPrompt}
        on:move={(e) => game && (game.input.touchMove = e.detail)}
        on:look={(e) => game?.input.addLook(e.detail.dx, e.detail.dy)}
        on:fire={() => game?.input.queueFire()}
        on:interact={() => game?.input.queueInteract()}
        on:jump={() => game?.input.queueJump()}
        on:crouch={(e) => game && (game.input.touchCrouch = e.detail.active)}
        on:aim={(e) => game && (game.input.touchAim = e.detail.active)}
        on:reload={() => game?.input.queueReload()}
      />
    {/if}

    {#if showStart}
      <StartScreen
        isTouch={$gameState.isTouch}
        resumed={$gameState.started}
        muted={$gameState.muted}
        dayNightCycle={$gameState.dayNightCycle}
        on:start={() => start($gameState.started ? $gameState.daily : false)}
        on:startdaily={() => start(true)}
        on:mute={toggleMute}
        on:togglecycle={toggleCycle}
        on:textmode={() => goto('/')}
      />
    {/if}

    {#if $gameState.perkOffer}
      <PerkOverlay
        perks={$gameState.perkOffer}
        wave={$gameState.wave}
        on:choose={(e) => choosePerk(e.detail)}
      />
    {/if}

    {#if $gameState.dead}
      <DeathScreen
        finalScore={$gameState.finalScore}
        allSectionsCleared={$gameState.allSectionsCleared}
        sectionsCount={$gameState.sectionsOpened.length}
        daily={$gameState.daily}
        dailyDay={$gameState.dailyDay}
        on:respawn={respawn}
      />
    {/if}

    {#if $gameState.openSection}
      <LootOverlay section={$gameState.openSection} on:close={closeOverlay} />
    {/if}
  {/if}

  <noscript>
    <div class="noscript">
      <p>This portfolio is a small browser game and needs JavaScript.</p>
      <p>Kashiful Haque — ML Engineer. github.com/kashifulhaque · linkedin.com/in/kashifulhaque</p>
    </div>
  </noscript>
</div>

<style>
  .game-root {
    position: fixed;
    inset: 0;
    width: 100vw;
    height: 100dvh;
    overflow: hidden;
    background: #87ceeb;
  }

  .game-canvas {
    width: 100%;
    height: 100%;
    display: block;
    touch-action: none;
  }

  .noscript {
    padding: 40px;
    font-family: monospace;
    color: #111;
  }
</style>
