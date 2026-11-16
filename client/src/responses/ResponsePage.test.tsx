// @vitest-environment jsdom
import { cleanup, screen } from '@testing-library/react'
import { http, HttpResponse } from 'msw'
import { setupServer } from 'msw/node'
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest'
import { getLegalLinksMockHandler } from '@/api/generated/api.msw'
import type { InvitationPage } from '@/api/generated/model'
import { renderApp, violations } from '@/testing/app'

const server = setupServer()
const secret = 'f'.repeat(64)

beforeAll(() => server.listen({ onUnhandledFrame: 'error' }))
afterEach(() => {
  server.resetHandlers()
  cleanup()
  window.history.replaceState(null, '', '/')
})
afterAll(() => server.close())

const page: InvitationPage = {
  firstName: 'Clara',
  gathering: {
    title: 'Rehearsal weekend',
    location: 'Drübeck Abbey',
    description: 'We rehearse the April programme.',
    firstDay: '2027-03-12',
    lastDay: '2027-03-14',
  },
  state: 'open',
}

function openLink(answer: (request: Request) => Response) {
  const requests: Request[] = []
  server.use(
    getLegalLinksMockHandler({ privacyUrl: 'https://choir.example.org/privacy' }),
    http.get('*/api/session', () => new HttpResponse(null, { status: 401 })),
    http.get('*/api/invitation', ({ request }) => {
      requests.push(request)
      return answer(request)
    }),
  )
  window.history.replaceState(null, '', `/invitation#${secret}`)
  return { requests, ...renderApp('/invitation') }
}

describe('response page', () => {
  it('sends the secret only in the Authorization header', async () => {
    const { requests, container } = openLink(() => HttpResponse.json(page))
    expect(await screen.findByRole('heading', { level: 1, name: 'Rehearsal weekend' })).toBeTruthy()
    expect(requests).toHaveLength(1)
    expect(requests[0].url).not.toContain(secret)
    expect(requests[0].headers.get('authorization')).toBe(`Bearer ${secret}`)
    expect(screen.getAllByRole('listitem').length).toBeGreaterThanOrEqual(3)
    expect(screen.getByRole('link', { name: 'Privacy notice' })).toBeTruthy()
    expect(await violations(container)).toEqual([])
  })

  it('says that a link is not valid and names nothing else', async () => {
    const { container } = openLink(() => new HttpResponse(null, { status: 404 }))
    expect(await screen.findByRole('heading', { level: 1, name: 'Link not valid' })).toBeTruthy()
    expect(screen.queryByText(/Rehearsal/)).toBeNull()
    expect(await violations(container)).toEqual([])
  })

  it('shows a cancelled gathering without a form', async () => {
    openLink(() => HttpResponse.json({ ...page, state: 'cancelled' }))
    expect(await screen.findByText('This gathering is cancelled.')).toBeTruthy()
    expect(screen.queryByRole('form')).toBeNull()
  })
})
