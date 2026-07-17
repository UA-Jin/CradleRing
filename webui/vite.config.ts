import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import vueJsx from '@vitejs/plugin-vue-jsx';
import { vitePluginForArco } from '@arco-plugins/vite-vue';
import Components from 'unplugin-vue-components/vite';
import { ArcoResolver } from 'unplugin-vue-components/resolvers';
import path from 'node:path';

export default defineConfig({
  plugins: [
    vue(),
    vueJsx(),
    vitePluginForArco({ style: 'css' }),
    Components({
      resolvers: [
        ArcoResolver({
          sideEffect: true,
          type: 'import',
        }),
      ],
    }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
  server: {
    host: '0.0.0.0',
    port: 5173,
    proxy: {
      '/api': 'http://127.0.0.1:18800',
      '/ws': {
        target: 'http://127.0.0.1:18800',
        ws: true,
      },
      '/webhook': 'http://127.0.0.1:18800',
    },
  },
  build: {
    outDir: 'dist',
    target: 'es2020',
    chunkSizeWarningLimit: 2000,
    rollupOptions: {
      output: {
        manualChunks: {
          arco: ['@arco-design/web-vue'],
          echarts: ['echarts', 'vue-echarts'],
          vue: ['vue', 'vue-router', 'pinia'],
        },
      },
    },
  },
});
