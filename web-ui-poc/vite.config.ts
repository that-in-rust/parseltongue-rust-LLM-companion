import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    port: 3000,
    // Fallback proxy in case CORS is not available on pt08
    proxy: {
      '/api': {
        target: 'http://localhost:7777',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
});
