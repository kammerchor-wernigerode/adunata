import { cleanup } from '@testing-library/react'
import type { RequestHandler } from 'msw'
import { setupServer } from 'msw/node'
import { afterAll, afterEach, beforeAll } from 'vitest'
import { cancelRenderedQueries } from './app'

const unhandledRequests: string[] = []

function describeFrame(data: unknown): string {
  const request = (data as { request?: unknown }).request
  return request instanceof Request ? `${request.method} ${request.url}` : String(data)
}

/** Fails with every request that no handler matched since the last call and forgets them. */
export function failOnUnhandledRequests() {
  const requests = unhandledRequests.splice(0)
  if (requests.length > 0) {
    throw new Error(`Requests without a handler:\n${requests.join('\n')}`)
  }
}

/** Starts an API mock for the tests of a file. A test fails if its page made a request that no handler matched. */
export function setupApiServer(...handlers: RequestHandler[]) {
  const server = setupServer(...handlers)
  beforeAll(() =>
    server.listen({
      onUnhandledFrame: ({ frame, defaults }) => {
        unhandledRequests.push(describeFrame(frame.data))
        defaults.error()
      },
    }),
  )
  afterEach(async () => {
    cleanup()
    await cancelRenderedQueries()
    try {
      failOnUnhandledRequests()
    } finally {
      server.resetHandlers()
    }
  })
  afterAll(() => {
    try {
      failOnUnhandledRequests()
    } finally {
      server.close()
    }
  })
  return server
}
