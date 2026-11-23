import { useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate } from '@tanstack/react-router'
import { addDays, differenceInCalendarDays, format, parseISO } from 'date-fns'
import { useEffect, useId, useState, type ReactNode } from 'react'
import { Controller, useForm, useWatch, type FieldPath } from 'react-hook-form'
import {
  changeGathering,
  createGathering,
  getGetGatheringQueryKey,
  getListGatheringsQueryKey,
} from '@/api/generated/api'
import type { FieldError as ServerFieldError, Gathering, GatheringInput } from '@/api/generated/model'
import { DatePicker } from '@/components/DatePicker'
import { Alert } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Field, FieldError, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { formatSpan } from '@/l10n/dates'
import { useLocale, useTranslate, type Translate } from '@/l10n/translate'
import { finished } from '@/lib/succeeded'

const fields = ['title', 'location', 'description', 'firstDay', 'lastDay'] as const

function message(translate: Translate, error: ServerFieldError): string {
  switch (error.code) {
    case 'required':
      return translate('error-field-required')
    case 'tooLong':
      return translate('error-field-too-long')
    case 'beforeFirstDay':
      return translate('error-last-day-before-first-day')
    case 'firstDayInPast':
      return translate('error-first-day-in-past')
    case 'ended':
      return translate('error-gathering-ended')
    default:
      return translate('error-unexpected')
  }
}

/** The form that creates a gathering or, given one, changes it. */
export function GatheringForm({ gathering }: { gathering?: Gathering }) {
  const id = useId()
  const translate = useTranslate()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { locale } = useLocale()
  const [refusal, setRefusal] = useState('')
  const [saved, setSaved] = useState<Gathering>()
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
  const chosenFirstDay = useWatch({ control: form.control, name: 'firstDay' })
  const [today] = useState(() => format(new Date(), 'yyyy-MM-dd'))

  useEffect(() => {
    if (saved) form.setFocus('title')
  }, [saved, form])

  function saveOrRefuse(addAnother: boolean) {
    return async (values: GatheringInput) => {
      if (!(await finished(() => save(values, addAnother)))) setRefusal(translate('error-unexpected'))
    }
  }

  async function save(values: GatheringInput, addAnother: boolean) {
    setRefusal('')
    setSaved(undefined)
    const result = gathering ? await changeGathering(gathering.id, values) : await createGathering(values)
    if (result.status === 200 || result.status === 201) {
      await queryClient.invalidateQueries({ queryKey: getListGatheringsQueryKey() })
      await queryClient.invalidateQueries({ queryKey: getGetGatheringQueryKey(result.data.id) })
      if (addAnother) return startNext(values, result.data)
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
    const first = fields.find((name) => result.data.errors.some((error) => error.field === name))
    if (first) form.setFocus(first)
  }

  function startNext(values: GatheringInput, stored: Gathering) {
    form.reset({ ...values, firstDay: '', lastDay: '' })
    setSaved(stored)
  }

  function savedLine(stored: Gathering) {
    const marker = '\u0000'
    const [before, after] = translate('gathering-saved', {
      title: marker,
      span: formatSpan(stored.firstDay, stored.lastDay, locale),
    }).split(marker)
    return (
      <p>
        {before}
        <Link to="/gatherings/$gatheringId" params={{ gatheringId: String(stored.id) }} className="underline underline-offset-4">
          {stored.title}
        </Link>
        {after}
      </p>
    )
  }

  function field(name: FieldPath<GatheringInput>, label: string, input: ReactNode) {
    return (
      <Field data-invalid={errors[name] ? true : undefined}>
        <FieldLabel id={`${id}-${name}-label`} htmlFor={`${id}-${name}`}>
          {label}
        </FieldLabel>
        {input}
        <FieldError id={`${id}-${name}-error`} errors={errors[name] ? [{ message: errors[name].message }] : []} />
      </Field>
    )
  }

  function moveLastDay(firstDay: string) {
    const { firstDay: previous, lastDay } = form.getValues()
    if (!previous || !lastDay) return
    const moved = addDays(parseISO(lastDay), differenceInCalendarDays(parseISO(firstDay), parseISO(previous)))
    form.setValue('lastDay', format(moved, 'yyyy-MM-dd'))
  }

  function described(name: FieldPath<GatheringInput>) {
    return {
      'aria-invalid': errors[name] ? true : undefined,
      'aria-describedby': errors[name] ? `${id}-${name}-error` : undefined,
    }
  }

  function control(name: FieldPath<GatheringInput>) {
    return { id: `${id}-${name}`, ...described(name), ...form.register(name) }
  }

  function day(name: 'firstDay' | 'lastDay') {
    return (
      <Controller
        control={form.control}
        name={name}
        render={({ field: { ref, value, onChange } }) => (
          <DatePicker
            ref={ref}
            id={`${id}-${name}`}
            labelId={`${id}-${name}-label`}
            value={value}
            onChange={(chosen) => {
              if (name === 'firstDay') moveLastDay(chosen)
              onChange(chosen)
            }}
            firstDay={name === 'firstDay' ? today : chosenFirstDay || undefined}
            gridLabel={translate(name === 'firstDay' ? 'gathering-first-day' : 'gathering-last-day')}
            navigationLabel={translate(name === 'firstDay' ? 'gathering-first-day-months' : 'gathering-last-day-months')}
            displayFormat={{ weekday: 'short', day: 'numeric', month: 'long', year: 'numeric' }}
            {...described(name)}
          />
        )}
      />
    )
  }

  return (
    <form noValidate onSubmit={form.handleSubmit(saveOrRefuse(false))} className="max-w-xl">
      <FieldGroup>
        {field('title', translate('gathering-title'), <Input autoComplete="off" {...control('title')} />)}
        {field('location', translate('gathering-location'), <Input autoComplete="off" {...control('location')} />)}
        {field(
          'description',
          translate('gathering-description'),
          <Textarea rows={4} {...control('description')} />,
        )}
        <div className="grid gap-4 sm:grid-cols-2">
          {field('firstDay', translate('gathering-first-day'), day('firstDay'))}
          {field('lastDay', translate('gathering-last-day'), day('lastDay'))}
        </div>
        <div className="flex flex-wrap gap-3">
          <Button type="submit" disabled={isSubmitting}>
            {translate('gathering-save')}
          </Button>
          {!gathering && (
            <Button type="button" variant="outline" disabled={isSubmitting} onClick={form.handleSubmit(saveOrRefuse(true))}>
              {translate('gathering-save-next')}
            </Button>
          )}
          <Button type="button" variant="outline" onClick={() => navigate({ to: '/gatherings' })}>
            {translate('member-cancel')}
          </Button>
        </div>
        <Alert role="status" className={saved ? undefined : 'sr-only'}>
          {saved && savedLine(saved)}
        </Alert>
        <p role="alert">{refusal}</p>
      </FieldGroup>
    </form>
  )
}
