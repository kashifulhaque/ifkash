<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { browser } from '$app/environment';

	export let value: string = '';
	export let language: string = 'plaintext';
	export let theme: string = 'vs-dark';
	export let onChange: ((value: string) => void) | undefined = undefined;

	let editorContainer: HTMLDivElement;
	let editor: any;
	let monaco: any;
	let loading = true;
	let error: string | null = null;

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
			editor = monaco.editor.create(editorContainer, {
				value,
				language,
				theme,
				automaticLayout: true,
				fontSize: 14,
				lineNumbers: 'on',
				minimap: { enabled: true },
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

			loading = false;
		} catch (err) {
			console.error('Failed to load Monaco Editor:', err);
			error = 'Failed to load editor. Please refresh the page.';
			loading = false;
		}
	});

	onDestroy(() => {
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
			<div class="message">Loading editor...</div>
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
	}

	.monaco-editor-container {
		width: 100%;
		height: 100%;
		min-height: 400px;
	}

	.overlay {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(30, 30, 30, 0.95);
		z-index: 1000;
	}

	.message {
		font-size: 16px;
		color: #888;
	}

	.overlay.error .message {
		color: #f44336;
	}
</style>
