// @vitest-environment jsdom
import { cleanup, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
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
  response: null,
  attendees: { coming: [{ name: 'Anna Becker', status: 'accepted' }, { name: 'Felix Richter', status: 'tentative' }], declined: 2, pending: 5 },
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
        return HttpResponse.json({ status: 'accepted', arrival: '2027-03-12T18:00', departure: null, beds: 'offer', bedCount: 2, comment: null })
      }),
    )
    await userEvent.click(await screen.findByRole('button', { name: 'Day of arrival Choose a day' }))
    await userEvent.click(await screen.findByRole('button', { name: /March 12th, 2027/ }))
    await userEvent.type(screen.getByLabelText('Time of arrival'), '18:00')
    await userEvent.click(screen.getByRole('radio', { name: 'I offer beds' }))
    await userEvent.type(screen.getByLabelText('How many?'), '2')
    await userEvent.click(screen.getByRole('button', { name: 'Save response' }))
    const status = await screen.findByText(/Response saved/)
    expect(status.closest('[role="status"]')?.textContent).toContain('Arrival Fri 18:00')
    expect(sent).toEqual({ status: 'accepted', arrival: '2027-03-12T18:00', departure: null, beds: 'offer', bedCount: 2, comment: null })
  })

  it('shows times of the instance whatever time zone the browser uses', async () => {
    openLink(() =>
      HttpResponse.json({ ...page, response: { status: 'accepted', arrival: '2027-03-12T18:00', departure: null, beds: 'none', bedCount: null, comment: null } }),
    )
    const time = (await screen.findByLabelText('Time of arrival')) as HTMLInputElement
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
})
