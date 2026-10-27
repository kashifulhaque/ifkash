<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { Send, Square, Plus, TriangleAlert, ChevronDown, RefreshCw } from 'lucide-svelte';
  import Conversation from './Conversation.svelte';
  import {
    vmApi,
    openRunStream,
    VmSignedOut,
    type PermissionMode,
    type Run
  } from '$lib/vmApi';
  import { applyEvent, fromMessages, userItem, type ConvItem } from '$lib/vmConversation';
  import { clip, fmtCost, relTime } from '$lib/vmFormat';

  export let active = true;
  /** Set by the Sessions tab to continue a transcript here. `nonce` makes repeats fire. */
  export let preset: { sessionId: string; cwd: string; nonce: number } | null = null;

  const MODES: { value: PermissionMode; label: string; hint: string }[] = [
    { value: 'plan', label: 'Plan', hint: 'Read-only. Claude proposes a plan and changes nothing.' },
    { value: 'acceptEdits', label: 'Accept edits', hint: 'Edits files without asking. Other risky tools may be blocked.' },
    { value: 'auto', label: 'Auto', hint: 'A classifier approves safe actions and blocks risky ones.' },
    { value: 'bypassPermissions', label: 'Bypass permissions', hint: 'Runs every tool with no checks. Use with care.' },
    { value: 'dontAsk', label: "Don't ask", hint: 'Denies anything not already allowed, without prompting.' },
    { value: 'manual', label: 'Manual', hint: "Needs approval that a headless run can't give, so most tools are blocked." }
  ];
  const MODELS = [
    { value: '', label: 'Default' },
    { value: 'sonnet', label: 'Sonnet' },
    { value: 'opus', label: 'Opus' },
    { value: 'haiku', label: 'Haiku' }
  ];

  let prompt = '';
  let cwd = '';
  let mode: PermissionMode = 'acceptEdits';
  let model = '';
  let showOpts = true;

  let baseItems: ConvItem[] = [];
  let runItems: ConvItem[] = [];
  $: items = [...baseItems, ...runItems];

  let sessionId: string | null = null;
  let runId: string | null = null;
  let running = false;
  let es: EventSource | null = null;
  let error = '';
  let starting = false;

  let cwds: string[] = [];
  let runs: Run[] = [];
  let runsLoading = false;

  $: modeInfo = MODES.find((m) => m.value === mode) ?? MODES[1];
  $: if (items.length > 0 && showOpts && !initialOptsSet) { showOpts = false; initialOptsSet = true; }
  let initialOptsSet = false;

  // ---- scrolling -----------------------------------------------------------

  let stick = true;
  function nearBottom() {
    const el = document.documentElement;
    return el.scrollHeight - (window.scrollY + window.innerHeight) < 140;
  }
  function onScroll() { stick = nearBottom(); }
  async function followBottom() {
    if (!stick || !active) return;
    await tick();
    window.scrollTo({ top: document.documentElement.scrollHeight });
  }
  $: items, followBottom();

  // ---- runs ----------------------------------------------------------------

  async function refreshRuns() {
    runsLoading = true;
    try {
      runs = (await vmApi.listRuns()).runs;
    } catch (e) {
      if (!(e instanceof VmSignedOut)) error = (e as Error).message;
    } finally {
      runsLoading = false;
    }
  }

  async function loadCwds() {
    try {
      const { projects } = await vmApi.listProjects();
      cwds = projects.map((p) => p.cwd);
    } catch {}
  }

  function closeStream() {
    if (es) { es.close(); es = null; }
  }

  function finish() {
    closeStream();
    running = false;
    refreshRuns();
  }

  function attach(id: string) {
    closeStream();
    const source = openRunStream(id);
    es = source;
    // The agent replays a run's events on every connect, so start from empty.
    source.onopen = () => { runItems = []; };
    source.onmessage = (m) => {
      let ev: any;
      try { ev = JSON.parse(m.data); } catch { return; }
      const sid = applyEvent(runItems, ev);
      if (sid) sessionId = sid;
      runItems = runItems;
      if (ev && ev.type === 'agent.exit') finish();
    };
    source.onerror = async () => {
      if (es !== source) return;
      // EventSource retries by itself; stop only if the run is no longer live.
      try {
        const { runs: rs } = await vmApi.listRuns();
        runs = rs;
        const r = rs.find((x) => x.runId === id);
        if (r && r.status !== 'running') finish();
      } catch {}
    };
  }

  async function send() {
    const text = prompt.trim();
    if (!text || running || starting) return;
    error = '';
    starting = true;
    try {
      const { runId: id } = await vmApi.createRun({
        prompt: text,
        cwd: cwd.trim() || undefined,
        sessionId: sessionId || undefined,
        permissionMode: mode,
        model: model || undefined
      });
      baseItems = [...baseItems, ...runItems, userItem(text)];
      runItems = [];
      prompt = '';
      runId = id;
      running = true;
      stick = true;
      attach(id);
      refreshRuns();
    } catch (e) {
      if (!(e instanceof VmSignedOut)) error = (e as Error).message;
    } finally {
      starting = false;
    }
  }

  async function stop() {
    if (!runId) return;
    try { await vmApi.stopRun(runId); } catch (e) {
      if (!(e instanceof VmSignedOut)) error = (e as Error).message;
    }
  }

  function reset() {
    closeStream();
    baseItems = [];
    runItems = [];
    sessionId = null;
    runId = null;
    running = false;
    error = '';
    showOpts = true;
    initialOptsSet = false;
  }

  function newChat() {
    reset();
    prompt = '';
  }

  function openRun(r: Run) {
    reset();
    sessionId = r.sessionId;
    runId = r.runId;
    cwd = r.cwd;
    mode = r.permissionMode;
    model = r.model ?? '';
    baseItems = [userItem(r.prompt)];
    running = r.status === 'running';
    stick = true;
    attach(r.runId);
  }

  let lastNonce = 0;
  async function applyPreset(p: { sessionId: string; cwd: string }) {
    reset();
    sessionId = p.sessionId;
    cwd = p.cwd;
    try {
      const t = await vmApi.getSession(p.sessionId);
      if (sessionId === p.sessionId && !runId) baseItems = fromMessages(t.messages);
    } catch {}
    initialOptsSet = true;
    showOpts = false;
  }
  $: if (preset && preset.nonce !== lastNonce) {
    lastNonce = preset.nonce;
    applyPreset(preset);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      send();
    }
  }

  onMount(() => {
    window.addEventListener('scroll', onScroll, { passive: true });
    refreshRuns();
    loadCwds();
  });
  onDestroy(() => {
    closeStream();
    if (typeof window !== 'undefined') window.removeEventListener('scroll', onScroll);
  });
</script>

<section class="chat" aria-label="Chat">
  <div class="bar">
    <div class="ctx">
      {#if sessionId}
        <span class="tag" title={sessionId}>session {sessionId.slice(0, 8)}</span>
      {:else}
        <span class="tag">new chat</span>
      {/if}
      {#if cwd}<span class="cwd">{cwd}</span>{/if}
    </div>
    <button type="button" class="btn btn-sm" on:click={newChat}><Plus size={14} aria-hidden="true" /> New chat</button>
  </div>

  <details class="runs" open={items.length === 0}>
    <summary>
      <ChevronDown size={14} class="chev" aria-hidden="true" /> Recent runs{runs.length ? ` (${runs.length})` : ''}
    </summary>
    <div class="runs-body">
      {#if runs.length === 0}
        <p class="muted">{runsLoading ? 'Loading…' : 'No runs yet.'}</p>
      {:else}
        <ul class="runlist">
          {#each runs.slice(0, 12) as r (r.runId)}
            <li>
              <button type="button" class="runbtn" class:current={r.runId === runId} on:click={() => openRun(r)}>
                <span class="led" class:led-hollow={r.status !== 'running'} class:bad={r.status === 'error'} aria-hidden="true"></span>
                <span class="rp">{clip(r.prompt, 90)}</span>
                <span class="rm">{r.status}{r.costUsd ? ` · ${fmtCost(r.costUsd)}` : ''} · {relTime(r.startedAt)}</span>
              </button>
            </li>
          {/each}
        </ul>
        <button type="button" class="btn btn-sm btn--ghost" on:click={refreshRuns} disabled={runsLoading}>
          <RefreshCw size={13} aria-hidden="true" /> Refresh
        </button>
      {/if}
    </div>
  </details>

  <div class="thread">
    {#if items.length === 0}
      <p class="empty">Tell Claude what to do on the VM. Pick a working directory and permission mode below.</p>
    {:else}
      <Conversation {items} {running} />
    {/if}
    {#if error}<p class="err" role="alert">{error}</p>{/if}
  </div>

  <div class="composer">
    <button type="button" class="opts-toggle" aria-expanded={showOpts} on:click={() => (showOpts = !showOpts)}>
      <ChevronDown size={14} class={showOpts ? 'chev open' : 'chev'} aria-hidden="true" />
      <span class="chips">
        <span>{modeInfo.label}</span><span>{MODELS.find((m) => m.value === model)?.label}</span><span>{cwd || 'Home'}</span>
      </span>
    </button>

    {#if showOpts}
      <div class="opts">
        <label class="lbl">
          Working directory
          <input
            class="field mono"
            list="vm-cwds"
            bind:value={cwd}
            placeholder="Home directory"
            disabled={!!sessionId}
            autocapitalize="off"
            autocomplete="off"
            spellcheck="false"
          />
          <datalist id="vm-cwds">{#each cwds as c}<option value={c}></option>{/each}</datalist>
        </label>
        <div class="two">
          <label class="lbl">
            Permission mode
            <select class="field" bind:value={mode}>
              {#each MODES as m}<option value={m.value}>{m.label}</option>{/each}
            </select>
          </label>
          <label class="lbl">
            Model
            <select class="field" bind:value={model}>
              {#each MODELS as m}<option value={m.value}>{m.label}</option>{/each}
            </select>
          </label>
        </div>
        <p class="hint" class:danger={mode === 'bypassPermissions'}>
          {#if mode === 'bypassPermissions'}<TriangleAlert size={14} aria-hidden="true" />{/if}
          {modeInfo.hint}
        </p>
        {#if sessionId}<p class="hint">The directory is fixed while you continue a session.</p>{/if}
      </div>
    {/if}

    <div class="entry" class:danger={mode === 'bypassPermissions'}>
      <label class="sr" for="vm-prompt">Prompt</label>
      <textarea
        id="vm-prompt"
        class="field"
        rows="3"
        bind:value={prompt}
        on:keydown={onKey}
        placeholder={sessionId ? 'Follow up…' : 'What should Claude do?'}
      ></textarea>
      {#if running}
        <button type="button" class="btn send stop" on:click={stop} aria-label="Stop run"><Square size={16} aria-hidden="true" /></button>
      {:else}
        <button type="button" class="btn btn-primary send" on:click={send} disabled={!prompt.trim() || starting} aria-label="Send prompt">
          <Send size={16} aria-hidden="true" />
        </button>
      {/if}
    </div>
    <p class="kbd">Cmd or Ctrl + Enter to send</p>
  </div>
</section>

<style>
  .chat { display: flex; flex-direction: column; min-width: 0; gap: var(--s-3); }
  .bar { display: flex; align-items: center; justify-content: space-between; gap: var(--s-3); }
  .ctx { display: flex; align-items: center; gap: var(--s-2); min-width: 0; }
  .cwd {
    min-width: 0;
    overflow: hidden;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .muted { margin: 0; color: var(--ink-3); font-size: var(--text-sm); }

  .runs { border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--surface); }
  .runs summary {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    min-height: 40px;
    padding: 0 var(--s-3);
    color: var(--ink-2);
    font-size: var(--text-sm);
    cursor: pointer;
    list-style: none;
  }
  .runs summary::-webkit-details-marker { display: none; }
  .runs summary:focus-visible { outline: 2px solid var(--fjord); outline-offset: -2px; }
  .runs summary :global(.chev) { transition: transform var(--dur-fast) var(--ease); transform: rotate(-90deg); }
  .runs[open] summary :global(.chev) { transform: none; }
  .runs-body { padding: 0 var(--s-3) var(--s-3); display: flex; flex-direction: column; gap: var(--s-2); align-items: flex-start; }
  .runlist { list-style: none; margin: 0; padding: 0; width: 100%; border-top: 1px solid var(--line); }
  .runlist li { border-bottom: 1px solid var(--line); }
  .runbtn {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    column-gap: var(--s-2);
    align-items: center;
    width: 100%;
    min-height: 48px;
    padding: var(--s-2) 0;
    border: 0;
    background: none;
    text-align: left;
    cursor: pointer;
  }
  .runbtn:hover .rp, .runbtn.current .rp { color: var(--fjord); }
  .runbtn:focus-visible { outline: 2px solid var(--fjord); outline-offset: 2px; }
  .rp { color: var(--ink); font-size: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rm { grid-column: 2; color: var(--ink-3); font-family: var(--font-mono); font-size: 11px; }
  .led.bad { background: var(--falu); box-shadow: none; }

  .thread { min-height: 120px; padding-bottom: var(--s-4); }
  .empty { margin: var(--s-5) 0; color: var(--ink-3); font-size: var(--text-sm); text-align: center; }
  .err { margin: var(--s-3) 0 0; color: var(--falu); font-size: var(--text-sm); }

  .composer {
    position: sticky;
    bottom: var(--vm-tabbar-h, 0px);
    z-index: 5;
    margin: 0 calc(-1 * var(--gutter));
    padding: var(--s-2) var(--gutter) var(--s-2);
    border-top: 1px solid var(--line);
    background: var(--bg);
  }
  .opts-toggle {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    width: 100%;
    min-height: 32px;
    padding: 0;
    border: 0;
    background: none;
    color: var(--ink-3);
    font-size: var(--text-xs);
    cursor: pointer;
    text-align: left;
  }
  .opts-toggle:focus-visible { outline: 2px solid var(--fjord); outline-offset: 2px; }
  .opts-toggle :global(.chev) { transition: transform var(--dur-fast) var(--ease); }
  .opts-toggle :global(.chev.open) { transform: rotate(180deg); }
  .chips { display: flex; gap: var(--s-2); min-width: 0; font-family: var(--font-mono); }
  .chips span { white-space: nowrap; }
  .chips span:last-child { min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .chips span + span::before { content: '·'; margin-right: var(--s-2); }
  .opts { display: flex; flex-direction: column; gap: var(--s-2); padding-bottom: var(--s-2); }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-2); }
  .lbl { display: flex; flex-direction: column; gap: 4px; min-width: 0; color: var(--ink-3); font-size: var(--text-xs); }
  .field:focus-visible { outline: none; }
  .mono { font-family: var(--font-mono); font-size: 13px; }
  .hint {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    margin: 0;
    color: var(--ink-3);
    font-size: var(--text-xs);
    line-height: 1.4;
  }
  .hint.danger { color: var(--falu); }

  .entry { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: var(--s-2); align-items: end; }
  .entry.danger textarea { border-color: var(--falu); }
  .entry textarea { resize: vertical; min-height: 76px; max-height: 40vh; font-size: 16px; }
  .send { width: 48px; min-height: 48px; padding: 0; }
  .stop { border-color: var(--falu); color: var(--falu); }
  .kbd { display: none; margin: 4px 0 0; color: var(--ink-3); font-family: var(--font-mono); font-size: 11px; }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }
  @media (hover: hover) and (pointer: fine) { .kbd { display: block; } }
  @media (prefers-reduced-motion: reduce) {
    .runs summary :global(.chev), .opts-toggle :global(.chev) { transition: none; }
  }
</style>
