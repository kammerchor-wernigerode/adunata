// @vitest-environment jsdom
import { cleanup, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { setupServer } from 'msw/node'
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest'
import { getLegalLinksMockHandler } from '@/api/generated/api.msw'
import { renderApp, violations } from '@/testing/app'

const server = setupServer()

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
    expect(screen.queryByRole('alert')).toBeNull()
  })
})
