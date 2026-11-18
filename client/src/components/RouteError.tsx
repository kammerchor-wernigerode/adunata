import { useTranslate } from '@/l10n/translate'

/** Shown in place of a page whose data could not load, for example while the server is unreachable. */
export function RouteError() {
  const translate = useTranslate()
  return <p role="alert">{translate('error-unexpected')}</p>
}
