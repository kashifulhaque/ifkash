import adapter from '@sveltejs/adapter-cloudflare';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),

  kit: {
    adapter: adapter({
      routes: {
        include: ['/*'],
        exclude: ['<all>']
      }
    }),
    prerender: {
      // /api/* is served by the Worker, not by the build, so the prerender crawl cannot
      // fetch it. The root layout links the résumé there, and every prerendered page reaches
      // that link. Any other broken link still fails the build.
      handleHttpError: ({ path, message }) => {
        if (path.startsWith('/api/')) return;
        throw new Error(message);
      }
    }
  }
};

export default config;
