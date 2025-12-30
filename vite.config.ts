import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit()
	],
	optimizeDeps: {
		exclude: ['monaco-editor']
	},
	ssr: {
		noExternal: [],
		external: ['monaco-editor']
	},
	build: {
		target: 'esnext'
	}
});
