import { describe, expect, it } from 'vitest'
import { createBundle, formatDateRange, formatMessage, negotiateLocale } from './bundle'

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
