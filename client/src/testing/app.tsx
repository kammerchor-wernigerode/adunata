import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createMemoryHistory, createRouter, RouterProvider } from '@tanstack/react-router'
import { render } from '@testing-library/react'
import axe from 'axe-core'
import { EnglishLocalization } from '@/l10n/testing'
import { routeTree } from '@/router'

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
}

/** Renders the whole client at a path, in English and against the API mocks of the test. */
export function renderApp(path: string) {
  const router = createRouter({ routeTree, history: createMemoryHistory({ initialEntries: [path] }) })
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const rendered = render(
    <QueryClientProvider client={queryClient}>
      <EnglishLocalization>
        <RouterProvider router={router} />
      </EnglishLocalization>
    </QueryClientProvider>,
  )
  return { ...rendered, router }
}

export async function violations(container: HTMLElement): Promise<string[]> {
  const results = await axe.run(container)
  return results.violations.map(
    (violation) => `${violation.id}: ${violation.help} at ${violation.nodes.map((node) => node.target.join(' ')).join(', ')}`,
  )
}
