<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { animate, stagger } from 'animejs';
  import { prefersReducedMotion } from '$lib/motion';
  import {
    inlineText,
    isSpecialToken,
    visualizeWhitespace,
    formatPercent,
    holdForSpeed
  } from '$lib/tokenLensView';
  import type {
    LensBackend,
    LensStep,
    LensWorkerToMain,
    MainToLensWorker
  } from '$lib/tokenLens/types';

  const PRESETS = [
    'Explain self-attention in one sentence.',
    'Write a haiku about tokens.',
    'What comes after Monday?'
  ];
  const TOP_K = 8;

  type Phase = 'idle' | 'downloading' | 'loading' | 'ready' | 'error';

  let worker: Worker | null = null;
  let phase: Phase = 'idle';
  let progress = 0;
  let progressMsg = '';
  let backend: LensBackend | null = null;
  let errorMsg = '';

  let prompt = PRESETS[0];
  let temperature = 0.8;
  let maxTokens = 48;
  let speed = 0.25;

  let promptTokens: string[] = [];
  let shown: LensStep[] = [];
  let queue: LensStep[] = [];
  let panelStep: LensStep | null = null;
  let panelLanded = false;
  let inspecting = false;
  let generating = false;
  let pumping = false;
  let doneInfo: { reason: string; tokensPerSecond: number } | null = null;
  let pendingDone: { reason: string; tokensPerSecond: number } | null = null;
  let runId = 0;
  let destroyed = false;

  let outputEl: HTMLDivElement;
  let panelEl: HTMLDivElement;

  $: running = generating || pumping;
  $: busy = phase === 'downloading' || phase === 'loading';
  $: chosenInList = panelStep
    ? panelStep.candidates.some((c) => c.id === panelStep!.chosen.id)
    : false;
  $: maxP = panelStep ? Math.max(...panelStep.candidates.map((c) => c.p), panelStep.chosen.p) : 1;

  function send(msg: MainToLensWorker) {
    worker?.postMessage(msg);
  }

  onMount(() => {
    worker = new Worker(new URL('$lib/tokenLens/worker.ts', import.meta.url), {
      type: 'module'
    });
    worker.onmessage = (e: MessageEvent<LensWorkerToMain>) => handle(e.data);
  });

  onDestroy(() => {
    destroyed = true;
    runId++;
    worker?.terminate();
    worker = null;
  });

  function handle(msg: LensWorkerToMain) {
    switch (msg.type) {
      case 'progress':
        phase = msg.status;
        progress = msg.progress;
        progressMsg = msg.message ?? '';
        break;
      case 'ready':
        phase = 'ready';
        backend = msg.backend;
        break;
      case 'prompt':
        promptTokens = msg.tokens;
        break;
      case 'step':
        if (!generating) break;
        queue.push(msg.step);
        queue = queue;
        void pump(runId);
        break;
      case 'done':
        generating = false;
        pendingDone = { reason: msg.reason, tokensPerSecond: msg.tokensPerSecond };
        if (!pumping) finish();
        break;
      case 'error':
        errorMsg = msg.error;
        generating = false;
        if (phase !== 'ready') phase = 'error';
        break;
    }
  }

  function finish() {
    doneInfo = pendingDone;
    pendingDone = null;
  }

  function loadModel() {
    errorMsg = '';
    phase = 'downloading';
    progress = 0;
    send({ type: 'load' });
  }

  function generate() {
    if (!prompt.trim() || phase !== 'ready') return;
    runId++;
    errorMsg = '';
    promptTokens = [];
    shown = [];
    queue = [];
    panelStep = null;
    inspecting = false;
    doneInfo = null;
    pendingDone = null;
    generating = true;
    send({ type: 'generate', prompt, options: { topK: TOP_K, maxNewTokens: maxTokens, temperature } });
  }

  function stop() {
    send({ type: 'stop' });
    // Drop what has not been revealed yet so Stop feels immediate.
    queue = [];
  }

  const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

  async function pump(id: number) {
    if (pumping) return;
    pumping = true;
    while (queue.length && !destroyed && id === runId) {
      const step = queue.shift()!;
      queue = queue;
      const reduced = prefersReducedMotion();
      const hold = reduced ? Math.min(holdForSpeed(speed), 500) : holdForSpeed(speed);
      const fancy = !reduced && hold >= 300;

      inspecting = false;
      panelStep = step;
      panelLanded = false;
      await tick();
      if (id !== runId) break;

      if (fancy && panelEl) {
        const bars = panelEl.querySelectorAll('.bar-fill');
        const rows = panelEl.querySelectorAll('.cand');
        animate(rows, { opacity: [0, 1], duration: 300, delay: stagger(40) });
        animate(bars, { scaleX: [0, 1], duration: 600, delay: stagger(50), ease: 'outExpo' });
        await sleep(hold * 0.55);
        if (id !== runId) break;
        const chosenRow = panelEl.querySelector('.cand.chosen');
        if (chosenRow) animate(chosenRow, { scale: [1, 1.03, 1], duration: 420, ease: 'inOutSine' });
        await sleep(hold * 0.35);
      } else if (hold > 0) {
        await sleep(hold);
      }
      if (id !== runId) break;

      panelLanded = true;
      shown = [...shown, step];
      await tick();
      const el = outputEl?.querySelector<HTMLElement>(`[data-gen="${step.index}"]`);
      if (el && !reduced && hold > 0) {
        animate(el, { opacity: [0, 1], translateY: [6, 0], duration: 320, ease: 'outCubic' });
      }
      outputEl?.scrollTo({ top: outputEl.scrollHeight });
      if (hold > 0) await sleep(Math.min(hold * 0.1, 240));
    }
    pumping = false;
    if (id === runId && !generating && !queue.length && pendingDone) finish();
  }

  function inspect(step: LensStep) {
    inspecting = true;
    panelStep = step;
    panelLanded = true;
    void tick().then(() => {
      if (!panelEl || prefersReducedMotion()) return;
      animate(panelEl.querySelectorAll('.bar-fill'), {
        scaleX: [0, 1],
        duration: 400,
        delay: stagger(30),
        ease: 'outExpo'
      });
    });
  }

</script>

<section class="lens">
  {#if phase !== 'ready'}
    <div class="load card">
      <p class="note">
        This runs Qwen3-0.6B in your browser. The first visit downloads about 400 MB, which your
        browser then caches. After that, everything runs on your device and nothing is sent to a
        server.
      </p>
      <div class="load-row">
        <button class="btn btn-primary" on:click={loadModel} disabled={busy}>
          {busy ? 'Loading…' : phase === 'error' ? 'Retry' : 'Load model'}
        </button>
        {#if busy}
          <div class="meter" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(progress)}>
            <span style="width: {Math.min(100, Math.max(0, progress))}%"></span>
          </div>
          <span class="mono dim">
            {phase === 'downloading' ? 'Downloading' : 'Loading'}
            {Math.round(progress)}%
          </span>
        {/if}
      </div>
      {#if progressMsg && busy}<p class="mono dim small">{progressMsg}</p>{/if}
    </div>
  {:else}
    <p class="mono dim small status">
      <span class="led" aria-hidden="true"></span>
      Model ready on {backend === 'webgpu' ? 'WebGPU' : 'WASM (CPU)'}
    </p>
  {/if}

  {#if errorMsg}<p class="error" role="alert">{errorMsg}</p>{/if}

  <div class="controls">
    <label class="field grow">
      <span class="label">Prompt</span>
      <textarea
        rows="2"
        bind:value={prompt}
        disabled={running}
        on:keydown={(e) => {
          if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) generate();
        }}
      ></textarea>
    </label>
    <div class="chips">
      {#each PRESETS as p}
        <button class="chip" disabled={running} on:click={() => (prompt = p)}>{p}</button>
      {/each}
    </div>

    <div class="sliders">
      <label class="field">
        <span class="label">
          Temperature <b class="mono">{temperature === 0 ? '0 (greedy)' : temperature.toFixed(2)}</b>
        </span>
        <input type="range" min="0" max="1.5" step="0.05" bind:value={temperature} disabled={running} />
      </label>
      <label class="field">
        <span class="label">Max tokens <b class="mono">{maxTokens}</b></span>
        <input type="range" min="8" max="128" step="4" bind:value={maxTokens} disabled={running} />
      </label>
      <label class="field">
        <span class="label">
          Pace <b class="mono">{speed >= 0.98 ? 'realtime' : speed <= 0.02 ? 'slowest' : 'slowed'}</b>
        </span>
        <input type="range" min="0" max="1" step="0.01" bind:value={speed} />
      </label>
    </div>

    <div class="actions">
      {#if running}
        <button class="btn" on:click={stop}>Stop</button>
      {:else}
        <button class="btn btn-primary" on:click={generate} disabled={phase !== 'ready' || !prompt.trim()}>
          Generate
        </button>
      {/if}
    </div>
  </div>

  <div class="stage">
    <div class="output" bind:this={outputEl} aria-live="off">
      {#if !promptTokens.length && !shown.length}
        <p class="dim placeholder">
          {phase === 'ready' ? 'Press Generate to start.' : 'Load the model, then generate.'}
        </p>
      {:else}
        <p class="stream">
          {#each promptTokens as t, i}<span class="tok prompt" class:alt={i % 2 === 1}>{#if isSpecialToken(t)}<span class="pill">{t}</span>{:else}{t}{/if}</span>{/each}{#if promptTokens.length}<span class="turn" aria-hidden="true"></span>{/if}{#each shown as s (s.index)}<button
              class="tok gen"
              class:alt={(promptTokens.length + s.index) % 2 === 1}
              class:active={panelStep === s}
              data-gen={s.index}
              title="Show candidates for this step"
              on:click={() => inspect(s)}
              >{#if isSpecialToken(s.chosen.text)}<span class="pill">{s.chosen.text}</span>{:else}{inlineText(s.chosen.text)}{/if}</button
            >{#if s.chosen.text.includes('\n')}<br />{/if}{/each}{#if running}<span class="caret" aria-hidden="true"></span>{/if}
        </p>
      {/if}
    </div>

    <div class="panel" bind:this={panelEl}>
      {#if panelStep}
        <header class="panel-head">
          <span class="label">
            {inspecting ? 'Step' : 'Choosing token'} {panelStep.index + 1}
          </span>
          <span class="mono dim small" title="Shannon entropy of the full distribution">
            uncertainty {panelStep.entropy.toFixed(2)} bits
          </span>
        </header>
        <ul class="cands">
          {#each panelStep.candidates as c (c.id)}
            <li class="cand" class:chosen={c.id === panelStep.chosen.id}>
              <span class="ctext mono">{visualizeWhitespace(c.text)}</span>
              <span class="bar"><span class="bar-fill" style="width: {(c.p / maxP) * 100}%"></span></span>
              <span class="cpct mono">{formatPercent(c.p)}</span>
            </li>
          {/each}
          {#if !chosenInList}
            <li class="cand chosen extra">
              <span class="ctext mono">{visualizeWhitespace(panelStep.chosen.text)}</span>
              <span class="bar"><span class="bar-fill" style="width: {(panelStep.chosen.p / maxP) * 100}%"></span></span>
              <span class="cpct mono">{formatPercent(panelStep.chosen.p)}</span>
            </li>
          {/if}
        </ul>
        <p class="dim small hint">
          Highlighted row is the sampled token{chosenInList ? '' : ' (outside the top ' + TOP_K + ')'}.
        </p>
      {:else}
        <p class="dim placeholder">The model's top candidates for each next token appear here.</p>
      {/if}
    </div>
  </div>

  <footer class="stats mono small">
    <span>{shown.length} token{shown.length === 1 ? '' : 's'} generated</span>
    {#if doneInfo}
      <span>{doneInfo.tokensPerSecond.toFixed(1)} tokens/sec</span>
      <span>
        {doneInfo.reason === 'eos' ? 'ended: model finished' : doneInfo.reason === 'length' ? 'ended: hit max tokens' : 'ended: stopped'}
      </span>
    {/if}
  </footer>

  <p class="dim small honest">
    Probabilities are the model's real softmax at temperature 1, whatever temperature you pick for
    sampling. Attention lines aren't shown because the ONNX build doesn't expose attention weights.
  </p>
</section>

<style>
  .lens {
    display: grid;
    gap: var(--s-5);
    max-width: 100%;
    min-width: 0;
  }
  .card {
    padding: var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
  }
  .note {
    margin: 0 0 var(--s-4);
    max-width: var(--measure);
    color: var(--ink-2);
    font-size: var(--text-sm);
    line-height: 1.55;
  }
  .load-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-3);
  }
  .meter {
    flex: 1 1 120px;
    height: 4px;
    background: var(--line);
    border-radius: var(--radius-pill);
    overflow: hidden;
  }
  .meter span {
    display: block;
    height: 100%;
    background: var(--fjord);
    transition: width var(--dur) var(--ease);
  }
  .mono {
    font-family: var(--font-mono);
  }
  .dim {
    color: var(--ink-3);
  }
  .small {
    font-size: var(--text-xs);
  }
  .status {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin: 0;
  }
  .error {
    margin: 0;
    color: var(--falu);
    font-size: var(--text-sm);
  }

  .controls {
    display: grid;
    gap: var(--s-3);
  }
  .field {
    display: grid;
    gap: var(--s-1);
    min-width: 0;
  }
  .label {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .label b {
    margin-left: var(--s-1);
    color: var(--ink);
    font-weight: 500;
    text-transform: none;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    padding: var(--s-3);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink);
    font: inherit;
    font-size: var(--text-base);
    resize: vertical;
  }
  textarea:focus-visible {
    outline: 2px solid var(--fjord);
    outline-offset: 1px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .chip {
    max-width: 100%;
    padding: var(--s-1) var(--s-3);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-pill);
    background: transparent;
    color: var(--ink-2);
    font: inherit;
    font-size: var(--text-xs);
    text-align: left;
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
  }
  .chip:hover:not(:disabled) {
    border-color: var(--fjord);
    color: var(--fjord);
  }
  .chip:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .sliders {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--s-4);
  }
  input[type='range'] {
    width: 100%;
    accent-color: var(--fjord);
  }
  .actions {
    display: flex;
    gap: var(--s-3);
  }

  .stage {
    display: grid;
    grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);
    gap: var(--s-4);
    align-items: start;
  }
  .output,
  .panel {
    min-width: 0;
    padding: var(--s-4);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
  }
  .output {
    min-height: 220px;
    max-height: 420px;
    overflow-y: auto;
  }
  .placeholder {
    margin: 0;
    font-size: var(--text-sm);
  }
  .turn {
    display: block;
    height: var(--s-3);
  }

  .stream {
    margin: 0;
    font-size: var(--text-md);
    line-height: 1.9;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .tok {
    padding: 1px 0;
    border: 0;
    border-bottom: 2px solid var(--line-strong);
    background: transparent;
    color: var(--ink);
    font: inherit;
    line-height: inherit;
    white-space: pre-wrap;
  }
  .tok.alt {
    border-bottom-color: var(--line);
    background: var(--fjord-wash);
  }
  .tok.prompt {
    color: var(--ink-3);
  }
  button.tok {
    cursor: pointer;
  }
  button.tok:hover,
  button.tok.active {
    border-bottom-color: var(--fjord);
    color: var(--fjord);
  }
  .pill {
    display: inline-block;
    padding: 0 var(--s-2);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-pill);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.6;
    vertical-align: middle;
  }
  .caret {
    display: inline-block;
    width: 2px;
    height: 1.1em;
    margin-left: 2px;
    background: var(--fjord);
    vertical-align: text-bottom;
    animation: blink 1s steps(2, start) infinite;
  }
  @keyframes blink {
    to {
      visibility: hidden;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .caret {
      animation: none;
    }
  }

  .panel-head {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: var(--s-2);
    margin-bottom: var(--s-3);
  }
  .cands {
    display: grid;
    gap: var(--s-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .cand {
    display: grid;
    grid-template-columns: minmax(48px, 30%) minmax(0, 1fr) 40px;
    align-items: center;
    gap: var(--s-2);
    padding: 2px var(--s-2);
    border: 1px solid transparent;
    border-radius: var(--radius-xs);
    font-size: var(--text-sm);
  }
  .cand.chosen {
    border-color: var(--fjord);
    background: var(--fjord-wash);
  }
  .cand.extra {
    margin-top: var(--s-2);
  }
  .ctext {
    overflow: hidden;
    color: var(--ink);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .bar {
    height: 8px;
    border-radius: var(--radius-pill);
    background: var(--line);
    overflow: hidden;
  }
  .bar-fill {
    display: block;
    height: 100%;
    background: var(--ink-3);
    transform-origin: left center;
  }
  .cand.chosen .bar-fill {
    background: var(--fjord);
  }
  .cpct {
    color: var(--ink-2);
    font-size: var(--text-xs);
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .hint {
    margin: var(--s-3) 0 0;
  }

  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2) var(--s-5);
    color: var(--ink-2);
  }
  .honest {
    margin: 0;
    max-width: var(--measure);
    line-height: 1.55;
  }

  @media (max-width: 720px) {
    .stage {
      grid-template-columns: minmax(0, 1fr);
    }
    .sliders {
      grid-template-columns: minmax(0, 1fr);
    }
    .output {
      max-height: 300px;
    }
  }
</style>
