import { useId } from 'react'
import { Label } from '@/components/ui/label'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { isLocale, type Locale } from './bundle'
import { useLocale, useTranslate } from './translate'

const languageNames: Record<Locale, string> = { de: 'Deutsch', en: 'English' }

export function LanguageSwitcher() {
  const id = useId()
  const translate = useTranslate()
  const { locale, setLocale } = useLocale()
  return (
    <div className="flex items-center gap-2">
      <Label htmlFor={id}>{translate('language-label')}</Label>
      <NativeSelect
        id={id}
        size="sm"
        value={locale}
        onChange={(event) => isLocale(event.target.value) && setLocale(event.target.value)}
        className="rounded-lg bg-background text-foreground"
      >
        {Object.entries(languageNames).map(([value, name]) => (
          <NativeSelectOption key={value} value={value} lang={value}>
            {name}
          </NativeSelectOption>
        ))}
      </NativeSelect>
    </div>
  )
}
