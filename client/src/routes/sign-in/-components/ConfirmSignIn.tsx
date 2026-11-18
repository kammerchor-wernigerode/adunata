import { useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { getCurrentSessionQueryKey, signIn } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { useSubmission } from '@/hooks/useSubmission'
import { useTranslate } from '@/l10n/translate'

export function ConfirmSignIn({ secret, onRequestNew }: { secret: string; onRequestNew: () => void }) {
  const translate = useTranslate()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [invalid, setInvalid] = useState(false)

  const { pending, failed, submit } = useSubmission()

  function confirm() {
    return submit(async () => {
      const result: { status: number } = await signIn({ headers: { Authorization: `Bearer ${secret}` } })
      if (result.status === 401) {
        setInvalid(true)
        return true
      }
      if (result.status !== 201) return false
      window.history.replaceState(null, '', window.location.pathname)
      queryClient.removeQueries({ type: 'inactive' })
      await queryClient.resetQueries({ queryKey: getCurrentSessionQueryKey() })
      await navigate({ to: '/gatherings' })
      return true
    })
  }

  function requestNew() {
    window.history.replaceState(null, '', window.location.pathname)
    onRequestNew()
  }

  return (
    <>
      <PageTitle title={translate('sign-in-confirm-title')} />
      <div className="flex max-w-md flex-col items-start gap-4">
        {invalid ? (
          <>
            <p role="alert">{translate('sign-in-invalid')}</p>
            <Button onClick={requestNew}>{translate('sign-in-request-new')}</Button>
          </>
        ) : (
          <>
            <p>{translate('sign-in-confirm-text')}</p>
            <Button disabled={pending} onClick={confirm}>
              {translate('sign-in-confirm')}
            </Button>
            <p role="alert">{failed ? translate('error-unexpected') : ''}</p>
          </>
        )}
      </div>
    </>
  )
}
