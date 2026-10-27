<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { RefreshCw, X } from 'lucide-svelte';
  import { vmApi, VmSignedOut, type SystemInfo, type DockerContainer } from '$lib/vmApi';
  import { humanBytes, humanUptime } from '$lib/vmFormat';

  export let active = true;

  let info: SystemInfo | null = null;
  let error = '';
  let loading = true;
  let updated = 0;

  let logName: string | null = null;
  let logText = '';
  let logLoading = false;
  let logError = '';
  let logBox: HTMLPreElement | undefined;

  async function load() {
    try {
      info = await vmApi.system();
      error = '';
      updated = Date.now();
    } catch (e) {
      if (!(e instanceof VmSignedOut)) error = (e as Error).message;
    } finally {
      loading = false;
    }
  }

  async function loadLogs(name: string) {
    logLoading = true;
    try {
      const stick = !logBox || logBox.scrollHeight - logBox.scrollTop - logBox.clientHeight < 40;
      logText = (await vmApi.dockerLogs(name, 200)).text;
      logError = '';
      if (stick) setTimeout(() => logBox && (logBox.scrollTop = logBox.scrollHeight), 0);
    } catch (e) {
      if (!(e instanceof VmSignedOut)) logError = (e as Error).message;
    } finally {
      logLoading = false;
    }
  }

  function openLogs(c: DockerContainer) {
    if (logName === c.name) { logName = null; return; }
    logName = c.name;
    logText = '';
    loadLogs(c.name);
  }

  function refresh() {
    load();
    if (logName) loadLogs(logName);
  }

  const pct = (used: number, total: number) => (total > 0 ? Math.min(100, Math.round((used / total) * 100)) : 0);
  $: memPct = info ? pct(info.mem.usedBytes, info.mem.totalBytes) : 0;
  $: diskPct = info ? pct(info.disk.usedBytes, info.disk.totalBytes) : 0;
  $: loadPct = info && info.cpus ? Math.min(100, Math.round((info.loadavg[0] / info.cpus) * 100)) : 0;
  const level = (p: number) => (p >= 90 ? 'hot' : p >= 75 ? 'warm' : '');

  let timer: ReturnType<typeof setInterval>;
  onMount(() => {
    load();
    timer = setInterval(() => {
      if (active && document.visibilityState === 'visible') refresh();
    }, 10000);
  });
  onDestroy(() => clearInterval(timer));
  // Fetch fresh numbers when the tab is shown again after a while.
  $: if (active && updated && Date.now() - updated > 10000) refresh();
</script>

<section class="sys" aria-label="System">
  <div class="bh">
    <h2>{info ? info.hostname : 'System'}</h2>
    <button type="button" class="btn btn-sm btn--ghost" on:click={refresh} aria-label="Refresh system info"><RefreshCw size={14} aria-hidden="true" /></button>
  </div>

  {#if error}<p class="err" role="alert">{error}</p>{/if}
  {#if loading}
    <p class="muted">Loading…</p>
  {:else if info}
    <dl class="facts">
      <div><dt>Uptime</dt><dd>{humanUptime(info.uptimeSec)}</dd></div>
      <div><dt>Load (1, 5, 15 min)</dt><dd>{info.loadavg.map((n) => n.toFixed(2)).join('  ')}</dd></div>
      <div><dt>CPUs</dt><dd>{info.cpus}</dd></div>
    </dl>

    <div class="meters">
      <div class="meter">
        <div class="mh"><span id="m-load">CPU load</span><span class="mv">{info.loadavg[0].toFixed(2)} / {info.cpus}</span></div>
        <div class="track" role="meter" aria-labelledby="m-load" aria-valuemin="0" aria-valuemax="100" aria-valuenow={loadPct}>
          <div class="fill {level(loadPct)}" style="width: {loadPct}%"></div>
        </div>
      </div>
      <div class="meter">
        <div class="mh"><span id="m-mem">Memory</span><span class="mv">{humanBytes(info.mem.usedBytes)} of {humanBytes(info.mem.totalBytes)} · {memPct}%</span></div>
        <div class="track" role="meter" aria-labelledby="m-mem" aria-valuemin="0" aria-valuemax="100" aria-valuenow={memPct}>
          <div class="fill {level(memPct)}" style="width: {memPct}%"></div>
        </div>
      </div>
      <div class="meter">
        <div class="mh"><span id="m-disk">Disk {info.disk.path}</span><span class="mv">{humanBytes(info.disk.usedBytes)} of {humanBytes(info.disk.totalBytes)} · {diskPct}%</span></div>
        <div class="track" role="meter" aria-labelledby="m-disk" aria-valuemin="0" aria-valuemax="100" aria-valuenow={diskPct}>
          <div class="fill {level(diskPct)}" style="width: {diskPct}%"></div>
        </div>
      </div>
    </div>

    <h2 class="h2">Docker containers</h2>
    {#if info.docker.length === 0}
      <p class="muted">No containers running.</p>
    {:else}
      <ol class="rows">
        {#each info.docker as c (c.name)}
          <li>
            <button type="button" class="crow" aria-expanded={logName === c.name} on:click={() => openLogs(c)}>
              <span class="led" class:led-hollow={!/^up/i.test(c.status)} aria-hidden="true"></span>
              <span class="cn">{c.name}</span>
              <span class="cs">{c.status}</span>
              <span class="ci">{c.image}{c.ports ? ` · ${c.ports}` : ''}</span>
            </button>
            {#if logName === c.name}
              <div class="logs">
                <div class="lbar">
                  <span>Last 200 lines{logLoading ? ' · refreshing…' : ''}</span>
                  <button type="button" class="btn btn-sm btn--ghost" on:click={() => (logName = null)} aria-label="Close logs"><X size={14} aria-hidden="true" /></button>
                </div>
                <!-- svelte-ignore a11y-no-noninteractive-tabindex -->
                <pre class="log" bind:this={logBox} tabindex="0" aria-label="Logs for {c.name}">{logText || ' '}</pre>
                {#if logError}<p class="err" role="alert">{logError}</p>{/if}
              </div>
            {/if}
          </li>
        {/each}
      </ol>
    {/if}
  {/if}
</section>

<style>
  .sys { display: flex; flex-direction: column; gap: var(--s-4); min-width: 0; }
  .bh { display: flex; align-items: center; justify-content: space-between; }
  h2 { margin: 0; font-family: var(--font-display); font-size: var(--text-lg); font-weight: 500; letter-spacing: -0.02em; }
  .h2 { font-size: var(--text-md); margin-top: var(--s-3); }
  .muted { margin: 0; color: var(--ink-3); font-size: var(--text-sm); }
  .err { margin: 0; color: var(--falu); font-size: var(--text-sm); }

  .facts { display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: var(--s-3); margin: 0; }
  .facts > div { padding: var(--s-3); border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--surface); }
  dt { color: var(--ink-3); font-size: var(--text-xs); }
  dd { margin: 4px 0 0; color: var(--ink); font-family: var(--font-mono); font-size: 15px; font-variant-numeric: tabular-nums; }

  .meters { display: grid; gap: var(--s-4); }
  .mh { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 2px var(--s-3); margin-bottom: 6px; color: var(--ink-2); font-size: var(--text-sm); }
  .mv { color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); font-variant-numeric: tabular-nums; }
  .track { height: 8px; overflow: hidden; border-radius: var(--radius-pill); background: var(--surface-2); }
  .fill { height: 100%; border-radius: var(--radius-pill); background: var(--fjord); transition: width var(--dur) var(--ease); }
  .fill.warm { background: var(--clay); }
  .fill.hot { background: var(--falu); }

  .crow {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    column-gap: var(--s-2);
    align-items: center;
    width: 100%;
    min-height: 56px;
    padding: var(--s-2) 0;
    border: 0;
    background: none;
    text-align: left;
    cursor: pointer;
  }
  .crow:hover .cn { color: var(--fjord); }
  .crow:focus-visible { outline: 2px solid var(--fjord); outline-offset: 2px; }
  .cn { font-family: var(--font-mono); font-size: 14px; font-weight: 500; overflow-wrap: anywhere; }
  .cs { color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); }
  .ci { grid-column: 2 / -1; color: var(--ink-3); font-family: var(--font-mono); font-size: 11px; overflow-wrap: anywhere; }

  .logs { padding-bottom: var(--s-3); }
  .lbar { display: flex; align-items: center; justify-content: space-between; color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); }
  .log {
    margin: 0;
    height: 300px;
    padding: var(--s-3);
    overflow: auto;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--code-surface);
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: 11.5px;
    line-height: 1.45;
    white-space: pre;
  }
  .log:focus-visible { outline: 2px solid var(--fjord); outline-offset: 1px; }
  @media (prefers-reduced-motion: reduce) { .fill { transition: none; } }
</style>
