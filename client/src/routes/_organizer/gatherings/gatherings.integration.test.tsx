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

beforeEach(() => {
  localStorage.clear()
})

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
const spring2026: Gathering = { ...march, id: 11, title: 'Spring concert', firstDay: '2026-04-18', lastDay: '2026-04-19', ended: true }
const newYear: Gathering = { ...march, id: 12, title: 'New Year retreat', firstDay: '2025-12-30', lastDay: '2026-01-02', ended: true }
const summer2025: Gathering = { ...march, id: 13, title: 'Summer tour', firstDay: '2025-07-04', lastDay: '2025-07-06', cancelled: true, ended: true }
const autumn2024: Gathering = { ...march, id: 14, title: 'Autumn camp 2024', firstDay: '2024-10-03', lastDay: '2024-10-06', ended: true }
const pastYears = [spring2026, newYear, summer2025, autumn2024]
const pastWithoutCurrentYear = [newYear, summer2025, autumn2024]
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

/** Renders the gathering list with the given past gatherings and returns its section "Past". */
async function renderPast(past: Gathering[]) {
  signedIn()
  server.use(getListGatheringsMockHandler({ upcoming: [], past }))
  const rendered = renderApp('/gatherings')
  const section = (await screen.findByRole('heading', { level: 2, name: 'Past' })).closest('section')!
  return { ...rendered, section }
}

function yearTrigger(section: HTMLElement, year: string) {
  return within(section).getByRole('button', { name: new RegExp(`^${year} `) })
}

function openYears(section: HTMLElement) {
  return within(section)
    .getAllByRole('button')
    .filter((button) => button.getAttribute('aria-expanded') === 'true')
    .map((button) => button.textContent.slice(0, 4))
}

function shownTitles(section: HTMLElement) {
  return within(section).queryAllByRole('link').map((link) => link.textContent)
}

describe('past gatherings by year', () => {
  it('shows the years latest first with their number of gatherings', async () => {
    const { container, section } = await renderPast(pastYears)
    expect(within(section).getAllByRole('heading', { level: 3 }).map((heading) => heading.textContent)).toEqual([
      '2026 1 gathering',
      '2025 2 gatherings',
      '2024 1 gathering',
    ])
    expect(await violations(container)).toEqual([])
  })

  it('opens only the latest year on a first visit', async () => {
    const { section } = await renderPast(pastWithoutCurrentYear)
    expect(openYears(section)).toEqual(['2025'])
    expect(yearTrigger(section, '2024').getAttribute('aria-expanded')).toBe('false')
    expect(shownTitles(section)).toEqual(['New Year retreat', 'Summer tour'])
  })

  it('opens several years at once', async () => {
    const { section } = await renderPast(pastWithoutCurrentYear)
    await userEvent.click(yearTrigger(section, '2024'))
    expect(openYears(section)).toEqual(['2025', '2024'])
    expect(shownTitles(section)).toEqual(['New Year retreat', 'Summer tour', 'Autumn camp 2024'])
  })

  it('hides the gatherings of a closed year but keeps its count', async () => {
    const { section } = await renderPast(pastWithoutCurrentYear)
    await userEvent.click(yearTrigger(section, '2025'))
    expect(yearTrigger(section, '2025').getAttribute('aria-expanded')).toBe('false')
    expect(yearTrigger(section, '2025').textContent).toBe('2025 2 gatherings')
    expect(shownTitles(section)).toEqual([])
  })

  it('remembers the years the organizer opened or closed', async () => {
    const first = await renderPast(pastWithoutCurrentYear)
    await userEvent.click(yearTrigger(first.section, '2025'))
    await userEvent.click(yearTrigger(first.section, '2024'))
    first.unmount()
    const { section } = await renderPast(pastWithoutCurrentYear)
    expect(openYears(section)).toEqual(['2024'])
    expect(shownTitles(section)).toEqual(['Autumn camp 2024'])
  })

  it('opens a new latest year next to the remembered ones', async () => {
    localStorage.setItem('adunata.past-years', JSON.stringify({ 2025: false, 2024: true }))
    const { section } = await renderPast(pastYears)
    expect(openYears(section)).toEqual(['2026', '2024'])
  })

  it('treats an unreadable entry as no choice', async () => {
    localStorage.setItem('adunata.past-years', 'not json')
    const { section } = await renderPast(pastYears)
    expect(openYears(section)).toEqual(['2026'])
  })

  it('lists a gathering over New Year in the year of its first day', async () => {
    const { section } = await renderPast(pastWithoutCurrentYear)
    const year2025 = yearTrigger(section, '2025').closest('[data-slot="collapsible"]') as HTMLElement
    expect(within(year2025).getByRole('link', { name: 'New Year retreat' })).toBeTruthy()
  })

  it('shows the badge and the date line of a cancelled gathering', async () => {
    const { section } = await renderPast(pastWithoutCurrentYear)
    const row = within(section).getByRole('link', { name: 'Summer tour' }).closest('li')!
    expect(within(row).getByText('Cancelled')).toBeTruthy()
    expect(within(row).getByText(/Drübeck Abbey/).textContent).toContain('2025')
  })

  it('shows no year without past gatherings', async () => {
    const { section } = await renderPast([])
    expect(within(section).queryAllByRole('heading', { level: 3 })).toEqual([])
    expect(within(section).getByText('None yet.')).toBeTruthy()
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

/** Answers each create with the stored gathering under the next id and lists every stored one as upcoming. */
function storing() {
  const stored: Gathering[] = []
  server.use(
    http.post('*/api/gatherings', async ({ request }) => {
      const input = (await request.json()) as Omit<Gathering, 'id' | 'cancelled' | 'ended'>
      const gathering = { ...input, id: 20 + stored.length, cancelled: false, ended: false }
      stored.push(gathering)
      return HttpResponse.json(gathering, { status: 201 })
    }),
    http.get('*/api/gatherings', () => HttpResponse.json({ upcoming: stored, past: [] })),
  )
  return stored
}

async function fillNew() {
  await userEvent.type(await screen.findByLabelText('Title'), 'Probenwochenende')
  await userEvent.type(screen.getByLabelText('Location'), 'Wernigerode')
  await userEvent.type(screen.getByLabelText('Description'), 'We rehearse.')
  await pickDay('First day', '12')
  await pickDay('Last day', '14')
}

describe('gathering form', () => {
  beforeEach(() => {
    vi.useFakeTimers({ toFake: ['Date'] })
    vi.setSystemTime(new Date(2027, 2, 10, 12))
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('offers save and add another only for a new gathering', async () => {
    signedIn()
    renderApp('/gatherings/new')
    await screen.findByLabelText('Title')
    const save = screen.getByRole('button', { name: 'Save' })
    const next = screen.getByRole('button', { name: 'Save & add another' })
    expect(save.compareDocumentPosition(next) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
  })

  it('offers no save and add another when changing a gathering', async () => {
    signedIn()
    server.use(getGetGatheringMockHandler(march))
    renderApp('/gatherings/1/edit')
    expect(await screen.findByRole('button', { name: 'Save' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Save & add another' })).toBeNull()
  })

  it('stores a gathering and keeps the form open for the next one', async () => {
    signedIn()
    storing()
    const { container } = renderApp('/gatherings/new')
    await fillNew()
    await userEvent.click(screen.getByRole('button', { name: 'Save & add another' }))
    const line = await screen.findByRole('status')
    expect(line.textContent.replace(/\s/g, ' ')).toBe('Probenwochenende, Fri, March 12 – Sun, March 14, 2027 saved.')
    expect(within(line).getByRole('link', { name: 'Probenwochenende' }).getAttribute('href')).toBe('/gatherings/20')
    const title = screen.getByLabelText('Title')
    expect((title as HTMLInputElement).value).toBe('Probenwochenende')
    expect(screen.getByLabelText<HTMLInputElement>('Location').value).toBe('Wernigerode')
    expect(screen.getByLabelText<HTMLInputElement>('Description').value).toBe('We rehearse.')
    expect(screen.getByLabelText('First day').textContent).not.toMatch(/2027/)
    expect(screen.getByLabelText('Last day').textContent).not.toMatch(/2027/)
    await waitFor(() => expect(document.activeElement).toBe(title))
    expect(await violations(container)).toEqual([])
  })

  it('names only the latest stored gathering', async () => {
    signedIn()
    storing()
    renderApp('/gatherings/new')
    await fillNew()
    await userEvent.click(screen.getByRole('button', { name: 'Save & add another' }))
    await screen.findByRole('status')
    await pickDay('First day', '19')
    await pickDay('Last day', '21')
    await userEvent.click(screen.getByRole('button', { name: 'Save & add another' }))
    await waitFor(() => expect(screen.getByRole('status').textContent.replace(/\s/g, ' ')).toBe('Probenwochenende, Fri, March 19 – Sun, March 21, 2027 saved.'))
    expect(screen.getAllByRole('status')).toHaveLength(1)
  })

  it('removes the line when the next save is refused', async () => {
    signedIn()
    storing()
    renderApp('/gatherings/new')
    await fillNew()
    await userEvent.click(screen.getByRole('button', { name: 'Save & add another' }))
    await screen.findByRole('status')
    server.use(
      http.post('*/api/gatherings', () =>
        HttpResponse.json({ errors: [{ field: 'firstDay', code: 'firstDayInPast' }] }, { status: 422 }),
      ),
    )
    await pickDay('First day', '18')
    await pickDay('Last day', '20')
    await userEvent.click(screen.getByRole('button', { name: 'Save & add another' }))
    const firstDay = screen.getByLabelText('First day')
    await waitFor(() => expect(firstDay.getAttribute('aria-invalid')).toBe('true'))
    expect(screen.queryByRole('status')).toBeNull()
    expect(document.activeElement).toBe(firstDay)
    expect(screen.getByLabelText<HTMLInputElement>('Title').value).toBe('Probenwochenende')
    expect(firstDay.textContent).toBe('Thu, March 18, 2027')
    expect(screen.getByLabelText('Last day').textContent).toBe('Sat, March 20, 2027')
  })

  it('stores the gathering and opens its page on Enter', async () => {
    signedIn()
    storing()
    server.use(
      getGetGatheringMockHandler({ ...march, id: 20, title: 'Probenwochenende' }),
      getListInvitationsMockHandler([]),
      getListMembersMockHandler([]),
      getOverviewMockHandler(emptyOverview),
    )
    renderApp('/gatherings/new')
    await fillNew()
    await userEvent.type(screen.getByLabelText('Title'), '{Enter}')
    expect(await screen.findByRole('heading', { level: 1, name: 'Probenwochenende' })).toBeTruthy()
  })

  it('disables both buttons while a save runs', async () => {
    signedIn()
    const pending: (() => void)[] = []
    server.use(
      http.post('*/api/gatherings', async () => {
        await new Promise<void>((resolve) => pending.push(resolve))
        return HttpResponse.json({ ...march, id: 20 }, { status: 201 })
      }),
    )
    renderApp('/gatherings/new')
    await fillNew()
    const save = screen.getByRole('button', { name: 'Save' })
    const next = screen.getByRole('button', { name: 'Save & add another' })
    await userEvent.click(next)
    await waitFor(() => expect(next.hasAttribute('disabled')).toBe(true))
    expect(save.hasAttribute('disabled')).toBe(true)
    for (const answer of pending) answer()
    await waitFor(() => expect(next.hasAttribute('disabled')).toBe(false))
    expect(save.hasAttribute('disabled')).toBe(false)
  })

  it('lists the gathering stored with save and add another', async () => {
    signedIn()
    storing()
    renderApp('/gatherings/new')
    await fillNew()
    await userEvent.click(screen.getByRole('button', { name: 'Save & add another' }))
    await screen.findByRole('status')
    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }))
    const upcoming = (await screen.findByRole('heading', { level: 2, name: 'Upcoming' })).closest('section')!
    expect(within(upcoming).getByRole('link', { name: 'Probenwochenende' })).toBeTruthy()
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
    const { container } = renderApp('/gatherings/1/edit')
    const firstDay = await screen.findByLabelText('First day')
    expect(await violations(container)).toEqual([])
    await userEvent.click(firstDay)
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
