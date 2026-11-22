import { Outlet, useRouterState } from '@tanstack/react-router'
import { useEffect, useRef } from 'react'
import { AccountMenu } from '@/components/AccountMenu'
import { LegalFooter } from '@/components/LegalFooter'
import { OrganizerNavigation } from '@/components/OrganizerNavigation'
import { useOrganizer } from '@/hooks/useOrganizer'
import { LanguageSwitcher } from '@/l10n/LanguageSwitcher'
import { useTranslate } from '@/l10n/translate'

/** Header, main area and footer around every page. Moves focus to the main heading after each route change. */
export function RootLayout() {
  const translate = useTranslate()
  const organizer = useOrganizer()
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
          <span className="font-heading text-lg font-semibold">{translate('product-name')}</span>
          <div className="flex min-w-0 flex-wrap items-center gap-x-6 gap-y-2">
            {organizer && <OrganizerNavigation />}
            {organizer && <AccountMenu organizer={organizer} />}
            {organizer === null && <LanguageSwitcher />}
          </div>
        </div>
      </header>
      <main ref={main} className="mx-auto w-full max-w-5xl flex-1 px-4 py-5 sm:px-5">
        <Outlet />
      </main>
      <LegalFooter />
    </div>
  )
}
