<svelte:head>
  <title>PDF Annotator — Tools</title>
  <meta
    name="description"
    content="Annotate PDFs in the browser — text, freehand drawing, signatures. Fully client-side."
  />
</svelte:head>

<script lang="ts">
  import { tick } from "svelte";
  import {
    MousePointer2,
    Type,
    Pencil,
    Highlighter,
    Image as ImageIcon,
    Undo2,
    Redo2,
    Trash2,
    Download,
    FolderOpen,
    ChevronLeft,
    Loader2,
  } from "lucide-svelte";
  import {
    RENDER_SCALE,
    loadPdfDocument,
    renderPage,
    exportAnnotatedPdf,
    downloadPdf,
    type Annotation,
    type DrawAnnotation,
    type HighlightAnnotation,
    type TextAnnotation,
    type ImageAnnotation,
    type ToolKind,
    type Point,
  } from "$lib/pdfAnnotator";
  import type { PDFDocumentProxy } from "pdfjs-dist";

  type PageMeta = {
    index: number;
    pageNumber: number;
    width: number;
    height: number;
  };

  let pdfDoc: PDFDocumentProxy | null = null;
  let originalBytes: Uint8Array | null = null;
  let fileName = "";
  let pages: PageMeta[] = [];
  let loadingPdf = false;
  let loadError = "";
  let exporting = false;

  let annotations: Annotation[] = [];
  let tool: ToolKind = "select";
  let color = "#e5484d";
  let strokeWidth = 3;
  let fontSize = 18;
  let selectedId: string | null = null;
  let editingId: string | null = null;

  const PRESETS = ["#e5484d", "#2f6fed", "#16a34a", "#eab308", "#111111"];

  // Undo/redo as full snapshots of the annotations model.
  let past: Annotation[][] = [];
  let future: Annotation[][] = [];

  // Live (in-progress) vector annotation while drawing.
  let current: DrawAnnotation | HighlightAnnotation | null = null;
  let drawing = false;
  let hlStart: Point = { x: 0, y: 0 };

  let pendingImage: { dataUrl: string; w: number; h: number } | null = null;

  const overlayMap = new Map<number, HTMLCanvasElement>();

  let pdfInput: HTMLInputElement;
  let imageInput: HTMLInputElement;

  const uid = () =>
    typeof crypto !== "undefined" && crypto.randomUUID
      ? crypto.randomUUID()
      : Math.random().toString(36).slice(2);

  $: hasPdf = pages.length > 0;
  $: captureCursor =
    tool === "text" ? "text" : tool === "image" ? "copy" : "crosshair";

  // ─── History ───────────────────────────────────────────────
  const snapshot = (): Annotation[] => JSON.parse(JSON.stringify(annotations));

  function pushHistory() {
    past = [...past, snapshot()];
    future = [];
  }

  function undo() {
    if (!past.length) return;
    future = [snapshot(), ...future];
    annotations = past[past.length - 1];
    past = past.slice(0, -1);
    selectedId = null;
    editingId = null;
  }

  function redo() {
    if (!future.length) return;
    past = [...past, snapshot()];
    annotations = future[0];
    future = future.slice(1);
    selectedId = null;
    editingId = null;
  }

  // ─── PDF loading ───────────────────────────────────────────
  async function openPdfFile(file: File) {
    loadError = "";
    loadingPdf = true;
    try {
      const buf = await file.arrayBuffer();
      fileName = file.name.replace(/\.pdf$/i, "");
      // Keep a clean copy for export (pdf.js detaches the buffer it receives).
      originalBytes = new Uint8Array(buf.slice(0));
      annotations = [];
      past = [];
      future = [];
      selectedId = null;
      editingId = null;
      pages = [];
      overlayMap.clear();

      const doc = await loadPdfDocument(buf);
      pdfDoc = doc;
      const list: PageMeta[] = [];
      for (let i = 1; i <= doc.numPages; i++) {
        const page = await doc.getPage(i);
        const vp = page.getViewport({ scale: RENDER_SCALE });
        list.push({
          index: i - 1,
          pageNumber: i,
          width: Math.floor(vp.width),
          height: Math.floor(vp.height),
        });
      }
      pages = list;
      tool = "select";
    } catch (e) {
      loadError = e instanceof Error ? e.message : "Failed to load PDF";
    } finally {
      loadingPdf = false;
    }
  }

  function onPdfInput(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (file) openPdfFile(file);
    (e.target as HTMLInputElement).value = "";
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    const file = e.dataTransfer?.files?.[0];
    if (file && file.type === "application/pdf") openPdfFile(file);
  }

  // ─── Rendering actions ─────────────────────────────────────
  function registerBase(node: HTMLCanvasElement, pageNumber: number) {
    if (pdfDoc) renderPage(pdfDoc, pageNumber, node, RENDER_SCALE).catch(() => {});
    return {};
  }

  function registerOverlay(node: HTMLCanvasElement, pageIndex: number) {
    overlayMap.set(pageIndex, node);
    redrawOverlay(pageIndex);
    return {
      destroy() {
        overlayMap.delete(pageIndex);
      },
    };
  }

  function hexToRgba(hex: string, alpha: number) {
    const c = hex.replace("#", "");
    const f =
      c.length === 3
        ? c
            .split("")
            .map((x) => x + x)
            .join("")
        : c;
    const n = parseInt(f, 16);
    return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${alpha})`;
  }

  function drawStroke(ctx: CanvasRenderingContext2D, a: DrawAnnotation) {
    if (!a.points.length) return;
    ctx.strokeStyle = a.color;
    ctx.fillStyle = a.color;
    ctx.lineWidth = a.width;
    ctx.lineJoin = "round";
    ctx.lineCap = "round";
    if (a.points.length === 1) {
      ctx.beginPath();
      ctx.arc(a.points[0].x, a.points[0].y, a.width / 2, 0, Math.PI * 2);
      ctx.fill();
      return;
    }
    ctx.beginPath();
    ctx.moveTo(a.points[0].x, a.points[0].y);
    for (let i = 1; i < a.points.length; i++)
      ctx.lineTo(a.points[i].x, a.points[i].y);
    ctx.stroke();
  }

  function redrawOverlay(pageIndex: number) {
    const cv = overlayMap.get(pageIndex);
    if (!cv) return;
    const ctx = cv.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, cv.width, cv.height);
    const list: Annotation[] = annotations.filter(
      (a) =>
        a.pageIndex === pageIndex &&
        (a.type === "draw" || a.type === "highlight")
    );
    if (current && current.pageIndex === pageIndex) list.push(current);
    for (const a of list) {
      if (a.type === "draw") drawStroke(ctx, a);
      else if (a.type === "highlight") {
        ctx.fillStyle = hexToRgba(a.color, 0.35);
        ctx.fillRect(a.x, a.y, a.w, a.h);
      }
    }
  }

  function redrawOverlays() {
    for (const p of pages) redrawOverlay(p.index);
  }

  // Redraw committed vector layers whenever the model changes.
  $: annotations, redrawOverlays();

  // ─── Capture-layer interactions (draw / highlight / place) ─
  function localPoint(e: PointerEvent, el: HTMLElement): Point {
    const r = el.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  }

  function onCaptureDown(e: PointerEvent, page: PageMeta) {
    const pt = localPoint(e, e.currentTarget as HTMLElement);
    if (tool === "text") {
      createText(page.index, pt);
      return;
    }
    if (tool === "image") {
      placeImage(page.index, pt);
      return;
    }
    if (tool === "draw") {
      pushHistory();
      current = {
        id: uid(),
        type: "draw",
        pageIndex: page.index,
        color,
        width: strokeWidth,
        points: [pt],
      };
      drawing = true;
      redrawOverlay(page.index);
    } else if (tool === "highlight") {
      pushHistory();
      hlStart = pt;
      current = {
        id: uid(),
        type: "highlight",
        pageIndex: page.index,
        color,
        x: pt.x,
        y: pt.y,
        w: 0,
        h: 0,
      };
      drawing = true;
    }
  }

  function onCaptureMove(e: PointerEvent, page: PageMeta) {
    if (!drawing || !current) return;
    const pt = localPoint(e, e.currentTarget as HTMLElement);
    if (current.type === "draw") {
      current.points = [...current.points, pt];
    } else {
      current.x = Math.min(hlStart.x, pt.x);
      current.y = Math.min(hlStart.y, pt.y);
      current.w = Math.abs(pt.x - hlStart.x);
      current.h = Math.abs(pt.y - hlStart.y);
    }
    redrawOverlay(page.index);
  }

  function onCaptureUp(page: PageMeta) {
    if (!drawing || !current) return;
    drawing = false;
    const committed = current;
    current = null;
    const keep =
      committed.type === "draw"
        ? committed.points.length > 0
        : committed.w > 2 && committed.h > 2;
    if (keep) {
      annotations = [...annotations, committed];
    } else {
      // Discard the empty gesture and the history entry it created.
      past = past.slice(0, -1);
      redrawOverlay(page.index);
    }
  }

  // ─── Text annotations ──────────────────────────────────────
  function createText(pageIndex: number, pt: Point) {
    pushHistory();
    const id = uid();
    const ann: TextAnnotation = {
      id,
      type: "text",
      pageIndex,
      color,
      x: pt.x,
      y: pt.y,
      w: 220,
      text: "Text",
      fontSize,
    };
    annotations = [...annotations, ann];
    selectedId = id;
    tool = "select";
    enterEdit(ann, true);
  }

  async function enterEdit(ann: TextAnnotation, selectAll = false) {
    if (editingId !== ann.id) {
      // Editing later (not on fresh create) is its own undo step.
      if (!selectAll) pushHistory();
      editingId = ann.id;
    }
    await tick();
    const el = document.querySelector<HTMLElement>(
      `[data-edit-id="${ann.id}"]`
    );
    if (!el) return;
    el.focus();
    if (selectAll) {
      const range = document.createRange();
      range.selectNodeContents(el);
      const sel = window.getSelection();
      sel?.removeAllRanges();
      sel?.addRange(range);
    }
  }

  function onTextInput(ann: TextAnnotation, e: Event) {
    ann.text = (e.currentTarget as HTMLElement).innerText.replace(/\n$/, "");
    annotations = annotations;
  }

  function onTextBlur(ann: TextAnnotation) {
    if (editingId === ann.id) editingId = null;
    if (!ann.text.trim()) {
      annotations = annotations.filter((a) => a.id !== ann.id);
      if (selectedId === ann.id) selectedId = null;
    }
  }

  // Sets the editable element's text without clobbering the caret while typing.
  function editableText(node: HTMLElement, text: string) {
    node.textContent = text;
    return {
      update(newText: string) {
        if (document.activeElement !== node && node.textContent !== newText)
          node.textContent = newText;
      },
    };
  }

  // ─── Image / signature annotations ─────────────────────────
  function pickImage() {
    imageInput?.click();
  }

  function onImageInput(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    (e.target as HTMLInputElement).value = "";
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = reader.result as string;
      const img = new Image();
      img.onload = () => {
        pendingImage = {
          dataUrl,
          w: img.naturalWidth || 200,
          h: img.naturalHeight || 120,
        };
        tool = "image";
      };
      img.src = dataUrl;
    };
    reader.readAsDataURL(file);
  }

  function placeImage(pageIndex: number, pt: Point) {
    if (!pendingImage) return;
    pushHistory();
    const id = uid();
    const ratio = pendingImage.w / pendingImage.h || 1;
    const w = Math.min(220, pendingImage.w);
    const ann: ImageAnnotation = {
      id,
      type: "image",
      pageIndex,
      color,
      x: pt.x,
      y: pt.y,
      w,
      h: w / ratio,
      dataUrl: pendingImage.dataUrl,
    };
    annotations = [...annotations, ann];
    selectedId = id;
    tool = "select";
    pendingImage = null;
  }

  // ─── Drag / resize (select mode) ───────────────────────────
  function startDrag(e: PointerEvent, ann: TextAnnotation | ImageAnnotation) {
    if (tool !== "select") return;
    if (editingId === ann.id) return; // let text editing place the caret
    e.stopPropagation();
    selectedId = ann.id;
    const sx = e.clientX;
    const sy = e.clientY;
    const ax = ann.x;
    const ay = ann.y;
    let moved = false;
    const move = (ev: PointerEvent) => {
      if (!moved) {
        if (Math.hypot(ev.clientX - sx, ev.clientY - sy) < 3) return;
        moved = true;
        pushHistory();
      }
      ann.x = ax + (ev.clientX - sx);
      ann.y = ay + (ev.clientY - sy);
      annotations = annotations;
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }

  function startResize(
    e: PointerEvent,
    ann: TextAnnotation | ImageAnnotation,
    mode: "image" | "text"
  ) {
    e.stopPropagation();
    selectedId = ann.id;
    const sx = e.clientX;
    const aw = ann.w;
    const ah = "h" in ann ? (ann as ImageAnnotation).h : 0;
    const ratio = ah ? aw / ah : 1;
    let moved = false;
    const move = (ev: PointerEvent) => {
      if (!moved) {
        moved = true;
        pushHistory();
      }
      const nw = Math.max(24, aw + (ev.clientX - sx));
      ann.w = nw;
      if (mode === "image") (ann as ImageAnnotation).h = nw / ratio;
      annotations = annotations;
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }

  function deleteAnn(ann: Annotation) {
    pushHistory();
    annotations = annotations.filter((a) => a.id !== ann.id);
    if (selectedId === ann.id) selectedId = null;
    if (editingId === ann.id) editingId = null;
  }

  function clearAll() {
    if (!annotations.length) return;
    pushHistory();
    annotations = [];
    selectedId = null;
    editingId = null;
  }

  function onStageClick(e: MouseEvent) {
    if (tool !== "select") return;
    if ((e.target as HTMLElement).closest(".anno")) return;
    selectedId = null;
  }

  // ─── Color / size apply to selection ───────────────────────
  function applyColor(c: string) {
    color = c;
    const sel = annotations.find((a) => a.id === selectedId);
    if (sel) {
      pushHistory();
      sel.color = c;
      annotations = annotations;
    }
  }

  function applyFontSize(v: number) {
    fontSize = v;
    const sel = annotations.find((a) => a.id === selectedId);
    if (sel && sel.type === "text") {
      sel.fontSize = v;
      annotations = annotations;
    }
  }

  function onColorInput(e: Event) {
    applyColor((e.target as HTMLInputElement).value);
  }

  function onFontInput(e: Event) {
    applyFontSize(+(e.target as HTMLInputElement).value);
  }

  // ─── Export ────────────────────────────────────────────────
  async function onDownload() {
    if (!originalBytes) return;
    exporting = true;
    try {
      const bytes = await exportAnnotatedPdf(
        originalBytes,
        annotations,
        RENDER_SCALE
      );
      downloadPdf(bytes, `${fileName || "document"}-annotated.pdf`);
    } catch (e) {
      loadError = e instanceof Error ? e.message : "Export failed";
    } finally {
      exporting = false;
    }
  }

  // ─── Keyboard ──────────────────────────────────────────────
  function onKeydown(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    const editing =
      !!t &&
      (t.isContentEditable ||
        t.tagName === "INPUT" ||
        t.tagName === "TEXTAREA");
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "z") {
      e.preventDefault();
      if (e.shiftKey) redo();
      else undo();
      return;
    }
    if (
      !editing &&
      (e.key === "Delete" || e.key === "Backspace") &&
      selectedId
    ) {
      e.preventDefault();
      const a = annotations.find((x) => x.id === selectedId);
      if (a) deleteAnn(a);
    }
  }

  const TOOLS: { id: ToolKind; icon: typeof Type; label: string }[] = [
    { id: "select", icon: MousePointer2, label: "Select" },
    { id: "text", icon: Type, label: "Text" },
    { id: "draw", icon: Pencil, label: "Draw" },
    { id: "highlight", icon: Highlighter, label: "Highlight" },
  ];

  function selectTool(id: ToolKind) {
    tool = id;
    if (id !== "select") {
      selectedId = null;
      editingId = null;
    }
  }
</script>

<svelte:window on:keydown={onKeydown} />

<input
  type="file"
  accept="application/pdf"
  bind:this={pdfInput}
  on:change={onPdfInput}
  hidden
/>
<input
  type="file"
  accept="image/png,image/jpeg"
  bind:this={imageInput}
  on:change={onImageInput}
  hidden
/>

<div class="annotator">
  <header class="bar">
    <div class="bar-group left">
      <a href="/tools" class="back" aria-label="Back to tools">
        <ChevronLeft size={18} />
      </a>
      <span class="title">PDF Annotator</span>
      {#if fileName}<span class="file">{fileName}.pdf</span>{/if}
    </div>

    {#if hasPdf}
      <div class="bar-group tools">
        {#each TOOLS as t}
          <button
            class="tb"
            class:active={tool === t.id}
            on:click={() => selectTool(t.id)}
            title={t.label}
            aria-label={t.label}
          >
            <svelte:component this={t.icon} size={17} />
          </button>
        {/each}
        <button
          class="tb"
          class:active={tool === "image"}
          on:click={pickImage}
          title="Image / signature"
          aria-label="Image or signature"
        >
          <ImageIcon size={17} />
        </button>

        <span class="divider"></span>

        <div class="swatches">
          {#each PRESETS as c}
            <button
              class="swatch"
              class:active={color === c}
              style="background:{c}"
              on:click={() => applyColor(c)}
              aria-label="Colour {c}"
            ></button>
          {/each}
          <input
            type="color"
            class="color-input"
            value={color}
            on:input={onColorInput}
            aria-label="Custom colour"
          />
        </div>

        {#if tool === "draw" || tool === "highlight"}
          <label class="slider" title="Stroke width">
            <Pencil size={13} />
            <input
              type="range"
              min="1"
              max="20"
              bind:value={strokeWidth}
            />
          </label>
        {/if}
        {#if tool === "text" || (selectedId && annotations.find((a) => a.id === selectedId)?.type === "text")}
          <label class="slider" title="Font size">
            <Type size={13} />
            <input
              type="range"
              min="10"
              max="56"
              value={fontSize}
              on:input={onFontInput}
            />
          </label>
        {/if}
      </div>

      <div class="bar-group right">
        <button
          class="tb"
          on:click={undo}
          disabled={!past.length}
          title="Undo"
          aria-label="Undo"
        >
          <Undo2 size={17} />
        </button>
        <button
          class="tb"
          on:click={redo}
          disabled={!future.length}
          title="Redo"
          aria-label="Redo"
        >
          <Redo2 size={17} />
        </button>
        <button
          class="tb"
          on:click={() => {
            const a = annotations.find((x) => x.id === selectedId);
            if (a) deleteAnn(a);
          }}
          disabled={!selectedId}
          title="Delete selected"
          aria-label="Delete selected"
        >
          <Trash2 size={17} />
        </button>
        <button
          class="tb"
          on:click={clearAll}
          disabled={!annotations.length}
          title="Clear all"
        >
          <span class="tb-text">Clear</span>
        </button>
        <span class="divider"></span>
        <button class="tb" on:click={() => pdfInput.click()} title="Open PDF">
          <FolderOpen size={17} />
          <span class="tb-text">Open</span>
        </button>
        <button
          class="tb primary"
          on:click={onDownload}
          disabled={exporting}
          title="Download annotated PDF"
        >
          {#if exporting}
            <Loader2 size={16} class="spin" />
          {:else}
            <Download size={16} />
          {/if}
          <span class="tb-text">Download</span>
        </button>
      </div>
    {/if}
  </header>

  {#if loadError}
    <div class="error">{loadError}</div>
  {/if}

  {#if !hasPdf}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="empty"
      on:dragover|preventDefault
      on:drop={onDrop}
    >
      {#if loadingPdf}
        <Loader2 size={28} class="spin" />
        <p>Loading PDF…</p>
      {:else}
        <div class="empty-card">
          <h2>Annotate a PDF</h2>
          <p>
            Add text, draw, and stamp a signature — then download. Everything
            stays in your browser.
          </p>
          <button class="open-btn" on:click={() => pdfInput.click()}>
            <FolderOpen size={18} /> Choose PDF
          </button>
          <span class="hint">or drop a file here</span>
        </div>
      {/if}
    </div>
  {:else}
    <!-- svelte-ignore a11y-no-static-element-interactions a11y-click-events-have-key-events -->
    <div class="stage" on:click={onStageClick}>
      {#each pages as page (page.index)}
        <div
          class="page"
          style="width:{page.width}px;height:{page.height}px;"
        >
          <canvas class="base" use:registerBase={page.pageNumber}></canvas>
          <canvas
            class="overlay"
            width={page.width}
            height={page.height}
            use:registerOverlay={page.index}
          ></canvas>

          <div class="anno-layer">
            {#each annotations.filter((a) => a.pageIndex === page.index) as ann (ann.id)}
              {#if ann.type === "text"}
                <!-- svelte-ignore a11y-no-static-element-interactions a11y-click-events-have-key-events -->
                <div
                  class="anno anno-text"
                  class:selected={selectedId === ann.id}
                  style="left:{ann.x}px;top:{ann.y}px;width:{ann.w}px;color:{ann.color};font-size:{ann.fontSize}px;"
                  on:pointerdown={(e) => startDrag(e, ann)}
                  on:dblclick={() => enterEdit(ann)}
                >
                  <div
                    class="te"
                    data-edit-id={ann.id}
                    contenteditable={editingId === ann.id}
                    use:editableText={ann.text}
                    on:input={(e) => onTextInput(ann, e)}
                    on:blur={() => onTextBlur(ann)}
                  ></div>
                  {#if selectedId === ann.id}
                    <button
                      class="h del"
                      on:pointerdown|stopPropagation
                      on:click={() => deleteAnn(ann)}
                      aria-label="Delete">×</button
                    >
                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                    <div
                      class="h grip-w"
                      on:pointerdown={(e) => startResize(e, ann, "text")}
                    ></div>
                  {/if}
                </div>
              {:else if ann.type === "image"}
                <!-- svelte-ignore a11y-no-static-element-interactions -->
                <div
                  class="anno anno-image"
                  class:selected={selectedId === ann.id}
                  style="left:{ann.x}px;top:{ann.y}px;width:{ann.w}px;height:{ann.h}px;"
                  on:pointerdown={(e) => startDrag(e, ann)}
                >
                  <img src={ann.dataUrl} alt="annotation" draggable="false" />
                  {#if selectedId === ann.id}
                    <button
                      class="h del"
                      on:pointerdown|stopPropagation
                      on:click={() => deleteAnn(ann)}
                      aria-label="Delete">×</button
                    >
                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                    <div
                      class="h grip-corner"
                      on:pointerdown={(e) => startResize(e, ann, "image")}
                    ></div>
                  {/if}
                </div>
              {/if}
            {/each}
          </div>

          {#if tool !== "select"}
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
              class="capture"
              style="cursor:{captureCursor};"
              on:pointerdown={(e) => onCaptureDown(e, page)}
              on:pointermove={(e) => onCaptureMove(e, page)}
              on:pointerup={() => onCaptureUp(page)}
              on:pointerleave={() => onCaptureUp(page)}
            ></div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .annotator {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    overflow: hidden;
  }

  .bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--rule-soft);
    flex-wrap: wrap;
  }

  .bar-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .bar-group.tools {
    flex: 1;
    flex-wrap: wrap;
    justify-content: center;
  }

  .bar-group.left {
    min-width: 0;
  }

  .back {
    display: inline-flex;
    align-items: center;
    color: var(--ink-soft);
    border: 1px solid var(--rule-soft);
    padding: 5px;
  }

  .back:hover {
    color: var(--blueprint);
    border-color: var(--blueprint);
  }

  .title {
    font-family: var(--font-display);
    font-size: 1.25rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--ink);
    white-space: nowrap;
  }

  .file {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--ink-mute);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 160px;
  }

  .tb {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 9px;
    background: transparent;
    color: var(--ink-soft);
    border: 1px solid transparent;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    cursor: pointer;
    transition: color 0.12s, background 0.12s, border-color 0.12s;
  }

  .tb:hover:not(:disabled) {
    color: var(--ink);
    background: var(--bg-surface-hover);
  }

  .tb.active {
    color: var(--blueprint);
    border-color: var(--blueprint);
    background: var(--blueprint-tint);
  }

  .tb:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .tb.primary {
    color: var(--bg);
    background: var(--blueprint);
    border-color: var(--blueprint);
  }

  .tb.primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .tb-text {
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .divider {
    width: 1px;
    height: 20px;
    background: var(--rule-soft);
    margin: 0 2px;
  }

  .swatches {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .swatch {
    width: 18px;
    height: 18px;
    border: 1px solid var(--rule-soft);
    cursor: pointer;
    padding: 0;
  }

  .swatch.active {
    outline: 2px solid var(--blueprint);
    outline-offset: 1px;
  }

  .color-input {
    width: 24px;
    height: 22px;
    padding: 0;
    border: 1px solid var(--rule-soft);
    background: transparent;
    cursor: pointer;
  }

  .slider {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--ink-soft);
  }

  .slider input {
    width: 90px;
    accent-color: var(--blueprint);
  }

  .error {
    padding: 8px 14px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--warn);
    color: var(--warn);
    font-family: var(--font-mono);
    font-size: 0.8rem;
  }

  /* ─── Empty state ─────────────────────────────────────── */
  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--ink-soft);
    flex-direction: column;
    gap: 12px;
  }

  .empty-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    text-align: center;
    max-width: 420px;
    padding: 40px;
    border: 1px dashed var(--rule-soft);
  }

  .empty-card h2 {
    font-family: var(--font-display);
    color: var(--blueprint);
  }

  .empty-card p {
    color: var(--ink-soft);
    font-size: 0.95rem;
  }

  .open-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 18px;
    background: var(--blueprint);
    color: var(--bg);
    border: none;
    font-family: var(--font-mono);
    font-size: 0.82rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    cursor: pointer;
  }

  .open-btn:hover {
    background: var(--accent-hover);
  }

  .hint {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--ink-mute);
  }

  /* ─── Document stage ──────────────────────────────────── */
  .stage {
    flex: 1;
    overflow: auto;
    padding: 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
  }

  .page {
    position: relative;
    background: #fff;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.4);
    flex-shrink: 0;
  }

  .base,
  .overlay {
    position: absolute;
    top: 0;
    left: 0;
    pointer-events: none;
  }

  .overlay {
    z-index: 2;
  }

  .anno-layer {
    position: absolute;
    inset: 0;
    z-index: 3;
    pointer-events: none;
  }

  .anno {
    position: absolute;
    pointer-events: auto;
  }

  .anno.selected {
    outline: 1px dashed var(--blueprint);
    outline-offset: 2px;
  }

  .anno-text {
    cursor: move;
    min-height: 1em;
  }

  .anno-text .te {
    font-family: var(--font-body);
    line-height: 1.2;
    outline: none;
    white-space: pre-wrap;
    word-break: break-word;
    min-width: 8px;
    min-height: 1em;
    cursor: text;
  }

  .anno-image {
    cursor: move;
  }

  .anno-image img {
    display: block;
    width: 100%;
    height: 100%;
    user-select: none;
  }

  .h {
    position: absolute;
    z-index: 4;
  }

  .del {
    top: -10px;
    right: -10px;
    width: 18px;
    height: 18px;
    line-height: 1;
    border: none;
    background: var(--warn);
    color: var(--bg);
    border-radius: 50%;
    cursor: pointer;
    font-size: 13px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .grip-corner {
    right: -6px;
    bottom: -6px;
    width: 12px;
    height: 12px;
    background: var(--blueprint);
    cursor: nwse-resize;
  }

  .grip-w {
    right: -6px;
    top: 50%;
    transform: translateY(-50%);
    width: 10px;
    height: 18px;
    background: var(--blueprint);
    cursor: ew-resize;
  }

  .capture {
    position: absolute;
    inset: 0;
    z-index: 5;
    touch-action: none;
  }

  :global(.spin) {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 768px) {
    .title {
      font-size: 1.05rem;
    }
    .file {
      display: none;
    }
    .stage {
      padding: 12px;
    }
  }
</style>
