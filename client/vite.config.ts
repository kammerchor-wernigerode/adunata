import { readFileSync } from 'node:fs'
import path from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig, type Plugin } from 'vite'

const fonts = ['literata', 'source-sans-3']

/** Ships the copyright notice and the SIL Open Font License of each font next to the font files. */
function fontLicenses(): Plugin {
  return {
    name: 'font-licenses',
    generateBundle() {
      for (const font of fonts) {
        this.emitFile({
          type: 'asset',
          fileName: `assets/LICENSE-${font}.txt`,
          source: readFileSync(path.resolve(import.meta.dirname, `node_modules/@fontsource-variable/${font}/LICENSE`)),
        })
      }
    },
  }
}

export default defineConfig({
  plugins: [react(), tailwindcss(), fontLicenses()],
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
