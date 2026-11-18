import { useQueryClient } from '@tanstack/react-query'
import { useRouter } from '@tanstack/react-router'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { useSubmission } from '@/hooks/useSubmission'
import { useTranslate } from '@/l10n/translate'
import { hasFailed } from '@/lib/dataOf'

/** Shown in place of a page whose data could not load, for example while the server is unreachable. */
export function RouteError() {
  const translate = useTranslate()
  const router = useRouter()
  const queryClient = useQueryClient()
  const { pending, submit } = useSubmission()

  function retry() {
    return submit(async () => {
      await queryClient.resetQueries({ predicate: (query) => hasFailed(query.state) })
      await router.invalidate()
      return true
    })
  }
  return (
    <>
      <PageTitle title={translate('error-page-title')} />
      <p role="alert" className="mb-4">
        {translate('error-unexpected')}
      </p>
      <Button disabled={pending} onClick={retry}>{translate('error-retry')}</Button>
    </>
  )
}
