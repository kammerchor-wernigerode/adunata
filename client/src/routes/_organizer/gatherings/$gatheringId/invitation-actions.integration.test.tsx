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
  getListInvitationsMockHandler,
  getListMembersMockHandler,
  getOverviewMockHandler,
} from '@/api/generated/api.msw'
import type { Gathering, Invitation, Member } from '@/api/generated/model'
import { renderApp, violations } from '@/testing/app'

const server = setupServer()

beforeAll(() => server.listen({ onUnhandledFrame: 'error' }))
afterEach(() => {
  server.resetHandlers()
  cleanup()
})
afterAll(() => server.close())

const ada: Member = { id: 1, firstName: 'Ada', lastName: 'Lovelace', mail: 'ada@example.org', language: 'en', active: true, organizer: true }
const clara: Member = { ...ada, id: 2, firstName: 'Clara', lastName: 'Schumann', mail: 'clara@example.org', organizer: false }
const johannes: Member = { ...ada, id: 3, firstName: 'Johannes', lastName: 'Brahms', mail: 'jb@example.org', organizer: false }
const weekend: Gathering = {
  id: 5,
  title: 'Rehearsal weekend',
  location: 'Drübeck Abbey',
  description: null,
  firstDay: '2027-03-12',
  lastDay: '2027-03-14',
  cancelled: false,
  ended: false,
}
const adaInvited: Invitation = { id: 40, memberId: 1, firstName: 'Ada', lastName: 'Lovelace', linkActive: true, mail: 'failed' }

function organizerPage(invitations: Invitation[] = [adaInvited]) {
  server.use(
    getLegalLinksMockHandler({}),
    getCurrentSessionMockHandler(ada),
    getGetGatheringMockHandler(weekend),
    getListMembersMockHandler([ada, clara, johannes]),
    getListInvitationsMockHandler(invitations),
    getOverviewMockHandler({ rows: [], totals: { accepted: 0, tentative: 0, declined: 0, pending: 0, bedsNeeded: 0, bedsOffered: 0 } }),
  )
  return renderApp('/gatherings/5')
}

describe('invitation actions', () => {
  it('invites every active member who is not invited yet', async () => {
    let body: unknown = null
    server.use(
      http.post('*/api/gatherings/5/invitations', async ({ request }) => {
        body = await request.json()
        return HttpResponse.json({ count: 2 }, { status: 201 })
      }),
    )
    const { container } = organizerPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Invite all 2 active members not invited yet' }))
    expect(body).toEqual({})
    expect(await screen.findByText('2 invitations sent.')).toBeTruthy()
    expect(await violations(container)).toEqual([])
  })

  it('invites a chosen member', async () => {
    let body: unknown = null
    server.use(
      http.post('*/api/gatherings/5/invitations', async ({ request }) => {
        body = await request.json()
        return HttpResponse.json({ count: 1 }, { status: 201 })
      }),
    )
    organizerPage()
    await userEvent.click(await screen.findByText('Choose members to invite', { selector: 'summary' }))
    await userEvent.click(screen.getByLabelText('Schumann, Clara'))
    await userEvent.click(screen.getByRole('button', { name: 'Invite one chosen member' }))
    await waitFor(() => expect(body).toEqual({ memberIds: [2] }))
  })

  it('asks before it cancels the gathering', async () => {
    let cancelled = false
    server.use(
      http.post('*/api/gatherings/5/cancellation', () => {
        cancelled = true
        return new HttpResponse(null, { status: 204 })
      }),
    )
    organizerPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Cancel gathering' }))
    expect(screen.getByText('Cancel Rehearsal weekend and inform every invited member by mail?')).toBeTruthy()
    expect(cancelled).toBe(false)
    await userEvent.click(screen.getAllByRole('button', { name: 'Cancel gathering' })[0]!)
    await waitFor(() => expect(cancelled).toBe(true))
  })
})
