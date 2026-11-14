import { useQuery } from '@tanstack/react-query'
import { useEffect, useMemo, useState, type ReactNode } from 'react'
import {
  createBundle,
  fallbackLocale,
  formatMessage,
  negotiateLocale,
  readStoredLocale,
  storeLocale,
  type Locale,
} from './bundle'
import { LocalizationContext, type Localization } from './translate'

async function fetchCatalog(locale: Locale): Promise<string> {
  const response = await fetch(`/api/catalogs/${locale}`)
  if (!response.ok) throw new Error(`Catalog ${locale} is unavailable`)
  return response.text()
}

function useCatalog(locale: Locale) {
  return useQuery({ queryKey: ['catalog', locale], queryFn: () => fetchCatalog(locale), staleTime: Infinity })
}

export function LocalizationProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(() => readStoredLocale() ?? negotiateLocale(navigator.languages))
  const catalog = useCatalog(locale)
  const fallback = useCatalog(fallbackLocale)
  useEffect(() => {
    document.documentElement.lang = locale
  }, [locale])
  const bundles = useMemo(
    () =>
      catalog.data && fallback.data
        ? [createBundle(locale, catalog.data), createBundle(fallbackLocale, fallback.data)]
        : null,
    [locale, catalog.data, fallback.data],
  )

  if (catalog.isError || fallback.isError) return <CatalogUnavailable />
  if (!bundles) return null
  const value: Localization = {
    locale,
    translate: (id, args) => formatMessage(bundles, id, args),
    setLocale: (next) => {
      storeLocale(next)
      setLocaleState(next)
    },
  }
  return <LocalizationContext.Provider value={value}>{children}</LocalizationContext.Provider>
}

/** Without a catalog no text can be translated, so this notice is the only text written into the client. */
function CatalogUnavailable() {
  return (
    <main role="alert" className="p-6">
      <p lang="de">Adunata kann seine Texte gerade nicht laden. Bitte lade die Seite neu.</p>
      <p lang="en">Adunata cannot load its texts right now. Please reload the page.</p>
    </main>
  )
}
