<script lang="ts">
  import MonacoEditor from "$lib/components/MonacoEditor.svelte";
  import { compileTypstToPdf } from "$lib/typst";
  import type { PageData } from "./$types";
  import {
    LogOut,
    UploadCloud,
    FileDown,
    History,
    ChevronDown,
    CheckCircle,
    AlertTriangle,
    Play,
    Loader2,
  } from "lucide-svelte";

  export let data: PageData;

  let typstContent = "";
  let pdfUrl = "";
  let loading = true;
  let compiling = false;
  let uploading = false;
  let uploadProgress = 0;
  let error = "";
  let compilationError = "";
  let hasUnsavedChanges = false;
  let isDownloadOpen = false;

  $: resume = data.resume;

  function loadTypstContent() {
    try {
      typstContent = resume.typst_source ?? "";
      pdfUrl = resume.pdf_url;
    } catch (e) {
      error = "Failed to load resume";
    } finally {
      loading = false;
    }
  }

  $: if (resume) loadTypstContent();

  function handleLogout() {
    // Cloudflare Access logout clears the edge session.
    window.location.href = "/cdn-cgi/access/logout";
  }

  async function handleCompile() {
    compilationError = "";
    compiling = true;

    try {
      // Compile entirely in the browser via typst.ts (WASM) — no server.
      const pdf = await compileTypstToPdf(typstContent);
      const blob = new Blob([pdf], { type: "application/pdf" });
      if (pdfUrl?.startsWith("blob:")) URL.revokeObjectURL(pdfUrl);
      pdfUrl = URL.createObjectURL(blob);
      hasUnsavedChanges = false;
    } catch (e: any) {
      compilationError = e?.message ?? String(e);
    } finally {
      compiling = false;
    }
  }

  function handleEditorChange(newValue: string) {
    typstContent = newValue;
    hasUnsavedChanges = true;
  }

  async function downloadPDF() {
    isDownloadOpen = false;
    if (pdfUrl.startsWith("blob:")) {
      const a = document.createElement("a");
      a.href = pdfUrl;
      a.download = "Kashiful_Haque.pdf";
      a.click();
    } else {
      window.open(resume.pdf_url, "_blank");
    }
  }

  async function downloadTypst() {
    isDownloadOpen = false;
    const blob = new Blob([typstContent], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = resume.typst_filename || "resume.typ";
    a.click();
    URL.revokeObjectURL(url);
  }

  async function handleUpload() {
    if (hasUnsavedChanges || !pdfUrl?.startsWith("blob:")) {
      await handleCompile();
      if (compilationError) {
        alert("Fix compilation errors first.");
        return;
      }
    }

    if (!confirm("Upload new version?")) return;

    uploading = true;
    uploadProgress = 5;

    const progressInterval = setInterval(() => {
      if (uploadProgress < 90) uploadProgress += Math.random() * 5;
    }, 300);

    try {
      const apiUrl = typeof window !== "undefined" && window.location.hostname === "localhost"
        ? "http://localhost:8787/api/resume/upload"
        : "https://ifkash.dev/api/resume/upload";

      const pdfBlob = await fetch(pdfUrl).then((r) => r.blob());
      const formData = new FormData();
      formData.append("typst_source", typstContent);
      formData.append("pdf_file", pdfBlob, "resume.pdf");
      formData.append("version", (parseInt(resume.version || "0") + 1).toString());

      // Auth is enforced at the edge by Cloudflare Access (cookie sent same-origin).
      const response = await fetch(apiUrl, {
        method: "POST",
        body: formData,
      });

      if (!response.ok) throw new Error(await response.text() || "Upload failed");

      uploadProgress = 100;
      clearInterval(progressInterval);
      setTimeout(() => window.location.reload(), 500);
    } catch (e: any) {
      clearInterval(progressInterval);
      uploadProgress = 0;
      alert("Upload failed: " + e.message);
      uploading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      handleCompile();
    }
  }
</script>

<svelte:head>
  <title>Editor — Kashif</title>
</svelte:head>

<svelte:window on:keydown={handleKeydown} />

<div class="editor">
  <!-- Toolbar -->
  <header class="toolbar">
    <div class="toolbar-left">
      <a href="/" class="toolbar-logo">KH</a>
      <div class="toolbar-title">
        <span>Editor</span>
        {#if resume.version}<span class="version">v{resume.version}</span>{/if}
      </div>
      <div class="status">
        {#if uploading}
          <Loader2 size={12} class="spin" /><span>Uploading</span>
        {:else if compiling}
          <Loader2 size={12} class="spin" /><span>Compiling</span>
        {:else if hasUnsavedChanges}
          <AlertTriangle size={12} /><span>Unsaved</span>
        {:else}
          <CheckCircle size={12} /><span>Saved</span>
        {/if}
      </div>
    </div>

    <div class="toolbar-right">
      <a href="/editor/history" class="tb-btn" title="History"><History size={16} /></a>
      <div class="tb-divider"></div>
      <button on:click={handleCompile} disabled={compiling || uploading} class="tb-btn" title="Compile">
        <Play size={14} /><span class="tb-label">Run</span>
      </button>
      <div class="dropdown">
        <button on:click={() => (isDownloadOpen = !isDownloadOpen)} class="tb-btn">
          <FileDown size={16} /><ChevronDown size={12} />
        </button>
        {#if isDownloadOpen}
          <div class="dropdown-menu">
            <button on:click={downloadPDF}>PDF</button>
            <button on:click={downloadTypst}>Typst</button>
          </div>
          <button class="dropdown-bg" on:click={() => (isDownloadOpen = false)} aria-label="Close"></button>
        {/if}
      </div>
      <button on:click={handleUpload} disabled={uploading || compiling} class="tb-btn primary">
        {#if uploading}<Loader2 size={14} class="spin" />{:else}<UploadCloud size={14} />{/if}
        <span class="tb-label">Upload</span>
      </button>
      <div class="tb-divider"></div>
      <button on:click={handleLogout} class="tb-btn logout" title="Logout"><LogOut size={16} /></button>
    </div>
  </header>

  <!-- Progress -->
  {#if uploading}
    <div class="progress"><div class="progress-bar" style="width: {uploadProgress}%"></div></div>
  {/if}

  <!-- Error -->
  {#if compilationError}
    <div class="error-bar"><AlertTriangle size={14} /> {compilationError}</div>
  {/if}

  <!-- Main -->
  <div class="main">
    <div class="editor-pane">
      {#if loading}
        <div class="loading"><Loader2 size={20} class="spin" /> Loading...</div>
      {/if}
      <MonacoEditor bind:value={typstContent} language="plaintext" theme="vs-dark" onChange={handleEditorChange} />
    </div>
    <div class="preview-pane">
      {#if compiling}
        <div class="compiling"><Loader2 size={20} class="spin" /> Compiling...</div>
      {/if}
      {#if pdfUrl}
        <iframe src={pdfUrl} title="PDF"></iframe>
      {:else}
        <div class="no-preview">No PDF. Compile to preview.</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .editor {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--paper);
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem;
    height: 52px;
    background: var(--surface-raised);
    border-bottom: 1px solid var(--border);
  }

  .toolbar-left, .toolbar-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .toolbar-logo {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 0.875rem;
    color: var(--paper);
    background: var(--ink);
    border-radius: var(--radius-sm);
  }

  .toolbar-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .version {
    font-size: 0.6875rem;
    color: var(--text-tertiary);
    padding: 0.125rem 0.375rem;
    background: var(--surface-sunken);
    border-radius: var(--radius-sm);
  }

  .status {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .tb-btn {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.625rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--dur-instant) var(--ease-out-quart);
  }

  .tb-btn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--surface-sunken);
  }

  .tb-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .tb-btn.primary {
    color: var(--paper);
    background: var(--ink);
  }

  .tb-btn.primary:hover:not(:disabled) {
    background: var(--paper-dim);
  }

  .tb-btn.logout:hover {
    color: var(--text-primary);
  }

  .tb-label {
    display: none;
  }

  @media (min-width: 640px) {
    .tb-label { display: inline; }
  }

  .tb-divider {
    width: 1px;
    height: 20px;
    background: var(--border);
  }

  .dropdown {
    position: relative;
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 0.5rem;
    padding: 0.375rem;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    z-index: 50;
  }

  .dropdown-menu button {
    display: block;
    width: 100%;
    padding: 0.5rem 0.75rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    text-align: left;
    cursor: pointer;
  }

  .dropdown-menu button:hover {
    color: var(--text-primary);
    background: var(--border);
  }

  .dropdown-bg {
    position: fixed;
    inset: 0;
    z-index: 40;
    background: transparent;
    border: none;
  }

  .progress {
    height: 2px;
    background: var(--surface-sunken);
  }

  .progress-bar {
    height: 100%;
    background: var(--ink);
    transition: width 0.3s;
  }

  .error-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    font-size: 0.8125rem;
    color: var(--text-primary);
    background: var(--surface-sunken);
    border-bottom: 1px solid var(--border);
  }

  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  @media (min-width: 768px) {
    .main { flex-direction: row; }
  }

  .editor-pane {
    flex: 1;
    position: relative;
    border-bottom: 1px solid var(--border);
    min-height: 300px;
  }

  @media (min-width: 768px) {
    .editor-pane {
      border-bottom: none;
      border-right: 1px solid var(--border);
    }
  }

  .preview-pane {
    flex: 1;
    position: relative;
    background: var(--surface-raised);
    min-height: 300px;
  }

  .preview-pane iframe {
    width: 100%;
    height: 100%;
    border: none;
  }

  .loading, .compiling, .no-preview {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    color: var(--text-tertiary);
    background: var(--surface-raised);
    z-index: 10;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
