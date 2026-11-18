import { useId, useState, type FormEvent } from 'react'
import { requestSignIn } from '@/api/generated/api'
import { PageTitle } from '@/components/PageTitle'
import { Button } from '@/components/ui/button'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { useTranslate } from '@/l10n/translate'

export function RequestSignIn() {
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
    <>
      <PageTitle title={translate('sign-in-title')} />
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
    </>
  )
}
