import { useQueryClient } from '@tanstack/react-query'
import { Navigate, useNavigate, useParams } from '@tanstack/react-router'
import { useId, useState } from 'react'
import { useForm, type FieldPath } from 'react-hook-form'
import {
  changeGathering,
  createGathering,
  getGetGatheringQueryKey,
  getListGatheringsQueryKey,
  useGetGathering,
} from '@/api/generated/api'
import type { FieldError as ServerFieldError, Gathering, GatheringInput } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { Field, FieldError, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Page } from '@/layout/Page'
import { useTranslate, type Translate } from '@/l10n/translate'

const fields = ['title', 'location', 'description', 'firstDay', 'lastDay'] as const

function message(translate: Translate, error: ServerFieldError): string {
  switch (error.code) {
    case 'required':
      return translate('error-field-required')
    case 'tooLong':
      return translate('error-field-too-long')
    case 'beforeFirstDay':
      return translate('error-last-day-before-first-day')
    case 'ended':
      return translate('error-gathering-ended')
    default:
      return translate('error-unexpected')
  }
}

export function NewGatheringPage() {
  const translate = useTranslate()
  return (
    <Page title={translate('gathering-new-title')}>
      <GatheringForm />
    </Page>
  )
}

export function EditGatheringPage() {
  const translate = useTranslate()
  const { gatheringId } = useParams({ from: '/organizer/gatherings/$gatheringId/edit' })
  const gathering = useGetGathering(Number(gatheringId))
  if (gathering.data?.status === 401) return <Navigate to="/sign-in" />
  if (gathering.data?.status === 404) return <Navigate to="/gatherings" />
  if (gathering.data?.status !== 200) return null
  return (
    <Page title={translate('gathering-edit-title', { title: gathering.data.data.title })}>
      <GatheringForm gathering={gathering.data.data} />
    </Page>
  )
}

function GatheringForm({ gathering }: { gathering?: Gathering }) {
  const id = useId()
  const translate = useTranslate()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [refusal, setRefusal] = useState('')
  const form = useForm<GatheringInput>({
    defaultValues: gathering
      ? {
          title: gathering.title,
          location: gathering.location,
          description: gathering.description ?? '',
          firstDay: gathering.firstDay,
          lastDay: gathering.lastDay,
        }
      : { title: '', location: '', description: '', firstDay: '', lastDay: '' },
  })
  const { errors, isSubmitting } = form.formState

  async function save(values: GatheringInput) {
    setRefusal('')
    const result = gathering ? await changeGathering(gathering.id, values) : await createGathering(values)
    if (result.status === 200 || result.status === 201) {
      await queryClient.invalidateQueries({ queryKey: getListGatheringsQueryKey() })
      await queryClient.invalidateQueries({ queryKey: getGetGatheringQueryKey(result.data.id) })
      return navigate({ to: '/gatherings/$gatheringId', params: { gatheringId: String(result.data.id) } })
    }
    if (result.status === 401) return navigate({ to: '/sign-in' })
    if (result.status !== 422) return setRefusal(translate('error-unexpected'))
    for (const error of result.data.errors) {
      if ((fields as readonly string[]).includes(error.field)) {
        form.setError(error.field as FieldPath<GatheringInput>, { type: 'server', message: message(translate, error) })
      } else {
        setRefusal(message(translate, error))
      }
    }
    const first = fields.find((field) => result.data.errors.some((error) => error.field === field))
    if (first) form.setFocus(first)
  }

  function field(name: FieldPath<GatheringInput>, label: string, input: React.ReactNode) {
    return (
      <Field data-invalid={errors[name] ? true : undefined}>
        <FieldLabel htmlFor={`${id}-${name}`}>{label}</FieldLabel>
        {input}
        <FieldError id={`${id}-${name}-error`} errors={errors[name] ? [{ message: errors[name]?.message }] : []} />
      </Field>
    )
  }

  function control(name: FieldPath<GatheringInput>) {
    return {
      id: `${id}-${name}`,
      'aria-invalid': errors[name] ? true : undefined,
      'aria-describedby': errors[name] ? `${id}-${name}-error` : undefined,
      ...form.register(name),
    }
  }

  return (
    <form noValidate onSubmit={form.handleSubmit(save)} className="max-w-xl">
      <FieldGroup>
        {field('title', translate('gathering-title'), <Input autoComplete="off" {...control('title')} />)}
        {field('location', translate('gathering-location'), <Input autoComplete="off" {...control('location')} />)}
        {field(
          'description',
          translate('gathering-description'),
          <textarea
            rows={4}
            className="w-full min-w-0 rounded-lg border border-input bg-transparent px-2.5 py-1.5 text-base outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring md:text-sm"
            {...control('description')}
          />,
        )}
        <div className="grid gap-4 sm:grid-cols-2">
          {field('firstDay', translate('gathering-first-day'), <Input type="date" {...control('firstDay')} />)}
          {field('lastDay', translate('gathering-last-day'), <Input type="date" {...control('lastDay')} />)}
        </div>
        <div className="flex flex-wrap gap-3">
          <Button type="submit" disabled={isSubmitting}>
            {translate('gathering-save')}
          </Button>
          <Button type="button" variant="outline" onClick={() => navigate({ to: '/gatherings' })}>
            {translate('member-cancel')}
          </Button>
        </div>
        <p role="alert">{refusal}</p>
      </FieldGroup>
    </form>
  )
}
