import { FluentBundle, FluentResource, type FluentVariable } from '@fluent/bundle'

export const supportedLocales = ['en', 'de'] as const
export type Locale = (typeof supportedLocales)[number]

export type MessageArgs = Record<string, FluentVariable>

export function negotiateLocale(preferred: readonly string[]): Locale {
  for (const tag of preferred) {
    const language = tag.toLowerCase().split('-')[0]
    const match = supportedLocales.find((locale) => locale === language)
    if (match) return match
  }
  return 'en'
}

export function createBundle(locale: Locale, catalog: string): FluentBundle {
  const bundle = new FluentBundle(locale, { useIsolating: false })
  bundle.addResource(new FluentResource(catalog))
  return bundle
}

export function formatMessage(bundle: FluentBundle, id: string, args?: MessageArgs): string {
  const message = bundle.getMessage(id)
  if (!message?.value) return id
  return bundle.formatPattern(message.value, args)
}
