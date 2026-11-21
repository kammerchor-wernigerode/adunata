import type { ReactNode } from 'react'
import { useTranslate } from '@/l10n/translate'

/** The main heading of a route, which also names the browser tab. The root layout focuses it after a route change. A page action sits beside it at the top right and wraps below it on a narrow screen. */
export function PageTitle({ title, action }: { title: string; action?: ReactNode }) {
  const translate = useTranslate()
  return (
    <>
      <title>{`${title} · ${translate('product-name')}`}</title>
      <div className="mb-4 flex flex-wrap items-start justify-between gap-4">
        <h1 tabIndex={-1} className="min-w-0 font-heading text-2xl leading-tight font-semibold break-words outline-none focus-visible:ring-3 focus-visible:ring-ring sm:text-[1.65rem]">
          {title}
        </h1>
        {action}
      </div>
    </>
  )
}
