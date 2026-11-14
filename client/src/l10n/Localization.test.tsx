// @vitest-environment jsdom
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { cleanup, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { setupServer } from 'msw/node'
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it } from 'vitest'
import { LanguageSwitcher } from './LanguageSwitcher'
import { LocalizationProvider } from './Localization'

const server = setupServer(
  http.get('*/api/catalogs/:locale', ({ params }) =>
    HttpResponse.text(readFileSync(path.join(import.meta.dirname, `../../../server/locales/${String(params.locale)}.ftl`), 'utf8')),
  ),
)

beforeAll(() => server.listen({ onUnhandledFrame: 'error' }))
beforeEach(() => localStorage.clear())
afterEach(() => cleanup())
afterAll(() => server.close())

function preferLanguages(languages: string[]) {
  Object.defineProperty(navigator, 'languages', { value: languages, configurable: true })
}

function openClient() {
  return render(
    <QueryClientProvider client={new QueryClient()}>
      <LocalizationProvider>
        <LanguageSwitcher />
      </LocalizationProvider>
    </QueryClientProvider>,
  )
}

describe('language of the client', () => {
  it('shows German text in a browser that prefers German', async () => {
    preferLanguages(['de-DE', 'en'])
    openClient()
    expect(await screen.findByLabelText('Sprache')).toBeTruthy()
    expect(document.documentElement.lang).toBe('de')
  })

  it('shows English text in a browser that prefers only French', async () => {
    preferLanguages(['fr-FR'])
    openClient()
    expect(await screen.findByLabelText('Language')).toBeTruthy()
  })

  it('keeps a switched language after a reload', async () => {
    preferLanguages(['de-DE'])
    const first = openClient()
    await userEvent.selectOptions(await screen.findByLabelText('Sprache'), 'en')
    expect(await screen.findByLabelText('Language')).toBeTruthy()
    first.unmount()

    openClient()
    expect(await screen.findByLabelText('Language')).toBeTruthy()
  })
})
