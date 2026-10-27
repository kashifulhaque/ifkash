<svelte:head>
  <title>VM console — Kashif</title>
  <meta name="robots" content="noindex" />
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import { MessageSquare, Layers, SquareTerminal, Activity, Folder, ShieldCheck, RefreshCw } from 'lucide-svelte';
  import VmChat from '$lib/components/vm/VmChat.svelte';
  import VmSessions from '$lib/components/vm/VmSessions.svelte';
  import VmTerminal from '$lib/components/vm/VmTerminal.svelte';
  import VmSystem from '$lib/components/vm/VmSystem.svelte';
  import VmFiles from '$lib/components/vm/VmFiles.svelte';
  import { vmApi, vmSignedOut, VmSignedOut, loginUrl, getVmBase, VM_DEV_BASE, type Health } from '$lib/vmApi';

  type Tab = 'chat' | 'sessions' | 'terminal' | 'system' | 'files';
  const TABS: { id: Tab; label: string; icon: typeof MessageSquare }[] = [
    { id: 'chat', label: 'Chat', icon: MessageSquare },
    { id: 'sessions', label: 'Sessions', icon: Layers },
    { id: 'terminal', label: 'Terminal', icon: SquareTerminal },
    { id: 'system', label: 'System', icon: Activity },
    { id: 'files', label: 'Files', icon: Folder }
  ];

  let status: 'loading' | 'ready' | 'signedout' | 'error' = 'loading';
  let health: Health | null = null;
  let errorMsg = '';
  let base = '';
  let tab: Tab = 'chat';
  // Tabs mount on first visit and then stay mounted, so a run stream or a
  // terminal connection survives switching tabs.
  let visited: Record<Tab, boolean> = { chat: true, sessions: false, terminal: false, system: false, files: false };

  let termSession = 'main';
  let chatPreset: { sessionId: string; cwd: string; nonce: number } | null = null;
  let nonce = 0;
  let kbOpen = false;

  function show(t: Tab) {
    tab = t;
    visited[t] = true;
    visited = visited;
    try { history.replaceState(history.state, '', `#${t}`); } catch {}
  }

  function openTerminal(name: string) {
    termSession = name;
    show('terminal');
  }

  function continueInChat(d: { sessionId: string; cwd: string }) {
    chatPreset = { ...d, nonce: ++nonce };
    show('chat');
  }

  async function boot() {
    status = 'loading';
    errorMsg = '';
    vmSignedOut.set(false);
    try {
      health = await vmApi.health();
      status = 'ready';
    } catch (e) {
      if (e instanceof VmSignedOut) status = 'signedout';
      else {
        errorMsg = (e as Error).message;
        status = 'error';
      }
    }
  }

  // Any call that fails at the network level flips the page to the sign-in card.
  $: if ($vmSignedOut && status === 'ready') status = 'signedout';

  onMount(() => {
    base = getVmBase();
    const h = location.hash.slice(1) as Tab;
    if (TABS.some((t) => t.id === h)) show(h);
    boot();

    const vv = window.visualViewport;
    const onVv = () => { kbOpen = !!vv && window.innerHeight - vv.height > 120; };
    vv?.addEventListener('resize', onVv);
    return () => vv?.removeEventListener('resize', onVv);
  });
</script>

<div class="page">
{#if status === 'loading'}
  <div class="center"><p class="muted"><span class="led led-hollow" aria-hidden="true"></span> Connecting to the VM…</p></div>
{:else if status === 'signedout'}
  <div class="center">
    <div class="card gate">
      <span class="gate-ico" aria-hidden="true"><ShieldCheck size={22} strokeWidth={1.5} /></span>
      <h1 class="gate-title">VM console</h1>
      <p class="gate-body">
        The console is protected by Cloudflare Access. Sign in to control Claude Code on the VM.
      </p>
      <div class="btn-row">
        <a class="btn btn-primary" href={loginUrl()}>Sign in</a>
        <button type="button" class="btn" on:click={boot}><RefreshCw size={14} aria-hidden="true" /> Retry</button>
      </div>
      <p class="gate-note">Agent: <span class="mono">{base}</span>{#if base === VM_DEV_BASE}. If you are developing locally, check that the agent is running.{/if}</p>
    </div>
  </div>
{:else if status === 'error'}
  <div class="center">
    <div class="card gate">
      <h1 class="gate-title">Can't reach the VM</h1>
      <p class="gate-body err" role="alert">{errorMsg}</p>
      <div class="btn-row"><button type="button" class="btn btn-primary" on:click={boot}>Retry</button></div>
    </div>
  </div>
{:else}
  <div class="vm" class:kb={kbOpen}>
    <header class="head">
      <div class="hl">
        <span class="led" aria-hidden="true"></span>
        <h1 class="title">{health?.host ?? 'VM'}</h1>
        {#if base === VM_DEV_BASE}<span class="tag">local agent</span>{/if}
        <a class="home" href="/">ifkash.dev</a>
      </div>
      <p class="hm">{health?.email ?? ''}{health?.claudeVersion ? ` · Claude Code ${health.claudeVersion}` : ''}</p>
    </header>

    <nav class="vm-tabbar" aria-label="VM sections">
      {#each TABS as t}
        <button
          type="button"
          class="tab"
          class:on={tab === t.id}
          aria-current={tab === t.id ? 'page' : undefined}
          on:click={() => show(t.id)}
        >
          <svelte:component this={t.icon} size={20} strokeWidth={1.6} aria-hidden="true" />
          <span>{t.label}</span>
        </button>
      {/each}
    </nav>

    <div class="content">
      {#if visited.chat}<div class="pane" hidden={tab !== 'chat'}><VmChat active={tab === 'chat'} preset={chatPreset} /></div>{/if}
      {#if visited.sessions}<div class="pane" hidden={tab !== 'sessions'}><VmSessions active={tab === 'sessions'} on:terminal={(e) => openTerminal(e.detail.name)} on:continue={(e) => continueInChat(e.detail)} /></div>{/if}
      {#if visited.terminal}<div class="pane" hidden={tab !== 'terminal'}><VmTerminal active={tab === 'terminal'} bind:session={termSession} /></div>{/if}
      {#if visited.system}<div class="pane" hidden={tab !== 'system'}><VmSystem active={tab === 'system'} /></div>{/if}
      {#if visited.files}<div class="pane" hidden={tab !== 'files'}><VmFiles /></div>{/if}
    </div>
  </div>
{/if}
</div>

<style>
  /* The layout renders /vm without the site chrome, so the page sets its own gutter. */
  .page { max-width: 1180px; margin: 0 auto; padding: 0 var(--s-4); }
  .home { margin-left: auto; color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); text-decoration: none; }
  .home:hover { color: var(--fjord); }
  .center { display: grid; place-items: center; min-height: 60vh; padding: var(--s-6) 0; }
  .muted { display: inline-flex; align-items: center; gap: var(--s-2); margin: 0; color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); }
  .gate { width: min(100%, 420px); padding: var(--s-5); display: flex; flex-direction: column; gap: var(--s-3); }
  .gate-ico { display: inline-flex; color: var(--fjord); }
  .gate-title { margin: 0; font-family: var(--font-display); font-size: var(--text-lg); font-weight: 500; letter-spacing: -0.02em; }
  .gate-body { margin: 0; color: var(--ink-2); font-size: 15px; line-height: 1.55; }
  .gate-body.err { color: var(--falu); overflow-wrap: anywhere; }
  .gate-note { margin: 0; color: var(--ink-3); font-size: var(--text-xs); line-height: 1.5; }
  .mono { font-family: var(--font-mono); }

  .vm {
    --vm-tabbar-h: calc(60px + env(safe-area-inset-bottom, 0px));
    padding-top: var(--s-4);
    padding-bottom: var(--vm-tabbar-h);
  }
  .vm.kb { --vm-tabbar-h: 0px; }
  .head { display: flex; flex-direction: column; gap: 2px; margin-bottom: var(--s-3); }
  .hl { display: flex; align-items: center; gap: var(--s-2); }
  .title { margin: 0; font-family: var(--font-display); font-size: var(--text-lg); font-weight: 500; letter-spacing: -0.02em; line-height: 1.2; }
  .hm { margin: 0; color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); overflow-wrap: anywhere; }

  .vm-tabbar {
    position: fixed;
    right: 0;
    bottom: 0;
    left: 0;
    z-index: 30;
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    padding-bottom: env(safe-area-inset-bottom, 0px);
    border-top: 1px solid var(--line);
    background: var(--bg);
  }
  .vm.kb .vm-tabbar { display: none; }
  .tab {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    min-height: 60px;
    padding: 0;
    border: 0;
    background: none;
    color: var(--ink-3);
    font-size: 11px;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease);
  }
  .tab.on { color: var(--fjord); box-shadow: inset 0 2px 0 var(--fjord); }
  .tab:hover { color: var(--ink); }
  .tab.on:hover { color: var(--fjord); }
  .tab:focus-visible { outline: 2px solid var(--fjord); outline-offset: -3px; }

  .content { min-width: 0; }
  .pane[hidden] { display: none; }

  @media (min-width: 900px) {
    .vm {
      --vm-tabbar-h: 0px;
      display: grid;
      grid-template-columns: 176px minmax(0, 1fr);
      grid-template-areas: 'head head' 'nav content';
      column-gap: var(--s-6);
      padding-top: var(--s-6);
      padding-bottom: 0;
    }
    .head { grid-area: head; flex-direction: row; align-items: baseline; justify-content: space-between; padding-bottom: var(--s-3); border-bottom: 1px solid var(--line); margin-bottom: var(--s-5); }
    .page { padding: 0 var(--s-6); }
    .vm-tabbar {
      position: sticky;
      top: var(--s-5);
      grid-area: nav;
      align-self: start;
      display: flex;
      flex-direction: column;
      padding: 0;
      border: 0;
      background: none;
    }
    .vm.kb .vm-tabbar { display: flex; }
    .tab { flex-direction: row; justify-content: flex-start; gap: var(--s-3); min-height: 40px; padding: 0 var(--s-3); border-radius: var(--radius-sm); font-size: 14px; }
    .tab.on { background: var(--fjord-wash); box-shadow: none; }
    .content { grid-area: content; }
  }
  @media (prefers-reduced-motion: reduce) { .tab { transition: none; } }
</style>
