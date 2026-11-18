// @vitest-environment jsdom
import { act, cleanup, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { setupServer } from 'msw/node'
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest'
import { getCurrentSessionMockHandler, getLegalLinksMockHandler } from '@/api/generated/api.msw'
import type { Member } from '@/api/generated/model'
import { renderApp, violations } from '@/testing/app'

const server = setupServer()

const ada: Member = {
  id: 1,
  firstName: 'Ada',
  lastName: 'Lovelace',
  mail: 'ada@example.org',
  language: 'en',
  active: true,
  organizer: true,
}

beforeAll(() => server.listen({ onUnhandledFrame: 'error' }))
afterEach(() => {
  server.resetHandlers()
  cleanup()
  window.history.replaceState(null, '', '/')
})
afterAll(() => server.close())

function signedOut() {
  server.use(
    getLegalLinksMockHandler({}),
    http.get('*/api/session', () => new HttpResponse(null, { status: 401 })),
  )
}

describe('sign-in page', () => {
  it('sends a link and confirms without naming the account', async () => {
    signedOut()
    let requested = ''
    server.use(
      http.post('*/api/sign-in-requests', async ({ request }) => {
        requested = ((await request.json()) as { mail: string }).mail
        return new HttpResponse(null, { status: 202 })
      }),
    )
    const { container } = renderApp('/sign-in')
    await userEvent.type(await screen.findByLabelText('Mail address'), 'ada@example.org')
    await userEvent.click(screen.getByRole('button', { name: 'Send sign-in link' }))
    expect((await screen.findByRole('status')).textContent).toContain('If this address belongs to an organizer')
    expect(requested).toBe('ada@example.org')
    expect(await violations(container)).toEqual([])
  })

  it('offers a new link when the link was used or has expired', async () => {
    signedOut()
    let authorization = ''
    server.use(
      http.post('*/api/sessions', ({ request }) => {
        authorization = request.headers.get('authorization') ?? ''
        return new HttpResponse(null, { status: 401 })
      }),
    )
    window.history.replaceState(null, '', '/sign-in#abc123')
    const { container } = renderApp('/sign-in')
    expect(await violations(container)).toEqual([])
    await userEvent.click(await screen.findByRole('button', { name: 'Sign in' }))
    expect((await screen.findByRole('alert')).textContent).toBe('This link was already used or has expired.')
    expect(authorization).toBe('Bearer abc123')
    await userEvent.click(screen.getByRole('button', { name: 'Request a new link' }))
    expect(await screen.findByLabelText('Mail address')).toBeTruthy()
  })

  it('sends a visitor without a session from an organizer page to the sign-in page', async () => {
    signedOut()
    const { router } = renderApp('/members')
    await screen.findByRole('heading', { level: 1, name: 'Sign in' })
    expect(router.state.location.pathname).toBe('/sign-in')
  })

  it('sends an organizer whose session ended while a page was open to the sign-in page', async () => {
    let expired = false
    server.use(
      getLegalLinksMockHandler({}),
      getCurrentSessionMockHandler(ada),
      http.get('*/api/members', () => (expired ? new HttpResponse(null, { status: 401 }) : HttpResponse.json([ada]))),
    )
    const { router } = renderApp('/members')
    await screen.findByRole('link', { name: 'Lovelace, Ada' })
    expired = true
    await act(() => router.options.context.queryClient.refetchQueries())
    await screen.findByRole('heading', { level: 1, name: 'Sign in' })
    expect(router.state.location.pathname).toBe('/sign-in')
  })

  it('hides the organizer navigation after signing out', async () => {
    let signedIn = true
    server.use(
      getLegalLinksMockHandler({}),
      http.get('*/api/session', () => (signedIn ? HttpResponse.json(ada) : new HttpResponse(null, { status: 401 }))),
      http.delete('*/api/session', () => {
        signedIn = false
        return new HttpResponse(null, { status: 204 })
      }),
      http.get('*/api/members', () => HttpResponse.json([ada])),
    )
    const { router } = renderApp('/members')
    await userEvent.click(await screen.findByRole('button', { name: 'Sign out' }))
    await screen.findByRole('heading', { level: 1, name: 'Sign in' })
    expect(router.state.location.pathname).toBe('/sign-in')
    await waitFor(() => expect(screen.queryByRole('navigation', { name: 'Main navigation' })).toBeNull())
  })

  it('signs in with a link and shows the organizer navigation', async () => {
    let signedIn = false
    server.use(
      getLegalLinksMockHandler({}),
      http.get('*/api/session', () => (signedIn ? HttpResponse.json(ada) : new HttpResponse(null, { status: 401 }))),
      http.post('*/api/sessions', () => {
        signedIn = true
        return HttpResponse.json(ada, { status: 201 })
      }),
      http.get('*/api/gatherings', () => HttpResponse.json({ upcoming: [], past: [] })),
    )
    window.location.hash = 'abc123'
    const { router } = renderApp('/sign-in')
    await userEvent.click(await screen.findByRole('button', { name: 'Sign in' }))
    await screen.findByRole('heading', { level: 1, name: 'Gatherings' })
    expect(router.state.location.pathname).toBe('/gatherings')
    expect(await screen.findByRole('navigation', { name: 'Main navigation' })).toBeTruthy()
  })

  it('names an unreachable server and offers to try again', async () => {
    server.use(getLegalLinksMockHandler({}), http.get('*/api/session', () => HttpResponse.error()))
    renderApp('/members')
    expect(await screen.findByRole('heading', { level: 1, name: 'Page unavailable' })).toBeTruthy()
    expect(screen.getByRole('alert').textContent).toBe('Something went wrong. Try again.')
    expect(screen.getByRole('button', { name: 'Try again' })).toBeTruthy()
  })

  it('follows a sign-in link opened while the page is open', async () => {
    signedOut()
    renderApp('/sign-in')
    expect(await screen.findByLabelText('Mail address')).toBeTruthy()
    window.location.hash = 'abc123'
    window.dispatchEvent(new HashChangeEvent('hashchange'))
    expect(await screen.findByRole('button', { name: 'Sign in' })).toBeTruthy()
  })

  it('forgets an invalid link when a new one opens in the same tab', async () => {
    signedOut()
    server.use(http.post('*/api/sessions', () => new HttpResponse(null, { status: 401 })))
    window.history.replaceState(null, '', '/sign-in#old')
    renderApp('/sign-in')
    await userEvent.click(await screen.findByRole('button', { name: 'Sign in' }))
    expect(await screen.findByRole('alert')).toBeTruthy()
    window.location.hash = 'new'
    window.dispatchEvent(new HashChangeEvent('hashchange'))
    expect(await screen.findByRole('button', { name: 'Sign in' })).toBeTruthy()
    expect(screen.getByRole('alert').textContent).toBe('')
  })
})
