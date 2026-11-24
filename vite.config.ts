// Configuração do Vite (multi-página) e do Vitest (cobertura 100%).
import vue from '@vitejs/plugin-vue';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [vue()],
  // Caminhos relativos: o Tauri serve o dist por protocolo próprio.
  base: './',
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    target: 'esnext', // WebView2 é moderno: sem polyfills
    minify: 'esbuild',
    sourcemap: false,
    // Uma entrada por janela do app.
    rollupOptions: {
      input: { index: 'index.html', webcam: 'webcam.html', region: 'region.html' },
    },
  },
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      include: ['src/**/*.{ts,vue}'],
      exclude: ['src/**/*.test.ts', 'src/**/*.d.ts'],
      reporter: ['text', 'text-summary'],
      thresholds: { lines: 100, branches: 100, functions: 100, statements: 100 },
    },
  },
});
