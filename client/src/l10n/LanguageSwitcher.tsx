import { useId } from 'react'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
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
      <Select value={locale} onValueChange={(value) => isLocale(value) && setLocale(value)}>
        <SelectTrigger id={id} size="sm" className="bg-background text-foreground">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {Object.entries(languageNames).map(([value, name]) => (
            <SelectItem key={value} value={value} lang={value}>
              {name}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
