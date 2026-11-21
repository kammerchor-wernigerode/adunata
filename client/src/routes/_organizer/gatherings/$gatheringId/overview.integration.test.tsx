// @vitest-environment jsdom
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { describe, expect, it } from 'vitest'
import {
  getCurrentSessionMockHandler,
  getGetGatheringMockHandler,
  getLegalLinksMockHandler,
  getListInvitationsMockHandler,
  getListMembersMockHandler,
  getOverviewMockHandler,
} from '@/api/generated/api.msw'
import type { Gathering, Overview, OverviewRow } from '@/api/generated/model'
import { renderApp, violations } from '@/testing/app'
import { setupApiServer } from '@/testing/server'

const server = setupApiServer()

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
const base: OverviewRow = {
  invitationId: 1,
  firstName: 'Anna',
  lastName: 'Becker',
  status: 'accepted',
  arrival: null,
  departure: null,
  beds: 'need',
  bedCount: 1,
  comment: null,
  changedBy: 'member',
  changedAt: '2027-02-20T09:30',
  outsideSpan: false,
  linkActive: true,
  mail: 'sent',
}
const rows: OverviewRow[] = [
  base,
  { ...base, invitationId: 2, firstName: 'Ben', lastName: 'Kraus', beds: 'need', bedCount: 1, arrival: '2027-03-12T18:00', departure: '2027-03-14T15:00' },
  { ...base, invitationId: 3, firstName: 'Lena', lastName: 'Wolf', beds: 'offer', bedCount: 3, comment: 'Brings scores for the altos', arrival: '2027-03-11T18:00', outsideSpan: true },
  { ...base, invitationId: 4, firstName: 'Moritz', lastName: 'Lange', status: 'declined', beds: null, bedCount: null, comment: 'Holiday' },
  { ...base, invitationId: 5, firstName: 'Sophie', lastName: 'Graf', status: 'pending', beds: null, bedCount: null, changedBy: null, changedAt: null, mail: 'failed' },
]
const overview: Overview = {
  rows,
  totals: { accepted: 3, tentative: 0, declined: 1, pending: 1, bedsNeeded: 2, bedsOffered: 3 },
}

function organizerPage() {
  server.use(
    getLegalLinksMockHandler({}),
    getCurrentSessionMockHandler({ id: 9, firstName: 'Ada', lastName: 'Lovelace', mail: 'ada@example.org', language: 'en', active: true, organizer: true }),
    getGetGatheringMockHandler(weekend),
    getListMembersMockHandler([]),
    getListInvitationsMockHandler([]),
    getOverviewMockHandler(overview),
  )
  return renderApp('/gatherings/5')
}

function names(): string[] {
  return within(screen.getByRole('table'))
    .getAllByRole('row')
    .slice(1)
    .map((row) => within(row).getAllByRole('cell')[0]!.textContent)
}

describe('overview', () => {
  it('recovers from a failed overview once the server answers again', async () => {
    let failing = true
    organizerPage()
    server.use(http.get('*/api/gatherings/5/overview', () => (failing ? new HttpResponse(null, { status: 500 }) : HttpResponse.json(overview))))
    const retry = await screen.findByRole('button', { name: 'Try again' })
    failing = false
    await userEvent.click(retry)
    expect(await screen.findByRole('heading', { level: 1, name: 'Rehearsal weekend' })).toBeTruthy()
    expect(await screen.findByText('Sophie Graf')).toBeTruthy()
  })

  it('plans beds with the totals and the balance', async () => {
    const { container } = organizerPage()
    const offered = (await screen.findByText('Beds offered')).parentElement!
    expect(within(offered).getByText('3')).toBeTruthy()
    expect(within(offered).getByText('one more than needed')).toBeTruthy()
    expect(within(screen.getByText('Beds needed').parentElement!).getByText('2')).toBeTruthy()
    expect(await violations(container)).toEqual([])
  })

  it('filters by status', async () => {
    organizerPage()
    await screen.findByRole('table')
    await userEvent.click(screen.getByRole('radio', { name: 'Pending' }))
    expect(names()).toEqual([expect.stringContaining('Sophie Graf')])
    await userEvent.click(screen.getByRole('radio', { name: 'All 5' }))
    expect(names()).toHaveLength(5)
  })

  it('shows a member without a period as present throughout and marks a period outside the days', async () => {
    organizerPage()
    const table = await screen.findByRole('table')
    const anna = within(table).getByText('Anna Becker').closest('tr')!
    expect(within(anna).getByText('whole gathering')).toBeTruthy()
    const lena = within(table).getByText('Lena Wolf').closest('tr')!
    expect(within(lena).getByText('outside the days')).toBeTruthy()
    const ben = within(table).getByText('Ben Kraus').closest('tr')!
    expect(within(ben).getByText('Fri 18:00 to Sun 15:00')).toBeTruthy()
  })

  it('shows a failed invitation mail at its invitation and resends it', async () => {
    let resent = false
    server.use(
      http.post('*/api/invitations/5/resend', () => {
        resent = true
        return new HttpResponse(null, { status: 204 })
      }),
    )
    organizerPage()
    const sophie = (await screen.findByText('Sophie Graf')).closest('tr')!
    expect(within(sophie).getByText(/mail failed/)).toBeTruthy()
    await userEvent.click(within(sophie).getByRole('button', { name: 'Resend: Sophie Graf' }))
    expect(resent).toBe(true)
    await waitFor(() => expect(within(sophie).queryByRole('button', { name: 'Resend: Sophie Graf' })).toBeNull())
    expect(within(sophie).getByRole('alert').textContent).toBe('')
  })

  it('keeps the resend action and names the failure when the server refuses it', async () => {
    server.use(http.post('*/api/invitations/5/resend', () => HttpResponse.json({ errors: [] }, { status: 422 })))
    organizerPage()
    const sophie = (await screen.findByText('Sophie Graf')).closest('tr')!
    await userEvent.click(within(sophie).getByRole('button', { name: 'Resend: Sophie Graf' }))
    await waitFor(() => expect(within(sophie).getByRole('alert').textContent).toBe('Something went wrong. Try again.'))
    expect(within(sophie).getByRole('button', { name: 'Resend: Sophie Graf' })).toBeTruthy()
  })

  it('names every column of a member in the cell, so the narrow layout keeps them all', async () => {
    organizerPage()
    const table = await screen.findByRole('table')
    for (const row of within(table).getAllByRole('row').slice(1)) {
      const labels = within(row)
        .getAllByRole('cell')
        .map((cell) => cell.firstElementChild?.textContent)
      expect(labels).toEqual(['Name', 'Status', 'Period', 'Beds', 'Comment'])
    }
  })
})
