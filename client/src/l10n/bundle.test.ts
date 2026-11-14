import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { createBundle, formatDateRange, formatMessage, negotiateLocale, type Locale } from './bundle'

function beds(locale: Locale, count: number): string {
  const catalog = readFileSync(new URL(`../../../server/locales/${locale}.ftl`, import.meta.url), 'utf8')
  return formatMessage([createBundle(locale, catalog)], 'beds-needed', { beds: count })
}

describe('catalog served by the server', () => {
  it('renders English plural forms', () => {
    expect([0, 1, 2].map((count) => beds('en', count))).toEqual(['0 beds needed.', 'One bed needed.', '2 beds needed.'])
  })

  it('renders German plural forms', () => {
    expect([0, 1, 2].map((count) => beds('de', count))).toEqual([
      '0 Betten benötigt.',
      'Ein Bett benötigt.',
      '2 Betten benötigt.',
    ])
  })
})

describe('negotiateLocale', () => {
  it('picks the first supported language', () => {
    expect(negotiateLocale(['fr-FR', 'de-AT', 'en'])).toBe('de')
  })

  it('falls back to English', () => {
    expect(negotiateLocale(['fr'])).toBe('en')
  })
})

describe('missing translation', () => {
  it('falls back to the English text', () => {
    const german = createBundle('de', 'greeting = Hallo\n')
    const english = createBundle('en', 'greeting = Hello\nonly-english = Only in English\n')
    expect(formatMessage([german, english], 'greeting')).toBe('Hallo')
    expect(formatMessage([german, english], 'only-english')).toBe('Only in English')
  })
})

describe('date range', () => {
  it('puts the day before the month in German', () => {
    expect(formatDateRange('de', '2027-03-14', '2027-03-16')).toBe('14.–16. März 2027')
  })

  it('puts the month first in English', () => {
    expect(formatDateRange('en', '2027-03-14', '2027-03-16')).toBe('March 14\u2009–\u200916, 2027')
  })
})
