import { defineConfig } from 'orval'

export default defineConfig({
  api: {
    input: { target: './openapi.json' },
    output: {
      client: 'react-query',
      httpClient: 'fetch',
      target: './src/api/generated/api.ts',
      schemas: './src/api/generated/model',
      mock: true,
      clean: true,
    },
  },
  zod: {
    input: { target: './openapi.json' },
    output: {
      client: 'zod',
      target: './src/api/generated/api.zod.ts',
    },
  },
})
