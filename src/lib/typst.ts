// Browser-side Typst compilation via typst.ts (WASM). Replaces the old
// self-hosted typst.ifkash.dev compile server: the editor now renders the
// resume entirely client-side, so there is nothing to keep running.
//
// PDF output only needs the web *compiler* wasm (the renderer wasm is only for
// SVG/canvas), which is loaded from jsDelivr pinned to the installed version.

const TYPST_TS_VERSION = '0.7.0';
const WASM_COMPILER_URL = `https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler@${TYPST_TS_VERSION}/pkg/typst_ts_web_compiler_bg.wasm`;

// Self-hosted fonts the résumé uses (served from /static/fonts).
const FONT_URLS = [
	'/fonts/CrimsonPro.ttf',
	'/fonts/CrimsonPro-Italic.ttf',
	'/fonts/Cardo-Regular.ttf',
	'/fonts/Cardo-Bold.ttf',
	'/fonts/Cardo-Italic.ttf'
];

// Cache the singleton compiler init so fonts/wasm load only once per session.
let initPromise: Promise<unknown> | null = null;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let typstSnippet: any = null;

async function getTypst() {
	// Dynamic import keeps the WASM + library out of SSR and the main bundle.
	const { $typst } = await import('@myriaddreamin/typst.ts/contrib/snippet');

	if (!initPromise) {
		const { preloadRemoteFonts } = await import('@myriaddreamin/typst.ts/options.init');
		$typst.setCompilerInitOptions({
			getModule: () => WASM_COMPILER_URL,
			beforeBuild: [preloadRemoteFonts(FONT_URLS)]
		});
		typstSnippet = $typst;
		initPromise = $typst.getCompiler();
	}

	await initPromise;
	return typstSnippet;
}

/** Compile Typst source to PDF bytes in the browser. */
export async function compileTypstToPdf(source: string): Promise<Uint8Array<ArrayBuffer>> {
	const $typst = await getTypst();
	const pdf = await $typst.pdf({ mainContent: source });
	if (!pdf) {
		throw new Error('Typst compilation produced no output');
	}
	// Copy into a plain ArrayBuffer-backed array so it satisfies `BlobPart`
	// (typst.ts may hand back a SharedArrayBuffer-backed view).
	const bytes = new Uint8Array(pdf.byteLength);
	bytes.set(pdf);
	return bytes;
}
