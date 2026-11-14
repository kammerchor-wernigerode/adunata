import { useQuery } from '@tanstack/react-query'
import { useEffect, useMemo, type ReactNode } from 'react'
import { createBundle, formatMessage, negotiateLocale, type Locale } from './bundle'
import { TranslateContext, type Translate } from './translate'

async function fetchCatalog(locale: Locale): Promise<string> {
  const response = await fetch(`/api/catalogs/${locale}`)
  if (!response.ok) throw new Error(`Catalog ${locale} is unavailable`)
  return response.text()
}

export function LocalizationProvider({ children }: { children: ReactNode }) {
  const locale = negotiateLocale(navigator.languages)
  const catalog = useQuery({ queryKey: ['catalog', locale], queryFn: () => fetchCatalog(locale), staleTime: Infinity })
  useEffect(() => {
    document.documentElement.lang = locale
  }, [locale])
  const bundle = useMemo(() => (catalog.data ? createBundle(locale, catalog.data) : null), [locale, catalog.data])

  if (!bundle) return null
  const translate: Translate = (id, args) => formatMessage(bundle, id, args)
  return (
    <TranslateContext.Provider value={translate}>
      {children}
    </TranslateContext.Provider>
  )
}
