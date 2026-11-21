// @vitest-environment jsdom
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { QueryClientProvider } from '@tanstack/react-query'
import { createMemoryHistory, RouterProvider } from '@tanstack/react-router'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import axe from 'axe-core'
import { http, HttpResponse } from 'msw'
import { afterEach, describe, expect, it } from 'vitest'
import { getCurrentSessionMockHandler, getLegalLinksMockHandler } from '@/api/generated/api.msw'
import type { Member } from '@/api/generated/model'
import { LocalizationProvider } from '@/l10n/LocalizationProvider'
import { createQueryClient } from '@/lib/queryClient'
import { createAppRouter } from '@/router'
import { renderApp } from '@/testing/app'
import { setupApiServer } from '@/testing/server'

const server = setupApiServer()

const ada: Member = {
  id: 1,
  firstName: 'Ada',
  lastName: 'Lovelace',
  mail: 'ada@example.org',
  language: 'en',
  active: true,
  organizer: true,
}

afterEach(() => {
  localStorage.clear()
  server.events.removeAllListeners()
})

function signedIn() {
  server.use(
    getLegalLinksMockHandler({}),
    getCurrentSessionMockHandler(ada),
    http.get('*/api/members', () => HttpResponse.json([ada])),
    http.get('*/api/members/1', () => HttpResponse.json(ada)),
  )
}

async function openMenu() {
  await userEvent.click(await screen.findByRole('button', { name: 'Account of Ada Lovelace' }))
  return screen.findByRole('menu')
}

/** Renders the whole client with the catalogs the server serves, so that switching the language takes effect. */
function renderLocalizedApp(pathname: string) {
  server.use(
    http.get('*/api/catalogs/:locale', ({ params }) =>
      HttpResponse.text(readFileSync(path.join(import.meta.dirname, `../../../server/locales/${String(params.locale)}.ftl`), 'utf8')),
    ),
  )
  const queryClient = createQueryClient({ retry: false })
  const router = createAppRouter(queryClient, createMemoryHistory({ initialEntries: [pathname] }))
  return render(
    <QueryClientProvider client={queryClient}>
      <LocalizationProvider>
        <RouterProvider router={router} />
      </LocalizationProvider>
    </QueryClientProvider>,
  )
}

describe('account menu', () => {
  it('lists the own member entry, the languages and sign-out last', async () => {
    signedIn()
    renderApp('/members')
    const menu = await openMenu()
    const entries = Array.from(menu.querySelectorAll('[role="menuitem"], [role="menuitemradio"], [role="separator"]')).map(
      (entry) => entry.getAttribute('role') === 'separator' ? '---' : entry.textContent,
    )
    expect(entries).toEqual(['My member entry', 'Deutsch', 'English', '---', 'Sign out'])
    const results = await axe.run(document.body)
    expect(results.violations.map((violation) => `${violation.id} at ${violation.nodes.map((node) => node.html).join(', ')}`)).toEqual([])
  })

  it('leads to the own member entry', async () => {
    signedIn()
    const { router } = renderApp('/members')
    await openMenu()
    await userEvent.click(screen.getByRole('menuitem', { name: 'My member entry' }))
    expect(await screen.findByDisplayValue('Lovelace')).toBeTruthy()
    expect(router.state.location.pathname).toBe('/members/1')
    await waitFor(() => expect(document.activeElement?.tagName).toBe('H1'))
  })

  it('signs out with the keyboard alone', async () => {
    let signedOut = false
    signedIn()
    server.use(
      http.get('*/api/session', () => (signedOut ? new HttpResponse(null, { status: 401 }) : HttpResponse.json(ada))),
      http.delete('*/api/session', () => {
        signedOut = true
        return new HttpResponse(null, { status: 204 })
      }),
    )
    const { router } = renderApp('/members')
    ;(await screen.findByRole('button', { name: 'Account of Ada Lovelace' })).focus()
    await userEvent.keyboard('{Enter}')
    await screen.findByRole('menu')
    await userEvent.keyboard('{ArrowDown}{ArrowDown}{ArrowDown}')
    expect(document.activeElement?.textContent).toBe('Sign out')
    await userEvent.keyboard('{Enter}')
    await screen.findByRole('heading', { level: 1, name: 'Sign in' })
    expect(signedOut).toBe(true)
    expect(router.state.location.pathname).toBe('/sign-in')
  })

  it('names a failed sign-out next to the trigger after the menu closes', async () => {
    signedIn()
    server.use(http.delete('*/api/session', () => new HttpResponse(null, { status: 500 })))
    const { router } = renderApp('/members')
    await openMenu()
    await userEvent.click(screen.getByRole('menuitem', { name: 'Sign out' }))
    await waitFor(() => expect(screen.getByRole('alert').textContent).toBe('Something went wrong. Try again.'))
    expect(screen.queryByRole('menu')).toBeNull()
    expect(router.state.location.pathname).toBe('/members')
  })

  it('is missing on the sign-in page, which keeps the language select', async () => {
    server.use(getLegalLinksMockHandler({}), http.get('*/api/session', () => new HttpResponse(null, { status: 401 })))
    renderApp('/sign-in')
    expect(await screen.findByLabelText('Language')).toBeTruthy()
    expect(screen.queryByRole('button', { name: /^Account of/ })).toBeNull()
    expect(screen.queryByRole('button', { name: 'Sign out' })).toBeNull()
  })

  it('switches the client to German without changing the stored language of mail', async () => {
    const changes: string[] = []
    server.events.on('request:start', ({ request }) => {
      if (new URL(request.url).pathname.startsWith('/api/members') && request.method !== 'GET') changes.push(request.method)
    })
    signedIn()
    const first = renderLocalizedApp('/members')
    await openMenu()
    expect(screen.queryByLabelText('Language')).toBeNull()
    await userEvent.click(screen.getByRole('menuitemradio', { name: 'Deutsch' }))
    expect(await screen.findByRole('heading', { level: 1, name: 'Mitglieder' })).toBeTruthy()
    first.unmount()

    renderLocalizedApp('/members')
    await userEvent.click(await screen.findByRole('button', { name: 'Konto von Ada Lovelace' }))
    expect(screen.getByRole('menuitemradio', { name: 'Deutsch' }).getAttribute('aria-checked')).toBe('true')
    expect(screen.queryByLabelText('Sprache')).toBeNull()
    expect(changes).toEqual([])
  })
})
