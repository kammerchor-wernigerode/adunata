// @vitest-environment jsdom
import { cleanup, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { setupServer } from 'msw/node'
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest'
import {
  getCurrentSessionMockHandler,
  getLegalLinksMockHandler,
  getListMembersMockHandler,
} from '@/api/generated/api.msw'
import type { Member } from '@/api/generated/model'
import { renderApp, violations } from '@/testing/app'

const server = setupServer()

beforeAll(() => server.listen({ onUnhandledFrame: 'error' }))
afterEach(() => {
  server.resetHandlers()
  cleanup()
})
afterAll(() => server.close())

const ada: Member = {
  id: 1,
  firstName: 'Ada',
  lastName: 'Lovelace',
  mail: 'ada@example.org',
  language: 'en',
  active: true,
  organizer: true,
}
const clara: Member = { ...ada, id: 2, firstName: 'Clara', lastName: 'Schumann', mail: 'clara@example.org', organizer: false }
const johannes: Member = {
  ...ada,
  id: 3,
  firstName: 'Johannes',
  lastName: 'Brahms',
  mail: 'jb@example.org',
  active: false,
  organizer: false,
}

function signedIn(members: Member[] = [johannes, ada, clara]) {
  server.use(getLegalLinksMockHandler({}), getCurrentSessionMockHandler(ada), getListMembersMockHandler(members))
}

function names(): string[] {
  return within(screen.getByRole('table'))
    .getAllByRole('link')
    .map((link) => link.textContent)
}

describe('member list', () => {
  it('searches by part of a last name ignoring case', async () => {
    signedIn()
    const { container } = renderApp('/members')
    await screen.findByRole('link', { name: 'Lovelace, Ada' })
    expect(await violations(container)).toEqual([])
    await userEvent.type(screen.getByLabelText('Search by name or mail address'), 'SCHU')
    expect(names()).toEqual(['Schumann, Clara'])
    expect(screen.getByRole('status').textContent).toBe('One member')
  })

  it('filters active and inactive members', async () => {
    signedIn()
    renderApp('/members')
    await screen.findByRole('link', { name: 'Lovelace, Ada' })
    await userEvent.click(screen.getByRole('radio', { name: 'Inactive' }))
    expect(names()).toEqual(['Brahms, Johannes'])
    await userEvent.click(screen.getByRole('radio', { name: 'Active' }))
    expect(names()).toEqual(['Lovelace, Ada', 'Schumann, Clara'])
  })

  it('names a failed load of the list instead of showing an empty list', async () => {
    server.use(getLegalLinksMockHandler({}), getCurrentSessionMockHandler(ada), http.get('*/api/members', () => new HttpResponse(null, { status: 500 })))
    renderApp('/members')
    expect(await screen.findByRole('heading', { level: 1, name: 'Page unavailable' })).toBeTruthy()
    expect(screen.queryByText('No member matches.')).toBeNull()
  })

  it('names the organizer pages in the navigation and marks the current one', async () => {
    signedIn()
    renderApp('/members')
    const navigation = await screen.findByRole('navigation', { name: 'Main navigation' })
    expect(within(navigation).getByRole('link', { name: 'Members' }).getAttribute('aria-current')).toBe('page')
    expect(within(navigation).getByRole('button', { name: 'Sign out' })).toBeTruthy()
  })
})

describe('member form', () => {
  it('names the address a new member collides with at the mail field', async () => {
    signedIn()
    server.use(
      http.post('*/api/members', () =>
        HttpResponse.json(
          { errors: [{ field: 'mail', code: 'taken', conflict: 'Ada@Example.org' }] },
          { status: 422 },
        ),
      ),
    )
    const { container } = renderApp('/members/new')
    await userEvent.type(await screen.findByLabelText('First name'), 'Augusta')
    await userEvent.type(screen.getByLabelText('Last name'), 'King')
    await userEvent.type(screen.getByLabelText('Mail address'), 'ada@example.org')
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    const mail = screen.getByLabelText('Mail address')
    await waitFor(() => expect(mail.getAttribute('aria-invalid')).toBe('true'))
    expect(document.activeElement).toBe(mail)
    const message = document.getElementById(mail.getAttribute('aria-describedby') ?? '')
    expect(message?.textContent).toBe('The address Ada@Example.org is already taken.')
    expect(await violations(container)).toEqual([])
  })

  it('explains why the last organizer cannot lose the role', async () => {
    signedIn()
    server.use(
      http.put('*/api/members/1', () =>
        HttpResponse.json({ errors: [{ field: 'organizer', code: 'lastOrganizer' }] }, { status: 422 }),
      ),
    )
    const { container } = renderApp('/members/1')
    await userEvent.click(await screen.findByLabelText('Organizer'))
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    expect(await screen.findByText('The instance needs at least one active organizer.')).toBeTruthy()
    expect(await violations(container)).toEqual([])
  })

  it('asks before it deletes a member', async () => {
    signedIn()
    let deleted = false
    server.use(
      http.delete('*/api/members/2', () => {
        deleted = true
        return new HttpResponse(null, { status: 204 })
      }),
    )
    const { router } = renderApp('/members/2')
    await userEvent.click(await screen.findByRole('button', { name: 'Delete member' }))
    expect(screen.getByText('Delete Clara Schumann with all invitations, responses and waiting mail?')).toBeTruthy()
    expect(deleted).toBe(false)
    await userEvent.click(screen.getByRole('button', { name: 'Delete' }))
    await waitFor(() => expect(router.state.location.pathname).toBe('/members'))
    expect(deleted).toBe(true)
  })
})
