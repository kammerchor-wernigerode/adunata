// @vitest-environment jsdom
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { render, screen, waitFor, within } from '@testing-library/react'
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
import { createBundle, formatMessage } from '@/l10n/bundle'
import { LocalizationContext } from '@/l10n/translate'
import { renderApp, violations } from '@/testing/app'
import { setupApiServer } from '@/testing/server'
import { DownloadOverview } from './-components/DownloadOverview'

const server = setupApiServer()

const weekend: Gathering = {
  id: 5,
  title: 'Rehearsal weekend',
  location: 'Drübeck Abbey',
  description: null,
  firstDay: '2027-03-12',
  lastDay: '2027-03-14',
  cancelled: false,
  deadlinePassed: false,
  begun: false,
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

function organizerPage(gathering: Gathering = weekend, shown: Overview = overview) {
  server.use(
    getLegalLinksMockHandler({}),
    getCurrentSessionMockHandler({ id: 9, firstName: 'Ada', lastName: 'Lovelace', mail: 'ada@example.org', language: 'en', active: true, organizer: true }),
    getGetGatheringMockHandler(gathering),
    getListMembersMockHandler([]),
    getListInvitationsMockHandler([]),
    getOverviewMockHandler(shown),
  )
  return renderApp('/gatherings/5')
}

function names(): string[] {
  return within(screen.getByRole('table'))
    .getAllByRole('row')
    .slice(1)
    .map((row) => within(row).getAllByRole('cell')[0]!.textContent)
}

async function actionsOf(name: string): Promise<HTMLElement> {
  const row = (await screen.findByText(name)).closest('tr')!
  return within(row).getAllByRole('cell').at(-1)!
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
    const sophie = await actionsOf('Sophie Graf')
    expect(within(sophie).getByText(/mail failed/)).toBeTruthy()
    const name = within(sophie.closest('tr')!).getAllByRole('cell')[0]!
    expect(within(name).queryByRole('button')).toBeNull()
    expect(within(name).queryByText(/mail failed/)).toBeNull()
    expect(within(name).getByRole('link').textContent).toBe('Sophie Graf')
    await userEvent.click(within(sophie).getByRole('button', { name: 'Resend: Sophie Graf' }))
    expect(resent).toBe(true)
    await waitFor(() => expect(within(sophie).queryByRole('button', { name: 'Resend: Sophie Graf' })).toBeNull())
    expect(within(sophie).getByRole('alert').textContent).toBe('')
  })

  it('keeps the resend action and names the failure when the server refuses it', async () => {
    server.use(http.post('*/api/invitations/5/resend', () => HttpResponse.json({ errors: [] }, { status: 422 })))
    organizerPage()
    const sophie = await actionsOf('Sophie Graf')
    await userEvent.click(within(sophie).getByRole('button', { name: 'Resend: Sophie Graf' }))
    await waitFor(() => expect(within(sophie).getByRole('alert').textContent).toBe('Something went wrong. Try again.'))
    expect(within(sophie).getByRole('button', { name: 'Resend: Sophie Graf' })).toBeTruthy()
  })

  it('names every column of a member in the cell, so the narrow layout keeps them all', async () => {
    organizerPage()
    const table = await screen.findByRole('table')
    expect(within(table).getAllByRole('columnheader').at(-1)!.textContent).toBe('Actions')
    const cells = within(table)
      .getAllByRole('row')
      .slice(1)
      .map((row) => within(row).getAllByRole('cell'))
    const columns = ['Name', 'Status', 'Period', 'Beds', 'Comment']
    expect(cells.map((row) => row.map((cell) => cell.firstElementChild?.textContent))).toEqual([
      ...Array.from({ length: 4 }, () => [...columns, undefined]),
      [...columns, 'Actions'],
    ])
    expect(cells.map((row) => row.at(-1)!.childNodes.length)).toEqual([0, 0, 0, 0, 2])
  })

  it('shows a revoked link with the resend action in the last cell', async () => {
    const revoked = { ...base, invitationId: 6, firstName: 'Paul', lastName: 'Roth', linkActive: false }
    organizerPage(weekend, { ...overview, rows: [...rows, revoked] })
    const paul = await actionsOf('Paul Roth')
    expect(within(paul).getByText(/link revoked/)).toBeTruthy()
    expect(within(paul).getByRole('button', { name: 'Resend: Paul Roth' })).toBeTruthy()
  })

  it('shows the notice without the resend action on an ended gathering', async () => {
    const { container } = organizerPage({ ...weekend, begun: true, ended: true })
    const sophie = await actionsOf('Sophie Graf')
    expect(sophie.firstElementChild?.textContent).toBe('Actions')
    expect(within(sophie).getByText(/mail failed/)).toBeTruthy()
    expect(within(sophie).queryByRole('button')).toBeNull()
    expect(await violations(container)).toEqual([])
  })
})

function remindCalls(status = 201) {
  const calls: string[] = []
  server.use(
    http.post('*/api/gatherings/5/reminders', ({ request }) => {
      calls.push(request.url)
      return status === 201 ? HttpResponse.json({ count: 2 }, { status }) : HttpResponse.json({ errors: [] }, { status })
    }),
  )
  return calls
}

describe('remind pending members', () => {
  const pending = { ...base, status: 'pending' as const, beds: null, bedCount: null, changedBy: null, changedAt: null }
  const remindRows: OverviewRow[] = [
    base,
    { ...pending, invitationId: 7, firstName: 'Clara', lastName: 'Schmidt' },
    { ...pending, invitationId: 8, firstName: 'Jonas', lastName: 'Weber', mail: 'failed' },
    { ...pending, invitationId: 9, firstName: 'Paul', lastName: 'Roth', linkActive: false },
  ]
  const remindOverview: Overview = { ...overview, rows: remindRows }

  it('names the members with a pending invitation and an active link', async () => {
    organizerPage(weekend, remindOverview)
    expect(await screen.findByRole('button', { name: 'Remind 2 members' })).toBeTruthy()
  })

  it('sends nothing when the organizer goes back', async () => {
    const calls = remindCalls()
    organizerPage(weekend, remindOverview)
    await userEvent.click(await screen.findByRole('button', { name: 'Remind 2 members' }))
    const dialog = await screen.findByRole('dialog', { name: 'Remind members?' })
    expect(within(dialog).getByText('2 members receive a new link. Their old link stops working.')).toBeTruthy()
    await userEvent.click(within(dialog).getByRole('button', { name: 'Back' }))
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
    expect(calls).toEqual([])
  })

  it('closes the dialog, reloads the overview and reports the count after confirmation', async () => {
    const calls = remindCalls()
    let loads = 0
    organizerPage(weekend, remindOverview)
    server.use(
      http.get('*/api/gatherings/5/overview', () => {
        loads += 1
        return HttpResponse.json(remindOverview)
      }),
    )
    await userEvent.click(await screen.findByRole('button', { name: 'Remind 2 members' }))
    const loadsBefore = loads
    const dialog = await screen.findByRole('dialog')
    await userEvent.click(within(dialog).getByRole('button', { name: 'Send reminders' }))
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
    expect(calls).toHaveLength(1)
    await waitFor(() => expect(loads).toBeGreaterThan(loadsBefore))
    const notice = within(screen.getByRole('region', { name: 'Responses' })).getByRole('status')
    await waitFor(() => expect(notice.textContent).toBe('2 reminders sent.'))
  })

  it('keeps the dialog open and names the failure when the server refuses', async () => {
    remindCalls(422)
    organizerPage(weekend, remindOverview)
    await userEvent.click(await screen.findByRole('button', { name: 'Remind 2 members' }))
    const dialog = await screen.findByRole('dialog')
    await userEvent.click(within(dialog).getByRole('button', { name: 'Send reminders' }))
    expect((await within(dialog).findByRole('alert')).textContent).toBe('Something went wrong. Try again.')
    expect(screen.getByRole('dialog')).toBeTruthy()
  })

  it('offers no reminder when nobody is pending with an active link', async () => {
    organizerPage(weekend, { ...overview, rows: [base, remindRows[3]!] })
    await screen.findByRole('table')
    expect(screen.queryByRole('button', { name: /Remind/ })).toBeNull()
  })

  it.each([
    ['cancelled', { ...weekend, cancelled: true }],
    ['ended', { ...weekend, begun: true, ended: true }],
    ['begun', { ...weekend, begun: true }],
  ])('offers no reminder for a %s gathering', async (_, gathering) => {
    organizerPage(gathering, remindOverview)
    await screen.findByRole('table')
    expect(screen.queryByRole('button', { name: /Remind/ })).toBeNull()
  })
})

describe('download the overview', () => {
  const notice = 'The file holds personal data. Adunata does not delete it, so delete it once you no longer need it.'

  it('offers the link with the notice next to it', async () => {
    const { container } = organizerPage()
    const link = await screen.findByRole('link', { name: 'Download CSV' })
    expect(link.getAttribute('href')).toBe('/api/gatherings/5/overview.csv?language=en')
    expect(link.getAttribute('aria-describedby')).toBe(screen.getByText(notice).id)
    expect(await violations(container)).toEqual([])
  })

  it('names the German language in the address', () => {
    const german = createBundle('de', readFileSync(path.join(import.meta.dirname, '../../../../../../server/locales/de.ftl'), 'utf8'))
    render(
      <LocalizationContext.Provider value={{ locale: 'de', translate: (id, args) => formatMessage([german], id, args), setLocale: () => {} }}>
        <DownloadOverview gatheringId={5} />
      </LocalizationContext.Provider>,
    )
    expect(screen.getByRole('link', { name: 'CSV herunterladen' }).getAttribute('href')).toBe('/api/gatherings/5/overview.csv?language=de')
    expect(
      screen.getByText('Die Datei enthält personenbezogene Daten. Adunata löscht sie nicht, also lösche sie, sobald du sie nicht mehr brauchst.'),
    ).toBeTruthy()
  })

  it.each([
    ['cancelled', { ...weekend, cancelled: true }],
    ['ended', { ...weekend, begun: true, ended: true }],
  ])('keeps the link for a %s gathering', async (_, gathering) => {
    organizerPage(gathering)
    expect(await screen.findByRole('link', { name: 'Download CSV' })).toBeTruthy()
    expect(screen.getByText(notice)).toBeTruthy()
  })
})
