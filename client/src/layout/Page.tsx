import { useEffect, type ReactNode } from 'react'
import { useTranslate } from '@/l10n/translate'

/** The main heading of a route, which also names the browser tab. The frame focuses it after a route change. */
export function Page({ title, children }: { title: string; children: ReactNode }) {
  const translate = useTranslate()
  const productName = translate('product-name')

  useEffect(() => {
    document.title = `${title} · ${productName}`
  }, [title, productName])

  return (
    <>
      <h1 tabIndex={-1} className="mb-6 font-heading text-3xl font-semibold outline-none focus-visible:ring-3 focus-visible:ring-ring">
        {title}
      </h1>
      {children}
    </>
  )
}
