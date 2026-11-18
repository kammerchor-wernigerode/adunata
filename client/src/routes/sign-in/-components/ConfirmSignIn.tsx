import { useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { signIn } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { useTranslate } from '@/l10n/translate'
import { isUnauthorized } from '@/lib/expiredSession'

export function ConfirmSignIn({ secret, onRequestNew }: { secret: string; onRequestNew: () => void }) {
  const translate = useTranslate()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [invalid, setInvalid] = useState(false)

  async function confirm() {
    const result = await signIn({ headers: { Authorization: `Bearer ${secret}` } })
    if (result.status !== 201) {
      setInvalid(true)
      return
    }
    window.history.replaceState(null, '', window.location.pathname)
    await queryClient.resetQueries({ predicate: (query) => isUnauthorized(query.state.data) })
    await navigate({ to: '/gatherings' })
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
            <Button onClick={confirm}>{translate('sign-in-confirm')}</Button>
          </>
        )}
      </div>
    </>
  )
}
