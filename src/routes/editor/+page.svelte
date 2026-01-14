<script lang="ts">
  import { auth } from "$lib/stores/auth";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import MonacoEditor from "$lib/components/MonacoEditor.svelte";
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
    Sparkles,
    Undo2,
    X,
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

  let isAiModalOpen = false;
  let isAiProcessing = false;
  let jobDescription = "";
  let selectedModel = "anthropic/claude-3.5-sonnet";
  let previousTypstContent = "";
  let canUndo = false;

  const AI_MODELS = [
    { id: "anthropic/claude-3.5-sonnet", name: "Claude 3.5 Sonnet" },
    { id: "google/gemini-flash-1.5", name: "Gemini 1.5 Flash" },
    { id: "openai/gpt-4o-mini", name: "GPT-4o Mini" },
    { id: "meta-llama/llama-3.1-70b-instruct", name: "Llama 3.1 70B" },
  ];

  $: resume = data.resume;

  async function loadTypstContent() {
    try {
      const response = await fetch(resume.typst_url);
      if (!response.ok) throw new Error("Failed to fetch Typst file");
      typstContent = await response.text();
      pdfUrl = resume.pdf_url;
    } catch (e) {
      error = "Failed to load Typst file";
    } finally {
      loading = false;
      if (browser) {
        const savedModel = localStorage.getItem("preferred_ai_model");
        if (savedModel) selectedModel = savedModel;
      }
    }
  }

  $: if (resume) loadTypstContent();

  function handleLogout() {
    auth.logout();
    goto("/");
  }

  async function handleCompile() {
    compilationError = "";
    compiling = true;

    try {
      const token = auth.getToken();
      const apiUrl = typeof window !== "undefined" && window.location.hostname === "localhost"
        ? "http://localhost:8787/api/typst/compile"
        : "https://ifkash.dev/api/typst/compile";

      const response = await fetch(apiUrl, {
        method: "POST",
        headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
        body: JSON.stringify({ code: typstContent }),
      });

      if (!response.ok) throw new Error(await response.text() || "Compilation failed");

      const blob = await response.blob();
      if (pdfUrl?.startsWith("blob:")) URL.revokeObjectURL(pdfUrl);
      pdfUrl = URL.createObjectURL(blob);
      hasUnsavedChanges = false;
    } catch (e: any) {
      compilationError = e.message;
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
      const token = auth.getToken();
      const apiUrl = typeof window !== "undefined" && window.location.hostname === "localhost"
        ? "http://localhost:8787/api/resume/upload"
        : "https://ifkash.dev/api/resume/upload";

      const pdfBlob = await fetch(pdfUrl).then((r) => r.blob());
      const formData = new FormData();
      formData.append("typst_file", new Blob([typstContent], { type: "text/plain" }), resume.typst_filename || "resume.typ");
      formData.append("pdf_file", pdfBlob, "resume.pdf");
      formData.append("version", (parseInt(resume.version || "0") + 1).toString());

      const response = await fetch(apiUrl, {
        method: "POST",
        headers: { Authorization: `Bearer ${token}` },
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

  function handleUndo() {
    if (canUndo && previousTypstContent) {
      typstContent = previousTypstContent;
      previousTypstContent = "";
      canUndo = false;
      hasUnsavedChanges = true;
    }
  }

  async function handleAiRewrite() {
    if (!jobDescription.trim()) return;

    isAiProcessing = true;
    previousTypstContent = typstContent;

    try {
      if (browser) localStorage.setItem("preferred_ai_model", selectedModel);

      const token = auth.getToken();
      const apiUrl = typeof window !== "undefined" && window.location.hostname === "localhost"
        ? "http://localhost:8787/api/ai/rewrite"
        : "https://ifkash.dev/api/ai/rewrite";

      const response = await fetch(apiUrl, {
        method: "POST",
        headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
        body: JSON.stringify({ code: typstContent, job_description: jobDescription, model: selectedModel }),
      });

      if (!response.ok) throw new Error(await response.text() || "AI failed");

      const data = await response.json();
      if (data.success && data.code) {
        typstContent = data.code;
        hasUnsavedChanges = true;
        canUndo = true;
        isAiModalOpen = false;
      } else {
        throw new Error(data.message || "Unknown error");
      }
    } catch (e: any) {
      alert("AI failed: " + e.message);
    } finally {
      isAiProcessing = false;
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
      {#if canUndo}
        <button on:click={handleUndo} class="tb-btn" title="Undo"><Undo2 size={16} /></button>
      {/if}
      <button on:click={() => (isAiModalOpen = true)} disabled={compiling || uploading} class="tb-btn" title="AI">
        <Sparkles size={16} />
      </button>
      <div class="tb-divider"></div>
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

  <!-- AI Modal -->
  {#if isAiModalOpen}
    <div class="modal-overlay">
      <div class="modal">
        <div class="modal-header">
          <h3><Sparkles size={16} /> AI Tailor</h3>
          <button on:click={() => (isAiModalOpen = false)}><X size={18} /></button>
        </div>
        <div class="modal-body">
          <label for="ai-model">Model</label>
          <select id="ai-model" bind:value={selectedModel}>
            {#each AI_MODELS as m}<option value={m.id}>{m.name}</option>{/each}
          </select>
          <label for="jd">Job Description</label>
          <textarea id="jd" bind:value={jobDescription} placeholder="Paste JD here..."></textarea>
        </div>
        <div class="modal-footer">
          <button on:click={() => (isAiModalOpen = false)} class="btn-cancel">Cancel</button>
          <button on:click={handleAiRewrite} disabled={isAiProcessing || !jobDescription.trim()} class="btn-submit">
            {#if isAiProcessing}<Loader2 size={14} class="spin" /> Processing{:else}Tailor{/if}
          </button>
        </div>
      </div>
    </div>
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
    background: var(--black);
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem;
    height: 52px;
    background: var(--gray-950);
    border-bottom: 1px solid var(--gray-800);
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
    color: var(--black);
    background: var(--white);
    border-radius: var(--radius-sm);
  }

  .toolbar-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--white);
  }

  .version {
    font-size: 0.6875rem;
    color: var(--gray-500);
    padding: 0.125rem 0.375rem;
    background: var(--gray-900);
    border-radius: var(--radius-sm);
  }

  .status {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.75rem;
    color: var(--gray-500);
  }

  .tb-btn {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.625rem;
    font-size: 0.8125rem;
    color: var(--gray-400);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--duration-fast) var(--ease-out);
  }

  .tb-btn:hover:not(:disabled) {
    color: var(--white);
    background: var(--gray-900);
  }

  .tb-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .tb-btn.primary {
    color: var(--black);
    background: var(--white);
  }

  .tb-btn.primary:hover:not(:disabled) {
    background: var(--gray-200);
  }

  .tb-btn.logout:hover {
    color: var(--white);
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
    background: var(--gray-800);
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
    background: var(--gray-900);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
    z-index: 50;
  }

  .dropdown-menu button {
    display: block;
    width: 100%;
    padding: 0.5rem 0.75rem;
    font-size: 0.8125rem;
    color: var(--gray-300);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    text-align: left;
    cursor: pointer;
  }

  .dropdown-menu button:hover {
    color: var(--white);
    background: var(--gray-800);
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
    background: var(--gray-900);
  }

  .progress-bar {
    height: 100%;
    background: var(--white);
    transition: width 0.3s;
  }

  .error-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    font-size: 0.8125rem;
    color: var(--white);
    background: var(--gray-900);
    border-bottom: 1px solid var(--gray-800);
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.8);
  }

  .modal {
    width: 100%;
    max-width: 420px;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-lg);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--gray-800);
  }

  .modal-header h3 {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--white);
  }

  .modal-header button {
    padding: 0.25rem;
    color: var(--gray-500);
    background: transparent;
    border: none;
    cursor: pointer;
  }

  .modal-header button:hover {
    color: var(--white);
  }

  .modal-body {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .modal-body label {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--gray-400);
  }

  .modal-body select,
  .modal-body textarea {
    padding: 0.625rem 0.875rem;
    font-size: 0.875rem;
    color: var(--white);
    background: var(--black);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
  }

  .modal-body textarea {
    min-height: 140px;
    resize: none;
  }

  .modal-body select:focus,
  .modal-body textarea:focus {
    outline: none;
    border-color: var(--gray-600);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--gray-800);
  }

  .btn-cancel {
    padding: 0.5rem 1rem;
    font-size: 0.8125rem;
    color: var(--gray-400);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .btn-cancel:hover {
    color: var(--white);
  }

  .btn-submit {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 1rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--black);
    background: var(--white);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .btn-submit:hover:not(:disabled) {
    background: var(--gray-200);
  }

  .btn-submit:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
    border-bottom: 1px solid var(--gray-800);
    min-height: 300px;
  }

  @media (min-width: 768px) {
    .editor-pane {
      border-bottom: none;
      border-right: 1px solid var(--gray-800);
    }
  }

  .preview-pane {
    flex: 1;
    position: relative;
    background: var(--gray-950);
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
    color: var(--gray-500);
    background: var(--gray-950);
    z-index: 10;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
