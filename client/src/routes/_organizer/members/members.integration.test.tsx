// @vitest-environment jsdom
import { act, fireEvent, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  getCurrentSessionMockHandler,
  getLegalLinksMockHandler,
  getListMembersMockHandler,
} from '@/api/generated/api.msw'
import type { Member } from '@/api/generated/model'
import { renderApp, violations } from '@/testing/app'
import { setupApiServer } from '@/testing/server'

const server = setupApiServer()

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

function countStatus(): HTMLElement {
  return screen.getAllByRole('status')[0]!
}

function copyStatus(): HTMLElement {
  return screen.getAllByRole('status')[1]!
}

function copyButton(row: string): HTMLElement {
  return within(screen.getByRole('row', { name: new RegExp(row) })).getByRole('button')
}

function names(): string[] {
  return within(screen.getByRole('table'))
    .getAllByRole('link')
    .map((link) => link.textContent)
}

describe('member list', () => {
  afterEach(() => {
    vi.useRealTimers()
    vi.restoreAllMocks()
  })

  it('searches by part of a last name ignoring case', async () => {
    signedIn()
    const { container } = renderApp('/members')
    await screen.findByRole('link', { name: 'Lovelace, Ada' })
    expect(await violations(container)).toEqual([])
    await userEvent.type(screen.getByLabelText('Search by name or mail address'), 'SCHU')
    expect(names()).toEqual(['Schumann, Clara'])
    expect(countStatus().textContent).toBe('One member')
  })

  it('offers to add a member in the page header before the search field', async () => {
    signedIn()
    renderApp('/members')
    const heading = await screen.findByRole('heading', { level: 1, name: 'Members' })
    const add = screen.getByRole('link', { name: 'Add member' })
    expect(heading.contains(add)).toBe(false)
    const controls = [...screen.getByRole('main').querySelectorAll('a, button, input')]
    expect(controls.slice(0, 2)).toEqual([add, screen.getByLabelText('Search by name or mail address')])
    expect(heading.compareDocumentPosition(add) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
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

  it('copies the mail address of a member and confirms it', async () => {
    signedIn()
    const user = userEvent.setup()
    const { container } = renderApp('/members')
    await screen.findByRole('link', { name: 'Lovelace, Ada' })
    await user.click(copyButton('Lovelace, Ada'))
    expect(await navigator.clipboard.readText()).toBe('ada@example.org')
    const button = copyButton('Lovelace, Ada')
    expect(button.getAttribute('aria-label')).toBe('Mail address copied')
    expect(button.querySelector('.lucide-check')).not.toBeNull()
    expect(copyStatus().textContent).toBe('Mail address copied')
    expect(countStatus().textContent).toBe('3 members')
    expect(await violations(container)).toEqual([])
  })

  it('ends the confirmation after two seconds', async () => {
    signedIn()
    userEvent.setup()
    renderApp('/members')
    await screen.findByRole('link', { name: 'Lovelace, Ada' })
    vi.useFakeTimers()
    const button = copyButton('Lovelace, Ada')
    fireEvent.click(button)
    await act(() => vi.advanceTimersByTimeAsync(0))
    expect(button.getAttribute('aria-label')).toBe('Mail address copied')
    await act(() => vi.advanceTimersByTimeAsync(1999))
    expect(button.getAttribute('aria-label')).toBe('Mail address copied')
    await act(() => vi.advanceTimersByTimeAsync(1))
    expect(button.getAttribute('aria-label')).toBe('Copy mail address')
    expect(button.querySelector('.lucide-copy')).not.toBeNull()
    expect(copyStatus().textContent).toBe('')
  })

  it.each([
    [{ firstName: 'Ada', lastName: 'Lovelace, Countess' }],
    [{ firstName: 'Ada "Bird"', lastName: 'Lovelace' }],
  ])('copies only the mail address for the name %o', async (name) => {
    signedIn([{ ...ada, ...name }])
    const user = userEvent.setup()
    renderApp('/members')
    await user.click(await screen.findByRole('button', { name: 'Copy mail address' }))
    expect(await navigator.clipboard.readText()).toBe('ada@example.org')
  })

  it('offers to copy the mail address of an inactive member', async () => {
    signedIn()
    renderApp('/members')
    await screen.findByRole('link', { name: 'Brahms, Johannes' })
    expect(copyButton('Brahms, Johannes').getAttribute('aria-label')).toBe('Copy mail address')
  })

  it.each([
    ['rejects the write', () => vi.spyOn(navigator.clipboard, 'writeText').mockRejectedValue(new DOMException('denied', 'NotAllowedError'))],
    ['offers no clipboard', () => vi.spyOn(navigator, 'clipboard', 'get').mockReturnValue(undefined as unknown as Clipboard)],
  ])('names a failed copy when the browser %s', async (_, breakClipboard) => {
    signedIn()
    const user = userEvent.setup()
    const { container } = renderApp('/members')
    await screen.findByRole('link', { name: 'Lovelace, Ada' })
    breakClipboard()
    await user.click(copyButton('Lovelace, Ada'))
    expect(copyStatus().textContent).toBe('Could not copy the mail address. Select and copy it by hand.')
    const button = copyButton('Lovelace, Ada')
    expect(button.getAttribute('aria-label')).toBe('Copy mail address')
    expect(button.querySelector('.lucide-copy')).not.toBeNull()
    expect(countStatus().textContent).toBe('3 members')
    expect(await violations(container)).toEqual([])
  })

  it('reports only the result of the latest copy', async () => {
    signedIn()
    const user = userEvent.setup()
    renderApp('/members')
    await screen.findByRole('link', { name: 'Lovelace, Ada' })
    let finishFirst: (() => void) | undefined
    vi.spyOn(navigator.clipboard, 'writeText')
      .mockReturnValueOnce(new Promise<void>((resolve) => (finishFirst = resolve)))
      .mockRejectedValueOnce(new DOMException('denied', 'NotAllowedError'))
    await user.click(copyButton('Lovelace, Ada'))
    await user.click(copyButton('Schumann, Clara'))
    await act(async () => finishFirst?.())
    expect(copyStatus().textContent).toBe('Could not copy the mail address. Select and copy it by hand.')
    expect(copyButton('Lovelace, Ada').getAttribute('aria-label')).toBe('Copy mail address')
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
    expect(within(navigation).queryAllByRole('button')).toEqual([])
    expect(within(navigation).getAllByRole('link').map((link) => link.textContent)).toEqual(['Gatherings', 'Members'])
    expect(screen.getByRole('button', { name: 'Account of Ada Lovelace' }).textContent).toBe('Ada Lovelace')
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
