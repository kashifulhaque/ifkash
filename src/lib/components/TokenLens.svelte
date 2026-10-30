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
    { label: 'Self-attention', text: 'Explain self-attention in one sentence.' },
    { label: 'Haiku', text: 'Write a haiku about tokens.' },
    { label: 'Next day', text: 'What comes after Monday?' }
  ];
  const TOP_K = 8;

  type Phase = 'idle' | 'downloading' | 'loading' | 'ready' | 'error';

  let worker: Worker | null = null;
  let phase: Phase = 'idle';
  let progress = 0;
  let progressMsg = '';
  let backend: LensBackend | null = null;
  let errorMsg = '';

  let prompt = PRESETS[0].text;
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
  $: paceLabel = speed >= 0.98 ? 'live' : `${(holdForSpeed(speed) / 1000).toFixed(1)}s`;
  $: backendLabel = backend === 'webgpu' ? 'WebGPU' : 'WASM';
  $: doneReason =
    doneInfo?.reason === 'eos'
      ? 'finished'
      : doneInfo?.reason === 'length'
        ? 'max tokens'
        : doneInfo
          ? 'stopped'
          : '';

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
      <div class="load-copy">
        <p class="label">On device</p>
        <p class="note">
          Qwen3-0.6B runs in your browser. The first visit downloads about 400 MB and caches it.
          After that, nothing leaves this page.
        </p>
      </div>
      <div class="load-actions">
        <button class="btn btn-primary" on:click={loadModel} disabled={busy}>
          {busy ? 'Loading…' : phase === 'error' ? 'Retry' : 'Load model'}
        </button>
        {#if busy}
          <div class="meter-block">
            <div
              class="meter"
              role="progressbar"
              aria-valuemin="0"
              aria-valuemax="100"
              aria-valuenow={Math.round(progress)}
            >
              <span style="width: {Math.min(100, Math.max(0, progress))}%"></span>
            </div>
            <span class="mono dim small">
              {phase === 'downloading' ? 'Downloading' : 'Loading'}
              {Math.round(progress)}%{progressMsg ? ` · ${progressMsg}` : ''}
            </span>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  {#if errorMsg}<p class="error" role="alert">{errorMsg}</p>{/if}

  <div class="controls card">
    <label class="control">
      <span class="label">Prompt</span>
      <textarea
        class="field"
        rows="3"
        bind:value={prompt}
        disabled={running}
        on:keydown={(e) => {
          if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) generate();
        }}
      ></textarea>
    </label>

    <div class="chips" role="group" aria-label="Example prompts">
      {#each PRESETS as p}
        <button
          class="chip"
          class:on={prompt === p.text}
          disabled={running}
          on:click={() => (prompt = p.text)}
        >
          {p.label}
        </button>
      {/each}
    </div>

    <div class="toolbar">
      <label class="control">
        <span class="control-label">
          <span class="label">Temperature</span>
          <b class="mono">{temperature === 0 ? 'greedy' : temperature.toFixed(2)}</b>
        </span>
        <input type="range" min="0" max="1.5" step="0.05" bind:value={temperature} disabled={running} />
      </label>
      <label class="control">
        <span class="control-label">
          <span class="label">Max tokens</span>
          <b class="mono">{maxTokens}</b>
        </span>
        <input type="range" min="8" max="128" step="4" bind:value={maxTokens} disabled={running} />
      </label>
      <label class="control">
        <span class="control-label">
          <span class="label">Pace</span>
          <b class="mono">{paceLabel}</b>
        </span>
        <input type="range" min="0" max="1" step="0.01" bind:value={speed} />
      </label>
      <div class="actions">
        {#if phase === 'ready'}
          <p class="status mono small">
            <span class="led" aria-hidden="true"></span>
            {backendLabel}
          </p>
        {/if}
        {#if running}
          <button class="btn" on:click={stop}>Stop</button>
        {:else}
          <button class="btn btn-primary" on:click={generate} disabled={phase !== 'ready' || !prompt.trim()}>
            Generate
          </button>
        {/if}
      </div>
    </div>
  </div>

  <div class="stage">
    <div class="output card">
      <header class="pane-head">
        <span class="label">Output</span>
        <span class="meta mono">
          {shown.length} token{shown.length === 1 ? '' : 's'}
          {#if doneInfo}
            <span class="sep" aria-hidden="true">·</span>
            {doneInfo.tokensPerSecond.toFixed(1)} /s
            <span class="sep" aria-hidden="true">·</span>
            {doneReason}
          {/if}
        </span>
      </header>
      <div class="output-body" bind:this={outputEl} aria-live="off">
        {#if !promptTokens.length && !shown.length}
          <p class="dim placeholder">
            {phase === 'ready' ? 'Generate to watch each token get chosen.' : 'Load the model, then generate.'}
          </p>
        {:else}
          <p class="stream">
            {#each promptTokens as t, i}<span class="tok prompt" class:alt={i % 2 === 1}>{#if isSpecialToken(t)}<span class="pill">{t}</span>{:else}{t}{/if}</span>{/each}{#if promptTokens.length && shown.length}<span class="turn" aria-hidden="true"></span>{/if}{#each shown as s (s.index)}<button
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
    </div>

    <div class="panel card" bind:this={panelEl}>
      {#if panelStep}
        <header class="pane-head">
          <span class="label">
            {inspecting ? 'Step' : 'Choosing'} {panelStep.index + 1}
          </span>
          <span class="meta mono" title="Shannon entropy of the full distribution">
            {panelStep.entropy.toFixed(2)} bits
          </span>
        </header>
        <ul class="cands">
          {#each panelStep.candidates as c, i (c.id)}
            <li class="cand" class:chosen={c.id === panelStep.chosen.id}>
              <span class="rank mono">{i + 1}</span>
              <span class="ctext mono">{visualizeWhitespace(c.text)}</span>
              <span class="bar" aria-hidden="true"><span class="bar-fill" style="width: {(c.p / maxP) * 100}%"></span></span>
              <span class="cpct mono">{formatPercent(c.p)}</span>
            </li>
          {/each}
          {#if !chosenInList}
            <li class="cand chosen extra">
              <span class="rank mono">+</span>
              <span class="ctext mono">{visualizeWhitespace(panelStep.chosen.text)}</span>
              <span class="bar" aria-hidden="true"><span class="bar-fill" style="width: {(panelStep.chosen.p / maxP) * 100}%"></span></span>
              <span class="cpct mono">{formatPercent(panelStep.chosen.p)}</span>
            </li>
          {/if}
        </ul>
        <p class="dim small hint">
          Highlighted row was sampled{chosenInList ? '' : `, outside the top ${TOP_K}`}.
        </p>
      {:else}
        <header class="pane-head">
          <span class="label">Candidates</span>
        </header>
        <p class="dim placeholder">Top candidates for the token being chosen show up here.</p>
      {/if}
    </div>
  </div>

  <p class="dim small honest">
    Probabilities are the model's softmax at temperature 1, whatever temperature you sample with.
    Attention isn't shown — this ONNX build doesn't expose those weights.
  </p>
</section>

<style>
  .lens {
    display: grid;
    gap: var(--s-5);
    min-width: 0;
  }

  .card {
    padding: var(--s-5);
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .dim {
    color: var(--ink-3);
  }
  .small {
    font-size: var(--text-xs);
  }
  .error {
    margin: 0;
    color: var(--danger, var(--falu));
    font-size: var(--text-sm);
  }

  .load {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: end;
    gap: var(--s-5);
  }
  .load .label {
    margin-bottom: var(--s-2);
  }
  .note {
    max-width: 52ch;
    margin: 0;
    color: var(--ink-2);
    font-size: var(--text-sm);
    line-height: 1.55;
  }
  .load-actions {
    display: grid;
    justify-items: end;
    gap: var(--s-3);
    min-width: min(100%, 220px);
  }
  .meter-block {
    display: grid;
    gap: var(--s-2);
    width: 100%;
  }
  .meter {
    height: 2px;
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

  .controls {
    display: grid;
    gap: var(--s-4);
  }
  .control {
    display: grid;
    gap: var(--s-2);
    min-width: 0;
  }
  .control-label {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-3);
  }
  .control-label b {
    color: var(--ink);
    font-weight: 500;
  }
  textarea.field {
    resize: vertical;
    min-height: 88px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .chip {
    height: 28px;
    padding: 0 var(--s-3);
    border: 1px solid var(--line);
    border-radius: var(--radius-pill);
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    cursor: pointer;
    transition:
      border-color var(--dur-fast) var(--ease),
      color var(--dur-fast) var(--ease),
      background-color var(--dur-fast) var(--ease);
  }
  .chip:hover:not(:disabled),
  .chip.on {
    border-color: var(--fjord);
    background: var(--fjord-wash);
    color: var(--fjord);
  }
  .chip:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .toolbar {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr)) auto;
    align-items: end;
    gap: var(--s-5);
    padding-top: var(--s-4);
    border-top: 1px solid var(--line);
  }
  input[type='range'] {
    width: 100%;
    margin: 0;
    accent-color: var(--fjord);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding-bottom: 2px;
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    margin: 0;
    color: var(--ink-3);
    white-space: nowrap;
  }

  .stage {
    display: grid;
    grid-template-columns: minmax(0, 1.45fr) minmax(280px, 0.9fr);
    gap: var(--s-4);
    align-items: stretch;
  }
  .output,
  .panel {
    min-width: 0;
    padding: 0;
  }
  .output {
    display: flex;
    flex-direction: column;
  }
  .pane-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-3) var(--s-4);
    border-bottom: 1px solid var(--line);
  }
  .meta {
    color: var(--ink-3);
    font-size: var(--text-xs);
  }
  .sep {
    margin: 0 0.35em;
    color: var(--line-strong);
  }
  .output-body {
    flex: 1;
    min-height: 180px;
    max-height: 480px;
    padding: var(--s-4);
    overflow-y: auto;
  }
  .placeholder {
    margin: 0;
    font-size: var(--text-sm);
    line-height: 1.5;
  }
  .turn {
    display: block;
    height: var(--s-4);
  }

  .stream {
    margin: 0;
    font-size: var(--text-md);
    line-height: 1.85;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .tok {
    padding: 1px 2px;
    border: 0;
    border-radius: var(--radius-xs);
    background: var(--surface-2);
    color: var(--ink);
    font: inherit;
    line-height: inherit;
    white-space: pre-wrap;
    box-decoration-break: clone;
    -webkit-box-decoration-break: clone;
  }
  .tok.alt {
    background: var(--fjord-wash);
  }
  .tok.prompt {
    color: var(--ink-2);
  }
  button.tok {
    cursor: pointer;
  }
  button.tok:hover,
  button.tok.active {
    box-shadow: inset 0 0 0 1px var(--fjord);
  }
  button.tok:focus-visible {
    outline: 2px solid var(--fjord);
    outline-offset: 1px;
  }
  .pill {
    display: inline-block;
    padding: 0 var(--s-1);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.6;
    vertical-align: baseline;
  }
  .caret {
    display: inline-block;
    width: 2px;
    height: 1em;
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

  .cands {
    display: grid;
    gap: 2px;
    margin: 0;
    padding: var(--s-3);
    list-style: none;
  }
  .cand {
    display: grid;
    grid-template-columns: 1.25rem minmax(4.5rem, 1.1fr) minmax(4rem, 1fr) 3rem;
    align-items: center;
    gap: var(--s-2);
    min-height: 32px;
    padding: 0 var(--s-2);
    border-radius: var(--radius-xs);
    font-size: var(--text-sm);
  }
  .cand.chosen {
    background: var(--fjord-wash);
    box-shadow: inset 0 0 0 1px var(--fjord);
  }
  .cand.extra {
    margin-top: var(--s-2);
  }
  .rank {
    color: var(--ink-3);
    font-size: var(--text-xs);
  }
  .ctext {
    overflow: hidden;
    color: var(--ink);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .bar {
    height: 6px;
    border-radius: var(--radius-pill);
    background: var(--surface-2);
    overflow: hidden;
  }
  .cand.chosen .bar {
    background: transparent;
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
  }
  .hint {
    margin: 0;
    padding: 0 var(--s-4) var(--s-4);
    line-height: 1.5;
  }
  .panel > .placeholder {
    padding: var(--s-4);
  }

  .honest {
    max-width: 68ch;
    margin: 0;
    line-height: 1.55;
  }

  @media (max-width: 860px) {
    .load {
      grid-template-columns: minmax(0, 1fr);
    }
    .load-actions {
      justify-items: start;
    }
    .toolbar,
    .stage {
      grid-template-columns: minmax(0, 1fr);
    }
    .actions {
      padding-bottom: 0;
    }
    .output-body {
      max-height: 320px;
    }
  }
</style>
