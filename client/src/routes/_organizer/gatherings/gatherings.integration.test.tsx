// @vitest-environment jsdom
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { describe, expect, it } from 'vitest'
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
const emptyOverview = { rows: [], totals: { accepted: 0, tentative: 0, declined: 0, pending: 0, bedsNeeded: 0, bedsOffered: 0 } }

/** Asserts that the page action follows the main heading outside of it, before any other control. */
function expectPageAction(heading: HTMLElement, action: HTMLElement) {
  expect(heading.contains(action)).toBe(false)
  expect(heading.compareDocumentPosition(action) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
  expect(screen.getByRole('main').querySelector('a, button, input')).toBe(action)
}

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

  it('offers to create a gathering in the page header', async () => {
    signedIn()
    server.use(getListGatheringsMockHandler({ upcoming: [march], past: [] }))
    renderApp('/gatherings')
    const heading = await screen.findByRole('heading', { level: 1, name: 'Gatherings' })
    const create = screen.getByRole('link', { name: 'Create gathering' })
    expectPageAction(heading, create)
    expect(create.getAttribute('href')).toBe('/gatherings/new')
    expect(document.title).toMatch(/^Gatherings · /)
  })
})

/** Opens the calendar of a day field and picks a day of the month it shows. Days 18 to 20 never appear twice in one month view. */
async function pickDay(label: string, dayOfMonth: string) {
  await userEvent.click(screen.getByLabelText(label))
  const grid = await screen.findByRole('grid', { name: label })
  await userEvent.click(within(grid).getByText(dayOfMonth))
  await userEvent.keyboard('{Escape}')
}

describe('gathering form', () => {
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
    await pickDay('First day', '20')
    await pickDay('Last day', '18')
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    const lastDay = screen.getByLabelText('Last day')
    await waitFor(() => expect(lastDay.getAttribute('aria-invalid')).toBe('true'))
    expect(document.activeElement).toBe(lastDay)
    expect(document.getElementById(lastDay.getAttribute('aria-describedby') ?? '')?.textContent).toBe(
      'The last day cannot lie before the first day.',
    )
    expect(await violations(container)).toEqual([])
  })

  it('opens the gathering with its span and the action to edit it', async () => {
    signedIn()
    server.use(
      getGetGatheringMockHandler({ ...march, description: 'We rehearse the April programme.' }),
      getListInvitationsMockHandler([]),
      getListMembersMockHandler([]),
      getOverviewMockHandler(emptyOverview),
    )
    const { container } = renderApp('/gatherings/1')
    const heading = await screen.findByRole('heading', { level: 1, name: 'Rehearsal weekend in March' })
    const edit = screen.getByRole('link', { name: 'Edit gathering' })
    expectPageAction(heading, edit)
    expect(edit.getAttribute('href')).toBe('/gatherings/1/edit')
    expect(edit.compareDocumentPosition(screen.getByText('We rehearse the April programme.')) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    expect(await violations(container)).toEqual([])
  })

  it('offers no edit for an ended gathering', async () => {
    signedIn()
    server.use(getGetGatheringMockHandler(autumn), getListInvitationsMockHandler([]), getOverviewMockHandler(emptyOverview))
    renderApp('/gatherings/3')
    expect(await screen.findByRole('heading', { level: 1, name: 'Autumn camp' })).toBeTruthy()
    expect(screen.queryByRole('link', { name: 'Edit gathering' })).toBeNull()
  })
})
