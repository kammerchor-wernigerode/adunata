// @vitest-environment jsdom
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  getCurrentSessionMockHandler,
  getGetGatheringMockHandler,
  getLegalLinksMockHandler,
  getListGatheringsMockHandler,
  getListInvitationsMockHandler,
  getListMembersMockHandler,
  getOverviewMockHandler,
} from '@/api/generated/api.msw'
import type { Gathering } from '@/api/generated/model'
import { renderApp, violations } from '@/testing/app'
import { setupApiServer } from '@/testing/server'

const server = setupApiServer()

const march: Gathering = {
  id: 1,
  title: 'Rehearsal weekend in March',
  location: 'Drübeck Abbey',
  description: null,
  firstDay: '2027-03-12',
  lastDay: '2027-03-14',
  cancelled: false,
  ended: false,
}
const april: Gathering = { ...march, id: 2, title: 'Concert in April', firstDay: '2027-04-30', lastDay: '2027-05-02' }
const autumn: Gathering = { ...march, id: 3, title: 'Autumn camp', firstDay: '2026-10-01', lastDay: '2026-10-04', ended: true }

function signedIn() {
  server.use(
    getLegalLinksMockHandler({}),
    getCurrentSessionMockHandler({
      id: 9,
      firstName: 'Ada',
      lastName: 'Lovelace',
      mail: 'ada@example.org',
      language: 'en',
      active: true,
      organizer: true,
    }),
  )
}

describe('gathering list', () => {
  it('shows upcoming gatherings first ordered by first day and past ones separately', async () => {
    signedIn()
    server.use(getListGatheringsMockHandler({ upcoming: [march, april], past: [autumn] }))
    const { container } = renderApp('/gatherings')
    const upcoming = (await screen.findByRole('heading', { level: 2, name: 'Upcoming' })).closest('section')!
    const past = screen.getByRole('heading', { level: 2, name: 'Past' }).closest('section')!
    expect(within(upcoming).getAllByRole('link').map((link) => link.textContent)).toEqual([
      'Rehearsal weekend in March',
      'Concert in April',
    ])
    expect(within(past).getAllByRole('link').map((link) => link.textContent)).toEqual(['Autumn camp'])
    expect(within(upcoming).getAllByText(/Drübeck Abbey/)[0]!.textContent).toContain('2027')
    expect(await violations(container)).toEqual([])
  })
})

/** Opens the calendar of a day field and picks a day of the month it shows. Days 18 to 20 never appear twice in one month view. */
async function pickDay(label: string, dayOfMonth: string) {
  await userEvent.click(screen.getByLabelText(label))
  const grid = await screen.findByRole('grid', { name: label })
  await userEvent.click(within(grid).getByText(dayOfMonth))
  await userEvent.keyboard('{Escape}')
}

/** The days of the month a calendar shows that the organizer can choose, by their accessible names. */
function enabledDays(grid: HTMLElement) {
  return within(grid)
    .getAllByRole('button')
    .filter((day) => !day.hasAttribute('disabled'))
    .map((day) => day.getAttribute('aria-label') ?? '')
}

describe('gathering form', () => {
  beforeEach(() => {
    vi.useFakeTimers({ toFake: ['Date'] })
    vi.setSystemTime(new Date(2027, 2, 10, 12))
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('marks a last day before the first day', async () => {
    signedIn()
    server.use(
      http.post('*/api/gatherings', () =>
        HttpResponse.json({ errors: [{ field: 'lastDay', code: 'beforeFirstDay' }] }, { status: 422 }),
      ),
    )
    const { container } = renderApp('/gatherings/new')
    await userEvent.type(await screen.findByLabelText('Title'), 'Rehearsal weekend')
    await userEvent.type(screen.getByLabelText('Location'), 'Drübeck Abbey')
    await pickDay('Last day', '18')
    await pickDay('First day', '20')
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    const lastDay = screen.getByLabelText('Last day')
    await waitFor(() => expect(lastDay.getAttribute('aria-invalid')).toBe('true'))
    expect(document.activeElement).toBe(lastDay)
    expect(document.getElementById(lastDay.getAttribute('aria-describedby') ?? '')?.textContent).toBe(
      'The last day cannot lie before the first day.',
    )
    expect(await violations(container)).toEqual([])
  })

  it('marks a first day in the past', async () => {
    signedIn()
    server.use(
      http.post('*/api/gatherings', () =>
        HttpResponse.json({ errors: [{ field: 'firstDay', code: 'firstDayInPast' }] }, { status: 422 }),
      ),
    )
    const { container } = renderApp('/gatherings/new')
    await userEvent.type(await screen.findByLabelText('Title'), 'Rehearsal weekend')
    await userEvent.type(screen.getByLabelText('Location'), 'Drübeck Abbey')
    await pickDay('First day', '18')
    await pickDay('Last day', '20')
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    const firstDay = screen.getByLabelText('First day')
    await waitFor(() => expect(firstDay.getAttribute('aria-invalid')).toBe('true'))
    expect(document.activeElement).toBe(firstDay)
    expect(document.getElementById(firstDay.getAttribute('aria-describedby') ?? '')?.textContent).toBe(
      'The first day cannot lie in the past.',
    )
    expect(await violations(container)).toEqual([])
  })

  it('offers no first day before today', async () => {
    signedIn()
    renderApp('/gatherings/new')
    await userEvent.click(await screen.findByLabelText('First day'))
    const enabled = enabledDays(await screen.findByRole('grid', { name: 'First day' }))
    expect(enabled[0]).toMatch(/March 10th, 2027/)
    expect(enabled.some((day) => /March 9th, 2027/.test(day))).toBe(false)
  })

  it('keeps the stored past first day of a running gathering', async () => {
    signedIn()
    server.use(getGetGatheringMockHandler({ ...march, firstDay: '2027-03-08', lastDay: '2027-03-12' }))
    renderApp('/gatherings/1/edit')
    await userEvent.click(await screen.findByLabelText('First day'))
    const enabled = enabledDays(await screen.findByRole('grid', { name: 'First day' }))
    expect(enabled.slice(0, 2).map((day) => /March (\d+)th/.exec(day)?.[1])).toEqual(['8', '10'])
  })

  it('offers no last day before the first day', async () => {
    signedIn()
    renderApp('/gatherings/new')
    await screen.findByLabelText('First day')
    await pickDay('First day', '12')
    await userEvent.click(screen.getByLabelText('Last day'))
    const enabled = enabledDays(await screen.findByRole('grid', { name: 'Last day' }))
    expect(enabled[0]).toMatch(/March 12th, 2027/)
    expect(enabled.some((day) => /March 11th, 2027/.test(day))).toBe(false)
  })

  it('moves the last day along with the first day', async () => {
    signedIn()
    renderApp('/gatherings/new')
    await screen.findByLabelText('First day')
    await pickDay('First day', '12')
    await pickDay('Last day', '14')
    await pickDay('First day', '15')
    expect(screen.getByLabelText('Last day').textContent).toBe('Wed, March 17, 2027')
  })

  it('moves no last day while the first day is empty', async () => {
    signedIn()
    renderApp('/gatherings/new')
    await screen.findByLabelText('Last day')
    await pickDay('Last day', '14')
    await pickDay('First day', '12')
    expect(screen.getByLabelText('Last day').textContent).toBe('Sun, March 14, 2027')
  })


  it('opens the gathering with its span and the action to edit it', async () => {
    signedIn()
    server.use(
      getGetGatheringMockHandler({ ...march, description: 'We rehearse the April programme.' }),
      getListInvitationsMockHandler([]),
      getListMembersMockHandler([]),
      getOverviewMockHandler({ rows: [], totals: { accepted: 0, tentative: 0, declined: 0, pending: 0, bedsNeeded: 0, bedsOffered: 0 } }),
    )
    const { container } = renderApp('/gatherings/1')
    expect(await screen.findByRole('heading', { level: 1, name: 'Rehearsal weekend in March' })).toBeTruthy()
    expect(screen.getByRole('link', { name: 'Edit gathering' }).getAttribute('href')).toBe('/gatherings/1/edit')
    expect(await violations(container)).toEqual([])
  })
})
