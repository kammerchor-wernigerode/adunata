import { useId, useState, type FormEvent } from 'react'
import { requestSignIn } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { useSubmission } from '@/hooks/useSubmission'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { useTranslate } from '@/l10n/translate'

export function RequestSignIn() {
  const id = useId()
  const translate = useTranslate()
  const [mail, setMail] = useState('')
  const [sent, setSent] = useState(false)
  const { pending, failed, submit, reset } = useSubmission()

  function send(event: FormEvent) {
    event.preventDefault()
    return submit(async () => {
      const result: { status: number } = await requestSignIn({ mail })
      if (result.status !== 202) return false
      setSent(true)
      return true
    })
  }

  return (
    <>
      <PageTitle title={translate('sign-in-title')} />
      <form onSubmit={send} className="max-w-md">
        <FieldGroup>
          <Field>
            <FieldLabel htmlFor={`${id}-mail`}>{translate('sign-in-mail')}</FieldLabel>
            <Input
              id={`${id}-mail`}
              type="email"
              autoComplete="email"
              required
              value={mail}
              onChange={(event) => {
                setMail(event.target.value)
                setSent(false)
                reset()
              }}
            />
          </Field>
          <Button type="submit" disabled={pending}>
            {translate('sign-in-send')}
          </Button>
          <p role="status">{sent ? translate('sign-in-sent') : ''}</p>
          <p role="alert">{failed ? translate('error-unexpected') : ''}</p>
        </FieldGroup>
      </form>
    </>
  )
}
