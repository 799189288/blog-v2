import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  server: {
    host: '0.0.0.0',
    port: 5180,
    proxy: {
      '/api': {
        target: 'http://localhost:8088',
        changeOrigin: true,
      },
      // Uploaded images live on the backend; proxy so the editor can
      // embed them with a relative `/uploads/...` URL and the SPA still
      // resolves them through this dev server.
      '/uploads': {
        target: 'http://localhost:8088',
        changeOrigin: true,
      },
    },
  },
})
