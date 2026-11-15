import { Outlet, useRouterState } from '@tanstack/react-router'
import { useEffect, useRef } from 'react'
import { useLegalLinks } from '@/api/generated/api'
import { LanguageSwitcher } from '@/l10n/LanguageSwitcher'
import { useTranslate } from '@/l10n/translate'

/** Header, main area and footer around every page. Moves focus to the main heading after each route change. */
export function Frame() {
  const translate = useTranslate()
  const main = useRef<HTMLElement>(null)
  const pathname = useRouterState({ select: (state) => state.resolvedLocation?.pathname })
  const shownPathname = useRef(pathname)

  useEffect(() => {
    if (shownPathname.current === pathname) return
    const firstPage = shownPathname.current === undefined
    shownPathname.current = pathname
    if (!firstPage) main.current?.querySelector<HTMLElement>('h1')?.focus()
  }, [pathname])

  return (
    <div className="flex min-h-screen flex-col">
      <header className="bg-primary text-primary-foreground">
        <div className="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-4 px-4 py-3">
          <span className="font-heading text-xl font-semibold">{translate('product-name')}</span>
          <LanguageSwitcher />
        </div>
      </header>
      <main ref={main} className="mx-auto w-full max-w-5xl flex-1 px-4 py-6">
        <Outlet />
      </main>
      <LegalFooter />
    </div>
  )
}

function LegalFooter() {
  const translate = useTranslate()
  const links = useLegalLinks().data?.data
  const entries = [
    { url: links?.privacyUrl, label: translate('privacy-notice') },
    { url: links?.imprintUrl, label: translate('imprint') },
  ].filter((entry): entry is { url: string; label: string } => Boolean(entry.url))
  if (entries.length === 0) return null
  return (
    <footer className="border-t border-border">
      <ul className="mx-auto flex max-w-5xl flex-wrap gap-6 px-4 py-4 text-sm">
        {entries.map((entry) => (
          <li key={entry.url}>
            <a href={entry.url} className="underline underline-offset-4">
              {entry.label}
            </a>
          </li>
        ))}
      </ul>
    </footer>
  )
}
