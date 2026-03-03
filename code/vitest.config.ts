import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

const projectRoot = path.resolve(__dirname, '..');
const codeRoot = __dirname;

export default defineConfig({
  plugins: [svelte({ hot: false })],
  resolve: {
    conditions: ['browser'],
    alias: {
      '@code': codeRoot,
      '@tauri-apps/api': path.resolve(codeRoot, 'node_modules/@tauri-apps/api'),
      '@tauri-apps/plugin-store': path.resolve(codeRoot, 'node_modules/@tauri-apps/plugin-store'),
      '@tauri-apps/plugin-autostart': path.resolve(codeRoot, 'node_modules/@tauri-apps/plugin-autostart'),
      '@testing-library/svelte': path.resolve(codeRoot, 'node_modules/@testing-library/svelte'),
      '@testing-library/jest-dom': path.resolve(codeRoot, 'node_modules/@testing-library/jest-dom'),
      'vitest-mock-extended': path.resolve(codeRoot, 'node_modules/vitest-mock-extended'),
    },
  },
  server: {
    fs: {
      allow: [projectRoot],
    },
  },
  test: {
    environment: 'jsdom',
    include: ['../tests/unit/**/*.test.ts'],
    globals: false,
  },
});
