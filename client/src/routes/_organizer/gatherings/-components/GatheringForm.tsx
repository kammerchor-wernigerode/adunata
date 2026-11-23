import { useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate } from '@tanstack/react-router'
import { addDays, differenceInCalendarDays, format, parseISO } from 'date-fns'
import { useEffect, useId, useState, type BaseSyntheticEvent, type ReactNode } from 'react'
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
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend, FieldSet } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { formatSpan } from '@/l10n/dates'
import { useLocale, useTranslate, type Translate } from '@/l10n/translate'
import { finished } from '@/lib/succeeded'
import { deadlineDays, lastDayToAnswer, type DeadlineUnit } from './deadline'

const fields = ['title', 'location', 'description', 'firstDay', 'lastDay', 'responseDeadlineDays'] as const

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
    case 'invalid':
      return translate('error-deadline-invalid')
    case 'deadlineInPast':
      return translate('error-deadline-in-past')
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
  const [deadlineAmount, setDeadlineAmount] = useState(gathering?.responseDeadlineDays ? String(gathering.responseDeadlineDays) : '')
  const [deadlineUnit, setDeadlineUnit] = useState<DeadlineUnit>('days')
  const chosenDeadline = deadlineDays(deadlineAmount, deadlineUnit)

  useEffect(() => {
    if (saved) form.setFocus('title')
  }, [saved, form])

  function saveOrRefuse(addAnother: boolean) {
    return async (values: GatheringInput) => {
      const responseDeadlineDays = deadlineDays(deadlineAmount, deadlineUnit)
      if (responseDeadlineDays === undefined) {
        form.setError('responseDeadlineDays', { type: 'client', message: translate('error-deadline-invalid') })
        return
      }
      const input = { ...values, responseDeadlineDays }
      if (!(await finished(() => save(input, addAnother)))) setRefusal(translate('error-unexpected'))
    }
  }

  function submit(addAnother: boolean) {
    return (event?: BaseSyntheticEvent) => {
      form.clearErrors('responseDeadlineDays')
      return form.handleSubmit(saveOrRefuse(addAnother))(event)
    }
  }

  async function save(values: GatheringInput, addAnother: boolean) {
    setRefusal('')
    setSaved(undefined)
    const result = gathering ? await changeGathering(gathering.id, values) : await createGathering(values)
    if (result.status === 200 || result.status === 201) {
      await queryClient.invalidateQueries({ queryKey: getListGatheringsQueryKey() })
      await queryClient.invalidateQueries({ queryKey: getGetGatheringQueryKey(result.data.id) })
      if (addAnother) return startNext(result.data)
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
    if (first) document.getElementById(`${id}-${first}`)?.focus()
  }

  function startNext(stored: Gathering) {
    form.reset({ ...form.getValues(), firstDay: '', lastDay: '' })
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

  function deadline() {
    const name = 'responseDeadlineDays'
    const lastDay = chosenFirstDay ? lastDayToAnswer(chosenFirstDay, 1) : undefined
    return (
      <FieldSet data-invalid={errors[name] ? true : undefined}>
        <FieldLegend variant="label">{translate('gathering-deadline')}</FieldLegend>
        <FieldDescription id={`${id}-${name}-hint`}>{translate('gathering-deadline-hint')}</FieldDescription>
        <div className="flex flex-wrap items-center gap-2">
          <Input
            id={`${id}-${name}`}
            type="number"
            inputMode="numeric"
            min={1}
            step={1}
            className="w-20"
            aria-label={translate('gathering-deadline-amount')}
            value={deadlineAmount}
            onChange={(event) => setDeadlineAmount(event.target.value)}
            aria-invalid={errors[name] ? true : undefined}
            aria-describedby={[`${id}-${name}-hint`, errors[name] ? `${id}-${name}-error` : ''].filter(Boolean).join(' ')}
          />
          <ToggleGroup
            type="single"
            value={deadlineUnit}
            onValueChange={(value) => value && setDeadlineUnit(value as DeadlineUnit)}
            aria-label={translate('gathering-deadline-unit')}
          >
            <ToggleGroupItem value="days">{translate('gathering-deadline-days')}</ToggleGroupItem>
            <ToggleGroupItem value="weeks">{translate('gathering-deadline-weeks')}</ToggleGroupItem>
          </ToggleGroup>
          <span id={`${id}-${name}-day-label`} className="sr-only">
            {translate('gathering-deadline-last-day')}
          </span>
          <div className="min-w-56 flex-1">
            <DatePicker
              id={`${id}-${name}-day`}
              labelId={`${id}-${name}-day-label`}
              value={chosenFirstDay && chosenDeadline ? lastDayToAnswer(chosenFirstDay, chosenDeadline) : ''}
              onChange={(chosen) => {
                setDeadlineAmount(String(differenceInCalendarDays(parseISO(chosenFirstDay), parseISO(chosen)) - 1))
                setDeadlineUnit('days')
              }}
              firstDay={today}
              lastDay={lastDay}
              disabled={!chosenFirstDay}
              gridLabel={translate('gathering-deadline-last-day')}
              navigationLabel={translate('gathering-deadline-last-day-months')}
              displayFormat={{ weekday: 'short', day: 'numeric', month: 'long', year: 'numeric' }}
            />
          </div>
        </div>
        <FieldError id={`${id}-${name}-error`} errors={errors[name] ? [{ message: errors[name].message }] : []} />
      </FieldSet>
    )
  }

  return (
    <form noValidate onSubmit={submit(false)} className="max-w-xl">
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
        {deadline()}
        <div className="flex flex-wrap gap-3">
          <Button type="submit" disabled={isSubmitting}>
            {translate('gathering-save')}
          </Button>
          {!gathering && (
            <Button type="button" variant="outline" disabled={isSubmitting} onClick={submit(true)}>
              {translate('gathering-save-next')}
            </Button>
          )}
          <Button type="button" variant="outline" onClick={() => navigate({ to: '/gatherings' })}>
            {translate('member-cancel')}
          </Button>
        </div>
        <div role="status" className={saved ? undefined : 'sr-only'}>
          {saved && <Alert role={undefined}>{savedLine(saved)}</Alert>}
        </div>
        <p role="alert">{refusal}</p>
      </FieldGroup>
    </form>
  )
}
