import { useId, useState, type FormEvent } from 'react'
import type { Beds, FieldError as ServerFieldError, ResponseStatus as Status, ResponseValues } from '@/api/generated/model'
import { DatePicker } from '@/components/DatePicker'
import { Button } from '@/components/ui/button'
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend, FieldSet } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { Textarea } from '@/components/ui/textarea'
import { formatWallClock, wallClock } from '@/l10n/dates'
import { useLocale, useTranslate, type Translate } from '@/l10n/translate'
import { carSummary } from '@/lib/car'
import { orFallback } from '@/lib/succeeded'

const formFields = ['status', 'arrival', 'departure', 'beds', 'bedCount', 'car', 'freeSeats', 'comment']
const statuses: Status[] = ['accepted', 'tentative', 'declined']
const bedChoices: Beds[] = ['need', 'offer', 'none']
const carChoices = ['yes', 'no'] as const

type Car = (typeof carChoices)[number]

export type SaveResult = { status: 'saved'; values: ResponseValues } | { status: 'invalid'; errors: ServerFieldError[] } | { status: 'failed' }

type Moment = {
  day: string
  time: string
}

function moment(value: string | null | undefined): Moment {
  const [day = '', time = ''] = value?.split('T') ?? []
  return { day, time }
}

function combined({ day, time }: Moment): string | null {
  return day && time ? `${day}T${time}` : null
}

function carChoice(car: boolean | null | undefined): Car | '' {
  if (car === true) return 'yes'
  if (car === false) return 'no'
  return ''
}

function requiredMessage(field: string): string {
  switch (field) {
    case 'bedCount':
      return 'response-error-bed-count'
    case 'car':
      return 'response-error-car'
    case 'freeSeats':
      return 'response-error-free-seats'
    default:
      return 'response-error-beds'
  }
}

function message(translate: Translate, error: ServerFieldError): string {
  if (error.field === 'freeSeats' && error.code === 'invalid') return translate('response-error-free-seats')
  switch (error.code) {
    case 'required':
      return translate(requiredMessage(error.field))
    case 'tooLarge':
      return translate('response-error-bed-count-too-large')
    case 'tooLong':
      return translate('response-error-comment-too-long')
    case 'beforeArrival':
      return translate('response-error-departure-before-arrival')
    case 'invalid':
      return translate('response-error-time')
    case 'begun':
      return translate('response-error-begun')
    case 'deadlinePassed':
      return translate('response-error-deadline-passed')
    case 'ended':
      return translate('invitation-state-ended')
    case 'cancelled':
      return translate('invitation-state-cancelled')
    default:
      return translate('error-unexpected')
  }
}

/** The questions of a response in their order: status, arrival and departure, beds with their count, car with its free seats and comment. With `declineOnly` it offers only the status declined and the comment. */
export function ResponseForm({
  question,
  firstDay,
  lastDay,
  initial,
  saveNotice,
  declineOnly = false,
  onSave,
}: {
  question: string
  firstDay: string
  lastDay: string
  initial: ResponseValues | null | undefined
  saveNotice: string
  declineOnly?: boolean
  onSave: (values: ResponseValues) => Promise<SaveResult>
}) {
  const id = useId()
  const translate = useTranslate()
  const { locale } = useLocale()
  const [status, setStatus] = useState<Status>(declineOnly ? 'declined' : (initial?.status ?? 'accepted'))
  const [arrival, setArrival] = useState<Moment>(moment(initial?.arrival))
  const [departure, setDeparture] = useState<Moment>(moment(initial?.departure))
  const [beds, setBeds] = useState<Beds | ''>(initial?.beds ?? '')
  const [bedCount, setBedCount] = useState(initial?.bedCount ? String(initial.bedCount) : '')
  const [car, setCar] = useState<Car | ''>(carChoice(initial?.car))
  const [freeSeats, setFreeSeats] = useState(typeof initial?.freeSeats === 'number' ? String(initial.freeSeats) : '')
  const [comment, setComment] = useState(initial?.comment ?? '')
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [saved, setSaved] = useState<ResponseValues | null>(null)
  const [busy, setBusy] = useState(false)
  const coming = status !== 'declined'
  const days = Math.round((wallClock(`${lastDay}T00:00`).getTime() - wallClock(`${firstDay}T00:00`).getTime()) / 86_400_000) + 1

  function incomplete(): Record<string, string> {
    const found: Record<string, string> = {}
    if (!coming) return found
    for (const [field, value] of [['arrival', arrival], ['departure', departure]] as const) {
      if (Boolean(value.day) !== Boolean(value.time)) found[field] = translate('response-error-day-and-time')
    }
    if (beds === '') found.beds = translate('response-error-beds')
    if ((beds === 'need' || beds === 'offer') && !/^(?:[1-9]|1\d|20)$/.test(bedCount.trim())) {
      found.bedCount = translate('response-error-bed-count')
    }
    if (car === '') found.car = translate('response-error-car')
    if (car === 'yes' && !/^[0-8]$/.test(freeSeats.trim())) found.freeSeats = translate('response-error-free-seats')
    return found
  }

  async function submit(event: FormEvent) {
    event.preventDefault()
    setSaved(null)
    const found = incomplete()
    if (Object.keys(found).length === 0) {
      setBusy(true)
      const values: ResponseValues = {
        status,
        arrival: coming ? combined(arrival) : null,
        departure: coming ? combined(departure) : null,
        beds: coming && beds !== '' ? beds : null,
        bedCount: coming && (beds === 'need' || beds === 'offer') ? Number(bedCount) : null,
        car: coming && car !== '' ? car === 'yes' : null,
        freeSeats: coming && car === 'yes' ? Number(freeSeats) : null,
        comment: comment.trim() === '' ? null : comment,
      }
      const result = await orFallback(() => onSave(values), { status: 'failed' })
      setBusy(false)
      if (result.status === 'saved') {
        setErrors({})
        setSaved(result.values)
        return
      }
      if (result.status === 'invalid') {
        for (const serverError of result.errors) {
          found[formFields.includes(serverError.field) ? serverError.field : 'form'] = message(translate, serverError)
        }
      } else {
        found.form = translate('error-unexpected')
      }
    }
    setErrors(found)
    const first = formFields.find((field) => found[field])
    if (first) document.getElementById(`${id}-${first}`)?.focus()
  }

  function error(field: string) {
    return <FieldError id={`${id}-${field}-error`} errors={errors[field] ? [{ message: errors[field] }] : []} />
  }

  function described(field: string, hint?: string) {
    const ids = [hint, errors[field] ? `${id}-${field}-error` : undefined].filter(Boolean).join(' ')
    return { 'aria-invalid': errors[field] ? true : undefined, 'aria-describedby': ids || undefined }
  }

  function when(field: 'arrival' | 'departure', value: Moment, change: (next: Moment) => void) {
    const dayLabel = translate(`response-${field}-day`)
    return (
      <FieldSet data-invalid={errors[field] ? true : undefined} className="min-w-0">
        <FieldLegend variant="label">{translate(`response-${field}`)}</FieldLegend>
        <div className="grid grid-cols-[repeat(auto-fit,minmax(min(100%,8.5rem),1fr))] gap-2">
          <Field>
            <FieldLabel id={`${id}-${field}-label`} htmlFor={`${id}-${field}`} className="sr-only">
              {dayLabel}
            </FieldLabel>
            <DatePicker
              id={`${id}-${field}`}
              labelId={`${id}-${field}-label`}
              value={value.day}
              onChange={(day) => change({ ...value, day })}
              gridLabel={dayLabel}
              navigationLabel={translate(`response-${field}-months`)}
              displayFormat={{ weekday: 'short', day: 'numeric', month: 'long' }}
              firstDay={firstDay}
              lastDay={lastDay}
              {...described(field)}
            />
          </Field>
          <Field>
            <FieldLabel htmlFor={`${id}-${field}-time`} className="sr-only">
              {translate(`response-${field}-time`)}
            </FieldLabel>
            <Input
              id={`${id}-${field}-time`}
              type="time"
              step={1800}
              value={value.time}
              onChange={(event) => change({ ...value, time: event.target.value.slice(0, 5) })}
              className="appearance-none [&::-webkit-calendar-picker-indicator]:hidden"
              {...described(field)}
            />
          </Field>
        </div>
        {error(field)}
      </FieldSet>
    )
  }

  const rowClass =
    'has-data-checked:border-primary has-data-checked:bg-primary/10 has-data-checked:shadow-[inset_0_0_0_1px_var(--primary)]'

  return (
    <form noValidate onSubmit={submit} aria-label={question}>
      <FieldGroup>
        <FieldSet>
          <FieldLegend className="text-base font-semibold">{question}</FieldLegend>
          <RadioGroup id={`${id}-status`} value={status} onValueChange={(value) => setStatus(value as Status)}>
            {(declineOnly ? (['declined'] as Status[]) : statuses).map((option) => (
              <FieldLabel key={option} htmlFor={`${id}-status-${option}`} className={rowClass}>
                <Field orientation="horizontal">
                  <RadioGroupItem
                    id={`${id}-status-${option}`}
                    value={option}
                    aria-labelledby={`${id}-status-${option}-label`}
                  />
                  <span id={`${id}-status-${option}-label`} className="font-semibold">
                    {translate(`response-status-${option}`)}
                  </span>
                </Field>
              </FieldLabel>
            ))}
          </RadioGroup>
        </FieldSet>

        {coming && (
          <FieldSet>
            <FieldLegend className="text-base font-semibold">{translate('response-when')}</FieldLegend>
            <div className="grid gap-4 sm:grid-cols-2">
              {when('arrival', arrival, setArrival)}
              {when('departure', departure, setDeparture)}
            </div>
            <FieldDescription>{translate('response-when-hint', { days })}</FieldDescription>
          </FieldSet>
        )}

        {coming && (
          <FieldSet data-invalid={errors.beds ? true : undefined}>
            <FieldLegend className="text-base font-semibold">{translate('response-beds')}</FieldLegend>
            <RadioGroup
              id={`${id}-beds`}
              value={beds}
              onValueChange={(value) => setBeds(value as Beds)}
              {...described('beds')}
            >
              {bedChoices.map((option) => (
                <FieldLabel key={option} htmlFor={`${id}-beds-${option}`} className={rowClass}>
                  <Field orientation="horizontal">
                    <RadioGroupItem
                      id={`${id}-beds-${option}`}
                      value={option}
                      aria-labelledby={`${id}-beds-${option}-label`}
                    />
                    <span id={`${id}-beds-${option}-label`} className="font-semibold">
                      {translate(`response-beds-${option}`)}
                    </span>
                  </Field>
                </FieldLabel>
              ))}
            </RadioGroup>
            {error('beds')}
            {(beds === 'need' || beds === 'offer') && (
              <Field orientation="horizontal" data-invalid={errors.bedCount ? true : undefined}>
                <FieldLabel htmlFor={`${id}-bedCount`}>{translate('response-bed-count')}</FieldLabel>
                <Input
                  id={`${id}-bedCount`}
                  type="number"
                  inputMode="numeric"
                  min={1}
                  max={20}
                  className="w-20"
                  value={bedCount}
                  onChange={(event) => setBedCount(event.target.value)}
                  {...described('bedCount')}
                />
              </Field>
            )}
            {error('bedCount')}
          </FieldSet>
        )}

        {coming && (
          <FieldSet data-invalid={errors.car ? true : undefined}>
            <FieldLegend className="text-base font-semibold">{translate('response-car')}</FieldLegend>
            <FieldDescription id={`${id}-car-hint`}>{translate('response-car-hint')}</FieldDescription>
            <RadioGroup
              id={`${id}-car`}
              value={car}
              onValueChange={(value) => setCar(value as Car)}
              {...described('car', `${id}-car-hint`)}
            >
              {carChoices.map((option) => (
                <FieldLabel key={option} htmlFor={`${id}-car-${option}`} className={rowClass}>
                  <Field orientation="horizontal">
                    <RadioGroupItem
                      id={`${id}-car-${option}`}
                      value={option}
                      aria-labelledby={`${id}-car-${option}-label`}
                    />
                    <span id={`${id}-car-${option}-label`} className="font-semibold">
                      {translate(`response-car-${option}`)}
                    </span>
                  </Field>
                </FieldLabel>
              ))}
            </RadioGroup>
            {error('car')}
            {car === 'yes' && (
              <Field orientation="horizontal" data-invalid={errors.freeSeats ? true : undefined}>
                <FieldLabel htmlFor={`${id}-freeSeats`}>{translate('response-free-seats')}</FieldLabel>
                <Input
                  id={`${id}-freeSeats`}
                  type="number"
                  inputMode="numeric"
                  min={0}
                  max={8}
                  required
                  className="w-20"
                  value={freeSeats}
                  onChange={(event) => setFreeSeats(event.target.value)}
                  {...described('freeSeats')}
                />
              </Field>
            )}
            {error('freeSeats')}
          </FieldSet>
        )}

        <Field data-invalid={errors.comment ? true : undefined}>
          <FieldLabel htmlFor={`${id}-comment`}>{translate('response-comment')}</FieldLabel>
          <FieldDescription id={`${id}-comment-hint`}>{translate('response-comment-hint')}</FieldDescription>
          <Textarea
            id={`${id}-comment`}
            rows={3}
            maxLength={2000}
            value={comment}
            onChange={(event) => setComment(event.target.value)}
            {...described('comment', `${id}-comment-hint`)}
          />
          {error('comment')}
        </Field>

        <Button type="submit" size="lg" className="w-full" disabled={busy}>
          {translate('response-save')}
        </Button>
        <div role="status" className={saved ? 'rounded-lg bg-muted px-3 py-2 text-sm' : ''}>
          {saved && (
            <>
              <p>{saveNotice}</p>
              <p>
                {translate(`response-status-${saved.status}`)}
                {saved.arrival ? `, ${translate('response-arrival')} ${formatWallClock(saved.arrival, locale)}` : ''}
                {saved.departure ? `, ${translate('response-departure')} ${formatWallClock(saved.departure, locale)}` : ''}
                {saved.beds === 'need' || saved.beds === 'offer'
                  ? `, ${translate(`overview-beds-${saved.beds}`, { count: saved.bedCount ?? 0 })}`
                  : ''}
                {typeof saved.car === 'boolean' ? `, ${carSummary(translate, saved)}` : ''}
              </p>
            </>
          )}
        </div>
        <p role="alert">{errors.form ?? ''}</p>
      </FieldGroup>
    </form>
  )
}
