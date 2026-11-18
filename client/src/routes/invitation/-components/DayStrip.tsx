import { eachDayOfInterval, parseISO } from 'date-fns'
import { useLocale, useTranslate } from '@/l10n/translate'

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
      {eachDayOfInterval({ start: parseISO(firstDay), end: parseISO(lastDay) }).map((day) => (
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
