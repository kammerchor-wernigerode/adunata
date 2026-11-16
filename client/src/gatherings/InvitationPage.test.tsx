// @vitest-environment jsdom
import { cleanup, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { setupServer } from 'msw/node'
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest'
import {
  getCurrentSessionMockHandler,
  getGetGatheringMockHandler,
  getLegalLinksMockHandler,
  getOverviewMockHandler,
} from '@/api/generated/api.msw'
import type { OverviewRow } from '@/api/generated/model'
import { renderApp, violations } from '@/testing/app'

const server = setupServer()

beforeAll(() => server.listen({ onUnhandledFrame: 'error' }))
afterEach(() => {
  server.resetHandlers()
  cleanup()
})
afterAll(() => server.close())

const sophie: OverviewRow = {
  invitationId: 40,
  firstName: 'Sophie',
  lastName: 'Graf',
  status: 'pending',
  arrival: null,
  departure: null,
  beds: null,
  bedCount: null,
  comment: null,
  changedBy: null,
  changedAt: null,
  outsideSpan: false,
  linkActive: true,
  mail: 'failed',
}

function invitationPage() {
  server.use(
    getLegalLinksMockHandler({}),
    getCurrentSessionMockHandler({ id: 9, firstName: 'Ada', lastName: 'Lovelace', mail: 'ada@example.org', language: 'en', active: true, organizer: true }),
    getGetGatheringMockHandler({
      id: 5,
      title: 'Rehearsal weekend',
      location: 'Drübeck Abbey',
      description: null,
      firstDay: '2027-03-12',
      lastDay: '2027-03-14',
      cancelled: false,
      ended: false,
    }),
    getOverviewMockHandler({
      rows: [sophie],
      totals: { accepted: 0, tentative: 0, declined: 0, pending: 1, bedsNeeded: 0, bedsOffered: 0 },
    }),
  )
  return renderApp('/gatherings/5/invitations/40')
}

describe('invitation page for organizers', () => {
  it('enters a response for the member', async () => {
    let sent: unknown = null
    server.use(
      http.put('*/api/invitations/40/response', async ({ request }) => {
        const body = (await request.json()) as Record<string, unknown>
        sent = body
        return HttpResponse.json(body)
      }),
    )
    const { container } = invitationPage()
    expect(await screen.findByRole('heading', { level: 1, name: 'Invitation of Sophie Graf' })).toBeTruthy()
    await userEvent.click(screen.getByRole('radio', { name: 'No' }))
    await userEvent.click(screen.getByRole('button', { name: 'Save response' }))
    expect(await screen.findByText('Response of Sophie Graf saved.')).toBeTruthy()
    expect(sent).toEqual({ status: 'declined', arrival: null, departure: null, beds: null, bedCount: null, comment: null })
    expect(await violations(container)).toEqual([])
  })

  it('resends and revokes the personal link', async () => {
    const calls: string[] = []
    server.use(
      http.post('*/api/invitations/40/resend', () => {
        calls.push('resend')
        return new HttpResponse(null, { status: 204 })
      }),
      http.post('*/api/invitations/40/revocation', () => {
        calls.push('revoke')
        return new HttpResponse(null, { status: 204 })
      }),
    )
    invitationPage()
    expect(await screen.findByText(/mail failed/)).toBeTruthy()
    await userEvent.click(screen.getByRole('button', { name: 'Resend' }))
    expect(await screen.findByText('New link sent to Sophie.')).toBeTruthy()
    await userEvent.click(screen.getByRole('button', { name: 'Revoke link' }))
    await waitFor(() => expect(calls).toEqual(['resend', 'revoke']))
  })
})
