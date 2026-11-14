import { readFileSync } from 'node:fs'
import path from 'node:path'
import type { ReactNode } from 'react'
import { createBundle, formatMessage } from './bundle'
import { LocalizationContext } from './translate'

const english = createBundle(
  'en',
  readFileSync(path.join(import.meta.dirname, '../../../server/locales/en.ftl'), 'utf8'),
)

/** Provides the English catalog to components under test without fetching it. */
export function EnglishLocalization({ children }: { children: ReactNode }) {
  return (
    <LocalizationContext.Provider
      value={{ locale: 'en', translate: (id, args) => formatMessage([english], id, args), setLocale: () => {} }}
    >
      {children}
    </LocalizationContext.Provider>
  )
}
