// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest'
import { failOnUnhandledRequests, setupApiServer } from './server'

setupApiServer()

describe('API server', () => {
  it('reports a request without a handler', async () => {
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})
    try {
      await fetch('http://localhost/api/unhandled').catch(() => undefined)
      expect(failOnUnhandledRequests).toThrow('GET http://localhost/api/unhandled')
    } finally {
      consoleError.mockRestore()
    }
  })
})
