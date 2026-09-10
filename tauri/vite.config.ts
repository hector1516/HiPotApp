import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'node:path';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    proxy: { '/api': 'http://localhost:8000' },
    watch: { ignored: ['**/src-tauri/target/**'] }
  },
  build: {
    target: 'esnext',
    rollupOptions: {
      input: {
        main: resolve(root, 'index.html'),
        splash: resolve(root, 'splash.html'),
      },
    },
  },
});
