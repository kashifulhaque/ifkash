<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import * as monaco from 'monaco-editor';
	import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';

	export let value: string = '';
	export let language: string = 'plaintext';
	export let theme: string = 'vs-dark';
	export let onChange: ((value: string) => void) | undefined = undefined;

	let editorContainer: HTMLDivElement;
	let editor: monaco.editor.IStandaloneCodeEditor;

	// Configure Monaco workers
	self.MonacoEnvironment = {
		getWorker(_: any, label: string) {
			return new editorWorker();
		}
	};

	onMount(() => {
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

<div bind:this={editorContainer} class="monaco-editor-container" />

<style>
	.monaco-editor-container {
		width: 100%;
		height: 100%;
		min-height: 400px;
	}
</style>
