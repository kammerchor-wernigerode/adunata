import type { Gathering, OverviewRow } from '@/api/generated/model'
import { formatWallClock, wallClock } from '@/l10n/dates'
import { useLocale, useTranslate } from '@/l10n/translate'

/** Shows the stated period as text with a bar across the days of the gathering. Without a period the member is present throughout. */
export function Period({ gathering, row }: { gathering: Gathering; row: OverviewRow }) {
  const translate = useTranslate()
  const { locale } = useLocale()
  if (row.status === 'declined' || row.status === 'pending') return <span>–</span>
  const start = wallClock(`${gathering.firstDay}T00:00`)
  const end = wallClock(`${gathering.lastDay}T00:00`)
  end.setDate(end.getDate() + 1)
  const span = end.getTime() - start.getTime()
  const days = Math.round(span / 86_400_000)
  const share = (value: string | null | undefined, fallback: number) =>
    value ? Math.min(100, Math.max(0, ((wallClock(value).getTime() - start.getTime()) / span) * 100)) : fallback
  const from = share(row.arrival, 0)
  const to = share(row.departure, 100)
  const text =
    row.arrival || row.departure
      ? translate('overview-period', {
          arrival: row.arrival ? formatWallClock(row.arrival, locale) : translate('overview-period-start'),
          departure: row.departure ? formatWallClock(row.departure, locale) : translate('overview-period-end'),
        })
      : translate('overview-whole-gathering')
  return (
    <span className="inline-flex flex-wrap items-center gap-x-2">
      <span aria-hidden="true" className="relative flex h-2 w-18 shrink-0 gap-0.5 overflow-hidden rounded-full">
        {Array.from({ length: days }, (_, day) => (
          <span key={day} className="h-full flex-1 bg-muted" />
        ))}
        <span className="absolute inset-y-0 rounded-full bg-muted-foreground" style={{ left: `${from}%`, right: `${100 - to}%` }} />
      </span>
      <span>{text}</span>
      {row.outsideSpan && <span className="font-semibold">{translate('overview-outside-span')}</span>}
    </span>
  )
}
