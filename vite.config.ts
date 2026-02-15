import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit()
	],
	optimizeDeps: {
		exclude: ['monaco-editor', '@huggingface/transformers']
	},
	ssr: {
		noExternal: [],
		external: ['monaco-editor']
	},
	build: {
		target: 'esnext'
	},
	server: {
		headers: {
			'Cross-Origin-Embedder-Policy': 'require-corp',
			'Cross-Origin-Opener-Policy': 'same-origin',
		}
	}
});
