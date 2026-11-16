// @vitest-environment jsdom
import { cleanup, render } from '@testing-library/react'
import axe from 'axe-core'
import { afterEach, describe, expect, it } from 'vitest'
import { EnglishLocalization } from '@/l10n/testing'
import { StatusBadge, type ResponseStatus } from './StatusBadge'

afterEach(cleanup)

const texts: Record<ResponseStatus, string> = {
  accepted: 'Accepted',
  tentative: 'Tentative',
  declined: 'Declined',
  pending: 'Pending',
}

describe('status badge', () => {
  for (const [status, text] of Object.entries(texts) as [ResponseStatus, string][]) {
    it(`names ${status} by its text alone`, async () => {
      const { container } = render(
        <EnglishLocalization>
          <ul>
            <li>
              <StatusBadge status={status} />
            </li>
          </ul>
        </EnglishLocalization>,
      )
      const badge = container.querySelector('li > span')!
      const results = await axe.run(container)
      expect(results.violations.map((violation) => violation.id)).toEqual([])
      expect(badge.querySelector('svg')?.getAttribute('aria-hidden')).toBe('true')
      expect(badge.textContent).toBe(text)
      axe.setup(container)
      expect(axe.commons.text.accessibleText(badge as HTMLElement)).toBe(text)
      axe.teardown()
    })
  }
})
