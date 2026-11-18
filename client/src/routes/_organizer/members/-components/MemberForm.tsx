import { useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { useId, useState } from 'react'
import { useForm, type FieldPath } from 'react-hook-form'
import { addMember, changeMember, getListMembersQueryKey } from '@/api/generated/api'
import type { FieldError as ServerFieldError, Member, MemberChange } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { useTranslate } from '@/l10n/translate'
import { DeleteMember } from './DeleteMember'
import { memberErrorMessage } from './memberErrorMessage'

const fields = ['firstName', 'lastName', 'mail', 'language', 'active', 'organizer'] as const

/** The form that adds a member or, given one, changes it. */
export function MemberForm({ member }: { member?: Member }) {
  const id = useId()
  const translate = useTranslate()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [refusal, setRefusal] = useState('')
  const form = useForm<MemberChange>({
    defaultValues: member ?? { firstName: '', lastName: '', mail: '', language: 'de', active: true, organizer: false },
  })
  const { errors, isSubmitting } = form.formState

  async function done() {
    await queryClient.invalidateQueries({ queryKey: getListMembersQueryKey() })
    await navigate({ to: '/members' })
  }

  function report(serverErrors: ServerFieldError[]) {
    for (const error of serverErrors) {
      const field = (fields as readonly string[]).includes(error.field) ? (error.field as FieldPath<MemberChange>) : null
      if (field) form.setError(field, { type: 'server', message: memberErrorMessage(translate, error) })
      else setRefusal(memberErrorMessage(translate, error))
    }
    const first = fields.find((field) => serverErrors.some((error) => error.field === field))
    if (first) form.setFocus(first)
  }

  async function save(values: MemberChange) {
    setRefusal('')
    const result = member ? await changeMember(member.id, values) : await addMember(values)
    if (result.status === 200 || result.status === 201) return done()
    if (result.status === 422) return report(result.data.errors)
    if (result.status === 401) return navigate({ to: '/sign-in' })
    setRefusal(translate('error-unexpected'))
  }

  function described(field: FieldPath<MemberChange>) {
    return {
      'aria-invalid': errors[field] ? true : undefined,
      'aria-describedby': errors[field] ? `${id}-${field}-error` : undefined,
    }
  }

  function errorOf(field: FieldPath<MemberChange>) {
    const error = errors[field]
    return <FieldError id={`${id}-${field}-error`} errors={error ? [{ message: error.message }] : []} />
  }

  return (
    <form noValidate onSubmit={form.handleSubmit(save)} className="max-w-xl">
      <FieldGroup>
        <div className="grid gap-4 sm:grid-cols-2">
          <Field data-invalid={errors.firstName ? true : undefined}>
            <FieldLabel htmlFor={`${id}-firstName`}>{translate('member-first-name')}</FieldLabel>
            <Input id={`${id}-firstName`} autoComplete="off" {...described('firstName')} {...form.register('firstName')} />
            {errorOf('firstName')}
          </Field>
          <Field data-invalid={errors.lastName ? true : undefined}>
            <FieldLabel htmlFor={`${id}-lastName`}>{translate('member-last-name')}</FieldLabel>
            <Input id={`${id}-lastName`} autoComplete="off" {...described('lastName')} {...form.register('lastName')} />
            {errorOf('lastName')}
          </Field>
        </div>
        <Field data-invalid={errors.mail ? true : undefined}>
          <FieldLabel htmlFor={`${id}-mail`}>{translate('member-mail')}</FieldLabel>
          <Input id={`${id}-mail`} type="email" autoComplete="off" {...described('mail')} {...form.register('mail')} />
          {errorOf('mail')}
        </Field>
        <Field>
          <FieldLabel htmlFor={`${id}-language`}>{translate('member-language')}</FieldLabel>
          <select
            id={`${id}-language`}
            className="h-8 w-fit rounded-lg border border-input bg-background px-2.5 text-base text-foreground md:text-sm"
            {...form.register('language')}
          >
            <option value="de" lang="de">
              {translate('language-name-de')}
            </option>
            <option value="en" lang="en">
              {translate('language-name-en')}
            </option>
          </select>
        </Field>
        {member && (
          <>
            <Field orientation="horizontal" data-invalid={errors.active ? true : undefined}>
              <input
                id={`${id}-active`}
                type="checkbox"
                className="size-4 accent-primary"
                {...described('active')}
                {...form.register('active')}
              />
              <FieldLabel htmlFor={`${id}-active`}>{translate('member-active')}</FieldLabel>
            </Field>
            {errorOf('active')}
            <Field orientation="horizontal" data-invalid={errors.organizer ? true : undefined}>
              <input
                id={`${id}-organizer`}
                type="checkbox"
                className="size-4 accent-primary"
                aria-describedby={`${id}-organizer-hint${errors.organizer ? ` ${id}-organizer-error` : ''}`}
                aria-invalid={errors.organizer ? true : undefined}
                {...form.register('organizer')}
              />
              <FieldLabel htmlFor={`${id}-organizer`}>{translate('member-organizer')}</FieldLabel>
            </Field>
            <FieldDescription id={`${id}-organizer-hint`}>{translate('member-organizer-hint')}</FieldDescription>
            {errorOf('organizer')}
          </>
        )}
        <div className="flex flex-wrap gap-3">
          <Button type="submit" disabled={isSubmitting}>
            {translate('member-save')}
          </Button>
          <Button type="button" variant="outline" onClick={() => navigate({ to: '/members' })}>
            {translate('member-cancel')}
          </Button>
        </div>
        <p role="alert">{refusal}</p>
      </FieldGroup>
      {member && <DeleteMember member={member} onDeleted={done} onRefused={setRefusal} />}
    </form>
  )
}
