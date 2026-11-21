// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  RouterProvider,
} from '@tanstack/react-router'
import { act, render, screen, waitFor } from '@testing-library/react'
import axe from 'axe-core'
import { http, HttpResponse } from 'msw'
import { describe, expect, it } from 'vitest'
import { getLegalLinksMockHandler } from '@/api/generated/api.msw'
import { EnglishLocalization } from '@/l10n/testing'
import { PageTitle } from '@/components/PageTitle'
import { RootLayout } from '@/components/RootLayout'
import { setupApiServer } from '@/testing/server'

const server = setupApiServer()

function renderLayout() {
  server.use(http.get('*/api/session', () => new HttpResponse(null, { status: 401 })))
  const root = createRootRoute({ component: RootLayout })
  const first = createRoute({
    getParentRoute: () => root,
    path: '/',
    component: () => <PageTitle title="Gatherings" />,
  })
  const second = createRoute({
    getParentRoute: () => root,
    path: '/members',
    component: () => <PageTitle title="Members" />,
  })
  const history = createMemoryHistory({ initialEntries: ['/'] })
  const router = createRouter({ routeTree: root.addChildren([first, second]), history })
  const rendered = render(
    <QueryClientProvider client={new QueryClient()}>
      <EnglishLocalization>
        <RouterProvider router={router} />
      </EnglishLocalization>
    </QueryClientProvider>,
  )
  return { ...rendered, history }
}

describe('root layout', () => {
  it('links privacy notice and imprint when both addresses are set', async () => {
    server.use(
      getLegalLinksMockHandler({
        privacyUrl: 'https://choir.example.org/privacy',
        imprintUrl: 'https://choir.example.org/imprint',
      }),
    )
    const { container } = renderLayout()
    expect((await screen.findByRole('link', { name: 'Privacy notice' })).getAttribute('href')).toBe(
      'https://choir.example.org/privacy',
    )
    expect(screen.getByRole('link', { name: 'Imprint' }).getAttribute('href')).toBe('https://choir.example.org/imprint')
    const results = await axe.run(container)
    expect(results.violations.map((violation) => violation.id)).toEqual([])
  })

  it('shows no such links when no address is set', async () => {
    server.use(getLegalLinksMockHandler({}))
    renderLayout()
    await screen.findByRole('heading', { level: 1, name: 'Gatherings' })
    await waitFor(() => expect(screen.queryByRole('contentinfo')).toBeNull())
    expect(screen.queryByRole('link', { name: 'Privacy notice' })).toBeNull()
  })

  it('names the tab and moves focus to the main heading after a route change', async () => {
    server.use(getLegalLinksMockHandler({}))
    const { history } = renderLayout()
    await screen.findByRole('heading', { level: 1, name: 'Gatherings' })
    expect(document.title).toBe('Gatherings · Adunata')
    act(() => history.push('/members'))
    const heading = await screen.findByRole('heading', { level: 1, name: 'Members' })
    await waitFor(() => expect(document.activeElement).toBe(heading))
    expect(document.title).toBe('Members · Adunata')
  })
})
