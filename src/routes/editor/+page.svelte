<script lang="ts">
  import MonacoEditor from "$lib/components/MonacoEditor.svelte";
  import LoadingState from "$lib/components/LoadingState.svelte";
  import { compileTypstToPdf } from "$lib/typst";
  import type { PageData } from "./$types";
  import {
    LogOut,
    UploadCloud,
    FileDown,
    History,
    ChevronDown,
    AlertTriangle,
    Play,
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
  <!-- App bar -->
  <header class="appbar">
    <div class="appbar-left">
      <a href="/" class="wordmark" aria-label="ifkash.dev home"><span class="dot" aria-hidden="true"></span> ifkash</a>
      <div class="crumb">
        <span class="sep" aria-hidden="true">/</span>
        <span class="crumb-page">Editor</span>
        {#if resume.version}<span class="version">v{resume.version}</span>{/if}
      </div>
      <div class="status">
        {#if uploading}
          <LoadingState label="Uploading" />
        {:else if compiling}
          <LoadingState label="Compiling" />
        {:else if hasUnsavedChanges}
          <span class="led led-hollow" aria-hidden="true"></span><span class="status-text">Unsaved</span>
        {:else}
          <span class="led" aria-hidden="true"></span><span class="status-text">Saved</span>
        {/if}
      </div>
    </div>

    <div class="appbar-actions">
      <a href="/editor/history" class="btn btn-sm btn--ghost icon" title="History" aria-label="History"><History size={16} /></a>
      <span class="divider" aria-hidden="true"></span>
      <button on:click={handleCompile} disabled={compiling || uploading} class="btn btn-sm" title="Compile">
        <Play size={14} /><span class="tb-label">Run</span>
      </button>
      <div class="dropdown">
        <button on:click={() => (isDownloadOpen = !isDownloadOpen)} class="btn btn-sm" aria-label="Download" aria-expanded={isDownloadOpen}>
          <FileDown size={15} /><ChevronDown size={12} />
        </button>
        {#if isDownloadOpen}
          <div class="dropdown-menu">
            <button on:click={downloadPDF}>PDF</button>
            <button on:click={downloadTypst}>Typst</button>
          </div>
          <button class="dropdown-bg" on:click={() => (isDownloadOpen = false)} aria-label="Close"></button>
        {/if}
      </div>
      <button on:click={handleUpload} disabled={uploading || compiling} class="btn btn-sm btn-primary">
        <UploadCloud size={14} />
        <span class="tb-label">Upload</span>
      </button>
      <span class="divider" aria-hidden="true"></span>
      <button on:click={handleLogout} class="btn btn-sm btn--ghost icon" title="Logout" aria-label="Logout"><LogOut size={16} /></button>
    </div>
  </header>

  <!-- Progress -->
  {#if uploading}
    <div class="progress"><div class="progress-bar" style="width: {uploadProgress}%"></div></div>
  {/if}

  <!-- Error -->
  {#if compilationError}
    <div class="error-bar" role="alert"><AlertTriangle size={14} /> <span class="error-text">{compilationError}</span></div>
  {/if}

  <!-- Main -->
  <div class="main">
    <section class="pane editor-pane">
      <div class="pane-head" aria-hidden="true">
        <span>Source</span>
        <span class="pane-meta">{resume.typst_filename || "resume.typ"}</span>
      </div>
      <div class="pane-body">
        {#if loading}
          <div class="loading"><LoadingState label="Loading" /></div>
        {/if}
        <MonacoEditor bind:value={typstContent} language="plaintext" theme="auto" onChange={handleEditorChange} />
      </div>
    </section>
    <section class="pane preview-pane">
      <div class="pane-head" aria-hidden="true">
        <span>Preview</span>
        <span class="pane-meta">pdf</span>
      </div>
      <div class="pane-body">
        {#if compiling}
          <div class="compiling"><LoadingState label="Compiling" /></div>
        {/if}
        {#if pdfUrl}
          <iframe src={pdfUrl} title="PDF"></iframe>
        {:else}
          <div class="no-preview"><span class="led led-hollow" aria-hidden="true"></span> No PDF. Compile to preview.</div>
        {/if}
      </div>
    </section>
  </div>
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    height: 100vh;
    height: 100dvh;
    background: var(--bg);
    color: var(--ink-2);
  }

  /* ── App bar ─────────────────────────────────────────── */

  .appbar {
    position: relative;
    z-index: 30;
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
    height: 56px;
    padding: 0 var(--s-4);
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
  .crumb-page {
    color: var(--ink);
  }

  .version {
    padding: 1px 6px;
    border: 1px solid var(--line);
    border-radius: var(--radius-pill);
    color: var(--ink-3);
    font-variant-numeric: tabular-nums;
  }

  .status {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-left: var(--s-2);
    padding-left: var(--s-4);
    border-left: 1px solid var(--line);
    height: 20px;
    min-width: 0;
    overflow: hidden;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    white-space: nowrap;
  }

  .appbar-actions {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: var(--s-2);
  }

  .btn.icon {
    width: 32px;
    padding: 0;
  }

  .tb-label {
    display: none;
  }

  .divider {
    width: 1px;
    height: 20px;
    margin: 0 var(--s-1);
    background: var(--line);
  }

  .dropdown {
    position: relative;
  }

  .dropdown .btn {
    gap: var(--s-1);
  }

  .dropdown-menu {
    position: absolute;
    top: calc(100% + var(--s-2));
    right: 0;
    z-index: 50;
    min-width: 140px;
    padding: var(--s-1);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
    box-shadow: 0 12px 32px -12px var(--shadow-color);
    animation: fade-in var(--dur-fast) var(--ease);
  }

  .dropdown-menu button {
    display: block;
    width: 100%;
    padding: var(--s-2) var(--s-3);
    border: none;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--ink-2);
    font-size: var(--text-sm);
    text-align: left;
    cursor: pointer;
  }

  .dropdown-menu button:hover {
    background: var(--hover-wash);
    color: var(--ink);
  }

  .dropdown-bg {
    position: fixed;
    inset: 0;
    z-index: 40;
    border: none;
    background: transparent;
  }

  /* ── Progress and errors ─────────────────────────────── */

  .progress {
    flex-shrink: 0;
    height: 1px;
    background: var(--line);
  }

  .progress-bar {
    height: 100%;
    background: var(--fjord);
    transition: width var(--dur) var(--ease);
  }

  .error-bar {
    display: flex;
    flex-shrink: 0;
    align-items: flex-start;
    gap: var(--s-2);
    max-height: 30vh;
    overflow: auto;
    padding: var(--s-3) var(--s-4);
    border-bottom: 1px solid var(--line);
    background: var(--surface);
    color: var(--danger);
    font-size: var(--text-sm);
  }
  .error-bar :global(svg) {
    flex-shrink: 0;
    margin-top: 2px;
  }
  .error-text {
    min-width: 0;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.6;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  /* ── Panes ───────────────────────────────────────────── */

  .main {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .pane {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    min-height: 300px;
  }

  .pane-head {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    height: 36px;
    padding: 0 var(--s-4);
    border-bottom: 1px solid var(--line);
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .pane-meta {
    overflow: hidden;
    color: var(--ink-3);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pane-body {
    position: relative;
    flex: 1;
    min-height: 0;
  }

  .editor-pane {
    border-bottom: 1px solid var(--line);
    background: var(--surface);
  }

  .preview-pane {
    background: var(--surface-2);
  }
  .preview-pane .pane-head {
    background: var(--bg);
  }

  .preview-pane iframe {
    display: block;
    width: 100%;
    height: 100%;
    border: none;
  }

  .loading,
  .compiling,
  .no-preview {
    position: absolute;
    inset: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--s-2);
    color: var(--ink-3);
    font-size: var(--text-sm);
  }
  .loading {
    background: var(--surface);
  }
  .compiling {
    background: color-mix(in srgb, var(--surface-2) 88%, transparent);
  }

  @media (max-width: 479px) {
    .status-text,
    .divider {
      display: none;
    }
    .status {
      margin-left: 0;
      padding-left: var(--s-3);
    }
  }

  @media (max-width: 400px) {
    .crumb {
      display: none;
    }
    .appbar-actions {
      gap: var(--s-1);
    }
  }

  @media (min-width: 640px) {
    .tb-label {
      display: inline;
    }
    .appbar {
      padding: 0 var(--s-5);
    }
    .pane-head {
      padding: 0 var(--s-5);
    }
  }

  @media (min-width: 768px) {
    .main {
      flex-direction: row;
    }
    .editor-pane {
      border-bottom: none;
      border-right: 1px solid var(--line);
    }
  }
</style>
