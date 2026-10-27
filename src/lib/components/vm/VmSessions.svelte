<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy, tick } from 'svelte';
  import { Terminal as TermIcon, Eye, Trash2, Plus, RefreshCw, Send } from 'lucide-svelte';
  import VmTranscript from './VmTranscript.svelte';
  import {
    vmApi,
    VmSignedOut,
    type TmuxSession,
    type TmuxKey,
    type Project,
    type TranscriptSummary
  } from '$lib/vmApi';
  import { clip, relTime } from '$lib/vmFormat';

  export let active = true;

  const dispatch = createEventDispatcher<{
    terminal: { name: string };
    continue: { sessionId: string; cwd: string };
  }>();

  // ---- live tmux -----------------------------------------------------------

  let tmux: TmuxSession[] = [];
  let tmuxLoading = true;
  let tmuxError = '';

  async function loadTmux() {
    try {
      tmux = (await vmApi.listTmux()).sessions;
      tmuxError = '';
    } catch (e) {
      if (!(e instanceof VmSignedOut)) tmuxError = (e as Error).message;
    } finally {
      tmuxLoading = false;
    }
  }

  let peek: string | null = null;
  let peekText = '';
  let peekLoading = false;
  let peekError = '';
  let peekBox: HTMLPreElement | undefined;
  let sendText = '';

  async function capture(name: string) {
    peekLoading = true;
    try {
      peekText = (await vmApi.captureTmux(name, 300)).text;
      peekError = '';
      await tick();
      if (peekBox) peekBox.scrollTop = peekBox.scrollHeight;
    } catch (e) {
      if (!(e instanceof VmSignedOut)) peekError = (e as Error).message;
    } finally {
      peekLoading = false;
    }
  }

  function togglePeek(name: string) {
    if (peek === name) { peek = null; return; }
    peek = name;
    peekText = '';
    sendText = '';
    capture(name);
  }

  async function sendTmux(body: { text?: string; enter?: boolean; key?: TmuxKey }) {
    if (!peek) return;
    const name = peek;
    try {
      await vmApi.sendTmux({ name, ...body });
      peekError = '';
      setTimeout(() => peek === name && capture(name), 400);
    } catch (e) {
      if (!(e instanceof VmSignedOut)) peekError = (e as Error).message;
    }
  }

  function submitText() {
    if (!sendText) return;
    sendTmux({ text: sendText, enter: true });
    sendText = '';
  }

  const KEYS: { key: TmuxKey; label: string; aria: string }[] = [
    { key: 'Escape', label: 'Esc', aria: 'Escape' },
    { key: 'BTab', label: 'Shift+Tab', aria: 'Shift Tab' },
    { key: 'C-c', label: 'Ctrl+C', aria: 'Control C' },
    { key: 'Up', label: '↑', aria: 'Up arrow' },
    { key: 'Down', label: '↓', aria: 'Down arrow' },
    { key: 'Left', label: '←', aria: 'Left arrow' },
    { key: 'Right', label: '→', aria: 'Right arrow' }
  ];

  let confirmKill: string | null = null;
  async function kill(name: string) {
    if (confirmKill !== name) {
      confirmKill = name;
      return;
    }
    confirmKill = null;
    try {
      await vmApi.killTmux(name);
      if (peek === name) peek = null;
      await loadTmux();
    } catch (e) {
      if (!(e instanceof VmSignedOut)) tmuxError = (e as Error).message;
    }
  }

  // New session form
  let newName = '';
  let newCwd = '';
  let newStart: 'claude' | 'shell' = 'claude';
  let creating = false;
  let formError = '';
  $: nameOk = /^[A-Za-z0-9_-]{1,32}$/.test(newName);

  async function create() {
    if (!nameOk || creating) return;
    creating = true;
    formError = '';
    try {
      await vmApi.newTmux({ name: newName, cwd: newCwd.trim() || undefined, start: newStart });
      newName = '';
      await loadTmux();
    } catch (e) {
      if (!(e instanceof VmSignedOut)) formError = (e as Error).message;
    } finally {
      creating = false;
    }
  }

  // ---- transcripts ---------------------------------------------------------

  let projects: Project[] = [];
  let projectCwd = '';
  let transcripts: TranscriptSummary[] = [];
  let trLoading = true;
  let trError = '';
  let open: TranscriptSummary | null = null;

  async function loadProjects() {
    try { projects = (await vmApi.listProjects()).projects; } catch {}
  }
  async function loadTranscripts() {
    trLoading = true;
    try {
      transcripts = (await vmApi.listSessions(projectCwd || undefined)).sessions;
      trError = '';
    } catch (e) {
      if (!(e instanceof VmSignedOut)) trError = (e as Error).message;
    } finally {
      trLoading = false;
    }
  }

  let timer: ReturnType<typeof setInterval>;
  onMount(() => {
    loadTmux();
    loadProjects();
    loadTranscripts();
    timer = setInterval(() => {
      if (active && document.visibilityState === 'visible') loadTmux();
    }, 10000);
  });
  onDestroy(() => clearInterval(timer));
</script>

{#if open}
  <VmTranscript
    id={open.id}
    title={open.title}
    cwd={open.cwd}
    on:back={() => (open = null)}
    on:continue={(e) => dispatch('continue', e.detail)}
  />
{:else}
  <section class="block" aria-labelledby="live-h">
    <div class="bh">
      <h2 id="live-h">Live sessions</h2>
      <button type="button" class="btn btn-sm btn--ghost" on:click={loadTmux} aria-label="Refresh live sessions"><RefreshCw size={14} aria-hidden="true" /></button>
    </div>

    {#if tmuxError}<p class="err" role="alert">{tmuxError}</p>{/if}
    {#if tmuxLoading}
      <p class="muted">Loading…</p>
    {:else if tmux.length === 0}
      <p class="muted">No tmux sessions running.</p>
    {:else}
      <ul class="cards">
        {#each tmux as s (s.name)}
          <li class="card tcard">
            <div class="ttop">
              <span class="led" class:led-hollow={!s.attached} aria-hidden="true"></span>
              <strong class="tn">{s.name}</strong>
              <span class="tag">{s.command || 'shell'}</span>
            </div>
            <p class="tmeta">
              <span class="path">{s.cwd}</span>
              <span>{s.attached ? 'attached' : 'detached'} · {relTime(s.createdAt)}</span>
            </p>
            <div class="btn-row">
              <button type="button" class="btn btn-sm" on:click={() => dispatch('terminal', { name: s.name })}><TermIcon size={14} aria-hidden="true" /> Open in terminal</button>
              <button type="button" class="btn btn-sm" aria-pressed={peek === s.name} on:click={() => togglePeek(s.name)}><Eye size={14} aria-hidden="true" /> Peek</button>
              <button
                type="button"
                class="btn btn-sm kill"
                class:armed={confirmKill === s.name}
                on:click={() => kill(s.name)}
                on:blur={() => confirmKill === s.name && (confirmKill = null)}
              >
                <Trash2 size={14} aria-hidden="true" /> {confirmKill === s.name ? 'Confirm kill' : 'Kill'}
              </button>
            </div>

            {#if peek === s.name}
              <div class="peek">
                <div class="peekbar">
                  <span>{peekLoading ? 'Refreshing…' : 'Pane output'}</span>
                  <button type="button" class="btn btn-sm btn--ghost" on:click={() => capture(s.name)} disabled={peekLoading}><RefreshCw size={13} aria-hidden="true" /> Refresh</button>
                </div>
                <!-- svelte-ignore a11y-no-noninteractive-tabindex -->
                <pre class="pane" bind:this={peekBox} tabindex="0" aria-label="Output of {s.name}">{peekText || ' '}</pre>
                {#if peekError}<p class="err" role="alert">{peekError}</p>{/if}
                <form class="sendrow" on:submit|preventDefault={submitText}>
                  <label class="sr" for="peek-send">Text to send to {s.name}</label>
                  <input id="peek-send" class="field mono" bind:value={sendText} placeholder="Type, then Enter" autocapitalize="off" autocomplete="off" spellcheck="false" />
                  <button type="submit" class="btn btn-primary" aria-label="Send text and press Enter" disabled={!sendText}><Send size={15} aria-hidden="true" /></button>
                </form>
                <div class="keys">
                  <button type="button" class="btn btn-sm" on:click={() => sendTmux({ key: 'Enter' })}>Enter</button>
                  {#each KEYS as k}
                    <button type="button" class="btn btn-sm" aria-label={k.aria} on:click={() => sendTmux({ key: k.key })}>{k.label}</button>
                  {/each}
                </div>
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}

    <form class="card newform" on:submit|preventDefault={create}>
      <h3>New session</h3>
      <div class="grid">
        <label class="lbl">Name
          <input class="field mono" bind:value={newName} maxlength="32" placeholder="work" autocapitalize="off" autocomplete="off" spellcheck="false" aria-invalid={newName !== '' && !nameOk} />
        </label>
        <label class="lbl">Start
          <select class="field" bind:value={newStart}>
            <option value="claude">Claude Code</option>
            <option value="shell">Shell</option>
          </select>
        </label>
        <label class="lbl wide">Working directory
          <input class="field mono" bind:value={newCwd} placeholder="Home directory" autocapitalize="off" autocomplete="off" spellcheck="false" />
        </label>
      </div>
      {#if newName !== '' && !nameOk}<p class="hint">Use letters, numbers, underscores, and hyphens, up to 32 characters.</p>{/if}
      {#if formError}<p class="err" role="alert">{formError}</p>{/if}
      <div class="btn-row"><button type="submit" class="btn btn-primary" disabled={!nameOk || creating}><Plus size={15} aria-hidden="true" /> Create session</button></div>
    </form>
  </section>

  <section class="block" aria-labelledby="tr-h">
    <div class="bh">
      <h2 id="tr-h">Claude transcripts</h2>
      <label class="filter">
        <span class="sr">Project</span>
        <select class="field" bind:value={projectCwd} on:change={loadTranscripts}>
          <option value="">All projects</option>
          {#each projects as p}<option value={p.cwd}>{p.cwd} ({p.sessions})</option>{/each}
        </select>
      </label>
    </div>
    {#if trError}<p class="err" role="alert">{trError}</p>{/if}
    {#if trLoading}
      <p class="muted">Loading…</p>
    {:else if transcripts.length === 0}
      <p class="muted">No transcripts found.</p>
    {:else}
      <ol class="rows trlist">
        {#each transcripts as t (t.id)}
          <li>
            <button type="button" class="trrow" on:click={() => (open = t)}>
              <span class="trt">{clip(t.title || 'Untitled session', 120)}</span>
              <span class="trm">
                <span class="path">{t.cwd}</span>
                <span>{relTime(t.updatedAt)} · {t.messages} msgs</span>
              </span>
            </button>
          </li>
        {/each}
      </ol>
    {/if}
  </section>
{/if}

<style>
  .block { display: flex; flex-direction: column; gap: var(--s-3); min-width: 0; margin-bottom: var(--s-6); }
  .bh { display: flex; align-items: center; justify-content: space-between; gap: var(--s-3); }
  h2 { margin: 0; font-family: var(--font-display); font-size: var(--text-md); font-weight: 500; letter-spacing: -0.01em; }
  h3 { margin: 0 0 var(--s-3); font-size: 15px; font-weight: 500; }
  .muted { margin: 0; color: var(--ink-3); font-size: var(--text-sm); }
  .err { margin: 0; color: var(--falu); font-size: var(--text-sm); }
  .hint { margin: 0; color: var(--ink-3); font-size: var(--text-xs); }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }

  .cards { list-style: none; margin: 0; padding: 0; display: grid; gap: var(--s-3); grid-template-columns: repeat(auto-fill, minmax(min(100%, 340px), 1fr)); }
  .tcard { padding: var(--s-3); display: flex; flex-direction: column; gap: var(--s-2); min-width: 0; }
  .ttop { display: flex; align-items: center; gap: var(--s-2); min-width: 0; }
  .tn { font-family: var(--font-mono); font-size: 14px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; }
  .ttop .tag { margin-left: auto; }
  .tmeta { display: flex; flex-direction: column; gap: 2px; margin: 0; color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); }
  .path { overflow-wrap: anywhere; }
  .btn-row .btn { min-height: 40px; }
  .kill:hover:not(:disabled) { border-color: var(--falu); color: var(--falu); }
  .kill.armed { border-color: var(--falu); background: var(--falu); color: var(--ink-inverse); }
  .kill.armed:hover:not(:disabled) { background: var(--falu); color: var(--ink-inverse); }

  .peek { display: flex; flex-direction: column; gap: var(--s-2); margin-top: var(--s-2); padding-top: var(--s-3); border-top: 1px solid var(--line); }
  .peekbar { display: flex; align-items: center; justify-content: space-between; color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); }
  .pane {
    margin: 0;
    height: 260px;
    padding: var(--s-3);
    overflow: auto;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--code-surface);
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.45;
    white-space: pre;
  }
  .pane:focus-visible { outline: 2px solid var(--fjord); outline-offset: 1px; }
  .sendrow { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: var(--s-2); }
  .sendrow .btn { width: 44px; padding: 0; }
  .field.mono, .mono { font-family: var(--font-mono); font-size: 14px; }
  .keys { display: flex; flex-wrap: wrap; gap: var(--s-2); }
  .keys .btn { min-height: 40px; min-width: 44px; font-family: var(--font-mono); }

  .newform { padding: var(--s-4); }
  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-3); margin-bottom: var(--s-3); }
  .lbl { display: flex; flex-direction: column; gap: 4px; min-width: 0; color: var(--ink-3); font-size: var(--text-xs); }
  .wide { grid-column: 1 / -1; }
  .newform .btn-row { margin-top: var(--s-3); }

  .filter { min-width: 0; max-width: 60%; }
  .filter .field { font-size: 14px; }
  .trlist { margin: 0; }
  .trrow { display: flex; flex-direction: column; gap: 4px; width: 100%; padding: var(--s-3) 0; border: 0; background: none; text-align: left; cursor: pointer; min-height: 56px; }
  .trrow:hover .trt { color: var(--fjord); }
  .trrow:focus-visible { outline: 2px solid var(--fjord); outline-offset: 2px; }
  .trt { color: var(--ink); font-size: 15px; line-height: 1.35; overflow-wrap: anywhere; }
  .trm { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 2px var(--s-3); color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); }
</style>
