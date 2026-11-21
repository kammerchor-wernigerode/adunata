import { type QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createMemoryHistory, RouterProvider } from '@tanstack/react-router'
import { render } from '@testing-library/react'
import axe from 'axe-core'
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

export async function violations(container: HTMLElement): Promise<string[]> {
  const results = await axe.run(container)
  return results.violations.map(
    (violation) => `${violation.id}: ${violation.help} at ${violation.nodes.map((node) => node.target.join(' ')).join(', ')}`,
  )
}
