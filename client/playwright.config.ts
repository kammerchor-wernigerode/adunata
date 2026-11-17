import { existsSync } from 'node:fs'
import path from 'node:path'
import { defineConfig, devices } from '@playwright/test'

const localSettings = path.resolve(import.meta.dirname, '.env.local')
if (existsSync(localSettings)) {
  process.loadEnvFile(localSettings)
}

export default defineConfig({
  testDir: './e2e',
  testMatch: '**/*.e2e.ts',
  reporter: 'list',
  use: {
    baseURL: process.env.ADUNATA_E2E_URL ?? 'http://localhost:8080',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
})
