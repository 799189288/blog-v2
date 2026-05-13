import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  server: {
    port: 5178,
    host: '0.0.0.0',
    proxy: {
      '/api': {
        target: 'http://localhost:8088',
        changeOrigin: true,
      },
      // Uploaded images live on the backend.
      '/uploads': {
        target: 'http://localhost:8088',
        changeOrigin: true,
      },
    },
  },
})
