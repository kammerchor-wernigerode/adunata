import { useTranslate } from '@/l10n/translate'

/** The main heading of a route, which also names the browser tab. The root layout focuses it after a route change. */
export function PageTitle({ title }: { title: string }) {
  const translate = useTranslate()
  return (
    <>
      <title>{`${title} · ${translate('product-name')}`}</title>
      <h1 tabIndex={-1} className="mb-4 font-heading text-2xl leading-tight font-semibold break-words outline-none focus-visible:ring-3 focus-visible:ring-ring sm:text-[1.65rem]">
        {title}
      </h1>
    </>
  )
}
