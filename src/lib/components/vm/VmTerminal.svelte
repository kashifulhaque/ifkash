<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { RefreshCw, CornerDownLeft } from 'lucide-svelte';
  import '@xterm/xterm/css/xterm.css';
  import type { Terminal } from '@xterm/xterm';
  import type { FitAddon } from '@xterm/addon-fit';
  import { vmApi, openTerminalSocket, VmSignedOut } from '$lib/vmApi';

  export let active = true;
  /** tmux session to attach. Changing it reconnects. */
  export let session = 'main';

  type ConnState = 'connecting' | 'connected' | 'closed';

  let panel: HTMLDivElement;
  let host: HTMLDivElement;
  let keybar: HTMLDivElement;
  let term: Terminal | null = null;
  let fit: FitAddon | null = null;
  let ws: WebSocket | null = null;
  let state: ConnState = 'closed';
  let ctrl = false;
  let names: string[] = [];
  let panelHeight = 420;
  let ready = false;
  let ro: ResizeObserver | null = null;
  let themeObserver: MutationObserver | null = null;

  const enc = new TextEncoder();

  $: options = Array.from(new Set(['main', ...names, session]));

  async function loadNames() {
    try { names = (await vmApi.listTmux()).sessions.map((s) => s.name); } catch (e) {
      if (!(e instanceof VmSignedOut)) names = names;
    }
  }

  // ---- theme ---------------------------------------------------------------

  let probe: CanvasRenderingContext2D | null = null;
  /** Resolve any CSS color (including color-mix) to a value xterm can parse. */
  function resolveColor(value: string, fallback: string): string {
    if (!value) return fallback;
    try {
      if (!probe) {
        const c = document.createElement('canvas');
        c.width = c.height = 1;
        probe = c.getContext('2d', { willReadFrequently: true });
      }
      if (!probe) return fallback;
      probe.clearRect(0, 0, 1, 1);
      probe.fillStyle = '#000';
      probe.fillStyle = value;
      probe.fillRect(0, 0, 1, 1);
      const [r, g, b, a] = probe.getImageData(0, 0, 1, 1).data;
      return a === 255 ? `rgb(${r}, ${g}, ${b})` : `rgba(${r}, ${g}, ${b}, ${(a / 255).toFixed(3)})`;
    } catch {
      return fallback;
    }
  }

  function readTheme() {
    const cs = getComputedStyle(document.documentElement);
    const v = (name: string) => cs.getPropertyValue(name).trim();
    const c = (name: string, fb: string) => resolveColor(v(name), fb);
    const bg = c('--surface-2', '#ebe6dc');
    const ink = c('--ink', '#25221f');
    return {
      background: bg,
      foreground: ink,
      cursor: c('--fjord', ink),
      cursorAccent: bg,
      selectionBackground: resolveColor(v('--fjord'), '#2f5d74').replace('rgb(', 'rgba(').replace(')', ', 0.3)'),
      black: ink,
      red: c('--falu', '#a8392b'),
      green: c('--moss', '#4f6340'),
      yellow: c('--clay', '#a4502f'),
      blue: c('--fjord', '#2f5d74'),
      magenta: c('--danger', '#8a2e3b'),
      cyan: c('--fjord-strong', '#22475a'),
      white: c('--ink-2', '#57524b'),
      brightBlack: c('--ink-3', '#6b665e'),
      brightRed: c('--falu', '#a8392b'),
      brightGreen: c('--success', '#3f6b3a'),
      brightYellow: c('--clay', '#a4502f'),
      brightBlue: c('--fjord-strong', '#22475a'),
      brightMagenta: c('--danger', '#8a2e3b'),
      brightCyan: c('--fjord', '#2f5d74'),
      brightWhite: ink
    };
  }

  function fontFamily() {
    const f = getComputedStyle(document.documentElement).getPropertyValue('--font-mono').trim();
    return f || 'ui-monospace, Menlo, monospace';
  }

  // ---- sizing --------------------------------------------------------------

  function layout() {
    if (!panel || !active) return;
    const vv = window.visualViewport;
    const vh = vv ? vv.height : window.innerHeight;
    const off = vv ? vv.offsetTop : 0;
    const top = panel.getBoundingClientRect().top - off;
    const kb = keybar ? keybar.offsetHeight : 0;
    const tabbar = document.querySelector<HTMLElement>('.vm-tabbar');
    // offsetParent is always null for fixed elements, so test for a box instead.
    const tb = tabbar && getComputedStyle(tabbar).position === 'fixed' && tabbar.getClientRects().length > 0
      ? tabbar.getBoundingClientRect().height : 0;
    const keyboardOpen = window.innerHeight - vh > 120;
    panelHeight = Math.max(180, Math.floor(vh - top - kb - (keyboardOpen ? 0 : tb) - 12));
    tick().then(refit);
  }

  function refit() {
    if (!term || !fit || !active || !host || host.clientWidth === 0) return;
    try { fit.fit(); } catch {}
  }

  // ---- connection ----------------------------------------------------------

  function sendInput(data: string) {
    if (ws && ws.readyState === WebSocket.OPEN) ws.send(enc.encode('0' + data));
  }
  function sendResize() {
    if (ws && ws.readyState === WebSocket.OPEN && term) {
      ws.send(enc.encode('1' + JSON.stringify({ columns: term.cols, rows: term.rows })));
    }
  }

  function disconnect() {
    if (ws) {
      ws.onopen = ws.onclose = ws.onerror = ws.onmessage = null;
      try { ws.close(); } catch {}
      ws = null;
    }
  }

  function connect() {
    if (!term) return;
    disconnect();
    state = 'connecting';
    term.reset();
    refit();
    const socket = openTerminalSocket(session);
    ws = socket;
    socket.onopen = () => {
      if (ws !== socket || !term) return;
      socket.send(enc.encode(JSON.stringify({ AuthToken: '', columns: term.cols, rows: term.rows })));
      state = 'connected';
      sendResize();
    };
    socket.onmessage = (ev) => {
      if (!term || !(ev.data instanceof ArrayBuffer)) return;
      const bytes = new Uint8Array(ev.data);
      if (bytes.length === 0) return;
      const cmd = String.fromCharCode(bytes[0]);
      if (cmd === '0') term.write(bytes.subarray(1));
      // "1" (window title) and "2" (preferences JSON) are ignored.
    };
    const closed = () => {
      if (ws !== socket) return;
      state = 'closed';
    };
    socket.onclose = closed;
    socket.onerror = closed;
  }

  // Reconnect when the session changes (after first connect).
  let connectedSession = '';
  $: if (ready && session && session !== connectedSession) {
    connectedSession = session;
    connect();
  }

  // ---- keys ----------------------------------------------------------------

  function key(seq: string) {
    sendInput(seq);
    term?.focus();
  }
  function toggleCtrl() {
    ctrl = !ctrl;
    term?.focus();
  }
  function onData(d: string) {
    if (ctrl && d.length === 1) {
      const code = d.toUpperCase().charCodeAt(0);
      if (code >= 64 && code <= 95) d = String.fromCharCode(code - 64);
      ctrl = false;
    }
    sendInput(d);
  }

  const KEYS: { label: string; seq: string; aria?: string }[] = [
    { label: 'Esc', seq: '\x1b' },
    { label: 'Tab', seq: '\t' },
    { label: 'Shift+Tab', seq: '\x1b[Z', aria: 'Shift Tab' },
    { label: 'Ctrl+C', seq: '\x03', aria: 'Control C' },
    { label: '↑', seq: '\x1b[A', aria: 'Up arrow' },
    { label: '↓', seq: '\x1b[B', aria: 'Down arrow' },
    { label: '←', seq: '\x1b[D', aria: 'Left arrow' },
    { label: '→', seq: '\x1b[C', aria: 'Right arrow' }
  ];

  // ---- lifecycle -----------------------------------------------------------

  onMount(() => {
    let cancelled = false;
    (async () => {
      const [{ Terminal }, { FitAddon }] = await Promise.all([import('@xterm/xterm'), import('@xterm/addon-fit')]);
      try { await document.fonts.load(`13px ${fontFamily().split(',')[0]}`); } catch {}
      if (cancelled) return;
      term = new Terminal({
        fontFamily: fontFamily(),
        fontSize: 13,
        lineHeight: 1.15,
        cursorBlink: true,
        scrollback: 5000,
        allowProposedApi: true,
        theme: readTheme()
      });
      fit = new FitAddon();
      term.loadAddon(fit);
      term.open(host);
      term.onData(onData);
      term.onResize(sendResize);
      layout();
      ready = true;
    })();

    loadNames();
    const onViewport = () => layout();
    window.addEventListener('resize', onViewport);
    window.visualViewport?.addEventListener('resize', onViewport);
    window.visualViewport?.addEventListener('scroll', onViewport);
    window.addEventListener('scroll', onViewport, { passive: true });
    ro = new ResizeObserver(() => refit());
    ro.observe(host);
    themeObserver = new MutationObserver(() => {
      if (term) {
        term.options.theme = readTheme();
        term.options.fontFamily = fontFamily();
      }
    });
    themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme', 'data-design'] });

    return () => {
      cancelled = true;
      window.removeEventListener('resize', onViewport);
      window.visualViewport?.removeEventListener('resize', onViewport);
      window.visualViewport?.removeEventListener('scroll', onViewport);
      window.removeEventListener('scroll', onViewport);
    };
  });

  onDestroy(() => {
    ro?.disconnect();
    themeObserver?.disconnect();
    disconnect();
    term?.dispose();
  });

  // Refit and refresh the session list whenever the tab becomes visible.
  $: if (active) {
    loadNames();
    tick().then(() => {
      layout();
      if (window.matchMedia('(pointer: fine)').matches) term?.focus();
    });
  }

  const stateLabel: Record<ConnState, string> = {
    connecting: 'Connecting…',
    connected: 'Connected',
    closed: 'Disconnected'
  };
</script>

<section class="term" aria-label="Terminal">
  <div class="top">
    <label class="sel">
      <span class="sr">tmux session</span>
      <select class="field mono" bind:value={session}>
        {#each options as n}<option value={n}>{n}</option>{/each}
      </select>
    </label>
    <span class="state" class:ok={state === 'connected'} role="status">
      <span class="led" class:led-hollow={state !== 'connected'} aria-hidden="true"></span>
      {stateLabel[state]}
    </span>
    <button type="button" class="btn btn-sm" on:click={connect} disabled={!ready}><RefreshCw size={14} aria-hidden="true" /> Reconnect</button>
  </div>

  <div class="panel" bind:this={panel} style="height: {panelHeight}px">
    <div class="host" bind:this={host}></div>
  </div>

  <div class="keybar" bind:this={keybar} role="toolbar" aria-label="Terminal keys">
    <button type="button" class="k ctrl" aria-pressed={ctrl} on:mousedown|preventDefault on:click={toggleCtrl}>Ctrl</button>
    {#each KEYS as k}
      <button type="button" class="k" aria-label={k.aria ?? k.label} on:mousedown|preventDefault on:click={() => key(k.seq)}>{k.label}</button>
    {/each}
    <button type="button" class="k" aria-label="Enter" on:mousedown|preventDefault on:click={() => key('\r')}><CornerDownLeft size={16} aria-hidden="true" /></button>
  </div>
</section>

<style>
  .term { display: flex; flex-direction: column; gap: var(--s-2); min-width: 0; }
  .top { display: flex; align-items: center; gap: var(--s-3); flex-wrap: wrap; }
  .sel { min-width: 0; flex: 0 1 180px; }
  .mono { font-family: var(--font-mono); font-size: 14px; }
  .state { display: inline-flex; align-items: center; gap: var(--s-2); margin-right: auto; color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); }
  .state.ok { color: var(--ink-2); }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }

  .panel {
    min-height: 0;
    padding: var(--s-2);
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
  }
  .host { width: 100%; height: 100%; }
  .host :global(.xterm) { height: 100%; }
  .host :global(.xterm-viewport) { background: transparent !important; }

  .keybar {
    position: sticky;
    bottom: var(--vm-tabbar-h, 0px);
    z-index: 5;
    display: flex;
    gap: 6px;
    padding: 6px 0;
    overflow-x: auto;
    background: var(--bg);
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
  }
  .keybar::-webkit-scrollbar { display: none; }
  .k {
    flex: 0 0 auto;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 44px;
    min-height: 44px;
    padding: 0 var(--s-3);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: 13px;
    cursor: pointer;
    touch-action: manipulation;
  }
  .k:hover { border-color: var(--ink); }
  .k:active { background: var(--press-wash); }
  .k:focus-visible { outline: 2px solid var(--fjord); outline-offset: 1px; }
  .k[aria-pressed='true'] { border-color: var(--fjord); background: var(--fjord-wash); color: var(--fjord); }
</style>
