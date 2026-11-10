import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
// @ts-expect-error no @types/node here
import process from 'node:process';

const host = process.env.TAURI_DEV_HOST;

// https://v2.tauri.app/start/frontend/sveltekit/
export default defineConfig(() => ({
  plugins: [sveltekit()],
  // Keep Rust errors on screen.
  clearScreen: false,
  server: {
    // Tauri expects this exact port (1420 is Krak's).
    port: 1430,
    strictPort: true,
    host: host || '127.0.0.1',
    hmr: host ? { protocol: 'ws', host, port: 1431 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },
}));
