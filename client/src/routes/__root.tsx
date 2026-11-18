import { createRootRouteWithContext, Outlet, useRouterState } from '@tanstack/react-router'
import { useEffect, useRef } from 'react'
import { LegalFooter } from '@/components/LegalFooter'
import { OrganizerNavigation } from '@/components/OrganizerNavigation'
import { LanguageSwitcher } from '@/l10n/LanguageSwitcher'
import { useTranslate } from '@/l10n/translate'
import type { RouterContext } from '@/router'

export const Route = createRootRouteWithContext<RouterContext>()({ component: RootLayout })

/** Header, main area and footer around every page. Moves focus to the main heading after each route change. */
function RootLayout() {
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
          <span className="font-heading text-lg font-semibold">{translate('product-name')}</span>
          <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
            <OrganizerNavigation />
            <LanguageSwitcher />
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
