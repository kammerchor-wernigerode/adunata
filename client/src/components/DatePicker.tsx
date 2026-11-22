import { format, isAfter, isBefore, isSameDay, parseISO } from 'date-fns'
import { CalendarDays } from 'lucide-react'
import type { ComponentProps } from 'react'
import { de, enUS } from 'react-day-picker/locale'
import { Button } from '@/components/ui/button'
import { Calendar } from '@/components/ui/calendar'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { formatDay } from '@/l10n/dates'
import { useLocale, useTranslate } from '@/l10n/translate'

type DatePickerProps = Omit<ComponentProps<typeof Button>, 'id' | 'value' | 'onChange'> & {
  id: string
  labelId: string
  value: string
  onChange: (value: string) => void
  gridLabel: string
  navigationLabel: string
  displayFormat: Intl.DateTimeFormatOptions
  firstDay?: string
  lastDay?: string
}

/** A button that names an ISO day such as 2027-03-14 and opens a calendar to choose another. The day is a local date, so it never moves between time zones. The calendar disables days before `firstDay` and after `lastDay`, except the chosen day. */
export function DatePicker({
  id,
  labelId,
  value,
  onChange,
  gridLabel,
  navigationLabel,
  displayFormat,
  firstDay,
  lastDay,
  ...trigger
}: DatePickerProps) {
  const translate = useTranslate()
  const { locale } = useLocale()
  const selected = value ? parseISO(value) : undefined
  const first = firstDay ? parseISO(firstDay) : undefined
  const last = lastDay ? parseISO(lastDay) : undefined

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          id={id}
          type="button"
          variant="outline"
          className="w-full justify-between border-input font-normal"
          aria-labelledby={`${labelId} ${id}-value`}
          {...trigger}
        >
          <span id={`${id}-value`}>{value ? formatDay(value, locale, displayFormat) : translate('date-picker-choose-day')}</span>
          <CalendarDays aria-hidden="true" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0" align="start">
        <Calendar
          mode="single"
          required
          locale={locale === 'de' ? de : enUS}
          weekStartsOn={1}
          defaultMonth={selected ?? first}
          selected={selected}
          onSelect={(day) => onChange(format(day, 'yyyy-MM-dd'))}
          disabled={(day) => !(selected && isSameDay(day, selected)) && ((!!first && isBefore(day, first)) || (!!last && isAfter(day, last)))}
          labels={{ labelGrid: () => gridLabel, labelNav: () => navigationLabel }}
        />
      </PopoverContent>
    </Popover>
  )
}
