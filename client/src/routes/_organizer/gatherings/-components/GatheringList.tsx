import { Link } from '@tanstack/react-router'
import type { Gathering } from '@/api/generated/model'
import { Badge } from '@/components/ui/badge'
import { formatSpan } from '@/l10n/dates'
import { useLocale, useTranslate } from '@/l10n/translate'

export function GatheringList({ gatherings }: { gatherings: Gathering[] }) {
  const translate = useTranslate()
  const { locale } = useLocale()
  return (
    <ul className="divide-y divide-border overflow-hidden rounded-lg border border-border">
      {gatherings.map((gathering) => (
        <li key={gathering.id} className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 px-4 py-3">
          <div className="min-w-0">
            <Link
              to="/gatherings/$gatheringId"
              params={{ gatheringId: String(gathering.id) }}
              className="font-semibold text-foreground underline underline-offset-4"
            >
              {gathering.title}
            </Link>
            <p className="text-sm text-muted-foreground">
              {formatSpan(gathering.firstDay, gathering.lastDay, locale)}, {gathering.location}
            </p>
          </div>
          {gathering.cancelled && (
            <Badge variant="secondary" className="h-auto bg-muted px-2.5 text-sm font-semibold text-muted-foreground">
              {translate('gathering-cancelled')}
            </Badge>
          )}
        </li>
      ))}
    </ul>
  )
}
