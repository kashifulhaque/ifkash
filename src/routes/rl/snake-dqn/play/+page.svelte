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

  // Board palette, read from the theme tokens so the board follows
  // the active design and light or dark theme. Re-read whenever
  // <html data-theme> or <html data-design> changes.
  const palette = {
    surface: '',
    line: '',
    ink: '',
    ink2: '',
    falu: ''
  };
  let dpr = 1;

  function readPalette(): void {
    const css = getComputedStyle(document.documentElement);
    const v = (name: string) => css.getPropertyValue(name).trim();
    palette.surface = v('--surface');
    palette.line = v('--line');
    palette.ink = v('--ink');
    palette.ink2 = v('--ink-2');
    palette.falu = v('--falu');
  }

  function draw(): void {
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, GAME_W, GAME_H);

    // board
    ctx.fillStyle = palette.surface;
    ctx.fillRect(0, 0, GAME_W, GAME_H);

    // hairline grid: one device pixel, snapped to the pixel grid
    const hair = 1 / dpr;
    ctx.fillStyle = palette.line;
    for (let x = BLOCK_SIZE; x < GAME_W; x += BLOCK_SIZE) {
      ctx.fillRect(x, 0, hair, GAME_H);
    }
    for (let y = BLOCK_SIZE; y < GAME_H; y += BLOCK_SIZE) {
      ctx.fillRect(0, y, GAME_W, hair);
    }

    // food: the Falu-red dot
    const r = BLOCK_SIZE / 2;
    ctx.fillStyle = palette.falu;
    ctx.beginPath();
    ctx.arc(game.food.x + r, game.food.y + r, r - 4, 0, Math.PI * 2);
    ctx.fill();

    // snake: square ink cells, the tail easing toward --ink-2
    const inset = 2;
    const n = game.snake.length;
    for (let i = n - 1; i >= 0; i--) {
      const s = game.snake[i];
      ctx.fillStyle = i === 0 ? palette.ink : palette.ink2;
      ctx.globalAlpha = i === 0 ? 1 : 1 - (0.45 * i) / Math.max(n - 1, 1);
      ctx.fillRect(s.x + inset, s.y + inset, BLOCK_SIZE - inset * 2, BLOCK_SIZE - inset * 2);
    }
    ctx.globalAlpha = 1;
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
    dpr = Math.min(window.devicePixelRatio || 1, 3);
    canvas.width = GAME_W * dpr;
    canvas.height = GAME_H * dpr;
    ctx = canvas.getContext('2d')!;
    readPalette();

    const themeObserver = new MutationObserver(() => {
      readPalette();
      draw();
    });
    themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['data-theme', 'data-design']
    });

    start();
    draw();

    return () => themeObserver.disconnect();
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
  <header class="page-head">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href="/rl">RL</a>
      <span aria-hidden="true">/</span>
      <a href="/rl/snake-dqn">Snake DQN</a>
      <span aria-hidden="true">/</span>
      <span aria-current="page">Play</span>
    </nav>
    <h1 class="page-title">Watch the Agent Play</h1>
    <p class="lead">
      The trained dueling Double DQN weights running live in your browser — a
      faithful TypeScript port of the Python experiment, greedy action
      selection, no exploration.
    </p>
  </header>

  <section class="stage">
    <div class="tray">
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
      <div class="tray-foot">
        <span class="state">
          <span class="led" class:led-hollow={!running} aria-hidden="true"></span>
          {running ? 'Live' : 'Paused'}
        </span>
        <span>32 × 24</span>
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
        <div class="btn-row">
          <button class="btn btn-sm btn-primary" class:active={running} on:click={running ? stop : start}>
            {running ? 'Pause' : 'Resume'}
          </button>
          <button class="btn btn-sm" on:click={restart}>Restart</button>
        </div>
        <div class="speed">
          <span class="speed-label" id="speed-label">Speed</span>
          <div class="segmented" role="group" aria-labelledby="speed-label">
            {#each [1, 2, 4, 8] as s}
              <button
                class="speed-btn"
                class:active={speed === s}
                aria-pressed={speed === s}
                on:click={() => (speed = s)}
              >
                {s}×
              </button>
            {/each}
          </div>
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
    animation: rise var(--dur-slow) var(--ease-out) both;
  }

  /* ── Stage: the board as an object, readouts beside it ─────── */

  .stage {
    display: grid;
    grid-template-columns: minmax(0, 760px) minmax(220px, 264px);
    gap: var(--s-7);
    align-items: start;
    padding-top: var(--s-7);
  }

  /* A shallow tray holding the board, like a boxed board game */
  .tray {
    padding: var(--s-3);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface-2);
  }

  .canvas-wrap {
    position: relative;
    width: 100%;
    aspect-ratio: 4 / 3;
    overflow: hidden;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-xs);
    background: var(--surface);
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  .overlay-status {
    position: absolute;
    top: var(--s-3);
    left: var(--s-3);
    padding: 2px var(--s-2);
    border: 1px solid var(--line);
    border-radius: var(--radius-xs);
    background: var(--surface);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.02em;
    opacity: 0;
    transition: opacity var(--dur-fast) var(--ease);
  }

  .overlay-status.show {
    opacity: 1;
  }

  .tray-foot {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-3) var(--s-1) 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.02em;
    font-variant-numeric: tabular-nums;
  }

  .state {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
  }

  /* ── Readouts and controls ─────────────────────────────────── */

  .controls {
    display: grid;
    gap: var(--s-6);
  }

  .stats {
    border-top: 1px solid var(--ink);
  }

  .stat {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: var(--s-4);
    padding: var(--s-4) 0;
    border-bottom: 1px solid var(--line);
  }

  .stat-label {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.02em;
  }

  .stat-value {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xl);
    font-weight: 400;
    line-height: 1;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
  }

  .actions {
    display: grid;
    gap: var(--s-5);
  }

  .actions .btn-row .btn {
    flex: 1 1 0;
  }

  .speed {
    display: grid;
    gap: var(--s-2);
  }

  .speed-label {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.02em;
  }

  /* Hairline segmented control */
  .segmented {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .speed-btn {
    min-height: 32px;
    padding: 0 var(--s-2);
    border: 0;
    border-left: 1px solid var(--line);
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    cursor: pointer;
    transition:
      background-color var(--dur-fast) var(--ease),
      color var(--dur-fast) var(--ease);
  }

  .speed-btn:first-child {
    border-left: 0;
  }

  .speed-btn:hover {
    background: var(--hover-wash);
    color: var(--ink);
  }

  .speed-btn.active {
    background: var(--fjord-wash);
    color: var(--fjord);
  }

  .speed-btn:focus-visible {
    outline-offset: -2px;
  }

  /* ── Note ──────────────────────────────────────────────────── */

  .note {
    max-width: var(--measure);
    margin-top: var(--s-8);
    padding-top: var(--s-5);
    border-top: 1px solid var(--line);
  }

  .note p {
    margin: 0;
    color: var(--ink-2);
    font-size: var(--text-sm);
    line-height: 1.7;
  }

  @media (max-width: 900px) {
    .stage {
      grid-template-columns: minmax(0, 1fr);
      gap: var(--s-6);
      padding-top: var(--s-6);
    }

    .stats {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      border-bottom: 1px solid var(--line);
    }

    .stat {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--s-2);
      padding: var(--s-4) var(--s-4) var(--s-4) 0;
      border-bottom: 0;
    }

    .stat + .stat {
      padding-left: var(--s-4);
      border-left: 1px solid var(--line);
    }

    .actions {
      grid-template-columns: auto auto;
      justify-content: space-between;
      align-items: end;
    }

    .actions .btn-row .btn {
      flex: 0 0 auto;
    }

    .segmented {
      min-width: 176px;
    }
  }

  @media (max-width: 480px) {
    .tray {
      padding: var(--s-2);
    }

    .actions {
      grid-template-columns: minmax(0, 1fr);
    }

    .actions .btn-row .btn {
      flex: 1 1 0;
    }

    .stat-value {
      font-size: var(--text-lg);
    }

    .note {
      margin-top: var(--s-7);
    }
  }
</style>
