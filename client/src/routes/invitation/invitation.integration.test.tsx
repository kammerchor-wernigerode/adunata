// @vitest-environment jsdom
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { getLegalLinksMockHandler } from '@/api/generated/api.msw'
import type { InvitationPage } from '@/api/generated/model'
import { renderApp, violations } from '@/testing/app'
import { setupApiServer } from '@/testing/server'

const server = setupApiServer()
const secret = 'f'.repeat(64)

afterEach(() => window.history.replaceState(null, '', '/'))

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
  response: null,
  attendees: { coming: [{ name: 'Anna Becker', status: 'accepted' }, { name: 'Felix Richter', status: 'tentative' }], declined: 2, pending: 5 },
  calendar: false,
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
    expect(requests[0]!.url).not.toContain(secret)
    expect(requests[0]!.headers.get('authorization')).toBe(`Bearer ${secret}`)
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

  it('offers no calendar file when the page says so', async () => {
    openLink(() => HttpResponse.json(page))
    expect(await screen.findByRole('heading', { level: 1, name: 'Rehearsal weekend' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Add to calendar' })).toBeNull()
  })

  it('saves the calendar file with the secret in the Authorization header', async () => {
    const downloads: Request[] = []
    server.use(
      http.get('*/api/invitation/calendar', ({ request }) => {
        downloads.push(request)
        return new HttpResponse('BEGIN:VCALENDAR\r\n', { headers: { 'Content-Type': 'text/calendar; charset=utf-8' } })
      }),
    )
    const saved: string[] = []
    const createObjectURL = vi.fn<() => string>(() => 'blob:calendar')
    Object.assign(URL, { createObjectURL, revokeObjectURL: vi.fn<() => void>() })
    const click = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(function (this: HTMLAnchorElement) {
      saved.push(this.download)
    })
    const { container } = openLink(() => HttpResponse.json({ ...page, state: 'cancelled', calendar: true }))
    await userEvent.click(await screen.findByRole('button', { name: 'Add to calendar' }))
    expect(screen.getByText('Your calendar does not follow later changes on its own. A new download replaces the entry.')).toBeTruthy()
    await waitFor(() => expect(saved).toEqual(['gathering.ics']))
    expect(downloads).toHaveLength(1)
    expect(downloads[0]!.url).not.toContain(secret)
    expect(downloads[0]!.headers.get('authorization')).toBe(`Bearer ${secret}`)
    expect(createObjectURL).toHaveBeenCalledOnce()
    expect(await violations(container)).toEqual([])
    click.mockRestore()
  })

  it('says when the calendar file could not be loaded', async () => {
    server.use(http.get('*/api/invitation/calendar', () => new HttpResponse(null, { status: 404 })))
    openLink(() => HttpResponse.json({ ...page, calendar: true }))
    await userEvent.click(await screen.findByRole('button', { name: 'Add to calendar' }))
    const message = await screen.findByText('The calendar file could not be loaded. Reload the page and try again.')
    expect(message.getAttribute('role')).toBe('alert')
  })

  it('lets the member read who sees name, status and comment', async () => {
    openLink(() => HttpResponse.json(page))
    expect(await screen.findByText('Other invited members see your name and whether you come.')).toBeTruthy()
    const comment = screen.getByLabelText('Comment')
    expect(document.getElementById(comment.getAttribute('aria-describedby') ?? '')?.textContent).toBe(
      'Only organizers read your comment.',
    )
  })

  it('lists who comes with a hidden word for tentative attendees', async () => {
    openLink(() => HttpResponse.json(page))
    const attendees = (await screen.findByRole('heading', { level: 2, name: 'Coming: 1, maybe 1' })).closest('section')!
    expect(within(attendees).getAllByRole('listitem').map((item) => item.textContent)).toEqual([
      'Anna Becker',
      'Felix Richter (Maybe)',
    ])
    expect(within(attendees).getByText('Declined: 2. No response yet: 5.')).toBeTruthy()
  })

  it('saves a response and states its values', async () => {
    let sent: unknown = null
    openLink(() => HttpResponse.json(page))
    server.use(
      http.put('*/api/invitation/response', async ({ request }) => {
        sent = await request.json()
        return HttpResponse.json({ status: 'accepted', arrival: '2027-03-12T18:00', departure: null, beds: 'offer', bedCount: 2, car: true, freeSeats: 3, comment: null })
      }),
    )
    await userEvent.click(await screen.findByRole('button', { name: 'Day of arrival Choose a day' }))
    await userEvent.click(await screen.findByRole('button', { name: /March 12th, 2027/ }))
    await userEvent.type(screen.getByLabelText('Time of arrival'), '18:00')
    await userEvent.click(screen.getByRole('radio', { name: 'I offer beds' }))
    await userEvent.type(screen.getByLabelText('How many?'), '2')
    await userEvent.click(screen.getByRole('radio', { name: 'Yes, by car' }))
    await userEvent.type(screen.getByLabelText('How many seats are free, not counting you?'), '3')
    await userEvent.click(screen.getByRole('button', { name: 'Save response' }))
    const status = await screen.findByText(/Response saved/)
    expect(status.closest('[role="status"]')?.textContent).toContain('Arrival Fri 18:00')
    expect(status.closest('[role="status"]')?.textContent).toContain('3 free seats')
    expect(sent).toEqual({ status: 'accepted', arrival: '2027-03-12T18:00', departure: null, beds: 'offer', bedCount: 2, car: true, freeSeats: 3, comment: null })
  })

  it('shows times of the instance whatever time zone the browser uses', async () => {
    openLink(() =>
      HttpResponse.json({ ...page, response: { status: 'accepted', arrival: '2027-03-12T18:00', departure: null, beds: 'none', bedCount: null, comment: null } }),
    )
    const time = await screen.findByLabelText<HTMLInputElement>('Time of arrival')
    expect(time.value).toBe('18:00')
    expect(screen.getByRole('button', { name: /Fri, March 12/ })).toBeTruthy()
  })

  it('operates the calendar by keyboard and offers only the days of the gathering', async () => {
    const { container } = openLink(() => HttpResponse.json(page))
    const trigger = await screen.findByRole('button', { name: 'Day of arrival Choose a day' })
    trigger.focus()
    await userEvent.keyboard('{Enter}')
    const grid = await screen.findByRole('grid', { name: 'Day of arrival' })
    expect(screen.getByRole('navigation', { name: 'Months for arrival' })).toBeTruthy()
    const enabled = within(grid)
      .getAllByRole('button')
      .filter((day) => !day.hasAttribute('disabled'))
      .map((day) => day.getAttribute('aria-label'))
    expect(enabled).toEqual(['Friday, March 12th, 2027', 'Saturday, March 13th, 2027', 'Sunday, March 14th, 2027'])
    within(grid).getByRole('button', { name: 'Friday, March 12th, 2027' }).focus()
    await userEvent.keyboard('{ArrowRight}')
    expect(document.activeElement?.getAttribute('aria-label')).toBe('Saturday, March 13th, 2027')
    await userEvent.keyboard('{Enter}')
    await waitFor(() => expect(screen.getByRole('button', { name: /Sat, March 13/ })).toBeTruthy())
    expect(await violations(container)).toEqual([])
  })

  it('explains a refusal because the gathering has begun', async () => {
    openLink(() => HttpResponse.json(page))
    server.use(
      http.put('*/api/invitation/response', () =>
        HttpResponse.json({ errors: [{ field: 'gathering', code: 'begun' }] }, { status: 422 }),
      ),
    )
    await userEvent.click(await screen.findByRole('radio', { name: 'No' }))
    await userEvent.click(screen.getByRole('button', { name: 'Save response' }))
    expect((await screen.findByRole('alert')).textContent).toContain('The gathering has begun')
    expect(screen.getByRole('button', { name: 'Save response' }).hasAttribute('disabled')).toBe(false)
  })

  it('names the last day to answer while the link is open', async () => {
    openLink(() => HttpResponse.json({ ...page, gathering: { ...page.gathering, lastDayToAnswer: '2027-03-06' } }))
    expect(await screen.findByText('Please answer by Mar 6, 2027.')).toBeTruthy()
  })

  it('names no last day to answer without a deadline', async () => {
    openLink(() => HttpResponse.json(page))
    await screen.findByRole('form')
    expect(screen.queryByText(/Please answer by/)).toBeNull()
  })

  it('offers only a decline with a comment after the deadline', async () => {
    let sent: unknown = null
    const { container } = openLink(() =>
      HttpResponse.json({
        ...page,
        state: 'deadlinePassed',
        gathering: { ...page.gathering, lastDayToAnswer: '2027-03-06' },
        response: { status: 'accepted', arrival: null, departure: null, beds: 'none', bedCount: null, comment: null },
      }),
    )
    server.use(
      http.put('*/api/invitation/response', async ({ request }) => {
        sent = await request.json()
        return HttpResponse.json({ status: 'declined', arrival: null, departure: null, beds: null, bedCount: null, car: null, freeSeats: null, comment: 'Krank' })
      }),
    )
    expect(await screen.findByText('The deadline for responses passed on Mar 6, 2027. You can still decline.')).toBeTruthy()
    expect(screen.getByText(/Your response/).textContent).toContain('Yes')
    expect(screen.getAllByRole('radio')).toHaveLength(1)
    expect(screen.getByRole('radio', { name: 'No' }).getAttribute('aria-checked')).toBe('true')
    expect(screen.queryByText('I offer beds')).toBeNull()
    expect(screen.getByRole('heading', { level: 2, name: 'Coming: 1, maybe 1' })).toBeTruthy()
    await userEvent.type(screen.getByLabelText('Comment'), 'Krank')
    await userEvent.click(screen.getByRole('button', { name: 'Save response' }))
    await screen.findByText('Response saved.')
    expect(sent).toEqual({ status: 'declined', arrival: null, departure: null, beds: null, bedCount: null, car: null, freeSeats: null, comment: 'Krank' })
    expect(await violations(container)).toEqual([])
  })

  it('frees the save button when saving fails unexpectedly', async () => {
    openLink(() => HttpResponse.json(page))
    server.use(http.put('*/api/invitation/response', () => new HttpResponse('not json', { status: 422 })))
    await userEvent.click(await screen.findByRole('radio', { name: 'No' }))
    await userEvent.click(screen.getByRole('button', { name: 'Save response' }))
    expect((await screen.findByRole('alert')).textContent).toBe('Something went wrong. Try again.')
    expect(screen.getByRole('button', { name: 'Save response' }).hasAttribute('disabled')).toBe(false)
  })

  it('rejects a bed count that is no whole number from 1 to 20', async () => {
    openLink(() => HttpResponse.json(page))
    await userEvent.click(await screen.findByRole('radio', { name: 'I need a bed' }))
    await userEvent.type(screen.getByLabelText('How many?'), '2.5')
    await userEvent.click(screen.getByRole('button', { name: 'Save response' }))
    const count = screen.getByLabelText('How many?')
    expect(count.getAttribute('aria-invalid')).toBe('true')
    expect(document.activeElement).toBe(count)
  })

  it('tells at the car question that only organizers see the car', async () => {
    const { container } = openLink(() => HttpResponse.json(page))
    const question = (await screen.findByText('Do you come by car?')).closest('fieldset')!
    expect(within(question).getByText('Only organizers see your car and free seats.')).toBeTruthy()
    expect(screen.getByRole('radiogroup', { description: /Only organizers see your car/ })).toBeTruthy()
    expect(await violations(container)).toEqual([])
  })

  it('opens a response without a statement about a car with the question unanswered', async () => {
    let sent = false
    openLink(() =>
      HttpResponse.json({ ...page, response: { status: 'accepted', arrival: null, departure: null, beds: 'none', bedCount: null, comment: null } }),
    )
    server.use(
      http.put('*/api/invitation/response', () => {
        sent = true
        return new HttpResponse(null, { status: 500 })
      }),
    )
    const yes = await screen.findByRole('radio', { name: 'Yes, by car' })
    expect(yes.getAttribute('aria-checked')).toBe('false')
    expect(screen.getByRole('radio', { name: 'No car' }).getAttribute('aria-checked')).toBe('false')
    await userEvent.click(screen.getByRole('button', { name: 'Save response' }))
    expect(await screen.findByText('Choose whether you come by car.')).toBeTruthy()
    expect(sent).toBe(false)
  })

  it('marks the car question when the server requires an answer', async () => {
    openLink(() => HttpResponse.json(page))
    server.use(
      http.put('*/api/invitation/response', () =>
        HttpResponse.json({ errors: [{ field: 'freeSeats', code: 'invalid' }] }, { status: 422 }),
      ),
    )
    await userEvent.click(await screen.findByRole('radio', { name: 'Neither' }))
    await userEvent.click(screen.getByRole('radio', { name: 'Yes, by car' }))
    await userEvent.type(screen.getByLabelText('How many seats are free, not counting you?'), '8')
    await userEvent.click(screen.getByRole('button', { name: 'Save response' }))
    expect(await screen.findByText('State the free seats, from 0 to 8.')).toBeTruthy()
    expect(screen.getByLabelText('How many seats are free, not counting you?').getAttribute('aria-invalid')).toBe('true')
  })

  it('rejects free seats outside 0 to 8 in the form', async () => {
    openLink(() => HttpResponse.json(page))
    await userEvent.click(await screen.findByRole('radio', { name: 'Yes, by car' }))
    await userEvent.type(screen.getByLabelText('How many seats are free, not counting you?'), '9')
    await userEvent.click(screen.getByRole('button', { name: 'Save response' }))
    const seats = screen.getByLabelText('How many seats are free, not counting you?')
    expect(seats.getAttribute('aria-invalid')).toBe('true')
  })

  it('names the car in the read-only view after the gathering has begun', async () => {
    openLink(() =>
      HttpResponse.json({
        ...page,
        state: 'begun',
        response: { status: 'accepted', arrival: null, departure: null, beds: 'none', bedCount: null, car: true, freeSeats: 2, comment: null },
      }),
    )
    expect((await screen.findByText(/^Your response:/)).textContent).toBe('Your response: Yes, 2 free seats')
  })
})
