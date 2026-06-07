// Browser-side PDF annotation helpers.
//
// Rendering is done with pdf.js (pdfjs-dist) onto a canvas; annotations live in
// a plain data model (see `Annotation`) captured in *render-canvas pixels* at a
// fixed `scale`. Export burns the annotations into the *original* PDF bytes with
// pdf-lib, translating canvas pixels (top-left origin) to PDF points (bottom-left
// origin). Everything runs client-side, so the route using this must set
// `ssr = false`.

import type * as Pdfjs from 'pdfjs-dist';
import type { PDFDocumentProxy } from 'pdfjs-dist';

// Use the *legacy* build: it ships the polyfills (e.g. Map.getOrInsertComputed)
// pdf.js v6 relies on, so rendering works across all browsers, not just the
// newest engines. The worker is bundled locally (not from a CDN) so it satisfies
// the dev server's COEP `require-corp` policy set in vite.config.ts.
import workerUrl from 'pdfjs-dist/legacy/build/pdf.worker.min.mjs?url';

/** Render scale: 1 canvas pixel maps to (1 / RENDER_SCALE) PDF points. */
export const RENDER_SCALE = 1.5;

export type Point = { x: number; y: number };

type AnnotationBase = {
	id: string;
	/** Zero-based page index. */
	pageIndex: number;
	/** Hex colour, e.g. `#ebe5d2`. */
	color: string;
};

export type DrawAnnotation = AnnotationBase & {
	type: 'draw';
	/** Freehand points in canvas pixels. */
	points: Point[];
	/** Stroke width in canvas pixels. */
	width: number;
};

export type HighlightAnnotation = AnnotationBase & {
	type: 'highlight';
	x: number;
	y: number;
	w: number;
	h: number;
};

export type TextAnnotation = AnnotationBase & {
	type: 'text';
	/** Top-left of the text box in canvas pixels. */
	x: number;
	y: number;
	/** Box width in canvas pixels (used for wrapping on export). */
	w: number;
	text: string;
	/** Font size in canvas pixels. */
	fontSize: number;
};

export type ImageAnnotation = AnnotationBase & {
	type: 'image';
	x: number;
	y: number;
	w: number;
	h: number;
	/** `data:image/png;base64,...` or `data:image/jpeg;base64,...`. */
	dataUrl: string;
};

export type Annotation =
	| DrawAnnotation
	| HighlightAnnotation
	| TextAnnotation
	| ImageAnnotation;

export type ToolKind = 'select' | 'text' | 'draw' | 'highlight' | 'image';

// Cache the pdf.js module so the worker only initialises once per session.
let pdfjsPromise: Promise<typeof Pdfjs> | null = null;

async function getPdfjs() {
	if (!pdfjsPromise) {
		pdfjsPromise = import('pdfjs-dist/legacy/build/pdf.mjs').then((lib) => {
			const m = lib as unknown as typeof Pdfjs;
			m.GlobalWorkerOptions.workerSrc = workerUrl;
			return m;
		});
	}
	return pdfjsPromise;
}

/**
 * Load a PDF document. pdf.js detaches the supplied buffer (transfers it to the
 * worker), so a copy is passed and the caller keeps the original bytes for export.
 */
export async function loadPdfDocument(data: ArrayBuffer): Promise<PDFDocumentProxy> {
	const lib = await getPdfjs();
	return lib.getDocument({ data: data.slice(0) }).promise;
}

/** Render a 1-based page number into `canvas` at `scale`; returns pixel size. */
export async function renderPage(
	doc: PDFDocumentProxy,
	pageNumber: number,
	canvas: HTMLCanvasElement,
	scale: number = RENDER_SCALE
): Promise<{ width: number; height: number }> {
	const page = await doc.getPage(pageNumber);
	const viewport = page.getViewport({ scale });
	const width = Math.floor(viewport.width);
	const height = Math.floor(viewport.height);
	canvas.width = width;
	canvas.height = height;
	await page.render({ canvas, viewport }).promise;
	return { width, height };
}

function hexToRgb01(hex: string): { r: number; g: number; b: number } {
	const clean = hex.replace('#', '');
	const full =
		clean.length === 3
			? clean
					.split('')
					.map((c) => c + c)
					.join('')
			: clean;
	const num = parseInt(full || 'ebe5d2', 16);
	return {
		r: ((num >> 16) & 255) / 255,
		g: ((num >> 8) & 255) / 255,
		b: (num & 255) / 255
	};
}

function dataUrlToBytes(dataUrl: string): Uint8Array {
	const base64 = dataUrl.split(',')[1] ?? '';
	const binary = atob(base64);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return bytes;
}

/**
 * Burn annotations into the original PDF bytes and return the new PDF bytes.
 * `scale` must match the scale the annotations were captured at (`RENDER_SCALE`).
 */
export async function exportAnnotatedPdf(
	originalBytes: Uint8Array,
	annotations: Annotation[],
	scale: number = RENDER_SCALE
): Promise<Uint8Array> {
	const { PDFDocument, StandardFonts, rgb } = await import('pdf-lib');
	const pdfDoc = await PDFDocument.load(originalBytes);
	const font = await pdfDoc.embedFont(StandardFonts.Helvetica);
	const pages = pdfDoc.getPages();

	for (const ann of annotations) {
		const page = pages[ann.pageIndex];
		if (!page) continue;

		const pageHeight = page.getHeight();
		const { r, g, b } = hexToRgb01(ann.color);
		const color = rgb(r, g, b);
		// Canvas px (top-left origin) -> PDF points (bottom-left origin).
		const toPdf = (cx: number, cy: number) => ({ x: cx / scale, y: pageHeight - cy / scale });

		if (ann.type === 'draw') {
			if (ann.points.length === 1) {
				const p = toPdf(ann.points[0].x, ann.points[0].y);
				page.drawCircle({ x: p.x, y: p.y, size: ann.width / scale / 2, color });
			} else {
				for (let i = 1; i < ann.points.length; i++) {
					const start = toPdf(ann.points[i - 1].x, ann.points[i - 1].y);
					const end = toPdf(ann.points[i].x, ann.points[i].y);
					page.drawLine({ start, end, thickness: ann.width / scale, color });
				}
			}
		} else if (ann.type === 'highlight') {
			const bottomLeft = toPdf(ann.x, ann.y + ann.h);
			page.drawRectangle({
				x: bottomLeft.x,
				y: bottomLeft.y,
				width: ann.w / scale,
				height: ann.h / scale,
				color,
				opacity: 0.35,
				borderWidth: 0
			});
		} else if (ann.type === 'text') {
			const fontSizePts = ann.fontSize / scale;
			// Baseline of the first line ~ top + ~0.9em (ascent + half-leading).
			const baselineFromTop = ann.y / scale + fontSizePts * 0.9;
			page.drawText(ann.text, {
				x: ann.x / scale,
				y: pageHeight - baselineFromTop,
				size: fontSizePts,
				font,
				color,
				lineHeight: fontSizePts * 1.2,
				maxWidth: ann.w / scale
			});
		} else if (ann.type === 'image') {
			const bytes = dataUrlToBytes(ann.dataUrl);
			const img = ann.dataUrl.startsWith('data:image/png')
				? await pdfDoc.embedPng(bytes)
				: await pdfDoc.embedJpg(bytes);
			const bottomLeft = toPdf(ann.x, ann.y + ann.h);
			page.drawImage(img, {
				x: bottomLeft.x,
				y: bottomLeft.y,
				width: ann.w / scale,
				height: ann.h / scale
			});
		}
	}

	return pdfDoc.save();
}

/** Trigger a browser download of PDF bytes. */
export function downloadPdf(bytes: Uint8Array, filename: string): void {
	const ab = new ArrayBuffer(bytes.byteLength);
	new Uint8Array(ab).set(bytes);
	const blob = new Blob([ab], { type: 'application/pdf' });
	const url = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = url;
	a.download = filename;
	document.body.appendChild(a);
	a.click();
	a.remove();
	URL.revokeObjectURL(url);
}
