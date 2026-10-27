<script lang="ts">
  import { onMount } from 'svelte';
  import { Folder, File as FileIcon, Link2, ArrowUp } from 'lucide-svelte';
  import hljs from 'highlight.js/lib/common';
  import { vmApi, VmSignedOut, type FsEntry, type FsFile } from '$lib/vmApi';
  import { humanBytes, fmtMtime } from '$lib/vmFormat';

  let home = '';
  let path = '';
  let parent: string | null = null;
  let entries: FsEntry[] = [];
  let file: FsFile | null = null;
  let loading = true;
  let error = '';

  const join = (a: string, b: string) => (a.endsWith('/') ? a + b : a + '/' + b);

  async function go(p?: string) {
    loading = true;
    try {
      const r = await vmApi.fs(p);
      error = '';
      if (!home && !p) home = r.path;
      path = r.path;
      parent = r.parent;
      if (r.type === 'dir') {
        file = null;
        entries = [...r.entries].sort((a, b) => {
          const da = a.type === 'dir' ? 0 : 1;
          const db = b.type === 'dir' ? 0 : 1;
          return da - db || a.name.localeCompare(b.name);
        });
      } else {
        file = r;
      }
    } catch (e) {
      if (!(e instanceof VmSignedOut)) error = (e as Error).message;
    } finally {
      loading = false;
    }
  }

  // Breadcrumb: "~" for home, then each path segment under it.
  $: crumbs = (() => {
    const out: { label: string; path: string }[] = [];
    if (!home) return out;
    out.push({ label: '~', path: home });
    const dir = file ? (file.parent ?? home) : path;
    if (dir.startsWith(home) && dir !== home) {
      let acc = home;
      for (const seg of dir.slice(home.length).split('/').filter(Boolean)) {
        acc = join(acc, seg);
        out.push({ label: seg, path: acc });
      }
    }
    return out;
  })();

  const escapeHtml = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

  function highlight(f: FsFile): string {
    const text = f.text ?? '';
    if (text.length > 200_000) return escapeHtml(text);
    const name = f.path.split('/').pop() ?? '';
    const ext = name.includes('.') ? name.split('.').pop()!.toLowerCase() : name.toLowerCase();
    if (!hljs.getLanguage(ext)) return escapeHtml(text);
    try {
      return hljs.highlight(text, { language: ext, ignoreIllegals: true }).value;
    } catch {
      return escapeHtml(text);
    }
  }
  $: html = file && file.text !== null ? highlight(file) : '';

  onMount(() => go());
</script>

<section class="files" aria-label="Files">
  <nav class="crumbs" aria-label="Path">
    {#each crumbs as c, i}
      {#if i > 0}<span class="sep" aria-hidden="true">/</span>{/if}
      {#if i === crumbs.length - 1 && !file}
        <span class="cur" aria-current="page">{c.label}</span>
      {:else}
        <button type="button" class="crumb" on:click={() => go(c.path)}>{c.label}</button>
      {/if}
    {/each}
    {#if file}<span class="sep" aria-hidden="true">/</span><span class="cur" aria-current="page">{file.path.split('/').pop()}</span>{/if}
  </nav>

  {#if error}<p class="err" role="alert">{error}</p>{/if}

  {#if loading && entries.length === 0 && !file}
    <p class="muted">Loading…</p>
  {:else if file}
    <div class="fhead">
      <button type="button" class="btn btn-sm" on:click={() => go(file?.parent ?? home)}><ArrowUp size={14} aria-hidden="true" /> Up</button>
      <span class="fmeta">{humanBytes(file.size)} · {fmtMtime(file.mtime)}</span>
    </div>
    {#if file.binary || file.text === null}
      <p class="notice">This is a binary file, so there is no preview.</p>
    {:else}
      {#if file.truncated}<p class="notice">Showing the first 512 KB of {humanBytes(file.size)}.</p>{/if}
      <!-- svelte-ignore a11y-no-noninteractive-tabindex -->
      <pre class="code" tabindex="0" aria-label="File contents"><code class="hljs">{@html html}</code></pre>
    {/if}
  {:else}
    {#if parent}
      <button type="button" class="btn btn-sm up" on:click={() => go(parent ?? undefined)}><ArrowUp size={14} aria-hidden="true" /> Up</button>
    {/if}
    {#if entries.length === 0}
      <p class="muted">This folder is empty.</p>
    {:else}
      <ol class="rows">
        {#each entries as e (e.name)}
          <li>
            <button type="button" class="erow" on:click={() => go(join(path, e.name))}>
              <span class="ico" aria-hidden="true">
                {#if e.type === 'dir'}<Folder size={16} strokeWidth={1.6} />{:else if e.type === 'link'}<Link2 size={16} strokeWidth={1.6} />{:else}<FileIcon size={16} strokeWidth={1.6} />{/if}
              </span>
              <span class="en" class:dir={e.type === 'dir'}>{e.name}</span>
              <span class="es">{e.type === 'dir' ? '' : humanBytes(e.size)}</span>
            </button>
          </li>
        {/each}
      </ol>
    {/if}
  {/if}
</section>

<style>
  .files { display: flex; flex-direction: column; gap: var(--s-3); min-width: 0; }
  .crumbs { display: flex; flex-wrap: wrap; align-items: center; gap: 2px; font-family: var(--font-mono); font-size: var(--text-sm); }
  .crumb { min-height: 32px; padding: 0 6px; border: 0; border-radius: var(--radius-xs); background: none; color: var(--ink-2); cursor: pointer; }
  .crumb:hover { color: var(--fjord); background: var(--hover-wash); }
  .crumb:focus-visible { outline: 2px solid var(--fjord); outline-offset: 1px; }
  .cur { padding: 0 6px; color: var(--ink); overflow-wrap: anywhere; }
  .sep { color: var(--ink-3); }
  .muted { margin: 0; color: var(--ink-3); font-size: var(--text-sm); }
  .err { margin: 0; color: var(--falu); font-size: var(--text-sm); }
  .up { align-self: flex-start; }
  .notice { margin: 0; padding: var(--s-3); border: 1px solid var(--line); border-radius: var(--radius-sm); background: var(--surface-2); color: var(--ink-2); font-size: var(--text-sm); }

  .erow { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: var(--s-3); width: 100%; min-height: 48px; padding: 0 var(--s-1); border: 0; background: none; text-align: left; cursor: pointer; }
  .erow:hover { background: var(--hover-wash); }
  .erow:focus-visible { outline: 2px solid var(--fjord); outline-offset: -2px; }
  .ico { display: inline-flex; color: var(--ink-3); }
  .en { font-family: var(--font-mono); font-size: 14px; overflow-wrap: anywhere; }
  .en.dir { color: var(--fjord); font-weight: 500; }
  .es { color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); font-variant-numeric: tabular-nums; }

  .fhead { display: flex; align-items: center; justify-content: space-between; gap: var(--s-3); }
  .fmeta { color: var(--ink-3); font-family: var(--font-mono); font-size: var(--text-xs); }
  .code {
    margin: 0;
    max-height: 70vh;
    padding: var(--s-3);
    overflow: auto;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--code-surface);
    -webkit-overflow-scrolling: touch;
  }
  .code:focus-visible { outline: 2px solid var(--fjord); outline-offset: 1px; }
  .code code { display: block; padding: 0; background: none; color: var(--ink); font-family: var(--font-mono); font-size: 12.5px; line-height: 1.55; white-space: pre; }
  .code :global(.hljs-comment), .code :global(.hljs-quote) { color: var(--ink-3); font-style: italic; }
  .code :global(.hljs-keyword), .code :global(.hljs-selector-tag), .code :global(.hljs-built_in), .code :global(.hljs-literal) { color: var(--fjord); }
  .code :global(.hljs-string), .code :global(.hljs-regexp), .code :global(.hljs-addition) { color: var(--moss); }
  .code :global(.hljs-number), .code :global(.hljs-symbol), .code :global(.hljs-bullet) { color: var(--clay); }
  .code :global(.hljs-title), .code :global(.hljs-section), .code :global(.hljs-attr), .code :global(.hljs-name) { color: var(--ink); font-weight: 500; }
  .code :global(.hljs-type), .code :global(.hljs-attribute), .code :global(.hljs-variable) { color: var(--fjord-strong); }
  .code :global(.hljs-deletion), .code :global(.hljs-meta) { color: var(--falu); }
</style>
