import { useId } from 'react'
import { isLocale, type Locale } from './bundle'
import { useLocale, useTranslate } from './translate'

const languageNames: Record<Locale, string> = { de: 'Deutsch', en: 'English' }

export function LanguageSwitcher() {
  const id = useId()
  const translate = useTranslate()
  const { locale, setLocale } = useLocale()
  return (
    <div className="flex items-center gap-2">
      <label htmlFor={id}>{translate('language-label')}</label>
      <select
        id={id}
        value={locale}
        onChange={(event) => isLocale(event.target.value) && setLocale(event.target.value)}
        className="rounded border px-2 py-1"
      >
        {Object.entries(languageNames).map(([value, name]) => (
          <option key={value} value={value} lang={value}>
            {name}
          </option>
        ))}
      </select>
    </div>
  )
}
