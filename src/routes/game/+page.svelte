<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser, dev } from '$app/environment';
  import { goto } from '$app/navigation';
  import { sections, name } from '$lib/content';
  import { gameState } from '$lib/game/store';
  import type { Game } from '$lib/game/Game';
  import type { Section } from '$lib/content';
  import Hud from '$lib/game/ui/Hud.svelte';
  import LootOverlay from '$lib/game/ui/LootOverlay.svelte';
  import TouchControls from '$lib/game/ui/TouchControls.svelte';
  import StartScreen from '$lib/game/ui/StartScreen.svelte';

  let canvas: HTMLCanvasElement;
  let game: Game | null = null;
  let destroyed = false;

  onMount(async () => {
    if (!browser) return;
    const isTouch = window.matchMedia('(pointer: coarse)').matches || 'ontouchstart' in window;
    gameState.update((s) => ({ ...s, isTouch }));
    try {
      const { Game: GameClass } = await import('$lib/game/Game');
      // The layout's full-width branch flip can destroy and remount this page
      // mid-import; the stale instance must not touch the (unbound) canvas.
      if (destroyed || !canvas) return;
      game = new GameClass(canvas, sections, {
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
        onPlayerDeath: () => gameState.update((s) => ({ ...s, deathCount: s.deathCount + 1 })),
        onHitMarker: (headshot, killed) =>
          gameState.update((s) => ({
            ...s,
            hitMarker: { id: (s.hitMarker?.id ?? 0) + 1, headshot, killed }
          })),
        onAmmoChange: (ammo, reloading) => gameState.update((s) => ({ ...s, ammo, reloading })),
        onScoreChange: (score, streak) => gameState.update((s) => ({ ...s, score, streak })),
        onAimChange: (aiming) => gameState.update((s) => ({ ...s, aiming })),
        onYawChange: (yaw) => gameState.update((s) => ({ ...s, yaw }))
      });
      gameState.update((s) => ({ ...s, muted: game!.audio.muted }));
      if (dev) (window as unknown as Record<string, unknown>).__game = game;
    } catch (err) {
      console.error('WebGL init failed', err);
      gameState.update((s) => ({ ...s, webglFailed: true }));
      goto('/');
    }
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
      ammo: 12,
      reloading: false,
      score: 0,
      streak: 0,
      aiming: false,
      yaw: 0,
      hitMarker: null
    });
  });

  function start() {
    gameState.update((s) => ({ ...s, started: true }));
    game?.setPaused(false);
    game?.startAudio();
    if (!$gameState.isTouch) game?.requestLock();
  }

  function toggleMute() {
    if (!game) return;
    const muted = game.toggleMute();
    gameState.update((s) => ({ ...s, muted }));
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
      reloading={$gameState.reloading}
      score={$gameState.score}
      streak={$gameState.streak}
      aiming={$gameState.aiming}
      yaw={$gameState.yaw}
      hitMarker={$gameState.hitMarker}
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
        on:start={start}
        on:textmode={() => goto('/')}
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
