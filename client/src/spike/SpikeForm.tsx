import { zodResolver } from '@hookform/resolvers/zod'
import { useId, useState } from 'react'
import { Controller, useForm, useWatch, type FieldError as FormFieldError, type FieldPath } from 'react-hook-form'
import { useCreateResponse } from '@/api/generated/api'
import { CreateResponseBody } from '@/api/generated/api.zod'
import type { ErrorCode, SpikeResponse } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend, FieldSet } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { useTranslate } from '@/l10n/translate'

const serverErrorMessages: Record<ErrorCode, string> = {
  required: 'error-required',
  tooLong: 'error-too-long',
  tooLarge: 'error-too-large',
  taken: 'error-taken',
}

const attendanceOptions = ['accepted', 'tentative', 'declined'] as const

/** Names the catalog message for a field error, whether the server or the generated schema reported it. */
function errorMessageId(field: FieldPath<SpikeResponse>, error: FormFieldError): string {
  if (error.type === 'server' && error.message) return error.message
  switch (error.type) {
    case 'too_small':
      return field === 'name' ? 'error-required' : 'error-invalid'
    case 'too_big':
      return field === 'name' ? 'error-too-long' : 'error-too-large'
    case 'invalid_type':
      return 'error-number'
    case 'invalid_format':
      return 'error-date'
    default:
      return 'error-invalid'
  }
}

export function SpikeForm() {
  const id = useId()
  const translate = useTranslate()
  const [stored, setStored] = useState<SpikeResponse | null>(null)
  const mutation = useCreateResponse()
  const form = useForm<SpikeResponse>({
    resolver: zodResolver(CreateResponseBody),
    defaultValues: { name: '', attendance: 'accepted', arrival: null, beds: 0 },
  })
  const { errors, isSubmitting } = form.formState
  const beds = useWatch({ control: form.control, name: 'beds' })

  function errorsOf(field: FieldPath<SpikeResponse>) {
    const error = errors[field as keyof typeof errors] as FormFieldError | undefined
    return error ? [{ message: translate(errorMessageId(field, error)) }] : []
  }

  async function submit(values: SpikeResponse) {
    const result = await mutation.mutateAsync({ data: values })
    if (result.status === 201) {
      setStored(result.data)
      return
    }
    for (const error of result.data.errors) {
      form.setError(error.field as FieldPath<SpikeResponse>, {
        type: 'server',
        message: serverErrorMessages[error.code],
      })
    }
  }

  return (
    <form noValidate onSubmit={form.handleSubmit(submit)} className="max-w-md">
      <FieldGroup>
        <Field data-invalid={errors.name ? true : undefined}>
          <FieldLabel htmlFor={`${id}-name`}>{translate('spike-name')}</FieldLabel>
          <Input
            id={`${id}-name`}
            autoComplete="name"
            aria-invalid={errors.name ? true : undefined}
            aria-describedby={errors.name ? `${id}-name-error` : undefined}
            {...form.register('name')}
          />
          <FieldError id={`${id}-name-error`} errors={errorsOf('name')} />
        </Field>

        <FieldSet>
          <FieldLegend>{translate('spike-attendance')}</FieldLegend>
          <Controller
            control={form.control}
            name="attendance"
            render={({ field }) => (
              <RadioGroup value={field.value} onValueChange={field.onChange} name={field.name}>
                {attendanceOptions.map((option) => (
                  <Field key={option} orientation="horizontal">
                    <RadioGroupItem id={`${id}-attendance-${option}`} value={option} />
                    <FieldLabel htmlFor={`${id}-attendance-${option}`}>
                      {translate(`spike-attendance-${option}`)}
                    </FieldLabel>
                  </Field>
                ))}
              </RadioGroup>
            )}
          />
        </FieldSet>

        <Field data-invalid={errors.arrival ? true : undefined}>
          <FieldLabel htmlFor={`${id}-arrival`}>{translate('spike-arrival')}</FieldLabel>
          <Input
            id={`${id}-arrival`}
            type="date"
            aria-invalid={errors.arrival ? true : undefined}
            aria-describedby={errors.arrival ? `${id}-arrival-error` : undefined}
            {...form.register('arrival', { setValueAs: (value: string) => (value === '' ? null : value) })}
          />
          <FieldError id={`${id}-arrival-error`} errors={errorsOf('arrival')} />
        </Field>

        <Field data-invalid={errors.beds ? true : undefined}>
          <FieldLabel htmlFor={`${id}-beds`}>{translate('spike-beds')}</FieldLabel>
          <Input
            id={`${id}-beds`}
            type="number"
            min={0}
            max={20}
            aria-invalid={errors.beds ? true : undefined}
            aria-describedby={errors.beds ? `${id}-beds-error` : undefined}
            {...form.register('beds', { valueAsNumber: true })}
          />
          <FieldDescription aria-live="polite">
            {Number.isInteger(beds) ? translate('beds-needed', { beds }) : ''}
          </FieldDescription>
          <FieldError id={`${id}-beds-error`} errors={errorsOf('beds')} />
        </Field>

        <Button type="submit" disabled={isSubmitting}>
          {translate('spike-save')}
        </Button>
      </FieldGroup>

      <p role="status" className="mt-4">
        {stored ? translate('spike-saved', { name: stored.name }) : ''}
      </p>
    </form>
  )
}
