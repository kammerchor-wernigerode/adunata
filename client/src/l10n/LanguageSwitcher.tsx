import { useId } from 'react'
import { Label } from '@/components/ui/label'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { isLocale, supportedLocales } from './bundle'
import { useLocale, useTranslate } from './translate'

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
        {supportedLocales.map((language) => (
          <NativeSelectOption key={language} value={language} lang={language}>
            {translate(`language-name-${language}`)}
          </NativeSelectOption>
        ))}
      </NativeSelect>
    </div>
  )
}
