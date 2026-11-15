import path from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: { '@': path.resolve(import.meta.dirname, './src') },
  },
  server: {
    port: Number(process.env.ADUNATA_VITE_PORT || 5173),
    strictPort: true,
    proxy: {
      '/api': {
        target: `http://${process.env.ADUNATA_LISTEN_ADDRESS || '127.0.0.1:8080'}`,
        headers: { origin: new URL(process.env.ADUNATA_PUBLIC_URL || 'http://localhost:8080').origin },
      },
    },
  },
})
