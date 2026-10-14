<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { browser } from '$app/environment';
	import LoadingState from '$lib/components/LoadingState.svelte';
	import { theme as siteTheme } from '$lib/stores/theme';

	export let value: string = '';
	export let language: string = 'plaintext';
	/** 'auto' follows the site theme (Nordisk daylight / polar night).
	 *  Any other value is passed to Monaco as a theme name. */
	export let theme: string = 'auto';
	export let onChange: ((value: string) => void) | undefined = undefined;

	let editorContainer: HTMLDivElement;
	let editor: any;
	let monaco: any;
	let loading = true;
	let error: string | null = null;
	let unsubscribeTheme: (() => void) | undefined;

	// Read a token from the live CSS so the editor matches app.css in
	// both themes. Monaco only accepts hex, so drop anything else.
	function token(name: string): string | undefined {
		const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
		return /^#[0-9a-f]{6}$/i.test(v) ? v : undefined;
	}

	function defineNordiskTheme(mode: 'light' | 'dark'): string {
		const name = `nordisk-${mode}`;
		const t = (n: string) => token(n);
		const bare = (n: string) => t(n)?.slice(1);
		const alpha = (n: string, a: string) => (t(n) ? t(n) + a : undefined);
		const colors: Record<string, string | undefined> = {
			'editor.background': t('--surface'),
			'editor.foreground': t('--ink'),
			'editorLineNumber.foreground': t('--ink-3'),
			'editorLineNumber.activeForeground': t('--ink'),
			'editor.lineHighlightBackground': alpha('--surface-2', '80'),
			'editor.lineHighlightBorder': alpha('--surface-2', '00'),
			'editor.selectionBackground': alpha('--fjord', '33'),
			'editor.inactiveSelectionBackground': alpha('--fjord', '1f'),
			'editor.selectionHighlightBackground': alpha('--fjord', '1a'),
			'editor.findMatchBackground': alpha('--clay', '40'),
			'editor.findMatchHighlightBackground': alpha('--clay', '26'),
			'editorCursor.foreground': t('--fjord'),
			'editorWhitespace.foreground': t('--line'),
			'editorIndentGuide.background1': t('--line'),
			'editorIndentGuide.activeBackground1': t('--line-strong'),
			'editorGutter.background': t('--surface'),
			'editorWidget.background': t('--surface'),
			'editorWidget.border': t('--line'),
			'editorHoverWidget.background': t('--surface'),
			'editorHoverWidget.border': t('--line'),
			'input.background': t('--surface'),
			'input.border': t('--line-strong'),
			'focusBorder': t('--fjord'),
			'scrollbar.shadow': alpha('--ink', '00'),
			'scrollbarSlider.background': alpha('--line-strong', '80'),
			'scrollbarSlider.hoverBackground': t('--line-strong'),
			'scrollbarSlider.activeBackground': t('--ink-3'),
			'minimap.background': t('--surface'),
			'minimapSlider.background': alpha('--line-strong', '40'),
			'editorOverviewRuler.border': alpha('--line', '00'),
		};
		monaco.editor.defineTheme(name, {
			base: mode === 'dark' ? 'vs-dark' : 'vs',
			inherit: true,
			rules: [
				{ token: '', foreground: bare('--ink') },
				{ token: 'comment', foreground: bare('--ink-3'), fontStyle: 'italic' },
				{ token: 'keyword', foreground: bare('--fjord') },
				{ token: 'string', foreground: bare('--moss') },
				{ token: 'number', foreground: bare('--clay') },
			].filter((r) => r.foreground),
			colors: Object.fromEntries(
				Object.entries(colors).filter((e): e is [string, string] => !!e[1])
			),
		});
		return name;
	}

	function resolveTheme(mode: 'light' | 'dark'): string {
		return theme === 'auto' ? defineNordiskTheme(mode) : theme;
	}

	onMount(async () => {
		if (!browser) return;

		try {
			// Dynamically import Monaco Editor only on the client side
			const monacoModule = await import('monaco-editor');
			monaco = monacoModule;

			// Dynamically import the editor worker
			const editorWorkerModule = await import('monaco-editor/esm/vs/editor/editor.worker?worker');
			const EditorWorker = editorWorkerModule.default;

			// Configure Monaco workers
			self.MonacoEnvironment = {
				getWorker(_: any, label: string) {
					return new EditorWorker();
				}
			};

			// Create editor
			let mode: 'light' | 'dark' = 'light';
			siteTheme.subscribe((m) => (mode = m))();
			editor = monaco.editor.create(editorContainer, {
				value,
				language,
				theme: resolveTheme(mode),
				automaticLayout: true,
				fontFamily: '"JetBrains Mono Variable", "JetBrains Mono", ui-monospace, "SF Mono", Menlo, monospace',
				fontSize: 14,
				lineHeight: 22,
				lineNumbers: 'on',
				minimap: { enabled: true, renderCharacters: false },
				padding: { top: 16, bottom: 16 },
				renderLineHighlight: 'all',
				overviewRulerBorder: false,
				hideCursorInOverviewRuler: true,
				scrollbar: { verticalScrollbarSize: 10, horizontalScrollbarSize: 10, useShadows: false },
				scrollBeyondLastLine: false,
				wordWrap: 'on',
				tabSize: 2,
			});

			// Listen for changes
			editor.onDidChangeModelContent(() => {
				const newValue = editor.getValue();
				value = newValue;
				if (onChange) {
					onChange(newValue);
				}
			});

			// Re-theme when the site switches between daylight and polar night.
			// theme.ts subscribed first, so <html data-theme> is already updated.
			unsubscribeTheme = siteTheme.subscribe((m) => {
				if (monaco) monaco.editor.setTheme(resolveTheme(m));
			});

			loading = false;
		} catch (err) {
			console.error('Failed to load Monaco Editor:', err);
			error = 'Failed to load editor. Please refresh the page.';
			loading = false;
		}
	});

	onDestroy(() => {
		unsubscribeTheme?.();
		if (editor) {
			editor.dispose();
		}
	});

	// Update editor when value changes externally
	$: if (editor && value !== editor.getValue()) {
		editor.setValue(value);
	}
</script>

<div class="monaco-wrapper">
	<div bind:this={editorContainer} class="monaco-editor-container" />
	
	{#if loading}
		<div class="overlay loading">
			<LoadingState label="Loading editor" />
		</div>
	{:else if error}
		<div class="overlay error">
			<div class="message">{error}</div>
		</div>
	{/if}
</div>

<style>
	.monaco-wrapper {
		position: relative;
		width: 100%;
		height: 100%;
		min-height: 400px;
		background: var(--surface);
	}

	.monaco-editor-container {
		width: 100%;
		height: 100%;
		min-height: 400px;
	}

	.overlay {
		position: absolute;
		inset: 0;
		z-index: 10;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--s-5);
		background: var(--surface);
		color: var(--ink-3);
	}

	.message {
		max-width: 40ch;
		color: var(--ink-3);
		font-size: var(--text-sm);
		text-align: center;
	}

	.overlay.error .message {
		color: var(--danger);
	}
</style>
