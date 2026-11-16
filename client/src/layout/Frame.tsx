import { useQueryClient } from '@tanstack/react-query'
import { Link, Outlet, useNavigate, useRouterState } from '@tanstack/react-router'
import { useEffect, useRef } from 'react'
import { signOut, useLegalLinks } from '@/api/generated/api'
import { useOrganizer } from '@/access/session'
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

function OrganizerNavigation() {
  const translate = useTranslate()
  const organizer = useOrganizer()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  if (!organizer) return null

  async function leave() {
    await signOut()
    queryClient.clear()
    await navigate({ to: '/sign-in' })
  }

  const item =
    'rounded-sm underline-offset-4 decoration-2 outline-none hover:underline focus-visible:ring-3 focus-visible:ring-primary-foreground aria-[current=page]:underline'
  return (
    <nav aria-label={translate('nav-label')}>
      <ul className="flex flex-wrap gap-4">
        <li>
          <Link to="/gatherings" className={item}>
            {translate('nav-gatherings')}
          </Link>
        </li>
        <li>
          <Link to="/members" className={item}>
            {translate('nav-members')}
          </Link>
        </li>
        <li>
          <button type="button" onClick={leave} className={item}>
            {translate('nav-sign-out')}
          </button>
        </li>
      </ul>
    </nav>
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
