// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { cleanup, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { setupServer } from 'msw/node'
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest'
import { getCreateResponseMockHandler } from '@/api/generated/api.msw'
import { EnglishLocalization } from '@/l10n/testing'
import { SpikeForm } from './SpikeForm'

const server = setupServer()

beforeAll(() => server.listen({ onUnhandledFrame: 'error' }))
afterEach(() => {
  server.resetHandlers()
  cleanup()
})
afterAll(() => server.close())

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
}

function renderForm() {
  render(
    <QueryClientProvider client={new QueryClient()}>
      <EnglishLocalization>
        <SpikeForm />
      </EnglishLocalization>
    </QueryClientProvider>,
  )
}

async function submit(name: string) {
  await userEvent.type(screen.getByLabelText('Name'), name)
  await userEvent.click(screen.getByRole('button', { name: 'Save' }))
}

describe('spike form against the API mocks', () => {
  it('shows client-side validation errors from the catalog', async () => {
    renderForm()
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    expect(await screen.findByText('Enter a name.')).toBeTruthy()
  })

  it('confirms a stored response', async () => {
    server.use(getCreateResponseMockHandler((info) => info.request.json()))
    renderForm()
    await submit('Ada')
    expect(await screen.findByText('Saved response of Ada.')).toBeTruthy()
  })

  it('shows a field error from the server at the field', async () => {
    server.use(
      http.post('*/api/spike-responses', () =>
        HttpResponse.json({ errors: [{ field: 'name', code: 'taken' }] }, { status: 422 }),
      ),
    )
    renderForm()
    await submit('Ada')
    expect(await screen.findByText('This name is already taken.')).toBeTruthy()
    expect(screen.getByLabelText('Name').getAttribute('aria-invalid')).toBe('true')
  })
})
