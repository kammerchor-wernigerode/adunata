import { zodResolver } from '@hookform/resolvers/zod'
import { useId, useState } from 'react'
import { Controller, useForm, useWatch, type FieldPath } from 'react-hook-form'
import { useCreateResponse } from '@/api/generated/api'
import { CreateResponseBody } from '@/api/generated/api.zod'
import type { ErrorCode, SpikeResponse } from '@/api/generated/model'
import { Button } from '@/components/ui/button'
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend, FieldSet } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { useTranslate } from '@/l10n/translate'

const errorMessages: Record<ErrorCode, string> = {
  required: 'Enter a name.',
  tooLong: 'The name is too long.',
  tooLarge: 'The number is too large.',
  taken: 'This name is already taken.',
}

const attendanceOptions = [
  { value: 'accepted', label: 'Accepted' },
  { value: 'tentative', label: 'Tentative' },
  { value: 'declined', label: 'Declined' },
] as const

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

  async function submit(values: SpikeResponse) {
    const result = await mutation.mutateAsync({ data: values })
    if (result.status === 201) {
      setStored(result.data)
      return
    }
    for (const error of result.data.errors) {
      form.setError(error.field as FieldPath<SpikeResponse>, { message: errorMessages[error.code] })
    }
  }

  return (
    <form noValidate onSubmit={form.handleSubmit(submit)} className="max-w-md">
      <FieldGroup>
        <Field data-invalid={errors.name ? true : undefined}>
          <FieldLabel htmlFor={`${id}-name`}>Name</FieldLabel>
          <Input
            id={`${id}-name`}
            autoComplete="name"
            aria-invalid={errors.name ? true : undefined}
            aria-describedby={errors.name ? `${id}-name-error` : undefined}
            {...form.register('name')}
          />
          <FieldError id={`${id}-name-error`} errors={[errors.name]} />
        </Field>

        <FieldSet>
          <FieldLegend>Attendance</FieldLegend>
          <Controller
            control={form.control}
            name="attendance"
            render={({ field }) => (
              <RadioGroup value={field.value} onValueChange={field.onChange} name={field.name}>
                {attendanceOptions.map((option) => (
                  <Field key={option.value} orientation="horizontal">
                    <RadioGroupItem id={`${id}-attendance-${option.value}`} value={option.value} />
                    <FieldLabel htmlFor={`${id}-attendance-${option.value}`}>{option.label}</FieldLabel>
                  </Field>
                ))}
              </RadioGroup>
            )}
          />
        </FieldSet>

        <Field data-invalid={errors.arrival ? true : undefined}>
          <FieldLabel htmlFor={`${id}-arrival`}>Arrival</FieldLabel>
          <Input
            id={`${id}-arrival`}
            type="date"
            aria-invalid={errors.arrival ? true : undefined}
            aria-describedby={errors.arrival ? `${id}-arrival-error` : undefined}
            {...form.register('arrival', { setValueAs: (value: string) => (value === '' ? null : value) })}
          />
          <FieldError id={`${id}-arrival-error`} errors={[errors.arrival]} />
        </Field>

        <Field data-invalid={errors.beds ? true : undefined}>
          <FieldLabel htmlFor={`${id}-beds`}>Beds</FieldLabel>
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
          <FieldError id={`${id}-beds-error`} errors={[errors.beds]} />
        </Field>

        <Button type="submit" disabled={isSubmitting}>
          Save
        </Button>
      </FieldGroup>

      <p role="status" className="mt-4">
        {stored ? `Saved response of ${stored.name}.` : ''}
      </p>
    </form>
  )
}
