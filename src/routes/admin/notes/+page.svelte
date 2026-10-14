<script lang="ts">
  import { onMount } from "svelte";
  import { Marked } from "marked";
  import katex from "katex";
  import "katex/dist/katex.min.css";
  import DOMPurify from "dompurify";
  import TreeDir from "./TreeDir.svelte";
  import LoadingState from "$lib/components/LoadingState.svelte";
  import {
    FileText,
    File,
    RefreshCw,
    Save,
    Eye,
    Code,
    Plus,
    Upload,
    Download,
    Trash2,
    AlertTriangle,
  } from "lucide-svelte";
  import type { PageData } from "./$types";

  export let data: PageData;

  type TreeFile = { path: string; size: number; sha: string };
  type DirNode = {
    name: string;
    path: string;
    dirs: Map<string, DirNode>;
    files: TreeFile[];
  };

  const TEXT_EXTENSIONS = ["md", "txt", "canvas", "json", "csv", "yml", "yaml"];
  const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "gif", "webp", "svg", "avif", "bmp"];

  let branch = "";
  let files: TreeFile[] = [];
  let root: DirNode | null = null;
  let expanded = new Set<string>();

  let currentPath = "";
  let currentSha: string | undefined;
  let content = "";
  let savedContent = "";
  let view: "source" | "preview" = "source";
  let binary = false;
  let uploadInput: HTMLInputElement;
  let uploading = false;

  let loadingTree = true;
  let loadingFile = false;
  let saving = false;
  let statusMsg = "";
  let errorMsg = "";

  $: dirty = !binary && content !== savedContent;
  $: renderedHtml =
    view === "preview" && currentPath.endsWith(".md")
      ? DOMPurify.sanitize(md.parse(preprocessObsidian(content), { async: false }) as string)
      : "";
  $: currentExt = currentPath.split(".").pop()?.toLowerCase() ?? "";

  // fetch that detects an expired Cloudflare Access session (the edge then
  // answers with the login HTML instead of JSON) and says so plainly.
  async function apiFetch(input: string, init?: RequestInit): Promise<Response> {
    const res = await fetch(input, init);
    const type = res.headers.get("content-type") ?? "";
    if ((res.ok || res.status === 409) && !type.includes("json")) {
      throw new Error("Session expired — reload the page to sign in again.");
    }
    return res;
  }

  function rawUrl(path: string): string {
    return `/admin/notes/api/raw?path=${encodeURIComponent(path)}`;
  }

  // Resolve an embed/link target the way Obsidian does: exact vault path,
  // then relative to the current note's folder, then unique-basename lookup
  // (covers attachments living in a separate folder).
  function resolveVaultPath(target: string): string | null {
    const clean = decodeURIComponent(target).replace(/^\.\//, "").replace(/^\//, "");
    if (files.some((f) => f.path === clean)) return clean;
    const dir = currentPath.includes("/")
      ? currentPath.slice(0, currentPath.lastIndexOf("/"))
      : "";
    if (dir && files.some((f) => f.path === `${dir}/${clean}`)) return `${dir}/${clean}`;
    const base = clean.split("/").pop()!.toLowerCase();
    const match = files.find((f) => f.path.split("/").pop()!.toLowerCase() === base);
    return match?.path ?? null;
  }

  // Translate Obsidian-specific syntax into standard markdown before parsing:
  // YAML frontmatter, ![[img.png|300]] embeds and [[Note|alias]] wikilinks.
  function preprocessObsidian(src: string): string {
    return src
      .replace(/^---\r?\n([\s\S]*?)\r?\n---(\r?\n|$)/, (_, yaml) => `\`\`\`yaml\n${yaml}\n\`\`\`\n\n`)
      .replace(/!\[\[([^\]|]+)(?:\|([^\]]*))?\]\]/g, (whole, target, size) => {
        const resolved = resolveVaultPath(target.trim());
        if (!resolved) return whole;
        const ext = resolved.split(".").pop()?.toLowerCase() ?? "";
        if (IMAGE_EXTENSIONS.includes(ext)) {
          // Pass the |300 size hint through the alt text; the image renderer
          // turns a purely numeric alt into a width attribute.
          return `![${size?.trim() ?? ""}](${rawUrl(resolved)})`;
        }
        return `[${target.trim()}](#wiki:${encodeURIComponent(resolved)})`;
      })
      .replace(/(^|[^!])\[\[([^\]|]+)(?:\|([^\]]*))?\]\]/g, (whole, pre, target, alias) => {
        const t = target.trim();
        const resolved = resolveVaultPath(t) ?? resolveVaultPath(`${t}.md`);
        const label = alias?.trim() || t;
        if (!resolved) return `${pre}${label}`;
        return `${pre}[${label}](#wiki:${encodeURIComponent(resolved)})`;
      });
  }

  // Scoped Marked instance (the global singleton would accumulate extensions
  // on every remount of this page).
  const md = new Marked();
  md.use({
    renderer: {
      image({ href, title, text }: { href: string; title: string | null; text: string }) {
        let src = href ?? "";
        // Standard markdown images with vault-relative paths also need to be
        // served through the raw endpoint.
        if (!/^(https?:|data:|\/admin\/notes\/api\/raw)/.test(src)) {
          const resolved = resolveVaultPath(src);
          if (resolved) src = rawUrl(resolved);
        }
        const sizeHint = /^\d+$/.test(text) ? ` width="${text}"` : "";
        const alt = sizeHint ? "" : text;
        return `<img src="${src}" alt="${alt}"${title ? ` title="${title}"` : ""}${sizeHint} loading="lazy" />`;
      },
    },
    extensions: [
      {
        name: "blockMath",
        level: "block",
        start: (src: string) => src.indexOf("$$"),
        tokenizer(src: string) {
          const match = /^\$\$([\s\S]+?)\$\$/.exec(src);
          if (match) return { type: "blockMath", raw: match[0], text: match[1].trim() };
        },
        renderer(token: any) {
          return katex.renderToString(token.text, { displayMode: true, throwOnError: false });
        },
      },
      {
        name: "inlineMath",
        level: "inline",
        start: (src: string) => src.indexOf("$"),
        tokenizer(src: string) {
          // $...$ with no spaces hugging the delimiters, so "$5 and $10" stays text
          const match = /^\$([^\s$](?:[^$\n]*[^\s$])?)\$/.exec(src);
          if (match) return { type: "inlineMath", raw: match[0], text: match[1] };
        },
        renderer(token: any) {
          return katex.renderToString(token.text, { displayMode: false, throwOnError: false });
        },
      },
    ],
  });

  function handlePreviewClick(e: MouseEvent) {
    const a = (e.target as HTMLElement).closest("a");
    if (!a) return;
    const href = a.getAttribute("href") ?? "";
    if (href.startsWith("#wiki:")) {
      e.preventDefault();
      openFile(decodeURIComponent(href.slice("#wiki:".length)));
    } else if (/^https?:/.test(href)) {
      a.target = "_blank";
      a.rel = "noopener";
    }
  }

  function buildTree(list: TreeFile[]): DirNode {
    const rootNode: DirNode = { name: "", path: "", dirs: new Map(), files: [] };
    for (const f of list) {
      const parts = f.path.split("/");
      let node = rootNode;
      for (let i = 0; i < parts.length - 1; i++) {
        const dirPath = parts.slice(0, i + 1).join("/");
        if (!node.dirs.has(parts[i])) {
          node.dirs.set(parts[i], { name: parts[i], path: dirPath, dirs: new Map(), files: [] });
        }
        node = node.dirs.get(parts[i])!;
      }
      node.files.push(f);
    }
    return rootNode;
  }

  function sortedDirs(node: DirNode): DirNode[] {
    return [...node.dirs.values()].sort((a, b) => a.name.localeCompare(b.name));
  }

  function sortedFiles(node: DirNode): TreeFile[] {
    return [...node.files].sort((a, b) => a.path.localeCompare(b.path));
  }

  function isTextFile(path: string): boolean {
    const ext = path.split(".").pop()?.toLowerCase() ?? "";
    return TEXT_EXTENSIONS.includes(ext) || !path.includes(".");
  }

  function fileName(path: string): string {
    return path.split("/").pop() ?? path;
  }

  function toggleDir(path: string) {
    if (expanded.has(path)) expanded.delete(path);
    else expanded.add(path);
    expanded = new Set(expanded); // new identity for reactivity
  }

  async function loadTree() {
    loadingTree = true;
    errorMsg = "";
    try {
      const res = await apiFetch("/admin/notes/api/tree");
      if (!res.ok) throw new Error(`Failed to load file tree (${res.status})`);
      const tree = await res.json();
      branch = tree.branch;
      files = tree.files;
      root = buildTree(files);
      if (tree.truncated) statusMsg = "Repo is large — tree listing was truncated by GitHub.";
    } catch (e: any) {
      errorMsg = e?.message ?? String(e);
    } finally {
      loadingTree = false;
    }
  }

  async function openFile(path: string) {
    if (dirty && !confirm("Discard unsaved changes?")) return;
    errorMsg = "";
    statusMsg = "";
    if (!isTextFile(path)) {
      // Binary attachment: previewed/downloaded straight from the raw
      // endpoint, no content fetch needed.
      currentPath = path;
      currentSha = files.find((f) => f.path === path)?.sha;
      content = "";
      savedContent = "";
      binary = true;
      return;
    }
    binary = false;
    loadingFile = true;
    try {
      const res = await apiFetch(`/admin/notes/api/file?path=${encodeURIComponent(path)}`);
      if (!res.ok) throw new Error(`Failed to open ${path} (${res.status})`);
      const file = await res.json();
      currentPath = file.path;
      currentSha = file.sha;
      content = file.content;
      savedContent = file.content;
      view = "source";
    } catch (e: any) {
      errorMsg = e?.message ?? String(e);
    } finally {
      loadingFile = false;
    }
  }

  async function saveFile() {
    if (!currentPath || saving) return;
    saving = true;
    errorMsg = "";
    statusMsg = "";
    try {
      const res = await apiFetch("/admin/notes/api/file", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path: currentPath, content, sha: currentSha }),
      });
      if (res.status === 409) {
        errorMsg =
          "This file changed on GitHub since you opened it. Re-open it to load the latest version (your text stays in the editor until you do).";
        return;
      }
      if (!res.ok) throw new Error(`Save failed (${res.status})`);
      const saved = await res.json();
      currentSha = saved.sha;
      savedContent = content;
      statusMsg = `Committed ${saved.commit?.sha?.slice(0, 7) ?? ""} to ${branch}`;
      files = [
        ...files.filter((f) => f.path !== currentPath),
        { path: currentPath, size: content.length, sha: saved.sha },
      ];
      root = buildTree(files);
    } catch (e: any) {
      errorMsg = e?.message ?? String(e);
    } finally {
      saving = false;
    }
  }

  function newFile() {
    if (dirty && !confirm("Discard unsaved changes?")) return;
    let path = prompt("New note path (e.g. inbox/idea.md):");
    if (!path) return;
    path = path.replace(/^\/+/, "");
    if (!path.includes(".")) path += ".md";
    if (files.some((f) => f.path === path)) {
      openFile(path);
      return;
    }
    currentPath = path;
    currentSha = undefined; // no sha → GitHub creates the file on save
    content = `# ${fileName(path).replace(/\.md$/, "")}\n\n`;
    savedContent = ""; // differs from content, so Save is enabled
    binary = false;
    view = "source";
    statusMsg = "New note — saving will create it on GitHub.";
  }

  async function deleteFile() {
    if (!currentPath || saving) return;
    const sha = currentSha ?? files.find((f) => f.path === currentPath)?.sha;
    if (!sha) {
      // Unsaved new note — nothing on GitHub yet, just discard it.
      currentPath = "";
      content = savedContent = "";
      return;
    }
    if (!confirm(`Delete ${currentPath}? This commits the deletion to GitHub.`)) return;
    saving = true;
    errorMsg = "";
    statusMsg = "";
    try {
      const res = await apiFetch("/admin/notes/api/file", {
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path: currentPath, sha }),
      });
      if (res.status === 409) {
        errorMsg = "This file changed on GitHub since you opened it. Refresh and try again.";
        return;
      }
      if (!res.ok) throw new Error(`Delete failed (${res.status})`);
      const deleted = await res.json();
      files = files.filter((f) => f.path !== currentPath);
      root = buildTree(files);
      statusMsg = `Deleted ${currentPath} (${deleted.commit?.sha?.slice(0, 7) ?? ""})`;
      currentPath = "";
      currentSha = undefined;
      content = savedContent = "";
      binary = false;
    } catch (e: any) {
      errorMsg = e?.message ?? String(e);
    } finally {
      saving = false;
    }
  }

  function bytesToBase64(bytes: Uint8Array): string {
    let bin = "";
    const chunk = 0x8000;
    for (let i = 0; i < bytes.length; i += chunk) {
      bin += String.fromCharCode(...bytes.subarray(i, i + chunk));
    }
    return btoa(bin);
  }

  async function handleUploadChange() {
    const file = uploadInput.files?.[0];
    uploadInput.value = "";
    if (!file) return;
    const dir = currentPath.includes("/")
      ? currentPath.slice(0, currentPath.lastIndexOf("/"))
      : "";
    let path = prompt("Upload as:", dir ? `${dir}/${file.name}` : file.name);
    if (!path) return;
    path = path.replace(/^\/+/, "");
    const existing = files.find((f) => f.path === path);
    if (existing && !confirm(`${path} already exists. Overwrite it?`)) return;

    uploading = true;
    errorMsg = "";
    statusMsg = "";
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      const res = await apiFetch("/admin/notes/api/file", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          path,
          content: bytesToBase64(bytes),
          encoding: "base64",
          sha: existing?.sha,
        }),
      });
      if (res.status === 409) {
        errorMsg = "This file changed on GitHub. Refresh the tree and retry the upload.";
        return;
      }
      if (!res.ok) throw new Error(`Upload failed (${res.status})`);
      const saved = await res.json();
      files = [
        ...files.filter((f) => f.path !== path),
        { path, size: bytes.length, sha: saved.sha },
      ];
      root = buildTree(files);
      statusMsg = `Uploaded ${path} (${saved.commit?.sha?.slice(0, 7) ?? ""})`;
    } catch (e: any) {
      errorMsg = e?.message ?? String(e);
    } finally {
      uploading = false;
    }
  }

  function handleBeforeUnload(e: BeforeUnloadEvent) {
    if (dirty) e.preventDefault();
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "s") {
      e.preventDefault();
      if (dirty) saveFile();
    }
  }

  onMount(loadTree);
</script>

<svelte:head>
  <title>Notes — Kashif</title>
  <meta name="robots" content="noindex" />
</svelte:head>

<svelte:window on:keydown={handleKeydown} on:beforeunload={handleBeforeUnload} />

<div class="notes-shell">
  <header class="appbar">
    <div class="appbar-left">
      <a href="/" class="wordmark" aria-label="ifkash.dev home"><span class="dot" aria-hidden="true"></span> ifkash</a>
      <nav class="crumb" aria-label="Breadcrumb">
        <span class="sep" aria-hidden="true">/</span>
        <a href="/admin" class="back-link">Admin</a>
        <span class="sep" aria-hidden="true">/</span>
        <h1>Notes{#if branch}<span class="branch"> · {branch}</span>{/if}</h1>
      </nav>
      <span class="signed-in">{data.email}</span>
    </div>
    <div class="appbar-actions">
      <button class="btn btn-sm" on:click={newFile} title="New note">
        <Plus size={14} /> <span class="lbl">New</span>
      </button>
      <button
        class="btn btn-sm"
        on:click={() => uploadInput.click()}
        disabled={uploading}
        title="Upload a file (image, PDF, …) to the vault"
      >
        {#if uploading}<LoadingState label="Uploading" />{:else}<Upload size={14} /> <span class="lbl">Upload</span>{/if}
      </button>
      <button class="btn btn-sm" on:click={loadTree} disabled={loadingTree} title="Re-fetch the file tree from GitHub">
        <RefreshCw size={14} /> <span class="lbl">Refresh</span>
      </button>
      <button
        class="btn btn-sm"
        on:click={() => (view = view === "source" ? "preview" : "source")}
        disabled={!currentPath || !currentPath.endsWith(".md")}
        title="Toggle source / rendered view"
      >
        {#if view === "source"}<Eye size={14} /> <span class="lbl">Preview</span>{:else}<Code size={14} /> <span class="lbl">Source</span>{/if}
      </button>
      <button
        class="btn btn-sm danger"
        on:click={deleteFile}
        disabled={!currentPath || saving}
        title="Delete this file from the vault"
      >
        <Trash2 size={14} /> <span class="lbl">Delete</span>
      </button>
      <span class="divider" aria-hidden="true"></span>
      <button
        class="btn btn-sm btn-primary"
        on:click={saveFile}
        disabled={!currentPath || binary || !dirty || saving}
      >
        {#if saving}<LoadingState label="Saving" />{:else}<Save size={14} /> Save &amp; push{/if}
      </button>
    </div>
    <input
      type="file"
      bind:this={uploadInput}
      on:change={handleUploadChange}
      style="display: none"
    />
  </header>

  {#if errorMsg}
    <div class="banner error" role="alert"><AlertTriangle size={14} /> {errorMsg}</div>
  {:else if statusMsg}
    <div class="banner ok" role="status"><span class="led" aria-hidden="true"></span> {statusMsg}</div>
  {/if}

  <div class="notes-body">
    <aside class="file-tree">
      {#if loadingTree}
        <div class="tree-hint"><LoadingState label="Loading vault" /></div>
      {:else if root}
        <div class="tree-head" aria-hidden="true">
          <span>Vault</span>
          <span class="tree-count">{files.length}</span>
        </div>
        <ul class="tree-root">
          {#each sortedDirs(root) as dir (dir.path)}
            <TreeDir node={dir} {expanded} {currentPath} onToggle={toggleDir} onOpen={openFile} />
          {/each}
          {#each sortedFiles(root) as f (f.path)}
            <li>
              <button
                class="tree-file"
                class:active={f.path === currentPath}
                on:click={() => openFile(f.path)}
              >
                {#if f.path.endsWith(".md")}<FileText size={13} />{:else}<File size={13} />{/if}
                <span class="tree-name">{fileName(f.path)}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </aside>

    <main class="editor-pane">
      {#if !currentPath}
        <div class="empty-state"><span class="led led-hollow" aria-hidden="true"></span> Select a note from the vault, or create a new one.</div>
      {:else if loadingFile}
        <div class="empty-state"><LoadingState label="Loading file" /></div>
      {:else if binary}
        <div class="doc-title"><span class="doc-path">{currentPath}</span></div>
        <div class="binary-pane">
          {#if IMAGE_EXTENSIONS.includes(currentExt)}
            <img class="binary-preview" src={rawUrl(currentPath)} alt={currentPath} />
          {:else if currentExt === "pdf"}
            <iframe class="binary-frame" src={rawUrl(currentPath)} title={currentPath}></iframe>
          {:else if ["mp3", "wav", "ogg", "m4a"].includes(currentExt)}
            <audio controls src={rawUrl(currentPath)}></audio>
          {:else if ["mp4", "webm", "mov"].includes(currentExt)}
            <!-- svelte-ignore a11y-media-has-caption -->
            <video controls src={rawUrl(currentPath)}></video>
          {:else}
            <p class="binary-note">No preview for .{currentExt} files.</p>
          {/if}
          <a class="btn btn-sm download-btn" href="{rawUrl(currentPath)}&download=1">
            <Download size={14} /> Download {fileName(currentPath)}
          </a>
        </div>
      {:else}
        <div class="doc-title">
          <span class="doc-path">{currentPath}</span>{#if dirty}<span class="dirty-dot" title="Unsaved changes">●</span>{/if}
          <span class="doc-mode">{view === "source" || !currentPath.endsWith(".md") ? "Source" : "Preview"}</span>
        </div>
        {#if view === "source" || !currentPath.endsWith(".md")}
          <textarea class="source-editor" bind:value={content} spellcheck="false" placeholder="Write…"
          ></textarea>
        {:else}
          <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
          <article class="preview" on:click={handlePreviewClick}>{@html renderedHtml}</article>
        {/if}
      {/if}
    </main>
  </div>
</div>

<style>
  .notes-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    height: 100dvh;
    background: var(--bg);
    color: var(--ink-2);
  }

  /* ── App bar ─────────────────────────────────────────── */

  .appbar {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
    min-height: 56px;
    padding: 0 var(--s-5);
    border-bottom: 1px solid var(--line);
    background: var(--bg);
  }

  .appbar-left {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    min-width: 0;
  }

  .wordmark {
    display: inline-flex;
    flex-shrink: 0;
    align-items: center;
    gap: 8px;
    color: var(--ink);
    font-size: 16px;
    font-weight: 500;
    letter-spacing: -0.025em;
    white-space: nowrap;
  }
  .wordmark .dot {
    width: 8px;
    height: 8px;
  }
  .wordmark:hover {
    color: var(--ink);
  }

  .crumb {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    min-width: 0;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    white-space: nowrap;
  }
  .sep {
    color: var(--line-strong);
  }

  .back-link {
    color: var(--ink-3);
  }
  .back-link:hover {
    color: var(--ink);
  }

  h1 {
    margin: 0;
    overflow: hidden;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
    letter-spacing: 0;
    line-height: 1.4;
    text-overflow: ellipsis;
  }

  .branch {
    color: var(--ink-3);
  }

  .signed-in {
    overflow: hidden;
    margin-left: var(--s-2);
    padding-left: var(--s-3);
    border-left: 1px solid var(--line);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 20px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .appbar-actions {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: var(--s-2);
  }

  .appbar-actions .btn {
    gap: 6px;
  }

  .divider {
    width: 1px;
    height: 20px;
    margin: 0 var(--s-1);
    background: var(--line);
  }

  .btn.danger:hover:not(:disabled) {
    border-color: var(--danger);
    color: var(--danger);
    background: transparent;
  }

  /* ── Banners ─────────────────────────────────────────── */

  .banner {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-5);
    border-bottom: 1px solid var(--line);
    background: var(--surface);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.6;
  }

  .banner.error {
    color: var(--danger);
  }

  .banner.ok {
    color: var(--ink-2);
  }

  /* ── Body ────────────────────────────────────────────── */

  .notes-body {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .file-tree {
    flex-shrink: 0;
    width: 280px;
    overflow-y: auto;
    padding: var(--s-4) var(--s-3) var(--s-6);
    border-right: 1px solid var(--line);
    background: var(--bg);
  }

  .tree-head {
    display: flex;
    justify-content: space-between;
    padding: 0 var(--s-2) var(--s-3);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .tree-count {
    font-variant-numeric: tabular-nums;
  }

  .tree-hint {
    padding: var(--s-1) var(--s-2);
    color: var(--ink-3);
  }

  .tree-root {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .tree-root > li {
    margin: 0;
  }

  .tree-file {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    width: 100%;
    min-height: 28px;
    padding: 0 var(--s-2);
    border: 0;
    border-radius: var(--radius-xs);
    background: none;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    text-align: left;
    white-space: nowrap;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
  }
  .tree-file :global(svg) {
    flex-shrink: 0;
    color: var(--ink-3);
  }
  .tree-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tree-file:hover {
    background: var(--hover-wash);
    color: var(--ink);
  }

  .tree-file.active {
    background: var(--fjord-wash);
    color: var(--fjord);
  }
  .tree-file.active :global(svg) {
    color: var(--fjord);
  }

  .editor-pane {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    background: var(--surface);
  }

  .empty-state {
    display: flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    padding: var(--s-6);
    color: var(--ink-3);
    font-size: var(--text-sm);
    text-align: center;
  }

  .doc-title {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: var(--s-2);
    height: 40px;
    padding: 0 var(--s-5);
    border-bottom: 1px solid var(--line);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .doc-path {
    min-width: 0;
    overflow: hidden;
    color: var(--ink);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dirty-dot {
    flex-shrink: 0;
    color: var(--clay);
    font-size: 8px;
  }

  .doc-mode {
    flex-shrink: 0;
    margin-left: auto;
    color: var(--ink-3);
  }

  .source-editor {
    flex: 1;
    width: 100%;
    padding: var(--s-6) max(var(--s-5), calc((100% - 76ch) / 2)) var(--s-9);
    border: 0;
    outline: none;
    resize: none;
    background: none;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: 14px;
    line-height: 1.75;
    tab-size: 2;
  }
  .source-editor::placeholder {
    color: var(--ink-3);
  }

  .binary-pane {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-5);
    min-height: 0;
    overflow: auto;
    padding: var(--s-6) var(--s-5);
    background: var(--surface-2);
  }

  .binary-preview {
    max-width: 100%;
    max-height: 70vh;
    object-fit: contain;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
  }

  .binary-frame {
    flex: 1;
    width: 100%;
    min-height: 0;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
  }

  .binary-pane video {
    max-width: 100%;
    max-height: 70vh;
    border-radius: var(--radius-sm);
  }

  .binary-note {
    margin: 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
  }

  .download-btn {
    flex-shrink: 0;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* ── Rendered note: a calm reading column ────────────── */

  .preview {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-7) max(var(--s-5), calc((100% - var(--measure)) / 2)) var(--s-10);
    color: var(--ink-2);
    font-size: 17px;
    line-height: 1.7;
  }

  .preview :global(h1),
  .preview :global(h2),
  .preview :global(h3),
  .preview :global(h4) {
    margin: 1.8em 0 0.5em;
    color: var(--ink);
    font-weight: 500;
    line-height: 1.2;
  }

  .preview :global(h1) {
    margin-top: 0;
    font-size: var(--text-2xl);
    line-height: 1.08;
    letter-spacing: -0.03em;
  }

  .preview :global(h2) {
    font-size: var(--text-xl);
    letter-spacing: -0.022em;
  }

  .preview :global(h3) {
    font-size: var(--text-lg);
    letter-spacing: -0.015em;
  }

  .preview :global(h4) {
    font-size: var(--text-md);
  }

  .preview :global(p) {
    margin: 0 0 1em;
    color: var(--ink-2);
  }

  .preview :global(strong) {
    color: var(--ink);
  }

  .preview :global(a) {
    color: var(--ink);
    text-decoration: underline;
    text-decoration-color: var(--line-strong);
    text-decoration-thickness: 1px;
    text-underline-offset: 0.22em;
  }

  .preview :global(a:hover) {
    color: var(--fjord);
    text-decoration-color: var(--fjord);
  }

  .preview :global(img) {
    display: block;
    max-width: 100%;
    height: auto;
    margin: var(--s-5) 0;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
  }

  .preview :global(ul),
  .preview :global(ol) {
    margin: 0 0 1em;
    padding-left: 1.4em;
  }

  .preview :global(li) {
    margin: 0.25em 0;
  }

  .preview :global(li::marker) {
    color: var(--ink-3);
  }

  .preview :global(li > input[type="checkbox"]) {
    margin-right: var(--s-2);
    accent-color: var(--moss);
  }

  .preview :global(pre) {
    margin: var(--s-5) 0;
    padding: var(--s-4) var(--s-5);
    overflow-x: auto;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--code-surface);
    font-size: var(--text-sm);
    line-height: 1.6;
  }

  .preview :global(code) {
    font-family: var(--font-mono);
    font-size: 0.86em;
  }

  .preview :global(:not(pre) > code) {
    padding: 0.1em 0.35em;
    border-radius: var(--radius-xs);
    background: var(--code-surface);
    color: var(--ink);
  }

  .preview :global(blockquote) {
    margin: var(--s-5) 0;
    padding: 0 0 0 var(--s-4);
    border-left: 2px solid var(--line-strong);
    color: var(--ink-2);
    font-family: var(--font-serif);
    font-size: 1.1em;
    font-style: italic;
  }

  .preview :global(hr) {
    margin: var(--s-7) 0;
    border: 0;
    border-top: 1px solid var(--line);
  }

  .preview :global(table) {
    width: 100%;
    margin: var(--s-5) 0;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  .preview :global(th),
  .preview :global(td) {
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--line);
    text-align: left;
  }

  .preview :global(th) {
    background: var(--surface-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
  }

  /* ── Responsive ──────────────────────────────────────── */

  @media (max-width: 1100px) {
    .signed-in {
      display: none;
    }
  }

  @media (max-width: 960px) {
    .appbar-actions .btn:not(.btn-primary) .lbl {
      display: none;
    }
    .appbar-actions .btn:not(.btn-primary) {
      min-width: 32px;
      padding: 0 var(--s-2);
    }
  }

  @media (max-width: 768px) {
    .notes-body {
      flex-direction: column;
    }

    .file-tree {
      width: 100%;
      max-height: 35vh;
      border-right: 0;
      border-bottom: 1px solid var(--line);
    }
  }

  @media (max-width: 640px) {
    .appbar {
      flex-wrap: wrap;
      gap: 0;
      padding: 0 var(--s-4);
    }
    .appbar-left {
      height: 56px;
    }
    .appbar-actions {
      width: calc(100% + 2 * var(--s-4));
      margin: 0 calc(-1 * var(--s-4));
      padding: var(--s-2) var(--s-4);
      overflow-x: auto;
      border-top: 1px solid var(--line);
      scrollbar-width: none;
    }
    .appbar-actions::-webkit-scrollbar {
      display: none;
    }
    .appbar-actions .divider {
      margin-left: auto;
      visibility: hidden;
    }
    .banner,
    .doc-title {
      padding-left: var(--s-4);
      padding-right: var(--s-4);
    }
  }
</style>
