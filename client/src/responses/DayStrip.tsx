import { useLocale, useTranslate } from '@/l10n/translate'

function days(firstDay: string, lastDay: string): Date[] {
  const [year, month, day] = firstDay.split('-').map(Number)
  const [lastYear, lastMonth, lastDayOfMonth] = lastDay.split('-').map(Number)
  const last = new Date(lastYear, lastMonth - 1, lastDayOfMonth)
  const result: Date[] = []
  for (let current = new Date(year, month - 1, day); current <= last; current.setDate(current.getDate() + 1)) {
    result.push(new Date(current))
  }
  return result
}

/** One tile per day of the gathering with weekday, date and month. */
export function DayStrip({ firstDay, lastDay }: { firstDay: string; lastDay: string }) {
  const translate = useTranslate()
  const { locale } = useLocale()
  const weekday = new Intl.DateTimeFormat(locale, { weekday: 'short' })
  const date = new Intl.DateTimeFormat(locale, { day: 'numeric' })
  const month = new Intl.DateTimeFormat(locale, { month: 'long' })
  const full = new Intl.DateTimeFormat(locale, { dateStyle: 'full' })
  return (
    <ul aria-label={translate('response-days')} className="mt-4 flex flex-wrap gap-2">
      {days(firstDay, lastDay).map((day) => (
        <li
          key={day.toISOString()}
          className="min-w-[4.5rem] flex-1 rounded-lg border border-border bg-card px-2 py-1.5 text-center text-sm text-muted-foreground"
        >
          <span className="sr-only">{full.format(day)}</span>
          <span aria-hidden="true">
            {weekday.format(day)}
            <b className="block font-heading text-xl font-semibold text-foreground">{date.format(day)}.</b>
            {month.format(day)}
          </span>
        </li>
      ))}
    </ul>
  )
}
