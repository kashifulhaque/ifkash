<svelte:head>
  <title>Snake RL — Watch the Agent Play</title>
  <meta
    name="description"
    content="Watch the trained dueling Double DQN agent play snake in the browser — the best.pth weights running as a TypeScript port on canvas."
  />
</svelte:head>

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';
  import { SnakeGame, getState, getAction, BLOCK_SIZE } from '$lib/rl/snake';

  const GAME_W = 640;
  const GAME_H = 480;
  const BASE_STEPS_PER_SEC = 10; // the trained agent acts ~10x/sec

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let game: SnakeGame;
  let running = false;
  let rafId = 0;
  let lastFrame = 0;
  let lastStep = 0;
  let stepsPerSec = BASE_STEPS_PER_SEC;
  let speed = 1;

  let score = 0;
  let record = 0;
  let games = 0;
  let status = 'starting';

  function stepOnce(): void {
    const action = getAction(game);
    const { reward, done } = game.playStep(action);
    score = game.score;
    if (reward > 0) status = 'ate';
    else status = done ? 'died' : 'moving';
    if (done) {
      games += 1;
      if (score > record) record = score;
      resetGame();
      status = 'starting';
    }
  }

  function resetGame(): void {
    game.reset();
    score = 0;
  }

  function draw(): void {
    if (!ctx) return;
    ctx.clearRect(0, 0, GAME_W, GAME_H);

    // board
    ctx.fillStyle = 'rgba(0, 0, 0, 0.35)';
    ctx.fillRect(0, 0, GAME_W, GAME_H);
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.05)';
    ctx.lineWidth = 1;
    for (let x = 0; x < GAME_W; x += BLOCK_SIZE) {
      ctx.beginPath();
      ctx.moveTo(x + 0.5, 0);
      ctx.lineTo(x + 0.5, GAME_H);
      ctx.stroke();
    }
    for (let y = 0; y < GAME_H; y += BLOCK_SIZE) {
      ctx.beginPath();
      ctx.moveTo(0, y + 0.5);
      ctx.lineTo(GAME_W, y + 0.5);
      ctx.stroke();
    }

    // food
    ctx.fillStyle = '#c84040';
    ctx.fillRect(game.food.x, game.food.y, BLOCK_SIZE, BLOCK_SIZE);

    // snake
    for (let i = 0; i < game.snake.length; i++) {
      const s = game.snake[i];
      if (i === 0) ctx.fillStyle = '#8ab4f8';
      else ctx.fillStyle = i % 2 === 0 ? '#3b6fb5' : '#3a6eae';
      ctx.fillRect(s.x + 1, s.y + 1, BLOCK_SIZE - 2, BLOCK_SIZE - 2);
    }
  }

  function onRaf(t: number): void {
    if (!running) return;
    rafId = requestAnimationFrame(onRaf);
    if (t - lastFrame < 16) return;
    lastFrame = t;
    while (t - lastStep >= 1000 / stepsPerSec) {
      stepOnce();
      lastStep += 1000 / stepsPerSec;
    }
    draw();
  }

  function start(): void {
    if (running) return;
    running = true;
    lastStep = performance.now();
    lastFrame = 0;
    status = 'starting';
    rafId = requestAnimationFrame(onRaf);
  }

  function stop(): void {
    running = false;
    cancelAnimationFrame(rafId);
    status = 'paused';
  }

  function restart(): void {
    resetGame();
    if (running) {
      lastStep = performance.now();
    } else {
      start();
    }
  }

  onMount(() => {
    game = new SnakeGame();
    ctx = canvas.getContext('2d')!;
    start();
    draw();
  });

  onDestroy(() => {
    if (!browser) return;
    running = false;
    cancelAnimationFrame(rafId);
  });

  $: stepsPerSec = BASE_STEPS_PER_SEC * speed;
  $: scoreLabel = String(score).padStart(2, '0');
  $: recordLabel = String(record).padStart(2, '0');
  $: gamesLabel = String(games);
</script>

<div class="page">
  <header class="page-header">
    <div class="breadcrumb">
      <a href="/rl">RL</a>
      <span class="separator">/</span>
      <a href="/rl/snake-dqn">snake-dqn</a>
      <span class="separator">/</span>
      <span>play</span>
    </div>
    <h1 class="page-title">Watch the Agent Play</h1>
    <p class="page-desc">
      The trained dueling Double DQN weights running live in your browser — a
      faithful TypeScript port of the Python experiment, greedy action
      selection, no exploration.
    </p>
  </header>

  <section class="stage">
    <div class="canvas-wrap">
      <canvas
        bind:this={canvas}
        width={GAME_W}
        height={GAME_H}
        aria-label="Snake RL agent playing"
      ></canvas>
      <div class="overlay-status" class:show={status === 'starting'}>
        {#if status === 'starting'}new game{/if}
      </div>
    </div>

    <div class="controls">
      <div class="stats">
        <div class="stat">
          <div class="stat-label">Score</div>
          <div class="stat-value">{scoreLabel}</div>
        </div>
        <div class="stat">
          <div class="stat-label">Record</div>
          <div class="stat-value">{recordLabel}</div>
        </div>
        <div class="stat">
          <div class="stat-label">Games</div>
          <div class="stat-value">{gamesLabel}</div>
        </div>
      </div>
      <div class="actions">
        <button class="btn" class:active={running} on:click={running ? stop : start}>
          {running ? 'Pause' : 'Resume'}
        </button>
        <button class="btn" on:click={restart}>Restart</button>
        <div class="speed">
          <span class="speed-label">Speed</span>
          {#each [1, 2, 4, 8] as s}
            <button class="speed-btn" class:active={speed === s} on:click={() => (speed = s)}>
              {s}×
            </button>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <section class="note">
    <p>
      The agent runs fully client-side: the exported
      <code>best.pth</code> weights (a 28 → 256 → 256 dueling MLP) are loaded
      from <code>src/lib/rl/snake-dqn-weights.json</code> and stepped at ~10
      actions/sec. The game rules, 28-feature state encoding, and greedy
      action selection mirror <code>snake_env.py</code> / <code>state.py</code>
      / <code>play.py</code> exactly.
    </p>
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    max-width: 48rem;
    animation: fade-up var(--dur-base) var(--ease-out-quart);
  }

  .page-header {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--line-soft);
  }

  .breadcrumb {
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-mute);
  }

  .breadcrumb a {
    color: var(--ink-mute);
    transition: color var(--dur-instant) var(--ease-out-quart);
  }

  .breadcrumb a:hover {
    color: var(--signal-hi);
  }

  .separator {
    margin: 0 0.5rem;
  }

  .page-desc {
    font-size: 1.0625rem;
    line-height: 1.6;
    color: var(--ink-soft);
    margin: 0;
  }

  .stage {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .canvas-wrap {
    position: relative;
    width: 100%;
    max-width: 640px;
    aspect-ratio: 4 / 3;
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: #0b0d10;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  .overlay-status {
    position: absolute;
    top: 0.75rem;
    left: 0.75rem;
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-mute);
    opacity: 0;
    transition: opacity var(--dur-instant) var(--ease-out-quart);
  }

  .overlay-status.show {
    opacity: 1;
  }

  .controls {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
    max-width: 640px;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 1rem 1.25rem;
    background: var(--panel);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
  }

  .stat-label {
    font-family: var(--font-mono-g);
    font-size: 0.65rem;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-mute);
  }

  .stat-value {
    font-family: var(--font-dots);
    font-size: 1.75rem;
    font-weight: 600;
    color: var(--ink);
    line-height: 1;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .btn {
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-soft);
    padding: 0.5rem 1rem;
    background: var(--panel);
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    cursor: var(--cursor-pointer);
    transition:
      color var(--dur-instant) var(--ease-out-quart),
      background-color var(--dur-instant) var(--ease-out-quart),
      border-color var(--dur-instant) var(--ease-out-quart);
  }

  .btn:hover,
  .btn.active {
    background: var(--ink);
    color: var(--void);
    border-color: var(--ink);
  }

  .speed {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .speed-label {
    font-family: var(--font-mono-g);
    font-size: 0.65rem;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-mute);
  }

  .speed-btn {
    font-family: var(--font-mono-g);
    font-size: 0.7rem;
    color: var(--ink-soft);
    padding: 0.35rem 0.6rem;
    background: transparent;
    border: 1px solid var(--line-soft);
    border-radius: var(--radius-sm);
    cursor: var(--cursor-pointer);
    transition:
      color var(--dur-instant) var(--ease-out-quart),
      background-color var(--dur-instant) var(--ease-out-quart),
      border-color var(--dur-instant) var(--ease-out-quart);
  }

  .speed-btn:hover,
  .speed-btn.active {
    background: var(--ink);
    color: var(--void);
    border-color: var(--ink);
  }

  .note p {
    font-size: 0.875rem;
    line-height: 1.7;
    color: var(--ink-mute);
  }

  .note code {
    font-family: var(--font-mono-g);
    font-size: 0.8125rem;
    color: var(--ink-soft);
    background: var(--code-bg);
    border: 1px solid var(--line-soft);
    padding: 0.125rem 0.375rem;
    border-radius: var(--radius-sm);
  }

  @keyframes fade-up {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
