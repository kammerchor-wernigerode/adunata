// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { cleanup, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import axe from 'axe-core'
import { afterEach, describe, expect, it } from 'vitest'
import { EnglishLocalization } from '@/l10n/testing'
import { SpikeForm } from './SpikeForm'

afterEach(cleanup)

globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
}

function renderForm() {
  return render(
    <QueryClientProvider client={new QueryClient()}>
      <EnglishLocalization>
        <SpikeForm />
      </EnglishLocalization>
    </QueryClientProvider>,
  )
}

async function violations(container: HTMLElement) {
  const results = await axe.run(container)
  return results.violations.map((violation) => `${violation.id}: ${violation.help}`)
}

describe('spike form accessibility', () => {
  it('has no violations when empty', async () => {
    const { container } = renderForm()
    expect(await violations(container)).toEqual([])
  })

  it('has no violations while showing errors', async () => {
    const { container } = renderForm()
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    await screen.findAllByRole('alert')
    expect(screen.getByLabelText('Name')).toHaveProperty('ariaInvalid', 'true')
    expect(await violations(container)).toEqual([])
  })
})
