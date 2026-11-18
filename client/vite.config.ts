import { existsSync, readFileSync } from 'node:fs'
import path from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig, loadEnv, type Plugin } from 'vite'
import notices from './notices.json' with { type: 'json' }

const fonts = ['literata', 'source-sans-3']
const bundledLicensesFile = '.vite/licenses.json'
const noticesFile = 'licenses.txt'

interface Notice {
  name: string
  version: string
  identifier?: string
  text?: string
}

function fromClient(file: string): string {
  return path.resolve(import.meta.dirname, file)
}

function readOrFail(file: string, reason: string): string {
  if (!existsSync(fromClient(file))) {
    throw new Error(`${file} does not exist, but ${reason}`)
  }
  return readFileSync(fromClient(file), 'utf-8').trim()
}

function installedPackage(name: string): Notice {
  const directory = `node_modules/${name}`
  const manifest = JSON.parse(readOrFail(`${directory}/package.json`, `client/notices.json names ${name}`))
  return {
    name,
    version: manifest.version,
    identifier: manifest.license,
    text: readOrFail(`${directory}/LICENSE`, `${name} reaches the browser through the stylesheet`),
  }
}

function withSuppliedText(notice: Notice): Notice {
  if (notice.text) return notice
  const supplied = (notices.licenseTexts as Record<string, string>)[notice.name]
  if (!supplied) {
    throw new Error(`${notice.name} ${notice.version} ships no license file. Add a text for it to licenseTexts in client/notices.json.`)
  }
  return { ...notice, text: readOrFail(supplied, `client/notices.json names it for ${notice.name}`) }
}

function copiedCode(): string[] {
  return notices.copied.map((copy) => {
    readOrFail(copy.text, `client/notices.json names it for ${copy.path}`)
    if (!existsSync(fromClient(copy.path))) {
      throw new Error(`client/notices.json names the copied code ${copy.path}, which does not exist`)
    }
    return [
      `${copy.name} ${copy.version} (${copy.license}), copied into client/${copy.path}`,
      `Source: ${copy.source}`,
      '',
      readFileSync(fromClient(copy.text), 'utf-8').trim(),
    ].join('\n')
  })
}

function section(notice: Notice): string {
  return `${notice.name} ${notice.version} (${notice.identifier ?? 'no identifier'})\n\n${notice.text}`
}

/** Joins the licenses of the bundled packages with the notices of client/notices.json into licenses.txt. */
function thirdPartyNotices(): Plugin {
  return {
    name: 'third-party-notices',
    generateBundle: {
      order: 'post',
      handler(_, bundle) {
        const bundled = bundle[bundledLicensesFile]
        if (bundled?.type !== 'asset') {
          throw new Error(`Vite wrote no ${bundledLicensesFile}`)
        }
        delete bundle[bundledLicensesFile]
        const packages = (JSON.parse(String(bundled.source)) as Notice[]).map(withSuppliedText)
        const stylesheetPackages = notices.stylesheetPackages.map(installedPackage)
        const sections = [...[...packages, ...stylesheetPackages].map(section), ...copiedCode()]
        this.emitFile({
          type: 'asset',
          fileName: noticesFile,
          source: `Third-party notices of the Adunata client\n\n${sections.join(`\n\n${'='.repeat(72)}\n\n`)}\n`,
        })
      },
    },
  }
}

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

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, import.meta.dirname, '')
  return {
    plugins: [react(), tailwindcss(), fontLicenses(), thirdPartyNotices()],
    build: { license: { fileName: bundledLicensesFile } },
    resolve: {
      alias: { '@': path.resolve(import.meta.dirname, './src') },
    },
    server: {
      port: Number(env.ADUNATA_VITE_PORT || 5173),
      strictPort: true,
      proxy: {
        '/api': {
          target: `http://${env.ADNT_HTTP_LISTEN_ADDRESS || '127.0.0.1:8080'}`,
          headers: { origin: new URL(env.ADNT_HTTP_PUBLIC_URL || 'http://localhost:8080').origin },
        },
      },
    },
  }
})
