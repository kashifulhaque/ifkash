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
  } from "lucide-svelte";
  import LoadingState from "$lib/components/LoadingState.svelte";
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
    tool === "text"
      ? "text"
      : tool === "image"
        ? "copy"
        : "crosshair";

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
  <header class="appbar">
    <div class="appbar-left">
      <a href="/" class="wordmark" aria-label="ifkash.dev home"><span class="dot" aria-hidden="true"></span> ifkash</a>
      <nav class="crumb" aria-label="Breadcrumb">
        <span class="sep" aria-hidden="true">/</span>
        <a href="/tools" class="back" aria-label="Back to tools">tools</a>
        <span class="sep" aria-hidden="true">/</span>
        <span class="title" aria-current="page">PDF Annotator</span>
        {#if fileName}<span class="file">{fileName}.pdf</span>{/if}
      </nav>
    </div>

    {#if hasPdf}
      <div class="appbar-actions">
        <button class="btn btn-sm" on:click={() => pdfInput.click()} title="Open PDF">
          <FolderOpen size={15} />
          <span class="tb-text">Open</span>
        </button>
        <button
          class="btn btn-sm btn-primary"
          on:click={onDownload}
          disabled={exporting}
          title="Download annotated PDF"
        >
          {#if exporting}
            <LoadingState label="Exporting" />
          {:else}
            <Download size={15} />
            <span class="tb-text">Download</span>
          {/if}
        </button>
      </div>
    {/if}
  </header>

  {#if hasPdf}
    <div class="toolstrip" role="toolbar" aria-label="Annotation tools">
      <div class="bar-group tools">
        {#each TOOLS as t}
          <button
            class="tb"
            class:active={tool === t.id}
            on:click={() => selectTool(t.id)}
            title={t.label}
            aria-label={t.label}
            aria-pressed={tool === t.id}
          >
            <svelte:component this={t.icon} size={16} />
          </button>
        {/each}
        <button
          class="tb"
          class:active={tool === "image"}
          on:click={pickImage}
          title="Image / signature"
          aria-label="Image or signature"
          aria-pressed={tool === "image"}
        >
          <ImageIcon size={16} />
        </button>

        <span class="divider" aria-hidden="true"></span>

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
          <span class="divider" aria-hidden="true"></span>
          <label class="slider" title="Stroke width">
            <Pencil size={13} />
            <input
              type="range"
              min="1"
              max="20"
              bind:value={strokeWidth}
            />
            <span class="slider-val">{strokeWidth}</span>
          </label>
        {/if}
        {#if tool === "text" || (selectedId && annotations.find((a) => a.id === selectedId)?.type === "text")}
          <span class="divider" aria-hidden="true"></span>
          <label class="slider" title="Font size">
            <Type size={13} />
            <input
              type="range"
              min="10"
              max="56"
              value={fontSize}
              on:input={onFontInput}
            />
            <span class="slider-val">{fontSize}</span>
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
          <Undo2 size={16} />
        </button>
        <button
          class="tb"
          on:click={redo}
          disabled={!future.length}
          title="Redo"
          aria-label="Redo"
        >
          <Redo2 size={16} />
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
          <Trash2 size={16} />
        </button>
        <button
          class="tb tb-wide"
          on:click={clearAll}
          disabled={!annotations.length}
          title="Clear all"
        >
          <span>Clear</span>
        </button>
      </div>
    </div>
  {/if}

  {#if loadError}
    <div class="error" role="alert">{loadError}</div>
  {/if}

  {#if !hasPdf}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="empty"
      on:dragover|preventDefault
      on:drop={onDrop}
    >
      {#if loadingPdf}
        <LoadingState label="Loading PDF" />
      {:else}
        <div class="empty-card">
          <p class="eyebrow">Client-side · nothing uploaded</p>
          <h2>Annotate a PDF</h2>
          <p>
            Add text, draw, and stamp a signature — then download. Everything
            stays in your browser.
          </p>
          <button class="btn btn-primary open-btn" on:click={() => pdfInput.click()}>
            <FolderOpen size={16} /> Choose PDF
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
    display: flex;
    flex-direction: column;
    height: 100vh;
    height: 100dvh;
    overflow: hidden;
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
    height: 56px;
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
  .back {
    color: var(--ink-3);
  }
  .back:hover {
    color: var(--ink);
  }
  .title {
    color: var(--ink);
  }
  .file {
    max-width: 200px;
    margin-left: var(--s-2);
    padding: 1px 8px;
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: var(--radius-pill);
    color: var(--ink-3);
    text-overflow: ellipsis;
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

  /* ── Tool strip ──────────────────────────────────────── */

  .toolstrip {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    min-height: 48px;
    padding: var(--s-1) var(--s-5);
    overflow-x: auto;
    border-bottom: 1px solid var(--line);
    background: var(--surface);
    scrollbar-width: none;
  }
  .toolstrip::-webkit-scrollbar {
    display: none;
  }

  .bar-group {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: 2px;
  }

  .tb {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-2);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background-color var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
  }
  .tb-wide {
    width: auto;
    padding: 0 var(--s-3);
    font-size: var(--text-sm);
  }

  .tb:hover:not(:disabled) {
    background: var(--hover-wash);
    color: var(--ink);
  }

  .tb.active {
    border-color: var(--line);
    background: var(--fjord-wash);
    color: var(--fjord);
  }

  .tb:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .divider {
    flex-shrink: 0;
    width: 1px;
    height: 20px;
    margin: 0 var(--s-2);
    background: var(--line);
  }

  .swatches {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  /* Swatch fills are annotation ink (user data), not theme colour. */
  .swatch {
    width: 16px;
    height: 16px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    box-shadow: inset 0 0 0 1px var(--line-strong);
    cursor: pointer;
    transition: transform var(--dur-fast) var(--ease);
  }
  .swatch:hover {
    transform: scale(1.12);
  }

  .swatch.active {
    outline: 1px solid var(--ink);
    outline-offset: 2px;
  }

  .color-input {
    width: 22px;
    height: 22px;
    padding: 0;
    overflow: hidden;
    border: 1px solid var(--line-strong);
    border-radius: 50%;
    background: transparent;
    cursor: pointer;
  }
  .color-input::-webkit-color-swatch-wrapper {
    padding: 0;
  }
  .color-input::-webkit-color-swatch {
    border: 0;
    border-radius: 50%;
  }
  .color-input::-moz-color-swatch {
    border: 0;
    border-radius: 50%;
  }

  .slider {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    color: var(--ink-3);
  }

  .slider input {
    width: 88px;
    accent-color: var(--fjord);
  }

  .slider-val {
    min-width: 2ch;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .error {
    flex-shrink: 0;
    padding: var(--s-2) var(--s-5);
    border-bottom: 1px solid var(--line);
    background: var(--surface);
    color: var(--danger);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  /* ── Empty state ─────────────────────────────────────── */

  .empty {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-3);
    padding: var(--s-5);
    color: var(--ink-3);
  }

  .empty-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-3);
    width: 100%;
    max-width: 480px;
    padding: var(--s-8) var(--s-6);
    border: 1px dashed var(--line-strong);
    border-radius: var(--radius-md);
    background: var(--surface);
    text-align: center;
  }

  .empty-card h2 {
    margin: 0;
    color: var(--ink);
    font-size: var(--text-xl);
    letter-spacing: -0.025em;
  }

  .empty-card p:not(.eyebrow) {
    max-width: 36ch;
    margin: 0 0 var(--s-3);
    color: var(--ink-2);
    font-size: 15px;
  }

  .hint {
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  /* ── Document stage ──────────────────────────────────── */

  .stage {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    gap: var(--s-5);
    overflow: auto;
    padding: var(--s-6);
    background: var(--surface-2);
  }

  /* The page is media; the PDF render paints its own paper. */
  .page {
    position: relative;
    flex-shrink: 0;
    background: var(--surface);
    box-shadow: 0 0 0 1px var(--line);
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
    outline: 1px dashed var(--fjord);
    outline-offset: 3px;
  }

  .anno-text {
    min-height: 1em;
    cursor: move;
  }

  .anno-text .te {
    min-width: 8px;
    min-height: 1em;
    outline: none;
    font-family: var(--font-sans);
    line-height: 1.2;
    white-space: pre-wrap;
    word-break: break-word;
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
    top: -11px;
    right: -11px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 1px solid var(--line-strong);
    border-radius: 50%;
    background: var(--surface);
    color: var(--danger);
    font-size: 13px;
    line-height: 1;
    cursor: pointer;
  }
  .del:hover {
    border-color: var(--danger);
  }

  .grip-corner {
    right: -5px;
    bottom: -5px;
    width: 10px;
    height: 10px;
    border: 1px solid var(--surface);
    border-radius: var(--radius-xs);
    background: var(--fjord);
    cursor: nwse-resize;
  }

  .grip-w {
    top: 50%;
    right: -5px;
    width: 8px;
    height: 18px;
    border: 1px solid var(--surface);
    border-radius: var(--radius-xs);
    background: var(--fjord);
    transform: translateY(-50%);
    cursor: ew-resize;
  }

  .capture {
    position: absolute;
    inset: 0;
    z-index: 5;
    touch-action: none;
  }

  @media (max-width: 900px) {
    .file {
      display: none;
    }
  }

  @media (max-width: 640px) {
    .appbar,
    .toolstrip,
    .error {
      padding-left: var(--s-4);
      padding-right: var(--s-4);
    }
    .tb-text {
      display: none;
    }
    .appbar-actions .btn {
      min-width: 32px;
      padding: 0 var(--s-2);
    }
    .stage {
      padding: var(--s-3);
    }
  }

  @media (max-width: 440px) {
    .crumb .back,
    .crumb .back + .sep {
      display: none;
    }
  }
</style>
