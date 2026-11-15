import { useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { useId, useState, type FormEvent } from 'react'
import { getCurrentSessionQueryKey, requestSignIn, signIn } from '@/api/generated/api'
import { Button } from '@/components/ui/button'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Page } from '@/layout/Page'
import { useTranslate } from '@/l10n/translate'

/** The sign-in link carries its secret in the fragment, which the browser never sends to the server. */
function linkSecret(): string {
  return window.location.hash.slice(1)
}

export function SignInPage() {
  const [secret, setSecret] = useState(linkSecret)
  return secret ? <ConfirmSignIn secret={secret} onRequestNew={() => setSecret('')} /> : <RequestSignIn />
}

function RequestSignIn() {
  const id = useId()
  const translate = useTranslate()
  const [mail, setMail] = useState('')
  const [sent, setSent] = useState(false)

  async function submit(event: FormEvent) {
    event.preventDefault()
    await requestSignIn({ mail })
    setSent(true)
  }

  return (
    <Page title={translate('sign-in-title')}>
      <form onSubmit={submit} className="max-w-md">
        <FieldGroup>
          <Field>
            <FieldLabel htmlFor={`${id}-mail`}>{translate('sign-in-mail')}</FieldLabel>
            <Input
              id={`${id}-mail`}
              type="email"
              autoComplete="email"
              required
              value={mail}
              onChange={(event) => setMail(event.target.value)}
            />
          </Field>
          <Button type="submit">{translate('sign-in-send')}</Button>
          <p role="status">{sent ? translate('sign-in-sent') : ''}</p>
        </FieldGroup>
      </form>
    </Page>
  )
}

function ConfirmSignIn({ secret, onRequestNew }: { secret: string; onRequestNew: () => void }) {
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
    await queryClient.invalidateQueries({ queryKey: getCurrentSessionQueryKey() })
    await navigate({ to: '/members' })
  }

  function requestNew() {
    window.history.replaceState(null, '', window.location.pathname)
    onRequestNew()
  }

  return (
    <Page title={translate('sign-in-confirm-title')}>
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
    </Page>
  )
}
