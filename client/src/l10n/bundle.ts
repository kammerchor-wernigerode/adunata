import { FluentBundle, FluentResource, type FluentVariable } from '@fluent/bundle'

export const supportedLocales = ['en', 'de'] as const
export type Locale = (typeof supportedLocales)[number]
export const fallbackLocale: Locale = 'en'

export type MessageArgs = Record<string, FluentVariable>

const storageKey = 'adunata.locale'

export function isLocale(value: string | null): value is Locale {
  return supportedLocales.some((locale) => locale === value)
}

export function negotiateLocale(preferred: readonly string[]): Locale {
  for (const tag of preferred) {
    const language = tag.toLowerCase().split('-')[0]
    if (isLocale(language)) return language
  }
  return fallbackLocale
}

export function readStoredLocale(): Locale | null {
  try {
    const stored = localStorage.getItem(storageKey)
    return isLocale(stored) ? stored : null
  } catch {
    return null
  }
}

export function storeLocale(locale: Locale): void {
  try {
    localStorage.setItem(storageKey, locale)
  } catch {
    // The choice then lasts only until the page reloads.
  }
}

export function createBundle(locale: Locale, catalog: string): FluentBundle {
  const bundle = new FluentBundle(locale, { useIsolating: false })
  bundle.addResource(new FluentResource(catalog))
  return bundle
}

/** Formats a message from the first bundle that has it, so the fallback catalog fills gaps. */
export function formatMessage(bundles: readonly FluentBundle[], id: string, args?: MessageArgs): string {
  for (const bundle of bundles) {
    const message = bundle.getMessage(id)
    if (message?.value) return bundle.formatPattern(message.value, args)
  }
  return id
}

/** Formats a range of calendar days, given as ISO dates, the way readers of the locale expect. */
export function formatDateRange(locale: Locale, start: string, end: string): string {
  const format = new Intl.DateTimeFormat(locale, { day: 'numeric', month: 'long', year: 'numeric', timeZone: 'UTC' })
  return format.formatRange(new Date(`${start}T00:00:00Z`), new Date(`${end}T00:00:00Z`))
}
