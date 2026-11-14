import { createContext, useContext } from 'react'
import type { Locale, MessageArgs } from './bundle'

export type Translate = (id: string, args?: MessageArgs) => string

export interface Localization {
  locale: Locale
  translate: Translate
  setLocale: (locale: Locale) => void
}

export const LocalizationContext = createContext<Localization>({
  locale: 'en',
  translate: (id) => id,
  setLocale: () => {},
})

export function useTranslate(): Translate {
  return useContext(LocalizationContext).translate
}

export function useLocale(): Pick<Localization, 'locale' | 'setLocale'> {
  const { locale, setLocale } = useContext(LocalizationContext)
  return { locale, setLocale }
}
