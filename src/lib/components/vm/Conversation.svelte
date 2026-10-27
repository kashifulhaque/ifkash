<script lang="ts">
  import { Brain, Wrench, ChevronRight } from 'lucide-svelte';
  import Markdown from './Markdown.svelte';
  import { fmtCost, fmtDuration } from '$lib/vmFormat';
  import type { ConvItem } from '$lib/vmConversation';

  export let items: ConvItem[] = [];
  export let running = false;

  const MAX = 6000;
  function inputText(input: unknown): string {
    if (input === null || input === undefined) return '';
    const s = typeof input === 'string' ? input : JSON.stringify(input, null, 2);
    return s.length > MAX ? s.slice(0, MAX) + '\n…' : s;
  }
  function clipText(s: string): string {
    return s.length > MAX ? s.slice(0, MAX) + '\n…' : s;
  }
</script>

<div class="conv" aria-live="off">
  {#each items as item (item.id)}
    {#if item.kind === 'user'}
      <div class="row user"><div class="bubble">{item.text}</div></div>
    {:else if item.kind === 'text'}
      <div class="row"><Markdown text={item.text} /></div>
    {:else if item.kind === 'thinking'}
      <details class="fold thinking">
        <summary><ChevronRight size={14} class="chev" aria-hidden="true" /><Brain size={14} aria-hidden="true" /> Thinking</summary>
        <pre class="body">{clipText(item.text)}</pre>
      </details>
    {:else if item.kind === 'tool'}
      <details class="fold tool" class:err={item.result?.isError}>
        <summary>
          <ChevronRight size={14} class="chev" aria-hidden="true" />
          <Wrench size={14} aria-hidden="true" />
          <span class="tname">{item.tool}</span>
          <span class="tsum">{item.summary}</span>
          {#if item.result?.isError}<span class="tbadge">error</span>{:else if !item.result}<span class="tbadge pending">running</span>{/if}
        </summary>
        {#if inputText(item.input)}<pre class="body">{inputText(item.input)}</pre>{/if}
        {#if item.result}
          <div class="rlabel" class:err={item.result.isError}>{item.result.isError ? 'Error' : 'Result'}</div>
          <pre class="body res" class:err={item.result.isError}>{clipText(item.result.text) || '(empty)'}</pre>
        {/if}
      </details>
    {:else if item.kind === 'result'}
      <div class="footer" class:err={item.isError}>
        {#if item.isError}<strong>Run failed.</strong>{/if}
        {#if item.text}<span class="ftext">{item.text}</span>{/if}
        <span class="fmeta">
          {#if item.cost !== null}<span>{fmtCost(item.cost)}</span>{/if}
          {#if item.turns !== null}<span>{item.turns} {item.turns === 1 ? 'turn' : 'turns'}</span>{/if}
          {#if item.durationMs !== null}<span>{fmtDuration(item.durationMs)}</span>{/if}
        </span>
      </div>
    {:else if item.kind === 'stderr'}
      <details class="fold">
        <summary><ChevronRight size={14} class="chev" aria-hidden="true" /> Agent log</summary>
        <pre class="body">{clipText(item.text)}</pre>
      </details>
    {:else if item.kind === 'note'}
      <p class="note" class:err={item.isError}>{item.text}</p>
    {/if}
  {/each}
  {#if running}
    <div class="working" role="status"><span class="led" aria-hidden="true"></span> Working…</div>
  {/if}
</div>

<style>
  .conv {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
    min-width: 0;
  }
  .row { min-width: 0; }
  .row.user { display: flex; justify-content: flex-end; }
  .bubble {
    max-width: min(560px, 88%);
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface-2);
    color: var(--ink);
    font-size: 15px;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .fold {
    min-width: 0;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
  }
  .fold summary {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    min-height: 36px;
    padding: 0 var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    cursor: pointer;
    list-style: none;
    min-width: 0;
  }
  .fold summary::-webkit-details-marker { display: none; }
  .fold summary:focus-visible { outline: 2px solid var(--fjord); outline-offset: -2px; }
  .fold summary :global(.chev) { flex-shrink: 0; transition: transform var(--dur-fast) var(--ease); }
  .fold[open] summary :global(.chev) { transform: rotate(90deg); }
  .tname { color: var(--ink); font-weight: 500; flex-shrink: 0; }
  .tsum {
    min-width: 0;
    flex: 1;
    overflow: hidden;
    color: var(--ink-2);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tbadge {
    flex-shrink: 0;
    padding: 0 6px;
    border: 1px solid var(--falu);
    border-radius: var(--radius-pill);
    color: var(--falu);
    font-size: 10.5px;
  }
  .tbadge.pending { border-color: var(--line-strong); color: var(--ink-3); }
  .fold.err { border-color: var(--falu); }
  .body {
    margin: 0;
    padding: var(--s-3);
    max-height: 320px;
    overflow: auto;
    border-top: 1px solid var(--line);
    background: var(--code-surface);
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .body.err { color: var(--falu); }
  .rlabel {
    padding: 4px var(--s-3);
    border-top: 1px solid var(--line);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 10.5px;
  }
  .rlabel.err { color: var(--falu); }
  .thinking summary { font-style: italic; }

  .footer {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--s-2) var(--s-4);
    padding-top: var(--s-3);
    border-top: 1px solid var(--line);
    color: var(--ink-3);
    font-size: var(--text-sm);
  }
  .footer.err { color: var(--falu); }
  .ftext { overflow-wrap: anywhere; }
  .fmeta {
    display: inline-flex;
    gap: var(--s-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
  .note { margin: 0; color: var(--ink-3); font-size: var(--text-sm); overflow-wrap: anywhere; }
  .note.err { color: var(--falu); }
  .working {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .working .led { animation: pulse 1.4s var(--ease) infinite; }
  @keyframes pulse { 50% { opacity: 0.25; } }
  @media (prefers-reduced-motion: reduce) {
    .working .led { animation: none; }
    .fold summary :global(.chev) { transition: none; }
  }
</style>
