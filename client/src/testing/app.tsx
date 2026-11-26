import { type QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createMemoryHistory, RouterProvider } from '@tanstack/react-router'
import { render } from '@testing-library/react'
import axe from 'axe-core'
import { onTestFinished, vi } from 'vitest'
import { EnglishLocalization } from '@/l10n/testing'
import { createQueryClient } from '@/lib/queryClient'
import { createAppRouter } from '@/router'

const renderedClients: QueryClient[] = []

/** Renders the whole client at a path, in English and against the API mocks of the test. */
export function renderApp(path: string) {
  const queryClient = createQueryClient({ retry: false })
  renderedClients.push(queryClient)
  const router = createAppRouter(queryClient, createMemoryHistory({ initialEntries: [path] }))
  const rendered = render(
    <QueryClientProvider client={queryClient}>
      <EnglishLocalization>
        <RouterProvider router={router} />
      </EnglishLocalization>
    </QueryClientProvider>,
  )
  return { ...rendered, router }
}

/** Cancels and clears the queries of every client rendered since the last call, so no request outlives its test. */
export async function cancelRenderedQueries() {
  const clients = renderedClients.splice(0)
  await Promise.all(clients.map((client) => client.cancelQueries()))
  clients.forEach((client) => client.clear())
}

function isErrorWithMessage(value: unknown, message: string): boolean {
  return typeof value === 'object' && value !== null && 'message' in value && value.message === message
}

function isRouteMatchWarning(args: unknown[]): boolean {
  return typeof args[0] === 'string' && args[0].startsWith('Warning: Error in route match: ')
}

/**
 * Silences React's report of the error with this message, which the current test provokes on purpose, and the router's warning that
 * directly follows it. Any other console output still shows.
 */
export function silenceProvokedErrors(message: string) {
  const { error, warn } = console
  let reportedProvokedError = false
  const errorSpy = vi.spyOn(console, 'error').mockImplementation((...args: unknown[]) => {
    reportedProvokedError = args.some((arg) => isErrorWithMessage(arg, message))
    if (!reportedProvokedError) error(...args)
  })
  const warnSpy = vi.spyOn(console, 'warn').mockImplementation((...args: unknown[]) => {
    const followsProvokedError = reportedProvokedError && isRouteMatchWarning(args)
    reportedProvokedError = false
    if (!followsProvokedError) warn(...args)
  })
  onTestFinished(() => {
    errorSpy.mockRestore()
    warnSpy.mockRestore()
  })
}

export async function violations(container: HTMLElement): Promise<string[]> {
  const results = await axe.run(container)
  return results.violations.map(
    (violation) => `${violation.id}: ${violation.help} at ${violation.nodes.map((node) => node.target.join(' ')).join(', ')}`,
  )
}
