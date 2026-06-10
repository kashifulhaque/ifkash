<script lang="ts">
  import { onMount } from "svelte";
  import { Marked } from "marked";
  import katex from "katex";
  import "katex/dist/katex.min.css";
  import DOMPurify from "dompurify";
  import TreeDir from "./TreeDir.svelte";
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
    Loader2,
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
  <header class="notes-header">
    <div class="header-left">
      <a href="/admin" class="back-link">&larr; Admin</a>
      <h1>Notes{#if branch}<span class="branch"> · {branch}</span>{/if}</h1>
      <span class="signed-in">{data.email}</span>
    </div>
    <div class="header-actions">
      <button class="btn" on:click={newFile} title="New note">
        <Plus size={14} /> New
      </button>
      <button
        class="btn"
        on:click={() => uploadInput.click()}
        disabled={uploading}
        title="Upload a file (image, PDF, …) to the vault"
      >
        {#if uploading}<Loader2 size={14} class="spin" />{:else}<Upload size={14} />{/if}
        Upload
      </button>
      <button class="btn" on:click={loadTree} disabled={loadingTree} title="Re-fetch the file tree from GitHub">
        <RefreshCw size={14} /> Refresh
      </button>
      <button
        class="btn"
        on:click={() => (view = view === "source" ? "preview" : "source")}
        disabled={!currentPath || !currentPath.endsWith(".md")}
        title="Toggle source / rendered view"
      >
        {#if view === "source"}<Eye size={14} /> Preview{:else}<Code size={14} /> Source{/if}
      </button>
      <button
        class="btn danger"
        on:click={deleteFile}
        disabled={!currentPath || saving}
        title="Delete this file from the vault"
      >
        <Trash2 size={14} /> Delete
      </button>
      <button
        class="btn primary"
        on:click={saveFile}
        disabled={!currentPath || binary || !dirty || saving}
      >
        {#if saving}<Loader2 size={14} class="spin" />{:else}<Save size={14} />{/if}
        Save &amp; push
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
    <div class="banner error"><AlertTriangle size={14} /> {errorMsg}</div>
  {:else if statusMsg}
    <div class="banner ok">{statusMsg}</div>
  {/if}

  <div class="notes-body">
    <aside class="file-tree">
      {#if loadingTree}
        <p class="tree-hint">Loading vault…</p>
      {:else if root}
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
                {fileName(f.path)}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </aside>

    <main class="editor-pane">
      {#if !currentPath}
        <div class="empty-state">Select a note from the vault, or create a new one.</div>
      {:else if loadingFile}
        <div class="empty-state"><Loader2 size={18} class="spin" /></div>
      {:else if binary}
        <div class="doc-title">{currentPath}</div>
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
          <a class="btn download-btn" href="{rawUrl(currentPath)}&download=1">
            <Download size={14} /> Download {fileName(currentPath)}
          </a>
        </div>
      {:else}
        <div class="doc-title">
          {currentPath}{#if dirty}<span class="dirty-dot" title="Unsaved changes">●</span>{/if}
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
    background: var(--paper, #fff);
  }

  .notes-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--rule-soft);
    flex-wrap: wrap;
  }

  .header-left {
    display: flex;
    align-items: baseline;
    gap: 12px;
    min-width: 0;
  }

  .back-link {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    color: var(--ink-mute);
  }

  .back-link:hover {
    color: var(--blueprint);
  }

  h1 {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 0.95rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ink);
  }

  .branch {
    color: var(--blueprint);
    text-transform: none;
    letter-spacing: 0;
  }

  .signed-in {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--ink-mute);
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 1px solid var(--rule-soft);
    background: none;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--ink-soft);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .btn:hover:not(:disabled) {
    background: var(--blueprint-tint);
    color: var(--blueprint);
  }

  .btn:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .btn.primary {
    border-color: var(--blueprint);
    color: var(--blueprint);
  }

  .btn.danger:hover:not(:disabled) {
    background: #fef3f2;
    border-color: #b42318;
    color: #b42318;
  }

  .binary-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    padding: 24px;
    overflow: auto;
    min-height: 0;
  }

  .binary-preview {
    max-width: 100%;
    max-height: 70vh;
    object-fit: contain;
    border-radius: 4px;
  }

  .binary-frame {
    width: 100%;
    flex: 1;
    min-height: 0;
    border: 1px solid var(--rule-soft);
  }

  .binary-pane video {
    max-width: 100%;
    max-height: 70vh;
  }

  .binary-note {
    font-family: var(--font-mono);
    font-size: 0.85rem;
    color: var(--ink-mute);
  }

  .download-btn {
    text-decoration: none;
    flex-shrink: 0;
  }

  .banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    border-bottom: 1px solid var(--rule-soft);
  }

  .banner.error {
    color: #b42318;
    background: #fef3f2;
  }

  .banner.ok {
    color: var(--blueprint);
    background: var(--blueprint-tint);
  }

  .notes-body {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .file-tree {
    width: 280px;
    flex-shrink: 0;
    overflow-y: auto;
    border-right: 1px solid var(--rule-soft);
    padding: 10px 8px;
  }

  .tree-hint {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    color: var(--ink-mute);
    padding: 4px 6px;
  }

  .tree-root {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .tree-root > li {
    margin: 0;
  }

  .tree-file {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 3px 6px;
    border: 0;
    background: none;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    color: var(--ink-soft);
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tree-file:hover,
  .tree-file.active {
    background: var(--blueprint-tint);
    color: var(--blueprint);
  }

  .editor-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono);
    font-size: 0.85rem;
    color: var(--ink-mute);
  }

  .doc-title {
    padding: 8px 20px;
    border-bottom: 1px solid var(--rule-soft);
    font-family: var(--font-mono);
    font-size: 0.78rem;
    color: var(--ink-mute);
  }

  .dirty-dot {
    margin-left: 8px;
    color: var(--blueprint);
  }

  .source-editor {
    flex: 1;
    width: 100%;
    padding: 20px;
    border: 0;
    resize: none;
    outline: none;
    background: none;
    font-family: var(--font-mono);
    font-size: 0.9rem;
    line-height: 1.7;
    color: var(--ink);
  }

  /* Notion-style rendered view: centered measure, calm typography. */
  .preview {
    flex: 1;
    overflow-y: auto;
    padding: 40px max(32px, calc((100% - 720px) / 2)) 120px;
    font-size: 1rem;
    line-height: 1.65;
    color: var(--ink);
    cursor: default;
  }

  .preview :global(h1),
  .preview :global(h2),
  .preview :global(h3),
  .preview :global(h4) {
    font-weight: 650;
    line-height: 1.3;
    color: var(--ink);
    margin: 1.6em 0 0.4em;
  }

  .preview :global(h1) {
    font-size: 1.8em;
    margin-top: 0.5em;
  }

  .preview :global(h2) {
    font-size: 1.4em;
  }

  .preview :global(h3) {
    font-size: 1.15em;
  }

  .preview :global(h4) {
    font-size: 1em;
  }

  .preview :global(p) {
    margin: 0.5em 0;
  }

  .preview :global(a) {
    color: var(--blueprint);
    text-decoration: underline;
    text-decoration-color: color-mix(in srgb, var(--blueprint) 35%, transparent);
    text-underline-offset: 3px;
  }

  .preview :global(a:hover) {
    text-decoration-color: var(--blueprint);
  }

  .preview :global(img) {
    display: block;
    max-width: 100%;
    height: auto;
    margin: 1em 0;
    border-radius: 4px;
  }

  .preview :global(ul),
  .preview :global(ol) {
    margin: 0.4em 0;
    padding-left: 1.5em;
  }

  .preview :global(li) {
    margin: 0.2em 0;
  }

  .preview :global(li > input[type="checkbox"]) {
    margin-right: 8px;
    accent-color: var(--blueprint);
  }

  .preview :global(pre) {
    background: color-mix(in srgb, var(--ink) 5%, transparent);
    border: 1px solid var(--rule-soft);
    border-radius: 6px;
    padding: 14px 16px;
    overflow-x: auto;
    font-size: 0.85rem;
    line-height: 1.55;
    margin: 1em 0;
  }

  .preview :global(code) {
    font-family: var(--font-mono);
    font-size: 0.85em;
  }

  .preview :global(:not(pre) > code) {
    background: color-mix(in srgb, var(--ink) 6%, transparent);
    border-radius: 4px;
    padding: 0.15em 0.4em;
    color: #c4554d;
  }

  .preview :global(blockquote) {
    border-left: 3px solid var(--ink);
    margin: 0.8em 0;
    padding: 0.1em 0 0.1em 16px;
    color: var(--ink-soft);
  }

  .preview :global(hr) {
    border: 0;
    border-top: 1px solid var(--rule-soft);
    margin: 2em 0;
  }

  .preview :global(table) {
    border-collapse: collapse;
    margin: 1em 0;
    font-size: 0.92em;
    width: 100%;
  }

  .preview :global(th),
  .preview :global(td) {
    border: 1px solid var(--rule-soft);
    padding: 6px 12px;
    text-align: left;
  }

  .preview :global(th) {
    background: color-mix(in srgb, var(--ink) 4%, transparent);
    font-weight: 600;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
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
      border-bottom: 1px solid var(--rule-soft);
    }
  }
</style>
