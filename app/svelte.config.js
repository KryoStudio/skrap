// A desktop app: no server, so SvelteKit builds a single-page app that Tauri serves.
// See https://v2.tauri.app/start/frontend/sveltekit/
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({ fallback: 'index.html' }),
  },
};

export default config;
