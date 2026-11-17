import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { createBundle, formatMessage, type Locale } from './bundle'

function beds(locale: Locale, count: number): string {
  const catalog = readFileSync(new URL(`../../../server/locales/${locale}.ftl`, import.meta.url), 'utf8')
  return formatMessage([createBundle(locale, catalog)], 'overview-beds-need', { count })
}

describe('catalog served by the server', () => {
  it('renders English plural forms', () => {
    expect([0, 1, 2].map((count) => beds('en', count))).toEqual(['needs 0', 'needs one', 'needs 2'])
  })

  it('renders German plural forms', () => {
    expect([0, 1, 2].map((count) => beds('de', count))).toEqual([
      'braucht 0',
      'braucht eins',
      'braucht 2',
    ])
  })
})
