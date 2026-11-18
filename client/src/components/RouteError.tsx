import { useRouter } from '@tanstack/react-router'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'

/** Shown in place of a page whose data could not load, for example while the server is unreachable. */
export function RouteError() {
  const translate = useTranslate()
  const router = useRouter()
  return (
    <>
      <PageTitle title={translate('error-page-title')} />
      <p role="alert" className="mb-4">
        {translate('error-unexpected')}
      </p>
      <Button onClick={() => router.invalidate()}>{translate('error-retry')}</Button>
    </>
  )
}
